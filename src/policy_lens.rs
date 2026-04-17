// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Cedar policy lens for Converge Truths.
//!
//! Extracts policy requirements from a Truth's governance blocks and maps
//! them to Cedar policy concepts. Makes the Cedar policy layer visible to
//! the tool layer without requiring the full `converge-policy` engine.
//!
//! This module:
//! - Extracts required gates, authority levels, and resource types from governance blocks
//! - Parses Cedar policy text to extract human-readable rule summaries
//! - Cross-references Truth governance with Cedar policies to show coverage gaps

use crate::truths::TruthGovernance;

/// A Cedar policy rule extracted from policy text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRule {
    pub kind: PolicyRuleKind,
    pub action: String,
    pub conditions: Vec<String>,
    pub source_line: usize,
}

/// Whether the rule permits or forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyRuleKind {
    Permit,
    Forbid,
}

impl std::fmt::Display for PolicyRuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Permit => write!(f, "permit"),
            Self::Forbid => write!(f, "forbid"),
        }
    }
}

/// What a Truth's governance blocks require from the policy layer.
#[derive(Debug, Clone, Default)]
pub struct PolicyRequirements {
    /// Actions the Truth expects to be gated (derived from Authority block).
    pub gated_actions: Vec<GatedAction>,
    /// Gates that must be passed (derived from Evidence.Requires).
    pub required_gates: Vec<String>,
    /// Authority level needed (derived from Authority block).
    pub authority_level: Option<String>,
    /// Whether human approval is required (derived from Authority.Requires Approval).
    pub requires_human_approval: bool,
    /// Resource type (derived from Constraint block context).
    pub resource_type: Option<String>,
    /// Spending limits (derived from Constraint.Cost Limit).
    pub spending_limits: Vec<String>,
    /// Escalation targets (derived from Exception.Escalates To).
    pub escalation_targets: Vec<String>,
}

/// An action that the Truth expects to be policy-gated.
#[derive(Debug, Clone)]
pub struct GatedAction {
    pub action: String,
    pub reason: String,
}

/// Cross-reference report between Truth governance and Cedar policy.
#[derive(Debug, Clone)]
pub struct PolicyCoverageReport {
    pub requirements: PolicyRequirements,
    pub rules: Vec<PolicyRule>,
    pub covered_actions: Vec<String>,
    pub uncovered_actions: Vec<String>,
    pub observations: Vec<String>,
}

/// Extract policy requirements from a Truth's governance blocks.
pub fn extract_requirements(governance: &TruthGovernance) -> PolicyRequirements {
    let mut reqs = PolicyRequirements::default();

    // Authority → gated actions + authority level
    if let Some(authority) = &governance.authority {
        if let Some(actor) = &authority.actor {
            // Derive authority level hint from actor name
            if actor.contains("board") || actor.contains("committee") {
                reqs.authority_level = Some("supervisory".into());
            } else if actor.contains("ceo") || actor.contains("cfo") || actor.contains("cro") {
                reqs.authority_level = Some("sovereign".into());
            }
        }

        if !authority.requires_approval.is_empty() {
            reqs.requires_human_approval = true;
            for approval in &authority.requires_approval {
                reqs.gated_actions.push(GatedAction {
                    action: "commit".into(),
                    reason: format!("Requires approval: {approval}"),
                });
            }
        }

        if !authority.must_not.is_empty() {
            for prohibition in &authority.must_not {
                reqs.gated_actions.push(GatedAction {
                    action: "commit".into(),
                    reason: format!("Must not: {prohibition}"),
                });
            }
        }

        // All truths with authority blocks imply gated promote and commit
        reqs.gated_actions.push(GatedAction {
            action: "promote".into(),
            reason: "Authority block present — promotion requires authorization.".into(),
        });
    }

    // Evidence → required gates
    if let Some(evidence) = &governance.evidence {
        for req in &evidence.requires {
            reqs.required_gates.push(req.clone());
        }
    }

    // Constraint → spending limits + resource type hints
    if let Some(constraint) = &governance.constraint {
        reqs.spending_limits = constraint.cost_limit.clone();
        if !constraint.cost_limit.is_empty() || !constraint.budget.is_empty() {
            reqs.resource_type = Some("spend".into());
        }
    }

    // Exception → escalation
    if let Some(exception) = &governance.exception {
        reqs.escalation_targets = exception.escalates_to.clone();
    }

    reqs
}

/// Parse Cedar policy text into structured rules.
pub fn parse_cedar_rules(policy_text: &str) -> Vec<PolicyRule> {
    let mut rules = Vec::new();
    let lines: Vec<&str> = policy_text.lines().collect();
    let mut idx = 0;

    while idx < lines.len() {
        let trimmed = lines[idx].trim();

        let kind = if trimmed.starts_with("permit(") {
            Some(PolicyRuleKind::Permit)
        } else if trimmed.starts_with("forbid(") {
            Some(PolicyRuleKind::Forbid)
        } else {
            None
        };

        if let Some(kind) = kind {
            let source_line = idx + 1;
            let mut action = String::new();
            let mut conditions = Vec::new();

            // Extract action from the rule head
            if let Some(action_start) = trimmed.find("Action::\"") {
                let after = &trimmed[action_start + 9..];
                if let Some(end) = after.find('"') {
                    action = after[..end].to_string();
                }
            }

            // Collect when/conditions block
            let mut depth = 0_i32;
            let start = idx;
            for line in &lines[start..] {
                depth += line.matches('{').count() as i32;
                depth -= line.matches('}').count() as i32;

                let condition = line.trim();
                if condition.starts_with("//") || condition.is_empty() {
                    // skip
                } else if condition.contains("==")
                    || condition.contains("<=")
                    || condition.contains('>')
                    || condition.contains(".contains(")
                {
                    // Clean up Cedar syntax for display
                    let clean = condition
                        .replace("resource.", "")
                        .replace("principal.", "")
                        .replace("context.", "")
                        .replace("&&", "")
                        .replace("||", "OR")
                        .replace("== false", "NOT SET")
                        .trim()
                        .trim_end_matches(';')
                        .trim()
                        .to_string();
                    if !clean.is_empty() {
                        conditions.push(clean);
                    }
                }

                idx += 1;
                if depth <= 0 && line.contains(';') {
                    break;
                }
            }

            rules.push(PolicyRule {
                kind,
                action,
                conditions,
                source_line,
            });
        } else {
            idx += 1;
        }
    }

    rules
}

/// Cross-reference Truth governance requirements with Cedar policy rules.
pub fn check_coverage(governance: &TruthGovernance, policy_text: &str) -> PolicyCoverageReport {
    let requirements = extract_requirements(governance);
    let rules = parse_cedar_rules(policy_text);

    let policy_actions: Vec<String> = rules
        .iter()
        .map(|r| r.action.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let mut covered = Vec::new();
    let mut uncovered = Vec::new();

    for gated in &requirements.gated_actions {
        if policy_actions.contains(&gated.action) {
            covered.push(gated.action.clone());
        } else {
            uncovered.push(gated.action.clone());
        }
    }

    // Deduplicate
    covered.sort();
    covered.dedup();
    uncovered.sort();
    uncovered.dedup();
    // Remove from uncovered if also covered
    uncovered.retain(|a| !covered.contains(a));

    let mut observations = Vec::new();

    if requirements.requires_human_approval {
        let has_approval_check = rules.iter().any(|r| {
            r.conditions
                .iter()
                .any(|c| c.contains("human_approval_present"))
        });
        if has_approval_check {
            observations.push("Cedar policy enforces human approval requirement.".into());
        } else {
            observations.push(
                "Truth requires approval but Cedar policy has no human_approval check.".into(),
            );
        }
    }

    if !requirements.required_gates.is_empty() {
        let gates_in_policy: Vec<&str> = rules
            .iter()
            .flat_map(|r| r.conditions.iter())
            .filter(|c| c.contains("gates_passed.contains"))
            .map(std::string::String::as_str)
            .collect();

        if gates_in_policy.is_empty() {
            observations.push(
                "Truth declares evidence gates but Cedar policy doesn't check gates_passed.".into(),
            );
        }
    }

    if !requirements.spending_limits.is_empty() {
        let has_amount_check = rules
            .iter()
            .any(|r| r.conditions.iter().any(|c| c.contains("amount")));
        if has_amount_check {
            observations.push("Cedar policy enforces spending limits.".into());
        } else {
            observations
                .push("Truth declares cost limits but Cedar policy has no amount checks.".into());
        }
    }

    PolicyCoverageReport {
        requirements,
        rules,
        covered_actions: covered,
        uncovered_actions: uncovered,
        observations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truths::parse_truth_document;

    const VENDOR_POLICY: &str = r#"
permit(principal, action == Action::"promote", resource)
when {
  principal.role == "governance_reviewer"
};

forbid(principal, action == Action::"commit", resource)
when {
  context.cost > 50000 &&
  context.cfo_approval == false
};
"#;

    #[test]
    fn extracts_requirements_from_governance() {
        let content = r#"Truth: Governed vendor selection

Intent:
  Outcome: Select a vendor.

Authority:
  Actor: governance_review_board
  Requires Approval: final_vendor_selection

Constraint:
  Cost Limit: first-year spend within budget.

Evidence:
  Requires: security_assessment
  Requires: pricing_analysis
  Audit: decision_log

Scenario: Vendors evaluated
  Given vendors exist
  When evaluated
  Then one is selected
"#;
        let doc = parse_truth_document(content).unwrap();
        let reqs = extract_requirements(&doc.governance);

        assert!(reqs.requires_human_approval);
        assert_eq!(reqs.required_gates.len(), 2);
        assert!(
            reqs.required_gates
                .contains(&"security_assessment".to_string())
        );
        assert_eq!(reqs.resource_type, Some("spend".into()));
        assert!(reqs.authority_level.is_some());
    }

    #[test]
    fn parses_cedar_rules() {
        let policy = r#"
permit(principal, action == Action::"validate", resource)
when {
  principal.authority == "supervisory"
};

forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 20000 &&
  context.human_approval_present == false
};
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].kind, PolicyRuleKind::Permit);
        assert_eq!(rules[0].action, "validate");
        assert_eq!(rules[1].kind, PolicyRuleKind::Forbid);
        assert_eq!(rules[1].action, "commit");
    }

    #[test]
    fn coverage_report_with_vendor_policy() {
        let content = r#"Truth: Governed vendor selection

Intent:
  Outcome: Select a vendor.

Authority:
  Actor: governance_review_board
  Requires Approval: cfo_sign_off

Constraint:
  Cost Limit: max 50k first year.

Evidence:
  Requires: due_diligence_report
  Audit: decision_log

Scenario: Vendors evaluated
  Given vendors exist
  When evaluated
  Then one is selected
"#;
        let doc = parse_truth_document(content).unwrap();
        let report = check_coverage(&doc.governance, VENDOR_POLICY);

        assert!(!report.covered_actions.is_empty());
        assert!(!report.observations.is_empty());
    }
}
