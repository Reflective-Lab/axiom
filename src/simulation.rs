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
//! use axiom_truth::simulation::{simulate, SimulationConfig};
//! use axiom_truth::truths::parse_truth_document;
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
    /// Enable vendor-selection-specific pre-flight checks when the spec
    /// appears to describe a vendor/procurement evaluation.
    pub check_vendor_selection: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            require_intent: true,
            require_authority: true,
            require_evidence: true,
            require_assertions: true,
            check_resource_availability: true,
            check_vendor_selection: true,
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

/// Vendor-selection-specific pre-flight coverage.
#[derive(Debug, Clone, Default)]
pub struct VendorSelectionCoverage {
    /// Whether the spec appears to describe vendor/procurement evaluation.
    pub detected: bool,
    /// Number of distinct evaluation dimensions found (compliance, cost, risk, etc.).
    pub evaluation_dimensions: usize,
    /// Vendor names or references found in scenarios.
    pub vendor_references: Vec<String>,
    /// Whether a scoring/ranking criterion is present.
    pub has_ranking_criterion: bool,
    /// Whether a commitment/approval gate is present.
    pub has_commitment_gate: bool,
}

/// The result of simulating a Truth spec.
#[derive(Debug, Clone)]
pub struct SimulationReport {
    pub verdict: Verdict,
    pub findings: Vec<SimulationFinding>,
    pub governance_coverage: GovernanceCoverage,
    pub scenario_count: usize,
    pub resource_summary: ResourceSummary,
    pub vendor_selection: VendorSelectionCoverage,
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
    let vendor_selection = if config.check_vendor_selection {
        check_vendor_selection(&doc.governance, &doc.gherkin, &mut findings)
    } else {
        VendorSelectionCoverage::default()
    };

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
        vendor_selection,
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
        summary.declared_evidence.clone_from(&evidence.requires);
    }

    // Extract resource references from scenario steps
    let resource_pattern = regex::Regex::new(r"[a-z][a-z0-9_]*(?:_[a-z0-9]+)+").ok();

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
    if let Some(authority) = &gov.authority
        && let Some(actor) = &authority.actor
    {
        let actor_referenced = gherkin.contains(actor);
        if !actor_referenced {
            findings.push(SimulationFinding {
                severity: FindingSeverity::Info,
                category: "resources",
                message: format!(
                    "Authority actor `{actor}` is declared but not referenced in any scenario."
                ),
                suggestion: Some(
                    "Consider adding a scenario step that involves the authorized actor.".into(),
                ),
            });
        }
    }

    summary
}

const VENDOR_KEYWORDS: &[&str] = &[
    "vendor",
    "procurement",
    "supplier",
    "rfp",
    "shortlist",
    "sourcing",
];

const EVALUATION_DIMENSIONS: &[&str] = &[
    "compliance",
    "cost",
    "risk",
    "security",
    "capability",
    "stability",
    "performance",
    "pricing",
    "budget",
    "certification",
    "regulatory",
    "timeline",
    "delivery",
];

fn check_vendor_selection(
    gov: &TruthGovernance,
    gherkin: &str,
    findings: &mut Vec<SimulationFinding>,
) -> VendorSelectionCoverage {
    let mut coverage = VendorSelectionCoverage::default();

    let combined = format!(
        "{} {}",
        gov.intent
            .as_ref()
            .and_then(|i| i.outcome.as_deref())
            .unwrap_or(""),
        gherkin
    )
    .to_lowercase();

    coverage.detected = VENDOR_KEYWORDS.iter().any(|kw| combined.contains(kw));
    if !coverage.detected {
        return coverage;
    }

    // Count evaluation dimensions mentioned
    let dimensions: Vec<&str> = EVALUATION_DIMENSIONS
        .iter()
        .copied()
        .filter(|d| combined.contains(d))
        .collect();
    coverage.evaluation_dimensions = dimensions.len();

    if coverage.evaluation_dimensions < 3 {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "vendor-selection",
            message: format!(
                "Vendor selection spec mentions only {} evaluation dimension(s): {}. \
                 At least 3 are recommended for meaningful differentiation.",
                coverage.evaluation_dimensions,
                if dimensions.is_empty() {
                    "none".to_string()
                } else {
                    dimensions.join(", ")
                }
            ),
            suggestion: Some(
                "Add evaluation criteria such as compliance, cost, risk, security, capability."
                    .into(),
            ),
        });
    }

    // Extract vendor references from Given steps
    let vendor_pattern = regex::Regex::new(r#"(?i)(?:vendors?|suppliers?)\s+"([^"]+)""#).ok();
    if let Some(pat) = &vendor_pattern {
        for cap in pat.captures_iter(gherkin) {
            if let Some(names) = cap.get(1) {
                for name in names.as_str().split(',') {
                    let trimmed = name.trim().to_string();
                    if !trimmed.is_empty() && !coverage.vendor_references.contains(&trimmed) {
                        coverage.vendor_references.push(trimmed);
                    }
                }
            }
        }
    }

    if coverage.vendor_references.len() < 3 {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Info,
            category: "vendor-selection",
            message: format!(
                "Only {} vendor(s) referenced in scenarios. \
                 3+ vendors recommended for meaningful comparison.",
                coverage.vendor_references.len()
            ),
            suggestion: Some(
                "Add more vendors in Given steps: Given vendors \"Acme, Beta, Gamma\"".into(),
            ),
        });
    }

    // Check for ranking/shortlist criteria in scenario steps only
    let scenario_text: String = gherkin
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Then ") || t.starts_with("And ")
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    coverage.has_ranking_criterion = scenario_text.contains("rank")
        || scenario_text.contains("shortlist")
        || scenario_text.contains("scored")
        || scenario_text.contains("recommendation");

    if !coverage.has_ranking_criterion {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "vendor-selection",
            message: "No ranking or shortlisting criterion detected.".into(),
            suggestion: Some(
                "Add a Then step asserting a ranked shortlist or recommendation is produced."
                    .into(),
            ),
        });
    }

    // Check for commitment/approval gate
    coverage.has_commitment_gate = gov
        .authority
        .as_ref()
        .is_some_and(|a| !a.requires_approval.is_empty());

    if !coverage.has_commitment_gate {
        findings.push(SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "vendor-selection",
            message: "No commitment approval gate found. Vendor selections with financial \
                      impact should require human approval."
                .into(),
            suggestion: Some(
                "Add `Requires Approval: vendor_commitment` to the Authority block.".into(),
            ),
        });
    }

    coverage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truths::{
        AuthorityBlock, ConstraintBlock, EvidenceBlock, ExceptionBlock, IntentBlock,
        parse_truth_document,
    };

    fn full_spec() -> &'static str {
        r#"Truth: Vendor selection is governed

Intent:
  Outcome: Select a vendor with auditable rationale.
  Goal: Evaluate candidates on cost, compliance, and risk.

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
  Given candidate vendors "Acme AI, Beta ML, Gamma LLM"
  And each vendor has a security_assessment and pricing_analysis
  When the governance_review_board evaluates each vendor
  Then each vendor should produce a compliance screening result
  And a ranked shortlist is produced
"#
    }

    fn minimal_valid_spec() -> &'static str {
        r"Truth: Minimal

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
"
    }

    // ─── Verdict display ───

    #[test]
    fn verdict_display() {
        assert_eq!(Verdict::Ready.to_string(), "ready");
        assert_eq!(Verdict::Risky.to_string(), "risky");
        assert_eq!(Verdict::WillNotConverge.to_string(), "will-not-converge");
    }

    #[test]
    fn finding_severity_display() {
        assert_eq!(FindingSeverity::Info.to_string(), "info");
        assert_eq!(FindingSeverity::Warning.to_string(), "warning");
        assert_eq!(FindingSeverity::Error.to_string(), "error");
    }

    // ─── SimulationReport::can_converge ───

    #[test]
    fn can_converge_ready() {
        let report = SimulationReport {
            verdict: Verdict::Ready,
            findings: vec![],
            governance_coverage: GovernanceCoverage::default(),
            scenario_count: 1,
            resource_summary: ResourceSummary::default(),
            vendor_selection: VendorSelectionCoverage::default(),
        };
        assert!(report.can_converge());
    }

    #[test]
    fn can_converge_risky() {
        let report = SimulationReport {
            verdict: Verdict::Risky,
            findings: vec![],
            governance_coverage: GovernanceCoverage::default(),
            scenario_count: 1,
            resource_summary: ResourceSummary::default(),
            vendor_selection: VendorSelectionCoverage::default(),
        };
        assert!(report.can_converge());
    }

    #[test]
    fn cannot_converge_will_not() {
        let report = SimulationReport {
            verdict: Verdict::WillNotConverge,
            findings: vec![],
            governance_coverage: GovernanceCoverage::default(),
            scenario_count: 0,
            resource_summary: ResourceSummary::default(),
            vendor_selection: VendorSelectionCoverage::default(),
        };
        assert!(!report.can_converge());
    }

    // ─── SimulationFinding construction ───

    #[test]
    fn finding_with_suggestion() {
        let f = SimulationFinding {
            severity: FindingSeverity::Warning,
            category: "test",
            message: "something is off".into(),
            suggestion: Some("fix it".into()),
        };
        assert_eq!(f.severity, FindingSeverity::Warning);
        assert_eq!(f.category, "test");
        assert!(f.suggestion.is_some());
    }

    #[test]
    fn finding_without_suggestion() {
        let f = SimulationFinding {
            severity: FindingSeverity::Info,
            category: "test",
            message: "just info".into(),
            suggestion: None,
        };
        assert!(f.suggestion.is_none());
    }

    // ─── simulate: complete spec ───

    #[test]
    fn complete_spec_is_ready() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::Ready);
        assert!(report.can_converge());
    }

    #[test]
    fn complete_spec_governance_coverage() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.governance_coverage.has_intent);
        assert!(report.governance_coverage.has_outcome);
        assert!(report.governance_coverage.has_authority);
        assert!(report.governance_coverage.has_actor);
        assert!(report.governance_coverage.has_constraint);
        assert!(report.governance_coverage.has_evidence);
        assert_eq!(report.governance_coverage.evidence_count, 2);
    }

    #[test]
    fn complete_spec_scenario_count() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.scenario_count, 1);
    }

    #[test]
    fn complete_spec_resource_summary() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.resource_summary.declared_evidence.len(), 2);
        assert!(report.resource_summary.missing.is_empty());
    }

    // ─── simulate: missing governance blocks ───

    #[test]
    fn missing_intent_will_not_converge() {
        let content = r"Truth: No intent

Scenario: Something happens
  Given a precondition
  When an action occurs
  Then a result is observed
";
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
    fn missing_authority_will_not_converge() {
        let content = r"Truth: No authority

Intent:
  Outcome: Do a thing.

Evidence:
  Requires: proof

Scenario: Action
  Given precondition
  When something happens
  Then outcome
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("Missing Authority"))
        );
    }

    #[test]
    fn missing_evidence_will_not_converge() {
        let content = r"Truth: No evidence

Intent:
  Outcome: Do a thing.

Authority:
  Actor: admin

Scenario: Action
  Given precondition
  When something happens
  Then outcome
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("Missing Evidence"))
        );
    }

    // ─── simulate: scenario issues ───

    #[test]
    fn missing_then_steps_will_not_converge() {
        let content = r"Truth: No assertions

Intent:
  Outcome: Do something.

Authority:
  Actor: admin

Evidence:
  Requires: report

Scenario: Missing outcome
  Given a shortlist of vendors
  When the workflow ranks them
";
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
    fn missing_given_steps_produces_warning() {
        let content = r"Truth: No given

Intent:
  Outcome: Do something.

Authority:
  Actor: admin

Evidence:
  Requires: report

Scenario: No preconditions
  When something happens
  Then it works
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("No Given steps"))
        );
    }

    #[test]
    fn missing_when_steps_produces_warning() {
        let content = r"Truth: No when

Intent:
  Outcome: Do something.

Authority:
  Actor: admin

Evidence:
  Requires: report

Scenario: No action
  Given a state
  Then it is fine
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("No When steps"))
        );
    }

    #[test]
    fn no_scenarios_will_not_converge() {
        let content = r"Truth: Empty

Intent:
  Outcome: Nothing to do.

Authority:
  Actor: admin

Evidence:
  Requires: proof
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("No scenarios found"))
        );
        assert_eq!(report.scenario_count, 0);
    }

    // ─── simulate: coherence checks ───

    #[test]
    fn approval_without_evidence_is_risky() {
        let content = r"Truth: Approval gate without evidence

Intent:
  Outcome: Approve a vendor.

Authority:
  Actor: board
  Requires Approval: cfo_sign_off

Scenario: Approval happens
  Given a vendor is shortlisted
  When the board reviews
  Then the vendor is approved
";
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
    fn constraint_without_authority_warns() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Do it".into()),
                    goal: None,
                }),
                authority: None,
                constraint: Some(ConstraintBlock {
                    budget: vec!["100k".into()],
                    cost_limit: vec![],
                    must_not: vec![],
                }),
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: None,
            },
            gherkin: "Scenario: Test\n  Given a state\n  When action\n  Then result".into(),
        };
        let config = SimulationConfig {
            require_authority: false,
            check_vendor_selection: false,
            ..SimulationConfig::default()
        };
        let report = simulate(&doc, &config);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("who enforces"))
        );
    }

    // ─── simulate: governance detail checks ───

    #[test]
    fn intent_without_outcome_warns() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: None,
                    goal: Some("A goal".into()),
                }),
                authority: Some(AuthorityBlock {
                    actor: Some("admin".into()),
                    ..AuthorityBlock::default()
                }),
                constraint: None,
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: None,
            },
            gherkin: "Scenario: Test\n  Given state\n  When admin acts\n  Then done".into(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("missing Outcome"))
        );
    }

    #[test]
    fn authority_without_actor_warns() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Do it".into()),
                    goal: None,
                }),
                authority: Some(AuthorityBlock::default()),
                constraint: None,
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: None,
            },
            gherkin: "Scenario: Test\n  Given state\n  When action\n  Then done".into(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("missing Actor"))
        );
    }

    #[test]
    fn empty_evidence_requires_warns() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Do it".into()),
                    goal: None,
                }),
                authority: Some(AuthorityBlock {
                    actor: Some("admin".into()),
                    ..AuthorityBlock::default()
                }),
                constraint: None,
                evidence: Some(EvidenceBlock {
                    requires: vec![],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: None,
            },
            gherkin: "Scenario: Test\n  Given state\n  When admin acts\n  Then done".into(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.message.contains("no Requires fields"))
        );
    }

    #[test]
    fn empty_constraint_block_info() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Do it".into()),
                    goal: None,
                }),
                authority: Some(AuthorityBlock {
                    actor: Some("admin".into()),
                    ..AuthorityBlock::default()
                }),
                constraint: Some(ConstraintBlock::default()),
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: None,
            },
            gherkin: "Scenario: Test\n  Given state\n  When admin acts\n  Then done".into(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.findings.iter().any(|f| {
            f.severity == FindingSeverity::Info && f.message.contains("no guardrails")
        }));
    }

    #[test]
    fn exception_with_escalation_path() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Do it".into()),
                    goal: None,
                }),
                authority: Some(AuthorityBlock {
                    actor: Some("admin".into()),
                    ..AuthorityBlock::default()
                }),
                constraint: None,
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec!["log".into()],
                }),
                exception: Some(ExceptionBlock {
                    escalates_to: vec!["ceo".into()],
                    requires: vec![],
                }),
            },
            gherkin: "Scenario: Test\n  Given state\n  When admin acts\n  Then done".into(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.governance_coverage.has_exception);
        assert!(report.governance_coverage.has_escalation_path);
    }

    // ─── SimulationConfig variations ───

    #[test]
    fn relaxed_config_allows_missing_governance() {
        let content = r"Truth: Bare minimum

Scenario: Just do it
  Given a state
  When an action
  Then a result
";
        let doc = parse_truth_document(content).unwrap();
        let config = SimulationConfig {
            require_intent: false,
            require_authority: false,
            require_evidence: false,
            require_assertions: false,
            check_resource_availability: false,
            check_vendor_selection: false,
        };
        let report = simulate(&doc, &config);
        let has_errors = report
            .findings
            .iter()
            .any(|f| f.severity == FindingSeverity::Error);
        assert!(!has_errors);
        assert_ne!(report.verdict, Verdict::WillNotConverge);
    }

    #[test]
    fn default_config_is_strict() {
        let config = SimulationConfig::default();
        assert!(config.require_intent);
        assert!(config.require_authority);
        assert!(config.require_evidence);
        assert!(config.require_assertions);
        assert!(config.check_resource_availability);
    }

    // ─── simulate_spec convenience ───

    #[test]
    fn simulate_spec_convenience() {
        let report = simulate_spec(minimal_valid_spec(), &SimulationConfig::default()).unwrap();
        assert!(report.can_converge());
    }

    #[test]
    fn simulate_spec_garbage_input() {
        let result = simulate_spec(
            "this is not a truth spec at all",
            &SimulationConfig::default(),
        );
        // Parser may be lenient; either an error or a WillNotConverge verdict is acceptable
        match result {
            Err(_) => {}
            Ok(report) => assert_eq!(report.verdict, Verdict::WillNotConverge),
        }
    }

    #[test]
    fn simulate_spec_empty_string() {
        let result = simulate_spec("", &SimulationConfig::default());
        match result {
            Err(_) => {}
            Ok(report) => assert_eq!(report.verdict, Verdict::WillNotConverge),
        }
    }

    // ─── resource checks ───

    #[test]
    fn undeclared_evidence_like_resource_warns() {
        let content = r"Truth: Resource mismatch

Intent:
  Outcome: Check resources.

Authority:
  Actor: admin

Evidence:
  Requires: security_assessment
  Audit: decision_log

Scenario: Uses undeclared evidence
  Given a vendor
  When admin reviews the compliance_report
  Then the security_assessment is valid
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .resource_summary
                .missing
                .contains(&"compliance_report".to_string())
        );
    }

    #[test]
    fn actor_not_referenced_in_scenarios_info() {
        let content = r"Truth: Unused actor

Intent:
  Outcome: Something.

Authority:
  Actor: mysterious_committee

Evidence:
  Requires: proof

Scenario: Nobody calls the actor
  Given a state
  When something happens
  Then it works
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.findings.iter().any(|f| {
            f.severity == FindingSeverity::Info
                && f.message.contains("mysterious_committee")
                && f.message.contains("not referenced")
        }));
    }

    // ─── multiple scenarios ───

    #[test]
    fn multiple_scenarios_counted() {
        let content = r"Truth: Multi-scenario

Intent:
  Outcome: Test multiple.

Authority:
  Actor: admin

Evidence:
  Requires: proof

Scenario: First
  Given state
  When admin acts
  Then result

Scenario: Second
  Given another state
  When admin acts again
  Then another result
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.scenario_count, 2);
    }

    // ─── only governance, no scenarios ───

    #[test]
    fn spec_with_only_governance_no_scenarios() {
        let doc = TruthDocument {
            governance: TruthGovernance {
                intent: Some(IntentBlock {
                    outcome: Some("Goal".into()),
                    goal: None,
                }),
                authority: Some(AuthorityBlock {
                    actor: Some("admin".into()),
                    ..AuthorityBlock::default()
                }),
                constraint: None,
                evidence: Some(EvidenceBlock {
                    requires: vec!["proof".into()],
                    provenance: vec![],
                    audit: vec![],
                }),
                exception: None,
            },
            gherkin: String::new(),
        };
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        assert_eq!(report.scenario_count, 0);
    }

    // ─── nil governance (all None) ───

    #[test]
    fn nil_governance_with_relaxed_config() {
        let content = r"Truth: Nil governance

Scenario: Solo
  Given x
  When y
  Then z
";
        let doc = parse_truth_document(content).unwrap();
        let config = SimulationConfig {
            require_intent: false,
            require_authority: false,
            require_evidence: false,
            require_assertions: true,
            check_resource_availability: false,
            check_vendor_selection: false,
        };
        let report = simulate(&doc, &config);
        assert!(!report.governance_coverage.has_intent);
        assert!(!report.governance_coverage.has_authority);
        assert!(!report.governance_coverage.has_evidence);
        assert_eq!(report.verdict, Verdict::Ready);
    }

    #[test]
    fn nil_governance_with_strict_config() {
        let content = r"Truth: Nil governance strict

Scenario: Solo
  Given x
  When y
  Then z
";
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert_eq!(report.verdict, Verdict::WillNotConverge);
        let error_count = report
            .findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Error)
            .count();
        assert!(error_count >= 3); // intent, authority, evidence
    }

    // ─── vendor selection checks ───

    #[test]
    fn vendor_spec_detected() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.detected);
    }

    #[test]
    fn vendor_spec_extracts_vendor_names() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(
            report
                .vendor_selection
                .vendor_references
                .contains(&"Acme AI".to_string())
        );
        assert!(
            report
                .vendor_selection
                .vendor_references
                .contains(&"Beta ML".to_string())
        );
    }

    #[test]
    fn vendor_spec_counts_evaluation_dimensions() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.evaluation_dimensions >= 2);
    }

    #[test]
    fn vendor_spec_detects_approval_gate() {
        let doc = parse_truth_document(full_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.has_commitment_gate);
    }

    #[test]
    fn non_vendor_spec_not_detected() {
        let doc = parse_truth_document(minimal_valid_spec()).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(!report.vendor_selection.detected);
    }

    #[test]
    fn vendor_spec_few_dimensions_warns() {
        let content = r#"Truth: Thin vendor eval

Intent:
  Outcome: Select a vendor.

Authority:
  Actor: admin
  Requires Approval: commitment

Evidence:
  Requires: proof

Scenario: Pick a vendor
  Given vendors "Acme"
  When evaluated
  Then a recommendation is produced
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.detected);
        assert!(report.findings.iter().any(|f| {
            f.category == "vendor-selection" && f.message.contains("evaluation dimension")
        }));
    }

    #[test]
    fn vendor_spec_no_ranking_warns() {
        let content = r#"Truth: No ranking vendor eval

Intent:
  Outcome: Select a vendor with compliance and cost and risk analysis.

Authority:
  Actor: board
  Requires Approval: commitment

Evidence:
  Requires: compliance_report

Scenario: Evaluate vendors
  Given vendors "Acme, Beta, Gamma"
  When the board evaluates
  Then all vendors are screened
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.detected);
        assert!(
            report
                .findings
                .iter()
                .any(|f| { f.category == "vendor-selection" && f.message.contains("ranking") })
        );
    }

    #[test]
    fn vendor_spec_no_approval_gate_warns() {
        let content = r#"Truth: No approval vendor eval

Intent:
  Outcome: Select a vendor with compliance and cost and risk.

Authority:
  Actor: admin

Evidence:
  Requires: report

Scenario: Pick vendor
  Given vendors "Acme, Beta, Gamma"
  When evaluated
  Then a ranked shortlist is produced
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.detected);
        assert!(report.findings.iter().any(|f| {
            f.category == "vendor-selection" && f.message.contains("commitment approval gate")
        }));
    }

    #[test]
    fn vendor_check_disabled() {
        let content = r#"Truth: Vendor eval

Intent:
  Outcome: Select a vendor.

Authority:
  Actor: admin

Evidence:
  Requires: proof

Scenario: Quick
  Given vendors "A"
  When checked
  Then done
"#;
        let doc = parse_truth_document(content).unwrap();
        let config = SimulationConfig {
            check_vendor_selection: false,
            ..SimulationConfig::default()
        };
        let report = simulate(&doc, &config);
        assert!(!report.vendor_selection.detected);
        assert!(
            !report
                .findings
                .iter()
                .any(|f| f.category == "vendor-selection")
        );
    }

    #[test]
    fn vendor_spec_complete_no_vendor_warnings() {
        let content = r#"Truth: Complete vendor selection

Intent:
  Outcome: Select a vendor with compliance, cost, risk, security, and capability analysis.

Authority:
  Actor: governance_review_board
  Requires Approval: vendor_commitment

Constraint:
  Cost Limit: annual spend within budget.

Evidence:
  Requires: compliance_assessment
  Requires: risk_assessment
  Requires: cost_analysis
  Audit: decision_log

Scenario: Full evaluation
  Given vendors "Acme AI, Beta ML, Gamma LLM"
  And each vendor has compliance and risk data
  When the governance_review_board evaluates
  Then a ranked shortlist is produced
  And the recommendation has evidence from all criteria
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = simulate(&doc, &SimulationConfig::default());
        assert!(report.vendor_selection.detected);
        assert!(report.vendor_selection.evaluation_dimensions >= 3);
        assert!(report.vendor_selection.has_ranking_criterion);
        assert!(report.vendor_selection.has_commitment_gate);
        assert_eq!(report.vendor_selection.vendor_references.len(), 3);
        assert!(!report.findings.iter().any(|f| {
            f.category == "vendor-selection" && f.severity == FindingSeverity::Warning
        }));
    }
}
