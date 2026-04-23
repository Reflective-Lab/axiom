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

/// A spending threshold extracted from Cedar policy conditions.
#[derive(Debug, Clone, PartialEq)]
pub struct SpendingThreshold {
    pub amount: u64,
    pub rule_kind: PolicyRuleKind,
    pub source_line: usize,
}

/// Cross-reference report between Truth governance and Cedar policy.
#[derive(Debug, Clone)]
pub struct PolicyCoverageReport {
    pub requirements: PolicyRequirements,
    pub rules: Vec<PolicyRule>,
    pub covered_actions: Vec<String>,
    pub uncovered_actions: Vec<String>,
    pub observations: Vec<String>,
    /// Spending thresholds extracted from Cedar amount/cost conditions.
    pub spending_thresholds: Vec<SpendingThreshold>,
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
        reqs.spending_limits.clone_from(&constraint.cost_limit);
        if !constraint.cost_limit.is_empty() || !constraint.budget.is_empty() {
            reqs.resource_type = Some("spend".into());
        }
    }

    // Exception → escalation
    if let Some(exception) = &governance.exception {
        reqs.escalation_targets.clone_from(&exception.escalates_to);
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

    let spending_thresholds = extract_spending_thresholds(&rules);

    PolicyCoverageReport {
        requirements,
        rules,
        covered_actions: covered,
        uncovered_actions: uncovered,
        observations,
        spending_thresholds,
    }
}

/// Extract spending thresholds from Cedar policy amount/cost conditions.
fn extract_spending_thresholds(rules: &[PolicyRule]) -> Vec<SpendingThreshold> {
    let amount_pattern = regex::Regex::new(r"(?:amount|cost)\s*[><=]+\s*(\d+)").ok();
    let Some(pattern) = &amount_pattern else {
        return Vec::new();
    };

    let mut thresholds = Vec::new();
    for rule in rules {
        for condition in &rule.conditions {
            for cap in pattern.captures_iter(condition) {
                if let Some(amount_str) = cap.get(1)
                    && let Ok(amount) = amount_str.as_str().parse::<u64>()
                {
                    thresholds.push(SpendingThreshold {
                        amount,
                        rule_kind: rule.kind,
                        source_line: rule.source_line,
                    });
                }
            }
        }
    }

    thresholds.sort_by_key(|t| t.amount);
    thresholds.dedup_by_key(|t| t.amount);
    thresholds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truths::{
        AuthorityBlock, ConstraintBlock, EvidenceBlock, ExceptionBlock, TruthGovernance,
        parse_truth_document,
    };
    use proptest::prelude::*;

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

    fn gov_with_authority(actor: &str) -> TruthGovernance {
        TruthGovernance {
            authority: Some(AuthorityBlock {
                actor: Some(actor.into()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    // ── PolicyRuleKind Display ──────────────────────────────────────

    #[test]
    fn display_permit() {
        assert_eq!(PolicyRuleKind::Permit.to_string(), "permit");
    }

    #[test]
    fn display_forbid() {
        assert_eq!(PolicyRuleKind::Forbid.to_string(), "forbid");
    }

    // ── PolicyRuleKind equality ─────────────────────────────────────

    #[test]
    fn rule_kind_equality() {
        assert_eq!(PolicyRuleKind::Permit, PolicyRuleKind::Permit);
        assert_eq!(PolicyRuleKind::Forbid, PolicyRuleKind::Forbid);
        assert_ne!(PolicyRuleKind::Permit, PolicyRuleKind::Forbid);
    }

    // ── Struct constructors ─────────────────────────────────────────

    #[test]
    fn policy_rule_fields() {
        let rule = PolicyRule {
            kind: PolicyRuleKind::Permit,
            action: "deploy".into(),
            conditions: vec!["role == admin".into()],
            source_line: 42,
        };
        assert_eq!(rule.kind, PolicyRuleKind::Permit);
        assert_eq!(rule.action, "deploy");
        assert_eq!(rule.conditions, vec!["role == admin"]);
        assert_eq!(rule.source_line, 42);
    }

    #[test]
    fn policy_requirements_default() {
        let reqs = PolicyRequirements::default();
        assert!(reqs.gated_actions.is_empty());
        assert!(reqs.required_gates.is_empty());
        assert!(reqs.authority_level.is_none());
        assert!(!reqs.requires_human_approval);
        assert!(reqs.resource_type.is_none());
        assert!(reqs.spending_limits.is_empty());
        assert!(reqs.escalation_targets.is_empty());
    }

    #[test]
    fn gated_action_fields() {
        let ga = GatedAction {
            action: "commit".into(),
            reason: "board approval".into(),
        };
        assert_eq!(ga.action, "commit");
        assert_eq!(ga.reason, "board approval");
    }

    #[test]
    fn coverage_report_fields() {
        let report = PolicyCoverageReport {
            requirements: PolicyRequirements::default(),
            rules: vec![],
            covered_actions: vec!["commit".into()],
            uncovered_actions: vec!["promote".into()],
            observations: vec!["note".into()],
            spending_thresholds: vec![],
        };
        assert_eq!(report.covered_actions.len(), 1);
        assert_eq!(report.uncovered_actions.len(), 1);
        assert_eq!(report.observations.len(), 1);
    }

    // ── extract_requirements ────────────────────────────────────────

    #[test]
    fn extracts_requirements_from_governance() {
        let content = r"Truth: Governed vendor selection

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
";
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
    fn extract_empty_governance() {
        let gov = TruthGovernance::default();
        let reqs = extract_requirements(&gov);
        assert!(reqs.gated_actions.is_empty());
        assert!(reqs.required_gates.is_empty());
        assert!(reqs.authority_level.is_none());
        assert!(!reqs.requires_human_approval);
        assert!(reqs.resource_type.is_none());
        assert!(reqs.spending_limits.is_empty());
        assert!(reqs.escalation_targets.is_empty());
    }

    #[test]
    fn extract_authority_board_sets_supervisory() {
        let reqs = extract_requirements(&gov_with_authority("review_board"));
        assert_eq!(reqs.authority_level, Some("supervisory".into()));
    }

    #[test]
    fn extract_authority_committee_sets_supervisory() {
        let reqs = extract_requirements(&gov_with_authority("ethics_committee"));
        assert_eq!(reqs.authority_level, Some("supervisory".into()));
    }

    #[test]
    fn extract_authority_ceo_sets_sovereign() {
        let reqs = extract_requirements(&gov_with_authority("ceo"));
        assert_eq!(reqs.authority_level, Some("sovereign".into()));
    }

    #[test]
    fn extract_authority_cfo_sets_sovereign() {
        let reqs = extract_requirements(&gov_with_authority("cfo_office"));
        assert_eq!(reqs.authority_level, Some("sovereign".into()));
    }

    #[test]
    fn extract_authority_cro_sets_sovereign() {
        let reqs = extract_requirements(&gov_with_authority("cro"));
        assert_eq!(reqs.authority_level, Some("sovereign".into()));
    }

    #[test]
    fn extract_authority_unknown_actor_no_level() {
        let reqs = extract_requirements(&gov_with_authority("ops_team"));
        assert_eq!(reqs.authority_level, None);
    }

    #[test]
    fn extract_authority_no_actor() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock::default()),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.authority_level, None);
        // Authority block present => promote gated action
        assert!(reqs.gated_actions.iter().any(|g| g.action == "promote"));
    }

    #[test]
    fn extract_approval_sets_human_approval_and_gated_commit() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["legal_review".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert!(reqs.requires_human_approval);
        let commit_actions: Vec<_> = reqs
            .gated_actions
            .iter()
            .filter(|g| g.action == "commit")
            .collect();
        assert!(!commit_actions.is_empty());
        assert!(commit_actions[0].reason.contains("legal_review"));
    }

    #[test]
    fn extract_multiple_approvals() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["legal".into(), "finance".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        let commit_gates: Vec<_> = reqs
            .gated_actions
            .iter()
            .filter(|g| g.action == "commit")
            .collect();
        assert_eq!(commit_gates.len(), 2);
    }

    #[test]
    fn extract_must_not_creates_gated_commit() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                must_not: vec!["bypass compliance".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        let must_not_gates: Vec<_> = reqs
            .gated_actions
            .iter()
            .filter(|g| g.reason.contains("Must not"))
            .collect();
        assert_eq!(must_not_gates.len(), 1);
        assert!(must_not_gates[0].reason.contains("bypass compliance"));
    }

    #[test]
    fn extract_authority_always_adds_promote_gate() {
        let gov = gov_with_authority("anyone");
        let reqs = extract_requirements(&gov);
        assert!(reqs.gated_actions.iter().any(|g| g.action == "promote"));
    }

    #[test]
    fn extract_evidence_gates() {
        let gov = TruthGovernance {
            evidence: Some(EvidenceBlock {
                requires: vec!["pen_test".into(), "code_review".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.required_gates, vec!["pen_test", "code_review"]);
    }

    #[test]
    fn extract_constraint_cost_limit_sets_resource_type() {
        let gov = TruthGovernance {
            constraint: Some(ConstraintBlock {
                cost_limit: vec!["10k".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.resource_type, Some("spend".into()));
        assert_eq!(reqs.spending_limits, vec!["10k"]);
    }

    #[test]
    fn extract_constraint_budget_sets_resource_type() {
        let gov = TruthGovernance {
            constraint: Some(ConstraintBlock {
                budget: vec!["quarterly".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.resource_type, Some("spend".into()));
    }

    #[test]
    fn extract_constraint_no_cost_or_budget() {
        let gov = TruthGovernance {
            constraint: Some(ConstraintBlock {
                must_not: vec!["exceed headcount".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.resource_type, None);
    }

    #[test]
    fn extract_exception_escalation_targets() {
        let gov = TruthGovernance {
            exception: Some(ExceptionBlock {
                escalates_to: vec!["cto".into(), "board".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let reqs = extract_requirements(&gov);
        assert_eq!(reqs.escalation_targets, vec!["cto", "board"]);
    }

    // ── parse_cedar_rules ───────────────────────────────────────────

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
    fn parse_empty_policy() {
        let rules = parse_cedar_rules("");
        assert!(rules.is_empty());
    }

    #[test]
    fn parse_whitespace_only_policy() {
        let rules = parse_cedar_rules("   \n\n  \n");
        assert!(rules.is_empty());
    }

    #[test]
    fn parse_comments_only() {
        let rules = parse_cedar_rules("// just a comment\n// another one\n");
        assert!(rules.is_empty());
    }

    #[test]
    fn parse_no_action_in_rule() {
        let policy = r#"
permit(principal, action, resource)
when {
  principal.role == "admin"
};
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].action, "");
        assert_eq!(rules[0].kind, PolicyRuleKind::Permit);
    }

    #[test]
    fn parse_source_line_is_one_indexed() {
        let policy = r#"permit(principal, action == Action::"deploy", resource)
when {
  principal.role == "deployer"
};
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules[0].source_line, 1);
    }

    #[test]
    fn parse_source_line_skips_blank_lines() {
        let policy = "\n\npermit(principal, action == Action::\"x\", resource)\nwhen { };\n";
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules[0].source_line, 3);
    }

    #[test]
    fn parse_condition_with_equality() {
        let policy = r#"
permit(principal, action == Action::"read", resource)
when {
  resource.classification == "public"
};
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules.len(), 1);
        assert!(
            rules[0]
                .conditions
                .iter()
                .any(|c| c.contains("classification"))
        );
    }

    #[test]
    fn parse_condition_with_contains() {
        let policy = r#"
permit(principal, action == Action::"access", resource)
when {
  context.gates_passed.contains("sec_review")
};
"#;
        let rules = parse_cedar_rules(policy);
        assert!(
            rules[0]
                .conditions
                .iter()
                .any(|c| c.contains("gates_passed.contains"))
        );
    }

    #[test]
    fn parse_condition_with_less_equal() {
        let policy = r#"
forbid(principal, action == Action::"spend", resource)
when {
  context.amount <= 1000
};
"#;
        let rules = parse_cedar_rules(policy);
        assert!(!rules[0].conditions.is_empty());
    }

    #[test]
    fn parse_condition_cleans_resource_prefix() {
        let policy = r#"
permit(principal, action == Action::"r", resource)
when {
  resource.owner == "team_a"
};
"#;
        let rules = parse_cedar_rules(policy);
        let owner_cond = rules[0].conditions.iter().find(|c| c.contains("owner"));
        assert!(owner_cond.is_some());
        assert!(!owner_cond.unwrap().contains("resource."));
    }

    #[test]
    fn parse_condition_cleans_principal_prefix() {
        let policy = r#"
permit(principal, action == Action::"r", resource)
when {
  principal.level == "senior"
};
"#;
        let rules = parse_cedar_rules(policy);
        let level_cond = rules[0].conditions.iter().find(|c| c.contains("level"));
        assert!(level_cond.is_some());
        assert!(!level_cond.unwrap().contains("principal."));
    }

    #[test]
    fn parse_condition_replaces_double_equals_false() {
        let policy = r#"
forbid(principal, action == Action::"x", resource)
when {
  context.approved == false
};
"#;
        let rules = parse_cedar_rules(policy);
        assert!(rules[0].conditions.iter().any(|c| c.contains("NOT SET")));
    }

    #[test]
    fn parse_condition_replaces_or_operator() {
        let policy = r#"
permit(principal, action == Action::"x", resource)
when {
  context.level == "a" || context.level == "b"
};
"#;
        let rules = parse_cedar_rules(policy);
        assert!(rules[0].conditions.iter().any(|c| c.contains("OR")));
    }

    #[test]
    fn parse_multiple_rules() {
        let policy = r#"
permit(principal, action == Action::"a", resource) when { };
permit(principal, action == Action::"b", resource) when { };
forbid(principal, action == Action::"c", resource) when { };
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].action, "a");
        assert_eq!(rules[1].action, "b");
        assert_eq!(rules[2].action, "c");
        assert_eq!(rules[2].kind, PolicyRuleKind::Forbid);
    }

    #[test]
    fn parse_nested_conditions_block() {
        let policy = r#"
forbid(principal, action == Action::"deploy", resource)
when {
  context.env == "production" &&
  context.approvals <= 0
};
"#;
        let rules = parse_cedar_rules(policy);
        assert_eq!(rules.len(), 1);
        assert!(rules[0].conditions.len() >= 2);
    }

    // ── check_coverage ──────────────────────────────────────────────

    #[test]
    fn coverage_report_with_vendor_policy() {
        let content = r"Truth: Governed vendor selection

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
";
        let doc = parse_truth_document(content).unwrap();
        let report = check_coverage(&doc.governance, VENDOR_POLICY);

        assert!(!report.covered_actions.is_empty());
        assert!(!report.observations.is_empty());
    }

    #[test]
    fn coverage_empty_governance_empty_policy() {
        let gov = TruthGovernance::default();
        let report = check_coverage(&gov, "");
        assert!(report.covered_actions.is_empty());
        assert!(report.uncovered_actions.is_empty());
        assert!(report.observations.is_empty());
        assert!(report.rules.is_empty());
    }

    #[test]
    fn coverage_no_policy_leaves_actions_uncovered() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["sign_off".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let report = check_coverage(&gov, "");
        assert!(!report.uncovered_actions.is_empty());
        assert!(report.covered_actions.is_empty());
    }

    #[test]
    fn coverage_all_actions_covered() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                actor: Some("ceo".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        // Authority block with no approvals/must_not => only "promote" gated
        let policy = r#"permit(principal, action == Action::"promote", resource) when { };"#;
        let report = check_coverage(&gov, policy);
        assert!(report.covered_actions.contains(&"promote".to_string()));
        assert!(report.uncovered_actions.is_empty());
    }

    #[test]
    fn coverage_deduplicates_actions() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["a".into(), "b".into()],
                must_not: vec!["x".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        // commit appears multiple times in gated_actions, but should dedup in covered
        let policy = r#"
permit(principal, action == Action::"commit", resource) when { };
permit(principal, action == Action::"promote", resource) when { };
"#;
        let report = check_coverage(&gov, policy);
        let commit_count = report
            .covered_actions
            .iter()
            .filter(|a| *a == "commit")
            .count();
        assert_eq!(commit_count, 1);
    }

    #[test]
    fn coverage_removes_covered_from_uncovered() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["sign".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        // commit is both gated (from approval) and covered (in policy)
        // promote is gated (always with authority) but not in policy
        let policy = r#"permit(principal, action == Action::"commit", resource) when { };"#;
        let report = check_coverage(&gov, policy);
        assert!(report.covered_actions.contains(&"commit".to_string()));
        assert!(!report.uncovered_actions.contains(&"commit".to_string()));
        assert!(report.uncovered_actions.contains(&"promote".to_string()));
    }

    #[test]
    fn coverage_observes_missing_human_approval_check() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["manager".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"permit(principal, action == Action::"commit", resource) when { };"#;
        let report = check_coverage(&gov, policy);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.contains("no human_approval check"))
        );
    }

    #[test]
    fn coverage_observes_present_human_approval_check() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                requires_approval: vec!["manager".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"
forbid(principal, action == Action::"commit", resource)
when {
  context.human_approval_present == false
};
"#;
        let report = check_coverage(&gov, policy);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.contains("enforces human approval"))
        );
    }

    #[test]
    fn coverage_observes_missing_gates_check() {
        let gov = TruthGovernance {
            evidence: Some(EvidenceBlock {
                requires: vec!["audit_trail".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"permit(principal, action == Action::"read", resource) when { };"#;
        let report = check_coverage(&gov, policy);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.contains("doesn't check gates_passed"))
        );
    }

    #[test]
    fn coverage_no_gate_observation_when_policy_checks_gates() {
        let gov = TruthGovernance {
            evidence: Some(EvidenceBlock {
                requires: vec!["pen_test".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"
permit(principal, action == Action::"commit", resource)
when {
  context.gates_passed.contains("pen_test")
};
"#;
        let report = check_coverage(&gov, policy);
        assert!(
            !report
                .observations
                .iter()
                .any(|o| o.contains("doesn't check gates_passed"))
        );
    }

    #[test]
    fn coverage_observes_missing_amount_check() {
        let gov = TruthGovernance {
            constraint: Some(ConstraintBlock {
                cost_limit: vec!["50k".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"permit(principal, action == Action::"spend", resource) when { };"#;
        let report = check_coverage(&gov, policy);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.contains("no amount checks"))
        );
    }

    #[test]
    fn coverage_observes_present_amount_check() {
        let gov = TruthGovernance {
            constraint: Some(ConstraintBlock {
                cost_limit: vec!["50k".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"
forbid(principal, action == Action::"spend", resource)
when {
  context.amount > 50000
};
"#;
        let report = check_coverage(&gov, policy);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.contains("enforces spending limits"))
        );
    }

    #[test]
    fn coverage_complex_multi_block_governance() {
        let gov = TruthGovernance {
            authority: Some(AuthorityBlock {
                actor: Some("cfo".into()),
                requires_approval: vec!["legal".into()],
                must_not: vec!["skip_audit".into()],
                ..Default::default()
            }),
            evidence: Some(EvidenceBlock {
                requires: vec!["risk_assessment".into()],
                ..Default::default()
            }),
            constraint: Some(ConstraintBlock {
                cost_limit: vec!["100k".into()],
                ..Default::default()
            }),
            exception: Some(ExceptionBlock {
                escalates_to: vec!["board".into()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let policy = r#"
permit(principal, action == Action::"commit", resource)
when {
  context.human_approval_present == true &&
  context.gates_passed.contains("risk_assessment") &&
  context.amount <= 100000
};
permit(principal, action == Action::"promote", resource)
when {
  principal.role == "cfo"
};
"#;
        let report = check_coverage(&gov, policy);
        assert_eq!(
            report.requirements.authority_level,
            Some("sovereign".into())
        );
        assert!(report.requirements.requires_human_approval);
        assert!(!report.requirements.escalation_targets.is_empty());
        assert!(report.covered_actions.contains(&"commit".to_string()));
        assert!(report.covered_actions.contains(&"promote".to_string()));
        assert!(report.uncovered_actions.is_empty());
    }

    // ── spending thresholds ────────────────────────────────────────

    #[test]
    fn extracts_spending_threshold_from_vendor_policy() {
        let gov = TruthGovernance::default();
        let report = check_coverage(&gov, VENDOR_POLICY);
        assert_eq!(report.spending_thresholds.len(), 1);
        assert_eq!(report.spending_thresholds[0].amount, 50000);
        assert_eq!(
            report.spending_thresholds[0].rule_kind,
            PolicyRuleKind::Forbid
        );
    }

    #[test]
    fn extracts_multiple_thresholds_sorted() {
        let policy = r#"
forbid(principal, action == Action::"commit", resource)
when { context.amount > 100000 };

permit(principal, action == Action::"commit", resource)
when { context.cost <= 10000 };

forbid(principal, action == Action::"commit", resource)
when { context.amount > 50000 };
"#;
        let gov = TruthGovernance::default();
        let report = check_coverage(&gov, policy);
        assert!(report.spending_thresholds.len() >= 2);
        // Sorted by amount
        for w in report.spending_thresholds.windows(2) {
            assert!(w[0].amount <= w[1].amount);
        }
    }

    #[test]
    fn no_thresholds_in_non_financial_policy() {
        let policy = r#"
permit(principal, action == Action::"read", resource)
when { principal.role == "viewer" };
"#;
        let gov = TruthGovernance::default();
        let report = check_coverage(&gov, policy);
        assert!(report.spending_thresholds.is_empty());
    }

    // ── Property tests ──────────────────────────────────────────────

    proptest! {
        #[test]
        fn coverage_total_gte_covered(
            n_approvals in 0..5_usize,
            n_must_not in 0..5_usize,
        ) {
            let gov = TruthGovernance {
                authority: Some(AuthorityBlock {
                    requires_approval: (0..n_approvals).map(|i| format!("a{i}")).collect(),
                    must_not: (0..n_must_not).map(|i| format!("m{i}")).collect(),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let report = check_coverage(&gov, VENDOR_POLICY);
            let total = report.covered_actions.len() + report.uncovered_actions.len();
            prop_assert!(total >= report.covered_actions.len());
            // No duplicates between covered and uncovered
            for a in &report.covered_actions {
                prop_assert!(!report.uncovered_actions.contains(a));
            }
        }

        #[test]
        fn parse_never_panics_on_arbitrary_input(s in ".*") {
            let _ = parse_cedar_rules(&s);
        }

        #[test]
        fn empty_governance_produces_empty_requirements(
            _dummy in 0..1_i32,
        ) {
            let gov = TruthGovernance::default();
            let reqs = extract_requirements(&gov);
            prop_assert!(reqs.gated_actions.is_empty());
            prop_assert!(reqs.required_gates.is_empty());
            prop_assert!(!reqs.requires_human_approval);
        }
    }
}
