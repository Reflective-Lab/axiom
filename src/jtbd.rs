// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Jobs To Be Done (JTBD) parsing for Converge Truths.
//!
//! This module provides parsing and validation of JTBD metadata blocks
//! embedded in `.truths` files as structured comments.
//!
//! # JTBD Format
//!
//! JTBD metadata is embedded as comment blocks using a consistent tag header.
//! We support two formats:
//!
//! ## YAML-in-comments (recommended)
//!
//! ```gherkin
//! Truth: Invoice issued after work and collected on time
//!
//!   # JTBD:
//!   #   actor: Founder
//!   #   job_functional: "Invoice customers and collect payment"
//!   #   job_emotional: "Feel confident that every invoice gets sent"
//!   #   job_relational: "Be seen as professional and reliable"
//!   #   so_that: "Cash flows predictably"
//! ```
//!
//! ## Plain structured text (human-friendly)
//!
//! ```gherkin
//! Truth: Invoice issued after work and collected on time
//!
//!   # JTBD
//!   # As: Founder
//!   # Functional: Invoice customers and collect payment
//!   # Emotional: Feel confident that every invoice gets sent
//!   # Relational: Be seen as professional and reliable
//!   # So that: Cash flows predictably
//! ```

use serde::{Deserialize, Serialize};

/// JTBD metadata extracted from a Truth or Scenario.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JTBDMetadata {
    /// Role performing the job (required).
    pub actor: String,
    /// Functional job statement (required).
    pub job_functional: String,
    /// Emotional job statement (recommended).
    pub job_emotional: Option<String>,
    /// Relational job statement (recommended).
    pub job_relational: Option<String>,
    /// Outcome intent (required).
    pub so_that: String,
    /// Scope information.
    pub scope: Option<Scope>,
    /// Success metrics.
    pub success_metrics: Vec<SuccessMetric>,
    /// Failure modes.
    pub failure_modes: Vec<String>,
    /// Exceptions or edge cases.
    pub exceptions: Vec<String>,
    /// Evidence required.
    pub evidence_required: Vec<String>,
    /// Audit requirements.
    pub audit_requirements: Vec<String>,
    /// Links and references.
    pub links: Vec<Link>,
}

/// Scope information for JTBD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scope {
    /// Pack name.
    pub pack: Option<String>,
    /// Business segment.
    pub segment: Option<String>,
    /// Canonical objects involved.
    pub objects: Vec<String>,
}

/// Success metric definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessMetric {
    /// Unique metric ID.
    pub id: String,
    /// Target value (e.g., "<= 0.05", ">= 0.95").
    pub target: String,
    /// Time window (e.g., "72h", "30d").
    pub window: String,
    /// Dimension: "functional" | "emotional" | "relational".
    pub dimension: Option<String>,
}

/// Link or reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// URL or internal reference.
    pub url: Option<String>,
    /// Reference (e.g., "@`invariant:closed_period_readonly`").
    pub ref_: Option<String>,
    /// Label for the link.
    pub label: Option<String>,
}

/// Error types for JTBD parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum JTBDError {
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Extracts JTBD metadata from a `.truths` file content.
///
/// Returns file-level JTBD if found, and scenario-level JTBD for each scenario.
///
/// # Example
///
/// ```rust,no_run
/// use axiom_truth::jtbd::extract_jtbd;
///
/// let content = r#"
/// Truth: Example
///   # JTBD:
///   #   actor: Founder
///   #   job_functional: "Do something"
///   #   so_that: "Achieve outcome"
/// "#;
///
/// let (file_jtbd, scenario_jtbds) = extract_jtbd(content).unwrap();
/// ```
pub fn extract_jtbd(content: &str) -> Result<(Option<JTBDMetadata>, Vec<JTBDMetadata>), JTBDError> {
    let mut file_jtbd = None;
    let mut scenario_jtbds = Vec::new();

    // Split content into lines for processing
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        // Look for file-level JTBD (after "Truth:" or "Feature:")
        if (lines[i].trim().starts_with("Truth:") || lines[i].trim().starts_with("Feature:"))
            && let Some((jtbd, next_i)) = parse_jtbd_block(&lines, i + 1)?
        {
            file_jtbd = Some(jtbd);
            i = next_i;
            continue;
        }

        // Look for scenario-level JTBD (after "Scenario:")
        if lines[i].trim().starts_with("Scenario:")
            && let Some((jtbd, next_i)) = parse_jtbd_block(&lines, i + 1)?
        {
            scenario_jtbds.push(jtbd);
            i = next_i;
            continue;
        }

        i += 1;
    }

    Ok((file_jtbd, scenario_jtbds))
}

/// Parses a JTBD block starting at the given line index.
///
/// Returns `Some((JTBDMetadata, next_line_index))` if a JTBD block is found,
/// or `None` if no block is found.
fn parse_jtbd_block(
    lines: &[&str],
    start: usize,
) -> Result<Option<(JTBDMetadata, usize)>, JTBDError> {
    if start >= lines.len() {
        return Ok(None);
    }

    // Skip blank lines before the JTBD block
    let mut start = start;
    while start < lines.len() && lines[start].trim().is_empty() {
        start += 1;
    }
    if start >= lines.len() {
        return Ok(None);
    }

    // Check if next line starts with "# JTBD:" (YAML format) or "# JTBD" (plain text)
    let first_line = lines[start].trim();
    if !first_line.starts_with("# JTBD") {
        return Ok(None);
    }

    // Collect all comment lines in the JTBD block
    let mut jtbd_lines = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let line = lines[i].trim();

        // Stop if we hit a non-comment line (that's not empty)
        if !line.is_empty() && !line.starts_with('#') {
            break;
        }

        // Stop if we hit an empty line after the block has started
        if line.is_empty() && !jtbd_lines.is_empty() {
            // Check if next non-empty line is still part of JTBD
            let mut peek = i + 1;
            while peek < lines.len() && lines[peek].trim().is_empty() {
                peek += 1;
            }
            if peek < lines.len() && !lines[peek].trim().starts_with('#') {
                break;
            }
        }

        if line.starts_with('#') {
            jtbd_lines.push(line);
        }

        i += 1;
    }

    if jtbd_lines.is_empty() {
        return Ok(None);
    }

    // Try to parse as YAML first, then fall back to plain text
    let jtbd = if jtbd_lines[0].contains("JTBD:") {
        parse_yaml_jtbd(&jtbd_lines)?
    } else {
        parse_plain_text_jtbd(&jtbd_lines)?
    };

    Ok(Some((jtbd, i)))
}

/// Parses YAML-format JTBD block.
fn parse_yaml_jtbd(lines: &[&str]) -> Result<JTBDMetadata, JTBDError> {
    // Remove "# " prefix and "JTBD:" header
    let yaml_lines: Vec<String> = lines
        .iter()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed == "# JTBD:" {
                "jtbd:".to_string()
            } else if let Some(rest) = trimmed.strip_prefix("# ") {
                rest.to_string()
            } else if let Some(rest) = trimmed.strip_prefix('#') {
                rest.to_string()
            } else {
                trimmed.to_string()
            }
        })
        .collect();

    let yaml_content = yaml_lines.join("\n");

    // Parse YAML
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(&yaml_content).map_err(|e| JTBDError::InvalidYaml(format!("{e}")))?;

    // Extract fields
    let actor = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("actor"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| JTBDError::MissingRequiredField("actor".to_string()))?
        .to_string();

    let job_functional = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("job_functional"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| JTBDError::MissingRequiredField("job_functional".to_string()))?
        .to_string();

    let job_emotional = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("job_emotional"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let job_relational = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("job_relational"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let so_that = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("so_that"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| JTBDError::MissingRequiredField("so_that".to_string()))?
        .to_string();

    // Parse scope
    let scope = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("scope"))
        .map(|s| {
            let pack = s.get("pack").and_then(|v| v.as_str()).map(String::from);
            let segment = s.get("segment").and_then(|v| v.as_str()).map(String::from);
            let objects = s
                .get("objects")
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Scope {
                pack,
                segment,
                objects,
            }
        });

    // Parse success metrics
    let success_metrics = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("success_metrics"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|m| {
                    Some(SuccessMetric {
                        id: m.get("id")?.as_str()?.to_string(),
                        target: m.get("target")?.as_str()?.to_string(),
                        window: m.get("window")?.as_str()?.to_string(),
                        dimension: m
                            .get("dimension")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse failure modes
    let failure_modes = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("failure_modes"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse exceptions
    let exceptions = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("exceptions"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse evidence_required
    let evidence_required = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("evidence_required"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse audit_requirements
    let audit_requirements = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("audit_requirements"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Parse links
    let links = yaml_value
        .get("jtbd")
        .and_then(|j| j.get("links"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .map(|l| Link {
                    url: l.get("url").and_then(|v| v.as_str()).map(String::from),
                    ref_: l.get("ref").and_then(|v| v.as_str()).map(String::from),
                    label: l.get("label").and_then(|v| v.as_str()).map(String::from),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(JTBDMetadata {
        actor,
        job_functional,
        job_emotional,
        job_relational,
        so_that,
        scope,
        success_metrics,
        failure_modes,
        exceptions,
        evidence_required,
        audit_requirements,
        links,
    })
}

/// Parses plain text format JTBD block.
fn parse_plain_text_jtbd(lines: &[&str]) -> Result<JTBDMetadata, JTBDError> {
    let mut actor = None;
    let mut job_functional = None;
    let mut job_emotional = None;
    let mut job_relational = None;
    let mut so_that = None;
    let mut scope_pack = None;
    let mut scope_segment = None;
    let mut scope_objects = Vec::new();
    let mut success_metrics = Vec::new();
    let mut failure_modes = Vec::new();
    let mut exceptions = Vec::new();
    let mut evidence_required = Vec::new();
    let mut audit_requirements = Vec::new();
    let links = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "# JTBD" {
            continue;
        }

        // Remove "# " prefix
        let content = if let Some(rest) = trimmed.strip_prefix("# ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix('#') {
            rest
        } else {
            continue;
        };

        // Parse key-value pairs
        if let Some((key, value)) = content.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().trim_matches('"').to_string();

            match key.as_str() {
                "as" => actor = Some(value),
                "functional" | "job_functional" => job_functional = Some(value),
                "emotional" | "job_emotional" => job_emotional = Some(value),
                "relational" | "job_relational" => job_relational = Some(value),
                "so that" | "so_that" => so_that = Some(value),
                _ => {
                    // Handle scope, metrics, etc. with more complex parsing
                    if key == "scope" {
                        // Parse "pack.segment [object1, object2]"
                        if let Some((pack_seg, objects_str)) = value.split_once('[') {
                            let parts: Vec<&str> = pack_seg.split('.').collect();
                            if !parts.is_empty() {
                                scope_pack = Some(parts[0].trim().to_string());
                            }
                            if parts.len() >= 2 {
                                scope_segment = Some(parts[1].trim().to_string());
                            }
                            if let Some(objs) = objects_str.strip_suffix(']') {
                                scope_objects =
                                    objs.split(',').map(|s| s.trim().to_string()).collect();
                            }
                        }
                    } else if key.starts_with("metric") || key.contains("metric") {
                        // Simple metric parsing (can be enhanced)
                        // Format: "Metric ID <= 0.05 (functional)"
                        if let Some((id_target, dim)) = value.rsplit_once('(') {
                            let dimension = dim.trim().trim_end_matches(')').to_string();
                            // Parse id and target (simplified)
                            if let Some((id, target)) = id_target.split_once(' ') {
                                success_metrics.push(SuccessMetric {
                                    id: id.trim().to_string(),
                                    target: target.trim().to_string(),
                                    window: "unknown".to_string(),
                                    dimension: Some(dimension),
                                });
                            }
                        }
                    } else if key == "failure mode" || key == "failure_mode" {
                        failure_modes.push(value);
                    } else if key == "exception" {
                        exceptions.push(value);
                    } else if key == "evidence" {
                        evidence_required.push(value);
                    } else if key == "audit" {
                        audit_requirements.push(value);
                    }
                }
            }
        }
    }

    Ok(JTBDMetadata {
        actor: actor.ok_or_else(|| JTBDError::MissingRequiredField("actor".to_string()))?,
        job_functional: job_functional
            .ok_or_else(|| JTBDError::MissingRequiredField("job_functional".to_string()))?,
        job_emotional,
        job_relational,
        so_that: so_that.ok_or_else(|| JTBDError::MissingRequiredField("so_that".to_string()))?,
        scope: if scope_pack.is_some() || scope_segment.is_some() || !scope_objects.is_empty() {
            Some(Scope {
                pack: scope_pack,
                segment: scope_segment,
                objects: scope_objects,
            })
        } else {
            None
        },
        success_metrics,
        failure_modes,
        exceptions,
        evidence_required,
        audit_requirements,
        links,
    })
}

/// Validates JTBD metadata for completeness.
///
/// Returns a list of validation issues (warnings for missing recommended fields,
/// errors for missing required fields).
pub fn validate_jtbd(jtbd: &JTBDMetadata, strict: bool) -> Vec<JTBDValidationIssue> {
    let mut issues = Vec::new();

    // Required fields are already validated during parsing
    // Check recommended fields
    if jtbd.job_emotional.is_none() {
        issues.push(JTBDValidationIssue {
            field: "job_emotional".to_string(),
            severity: if strict {
                ValidationSeverity::Error
            } else {
                ValidationSeverity::Warning
            },
            message: "Missing recommended field: job_emotional".to_string(),
        });
    }

    if jtbd.job_relational.is_none() {
        issues.push(JTBDValidationIssue {
            field: "job_relational".to_string(),
            severity: if strict {
                ValidationSeverity::Error
            } else {
                ValidationSeverity::Warning
            },
            message: "Missing recommended field: job_relational".to_string(),
        });
    }

    // Validate success metrics
    let metric_ids: Vec<&str> = jtbd.success_metrics.iter().map(|m| m.id.as_str()).collect();
    let unique_ids: std::collections::HashSet<&str> = metric_ids.iter().copied().collect();
    if metric_ids.len() != unique_ids.len() {
        issues.push(JTBDValidationIssue {
            field: "success_metrics".to_string(),
            severity: ValidationSeverity::Error,
            message: "Duplicate success metric IDs found".to_string(),
        });
    }

    issues
}

/// Validation issue for JTBD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JTBDValidationIssue {
    /// Field that has the issue.
    pub field: String,
    /// Severity level.
    pub severity: ValidationSeverity,
    /// Human-readable message.
    pub message: String,
}

/// Severity of a JTBD validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Warning - recommended but not required.
    Warning,
    /// Error - must be fixed.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_jtbd() {
        let content = r#"
Truth: Invoice issued after work and collected on time

  # JTBD:
  #   actor: Founder
  #   job_functional: "Invoice customers and collect payment"
  #   job_emotional: "Feel confident that every invoice gets sent"
  #   so_that: "Cash flows predictably"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Founder");
        assert_eq!(jtbd.job_functional, "Invoice customers and collect payment");
        assert_eq!(
            jtbd.job_emotional,
            Some("Feel confident that every invoice gets sent".to_string())
        );
        assert_eq!(jtbd.so_that, "Cash flows predictably");
    }

    #[test]
    fn test_parse_plain_text_jtbd() {
        let content = r#"
Truth: Example

  # JTBD
  # As: Founder
  # Functional: Invoice customers
  # So that: Cash flows predictably
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Founder");
        assert_eq!(jtbd.job_functional, "Invoice customers");
        assert_eq!(jtbd.so_that, "Cash flows predictably");
    }

    #[test]
    fn test_validate_jtbd() {
        let jtbd = JTBDMetadata {
            actor: "Founder".to_string(),
            job_functional: "Do something".to_string(),
            job_emotional: None,
            job_relational: None,
            so_that: "Achieve outcome".to_string(),
            scope: None,
            success_metrics: Vec::new(),
            failure_modes: Vec::new(),
            exceptions: Vec::new(),
            evidence_required: Vec::new(),
            audit_requirements: Vec::new(),
            links: Vec::new(),
        };

        let issues = validate_jtbd(&jtbd, false);
        assert_eq!(issues.len(), 2); // Missing job_emotional and job_relational
        assert!(
            issues
                .iter()
                .all(|i| i.severity == ValidationSeverity::Warning)
        );
    }
}
