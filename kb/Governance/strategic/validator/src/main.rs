//! Strategy Gate Fixture Validator
//!
//! Validates YAML fixtures against JSON schemas for strategy gates.
//!
//! Usage:
//!   cargo run --release
//!   cargo run --release -- --verbose

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use colored::Colorize;
use jsonschema::JSONSchema;
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Fields used by main binary but not all accessed in lib tests
pub struct LensPacksConfig {
    pub version: String,
    pub gates: std::collections::HashMap<String, GateConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Fields used by main binary but not all accessed in lib tests
#[allow(clippy::struct_field_names)] // Voices postfix is intentional and clear
pub struct GateConfig {
    pub primary_voices: Vec<String>,
    pub secondary_voices: Vec<String>,
    pub counter_voices: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(name = "strategy-validator")]
#[command(about = "Validate strategy gate fixtures and detect drift")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate fixtures against schemas (existing behavior)
    Validate {
        /// Show detailed error information
        #[arg(short, long)]
        verbose: bool,
        /// Path to schemas directory (default: ../schemas)
        #[arg(long)]
        schemas: Option<PathBuf>,
        /// Path to fixtures directory (default: ../fixtures)
        #[arg(long)]
        fixtures: Option<PathBuf>,
        /// Path to config directory (default: ../config)
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Detect drift and generate report
    Detect {
        /// Output directory for drift reports
        #[arg(short, long, default_value = "strategic/reports/drift")]
        output_dir: PathBuf,
        /// Week identifier (YYYY-WW format), defaults to current
        #[arg(long)]
        week: Option<String>,
        /// Path to fixtures directory (default: ../fixtures)
        #[arg(long)]
        fixtures: Option<PathBuf>,
        /// Path to config directory (default: ../config)
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Generate weekly digest report
    Digest {
        /// Path to fixtures directory
        #[arg(long, default_value = "strategic/fixtures")]
        fixtures_dir: PathBuf,
        /// Output directory for digest reports
        #[arg(long, default_value = "reports/digest")]
        output_dir: PathBuf,
        /// Week to generate digest for (YYYY-WW format)
        #[arg(long)]
        week: Option<String>,
        /// Path to threshold configuration file (TOML)
        #[arg(long)]
        config: Option<PathBuf>,
        /// Deliver digest via email after generation
        #[arg(long)]
        deliver: bool,
        /// Recipient email addresses (comma-separated)
        #[arg(long, value_delimiter = ',', requires = "deliver")]
        recipients: Vec<String>,
        /// Path to delivery log database
        #[arg(long, default_value = "reports/delivery.db")]
        delivery_db: PathBuf,
        /// Path to delivery policy configuration (TOML)
        #[arg(long)]
        delivery_policy: Option<PathBuf>,
        /// Path to acks directory for SLA breach detection
        #[arg(long, default_value = "reports/acks")]
        acks_dir: PathBuf,
    },
    /// Acknowledge a governance finding
    Ack {
        /// Finding ID to acknowledge (e.g., "content-publish-2026-W05-001")
        finding_id: String,
        /// Required notes explaining the acknowledgment
        #[arg(long, required = true)]
        notes: String,
        /// Output directory for ack files
        #[arg(long, default_value = "reports/acks")]
        output_dir: PathBuf,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Escalate a governance finding
    Escalate {
        /// Finding ID to escalate
        finding_id: String,
        /// Required notes explaining why escalation is needed
        #[arg(long, required = true)]
        notes: String,
        /// Output directory for escalation files
        #[arg(long, default_value = "reports/escalations")]
        output_dir: PathBuf,
        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Assign a finding to an owner (documentation only for v1)
    Assign {
        /// Finding ID to assign
        finding_id: String,
        /// Owner to assign (email or identifier)
        owner: String,
    },
    /// Query acknowledgment audit trail
    Audit {
        /// Filter by finding ID (exact match)
        #[arg(long)]
        finding: Option<String>,
        /// Filter by gate ID (prefix match)
        #[arg(long)]
        gate: Option<String>,
        /// Filter by week (YYYY-WW format)
        #[arg(long)]
        week: Option<String>,
        /// Path to acks directory
        #[arg(long, default_value = "reports/acks")]
        acks_dir: PathBuf,
    },
}

#[derive(Debug)]
#[allow(dead_code)] // All variants used by main binary
enum FixtureType {
    GateExecution,
    Acknowledgment,
    Override,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // All variants used by main binary
pub enum SemanticError {
    MissingPrimary { gate_id: String, missing_voice: String },
    AckMissing { fixture_path: String },
    AckNotReadable { ack_path: String, reason: String },
    CounterVoiceMismatch { ack_path: String, actual: String, valid: Vec<String> },
    AckWarningNotInResults { ack_path: String, check_id: String },
    StopNotClassA { fixture_path: String },
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPrimary { gate_id, missing_voice } =>
                write!(f, "E_VAL01_MISSING_PRIMARY: gate '{gate_id}' missing primary voice '{missing_voice}'"),
            Self::AckMissing { fixture_path } =>
                write!(f, "E_VAL02_ACK_MISSING: WARN outcome requires acknowledgment_ref in '{fixture_path}'"),
            Self::AckNotReadable { ack_path, reason } =>
                write!(f, "E_VAL02_ACK_MISSING: acknowledgment file not readable '{ack_path}': {reason}"),
            Self::CounterVoiceMismatch { ack_path, actual, valid } =>
                write!(f, "E_VAL02_COUNTER_VOICE_MISMATCH: '{ack_path}' consulted '{actual}', valid: {valid:?}"),
            Self::AckWarningNotInResults { ack_path, check_id } =>
                write!(f, "E_VAL02_ACK_MISSING: ack '{ack_path}' references '{check_id}' not found in lens_results"),
            Self::StopNotClassA { fixture_path } =>
                write!(f, "E_VAL03_STOP_NOT_CLASS_A: STOP outcome requires at least one Class A STOP finding in '{fixture_path}'"),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct GateExecution {
    gate_id: String,
    lens_results: Vec<LensResult>,
    decision: Decision,
    acknowledgment_ref: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // All fields used by main binary
struct LensResult {
    check_id: String,
    voice: String,
    class: String,
    severity: String,
}

#[derive(Debug, serde::Deserialize)]
struct Decision {
    outcome: String,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Used by main binary
struct Acknowledgment {
    warnings_accepted: Vec<WarningAccepted>,
    counter_voice_consulted: CounterVoice,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Used by main binary
struct WarningAccepted {
    check_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // Used by main binary
struct CounterVoice {
    voice: String,
}

#[allow(dead_code)] // Used by main binary
struct ValidationResult {
    path: PathBuf,
    fixture_type: FixtureType,
    success: bool,
    errors: Vec<String>,
    semantic_errors: Vec<SemanticError>,
}

#[allow(dead_code)] // Used by main binary
pub fn load_lens_packs(config_dir: &Path) -> LensPacksConfig {
    let config_path = config_dir.join("lens_packs.yaml");
    let content = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!("{} Lens packs config not found: {}", "Error:".red(), config_path.display());
        std::process::exit(1);
    });
    serde_yaml::from_str(&content).unwrap_or_else(|e| {
        eprintln!("{} Invalid lens packs config: {}", "Error:".red(), e);
        std::process::exit(1);
    })
}

#[allow(dead_code)] // Binary entry point
fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Backward compatibility: no subcommand defaults to validate
            handle_validate(false, None, None, None);
        }
        Some(Commands::Validate {
            verbose,
            schemas,
            fixtures,
            config,
        }) => {
            handle_validate(verbose, schemas, fixtures, config);
        }
        Some(Commands::Detect {
            output_dir,
            week,
            fixtures,
            config,
        }) => {
            handle_detect(&output_dir, week, fixtures, config);
        }
        Some(Commands::Digest {
            fixtures_dir,
            output_dir,
            week,
            config,
            deliver,
            recipients,
            delivery_db,
            delivery_policy,
            acks_dir,
        }) => {
            handle_digest(&fixtures_dir, &output_dir, week.as_ref(), config.as_ref(), deliver, recipients, &delivery_db, delivery_policy.as_ref(), &acks_dir);
        }
        Some(Commands::Ack {
            finding_id,
            notes,
            output_dir,
            yes,
        }) => {
            handle_ack(&finding_id, &notes, &output_dir, yes);
        }
        Some(Commands::Escalate {
            finding_id,
            notes,
            output_dir,
            yes,
        }) => {
            handle_escalate(&finding_id, &notes, &output_dir, yes);
        }
        Some(Commands::Assign {
            finding_id,
            owner,
        }) => {
            handle_assign(&finding_id, &owner);
        }
        Some(Commands::Audit {
            finding,
            gate,
            week,
            acks_dir,
        }) => {
            handle_audit(finding, gate, week, &acks_dir);
        }
    }
}

/// Handle validate subcommand
fn handle_validate(
    verbose: bool,
    schemas: Option<PathBuf>,
    fixtures: Option<PathBuf>,
    config: Option<PathBuf>,
) {
    // Resolve paths relative to binary or use provided paths
    let base_dir = std::env::current_dir().unwrap();
    let schemas_dir = schemas.unwrap_or_else(|| base_dir.join("../schemas"));
    let fixtures_dir = fixtures.unwrap_or_else(|| base_dir.join("../fixtures"));
    let config_dir = config.unwrap_or_else(|| base_dir.join("../config"));

    // Normalize paths
    let schemas_dir = fs::canonicalize(&schemas_dir).unwrap_or_else(|_| {
        eprintln!("{} Schemas directory not found: {}", "Error:".red(), schemas_dir.display());
        std::process::exit(1);
    });
    let fixtures_dir = fs::canonicalize(&fixtures_dir).unwrap_or_else(|_| {
        eprintln!("{} Fixtures directory not found: {}", "Error:".red(), fixtures_dir.display());
        std::process::exit(1);
    });
    let config_dir = fs::canonicalize(&config_dir).unwrap_or_else(|_| {
        eprintln!("{} Config directory not found: {}", "Error:".red(), config_dir.display());
        std::process::exit(1);
    });

    println!("{}", "━".repeat(55));
    println!(" Strategy Gate Fixture Validation");
    println!("{}", "━".repeat(55));
    println!();

    // Load schemas
    let gate_execution_schema = load_schema(&schemas_dir, "strategy_gate_execution");
    let ack_schema = load_schema(&schemas_dir, "strategy_ack");
    let override_schema = load_schema(&schemas_dir, "strategy_override");

    // Load lens packs configuration
    let lens_packs = load_lens_packs(&config_dir);

    if verbose {
        println!("Loaded lens packs config (version: {})", lens_packs.version);
        println!("Gates configured: {}", lens_packs.gates.len());
        println!();
    }

    let mut results: Vec<ValidationResult> = Vec::new();

    // Walk fixtures directory
    for entry in WalkDir::new(&fixtures_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml" || ext == "yml"))
    {
        let path = entry.path();
        let relative_path = path.strip_prefix(&fixtures_dir).unwrap_or(path);

        let (fixture_type, _schema_name) = classify_fixture(relative_path);
        let compiled_schema = match fixture_type {
            FixtureType::GateExecution => &gate_execution_schema,
            FixtureType::Acknowledgment => &ack_schema,
            FixtureType::Override => &override_schema,
        };

        let result = validate_fixture(path, compiled_schema, fixture_type, &fixtures_dir, &lens_packs);
        results.push(result);
    }

    // Group and print results
    print_results(&results, &fixtures_dir, verbose);

    // Exit with appropriate code
    let fail_count = results.iter().filter(|r| !r.success).count();
    if fail_count > 0 {
        std::process::exit(1);
    }
}

/// Handle detect subcommand
fn handle_detect(
    output_dir: &Path,
    week: Option<String>,
    fixtures: Option<PathBuf>,
    config: Option<PathBuf>,
) {
    use strategy_validator::{detector, DetectorError, DetectorExitCode};

    // Resolve paths relative to binary or use provided paths
    let base_dir = std::env::current_dir().unwrap();
    let fixtures_dir = fixtures.unwrap_or_else(|| base_dir.join("../fixtures"));
    let config_dir = config.unwrap_or_else(|| base_dir.join("../config"));

    // Normalize paths
    let fixtures_dir = fs::canonicalize(&fixtures_dir).unwrap_or_else(|_| {
        eprintln!("{} Fixtures directory not found: {}", "Error:".red(), fixtures_dir.display());
        std::process::exit(DetectorExitCode::ConfigError as i32);
    });
    let config_dir = fs::canonicalize(&config_dir).unwrap_or_else(|_| {
        eprintln!("{} Config directory not found: {}", "Error:".red(), config_dir.display());
        std::process::exit(DetectorExitCode::ConfigError as i32);
    });

    println!("{}", "━".repeat(55));
    println!(" Drift Detection");
    println!("{}", "━".repeat(55));
    println!();

    // Run detector
    match detector::run_detector(&fixtures_dir, &config_dir, output_dir, week) {
        Ok(report) => {
            println!("{} Drift report generated for week {}", "Success:".green(), report.week);
            println!("  Total findings: {}", report.summary.total);
            println!("  Output: {}", output_dir.display());

            // Exit with appropriate code based on findings
            let exit_code = if report.summary.total > 0 {
                DetectorExitCode::DriftDetected
            } else {
                DetectorExitCode::Success
            };
            std::process::exit(exit_code as i32);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);

            // Determine exit code based on error type
            let exit_code = match e {
                DetectorError::Config(_) => DetectorExitCode::ConfigError,
                DetectorError::Io(_) | DetectorError::Git(_) | DetectorError::Serialization(_) => DetectorExitCode::IoError,
            };
            std::process::exit(exit_code as i32);
        }
    }
}

/// Validate SMTP environment variables
fn validate_smtp_env() -> Result<Vec<String>, Vec<String>> {
    let required_vars = ["SMTP_HOST", "SMTP_USERNAME", "SMTP_PASSWORD", "SMTP_FROM_ADDRESS"];
    let missing: Vec<String> = required_vars
        .iter()
        .filter(|var| std::env::var(var).is_err())
        .map(|s| (*s).to_string())
        .collect();

    if missing.is_empty() {
        Ok(required_vars.iter().map(|s| (*s).to_string()).collect())
    } else {
        Err(missing)
    }
}

/// Handle digest subcommand
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
fn handle_digest(
    fixtures_dir: &Path,
    output_dir: &Path,
    week: Option<&String>,
    config: Option<&PathBuf>,
    deliver: bool,
    recipients: Vec<String>,
    delivery_db: &Path,
    delivery_policy_path: Option<&PathBuf>,
    acks_dir: &Path,
) {
    use strategy_validator::{digest, thresholds, routing, routing_config, idempotency, slack, sla};
    use strategy_validator::reliability::{RetryConfig, CircuitBreakerConfig};
    use chrono::Timelike;

    // If deliver is true, validate SMTP env vars before generation
    if deliver {
        if let Err(missing) = validate_smtp_env() {
            eprintln!("{} Missing required SMTP environment variables:", "Error:".red());
            for var in &missing {
                eprintln!("  - {var}");
            }
            eprintln!();
            eprintln!("Configure SMTP settings:");
            eprintln!("  SMTP_HOST         - Your SMTP provider (e.g., smtp.gmail.com)");
            eprintln!("  SMTP_USERNAME     - SMTP username");
            eprintln!("  SMTP_PASSWORD     - SMTP password or App Password");
            eprintln!("  SMTP_FROM_ADDRESS - Sender email address");
            eprintln!();
            eprintln!("Optional:");
            eprintln!("  SMTP_PORT         - Port (default: 587)");
            eprintln!("  SMTP_FROM_NAME    - Sender name (default: \"Strategy Validator\")");
            std::process::exit(2);
        }
    }

    // Normalize fixtures path
    let fixtures_dir = std::fs::canonicalize(fixtures_dir).unwrap_or_else(|_| {
        eprintln!("{} Fixtures directory not found: {}", "Error:".red(), fixtures_dir.display());
        std::process::exit(2);
    });

    // Load threshold configuration
    let threshold_config = thresholds::load_thresholds(config.map(std::ops::Deref::deref));
    let active_thresholds = thresholds::get_active_profile(&threshold_config);

    println!("{}", "━".repeat(55));
    println!(" Weekly Digest Generation");
    println!("{}", "━".repeat(55));
    println!();
    println!("Threshold profile: {}", threshold_config.active_profile);
    println!();

    // Run digest
    match digest::run_digest(&fixtures_dir, output_dir, week.cloned(), active_thresholds, &threshold_config.red_flags) {
        Ok(report) => {
            println!("{} Weekly digest generated for week {}", "Success:".green(), report.week);
            println!("  Total executions: {}", report.summary.total_executions);
            println!("  PASS: {}, WARN: {}, STOP: {}",
                report.summary.pass_count,
                report.summary.warn_count,
                report.summary.stop_count);
            println!("  Pressure signatures: {}", report.pressure_signatures.len());
            println!("  Red flags: {}", report.red_flags.len());

            // Detect SLA breaches
            let sla_breaches = sla::detect_sla_breaches(&fixtures_dir, acks_dir);
            if sla_breaches.is_empty() {
                println!("  SLA breaches: 0");
            } else {
                println!("  {} SLA breaches: {}", "⚠".yellow(), sla_breaches.len());
            }

            println!("  Output: {}", output_dir.display());

            // Append SLA breaches to the digest markdown file
            if !sla_breaches.is_empty() {
                let digest_file = output_dir.join(format!("{}.md", report.week));
                if let Ok(mut content) = std::fs::read_to_string(&digest_file) {
                    content.push_str("\n## SLA Breaches\n\n");
                    content.push_str(&sla::format_sla_breaches(&sla_breaches));
                    let _ = std::fs::write(&digest_file, content);
                }
            }

            // If deliver flag is set, send emails
            if deliver {
                println!();
                println!("{}", "━".repeat(55));
                println!(" Email Delivery");
                println!("{}", "━".repeat(55));
                println!();

                // Load delivery policy if provided
                let delivery_policy = if let Some(policy_path) = delivery_policy_path {
                    match routing_config::load_delivery_policy(policy_path) {
                        Ok(policy) => {
                            println!("Loaded delivery policy: {}", policy_path.display());
                            println!("  Routing rules: {}", policy.routing_rules.len());
                            println!("  Rate limit: {} alerts/hour", policy.rate_limiting.max_alerts_per_hour);
                            if let Some(ref qh) = policy.quiet_hours {
                                println!("  Quiet hours: {:02}:00-{:02}:00 ({})", qh.start_hour, qh.end_hour, qh.timezone);
                            }
                            println!();
                            Some(policy)
                        }
                        Err(e) => {
                            eprintln!("{} Failed to load delivery policy: {}", "Error:".red(), e);
                            std::process::exit(2);
                        }
                    }
                } else {
                    None
                };

                // Create alert throttler
                let throttler = if let Some(ref policy) = delivery_policy {
                    routing::AlertThrottler::new(policy.rate_limiting.max_alerts_per_hour)
                } else {
                    routing::AlertThrottler::new(10) // Default rate limit
                };

                // Check quiet hours before delivery
                #[allow(clippy::cast_possible_truncation)]
                let current_hour = chrono::Local::now().hour() as u8; // hour() returns 0-23, safe to cast
                if let Some(ref policy) = delivery_policy {
                    if let Some(ref qh) = policy.quiet_hours {
                        if routing::is_quiet_hours(current_hour, qh.start_hour, qh.end_hour) {
                            println!("{} Currently in quiet hours ({:02}:00-{:02}:00)", "⏸".yellow(), qh.start_hour, qh.end_hour);
                            println!("Delivery suppressed for non-critical alerts");
                            println!();
                        }
                    }
                }

                // Get recipients list
                let mut recipient_list: Vec<String> = if recipients.is_empty() {
                    // Try to read from SMTP_RECIPIENTS env var
                    if let Ok(env_recipients) = std::env::var("SMTP_RECIPIENTS") {
                        env_recipients.split(',').map(|s| s.trim().to_string()).collect()
                    } else {
                        eprintln!("{} No recipients specified. Use --recipients or set SMTP_RECIPIENTS environment variable.", "Error:".red());
                        std::process::exit(2);
                    }
                } else {
                    recipients
                };

                // Add owner to recipients if routing policy provides one
                if let Some(ref policy) = delivery_policy {
                    // For digest, we use the first gate in the report to determine owner
                    // (In practice, a digest may cover multiple gates - owner is informational)
                    for gate_id in report.by_gate.keys() {
                        if let Some(owner) = routing::find_owner(policy, gate_id) {
                            if !recipient_list.contains(&owner) {
                                println!("  Owner routing: adding {owner} for gate {gate_id}");
                                recipient_list.push(owner);
                            }
                            break; // Only add one owner per digest
                        }
                    }
                }

                // Create delivery logger
                let logger = match strategy_validator::delivery::DeliveryLogger::new(delivery_db.to_str().unwrap()) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("{} Failed to create delivery logger: {}", "Error:".red(), e);
                        std::process::exit(2);
                    }
                };

                // Create runtime for async operations
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{} Failed to create async runtime: {}", "Error:".red(), e);
                        std::process::exit(2);
                    }
                };

                // Send emails
                let mut success_count = 0;
                let mut failure_count = 0;
                let mut rate_limited_count = 0;
                let mut quiet_hours_count = 0;
                let mut duplicate_count = 0;

                for recipient in &recipient_list {
                    // Determine severity for this digest (use STOP if any STOP findings)
                    let severity = if report.summary.stop_count > 0 {
                        "STOP"
                    } else if report.summary.warn_count > 0 {
                        "WARN"
                    } else {
                        "PASS"
                    };

                    // Generate idempotency key
                    let idempotency_key = idempotency::generate_idempotency_key(
                        "digest",
                        "weekly",
                        severity,
                        &report.week,
                        recipient,
                    );

                    // Check for duplicate
                    if logger.has_idempotency_key(&idempotency_key).unwrap_or(false) {
                        println!("{} Skipping {} (duplicate)", "⊘".yellow(), recipient);
                        duplicate_count += 1;
                        continue;
                    }

                    // Check rate limit
                    if !throttler.check_recipient(recipient) {
                        println!("{} Skipping {} (rate limited)", "⊘".yellow(), recipient);
                        rate_limited_count += 1;
                        continue;
                    }

                    // Check quiet hours (skip for critical)
                    if let Some(ref policy) = delivery_policy {
                        if let Some(ref qh) = policy.quiet_hours {
                            if severity != "STOP" && routing::is_quiet_hours(current_hour, qh.start_hour, qh.end_hour) {
                                println!("{} Skipping {} (quiet hours)", "⏸".yellow(), recipient);
                                quiet_hours_count += 1;
                                continue;
                            }
                        }
                    }

                    let result = rt.block_on(send_digest_to_recipient(
                        recipient,
                        &report,
                        &logger,
                        Some(&idempotency_key),
                    ));

                    match result {
                        Ok(message_id) => {
                            println!("{} Sent to {} (message_id: {})", "✓".green(), recipient, message_id);
                            success_count += 1;
                        }
                        Err(e) => {
                            eprintln!("{} Failed to send to {}: {}", "✗".red(), recipient, e);
                            failure_count += 1;
                        }
                    }
                }

                println!();
                println!("Delivery summary:");
                println!("  Delivered: {success_count}");
                println!("  Failed: {failure_count}");
                if duplicate_count > 0 {
                    println!("  Duplicates: {duplicate_count}");
                }
                if rate_limited_count > 0 {
                    println!("  Rate limited: {rate_limited_count}");
                }
                if quiet_hours_count > 0 {
                    println!("  Quiet hours: {quiet_hours_count}");
                }

                // Send Slack alerts for WARN/STOP findings
                let slack_config_result = slack::SlackConfig::from_env();
                if let Ok(slack_config) = slack_config_result {
                    if let Some(ref policy) = delivery_policy {
                        println!();
                        println!("{}", "━".repeat(55));
                        println!(" Slack Alert Delivery");
                        println!("{}", "━".repeat(55));
                        println!();

                        let mut slack_success = 0;
                        let mut slack_failure = 0;
                        let mut slack_skipped = 0;

                        // Determine overall severity
                        let severity = if report.summary.stop_count > 0 {
                            "STOP"
                        } else if report.summary.warn_count > 0 {
                            "WARN"
                        } else {
                            "PASS"
                        };

                        // Only send Slack for WARN/STOP
                        if severity == "WARN" || severity == "STOP" {
                            // Find Slack channel from routing
                            // Use first gate_id from report for routing lookup
                            if let Some(gate_id) = report.by_gate.keys().next() {
                                if let Some(channel) = routing::find_slack_channel(policy, gate_id, Some(severity), None) {
                                    // Generate idempotency key for Slack
                                    let slack_idemp_key = idempotency::generate_idempotency_key(
                                        gate_id,
                                        "digest_summary",
                                        severity,
                                        &report.week,
                                        &format!("slack:{channel}"),
                                    );

                                    // Create ReliableSlackSender
                                    match slack::ReliableSlackSender::new(
                                        slack_config.clone(),
                                        delivery_db.to_str().unwrap(),
                                        RetryConfig::default(),
                                        CircuitBreakerConfig::default(),
                                    ) {
                                        Ok(sender) => {
                                            // Build finding details from digest
                                            let finding_details = format!(
                                                "Week {} Digest Summary:\n- PASS: {}\n- WARN: {}\n- STOP: {}\n- Red flags: {}",
                                                report.week,
                                                report.summary.pass_count,
                                                report.summary.warn_count,
                                                report.summary.stop_count,
                                                report.red_flags.len()
                                            );

                                            let finding_id = format!("digest-{}-{}", report.week, gate_id);

                                            let send_result = rt.block_on(sender.send(
                                                &channel,
                                                gate_id,
                                                "digest_summary",
                                                severity,
                                                &finding_details,
                                                &finding_id,
                                                &report.week,
                                                &slack_idemp_key,
                                            ));

                                            match send_result {
                                                Ok(()) => {
                                                    println!("{} Sent Slack alert to {}", "✓".green(), channel);
                                                    slack_success += 1;
                                                }
                                                Err(e) => {
                                                    eprintln!("{} Slack alert failed: {}", "✗".red(), e);
                                                    slack_failure += 1;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("{} Failed to create Slack sender: {}", "✗".red(), e);
                                            slack_failure += 1;
                                        }
                                    }
                                } else {
                                    println!("{} No Slack channel configured for gate {}", "⊘".yellow(), gate_id);
                                    slack_skipped += 1;
                                }
                            }
                        } else {
                            println!("Skipping Slack (severity is PASS)");
                            slack_skipped += 1;
                        }

                        println!();
                        println!("Slack delivery summary:");
                        println!("  Delivered: {slack_success}");
                        println!("  Failed: {slack_failure}");
                        println!("  Skipped: {slack_skipped}");
                    }
                }

                if failure_count > 0 {
                    std::process::exit(1);
                }
            }

            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}

/// Send digest email to a single recipient
async fn send_digest_to_recipient(
    recipient: &str,
    report: &strategy_validator::digest::DigestReport,
    logger: &strategy_validator::delivery::DeliveryLogger,
    idempotency_key: Option<&str>,
) -> Result<String, strategy_validator::delivery::DeliveryError> {
    use strategy_validator::delivery;

    // Load SMTP config
    let config = delivery::SmtpConfig::from_env()?;

    // Create subject line
    let subject = format!("Weekly Digest: Week {}", report.week);

    // Get markdown body
    let body = report.to_markdown();

    // Send email
    match delivery::send_digest_email(&config, recipient, &subject, &body).await {
        Ok(message_id) => {
            // Log success
            let _ = logger.log_success(recipient, &subject, &message_id, &report.week, idempotency_key);
            Ok(message_id)
        }
        Err(e) => {
            // Log failure
            let _ = logger.log_failure(recipient, &subject, &e, &report.week, idempotency_key);
            Err(e)
        }
    }
}

/// Handle ack subcommand - acknowledge a governance finding
fn handle_ack(finding_id: &str, notes: &str, output_dir: &Path, yes: bool) {
    use std::io::{self, Write};
    use strategy_validator::ack;

    println!("Acknowledging finding: {finding_id}");
    println!();

    // Prompt for confirmation unless -y flag is set
    if !yes {
        print!("Create acknowledgment file? [Y/n]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input == "n" || input == "no" {
            println!("Aborted");
            std::process::exit(0);
        }
    }

    match ack::create_ack_file(finding_id, notes, Some(output_dir), false) {
        Ok(path) => {
            println!("{} Acknowledged: {}", "✓".green(), path.display());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{} Failed to create acknowledgment: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}

/// Handle escalate subcommand - escalate a governance finding
fn handle_escalate(finding_id: &str, notes: &str, output_dir: &Path, yes: bool) {
    use std::io::{self, Write};
    use strategy_validator::ack;

    println!("Escalating finding: {finding_id}");
    println!();

    // Prompt for confirmation unless -y flag is set
    if !yes {
        print!("Create escalation file? [Y/n]: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input == "n" || input == "no" {
            println!("Aborted");
            std::process::exit(0);
        }
    }

    match ack::create_escalation_file(finding_id, notes, Some(output_dir)) {
        Ok(path) => {
            println!("{} Escalated: {}", "✓".green(), path.display());
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{} Failed to create escalation: {}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}

/// Handle assign subcommand - assign a finding to an owner (documentation only for v1)
fn handle_assign(finding_id: &str, owner: &str) {
    println!("Assignment recorded for v1 documentation purposes.");
    println!();
    println!("To complete the assignment, manually update delivery-policy.toml:");
    println!();
    println!("[routing_rules.{finding_id}]");
    println!("owner = \"{owner}\"");
    println!();
    println!("Then commit and push the config change.");
    std::process::exit(0);
}

/// Handle audit subcommand - query acknowledgment audit trail
fn handle_audit(
    finding: Option<String>,
    gate: Option<String>,
    week: Option<String>,
    acks_dir: &Path,
) {
    use strategy_validator::audit::{query_acks, format_audit_table, AuditFilter};

    let filter = AuditFilter {
        finding_id: finding,
        gate_id: gate,
        week,
    };

    let records = query_acks(acks_dir, &filter);

    if records.is_empty() {
        println!("No acknowledgments found");
        std::process::exit(0);
    }

    println!("{}", format_audit_table(&records));
    std::process::exit(0);
}

#[allow(dead_code)] // Used by main binary
fn load_schema(schemas_dir: &Path, name: &str) -> JSONSchema {
    let schema_path = schemas_dir.join(format!("{name}.schema.json"));
    let schema_content = fs::read_to_string(&schema_path).unwrap_or_else(|_| {
        eprintln!("{} Failed to read schema: {}", "Error:".red(), schema_path.display());
        std::process::exit(1);
    });

    let schema_json: Value = serde_json::from_str(&schema_content).unwrap_or_else(|e| {
        eprintln!("{} Invalid JSON in schema {}: {e}", "Error:".red(), schema_path.display());
        std::process::exit(1);
    });

    JSONSchema::compile(&schema_json).unwrap_or_else(|e| {
        eprintln!("{} Invalid schema {}: {e}", "Error:".red(), schema_path.display());
        std::process::exit(1);
    })
}

#[allow(dead_code)] // Used by main binary
fn classify_fixture(relative_path: &Path) -> (FixtureType, &'static str) {
    let path_str = relative_path.to_string_lossy();

    if path_str.contains("/acks/") || path_str.contains("\\acks\\") {
        (FixtureType::Acknowledgment, "strategy_ack")
    } else if path_str.contains("/overrides/") || path_str.contains("\\overrides\\") {
        (FixtureType::Override, "strategy_override")
    } else {
        (FixtureType::GateExecution, "strategy_gate_execution")
    }
}

#[allow(dead_code)] // Used by main binary
#[allow(clippy::too_many_lines)]
fn validate_semantic(
    path: &Path,
    fixtures_dir: &Path,
    content: &str,
    lens_packs: &LensPacksConfig,
    fixture_type: &FixtureType,
) -> Vec<SemanticError> {
    // Only validate gate executions (acks/overrides don't need semantic checks)
    if !matches!(fixture_type, FixtureType::GateExecution) {
        return vec![];
    }

    let execution: GateExecution = match serde_yaml::from_str(content) {
        Ok(e) => e,
        Err(_) => return vec![], // Schema validation will catch parse errors
    };

    let mut errors = Vec::new();
    let path_str = path.display().to_string();

    // VAL-01: Primaries present
    if let Some(gate_config) = lens_packs.gates.get(&execution.gate_id) {
        let voices_present: std::collections::HashSet<_> =
            execution.lens_results.iter().map(|r| r.voice.as_str()).collect();

        for primary in &gate_config.primary_voices {
            if !voices_present.contains(primary.as_str()) {
                errors.push(SemanticError::MissingPrimary {
                    gate_id: execution.gate_id.clone(),
                    missing_voice: primary.clone(),
                });
            }
        }
    }

    // VAL-02: WARN requires ack + counter-voice consulted
    if execution.decision.outcome == "WARN" {
        match &execution.acknowledgment_ref {
            None => {
                errors.push(SemanticError::AckMissing { fixture_path: path_str.clone() });
            }
            Some(ack_ref) => {
                // Resolve ack path relative to fixture's directory
                let parent_dir = path.parent().unwrap_or(fixtures_dir);
                let ack_path = parent_dir.join(ack_ref);

                match fs::read_to_string(&ack_path) {
                    Err(e) => {
                        errors.push(SemanticError::AckNotReadable {
                            ack_path: ack_path.display().to_string(),
                            reason: e.to_string(),
                        });
                    }
                    Ok(ack_content) => {
                        if let Ok(ack) = serde_yaml::from_str::<Acknowledgment>(&ack_content) {
                            // Check counter-voice validity
                            if let Some(gate_config) = lens_packs.gates.get(&execution.gate_id) {
                                if !gate_config.counter_voices.contains(&ack.counter_voice_consulted.voice) {
                                    errors.push(SemanticError::CounterVoiceMismatch {
                                        ack_path: ack_path.display().to_string(),
                                        actual: ack.counter_voice_consulted.voice.clone(),
                                        valid: gate_config.counter_voices.clone(),
                                    });
                                }
                            }

                            // Check ack references warnings that exist in lens_results
                            let result_check_ids: std::collections::HashSet<_> =
                                execution.lens_results.iter()
                                    .filter(|r| r.severity == "WARN")
                                    .map(|r| r.check_id.as_str())
                                    .collect();

                            for accepted in &ack.warnings_accepted {
                                if !result_check_ids.contains(accepted.check_id.as_str()) {
                                    errors.push(SemanticError::AckWarningNotInResults {
                                        ack_path: ack_path.display().to_string(),
                                        check_id: accepted.check_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // VAL-03: STOP only from Class A
    if execution.decision.outcome == "STOP" {
        let has_class_a_stop = execution.lens_results.iter()
            .any(|r| r.severity == "STOP" && r.class == "A");

        if !has_class_a_stop {
            errors.push(SemanticError::StopNotClassA { fixture_path: path_str });
        }
    }

    errors
}

#[allow(dead_code)] // Used by main binary
fn validate_fixture(
    path: &Path,
    schema: &JSONSchema,
    fixture_type: FixtureType,
    fixtures_dir: &Path,
    lens_packs: &LensPacksConfig,
) -> ValidationResult {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return ValidationResult {
                path: path.to_path_buf(),
                fixture_type,
                success: false,
                errors: vec![format!("Failed to read file: {}", e)],
                semantic_errors: vec![],
            };
        }
    };

    // Parse YAML
    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return ValidationResult {
                path: path.to_path_buf(),
                fixture_type,
                success: false,
                errors: vec![format!("Invalid YAML: {}", e)],
                semantic_errors: vec![],
            };
        }
    };

    // Convert to JSON for schema validation
    let json_value: Value = match serde_json::to_value(&yaml_value) {
        Ok(v) => v,
        Err(e) => {
            return ValidationResult {
                path: path.to_path_buf(),
                fixture_type,
                success: false,
                errors: vec![format!("YAML to JSON conversion failed: {}", e)],
                semantic_errors: vec![],
            };
        }
    };

    // Validate against schema
    let validation_result = schema.validate(&json_value);

    let schema_errors: Vec<String> = match validation_result {
        Ok(()) => vec![],
        Err(errors) => {
            errors
                .map(|e| {
                    let path = e.instance_path.to_string();
                    if path.is_empty() {
                        e.to_string()
                    } else {
                        format!("{path}: {e}")
                    }
                })
                .collect()
        }
    };

    // Run semantic validation if schema validation passed
    let semantic_errors = if schema_errors.is_empty() {
        validate_semantic(path, fixtures_dir, &content, lens_packs, &fixture_type)
    } else {
        vec![]
    };

    let success = schema_errors.is_empty() && semantic_errors.is_empty();

    ValidationResult {
        path: path.to_path_buf(),
        fixture_type,
        success,
        errors: schema_errors,
        semantic_errors,
    }
}

#[allow(dead_code)] // Used by main binary
fn print_results(results: &[ValidationResult], base_dir: &Path, verbose: bool) {
    let gate_executions: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.fixture_type, FixtureType::GateExecution))
        .collect();
    let acknowledgments: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.fixture_type, FixtureType::Acknowledgment))
        .collect();
    let overrides: Vec<_> = results
        .iter()
        .filter(|r| matches!(r.fixture_type, FixtureType::Override))
        .collect();

    if !gate_executions.is_empty() {
        println!("{}", "## Gate Executions".bold());
        for result in gate_executions {
            print_result(result, base_dir, verbose);
        }
        println!();
    }

    if !acknowledgments.is_empty() {
        println!("{}", "## Acknowledgments".bold());
        for result in acknowledgments {
            print_result(result, base_dir, verbose);
        }
        println!();
    }

    if !overrides.is_empty() {
        println!("{}", "## Overrides".bold());
        for result in overrides {
            print_result(result, base_dir, verbose);
        }
        println!();
    }

    let pass_count = results.iter().filter(|r| r.success).count();
    let fail_count = results.iter().filter(|r| !r.success).count();

    println!("{}", "━".repeat(55));
    println!(
        " Results: {}, {}",
        format!("{pass_count} passed").green(),
        format!("{fail_count} failed").red()
    );
    println!("{}", "━".repeat(55));
}

#[allow(dead_code)] // Used by main binary
fn print_result(result: &ValidationResult, base_dir: &Path, verbose: bool) {
    let relative_path = result.path.strip_prefix(base_dir).unwrap_or(&result.path);
    let display_path = relative_path.display();

    if result.success {
        println!("{} {}", "✓".green(), display_path);
    } else {
        println!("{} {}", "✗".red(), display_path);

        // Print schema errors in red
        for (i, error) in result.errors.iter().enumerate() {
            if verbose || i < 3 {
                println!("  {}", error.red().dimmed());
            }
        }
        if !verbose && result.errors.len() > 3 {
            println!("  {} more schema errors...", result.errors.len() - 3);
        }

        // Print semantic errors in yellow
        for (i, error) in result.semantic_errors.iter().enumerate() {
            if verbose || i < 3 {
                println!("  {}", error.to_string().yellow().dimmed());
            }
        }
        if !verbose && result.semantic_errors.len() > 3 {
            println!("  {} more semantic errors...", result.semantic_errors.len() - 3);
        }
    }
}

/// Test helper: validate semantic rules from YAML string
/// Allows unit tests to use inline fixtures without file I/O
#[cfg(test)]
pub fn validate_semantic_str(
    yaml: &str,
    lens_packs: &LensPacksConfig,
    _gate_id: &str,
) -> Vec<SemanticError> {
    // Parse and validate without file dependencies
    // This is a simplified version that doesn't do ack file cross-validation
    let execution: GateExecution = match serde_yaml::from_str(yaml) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    let mut errors = Vec::new();

    // VAL-01: Primaries present
    if let Some(gate_config) = lens_packs.gates.get(&execution.gate_id) {
        let voices_present: std::collections::HashSet<_> =
            execution.lens_results.iter().map(|r| r.voice.as_str()).collect();

        for primary in &gate_config.primary_voices {
            if !voices_present.contains(primary.as_str()) {
                errors.push(SemanticError::MissingPrimary {
                    gate_id: execution.gate_id.clone(),
                    missing_voice: primary.clone(),
                });
            }
        }
    }

    // VAL-02: WARN requires ack (simplified - just check field presence)
    if execution.decision.outcome == "WARN" && execution.acknowledgment_ref.is_none() {
        errors.push(SemanticError::AckMissing {
            fixture_path: "inline".to_string(),
        });
    }

    // VAL-03: STOP only from Class A
    if execution.decision.outcome == "STOP" {
        let has_class_a_stop = execution.lens_results.iter()
            .any(|r| r.severity == "STOP" && r.class == "A");

        if !has_class_a_stop {
            errors.push(SemanticError::StopNotClassA {
                fixture_path: "inline".to_string(),
            });
        }
    }

    errors
}
