//! SLA Breach Detection Module
//!
//! Detects SLA breaches for WARN findings that haven't been acknowledged
//! within the required 72-hour window.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;

use chrono::{DateTime, FixedOffset, Utc};
use walkdir::WalkDir;

use crate::audit::{parse_ack_file, AckRecord};

/// SLA threshold: WARN findings must be acknowledged within 72 hours
pub const SLA_HOURS: i64 = 72;

/// Represents an SLA breach for an unacknowledged or late-acknowledged finding
#[derive(Debug, Clone)]
pub struct SlaBreach {
    /// The finding ID that breached SLA
    pub finding_id: String,
    /// The gate ID for the finding
    pub gate_id: String,
    /// Severity of the finding
    pub severity: String,
    /// When the finding was originally created
    pub finding_timestamp: DateTime<FixedOffset>,
    /// How many hours overdue (beyond SLA threshold)
    pub hours_overdue: i64,
    /// True if acknowledged after SLA, false if still unacknowledged
    pub late_acked: bool,
    /// When the finding was acknowledged (if at all)
    pub acked_at: Option<DateTime<FixedOffset>>,
}

/// Internal struct for tracking WARN findings from fixtures
#[derive(Debug)]
struct WarnFinding {
    finding_id: String,
    gate_id: String,
    timestamp: DateTime<FixedOffset>,
}

/// Scan fixtures directory for WARN findings
fn scan_warn_findings(fixtures_dir: &Path) -> Vec<WarnFinding> {
    let mut findings = Vec::new();

    if !fixtures_dir.exists() {
        return findings;
    }

    for entry in WalkDir::new(fixtures_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "yaml" || ext == "yml"))
    {
        let path = entry.path();

        // Read and parse the YAML file
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };

        // Parse YAML to look for WARN outcomes
        let Ok(yaml): Result<serde_yaml::Value, _> = serde_yaml::from_str(&content) else {
            continue;
        };

        // Check if this is a gate execution with WARN outcome
        let outcome = yaml
            .get("decision")
            .and_then(|d| d.get("outcome"))
            .and_then(|o| o.as_str());

        if outcome != Some("WARN") {
            continue;
        }

        // Extract gate_id
        let gate_id = yaml
            .get("gate_id")
            .and_then(|g| g.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract timestamp (from metadata or file modified time)
        let timestamp = yaml
            .get("metadata")
            .and_then(|m| m.get("timestamp"))
            .and_then(|t| t.as_str())
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .unwrap_or_else(|| {
                // Fall back to file metadata
                if let Ok(meta) = fs::metadata(path) {
                    if let Ok(modified) = meta.modified() {
                        let utc: DateTime<Utc> = modified.into();
                        return utc.with_timezone(&FixedOffset::east_opt(0).unwrap());
                    }
                }
                // Ultimate fallback: current time (not ideal but prevents crash)
                Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap())
            });

        // Generate a finding_id from the file path/name
        let finding_id = path
            .file_stem()
            .map_or_else(
                || format!("{gate_id}-unknown"),
                |s| s.to_string_lossy().to_string(),
            );

        findings.push(WarnFinding {
            finding_id,
            gate_id,
            timestamp,
        });
    }

    findings
}

/// Load all acknowledgments from the acks directory into a lookup map
fn load_acks(acks_dir: &Path) -> HashMap<String, AckRecord> {
    let mut acks = HashMap::new();

    if !acks_dir.exists() {
        return acks;
    }

    for entry in WalkDir::new(acks_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        if let Ok(record) = parse_ack_file(entry.path()) {
            acks.insert(record.finding_id.clone(), record);
        }
    }

    acks
}

/// Detect SLA breaches by comparing WARN findings against acknowledgments
///
/// Returns breaches for:
/// 1. WARN findings not acknowledged within 72 hours (still unacked)
/// 2. WARN findings acknowledged late (acked after 72h threshold)
pub fn detect_sla_breaches(fixtures_dir: &Path, acks_dir: &Path) -> Vec<SlaBreach> {
    let mut breaches = Vec::new();

    // Get current time for comparison
    let now = Utc::now().with_timezone(&FixedOffset::east_opt(0).unwrap());

    // Scan for WARN findings
    let warn_findings = scan_warn_findings(fixtures_dir);

    // Load all acknowledgments
    let acks = load_acks(acks_dir);

    for finding in warn_findings {
        // Check if we have an ack for this finding
        if let Some(ack) = acks.get(&finding.finding_id) {
            // Calculate time between finding and ack
            let duration = ack.acknowledged_at.signed_duration_since(finding.timestamp);
            let hours_to_ack = duration.num_hours();

            if hours_to_ack > SLA_HOURS {
                // Late ack - acknowledged but after SLA window
                breaches.push(SlaBreach {
                    finding_id: finding.finding_id,
                    gate_id: finding.gate_id,
                    severity: "WARN".to_string(),
                    finding_timestamp: finding.timestamp,
                    hours_overdue: hours_to_ack - SLA_HOURS,
                    late_acked: true,
                    acked_at: Some(ack.acknowledged_at),
                });
            }
            // If acked within SLA, no breach
        } else {
            // No ack found - check if we're past the SLA window
            let duration = now.signed_duration_since(finding.timestamp);
            let hours_since_finding = duration.num_hours();

            if hours_since_finding > SLA_HOURS {
                // SLA breach - no acknowledgment within window
                breaches.push(SlaBreach {
                    finding_id: finding.finding_id,
                    gate_id: finding.gate_id,
                    severity: "WARN".to_string(),
                    finding_timestamp: finding.timestamp,
                    hours_overdue: hours_since_finding - SLA_HOURS,
                    late_acked: false,
                    acked_at: None,
                });
            }
        }
    }

    // Sort by hours_overdue descending (most overdue first)
    breaches.sort_by(|a, b| b.hours_overdue.cmp(&a.hours_overdue));

    breaches
}

/// Format SLA breaches as markdown for inclusion in digest
pub fn format_sla_breaches(breaches: &[SlaBreach]) -> String {
    if breaches.is_empty() {
        return "No SLA breaches detected.".to_string();
    }

    let mut output = String::new();

    for breach in breaches {
        if breach.late_acked {
            // Late acknowledgment
            let acked_at = breach.acked_at.map_or_else(
                || "unknown".to_string(),
                |t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            );

            let _ = writeln!(
                output,
                "- **LATE ACK**: Finding `{}` acknowledged {}h late (acked at {acked_at})",
                breach.finding_id, breach.hours_overdue
            );
        } else {
            // Still unacknowledged
            let _ = writeln!(
                output,
                "- **SLA BREACH**: Finding `{}` unacknowledged for {}h (SLA: {SLA_HOURS}h)",
                breach.finding_id,
                breach.hours_overdue + SLA_HOURS // Total hours since finding
            );
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_warn_fixture(dir: &Path, name: &str, gate_id: &str, hours_ago: i64) -> PathBuf {
        let timestamp = Utc::now()
            .checked_sub_signed(chrono::Duration::hours(hours_ago))
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        let content = format!(
            r#"gate_id: {gate_id}
metadata:
  timestamp: "{timestamp}"
lens_results:
  - check_id: test-check
    voice: compliance
    class: B
    severity: WARN
decision:
  outcome: WARN
"#,
            timestamp = timestamp.to_rfc3339()
        );

        let file_path = dir.join(format!("{name}.yaml"));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn create_ack_for_finding(dir: &Path, finding_id: &str, hours_ago: i64) -> PathBuf {
        let week_dir = dir.join("2026-W05");
        fs::create_dir_all(&week_dir).unwrap();

        let timestamp = Utc::now()
            .checked_sub_signed(chrono::Duration::hours(hours_ago))
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(0).unwrap());

        let content = format!(
            r#"# Acknowledgment: {finding_id}

**Finding ID:** {finding_id}
**Gate:** test-gate
**Severity:** WARN
**Acknowledged by:** test-user
**Acknowledged at:** {timestamp}

## Notes

Test notes
"#,
            timestamp = timestamp.to_rfc3339()
        );

        let file_path = week_dir.join(format!("{finding_id}.md"));
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_sla_hours_constant() {
        assert_eq!(SLA_HOURS, 72);
    }

    #[test]
    fn test_detect_sla_breach_unacked() {
        let temp_dir = TempDir::new().unwrap();
        let fixtures_dir = temp_dir.path().join("fixtures");
        let acks_dir = temp_dir.path().join("acks");
        fs::create_dir_all(&fixtures_dir).unwrap();
        fs::create_dir_all(&acks_dir).unwrap();

        // Create a WARN finding from 100 hours ago (beyond 72h SLA)
        create_warn_fixture(&fixtures_dir, "old-finding", "test-gate", 100);

        let breaches = detect_sla_breaches(&fixtures_dir, &acks_dir);

        assert_eq!(breaches.len(), 1);
        assert_eq!(breaches[0].finding_id, "old-finding");
        assert!(!breaches[0].late_acked); // Not acked at all
        assert!(breaches[0].hours_overdue > 0);
    }

    #[test]
    fn test_detect_sla_breach_late_acked() {
        let temp_dir = TempDir::new().unwrap();
        let fixtures_dir = temp_dir.path().join("fixtures");
        let acks_dir = temp_dir.path().join("acks");
        fs::create_dir_all(&fixtures_dir).unwrap();

        // Create a WARN finding from 100 hours ago
        create_warn_fixture(&fixtures_dir, "late-finding", "test-gate", 100);

        // Create an ack from 20 hours ago (80 hours after finding = 8h late)
        create_ack_for_finding(&acks_dir, "late-finding", 20);

        let breaches = detect_sla_breaches(&fixtures_dir, &acks_dir);

        assert_eq!(breaches.len(), 1);
        assert_eq!(breaches[0].finding_id, "late-finding");
        assert!(breaches[0].late_acked); // Acked, but late
        assert!(breaches[0].acked_at.is_some());
    }

    #[test]
    fn test_no_breach_timely_ack() {
        let temp_dir = TempDir::new().unwrap();
        let fixtures_dir = temp_dir.path().join("fixtures");
        let acks_dir = temp_dir.path().join("acks");
        fs::create_dir_all(&fixtures_dir).unwrap();

        // Create a WARN finding from 50 hours ago
        create_warn_fixture(&fixtures_dir, "timely-finding", "test-gate", 50);

        // Create an ack from 48 hours ago (within 72h SLA)
        create_ack_for_finding(&acks_dir, "timely-finding", 48);

        let breaches = detect_sla_breaches(&fixtures_dir, &acks_dir);

        assert!(breaches.is_empty(), "Should not have breach for timely ack");
    }

    #[test]
    fn test_format_sla_breaches_unacked() {
        let breaches = vec![SlaBreach {
            finding_id: "test-finding-001".to_string(),
            gate_id: "test-gate".to_string(),
            severity: "WARN".to_string(),
            finding_timestamp: Utc::now()
                .with_timezone(&FixedOffset::east_opt(0).unwrap()),
            hours_overdue: 28, // 28 hours past the 72h SLA
            late_acked: false,
            acked_at: None,
        }];

        let output = format_sla_breaches(&breaches);

        assert!(output.contains("**SLA BREACH**"));
        assert!(output.contains("test-finding-001"));
        assert!(output.contains("unacknowledged"));
    }

    #[test]
    fn test_format_sla_breaches_late_ack() {
        let acked_at = DateTime::parse_from_rfc3339("2026-01-28T10:00:00+00:00").unwrap();

        let breaches = vec![SlaBreach {
            finding_id: "test-finding-002".to_string(),
            gate_id: "test-gate".to_string(),
            severity: "WARN".to_string(),
            finding_timestamp: Utc::now()
                .with_timezone(&FixedOffset::east_opt(0).unwrap()),
            hours_overdue: 8, // 8 hours past the 72h SLA
            late_acked: true,
            acked_at: Some(acked_at),
        }];

        let output = format_sla_breaches(&breaches);

        assert!(output.contains("**LATE ACK**"));
        assert!(output.contains("test-finding-002"));
        assert!(output.contains("8h late"));
    }

    #[test]
    fn test_format_empty_breaches() {
        let breaches: Vec<SlaBreach> = vec![];
        let output = format_sla_breaches(&breaches);

        assert_eq!(output, "No SLA breaches detected.");
    }
}
