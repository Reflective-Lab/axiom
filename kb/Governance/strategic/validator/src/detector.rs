//! Drift Detection
//!
//! Detects and reports drift violations in strategy gate fixtures.
//!
//! Implements the drift taxonomy from DRIFT.md with stable codes:
//! - `D_SPEC`: Specification drift
//! - `D_FIX`: Fixture drift
//! - `D_NARR`: Narrative drift (advisory-only)
//! - `D_AUTH`: Authority drift
//! - `D_OPS`: Operational drift

use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fingerprint::{fingerprint_repo, ArtifactFingerprint};

/// Drift codes from DRIFT.md taxonomy (stable identifiers)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DriftCode {
    /// `D_SPEC`: Spec drift
    DSpec,
    /// `D_FIX`: Fixture drift
    DFix,
    /// `D_NARR`: Narrative drift (advisory-only in v4.1)
    DNarr,
    /// `D_AUTH`: Authority drift
    DAuth,
    /// `D_OPS`: Operational drift
    DOps,
}

impl std::fmt::Display for DriftCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DSpec => write!(f, "D_SPEC"),
            Self::DFix => write!(f, "D_FIX"),
            Self::DNarr => write!(f, "D_NARR"),
            Self::DAuth => write!(f, "D_AUTH"),
            Self::DOps => write!(f, "D_OPS"),
        }
    }
}

/// Severity levels from DRIFT.md
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// A single drift finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFinding {
    pub code: DriftCode,
    pub severity: Severity,
    pub artifact_path: PathBuf,
    pub commit_hash: Option<String>,
    pub tree_hash: Option<String>,
    pub dirty: bool,
    pub invariant: String,
    pub description: String,
}

/// Summary statistics for a drift report
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriftSummary {
    pub total: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub by_code: std::collections::HashMap<String, usize>,
}

/// Complete drift report for a week
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub week: String,
    pub generated_at: String,
    pub repo_fingerprint: ArtifactFingerprint,
    pub findings: Vec<DriftFinding>,
    pub summary: DriftSummary,
}

impl DriftReport {
    /// Serialize report to JSON string
    pub fn to_json(&self) -> Result<String, DetectorError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Generate markdown report
    pub fn to_markdown(&self) -> String {
        use std::fmt::Write;

        let mut md = String::new();

        // Header
        let _ = writeln!(md, "# Drift Report: Week {}\n", self.week);
        let _ = writeln!(md, "**Generated:** {}\n", self.generated_at);

        // Repository fingerprint
        md.push_str("## Repository Fingerprint\n\n");
        if let Some(commit) = &self.repo_fingerprint.commit_hash {
            let _ = writeln!(md, "- **Commit:** `{commit}`");
        }
        let _ = writeln!(md, "- **Dirty:** {}", self.repo_fingerprint.dirty);
        if let Some(tree) = &self.repo_fingerprint.tree_hash {
            let _ = writeln!(md, "- **Tree:** `{tree}`");
        }
        md.push('\n');

        // Summary
        md.push_str("## Summary\n\n");
        let _ = writeln!(md, "- **Total findings:** {}", self.summary.total);
        let _ = writeln!(md, "- **High severity:** {}", self.summary.high_count);
        let _ = writeln!(md, "- **Medium severity:** {}", self.summary.medium_count);
        let _ = writeln!(md, "- **Low severity:** {}", self.summary.low_count);
        md.push('\n');

        if !self.summary.by_code.is_empty() {
            md.push_str("### By Code\n\n");
            let mut codes: Vec<_> = self.summary.by_code.iter().collect();
            codes.sort_by_key(|(code, _)| *code);
            for (code, count) in codes {
                let _ = writeln!(md, "- **{code}:** {count}");
            }
            md.push('\n');
        }

        // Findings
        md.push_str("## Findings\n\n");
        if self.findings.is_empty() {
            md.push_str("No drift detected.\n\n");
        } else {
            for (i, finding) in self.findings.iter().enumerate() {
                let _ = writeln!(md, "### {}. {} ({:?})\n", i + 1, finding.code, finding.severity);
                let _ = writeln!(md, "**Artifact:** `{}`\n", finding.artifact_path.display());
                if let Some(commit) = &finding.commit_hash {
                    let _ = writeln!(md, "**Commit:** `{commit}`");
                }
                if let Some(tree) = &finding.tree_hash {
                    let _ = writeln!(md, "**Tree:** `{tree}`");
                }
                let _ = writeln!(md, "**Dirty:** {}\n", finding.dirty);
                let _ = writeln!(md, "**Invariant:** {}\n", finding.invariant);
                let _ = writeln!(md, "{}\n", finding.description);
            }
        }

        md
    }

    /// Write both markdown and JSON reports to output directory
    pub fn write_reports(&self, output_dir: &Path) -> Result<(), DetectorError> {
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir)?;

        // Write markdown report
        let md_path = output_dir.join(format!("{}.md", self.week));
        fs::write(&md_path, self.to_markdown())?;

        // Write JSON report
        let json_path = output_dir.join(format!("{}.json", self.week));
        fs::write(&json_path, self.to_json()?)?;

        Ok(())
    }
}

/// Detector errors
#[derive(Debug)]
pub enum DetectorError {
    Io(std::io::Error),
    Git(git2::Error),
    Config(String),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for DetectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Git(e) => write!(f, "Git error: {e}"),
            Self::Config(msg) => write!(f, "Config error: {msg}"),
            Self::Serialization(e) => write!(f, "Serialization error: {e}"),
        }
    }
}

impl std::error::Error for DetectorError {}

impl From<std::io::Error> for DetectorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<git2::Error> for DetectorError {
    fn from(err: git2::Error) -> Self {
        Self::Git(err)
    }
}

impl From<serde_json::Error> for DetectorError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

/// Cron-compatible exit codes for detector
#[derive(Debug, Clone, Copy)]
pub enum DetectorExitCode {
    Success = 0,       // No drift detected
    DriftDetected = 1, // Drift found, report generated
    ConfigError = 2,   // Invalid config/missing files
    IoError = 3,       // File system error
}

/// Get current ISO week in YYYY-WW format
fn current_iso_week() -> String {
    let now = Local::now();
    format!("{:04}-{:02}", now.iso_week().year(), now.iso_week().week())
}

/// Build summary statistics from findings
fn build_summary(findings: &[DriftFinding]) -> DriftSummary {
    let mut summary = DriftSummary {
        total: findings.len(),
        high_count: 0,
        medium_count: 0,
        low_count: 0,
        by_code: HashMap::new(),
    };

    for finding in findings {
        // Count by severity
        match finding.severity {
            Severity::High => summary.high_count += 1,
            Severity::Medium => summary.medium_count += 1,
            Severity::Low => summary.low_count += 1,
        }

        // Count by code
        let code_str = finding.code.to_string();
        *summary.by_code.entry(code_str).or_insert(0) += 1;
    }

    summary
}

/// Detect specification drift (`D_SPEC`) via validation
///
/// In v4.1, we focus on `D_SPEC` detection via schema validation.
/// Other drift types (`D_FIX`, `D_NARR`, `D_AUTH`, `D_OPS`) can be added incrementally.
fn detect_spec_drift(
    _fixtures_dir: &Path,
    _config_dir: &Path,
) -> Vec<DriftFinding> {
    // In v4.1 initial implementation, we return empty findings
    // This is a placeholder for actual validation-based drift detection
    // which would reuse the validation logic from main.rs
    //
    // Future implementation would:
    // 1. Walk fixtures directory
    // 2. For each fixture, run schema validation
    // 3. Create DriftFinding for validation failures
    // 4. Call fingerprint_repo on artifact's parent to populate commit_hash/tree_hash/dirty

    vec![]
}

/// Find git repository root starting from a given path
fn find_git_root(start_path: &Path) -> Result<PathBuf, DetectorError> {
    let mut current = start_path.to_path_buf();

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(DetectorError::Git(git2::Error::from_str(
                    "Could not find git repository root",
                )));
            }
        }
    }
}

/// Run drift detection on fixtures
///
/// Scans fixtures directory for drift violations and generates a report.
///
/// # Arguments
///
/// * `fixtures_dir` - Path to fixtures directory
/// * `config_dir` - Path to configuration directory
/// * `output_dir` - Path to output directory for reports
/// * `week` - Optional week identifier (YYYY-WW format), defaults to current
///
/// # Errors
///
/// Returns `DetectorError` if detection fails.
pub fn run_detector(
    fixtures_dir: &Path,
    config_dir: &Path,
    output_dir: &Path,
    week: Option<String>,
) -> Result<DriftReport, DetectorError> {
    // Get week (default to current ISO week)
    let week = week.unwrap_or_else(current_iso_week);

    // Find git repository root
    let repo_root = find_git_root(fixtures_dir)?;

    // Get repo fingerprint
    let repo_fingerprint = fingerprint_repo(&repo_root)?;

    // Collect findings via detect_spec_drift
    let findings = detect_spec_drift(fixtures_dir, config_dir);

    // Build summary
    let summary = build_summary(&findings);

    // Create report
    let report = DriftReport {
        week,
        generated_at: Local::now().to_rfc3339(),
        repo_fingerprint,
        findings,
        summary,
    };

    // Write reports
    report.write_reports(output_dir)?;

    Ok(report)
}
