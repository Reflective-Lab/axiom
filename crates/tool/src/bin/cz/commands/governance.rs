// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: LicenseRef-Proprietary

//! Governance workflow commands: validate, digest, ack, escalate, assign.
//!
//! These implement the uniquely "Converge" parts of the workflow:
//! - Validation produces findings in .converge/findings/
//! - Acknowledgements, escalations, and assignments create audit artifacts
//!
//! File conventions:
//! - .converge/findings/<id>.json       - Validation findings
//! - .converge/acks/<finding-id>.yaml   - Acknowledgement records
//! - .converge/escalations/<id>.yaml    - Escalation records
//! - .converge/assignments/<id>.yaml    - Assignment records
//! - .converge/policy/                  - Cedar policies and schema

use crate::cli::{AckArgs, AssignArgs, DigestArgs, EscalateArgs, OutputFormat, ValidateArgs};
use crate::commands::{CmdError, CmdResult, find_workspace_root};
use chrono::Utc;
use colored::Colorize;
use converge_core::llm::LlmProvider;
use converge_provider::AnthropicProvider;
use converge_tool::{
    GherkinValidator, IssueCategory, ProviderBridge, Severity, StaticLlmProvider, ValidationConfig,
    ValidationIssue,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;

// =============================================================================
// Data Structures
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct Finding {
    id: String,
    file: String,
    location: String,
    category: String,
    severity: String,
    message: String,
    suggestion: Option<String>,
    created_at: String,
    status: String,
}

impl Finding {
    fn from_issue(issue: &ValidationIssue, file: &str, id: &str) -> Self {
        Self {
            id: id.to_string(),
            file: file.to_string(),
            location: issue.location.clone(),
            category: format!("{:?}", issue.category),
            severity: format!("{:?}", issue.severity),
            message: issue.message.clone(),
            suggestion: issue.suggestion.clone(),
            created_at: Utc::now().to_rfc3339(),
            status: "open".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AckRecord {
    finding_id: String,
    acknowledged_by: String,
    acknowledged_at: String,
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EscalationRecord {
    finding_id: String,
    escalated_by: String,
    escalated_to: Option<String>,
    escalated_at: String,
    note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AssignmentRecord {
    finding_id: String,
    assigned_by: String,
    assigned_to: String,
    assigned_at: String,
    note: Option<String>,
}

// =============================================================================
// Validate Command
// =============================================================================

/// Validate specs and produce findings.
pub async fn validate(args: ValidateArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    println!();
    println!("{}", "cz validate".bright_blue().bold());
    println!();

    // Ensure .converge directory structure exists
    ensure_converge_dirs(&root)?;

    // Find all .truths/.truth/.feature files
    let mut truth_files = Vec::new();
    for path in &args.paths {
        let full_path = if Path::new(path).is_absolute() {
            Path::new(path).to_path_buf()
        } else {
            root.join(path)
        };
        find_truth_files(&full_path, &mut truth_files);
    }

    if truth_files.is_empty() {
        println!(
            "  {} No .truths, .truth, or .feature files found",
            "⚠".yellow()
        );
        return Ok(());
    }

    println!("  Found {} spec files", truth_files.len());
    println!();

    // Create the validator
    let (provider, config) = create_validator_config(&args)?;
    let validator = GherkinValidator::new(provider, config);

    // Validate each file
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut all_findings: Vec<Finding> = Vec::new();

    for file in &truth_files {
        let relative = file.strip_prefix(&root).unwrap_or(file);
        let file_str = relative.display().to_string();

        match validator.validate_file(file) {
            Ok(result) => {
                let errors = result
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Error)
                    .count();
                let warnings = result
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Warning)
                    .count();

                total_errors += errors;
                total_warnings += warnings;

                // Print file result
                if result.is_valid {
                    println!(
                        "  {} {} ({} scenarios)",
                        "✓".green(),
                        file_str,
                        result.scenario_count
                    );
                } else {
                    println!(
                        "  {} {} ({} errors, {} warnings)",
                        "✗".red(),
                        file_str,
                        errors,
                        warnings
                    );
                }

                // Print issues
                for issue in &result.issues {
                    print_issue(issue);

                    // Create finding for persistence
                    if args.persist {
                        let finding_id = format!(
                            "{}-{}-{}",
                            sanitize_filename(&file_str),
                            sanitize_filename(&issue.location),
                            all_findings.len()
                        );
                        all_findings.push(Finding::from_issue(issue, &file_str, &finding_id));
                    }
                }
            }
            Err(e) => {
                println!("  {} {} - {}", "✗".red(), file_str, e);
                total_errors += 1;
            }
        }
    }

    // Persist findings
    if args.persist && !all_findings.is_empty() {
        let findings_dir = root.join(".converge").join("findings");
        for finding in &all_findings {
            let finding_file = findings_dir.join(format!("{}.json", finding.id));
            let json = serde_json::to_string_pretty(&finding)
                .map_err(|e| CmdError::new(format!("Failed to serialize finding: {e}")))?;
            fs::write(&finding_file, json)
                .map_err(|e| CmdError::new(format!("Failed to write finding: {e}")))?;
        }
        println!();
        println!(
            "  {} {} findings written to .converge/findings/",
            "→".dimmed(),
            all_findings.len()
        );
    }

    // Summary
    println!();
    println!("───────────────────────────────────────────────────────────");
    if total_errors > 0 {
        println!(
            "  {} {} errors, {} warnings",
            "✗".red(),
            total_errors.to_string().red(),
            total_warnings
        );
    } else if total_warnings > 0 {
        println!(
            "  {} {} warnings",
            "⚠".yellow(),
            total_warnings.to_string().yellow()
        );
    } else {
        println!("  {} All specs valid", "✓".green());
    }
    println!();

    if args.enforce {
        println!(
            "  {} Cedar policy enforcement requested but not yet implemented",
            "⚠".yellow()
        );
        println!();
    }

    if total_errors > 0 {
        Err(CmdError::new(format!("{total_errors} validation error(s)")))
    } else {
        Ok(())
    }
}

fn create_validator_config(
    args: &ValidateArgs,
) -> Result<(Arc<dyn LlmProvider>, ValidationConfig), CmdError> {
    // Determine what checks to run
    let wants_llm_checks =
        !args.conventions_only && (!args.skip_business_sense || !args.skip_compilability);

    let config = ValidationConfig {
        check_business_sense: !args.conventions_only && !args.skip_business_sense,
        check_compilability: !args.conventions_only && !args.skip_compilability,
        check_conventions: true,
        min_confidence: 0.7,
    };

    // If LLM checks are requested, check for API key
    if wants_llm_checks {
        if std::env::var("ANTHROPIC_API_KEY").is_err() {
            // No API key - fall back to conventions-only with a warning
            println!(
                "  {} ANTHROPIC_API_KEY not set, running conventions-only mode",
                "⚠".yellow()
            );
            println!(
                "  {} Set ANTHROPIC_API_KEY for business sense and compilability checks",
                "→".dimmed()
            );
            println!();

            let config = ValidationConfig {
                check_business_sense: false,
                check_compilability: false,
                check_conventions: true,
                min_confidence: 0.7,
            };
            let provider: Arc<dyn LlmProvider> = Arc::new(StaticLlmProvider::constant("VALID"));
            return Ok((provider, config));
        }

        let model = std::env::var("CONVERGE_ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-6".to_string());
        let anthropic = task::block_in_place(|| AnthropicProvider::from_env(model))
            .map_err(|e| CmdError::new(format!("Failed to initialize Anthropic provider: {e}")))?;
        let provider: Arc<dyn LlmProvider> = Arc::new(ProviderBridge::new(anthropic));
        return Ok((provider, config));
    }

    // Conventions-only mode
    let provider: Arc<dyn LlmProvider> = Arc::new(StaticLlmProvider::constant("VALID"));
    Ok((provider, config))
}

fn print_issue(issue: &ValidationIssue) {
    let (icon, color) = match issue.severity {
        Severity::Error => ("✗", "red"),
        Severity::Warning => ("⚠", "yellow"),
        Severity::Info => ("ℹ", "blue"),
    };

    let icon_str = match color {
        "red" => format!("    {icon} ").red().to_string(),
        "yellow" => format!("    {icon} ").yellow().to_string(),
        "blue" => format!("    {icon} ").blue().to_string(),
        _ => format!("    {icon} "),
    };

    let category = match issue.category {
        IssueCategory::BusinessSense => "[business]",
        IssueCategory::Compilability => "[compile]",
        IssueCategory::Convention => "[convention]",
        IssueCategory::Syntax => "[syntax]",
        IssueCategory::NotRelatedError => "[error]",
    };

    println!(
        "{}{} {} {}",
        icon_str,
        category.dimmed(),
        issue.location.cyan(),
        issue.message
    );

    if let Some(suggestion) = &issue.suggestion {
        println!("      {} {}", "→".dimmed(), suggestion.dimmed());
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .to_lowercase()
}

// =============================================================================
// Digest Command
// =============================================================================

/// Produce "what needs attention" summary.
pub async fn digest(args: DigestArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    println!();
    println!("{}", "cz digest".bright_blue().bold());
    println!();

    let converge_dir = root.join(".converge");
    if !converge_dir.exists() {
        println!("  {} No .converge directory found", "⚠".yellow());
        println!("  {} Run 'cz validate' first", "→".dimmed());
        return Ok(());
    }

    // Load and count findings
    let findings_dir = converge_dir.join("findings");
    let acks_dir = converge_dir.join("acks");
    let escalations_dir = converge_dir.join("escalations");
    let assignments_dir = converge_dir.join("assignments");

    let findings = load_findings(&findings_dir);
    let acks = count_files(&acks_dir, "yaml");
    let escalations = count_files(&escalations_dir, "yaml");
    let assignments = count_files(&assignments_dir, "yaml");

    let open_findings: Vec<_> = findings
        .iter()
        .filter(|f| !acks_dir.join(format!("{}.yaml", f.id)).exists())
        .collect();

    let errors: Vec<_> = open_findings
        .iter()
        .filter(|f| f.severity == "Error")
        .collect();
    let warnings: Vec<_> = open_findings
        .iter()
        .filter(|f| f.severity == "Warning")
        .collect();

    match args.format {
        OutputFormat::Pretty => {
            println!("  Open findings:");
            println!(
                "    {} errors",
                if errors.is_empty() {
                    "0".green().to_string()
                } else {
                    errors.len().to_string().red().to_string()
                }
            );
            println!(
                "    {} warnings",
                if warnings.is_empty() {
                    "0".green().to_string()
                } else {
                    warnings.len().to_string().yellow().to_string()
                }
            );
            println!();
            println!("  Acknowledged: {}", acks.to_string().green());
            println!("  Escalated:    {}", escalations.to_string().yellow());
            println!("  Assigned:     {}", assignments.to_string().cyan());

            if !errors.is_empty() {
                println!();
                println!("  {} Open errors:", "Errors:".red().bold());
                for finding in errors.iter().take(5) {
                    println!(
                        "    {} {} - {}",
                        finding.id.cyan(),
                        finding.file.dimmed(),
                        finding.message
                    );
                }
                if errors.len() > 5 {
                    println!("    ... and {} more", errors.len() - 5);
                }
            }
        }
        OutputFormat::Json => {
            let summary = serde_json::json!({
                "open_errors": errors.len(),
                "open_warnings": warnings.len(),
                "acknowledged": acks,
                "escalated": escalations,
                "assigned": assignments,
                "findings": open_findings,
            });
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        }
        OutputFormat::Quiet => {}
    }

    println!();
    Ok(())
}

fn load_findings(dir: &Path) -> Vec<Finding> {
    if !dir.exists() {
        return Vec::new();
    }

    let mut findings = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .map(|e| e == "json")
                .unwrap_or(false)
            {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(finding) = serde_json::from_str::<Finding>(&content) {
                        findings.push(finding);
                    }
                }
            }
        }
    }
    findings
}

// =============================================================================
// Ack, Escalate, Assign Commands
// =============================================================================

/// Acknowledge a finding.
pub async fn ack(args: AckArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    ensure_converge_dirs(&root)?;

    // Verify finding exists
    let findings_dir = root.join(".converge").join("findings");
    let finding_file = findings_dir.join(format!("{}.json", args.finding_id));
    if !finding_file.exists() {
        return Err(CmdError::new(format!(
            "Finding not found: {}",
            args.finding_id
        )));
    }

    let record = AckRecord {
        finding_id: args.finding_id.clone(),
        acknowledged_by: get_current_user(),
        acknowledged_at: Utc::now().to_rfc3339(),
        note: args.note,
    };

    let acks_dir = root.join(".converge").join("acks");
    let ack_file = acks_dir.join(format!("{}.yaml", args.finding_id));

    let yaml = serde_yaml::to_string(&record)
        .map_err(|e| CmdError::new(format!("Failed to serialize: {e}")))?;

    fs::write(&ack_file, yaml)
        .map_err(|e| CmdError::new(format!("Failed to write ack file: {e}")))?;

    println!();
    println!(
        "{} Acknowledged finding: {}",
        "✓".green(),
        args.finding_id.cyan()
    );
    println!("  {} {}", "File:".dimmed(), ack_file.display());
    println!();

    Ok(())
}

/// Escalate a finding.
pub async fn escalate(args: EscalateArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    ensure_converge_dirs(&root)?;

    let record = EscalationRecord {
        finding_id: args.finding_id.clone(),
        escalated_by: get_current_user(),
        escalated_to: args.to,
        escalated_at: Utc::now().to_rfc3339(),
        note: args.note,
    };

    let escalations_dir = root.join(".converge").join("escalations");
    let escalation_file = escalations_dir.join(format!("{}.yaml", args.finding_id));

    let yaml = serde_yaml::to_string(&record)
        .map_err(|e| CmdError::new(format!("Failed to serialize: {e}")))?;

    fs::write(&escalation_file, yaml)
        .map_err(|e| CmdError::new(format!("Failed to write escalation file: {e}")))?;

    println!();
    println!(
        "{} Escalated finding: {}",
        "⚠".yellow(),
        args.finding_id.cyan()
    );
    println!("  {} {}", "File:".dimmed(), escalation_file.display());
    println!();

    Ok(())
}

/// Assign a finding to an owner.
pub async fn assign(args: AssignArgs) -> CmdResult {
    let root =
        find_workspace_root().ok_or_else(|| CmdError::new("Could not find workspace root"))?;

    ensure_converge_dirs(&root)?;

    let record = AssignmentRecord {
        finding_id: args.finding_id.clone(),
        assigned_by: get_current_user(),
        assigned_to: args.owner.clone(),
        assigned_at: Utc::now().to_rfc3339(),
        note: args.note,
    };

    let assignments_dir = root.join(".converge").join("assignments");
    let assignment_file = assignments_dir.join(format!("{}.yaml", args.finding_id));

    let yaml = serde_yaml::to_string(&record)
        .map_err(|e| CmdError::new(format!("Failed to serialize: {e}")))?;

    fs::write(&assignment_file, yaml)
        .map_err(|e| CmdError::new(format!("Failed to write assignment file: {e}")))?;

    println!();
    println!(
        "{} Assigned finding {} to {}",
        "→".cyan(),
        args.finding_id.cyan(),
        args.owner.green()
    );
    println!("  {} {}", "File:".dimmed(), assignment_file.display());
    println!();

    Ok(())
}

// =============================================================================
// Helpers
// =============================================================================

fn ensure_converge_dirs(root: &Path) -> CmdResult {
    let converge_dir = root.join(".converge");
    let dirs = [
        converge_dir.join("findings"),
        converge_dir.join("acks"),
        converge_dir.join("escalations"),
        converge_dir.join("assignments"),
        converge_dir.join("policy"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)
            .map_err(|e| CmdError::new(format!("Failed to create {}: {e}", dir.display())))?;
    }

    Ok(())
}

fn find_truth_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_file() {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext == "truths" || ext == "truth" || ext == "feature" {
                files.push(path.to_path_buf());
            }
        }
    } else if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                // Skip hidden directories and common non-source directories
                if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }
                }
                find_truth_files(&entry_path, files);
            }
        }
    }
}

fn count_files(dir: &Path, extension: &str) -> usize {
    if !dir.exists() {
        return 0;
    }

    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == extension)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn get_current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
