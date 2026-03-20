//! Acknowledgment and Escalation File Operations
//!
//! Provides functions for creating acknowledgment and escalation files
//! in response to governance alerts.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{Datelike, Local, Utc};

/// Metadata for acknowledgment or escalation records
#[derive(Debug, Clone)]
pub struct AckMetadata {
    /// The finding ID being acknowledged/escalated
    pub finding_id: String,
    /// Gate ID extracted from `finding_id` pattern
    pub gate_id: String,
    /// Severity of the finding
    pub severity: String,
    /// Person acknowledging (from USER env or git config)
    pub acknowledged_by: String,
    /// ISO 8601 timestamp of acknowledgment
    pub acknowledged_at: String,
    /// Required notes explaining the acknowledgment
    pub notes: String,
    /// Whether this is a late acknowledgment (>72h since finding)
    pub late_ack: bool,
}

/// Errors that can occur during ack/escalation operations
#[derive(Debug)]
pub enum AckError {
    /// IO error during file operations
    IoError(io::Error),
    /// Invalid or malformed finding ID
    InvalidFindingId(String),
}

impl fmt::Display for AckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO error: {e}"),
            Self::InvalidFindingId(id) => write!(f, "Invalid finding ID: {id}"),
        }
    }
}

impl std::error::Error for AckError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::InvalidFindingId(_) => None,
        }
    }
}

impl From<io::Error> for AckError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

/// Get the current user's name from environment or git config
pub fn get_user_name() -> String {
    // Try USER env var first
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            return user;
        }
    }

    // Fall back to git config user.name
    if let Ok(output) = Command::new("git")
        .args(["config", "user.name"])
        .output()
    {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }

    // Default if neither available
    "unknown".to_string()
}

/// Get the current ISO week identifier (YYYY-WW format)
fn get_current_week() -> String {
    let now = Local::now();
    let week = now.iso_week();
    format!("{:04}-W{:02}", week.year(), week.week())
}

/// Create an acknowledgment file for a finding
///
/// # Arguments
/// * `finding_id` - The ID of the finding being acknowledged
/// * `notes` - Required notes explaining the acknowledgment
/// * `output_dir` - Output directory (default: reports/acks)
/// * `late_ack` - Whether this is a late acknowledgment
///
/// # Returns
/// Path to the created acknowledgment file
pub fn create_ack_file(
    finding_id: &str,
    notes: &str,
    output_dir: Option<&Path>,
    late_ack: bool,
) -> Result<PathBuf, AckError> {
    // Validate finding_id (basic validation - non-empty)
    if finding_id.trim().is_empty() {
        return Err(AckError::InvalidFindingId("finding_id cannot be empty".to_string()));
    }

    // Extract gate_id from finding_id (assumes format like "gate-name-check-id")
    let gate_id = extract_gate_id(finding_id);

    // Build metadata
    let metadata = AckMetadata {
        finding_id: finding_id.to_string(),
        gate_id,
        severity: "WARN".to_string(), // Default severity for acks
        acknowledged_by: get_user_name(),
        acknowledged_at: Utc::now().to_rfc3339(),
        notes: notes.to_string(),
        late_ack,
    };

    // Determine output directory
    let base_dir = output_dir.map_or_else(
        || PathBuf::from("reports/acks"),
        PathBuf::from,
    );

    // Create week subdirectory
    let week = get_current_week();
    let week_dir = base_dir.join(&week);
    fs::create_dir_all(&week_dir)?;

    // Create file path
    let file_path = week_dir.join(format!("{finding_id}.md"));

    // Generate markdown content
    let content = generate_ack_markdown(&metadata);

    // Write file
    fs::write(&file_path, content)?;

    Ok(file_path)
}

/// Create an escalation file for a finding
///
/// # Arguments
/// * `finding_id` - The ID of the finding being escalated
/// * `notes` - Required notes explaining the escalation
/// * `output_dir` - Output directory (default: reports/escalations)
///
/// # Returns
/// Path to the created escalation file
pub fn create_escalation_file(
    finding_id: &str,
    notes: &str,
    output_dir: Option<&Path>,
) -> Result<PathBuf, AckError> {
    // Validate finding_id (basic validation - non-empty)
    if finding_id.trim().is_empty() {
        return Err(AckError::InvalidFindingId("finding_id cannot be empty".to_string()));
    }

    // Extract gate_id from finding_id
    let gate_id = extract_gate_id(finding_id);

    // Build metadata
    let metadata = AckMetadata {
        finding_id: finding_id.to_string(),
        gate_id,
        severity: "STOP".to_string(), // Escalations typically for STOP findings
        acknowledged_by: get_user_name(),
        acknowledged_at: Utc::now().to_rfc3339(),
        notes: notes.to_string(),
        late_ack: false, // Escalations don't have late marker
    };

    // Determine output directory (no week subdirectory for escalations)
    let base_dir = output_dir.map_or_else(
        || PathBuf::from("reports/escalations"),
        PathBuf::from,
    );
    fs::create_dir_all(&base_dir)?;

    // Create file path (directly in escalations, not week-organized)
    let file_path = base_dir.join(format!("{finding_id}.md"));

    // Generate markdown content
    let content = generate_escalation_markdown(&metadata);

    // Write file
    fs::write(&file_path, content)?;

    Ok(file_path)
}

/// Extract `gate_id` from `finding_id` pattern
/// Assumes format like "gate-name-check-id" or just returns the full ID if no pattern matches
fn extract_gate_id(finding_id: &str) -> String {
    // Try to extract gate portion (everything before last hyphen-separated segment)
    let parts: Vec<&str> = finding_id.split('-').collect();
    if parts.len() >= 2 {
        // Return all but the last segment as the gate_id
        parts[..parts.len() - 1].join("-")
    } else {
        finding_id.to_string()
    }
}

/// Generate markdown content for an acknowledgment file
fn generate_ack_markdown(metadata: &AckMetadata) -> String {
    let late_marker = if metadata.late_ack {
        "\n> **LATE ACKNOWLEDGMENT** - This acknowledgment was made more than 72 hours after the finding.\n"
    } else {
        ""
    };

    format!(
        r"# Acknowledgment: {finding_id}

**Finding ID:** {finding_id}
**Gate:** {gate_id}
**Severity:** {severity}
**Acknowledged by:** {acknowledged_by}
**Acknowledged at:** {acknowledged_at}
{late_marker}
## Notes

{notes}

## Status

- [x] Finding reviewed
- [ ] Remediation planned
- [ ] Remediation complete
",
        finding_id = metadata.finding_id,
        gate_id = metadata.gate_id,
        severity = metadata.severity,
        acknowledged_by = metadata.acknowledged_by,
        acknowledged_at = metadata.acknowledged_at,
        late_marker = late_marker,
        notes = metadata.notes
    )
}

/// Generate markdown content for an escalation file
fn generate_escalation_markdown(metadata: &AckMetadata) -> String {
    format!(
        r"# Escalation: {finding_id}

**Finding ID:** {finding_id}
**Gate:** {gate_id}
**Severity:** {severity}
**Escalated by:** {escalated_by}
**Escalated at:** {escalated_at}

## Escalation Notes

{notes}

## Escalation Details

### Impact Assessment

_TODO: Describe the impact of this finding_

### Proposed Resolution

_TODO: Outline the proposed resolution_

### Timeline

_TODO: Provide estimated timeline for resolution_

## Follow-up Actions

- [ ] Stakeholders notified
- [ ] Root cause analysis initiated
- [ ] Remediation plan created
- [ ] Remediation complete
- [ ] Post-mortem scheduled
",
        finding_id = metadata.finding_id,
        gate_id = metadata.gate_id,
        severity = metadata.severity,
        escalated_by = metadata.acknowledged_by,
        escalated_at = metadata.acknowledged_at,
        notes = metadata.notes
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_ack_file_creates_directory_and_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("acks");

        let result = create_ack_file(
            "test-gate-check-001",
            "Test acknowledgment notes",
            Some(&output_dir),
            false,
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert!(file_path.exists());
        assert!(file_path.to_string_lossy().contains("test-gate-check-001.md"));
    }

    #[test]
    fn test_create_ack_file_week_directory() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("acks");

        let result = create_ack_file(
            "test-finding",
            "Notes here",
            Some(&output_dir),
            false,
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();

        // Check that parent is a week directory (YYYY-WXX format)
        let parent = file_path.parent().unwrap();
        let parent_name = parent.file_name().unwrap().to_string_lossy();
        assert!(parent_name.contains("-W"), "Parent directory should be week format: {parent_name}");
    }

    #[test]
    fn test_create_escalation_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("escalations");

        let result = create_escalation_file(
            "critical-gate-stop-001",
            "This requires immediate attention",
            Some(&output_dir),
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();
        assert!(file_path.exists());

        // Escalations should be directly in output_dir, not in week subdirectory
        assert_eq!(file_path.parent().unwrap(), output_dir);

        // Check content includes escalation sections
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Escalation:"));
        assert!(content.contains("## Escalation Details"));
    }

    #[test]
    fn test_get_user_name_returns_string() {
        // Since env::set_var is unsafe in Rust 2024, we just verify get_user_name returns
        // a non-empty string (either from USER env, git config, or default "unknown")
        let user = get_user_name();
        assert!(!user.is_empty(), "get_user_name should return a non-empty string");
    }

    #[test]
    fn test_ack_file_markdown_format() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("acks");

        let result = create_ack_file(
            "my-gate-check-123",
            "These are my notes",
            Some(&output_dir),
            false,
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();

        // Check markdown structure
        assert!(content.contains("# Acknowledgment: my-gate-check-123"));
        assert!(content.contains("**Finding ID:** my-gate-check-123"));
        assert!(content.contains("**Gate:** my-gate-check"));
        assert!(content.contains("## Notes"));
        assert!(content.contains("These are my notes"));
        assert!(content.contains("## Status"));
    }

    #[test]
    fn test_late_ack_marker() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("acks");

        // Create a late acknowledgment
        let result = create_ack_file(
            "late-finding-001",
            "Late ack notes",
            Some(&output_dir),
            true, // late_ack = true
        );

        assert!(result.is_ok());
        let file_path = result.unwrap();
        let content = fs::read_to_string(&file_path).unwrap();

        // Check for late acknowledgment marker
        assert!(content.contains("**LATE ACKNOWLEDGMENT**"));
        assert!(content.contains("more than 72 hours"));
    }

    #[test]
    fn test_invalid_finding_id_empty() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("acks");

        let result = create_ack_file("", "Notes", Some(&output_dir), false);
        assert!(result.is_err());

        if let Err(AckError::InvalidFindingId(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected InvalidFindingId error");
        }
    }

    #[test]
    fn test_extract_gate_id() {
        assert_eq!(extract_gate_id("my-gate-check-123"), "my-gate-check");
        assert_eq!(extract_gate_id("simple-check"), "simple");
        assert_eq!(extract_gate_id("nodelimiter"), "nodelimiter");
    }
}
