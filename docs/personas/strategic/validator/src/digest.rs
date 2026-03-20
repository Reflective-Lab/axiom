//! Weekly Digest
//!
//! Aggregates gate execution fixtures into weekly statistical summaries.
//!
//! This module provides types and functions for transforming raw gate execution
//! fixtures into digestible weekly reports showing PASS/WARN/STOP patterns,
//! acknowledgment tracking, and counter-voice usage.

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use crate::thresholds::{ProfileThresholds, RedFlagConfig};

/// Decision outcome from a gate execution
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Decision {
    pub outcome: String,
}

/// Individual lens result within a gate execution
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LensResult {
    pub check_id: String,
    pub voice: String,
    pub class: String,
    pub severity: String,
    pub summary: String,
}

/// Gate execution fixture (parsed from YAML)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GateExecution {
    pub fixture_id: String,
    pub gate_id: String,
    pub timestamp: String,
    pub decision: Decision,
    pub lens_results: Vec<LensResult>,
    pub acknowledgment_ref: Option<String>,
}

/// Acknowledgment file (parsed from acks/ directories)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ack {
    pub gate_id: String,
    pub timestamp: String,
    pub warnings_accepted: Vec<AckWarning>,
    pub counter_voice_consulted: Option<CounterVoice>,
}

/// Individual warning acknowledgment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AckWarning {
    pub check_id: String,
    pub rationale: String,
    pub mitigation: String,
}

/// Counter-voice consultation record
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CounterVoice {
    pub voice: String,
    pub conclusion: String,
}

/// Override file (parsed from overrides/ directories)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Override {
    pub gate_id: String,
    pub stop_checks: Vec<String>,
    pub timestamp: String,
    pub justification: String,
    pub scope: String,
    pub expiry: String,
    pub signatures: Vec<Signature>,
}

/// Signature on an override
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Signature {
    pub name: String,
    pub role: String,
}

/// Statistics for a single gate
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GateStats {
    pub total: usize,
    pub pass_count: usize,
    pub warn_count: usize,
    pub stop_count: usize,
}

/// Pressure signatures requiring human attention
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PressureSignature {
    /// WARN spike detected (>3 warnings on a single gate)
    WarnSpike {
        gate_id: String,
        count: usize,
        threshold: usize,
    },
    /// Repeated trigger detected (same `check_id` triggered >2 times)
    RepeatedTrigger {
        check_id: String,
        count: usize,
        threshold: usize,
    },
    /// Override active on gate
    OverrideUsed {
        gate_id: String,
        stop_checks: Vec<String>,
    },
    /// Same `check_id` acknowledged repeatedly (>2 times)
    RepeatedAck {
        check_id: String,
        count: usize,
        threshold: usize,
    },
    /// Same counter-voice consulted repeatedly (>2 times)
    RepeatedCounterVoice {
        voice: String,
        count: usize,
        threshold: usize,
    },
}

/// Aggregated digest statistics from fixture files
#[derive(Debug, Clone, Default)]
pub struct DigestStats {
    /// Total gate execution fixtures processed
    pub total_fixtures: usize,

    /// Count by outcome (PASS, WARN, STOP)
    pub by_outcome: HashMap<String, usize>,

    /// Statistics per gate
    pub by_gate: HashMap<String, GateStats>,

    /// Count of which `check_ids` triggered WARN/STOP
    pub trigger_counts: HashMap<String, usize>,

    /// Count of fixtures with `acknowledgment_ref`
    pub ack_count: usize,

    /// Count of which `check_ids` got acknowledged (from ack files)
    pub ack_check_counts: HashMap<String, usize>,

    /// Count of counter-voice usage by voice name (from ack files)
    pub counter_voice_counts: HashMap<String, usize>,

    /// Collected override files
    pub overrides: Vec<Override>,

    /// Count of parse errors (non-fatal)
    pub parse_errors: usize,
}

/// Summary statistics for digest report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSummary {
    pub total_executions: usize,
    pub pass_count: usize,
    pub warn_count: usize,
    pub stop_count: usize,
    pub ack_count: usize,
    pub parse_errors: usize,
}

/// Complete weekly digest report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestReport {
    pub week: String,
    pub generated_at: String,
    pub summary: DigestSummary,
    pub by_gate: HashMap<String, GateStats>,
    pub top_triggers: Vec<(String, usize)>,
    pub pressure_signatures: Vec<PressureSignature>,
    pub red_flags: Vec<PressureSignature>,
    pub ack_rate: f64,
}

impl DigestReport {
    /// Serialize report to JSON string
    pub fn to_json(&self) -> Result<String, DigestError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Generate markdown report
    pub fn to_markdown(&self) -> String {
        use std::fmt::Write;

        let mut md = String::new();

        // Header
        let _ = writeln!(md, "# Weekly Digest: Week {}\n", self.week);
        let _ = writeln!(md, "**Generated:** {}\n", self.generated_at);

        // Summary
        md.push_str("## Summary\n\n");
        let total = self.summary.total_executions;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pass_pct = if total > 0 { (self.summary.pass_count as f64 / total as f64 * 100.0) as usize } else { 0 };
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let warn_pct = if total > 0 { (self.summary.warn_count as f64 / total as f64 * 100.0) as usize } else { 0 };
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let stop_pct = if total > 0 { (self.summary.stop_count as f64 / total as f64 * 100.0) as usize } else { 0 };

        let _ = writeln!(md, "- **Total gate executions:** {total}");
        let _ = writeln!(md, "- **PASS:** {} ({pass_pct}%)", self.summary.pass_count);
        let _ = writeln!(md, "- **WARN:** {} ({warn_pct}%)", self.summary.warn_count);
        let _ = writeln!(md, "- **STOP:** {} ({stop_pct}%)", self.summary.stop_count);
        let _ = writeln!(md, "- **Acknowledgments:** {} ({:.1}%)", self.summary.ack_count, self.ack_rate);
        md.push('\n');

        // Activity by Gate
        md.push_str("## Activity by Gate\n\n");
        md.push_str("| Gate | Total | PASS | WARN | STOP |\n");
        md.push_str("|------|-------|------|------|------|\n");

        let mut gates: Vec<_> = self.by_gate.iter().collect();
        gates.sort_by_key(|(gate_id, _)| *gate_id);
        for (gate_id, stats) in gates {
            let _ = writeln!(md, "| {} | {} | {} | {} | {} |", gate_id, stats.total, stats.pass_count, stats.warn_count, stats.stop_count);
        }
        md.push('\n');

        // Top Triggers
        md.push_str("## Top Triggers\n\n");
        md.push_str("Checks that produced WARN or STOP:\n\n");

        if self.top_triggers.is_empty() {
            md.push_str("No triggers detected.\n\n");
        } else {
            for (i, (check_id, count)) in self.top_triggers.iter().enumerate() {
                let _ = writeln!(md, "{}. **{}**: {} occurrences", i + 1, check_id, count);
            }
            md.push('\n');
        }

        // Pressure Signatures
        md.push_str("## Pressure Signatures\n\n");
        md.push_str("Patterns requiring attention:\n\n");

        if self.pressure_signatures.is_empty() {
            md.push_str("No pressure signatures detected.\n\n");
        } else {
            for signature in &self.pressure_signatures {
                let formatted = match signature {
                    PressureSignature::WarnSpike { gate_id, count, threshold } => {
                        format!("- WARN spike on gate `{gate_id}`: {count} warnings (threshold: {threshold})")
                    }
                    PressureSignature::RepeatedTrigger { check_id, count, threshold } => {
                        format!("- Repeated trigger `{check_id}`: {count} times (threshold: {threshold})")
                    }
                    PressureSignature::OverrideUsed { gate_id, stop_checks } => {
                        format!("- Override active on gate `{gate_id}`: bypassing {}", stop_checks.join(", "))
                    }
                    PressureSignature::RepeatedAck { check_id, count, threshold } => {
                        format!("- Repeated acknowledgment for `{check_id}`: {count} times (threshold: {threshold}) - may indicate unaddressed root cause")
                    }
                    PressureSignature::RepeatedCounterVoice { voice, count, threshold } => {
                        format!("- Counter-voice `{voice}` consulted {count} times (threshold: {threshold}) - may indicate recurring friction")
                    }
                };
                let _ = writeln!(md, "{formatted}");
            }
            md.push('\n');
        }

        // Red-Flag Moments
        md.push_str("## Red-Flag Moments\n\n");
        md.push_str("Institutional safeguard overruns (Marks/Thaler territory):\n\n");

        if self.red_flags.is_empty() {
            md.push_str("No red-flag moments detected.\n\n");
        } else {
            for red_flag in &self.red_flags {
                let formatted = match red_flag {
                    PressureSignature::OverrideUsed { gate_id, stop_checks } => {
                        format!("- **OVERRIDE ACTIVE**: Gate `{gate_id}` bypassing institutional STOP checks: {} - Requires executive review", stop_checks.join(", "))
                    }
                    PressureSignature::WarnSpike { gate_id, count, threshold } => {
                        format!("- **SELL/SCALE PRESSURE**: Gate `{gate_id}` showing {count} warnings (threshold: {threshold}) - Promotional pressure overrunning deliberate gates")
                    }
                    PressureSignature::RepeatedAck { check_id, count, threshold } => {
                        format!("- **INSTITUTIONAL FRICTION**: Check `{check_id}` acknowledged {count} times (threshold: {threshold}) - Suggests unaddressed systemic issue")
                    }
                    _ => String::new()
                };
                if !formatted.is_empty() {
                    let _ = writeln!(md, "{formatted}");
                }
            }
            md.push('\n');
        }

        md
    }

    /// Write both markdown and JSON reports to output directory
    pub fn write_reports(&self, output_dir: &Path) -> Result<(), DigestError> {
        use std::fs;

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

/// Digest errors
#[derive(Debug)]
pub enum DigestError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for DigestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Yaml(e) => write!(f, "YAML error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for DigestError {}

impl From<std::io::Error> for DigestError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_yaml::Error> for DigestError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Yaml(err)
    }
}

impl From<serde_json::Error> for DigestError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

/// Check if path is a gate execution file (YAML, NOT in acks/ or overrides/)
fn is_gate_execution(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if ext != "yaml" && ext != "yml" {
            return false;
        }
    } else {
        return false;
    }

    // Exclude acks/ and overrides/ directories
    let path_str = path.to_string_lossy();
    !path_str.contains("/acks/") && !path_str.contains("/overrides/")
}

/// Check if path is an ack file (YAML in acks/ directory)
fn is_ack_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if ext != "yaml" && ext != "yml" {
            return false;
        }
    } else {
        return false;
    }

    let path_str = path.to_string_lossy();
    path_str.contains("/acks/")
}

/// Check if path is an override file (YAML in overrides/ directory)
fn is_override_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if ext != "yaml" && ext != "yml" {
            return false;
        }
    } else {
        return false;
    }

    let path_str = path.to_string_lossy();
    path_str.contains("/overrides/")
}

/// Aggregate fixture files into weekly digest statistics
///
/// Performs a 3-pass walk of the fixtures directory:
/// 1. Gate executions: Count outcomes, gate stats, triggers, acknowledgments
/// 2. Ack files: Count `check_id` acknowledgments and counter-voice usage
/// 3. Override files: Collect override records
///
/// Parse errors are counted but non-fatal.
///
/// # Arguments
///
/// * `fixtures_dir` - Path to fixtures directory
///
/// # Errors
///
/// Returns `DigestError` if directory traversal fails.
pub fn aggregate_fixtures(fixtures_dir: &Path) -> Result<DigestStats, DigestError> {
    let mut stats = DigestStats::default();

    // PASS 1: Gate executions (exclude acks/, overrides/)
    for entry in WalkDir::new(fixtures_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_gate_execution(e.path()))
    {
        let path = entry.path();

        // Read and parse gate execution
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str::<GateExecution>(&content) {
                Ok(gate_exec) => {
                    // Increment total fixtures
                    stats.total_fixtures += 1;

                    // Count by outcome
                    *stats
                        .by_outcome
                        .entry(gate_exec.decision.outcome.clone())
                        .or_insert(0) += 1;

                    // Update gate stats
                    let gate_stats = stats.by_gate.entry(gate_exec.gate_id.clone()).or_default();
                    gate_stats.total += 1;

                    match gate_exec.decision.outcome.as_str() {
                        "PASS" => gate_stats.pass_count += 1,
                        "WARN" => gate_stats.warn_count += 1,
                        "STOP" => gate_stats.stop_count += 1,
                        _ => {}
                    }

                    // Count trigger check_ids (WARN or STOP severity)
                    for lens_result in &gate_exec.lens_results {
                        if lens_result.severity == "WARN" || lens_result.severity == "STOP" {
                            *stats
                                .trigger_counts
                                .entry(lens_result.check_id.clone())
                                .or_insert(0) += 1;
                        }
                    }

                    // Count acknowledgments
                    if gate_exec.acknowledgment_ref.is_some() {
                        stats.ack_count += 1;
                    }
                }
                Err(_) => {
                    stats.parse_errors += 1;
                }
            },
            Err(_) => {
                stats.parse_errors += 1;
            }
        }
    }

    // PASS 2: Ack files (only acks/ directories)
    for entry in WalkDir::new(fixtures_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_ack_file(e.path()))
    {
        let path = entry.path();

        // Read and parse ack file
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str::<Ack>(&content) {
                Ok(ack) => {
                    // Count ack check_ids
                    for warning in &ack.warnings_accepted {
                        *stats
                            .ack_check_counts
                            .entry(warning.check_id.clone())
                            .or_insert(0) += 1;
                    }

                    // Count counter-voice usage
                    if let Some(counter_voice) = &ack.counter_voice_consulted {
                        *stats
                            .counter_voice_counts
                            .entry(counter_voice.voice.clone())
                            .or_insert(0) += 1;
                    }
                }
                Err(_) => {
                    stats.parse_errors += 1;
                }
            },
            Err(_) => {
                stats.parse_errors += 1;
            }
        }
    }

    // PASS 3: Override files (only overrides/ directories)
    for entry in WalkDir::new(fixtures_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_override_file(e.path()))
    {
        let path = entry.path();

        // Read and parse override file
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str::<Override>(&content) {
                Ok(override_rec) => {
                    stats.overrides.push(override_rec);
                }
                Err(_) => {
                    stats.parse_errors += 1;
                }
            },
            Err(_) => {
                stats.parse_errors += 1;
            }
        }
    }

    Ok(stats)
}

/// Filter red-flag moments from pressure signatures
///
/// Red flags are institutional safeguard overruns requiring attention.
/// Distinct from pressure signatures which include operational awareness.
///
/// # Arguments
///
/// * `signatures` - All detected pressure signatures
/// * `thresholds` - Active threshold profile
/// * `config` - Red-flag configuration
pub fn filter_red_flags(
    signatures: &[PressureSignature],
    thresholds: &ProfileThresholds,
    config: &RedFlagConfig,
) -> Vec<PressureSignature> {
    signatures
        .iter()
        .filter(|sig| is_institutional_overrun(sig, thresholds, config))
        .cloned()
        .collect()
}

/// Check if pressure signature indicates institutional overrun
fn is_institutional_overrun(
    sig: &PressureSignature,
    thresholds: &ProfileThresholds,
    config: &RedFlagConfig,
) -> bool {
    match sig {
        // Override usage always indicates institutional bypass
        PressureSignature::OverrideUsed { .. } => thresholds.override_always_flags,

        // High WARN spike on SELL/SCALE gates = promotional pressure
        PressureSignature::WarnSpike { gate_id, count, threshold } => {
            let is_sell_scale = config.sell_scale_gates.contains(gate_id);
            #[allow(clippy::cast_precision_loss)]
            let exceeds_multiplier = *count as f64 > (*threshold as f64 * config.institutional_overrun_multiplier);
            is_sell_scale && exceeds_multiplier
        }

        // Repeated acks indicate persistent institutional friction
        PressureSignature::RepeatedAck { count, threshold, .. } => {
            #[allow(clippy::cast_precision_loss)]
            let exceeds = *count as f64 > (*threshold as f64 * config.institutional_overrun_multiplier);
            exceeds
        }

        // Other signatures are operational awareness, not red flags
        PressureSignature::RepeatedTrigger { .. } | PressureSignature::RepeatedCounterVoice { .. } => false,
    }
}

/// Detect pressure signatures in digest statistics
///
/// Applies configurable thresholds from profile.
///
/// # Arguments
///
/// * `stats` - Aggregated digest statistics
/// * `thresholds` - Active threshold profile
pub fn detect_pressure_signatures(stats: &DigestStats, thresholds: &ProfileThresholds) -> Vec<PressureSignature> {
    let mut signatures = Vec::new();

    // WARN spike detection
    for (gate_id, gate_stats) in &stats.by_gate {
        if gate_stats.warn_count > thresholds.warn_spike_threshold {
            signatures.push(PressureSignature::WarnSpike {
                gate_id: gate_id.clone(),
                count: gate_stats.warn_count,
                threshold: thresholds.warn_spike_threshold,
            });
        }
    }

    // Repeated trigger detection
    for (check_id, count) in &stats.trigger_counts {
        if *count > thresholds.repeated_trigger_threshold {
            signatures.push(PressureSignature::RepeatedTrigger {
                check_id: check_id.clone(),
                count: *count,
                threshold: thresholds.repeated_trigger_threshold,
            });
        }
    }

    // Override usage detection (any active override)
    for override_rec in &stats.overrides {
        signatures.push(PressureSignature::OverrideUsed {
            gate_id: override_rec.gate_id.clone(),
            stop_checks: override_rec.stop_checks.clone(),
        });
    }

    // Repeated ack detection
    for (check_id, count) in &stats.ack_check_counts {
        if *count > thresholds.repeated_ack_threshold {
            signatures.push(PressureSignature::RepeatedAck {
                check_id: check_id.clone(),
                count: *count,
                threshold: thresholds.repeated_ack_threshold,
            });
        }
    }

    // Repeated counter-voice detection
    for (voice, count) in &stats.counter_voice_counts {
        if *count > thresholds.repeated_counter_voice_threshold {
            signatures.push(PressureSignature::RepeatedCounterVoice {
                voice: voice.clone(),
                count: *count,
                threshold: thresholds.repeated_counter_voice_threshold,
            });
        }
    }

    signatures
}

/// Get current ISO week in YYYY-WW format
fn current_iso_week() -> String {
    let now = chrono::Local::now();
    format!("{:04}-{:02}", now.iso_week().year(), now.iso_week().week())
}

/// Compute top N triggers sorted by count descending
fn compute_top_triggers(trigger_counts: &HashMap<String, usize>, limit: usize) -> Vec<(String, usize)> {
    let mut triggers: Vec<_> = trigger_counts.iter()
        .map(|(check_id, count)| (check_id.clone(), *count))
        .collect();

    triggers.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    triggers.into_iter().take(limit).collect()
}

/// Run digest generation
///
/// Aggregates fixtures, detects pressure signatures, and generates a weekly digest report.
///
/// # Arguments
///
/// * `fixtures_dir` - Path to fixtures directory
/// * `output_dir` - Path to output directory for digest reports
/// * `week` - Optional week identifier (YYYY-WW format), defaults to current
/// * `thresholds` - Active threshold profile
/// * `red_flag_config` - Red-flag configuration
///
/// # Errors
///
/// Returns `DigestError` if aggregation or report writing fails.
pub fn run_digest(
    fixtures_dir: &Path,
    output_dir: &Path,
    week: Option<String>,
    thresholds: &ProfileThresholds,
    red_flag_config: &RedFlagConfig,
) -> Result<DigestReport, DigestError> {
    // Get week (default to current ISO week)
    let week = week.unwrap_or_else(current_iso_week);

    // Aggregate fixtures
    let stats = aggregate_fixtures(fixtures_dir)?;

    // Detect pressure signatures using configurable thresholds
    let pressure_signatures = detect_pressure_signatures(&stats, thresholds);

    // Filter red-flag moments
    let red_flags = filter_red_flags(&pressure_signatures, thresholds, red_flag_config);

    // Compute top triggers (top 5)
    let top_triggers = compute_top_triggers(&stats.trigger_counts, 5);

    // Calculate ack rate
    #[allow(clippy::cast_precision_loss)]
    let ack_rate = if stats.total_fixtures > 0 {
        stats.ack_count as f64 / stats.total_fixtures as f64 * 100.0
    } else {
        0.0
    };

    // Build summary
    let summary = DigestSummary {
        total_executions: stats.total_fixtures,
        pass_count: *stats.by_outcome.get("PASS").unwrap_or(&0),
        warn_count: *stats.by_outcome.get("WARN").unwrap_or(&0),
        stop_count: *stats.by_outcome.get("STOP").unwrap_or(&0),
        ack_count: stats.ack_count,
        parse_errors: stats.parse_errors,
    };

    // Create report
    let report = DigestReport {
        week,
        generated_at: chrono::Local::now().to_rfc3339(),
        summary,
        by_gate: stats.by_gate,
        top_triggers,
        pressure_signatures,
        red_flags,
        ack_rate,
    };

    // Write reports
    report.write_reports(output_dir)?;

    Ok(report)
}
