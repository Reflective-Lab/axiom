// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Pre-flight simulation for Converge Truths.
//!
//! Analyzes a Truth spec **before** execution to determine whether it has
//! a realistic chance of converging. Catches underspecification, missing
//! resources, and governance gaps early — no agents need to run.
//!
//! # Example
//!
//! ```ignore
//! use converge_axiom::simulation::{simulate, SimulationConfig};
//! use converge_axiom::truths::parse_truth_document;
//!
//! let doc = parse_truth_document(spec)?;
//! let report = simulate(&doc, &SimulationConfig::default());
//! if !report.can_converge() {
//!     for finding in &report.findings {
//!         eprintln!("{}: {}", finding.severity, finding.message);
//!     }
//! }
//! ```

use crate::gherkin::{ValidationError, extract_all_metas, preprocess_truths};
use crate::truths::{TruthDocument, TruthGovernance};

/// Configuration for simulation strictness.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Require Intent block with at least Outcome.
    pub require_intent: bool,
    /// Require Authority block with at least Actor.
    pub require_authority: bool,
    /// Require Evidence block with at least one Requires field.
    pub require_evidence: bool,
    /// Require at least one scenario with a Then step.
    pub require_assertions: bool,
    /// Require scenario Given steps to reference resources declared in Evidence.
    pub check_resource_availability: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            require_intent: true,
            require_authority: true,
            require_evidence: true,
            require_assertions: true,
            check_resource_availability: true,
        }
    }
}

/// Overall simulation verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The spec looks complete enough to converge.
    Ready,
    /// The spec has warnings but might converge.
    Risky,
    /// The spec is underspecified and will not converge.
    WillNotConverge,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "ready"),
            Self::Risky => write!(f, "risky"),
            Self::WillNotConverge => write!(f, "will-not-converge"),
        }
    }
}

/// Severity of a simulation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A single simulation finding.
#[derive(Debug, Clone)]
pub struct SimulationFinding {
    pub severity: FindingSeverity,
    pub category: &'static str,
    pub message: String,
    pub suggestion: Option<String>,
}

/// The result of simulating a Truth spec.
#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub verdict: Verdict,
    pub findings: Vec<SimulationFinding>,
    pub governance_coverage: GovernanceCoverage,
    pub scenario_count: usize,
    pub resource_summary: ResourceSummary,
}

impl SimulationReport {
    /// Whether the spec has a realistic chance of converging.
    pub fn can_converge(&self) -> bool {
        self.verdict != Verdict::WillNotConverge
    }
}

/// Which governance blocks are present and how complete they are.
#[derive(Debug, Clone, Default)]
pub struct GovernanceCoverage {
    pub has_intent: bool,
    pub has_outcome: bool,
    pub has_authority: bool,
    pub has_actor: bool,
    pub has_approval_gate: bool,
    pub has_constraint: bool,
    pub has_evidence: bool,
    pub evidence_count: usize,
    pub has_exception: bool,
    pub has_escalation_path: bool,
}

/// Summary of resources required vs available.
#[derive(Debug, Clone, Default)]
pub struct ResourceSummary {
    /// Resources declared in Evidence.Requires.
    pub declared_evidence: Vec<String>,
    /// Resources referenced in scenario steps.
    pub referenced_in_scenarios: Vec<String>,
    /// Resources referenced but not declared.
    pub missing: Vec<String>,
}

/// Run a pre-flight simulation on a parsed Truth document.
pub fn simulate(doc: &TruthDocument, config: &SimulationConfig) -> SimulationReport {
    let mut findings = Vec::new();

    let governance_coverage = check_governance(&doc.governance, config, &mut findings);
    let scenario_count = check_scenarios(&doc.gherkin, config, &mut findings);
    let resource_summary = check_resources(&doc.governance, &doc.gherkin, config, &mut findings);

    let has_errors = findings
        .iter()
        .any(|f| matches!(f.severity, FindingSeverity::Error));
    let has_warnings = findings
        .iter()
        .any(|f| matches!(f.severity, FindingSeverity::Warning));

    let verdict = if has_errors {
        Verdict::WillNotConverge
    } else if has_warnings {
        Verdict::Risky
    } else {
        Verdict::Ready
    };

    SimulationReport {
        verdict,
        findings,
        governance_coverage,
        scenario_count,
        resource_summary,
    }
}

/// Parse and simulate in one step.
pub fn simulate_spec(
    content: &str,
    config: &SimulationConfig,
) -> Result<SimulationReport, ValidationError> {
    let doc = crate::truths::parse_truth_document(content)?;
    Ok(simulate(&doc, config))
}

fn check_governance(
    gov: &TruthGovernance,
    config: &SimulationConfig,
    findings: &mut Vec<SimulationFinding>,
) -> GovernanceCoverage {
    let mut coverage = GovernanceCoverage::default();

    // Intent
    if let Some(intent) = &gov.intent {
        coverage.has_intent = true;
        coverage.has_outcome = intent.outcome.is_some();
        if intent.outcome.is_none() {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Warning,
                category: "governance",
                message: "Intent block present but missing Outcome field.".into(),
                suggestion: Some("Add `Outcome: <what should happen>` to the Intent block.".into()),
            });
        }
    } else if config.require_intent {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Error,
            category: "governance",
            message: "Missing Intent block — agents have no goal to converge toward.".into(),
            suggestion: Some("Add an Intent block with Outcome and optionally Goal.".into()),
        });
    }

    // Authority
    if let Some(authority) = &gov.authority {
        coverage.has_authority = true;
        coverage.has_actor = authority.actor.is_some();
        coverage.has_approval_gate = !authority.requires_approval.is_empty();
        if authority.actor.is_none() {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Warning,
                category: "governance",
                message: "Authority block present but missing Actor field.".into(),
                suggestion: Some("Add `Actor: <who can approve>` to the Authority block.".into()),
            });
        }
    } else if config.require_authority {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Error,
            category: "governance",
            message: "Missing Authority block — no one is authorized to promote decisions.".into(),
            suggestion: Some(
                "Add an Authority block with Actor and optionally Requires Approval.".into(),
            ),
        });
    }

    // Constraint
    if let Some(constraint) = &gov.constraint {
        coverage.has_constraint = true;
        if constraint.budget.is_empty()
            && constraint.cost_limit.is_empty()
            && constraint.must_not.is_empty()
        {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Info,
                category: "governance",
                message: "Constraint block is empty — agents have no guardrails.".into(),
                suggestion: None,
            });
        }
    }

    // Evidence
    if let Some(evidence) = &gov.evidence {
        coverage.has_evidence = true;
        coverage.evidence_count = evidence.requires.len();
        if evidence.requires.is_empty() {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Warning,
                category: "governance",
                message: "Evidence block present but no Requires fields — nothing to audit.".into(),
                suggestion: Some("Add `Requires: <evidence_name>` fields.".into()),
            });
        }
        if evidence.audit.is_empty() {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Info,
                category: "governance",
                message: "No Audit field in Evidence — decision trail may be incomplete.".into(),
                suggestion: Some("Add `Audit: <log_name>` for traceability.".into()),
            });
        }
    } else if config.require_evidence {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Error,
            category: "governance",
            message: "Missing Evidence block — no proof requirements declared.".into(),
            suggestion: Some("Add an Evidence block with Requires and Audit fields.".into()),
        });
    }

    // Exception
    if let Some(exception) = &gov.exception {
        coverage.has_exception = true;
        coverage.has_escalation_path = !exception.escalates_to.is_empty();
    }

    // Cross-block coherence
    if coverage.has_approval_gate && !coverage.has_evidence {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "coherence",
            message:
                "Authority requires approval but no Evidence block — approver has nothing to review."
                    .into(),
            suggestion: Some(
                "Add Evidence.Requires fields so the approver has artifacts to evaluate.".into(),
            ),
        });
    }

    if coverage.has_constraint && !coverage.has_authority {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "coherence",
            message: "Constraints declared but no Authority — who enforces the limits?".into(),
            suggestion: Some("Add an Authority block with an Actor.".into()),
        });
    }

    coverage
}

fn check_scenarios(
    gherkin: &str,
    config: &SimulationConfig,
    findings: &mut Vec<SimulationFinding>,
) -> usize {
    let preprocessed = preprocess_truths(gherkin);
    let metas = extract_all_metas(&preprocessed).unwrap_or_default();

    if metas.is_empty() {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Error,
            category: "scenario",
            message: "No scenarios found — nothing to execute.".into(),
            suggestion: Some("Add at least one Scenario with Given/When/Then steps.".into()),
        });
        return 0;
    }

    // Check for Then steps (assertions)
    if config.require_assertions {
        let has_then = gherkin.lines().any(|line| line.trim().starts_with("Then "));
        if !has_then {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Error,
                category: "scenario",
                message: "No Then steps found — scenarios have no success criteria.".into(),
                suggestion: Some("Add Then steps that assert expected outcomes.".into()),
            });
        }
    }

    // Check for Given steps (preconditions)
    let has_given = gherkin
        .lines()
        .any(|line| line.trim().starts_with("Given "));
    if !has_given {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "scenario",
            message: "No Given steps — scenarios have no declared preconditions.".into(),
            suggestion: Some("Add Given steps that establish the initial state.".into()),
        });
    }

    // Check for When steps (actions)
    let has_when = gherkin.lines().any(|line| line.trim().starts_with("When "));
    if !has_when {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "scenario",
            message: "No When steps — scenarios have no triggering action.".into(),
            suggestion: Some("Add When steps that describe the action being governed.".into()),
        });
    }

    metas.len()
}

fn check_resources(
    gov: &TruthGovernance,
    gherkin: &str,
    config: &SimulationConfig,
    findings: &mut Vec<SimulationFinding>,
) -> ResourceSummary {
    let mut summary = ResourceSummary::default();

    // Collect declared evidence resources
    if let Some(evidence) = &gov.evidence {
        summary.declared_evidence = evidence.requires.clone();
    }

    // Extract resource references from scenario steps
    let resource_pattern = regex::Regex::new(r#"[a-z][a-z0-9_]*(?:_[a-z0-9]+)+"#).ok();

    if let Some(pattern) = &resource_pattern {
        for line in gherkin.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Given ")
                || trimmed.starts_with("When ")
                || trimmed.starts_with("Then ")
                || trimmed.starts_with("And ")
            {
                for m in pattern.find_iter(trimmed) {
                    let resource = m.as_str().to_string();
                    if !summary.referenced_in_scenarios.contains(&resource) {
                        summary.referenced_in_scenarios.push(resource);
                    }
                }
            }
        }
    }

    // Find references that match evidence naming patterns but aren't declared
    if config.check_resource_availability && !summary.declared_evidence.is_empty() {
        for referenced in &summary.referenced_in_scenarios {
            let looks_like_evidence = referenced.ends_with("_assessment")
                || referenced.ends_with("_analysis")
                || referenced.ends_with("_report")
                || referenced.ends_with("_review")
                || referenced.ends_with("_log")
                || referenced.ends_with("_record")
                || referenced.ends_with("_bundle");

            if looks_like_evidence && !summary.declared_evidence.contains(referenced) {
                summary.missing.push(referenced.clone());
            }
        }

        if !summary.missing.is_empty() {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Warning,
                category: "resources",
                message: format!(
                    "Scenario references evidence-like resources not declared in Evidence block: {}",
                    summary.missing.join(", ")
                ),
                suggestion: Some(
                    "Add these as `Requires:` fields in the Evidence block, or rename to avoid evidence naming patterns.".into(),
                ),
            });
        }
    }

    // Check if authority actors are referenced in scenarios
    if let Some(authority) = &gov.authority {
        if let Some(actor) = &authority.actor {
            let actor_referenced = gherkin.contains(actor);
            if !actor_referenced {
                findings.push(SimulationFinding {
                    severity: FindingSeverity::Info,
                    category: "resources",
                    message: format!(
                        "Authority actor `{actor}` is declared but not referenced in any scenario."
                    ),
                    suggestion: Some(
                        "Consider adding a scenario step that involves the authorized actor."
                            .into(),
                    ),
                });
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truths::parse_truth_document;

    #[test]
    fn complete_spec_is_ready() {
        let content = r#"Truth: Vendor selection is governed

Intent:
  Outcome: Select a vendor with auditable rationale.
  Goal: Evaluate candidates on cost and compliance.

Authority:
  Actor: governance_review_board
  Requires Approval: final_vendor_selection

Constraint:
  Cost Limit: first-year spend must stay within budget.

Evidence:
  Requires: security_assessment
  Requires: pricing_analysis
  Audit: decision_log

Scenario: Vendors produce traceable outcomes
  Given candidate vendors "Acme AI, Beta ML"
  And each vendor has a security_assessment and pricing_analysis
  When the governance_review_board evaluates each vendor
  Then each vendor should produce a compliance screening result
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::Ready);
        assert!(report.can_converge());
    }

    #[test]
    fn missing_intent_will_not_converge() {
        let content = r#"Truth: No intent

Scenario: Something happens
  Given a precondition
  When an action occurs
  Then a result is observed
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(!report.can_converge());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("Missing Intent"))
        );
    }

    #[test]
    fn missing_then_steps_will_not_converge() {
        let content = r#"Truth: No assertions

Intent:
  Outcome: Do something.

Authority:
  Actor: admin

Evidence:
  Requires: report

Scenario: Missing outcome
  Given a shortlist of vendors
  When the workflow ranks them
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("No Then steps"))
        );
    }

    #[test]
    fn approval_without_evidence_is_risky() {
        let content = r#"Truth: Approval gate without evidence

Intent:
  Outcome: Approve a vendor.

Authority:
  Actor: board
  Requires Approval: cfo_sign_off

Scenario: Approval happens
  Given a vendor is shortlisted
  When the board reviews
  Then the vendor is approved
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("approver has nothing to review"))
        );
    }

    #[test]
    fn simulate_spec_convenience() {
        let content = r#"Truth: Quick test

Intent:
  Outcome: Works.

Authority:
  Actor: admin

Evidence:
  Requires: proof

Scenario: It works
  Given something exists
  When validated
  Then it passes
"#;
        let report = simulate_spec(content, &SimulationConfig::default()).unwrap();
        assert!(report.can_converge());
    }
}
