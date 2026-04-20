// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Validation response building for Converge Truth specs.
//!
//! Transforms raw `SpecValidation` results into structured step views
//! and summaries suitable for UI consumption.

use serde::Serialize;

use crate::gherkin::{IssueCategory, Severity, SpecValidation, ValidationConfig, ValidationIssue};
use crate::truths::TruthGovernance;

/// A validation step with status and summary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationStep {
    pub id: &'static str,
    pub label: &'static str,
    pub status: &'static str,
    pub summary: String,
    pub detail: Option<String>,
}

/// Governance block presence flags.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceFlags {
    pub intent: bool,
    pub authority: bool,
    pub constraint: bool,
    pub evidence: bool,
    pub exception: bool,
}

/// Build validation steps from a successful parse + validation.
pub fn build_steps(validation: &SpecValidation, config: &ValidationConfig) -> Vec<ValidationStep> {
    let convention_issues: Vec<_> = validation
        .issues
        .iter()
        .filter(|issue| matches!(issue.category, IssueCategory::Convention))
        .collect();
    let business_issues: Vec<_> = validation
        .issues
        .iter()
        .filter(|issue| matches!(issue.category, IssueCategory::BusinessSense))
        .collect();

    vec![
        ValidationStep {
            id: "syntax",
            label: "Syntax",
            status: "ok",
            summary: "Truth declarations and Gherkin structure parsed successfully.".into(),
            detail: None,
        },
        semantics_step(&convention_issues),
        business_analysis_step(config, Some(&business_issues)),
    ]
}

/// Build steps for a parse error (syntax failed).
pub fn build_parse_error_steps(message: &str, config: &ValidationConfig) -> Vec<ValidationStep> {
    vec![
        ValidationStep {
            id: "syntax",
            label: "Syntax",
            status: "issue",
            summary: "The Truth or Feature document could not be parsed.".into(),
            detail: Some(message.to_string()),
        },
        ValidationStep {
            id: "semantics",
            label: "Semantics",
            status: "unavailable",
            summary: "Governance and convention checks did not run because parsing failed.".into(),
            detail: None,
        },
        business_analysis_step(config, None),
    ]
}

/// Generate a human-readable summary of validation results.
pub fn summarize(validation: &SpecValidation) -> String {
    let errors = validation
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    let warnings = validation
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Warning)
        .count();
    let sw = pluralize("scenario", validation.scenario_count);

    if errors == 0 && warnings == 0 {
        format!(
            "Local checks passed across {} {}.",
            validation.scenario_count, sw
        )
    } else if errors == 0 {
        format!(
            "Local checks passed with {} {} across {} {}.",
            warnings,
            pluralize("warning", warnings),
            validation.scenario_count,
            sw
        )
    } else {
        format!(
            "Local checks found {} {} and {} {} across {} {}.",
            errors,
            pluralize("error", errors),
            warnings,
            pluralize("warning", warnings),
            validation.scenario_count,
            sw
        )
    }
}

/// Build governance flags from parsed governance blocks.
pub fn governance_flags(governance: &TruthGovernance) -> GovernanceFlags {
    GovernanceFlags {
        intent: governance.intent.is_some(),
        authority: governance.authority.is_some(),
        constraint: governance.constraint.is_some(),
        evidence: governance.evidence.is_some(),
        exception: governance.exception.is_some(),
    }
}

/// Offline validation note explaining what was and wasn't checked.
pub fn offline_note() -> String {
    "Local validation checks Converge Truth parsing, governance blocks, and Gherkin conventions. \
     Business-sense and compilability checks stay disabled until a live ChatBackend validator is configured."
        .into()
}

// ─── Internal ───

fn semantics_step(issues: &[&ValidationIssue]) -> ValidationStep {
    if issues.is_empty() {
        return ValidationStep {
            id: "semantics",
            label: "Semantics",
            status: "ok",
            summary: "Governance blocks and scenario conventions look consistent.".into(),
            detail: None,
        };
    }

    ValidationStep {
        id: "semantics",
        label: "Semantics",
        status: "issue",
        summary: format!(
            "{} governance or convention {} need attention.",
            issues.len(),
            pluralize("rule", issues.len())
        ),
        detail: issue_detail(issues),
    }
}

fn business_analysis_step(
    config: &ValidationConfig,
    issues: Option<&[&ValidationIssue]>,
) -> ValidationStep {
    if !config.check_business_sense {
        return ValidationStep {
            id: "business-analysis",
            label: "Business Analysis",
            status: "unavailable",
            summary: "Business analysis is disabled in offline mode.".into(),
            detail: Some(offline_note()),
        };
    }

    match issues {
        None | Some([]) => ValidationStep {
            id: "business-analysis",
            label: "Business Analysis",
            status: "ok",
            summary: "Business analysis completed without findings.".into(),
            detail: None,
        },
        Some(issues) => ValidationStep {
            id: "business-analysis",
            label: "Business Analysis",
            status: "issue",
            summary: format!(
                "{} business-analysis {} need attention.",
                issues.len(),
                pluralize("finding", issues.len())
            ),
            detail: issue_detail(issues),
        },
    }
}

fn issue_detail(issues: &[&ValidationIssue]) -> Option<String> {
    let mut lines = Vec::new();
    for issue in issues.iter().take(3) {
        let mut line = format!("{}: {}", issue.location, issue.message);
        if let Some(suggestion) = &issue.suggestion {
            line.push_str(&format!(" Suggestion: {suggestion}"));
        }
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn pluralize(word: &'static str, count: usize) -> &'static str {
    if count == 1 {
        word
    } else {
        match word {
            "warning" => "warnings",
            "error" => "errors",
            "scenario" => "scenarios",
            "rule" => "rules",
            "finding" => "findings",
            _ => word,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_validation(scenario_count: usize, issues: Vec<ValidationIssue>) -> SpecValidation {
        SpecValidation {
            is_valid: issues.is_empty(),
            file_path: "test.truth".into(),
            scenario_count,
            issues,
            confidence: 1.0,
            scenario_metas: vec![],
            governance: TruthGovernance::default(),
        }
    }

    fn make_issue(category: IssueCategory, severity: Severity) -> ValidationIssue {
        ValidationIssue {
            location: "Scenario: test".into(),
            category,
            severity,
            message: format!("{severity:?} in {category:?}"),
            suggestion: None,
        }
    }

    fn offline_config() -> ValidationConfig {
        ValidationConfig {
            check_business_sense: false,
            check_compilability: false,
            check_conventions: true,
            min_confidence: 0.0,
        }
    }

    fn online_config() -> ValidationConfig {
        ValidationConfig {
            check_business_sense: true,
            check_compilability: true,
            check_conventions: true,
            min_confidence: 0.8,
        }
    }

    // ─── summarize ───

    #[test]
    fn summary_no_issues() {
        let v = make_validation(2, vec![]);
        assert_eq!(summarize(&v), "Local checks passed across 2 scenarios.");
    }

    #[test]
    fn summary_single_scenario_no_issues() {
        let v = make_validation(1, vec![]);
        assert_eq!(summarize(&v), "Local checks passed across 1 scenario.");
    }

    #[test]
    fn summary_warnings_only() {
        let v = make_validation(
            3,
            vec![make_issue(IssueCategory::Convention, Severity::Warning)],
        );
        let s = summarize(&v);
        assert_eq!(s, "Local checks passed with 1 warning across 3 scenarios.");
    }

    #[test]
    fn summary_multiple_warnings() {
        let v = make_validation(
            2,
            vec![
                make_issue(IssueCategory::Convention, Severity::Warning),
                make_issue(IssueCategory::Convention, Severity::Warning),
                make_issue(IssueCategory::Convention, Severity::Warning),
            ],
        );
        let s = summarize(&v);
        assert_eq!(s, "Local checks passed with 3 warnings across 2 scenarios.");
    }

    #[test]
    fn summary_errors_only() {
        let v = make_validation(
            4,
            vec![make_issue(IssueCategory::Convention, Severity::Error)],
        );
        let s = summarize(&v);
        assert_eq!(
            s,
            "Local checks found 1 error and 0 warnings across 4 scenarios."
        );
    }

    #[test]
    fn summary_errors_and_warnings() {
        let v = make_validation(
            5,
            vec![
                make_issue(IssueCategory::Convention, Severity::Error),
                make_issue(IssueCategory::Convention, Severity::Error),
                make_issue(IssueCategory::Convention, Severity::Warning),
            ],
        );
        let s = summarize(&v);
        assert_eq!(
            s,
            "Local checks found 2 errors and 1 warning across 5 scenarios."
        );
    }

    #[test]
    fn summary_zero_scenarios() {
        let v = make_validation(0, vec![]);
        assert_eq!(summarize(&v), "Local checks passed across 0 scenarios.");
    }

    // ─── build_steps ───

    #[test]
    fn build_steps_all_pass() {
        let v = make_validation(2, vec![]);
        let steps = build_steps(&v, &offline_config());
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].id, "syntax");
        assert_eq!(steps[0].status, "ok");
        assert_eq!(steps[1].id, "semantics");
        assert_eq!(steps[1].status, "ok");
        assert_eq!(steps[2].id, "business-analysis");
        assert_eq!(steps[2].status, "unavailable");
    }

    #[test]
    fn build_steps_convention_issues_flag_semantics() {
        let v = make_validation(
            1,
            vec![make_issue(IssueCategory::Convention, Severity::Warning)],
        );
        let steps = build_steps(&v, &offline_config());
        assert_eq!(steps[1].status, "issue");
        assert!(steps[1].summary.contains('1'));
    }

    #[test]
    fn build_steps_multiple_convention_issues() {
        let v = make_validation(
            1,
            vec![
                make_issue(IssueCategory::Convention, Severity::Warning),
                make_issue(IssueCategory::Convention, Severity::Error),
            ],
        );
        let steps = build_steps(&v, &offline_config());
        assert_eq!(steps[1].status, "issue");
        assert!(steps[1].summary.contains('2'));
    }

    #[test]
    fn build_steps_business_issues_with_online_config() {
        let v = make_validation(
            1,
            vec![make_issue(IssueCategory::BusinessSense, Severity::Warning)],
        );
        let steps = build_steps(&v, &online_config());
        assert_eq!(steps[2].id, "business-analysis");
        assert_eq!(steps[2].status, "issue");
        assert!(steps[2].summary.contains('1'));
    }

    #[test]
    fn build_steps_no_business_issues_online() {
        let v = make_validation(2, vec![]);
        let steps = build_steps(&v, &online_config());
        assert_eq!(steps[2].status, "ok");
    }

    #[test]
    fn build_steps_only_business_issues_semantics_still_ok() {
        let v = make_validation(
            1,
            vec![make_issue(IssueCategory::BusinessSense, Severity::Error)],
        );
        let steps = build_steps(&v, &online_config());
        assert_eq!(steps[1].status, "ok");
        assert_eq!(steps[2].status, "issue");
    }

    // ─── build_parse_error_steps ───

    #[test]
    fn parse_error_steps() {
        let config = offline_config();
        let steps = build_parse_error_steps("bad syntax", &config);
        assert_eq!(steps[0].status, "issue");
        assert_eq!(steps[1].status, "unavailable");
        assert_eq!(steps[2].status, "unavailable");
    }

    #[test]
    fn parse_error_preserves_message() {
        let steps = build_parse_error_steps("unexpected token at line 5", &offline_config());
        assert_eq!(
            steps[0].detail.as_deref(),
            Some("unexpected token at line 5")
        );
    }

    #[test]
    fn parse_error_with_online_config() {
        let steps = build_parse_error_steps("oops", &online_config());
        assert_eq!(steps[0].status, "issue");
        assert_eq!(steps[1].status, "unavailable");
        // Business analysis still runs (None issues => ok)
        assert_eq!(steps[2].status, "ok");
    }

    #[test]
    fn parse_error_empty_message() {
        let steps = build_parse_error_steps("", &offline_config());
        assert_eq!(steps[0].detail.as_deref(), Some(""));
    }

    // ─── governance_flags ───

    #[test]
    fn governance_flags_all_none() {
        let gov = TruthGovernance::default();
        let flags = governance_flags(&gov);
        assert!(!flags.intent);
        assert!(!flags.authority);
        assert!(!flags.constraint);
        assert!(!flags.evidence);
        assert!(!flags.exception);
    }

    #[test]
    fn governance_flags_all_present() {
        use crate::truths::*;
        let gov = TruthGovernance {
            intent: Some(IntentBlock::default()),
            authority: Some(AuthorityBlock::default()),
            constraint: Some(ConstraintBlock::default()),
            evidence: Some(EvidenceBlock::default()),
            exception: Some(ExceptionBlock::default()),
        };
        let flags = governance_flags(&gov);
        assert!(flags.intent);
        assert!(flags.authority);
        assert!(flags.constraint);
        assert!(flags.evidence);
        assert!(flags.exception);
    }

    #[test]
    fn governance_flags_partial() {
        use crate::truths::*;
        let gov = TruthGovernance {
            intent: Some(IntentBlock::default()),
            authority: None,
            constraint: Some(ConstraintBlock::default()),
            evidence: None,
            exception: Some(ExceptionBlock::default()),
        };
        let flags = governance_flags(&gov);
        assert!(flags.intent);
        assert!(!flags.authority);
        assert!(flags.constraint);
        assert!(!flags.evidence);
        assert!(flags.exception);
    }

    // ─── offline_note ───

    #[test]
    fn offline_note_contains_key_phrases() {
        let note = offline_note();
        assert!(note.contains("Local validation"));
        assert!(note.contains("ChatBackend"));
    }

    #[test]
    fn offline_note_is_nonempty() {
        assert!(!offline_note().is_empty());
    }

    // ─── edge cases ───

    #[test]
    fn issue_with_suggestion_appears_in_detail() {
        let v = SpecValidation {
            is_valid: false,
            file_path: "test.truth".into(),
            scenario_count: 1,
            issues: vec![ValidationIssue {
                location: "Scenario: X".into(),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: "step is vague".into(),
                suggestion: Some("be more specific".into()),
            }],
            confidence: 0.8,
            scenario_metas: vec![],
            governance: TruthGovernance::default(),
        };
        let steps = build_steps(&v, &offline_config());
        let detail = steps[1].detail.as_ref().unwrap();
        assert!(detail.contains("step is vague"));
        assert!(detail.contains("Suggestion: be more specific"));
    }

    #[test]
    fn detail_truncates_after_three_issues() {
        let issues: Vec<ValidationIssue> = (0..5)
            .map(|i| ValidationIssue {
                location: format!("Scenario: S{i}"),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: format!("issue {i}"),
                suggestion: None,
            })
            .collect();
        let v = make_validation(5, issues);
        let steps = build_steps(&v, &offline_config());
        let detail = steps[1].detail.as_ref().unwrap();
        assert!(detail.contains("issue 0"));
        assert!(detail.contains("issue 2"));
        assert!(!detail.contains("issue 3"));
    }
}
