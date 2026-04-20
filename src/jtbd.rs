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
    use proptest::prelude::*;

    fn minimal_jtbd() -> JTBDMetadata {
        JTBDMetadata {
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
        }
    }

    fn full_jtbd() -> JTBDMetadata {
        JTBDMetadata {
            actor: "CFO".to_string(),
            job_functional: "Close the books on time".to_string(),
            job_emotional: Some("Feel in control of financials".to_string()),
            job_relational: Some("Be trusted by the board".to_string()),
            so_that: "Quarterly reports are accurate".to_string(),
            scope: Some(Scope {
                pack: Some("finance".to_string()),
                segment: Some("enterprise".to_string()),
                objects: vec!["Invoice".to_string(), "Payment".to_string()],
            }),
            success_metrics: vec![
                SuccessMetric {
                    id: "close_time".to_string(),
                    target: "<= 3d".to_string(),
                    window: "30d".to_string(),
                    dimension: Some("functional".to_string()),
                },
                SuccessMetric {
                    id: "accuracy".to_string(),
                    target: ">= 0.99".to_string(),
                    window: "90d".to_string(),
                    dimension: Some("functional".to_string()),
                },
            ],
            failure_modes: vec!["Late close".to_string(), "Mismatched totals".to_string()],
            exceptions: vec!["Holiday periods".to_string()],
            evidence_required: vec!["Reconciliation report".to_string()],
            audit_requirements: vec!["SOX compliance".to_string()],
            links: vec![Link {
                url: Some("https://example.com/policy".to_string()),
                ref_: Some("@invariant:closed_period_readonly".to_string()),
                label: Some("Policy doc".to_string()),
            }],
        }
    }

    // ── YAML parsing ──

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
    fn test_yaml_all_12_fields() {
        let content = r#"
Truth: Full JTBD

  # JTBD:
  #   actor: CFO
  #   job_functional: "Close the books"
  #   job_emotional: "Feel in control"
  #   job_relational: "Be trusted by the board"
  #   so_that: "Reports are accurate"
  #   scope:
  #     pack: finance
  #     segment: enterprise
  #     objects:
  #       - Invoice
  #       - Payment
  #   success_metrics:
  #     - id: close_time
  #       target: "<= 3d"
  #       window: 30d
  #       dimension: functional
  #   failure_modes:
  #     - "Late close"
  #   exceptions:
  #     - "Holiday periods"
  #   evidence_required:
  #     - "Reconciliation report"
  #   audit_requirements:
  #     - "SOX compliance"
  #   links:
  #     - url: "https://example.com"
  #       ref: "@invariant:closed_period_readonly"
  #       label: "Policy doc"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "CFO");
        assert_eq!(jtbd.job_functional, "Close the books");
        assert_eq!(jtbd.job_emotional.as_deref(), Some("Feel in control"));
        assert_eq!(
            jtbd.job_relational.as_deref(),
            Some("Be trusted by the board")
        );
        assert_eq!(jtbd.so_that, "Reports are accurate");

        let scope = jtbd.scope.unwrap();
        assert_eq!(scope.pack.as_deref(), Some("finance"));
        assert_eq!(scope.segment.as_deref(), Some("enterprise"));
        assert_eq!(scope.objects, vec!["Invoice", "Payment"]);

        assert_eq!(jtbd.success_metrics.len(), 1);
        assert_eq!(jtbd.success_metrics[0].id, "close_time");
        assert_eq!(jtbd.success_metrics[0].target, "<= 3d");
        assert_eq!(jtbd.success_metrics[0].window, "30d");
        assert_eq!(
            jtbd.success_metrics[0].dimension.as_deref(),
            Some("functional")
        );

        assert_eq!(jtbd.failure_modes, vec!["Late close"]);
        assert_eq!(jtbd.exceptions, vec!["Holiday periods"]);
        assert_eq!(jtbd.evidence_required, vec!["Reconciliation report"]);
        assert_eq!(jtbd.audit_requirements, vec!["SOX compliance"]);

        assert_eq!(jtbd.links.len(), 1);
        assert_eq!(jtbd.links[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(
            jtbd.links[0].ref_.as_deref(),
            Some("@invariant:closed_period_readonly")
        );
        assert_eq!(jtbd.links[0].label.as_deref(), Some("Policy doc"));
    }

    #[test]
    fn test_yaml_minimal_required_only() {
        let content = r#"
Truth: Minimal

  # JTBD:
  #   actor: Dev
  #   job_functional: "Ship code"
  #   so_that: "Users get value"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Dev");
        assert_eq!(jtbd.job_functional, "Ship code");
        assert!(jtbd.job_emotional.is_none());
        assert!(jtbd.job_relational.is_none());
        assert_eq!(jtbd.so_that, "Users get value");
        assert!(jtbd.scope.is_none());
        assert!(jtbd.success_metrics.is_empty());
        assert!(jtbd.failure_modes.is_empty());
        assert!(jtbd.exceptions.is_empty());
        assert!(jtbd.evidence_required.is_empty());
        assert!(jtbd.audit_requirements.is_empty());
        assert!(jtbd.links.is_empty());
    }

    #[test]
    fn test_yaml_multiple_success_metrics() {
        let content = r#"
Truth: Multi-metric

  # JTBD:
  #   actor: SRE
  #   job_functional: "Keep systems up"
  #   so_that: "Customers trust us"
  #   success_metrics:
  #     - id: uptime
  #       target: ">= 0.999"
  #       window: 30d
  #       dimension: functional
  #     - id: mttr
  #       target: "<= 15m"
  #       window: 90d
  #     - id: satisfaction
  #       target: ">= 4.5"
  #       window: 7d
  #       dimension: emotional
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.success_metrics.len(), 3);
        assert_eq!(jtbd.success_metrics[0].id, "uptime");
        assert_eq!(jtbd.success_metrics[1].id, "mttr");
        assert!(jtbd.success_metrics[1].dimension.is_none());
        assert_eq!(jtbd.success_metrics[2].id, "satisfaction");
        assert_eq!(
            jtbd.success_metrics[2].dimension.as_deref(),
            Some("emotional")
        );
    }

    #[test]
    fn test_yaml_feature_keyword() {
        let content = r#"
Feature: Payment processing

  # JTBD:
  #   actor: Accountant
  #   job_functional: "Process payments"
  #   so_that: "Vendors are paid on time"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();
        assert_eq!(jtbd.actor, "Accountant");
    }

    #[test]
    fn test_yaml_scenario_level_jtbd() {
        let content = r#"
Truth: Invoicing

  # JTBD:
  #   actor: Founder
  #   job_functional: "Invoice customers"
  #   so_that: "Cash flows"

  Scenario: Send invoice on time

    # JTBD:
    #   actor: BookKeeper
    #   job_functional: "Generate invoice within 24h"
    #   so_that: "No late fees"

  Scenario: Retry failed payment

    # JTBD:
    #   actor: System
    #   job_functional: "Retry payment automatically"
    #   so_that: "Revenue is not lost"
"#;

        let (file_jtbd, scenario_jtbds) = extract_jtbd(content).unwrap();

        let file = file_jtbd.unwrap();
        assert_eq!(file.actor, "Founder");

        assert_eq!(scenario_jtbds.len(), 2);
        assert_eq!(scenario_jtbds[0].actor, "BookKeeper");
        assert_eq!(
            scenario_jtbds[0].job_functional,
            "Generate invoice within 24h"
        );
        assert_eq!(scenario_jtbds[1].actor, "System");
        assert_eq!(scenario_jtbds[1].so_that, "Revenue is not lost");
    }

    #[test]
    fn test_yaml_multiple_failure_modes_and_exceptions() {
        let content = r#"
Truth: Robust parsing

  # JTBD:
  #   actor: Engineer
  #   job_functional: "Parse data"
  #   so_that: "Data is correct"
  #   failure_modes:
  #     - "Corrupt input"
  #     - "Timeout"
  #     - "Partial write"
  #   exceptions:
  #     - "Legacy format"
  #     - "Empty file"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.failure_modes.len(), 3);
        assert_eq!(jtbd.failure_modes[0], "Corrupt input");
        assert_eq!(jtbd.failure_modes[2], "Partial write");
        assert_eq!(jtbd.exceptions.len(), 2);
    }

    #[test]
    fn test_yaml_scope_without_objects() {
        let content = r#"
Truth: Scoped

  # JTBD:
  #   actor: Admin
  #   job_functional: "Manage users"
  #   so_that: "Access is controlled"
  #   scope:
  #     pack: identity
  #     segment: saas
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        let scope = jtbd.scope.unwrap();
        assert_eq!(scope.pack.as_deref(), Some("identity"));
        assert_eq!(scope.segment.as_deref(), Some("saas"));
        assert!(scope.objects.is_empty());
    }

    #[test]
    fn test_yaml_links_url_only() {
        let content = r#"
Truth: Linked

  # JTBD:
  #   actor: PM
  #   job_functional: "Track progress"
  #   so_that: "Nothing slips"
  #   links:
  #     - url: "https://docs.example.com"
  #     - label: "Internal wiki"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.links.len(), 2);
        assert_eq!(
            jtbd.links[0].url.as_deref(),
            Some("https://docs.example.com")
        );
        assert!(jtbd.links[0].ref_.is_none());
        assert!(jtbd.links[1].url.is_none());
        assert_eq!(jtbd.links[1].label.as_deref(), Some("Internal wiki"));
    }

    // ── Plain text parsing ──

    #[test]
    fn test_parse_plain_text_jtbd() {
        let content = r"
Truth: Example

  # JTBD
  # As: Founder
  # Functional: Invoice customers
  # So that: Cash flows predictably
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Founder");
        assert_eq!(jtbd.job_functional, "Invoice customers");
        assert_eq!(jtbd.so_that, "Cash flows predictably");
    }

    #[test]
    fn test_plain_text_all_basic_fields() {
        let content = r"
Truth: Full plain text

  # JTBD
  # As: Designer
  # Functional: Create mockups quickly
  # Emotional: Feel creative and unblocked
  # Relational: Be seen as design leader
  # So that: Product ships with great UX
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Designer");
        assert_eq!(jtbd.job_functional, "Create mockups quickly");
        assert_eq!(
            jtbd.job_emotional.as_deref(),
            Some("Feel creative and unblocked")
        );
        assert_eq!(
            jtbd.job_relational.as_deref(),
            Some("Be seen as design leader")
        );
        assert_eq!(jtbd.so_that, "Product ships with great UX");
    }

    #[test]
    fn test_plain_text_underscore_keys() {
        let content = r"
Truth: Underscore keys

  # JTBD
  # As: Ops
  # job_functional: Deploy services
  # job_emotional: Feel safe deploying
  # job_relational: Team trusts the pipeline
  # so_that: Zero-downtime releases
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Ops");
        assert_eq!(jtbd.job_functional, "Deploy services");
        assert_eq!(jtbd.job_emotional.as_deref(), Some("Feel safe deploying"));
        assert_eq!(
            jtbd.job_relational.as_deref(),
            Some("Team trusts the pipeline")
        );
        assert_eq!(jtbd.so_that, "Zero-downtime releases");
    }

    #[test]
    fn test_plain_text_with_scope() {
        let content = r"
Truth: Scoped plain

  # JTBD
  # As: Admin
  # Functional: Manage accounts
  # So that: Compliance is met
  # Scope: identity.enterprise [User, Role, Permission]
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        let scope = jtbd.scope.unwrap();
        assert_eq!(scope.pack.as_deref(), Some("identity"));
        assert_eq!(scope.segment.as_deref(), Some("enterprise"));
        assert_eq!(scope.objects, vec!["User", "Role", "Permission"]);
    }

    #[test]
    fn test_plain_text_failure_modes_and_exceptions() {
        let content = r"
Truth: Error paths

  # JTBD
  # As: Tester
  # Functional: Validate inputs
  # So that: Bugs are caught early
  # Failure mode: Invalid email format
  # Failure_mode: Missing required field
  # Exception: Legacy API clients
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.failure_modes.len(), 2);
        assert_eq!(jtbd.failure_modes[0], "Invalid email format");
        assert_eq!(jtbd.failure_modes[1], "Missing required field");
        assert_eq!(jtbd.exceptions, vec!["Legacy API clients"]);
    }

    #[test]
    fn test_plain_text_evidence_and_audit() {
        let content = r"
Truth: Auditable

  # JTBD
  # As: Compliance Officer
  # Functional: Audit transactions
  # So that: Regulations are met
  # Evidence: Transaction log
  # Audit: GDPR data retention
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.evidence_required, vec!["Transaction log"]);
        assert_eq!(jtbd.audit_requirements, vec!["GDPR data retention"]);
    }

    #[test]
    fn test_plain_text_scenario_level() {
        let content = r"
Truth: Multi-scenario

  # JTBD
  # As: PM
  # Functional: Oversee project
  # So that: Delivery is on track

  Scenario: Daily standup

    # JTBD
    # As: Developer
    # Functional: Report status
    # So that: Blockers are surfaced
";

        let (file_jtbd, scenario_jtbds) = extract_jtbd(content).unwrap();

        assert!(file_jtbd.is_some());
        assert_eq!(file_jtbd.unwrap().actor, "PM");
        assert_eq!(scenario_jtbds.len(), 1);
        assert_eq!(scenario_jtbds[0].actor, "Developer");
    }

    #[test]
    fn test_plain_text_quoted_values() {
        let content = r#"
Truth: Quoted

  # JTBD
  # As: "Product Manager"
  # Functional: "Define and prioritize features"
  # So that: "Team builds the right thing"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Product Manager");
        assert_eq!(jtbd.job_functional, "Define and prioritize features");
    }

    // ── Validation ──

    #[test]
    fn test_validate_jtbd() {
        let jtbd = minimal_jtbd();

        let issues = validate_jtbd(&jtbd, false);
        assert_eq!(issues.len(), 2);
        assert!(
            issues
                .iter()
                .all(|i| i.severity == ValidationSeverity::Warning)
        );
    }

    #[test]
    fn test_validate_strict_mode_errors_on_missing_recommended() {
        let jtbd = minimal_jtbd();

        let issues = validate_jtbd(&jtbd, true);
        assert_eq!(issues.len(), 2);
        assert!(
            issues
                .iter()
                .all(|i| i.severity == ValidationSeverity::Error)
        );
        assert!(issues.iter().any(|i| i.field == "job_emotional"));
        assert!(issues.iter().any(|i| i.field == "job_relational"));
    }

    #[test]
    fn test_validate_complete_jtbd_no_issues() {
        let jtbd = full_jtbd();
        let issues = validate_jtbd(&jtbd, true);
        assert!(
            issues.is_empty(),
            "Expected no issues for full JTBD, got: {issues:?}"
        );
    }

    #[test]
    fn test_validate_lenient_with_emotional_only() {
        let mut jtbd = minimal_jtbd();
        jtbd.job_emotional = Some("Feel good".to_string());

        let issues = validate_jtbd(&jtbd, false);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "job_relational");
        assert_eq!(issues[0].severity, ValidationSeverity::Warning);
    }

    #[test]
    fn test_validate_lenient_with_relational_only() {
        let mut jtbd = minimal_jtbd();
        jtbd.job_relational = Some("Be trusted".to_string());

        let issues = validate_jtbd(&jtbd, false);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].field, "job_emotional");
    }

    #[test]
    fn test_validate_duplicate_metric_ids() {
        let mut jtbd = full_jtbd();
        jtbd.success_metrics.push(SuccessMetric {
            id: "close_time".to_string(),
            target: ">= 0.5".to_string(),
            window: "7d".to_string(),
            dimension: None,
        });

        let issues = validate_jtbd(&jtbd, false);
        assert!(issues.iter().any(|i| i.field == "success_metrics"
            && i.severity == ValidationSeverity::Error
            && i.message.contains("Duplicate")));
    }

    #[test]
    fn test_validate_unique_metric_ids_no_error() {
        let jtbd = full_jtbd();
        let issues = validate_jtbd(&jtbd, true);
        assert!(!issues.iter().any(|i| i.field == "success_metrics"));
    }

    #[test]
    fn test_validation_severity_ordering() {
        assert!(ValidationSeverity::Warning < ValidationSeverity::Error);
    }

    // ── Error cases ──

    #[test]
    fn test_yaml_missing_actor() {
        let content = r#"
Truth: Missing actor

  # JTBD:
  #   job_functional: "Do thing"
  #   so_that: "Get result"
"#;

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "actor"));
    }

    #[test]
    fn test_yaml_missing_job_functional() {
        let content = r#"
Truth: Missing functional

  # JTBD:
  #   actor: Dev
  #   so_that: "Get result"
"#;

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "job_functional"));
    }

    #[test]
    fn test_yaml_missing_so_that() {
        let content = r#"
Truth: Missing so_that

  # JTBD:
  #   actor: Dev
  #   job_functional: "Do thing"
"#;

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "so_that"));
    }

    #[test]
    fn test_plain_text_missing_actor() {
        let content = r"
Truth: No actor

  # JTBD
  # Functional: Do thing
  # So that: Get result
";

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "actor"));
    }

    #[test]
    fn test_plain_text_missing_functional() {
        let content = r"
Truth: No functional

  # JTBD
  # As: Dev
  # So that: Get result
";

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "job_functional"));
    }

    #[test]
    fn test_plain_text_missing_so_that() {
        let content = r"
Truth: No so_that

  # JTBD
  # As: Dev
  # Functional: Do thing
";

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::MissingRequiredField(ref f) if f == "so_that"));
    }

    #[test]
    fn test_yaml_malformed_yaml() {
        let content = r"
Truth: Bad YAML

  # JTBD:
  #   actor: [invalid
  #   unclosed: bracket
";

        let err = extract_jtbd(content).unwrap_err();
        assert!(matches!(err, JTBDError::InvalidYaml(_)));
    }

    // ── Edge cases ──

    #[test]
    fn test_empty_content() {
        let (file_jtbd, scenario_jtbds) = extract_jtbd("").unwrap();
        assert!(file_jtbd.is_none());
        assert!(scenario_jtbds.is_empty());
    }

    #[test]
    fn test_no_jtbd_blocks() {
        let content = r"
Truth: No JTBD here

  Given something
  When action
  Then result
";

        let (file_jtbd, scenario_jtbds) = extract_jtbd(content).unwrap();
        assert!(file_jtbd.is_none());
        assert!(scenario_jtbds.is_empty());
    }

    #[test]
    fn test_truth_without_jtbd_followed_by_scenario_with_jtbd() {
        let content = r#"
Truth: No file-level JTBD

  Scenario: Has JTBD

    # JTBD:
    #   actor: Tester
    #   job_functional: "Test things"
    #   so_that: "Quality"
"#;

        let (file_jtbd, scenario_jtbds) = extract_jtbd(content).unwrap();
        assert!(file_jtbd.is_none());
        assert_eq!(scenario_jtbds.len(), 1);
        assert_eq!(scenario_jtbds[0].actor, "Tester");
    }

    #[test]
    fn test_unicode_in_fields() {
        let content = r#"
Truth: Unicode support

  # JTBD:
  #   actor: "Gründer"
  #   job_functional: "Rechnungen erstellen"
  #   job_emotional: "Sicherheit fühlen 🔒"
  #   so_that: "Geld fließt zuverlässig"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Gründer");
        assert_eq!(jtbd.job_functional, "Rechnungen erstellen");
        assert!(jtbd.job_emotional.as_ref().unwrap().contains("🔒"));
        assert_eq!(jtbd.so_that, "Geld fließt zuverlässig");
    }

    #[test]
    fn test_special_characters_in_values() {
        let content = r#"
Truth: Special chars

  # JTBD:
  #   actor: "Dev/Ops"
  #   job_functional: "Deploy (safely) & monitor"
  #   so_that: "99.9% uptime - no exceptions"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.actor, "Dev/Ops");
        assert!(jtbd.job_functional.contains('&'));
        assert!(jtbd.so_that.contains("99.9%"));
    }

    #[test]
    fn test_whitespace_only_content() {
        let (file_jtbd, scenario_jtbds) = extract_jtbd("   \n\n  \n").unwrap();
        assert!(file_jtbd.is_none());
        assert!(scenario_jtbds.is_empty());
    }

    #[test]
    fn test_multiple_blank_lines_between_truth_and_jtbd() {
        let content = r#"
Truth: Spaced out



  # JTBD:
  #   actor: Dev
  #   job_functional: "Code"
  #   so_that: "Ship"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();
        assert_eq!(jtbd.actor, "Dev");
    }

    #[test]
    fn test_no_space_after_hash() {
        let content = r"
Truth: Tight hashes

  # JTBD
  #As: Dev
  #Functional: Code
  #So that: Ship
";
        // Lines with "#As:" (no space) get stripped by strip_prefix('#') -> "As: Dev"
        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();
        assert_eq!(jtbd.actor, "Dev");
    }

    #[test]
    fn test_jtbd_block_stops_at_non_comment_line() {
        let content = r#"
Truth: Stopped

  # JTBD:
  #   actor: Dev
  #   job_functional: "Code"
  #   so_that: "Ship"
  Given something happens
  Then result
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();
        assert_eq!(jtbd.actor, "Dev");
        assert_eq!(jtbd.so_that, "Ship");
    }

    #[test]
    fn test_truth_at_start_of_line_no_indent() {
        let content = r#"
Truth: At start

# JTBD:
#   actor: Dev
#   job_functional: "Code"
#   so_that: "Ship"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();
        assert_eq!(jtbd.actor, "Dev");
    }

    #[test]
    fn test_scope_pack_only_no_segment() {
        let content = r"
Truth: Pack only scope

  # JTBD
  # As: Admin
  # Functional: Manage
  # So that: Control
  # Scope: billing [Invoice]
";

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        let scope = jtbd.scope.unwrap();
        assert_eq!(scope.pack.as_deref(), Some("billing"));
        assert!(scope.segment.is_none());
        assert_eq!(scope.objects, vec!["Invoice"]);
    }

    #[test]
    fn test_error_display_messages() {
        let missing = JTBDError::MissingRequiredField("actor".to_string());
        assert_eq!(missing.to_string(), "Missing required field: actor");

        let yaml = JTBDError::InvalidYaml("bad indent".to_string());
        assert_eq!(yaml.to_string(), "Invalid YAML: bad indent");

        let parse = JTBDError::ParseError("unexpected token".to_string());
        assert_eq!(parse.to_string(), "Parse error: unexpected token");
    }

    #[test]
    fn test_jtbd_metadata_clone_and_eq() {
        let a = full_jtbd();
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_jtbd_metadata_serde_roundtrip() {
        let original = full_jtbd();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: JTBDMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_validation_issue_serde_roundtrip() {
        let issue = JTBDValidationIssue {
            field: "job_emotional".to_string(),
            severity: ValidationSeverity::Warning,
            message: "Missing recommended field".to_string(),
        };
        let json = serde_json::to_string(&issue).unwrap();
        let deserialized: JTBDValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue, deserialized);
    }

    #[test]
    fn test_multiple_scenarios_no_file_jtbd() {
        let content = r#"
Truth: No file JTBD

  Scenario: First

    # JTBD:
    #   actor: A
    #   job_functional: "Do A"
    #   so_that: "Result A"

  Scenario: Second

    # JTBD:
    #   actor: B
    #   job_functional: "Do B"
    #   so_that: "Result B"

  Scenario: Third

    # JTBD:
    #   actor: C
    #   job_functional: "Do C"
    #   so_that: "Result C"
"#;

        let (file_jtbd, scenarios) = extract_jtbd(content).unwrap();
        assert!(file_jtbd.is_none());
        assert_eq!(scenarios.len(), 3);
        assert_eq!(scenarios[0].actor, "A");
        assert_eq!(scenarios[1].actor, "B");
        assert_eq!(scenarios[2].actor, "C");
    }

    #[test]
    fn test_scenario_without_jtbd_skipped() {
        let content = r#"
Truth: Mixed

  Scenario: No JTBD
    Given something
    Then result

  Scenario: Has JTBD

    # JTBD:
    #   actor: Dev
    #   job_functional: "Test"
    #   so_that: "Quality"
"#;

        let (_, scenarios) = extract_jtbd(content).unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].actor, "Dev");
    }

    #[test]
    fn test_yaml_evidence_and_audit_lists() {
        let content = r#"
Truth: Evidence

  # JTBD:
  #   actor: Auditor
  #   job_functional: "Review compliance"
  #   so_that: "Regulations met"
  #   evidence_required:
  #     - "Transaction log"
  #     - "User consent records"
  #   audit_requirements:
  #     - "SOX section 404"
  #     - "GDPR Art 30"
"#;

        let (file_jtbd, _) = extract_jtbd(content).unwrap();
        let jtbd = file_jtbd.unwrap();

        assert_eq!(jtbd.evidence_required.len(), 2);
        assert_eq!(jtbd.evidence_required[0], "Transaction log");
        assert_eq!(jtbd.audit_requirements.len(), 2);
        assert_eq!(jtbd.audit_requirements[1], "GDPR Art 30");
    }

    // ── Property tests ──

    proptest! {
        #[test]
        fn yaml_roundtrip_never_panics(
            actor in "[A-Za-z ]{1,30}",
            func in "[A-Za-z ]{1,50}",
            so_that in "[A-Za-z ]{1,50}",
        ) {
            let content = format!(
                "Truth: Prop test\n\n  # JTBD:\n  #   actor: \"{actor}\"\n  #   job_functional: \"{func}\"\n  #   so_that: \"{so_that}\"\n"
            );
            let result = extract_jtbd(&content);
            prop_assert!(result.is_ok());
            let (file_jtbd, _) = result.unwrap();
            prop_assert!(file_jtbd.is_some());
            let jtbd = file_jtbd.unwrap();
            prop_assert_eq!(jtbd.actor.trim(), actor.trim());
            prop_assert_eq!(jtbd.job_functional.trim(), func.trim());
            prop_assert_eq!(jtbd.so_that.trim(), so_that.trim());
        }

        #[test]
        fn plain_text_roundtrip_never_panics(
            actor in "[A-Za-z]{1,20}",
            func in "[A-Za-z ]{1,40}",
            so_that in "[A-Za-z ]{1,40}",
        ) {
            let content = format!(
                "Truth: Prop test\n\n  # JTBD\n  # As: {actor}\n  # Functional: {func}\n  # So that: {so_that}\n"
            );
            let result = extract_jtbd(&content);
            prop_assert!(result.is_ok());
            let (file_jtbd, _) = result.unwrap();
            prop_assert!(file_jtbd.is_some());
        }

        #[test]
        fn validate_never_panics(
            has_emotional in any::<bool>(),
            has_relational in any::<bool>(),
            strict in any::<bool>(),
        ) {
            let mut jtbd = minimal_jtbd();
            if has_emotional {
                jtbd.job_emotional = Some("Feel good".to_string());
            }
            if has_relational {
                jtbd.job_relational = Some("Be trusted".to_string());
            }
            let issues = validate_jtbd(&jtbd, strict);
            let expected = usize::from(!has_emotional) + usize::from(!has_relational);
            prop_assert_eq!(issues.len(), expected);
        }

        #[test]
        fn serde_roundtrip_preserves_data(
            actor in "[A-Za-z]{1,20}",
            func in "[A-Za-z ]{1,40}",
        ) {
            let jtbd = JTBDMetadata {
                actor: actor.clone(),
                job_functional: func.clone(),
                job_emotional: None,
                job_relational: None,
                so_that: "outcome".to_string(),
                scope: None,
                success_metrics: Vec::new(),
                failure_modes: Vec::new(),
                exceptions: Vec::new(),
                evidence_required: Vec::new(),
                audit_requirements: Vec::new(),
                links: Vec::new(),
            };
            let json = serde_json::to_string(&jtbd).unwrap();
            let back: JTBDMetadata = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(jtbd, back);
        }

        #[test]
        fn content_without_truth_keyword_yields_none(
            garbage in "[a-z ]{0,100}",
        ) {
            let result = extract_jtbd(&garbage);
            prop_assert!(result.is_ok());
            let (file_jtbd, scenarios) = result.unwrap();
            prop_assert!(file_jtbd.is_none());
            prop_assert!(scenarios.is_empty());
        }
    }
}
