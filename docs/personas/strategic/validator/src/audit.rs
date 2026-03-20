//! Audit Trail Query Module
//!
//! Provides functions for querying acknowledgment history and building
//! audit reports for governance compliance.

use std::fmt::{self, Write as FmtWrite};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset};
use regex::Regex;
use walkdir::WalkDir;

/// A parsed acknowledgment record from an ack file
#[derive(Debug, Clone)]
pub struct AckRecord {
    /// The finding ID that was acknowledged
    pub finding_id: String,
    /// The gate ID extracted from the finding
    pub gate_id: String,
    /// Severity of the finding (WARN, STOP, etc.)
    pub severity: String,
    /// Who acknowledged the finding
    pub acknowledged_by: String,
    /// When the acknowledgment was made
    pub acknowledged_at: DateTime<FixedOffset>,
    /// Notes provided with the acknowledgment
    pub notes: String,
    /// Whether this was a late acknowledgment (>72h)
    pub late_ack: bool,
    /// Path to the source ack file
    pub file_path: PathBuf,
}

/// Filter criteria for querying acknowledgments
#[derive(Debug, Default, Clone)]
pub struct AuditFilter {
    /// Filter by exact finding ID match
    pub finding_id: Option<String>,
    /// Filter by gate ID prefix match
    pub gate_id: Option<String>,
    /// Filter by week (YYYY-WW format, matches directory name)
    pub week: Option<String>,
}

/// Errors that can occur during audit operations
#[derive(Debug)]
pub enum AuditError {
    /// IO error reading files
    IoError(io::Error),
    /// Error parsing an ack file
    ParseError { path: PathBuf, reason: String },
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::ParseError { path, reason } => {
                write!(f, "Parse error in {}: {reason}", path.display())
            }
        }
    }
}

impl std::error::Error for AuditError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::ParseError { .. } => None,
        }
    }
}

impl From<io::Error> for AuditError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Parse a single ack file into an `AckRecord`
pub fn parse_ack_file(path: &Path) -> Result<AckRecord, AuditError> {
    let content = fs::read_to_string(path)?;

    // Extract finding_id from header: "# Acknowledgment: {finding_id}"
    let finding_id = extract_field(&content, r"# Acknowledgment: (.+)")
        .or_else(|| extract_field(&content, r"\*\*Finding ID:\*\* (.+)"))
        .ok_or_else(|| AuditError::ParseError {
            path: path.to_path_buf(),
            reason: "Could not extract finding_id".to_string(),
        })?;

    // Extract gate_id
    let gate_id = extract_field(&content, r"\*\*Gate:\*\* (.+)")
        .unwrap_or_else(|| extract_gate_from_finding(&finding_id));

    // Extract severity
    let severity = extract_field(&content, r"\*\*Severity:\*\* (.+)")
        .unwrap_or_else(|| "WARN".to_string());

    // Extract acknowledged_by
    let acknowledged_by = extract_field(&content, r"\*\*Acknowledged by:\*\* (.+)")
        .ok_or_else(|| AuditError::ParseError {
            path: path.to_path_buf(),
            reason: "Could not extract acknowledged_by".to_string(),
        })?;

    // Extract acknowledged_at timestamp
    let acknowledged_at_str = extract_field(&content, r"\*\*Acknowledged at:\*\* (.+)")
        .ok_or_else(|| AuditError::ParseError {
            path: path.to_path_buf(),
            reason: "Could not extract acknowledged_at".to_string(),
        })?;

    let acknowledged_at = DateTime::parse_from_rfc3339(&acknowledged_at_str)
        .map_err(|e| AuditError::ParseError {
            path: path.to_path_buf(),
            reason: format!("Invalid timestamp '{acknowledged_at_str}': {e}"),
        })?;

    // Extract notes from "## Notes" section
    let notes = extract_notes_section(&content).unwrap_or_default();

    // Check for late acknowledgment marker
    let late_ack = content.contains("**LATE ACKNOWLEDGMENT**")
        || content.contains("Late acknowledgment: Yes");

    Ok(AckRecord {
        finding_id,
        gate_id,
        severity,
        acknowledged_by,
        acknowledged_at,
        notes,
        late_ack,
        file_path: path.to_path_buf(),
    })
}

/// Query acknowledgments from a directory with optional filtering
pub fn query_acks(acks_dir: &Path, filter: &AuditFilter) -> Vec<AckRecord> {
    let mut records = Vec::new();

    if !acks_dir.exists() {
        return records;
    }

    for entry in WalkDir::new(acks_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
    {
        let path = entry.path();

        // Apply week filter based on parent directory name
        if let Some(ref week) = filter.week {
            if let Some(parent) = path.parent() {
                if let Some(parent_name) = parent.file_name() {
                    if !parent_name.to_string_lossy().contains(week) {
                        continue;
                    }
                }
            }
        }

        // Parse the ack file
        let Ok(record) = parse_ack_file(path) else {
            continue; // Skip files that can't be parsed
        };

        // Apply finding_id filter (exact match)
        if let Some(ref finding_id) = filter.finding_id {
            if record.finding_id != *finding_id {
                continue;
            }
        }

        // Apply gate_id filter (prefix match)
        if let Some(ref gate_id) = filter.gate_id {
            if !record.finding_id.starts_with(gate_id) && !record.gate_id.starts_with(gate_id) {
                continue;
            }
        }

        records.push(record);
    }

    // Sort by acknowledged_at descending (most recent first)
    records.sort_by(|a, b| b.acknowledged_at.cmp(&a.acknowledged_at));

    records
}

/// Format acknowledgment records as a human-readable table
pub fn format_audit_table(records: &[AckRecord]) -> String {
    if records.is_empty() {
        return "No acknowledgments found".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str("Finding ID                      | Acknowledged By | When                     | Late | Notes\n");
    output.push_str("--------------------------------|-----------------|--------------------------|------|----------------------------------------------\n");

    for record in records {
        // Truncate finding_id if too long
        let finding_id = if record.finding_id.len() > 30 {
            format!("{}...", &record.finding_id[..27])
        } else {
            format!("{:<30}", record.finding_id)
        };

        // Truncate ack_by if too long
        let ack_by = if record.acknowledged_by.len() > 15 {
            format!("{}...", &record.acknowledged_by[..12])
        } else {
            format!("{:<15}", record.acknowledged_by)
        };

        // Format timestamp
        let when = record.acknowledged_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Late indicator
        let late = if record.late_ack { "Yes " } else { "No  " };

        // Truncate notes
        let notes = truncate_notes(&record.notes, 40);

        let _ = writeln!(output, "{finding_id} | {ack_by} | {when} | {late} | {notes}");
    }

    output
}

// Helper functions

fn extract_field(content: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
}

fn extract_notes_section(content: &str) -> Option<String> {
    // Find "## Notes" section and extract content until next heading or end
    let notes_marker = "## Notes";
    if let Some(start_idx) = content.find(notes_marker) {
        let after_marker = &content[start_idx + notes_marker.len()..];

        // Find the next ## heading or end of content
        let end_idx = after_marker
            .find("\n## ")
            .or_else(|| after_marker.find("\n---"))
            .unwrap_or(after_marker.len());

        let notes = after_marker[..end_idx].trim();
        if !notes.is_empty() {
            return Some(notes.to_string());
        }
    }
    None
}

fn extract_gate_from_finding(finding_id: &str) -> String {
    // Extract gate from finding_id by taking all but the last segment
    let parts: Vec<&str> = finding_id.split('-').collect();
    if parts.len() >= 2 {
        parts[..parts.len() - 1].join("-")
    } else {
        finding_id.to_string()
    }
}

fn truncate_notes(notes: &str, max_len: usize) -> String {
    // Get first line and truncate
    let first_line = notes.lines().next().unwrap_or("");
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len - 3])
    } else {
        first_line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_ack_file(dir: &Path, week: &str, finding_id: &str, late: bool) -> PathBuf {
        let week_dir = dir.join(week);
        fs::create_dir_all(&week_dir).unwrap();

        let file_path = week_dir.join(format!("{finding_id}.md"));

        let late_marker = if late {
            "\n> **LATE ACKNOWLEDGMENT** - This acknowledgment was made more than 72 hours after the finding.\n"
        } else {
            ""
        };

        let content = format!(
            r#"# Acknowledgment: {finding_id}

**Finding ID:** {finding_id}
**Gate:** test-gate
**Severity:** WARN
**Acknowledged by:** test-user
**Acknowledged at:** 2026-01-28T10:00:00+00:00
{late_marker}
## Notes

Test acknowledgment notes for {finding_id}

## Status

- [x] Finding reviewed
"#
        );

        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_parse_ack_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_ack_file(temp_dir.path(), "2026-W05", "test-finding-001", false);

        let record = parse_ack_file(&file_path).unwrap();

        assert_eq!(record.finding_id, "test-finding-001");
        assert_eq!(record.gate_id, "test-gate");
        assert_eq!(record.severity, "WARN");
        assert_eq!(record.acknowledged_by, "test-user");
        assert!(!record.late_ack);
        assert!(record.notes.contains("Test acknowledgment notes"));
    }

    #[test]
    fn test_query_acks_finding_filter() {
        let temp_dir = TempDir::new().unwrap();
        create_test_ack_file(temp_dir.path(), "2026-W05", "gate-a-001", false);
        create_test_ack_file(temp_dir.path(), "2026-W05", "gate-b-002", false);

        let filter = AuditFilter {
            finding_id: Some("gate-a-001".to_string()),
            ..Default::default()
        };

        let records = query_acks(temp_dir.path(), &filter);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].finding_id, "gate-a-001");
    }

    #[test]
    fn test_query_acks_gate_filter() {
        let temp_dir = TempDir::new().unwrap();
        create_test_ack_file(temp_dir.path(), "2026-W05", "gate-a-001", false);
        create_test_ack_file(temp_dir.path(), "2026-W05", "gate-a-002", false);
        create_test_ack_file(temp_dir.path(), "2026-W05", "gate-b-001", false);

        let filter = AuditFilter {
            gate_id: Some("gate-a".to_string()),
            ..Default::default()
        };

        let records = query_acks(temp_dir.path(), &filter);

        assert_eq!(records.len(), 2);
        assert!(records.iter().all(|r| r.finding_id.starts_with("gate-a")));
    }

    #[test]
    fn test_query_acks_week_filter() {
        let temp_dir = TempDir::new().unwrap();
        create_test_ack_file(temp_dir.path(), "2026-W05", "finding-001", false);
        create_test_ack_file(temp_dir.path(), "2026-W06", "finding-002", false);

        let filter = AuditFilter {
            week: Some("2026-W05".to_string()),
            ..Default::default()
        };

        let records = query_acks(temp_dir.path(), &filter);

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].finding_id, "finding-001");
    }

    #[test]
    fn test_format_audit_table() {
        let records = vec![
            AckRecord {
                finding_id: "content-publish-001".to_string(),
                gate_id: "content-publish".to_string(),
                severity: "WARN".to_string(),
                acknowledged_by: "alice".to_string(),
                acknowledged_at: DateTime::parse_from_rfc3339("2026-01-28T10:00:00+00:00").unwrap(),
                notes: "Accepted risk for this release".to_string(),
                late_ack: false,
                file_path: PathBuf::from("test.md"),
            },
        ];

        let table = format_audit_table(&records);

        assert!(table.contains("Finding ID"));
        assert!(table.contains("content-publish-001"));
        assert!(table.contains("alice"));
        assert!(table.contains("No  ")); // Not late
    }

    #[test]
    fn test_empty_results_message() {
        let records: Vec<AckRecord> = vec![];
        let table = format_audit_table(&records);

        assert_eq!(table, "No acknowledgments found");
    }

    #[test]
    fn test_late_ack_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_ack_file(temp_dir.path(), "2026-W05", "late-finding", true);

        let record = parse_ack_file(&file_path).unwrap();

        assert!(record.late_ack);
    }
}
