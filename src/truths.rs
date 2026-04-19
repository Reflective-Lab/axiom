// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Converge Truths document parsing.
//!
//! Converge Truths v1 stays Gherkin-first while adding a narrow declaration
//! layer for governance metadata. This module extracts those declarations and
//! returns a Gherkin-compatible body for the existing parser pipeline.

use crate::gherkin::{ValidationError, preprocess_truths};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TruthDocument {
    pub gherkin: String,
    pub governance: TruthGovernance,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TruthGovernance {
    pub intent: Option<IntentBlock>,
    pub authority: Option<AuthorityBlock>,
    pub constraint: Option<ConstraintBlock>,
    pub evidence: Option<EvidenceBlock>,
    pub exception: Option<ExceptionBlock>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntentBlock {
    pub outcome: Option<String>,
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorityBlock {
    pub actor: Option<String>,
    pub may: Vec<String>,
    pub must_not: Vec<String>,
    pub requires_approval: Vec<String>,
    pub expires: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConstraintBlock {
    pub budget: Vec<String>,
    pub cost_limit: Vec<String>,
    pub must_not: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvidenceBlock {
    pub requires: Vec<String>,
    pub provenance: Vec<String>,
    pub audit: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExceptionBlock {
    pub escalates_to: Vec<String>,
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Intent,
    Authority,
    Constraint,
    Evidence,
    Exception,
}

impl BlockKind {
    fn from_header(line: &str) -> Option<Self> {
        match line.trim() {
            "Intent:" => Some(Self::Intent),
            "Authority:" => Some(Self::Authority),
            "Constraint:" => Some(Self::Constraint),
            "Evidence:" => Some(Self::Evidence),
            "Exception:" => Some(Self::Exception),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Intent => "Intent",
            Self::Authority => "Authority",
            Self::Constraint => "Constraint",
            Self::Evidence => "Evidence",
            Self::Exception => "Exception",
        }
    }
}

pub fn parse_truth_document(content: &str) -> Result<TruthDocument, ValidationError> {
    let preprocessed = preprocess_truths(content);
    let lines: Vec<&str> = preprocessed.lines().collect();
    let mut governance = TruthGovernance::default();
    let mut gherkin_lines = Vec::new();
    let mut idx = 0;
    let mut governance_window_open = true;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if governance_window_open && is_governance_boundary_start(trimmed) {
            governance_window_open = false;
            gherkin_lines.push(line);
            idx += 1;
            continue;
        }

        if governance_window_open {
            if let Some(kind) = BlockKind::from_header(trimmed) {
                ensure_block_not_duplicate(&governance, kind)?;
                idx = parse_block(&lines, idx + 1, kind, &mut governance)?;
                continue;
            }

            if is_unknown_block_header(trimmed) {
                return Err(ValidationError::ParseError(format!(
                    "unknown Converge Truths block header: {trimmed}"
                )));
            }
        }

        gherkin_lines.push(line);
        idx += 1;
    }

    let mut gherkin = gherkin_lines.join("\n");
    if content.ends_with('\n') {
        gherkin.push('\n');
    }

    Ok(TruthDocument {
        gherkin,
        governance,
    })
}

fn ensure_block_not_duplicate(
    governance: &TruthGovernance,
    kind: BlockKind,
) -> Result<(), ValidationError> {
    let duplicate = match kind {
        BlockKind::Intent => governance.intent.is_some(),
        BlockKind::Authority => governance.authority.is_some(),
        BlockKind::Constraint => governance.constraint.is_some(),
        BlockKind::Evidence => governance.evidence.is_some(),
        BlockKind::Exception => governance.exception.is_some(),
    };

    if duplicate {
        Err(ValidationError::ParseError(format!(
            "duplicate Converge Truths block: {}",
            kind.as_str()
        )))
    } else {
        Ok(())
    }
}

fn parse_block(
    lines: &[&str],
    mut idx: usize,
    kind: BlockKind,
    governance: &mut TruthGovernance,
) -> Result<usize, ValidationError> {
    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            idx += 1;
            continue;
        }

        if BlockKind::from_header(trimmed).is_some() || is_governance_boundary_start(trimmed) {
            break;
        }

        let (field, value) = parse_field_line(trimmed)?;
        apply_field(governance, kind, field, value)?;
        idx += 1;
    }

    Ok(idx)
}

fn parse_field_line(trimmed: &str) -> Result<(&str, &str), ValidationError> {
    let (field, value) = trimmed.split_once(':').ok_or_else(|| {
        ValidationError::ParseError(format!("invalid declaration line: {trimmed}"))
    })?;

    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::ParseError(format!(
            "declaration field must have a value: {trimmed}"
        )));
    }

    Ok((field.trim(), value))
}

fn apply_field(
    governance: &mut TruthGovernance,
    kind: BlockKind,
    field: &str,
    value: &str,
) -> Result<(), ValidationError> {
    match kind {
        BlockKind::Intent => {
            let block = governance.intent.get_or_insert_with(IntentBlock::default);
            match field {
                "Outcome" => block.outcome = Some(value.to_string()),
                "Goal" => block.goal = Some(value.to_string()),
                _ => return Err(unknown_field(kind, field)),
            }
        }
        BlockKind::Authority => {
            let block = governance
                .authority
                .get_or_insert_with(AuthorityBlock::default);
            match field {
                "Actor" => block.actor = Some(value.to_string()),
                "May" => block.may.push(value.to_string()),
                "Must Not" => block.must_not.push(value.to_string()),
                "Requires Approval" => block.requires_approval.push(value.to_string()),
                "Expires" => block.expires = Some(value.to_string()),
                _ => return Err(unknown_field(kind, field)),
            }
        }
        BlockKind::Constraint => {
            let block = governance
                .constraint
                .get_or_insert_with(ConstraintBlock::default);
            match field {
                "Budget" => block.budget.push(value.to_string()),
                "Cost Limit" => block.cost_limit.push(value.to_string()),
                "Must Not" => block.must_not.push(value.to_string()),
                _ => return Err(unknown_field(kind, field)),
            }
        }
        BlockKind::Evidence => {
            let block = governance
                .evidence
                .get_or_insert_with(EvidenceBlock::default);
            match field {
                "Requires" => block.requires.push(value.to_string()),
                "Provenance" => block.provenance.push(value.to_string()),
                "Audit" => block.audit.push(value.to_string()),
                _ => return Err(unknown_field(kind, field)),
            }
        }
        BlockKind::Exception => {
            let block = governance
                .exception
                .get_or_insert_with(ExceptionBlock::default);
            match field {
                "Escalates To" => block.escalates_to.push(value.to_string()),
                "Requires" => block.requires.push(value.to_string()),
                _ => return Err(unknown_field(kind, field)),
            }
        }
    }

    Ok(())
}

fn unknown_field(kind: BlockKind, field: &str) -> ValidationError {
    ValidationError::ParseError(format!(
        "unknown field `{field}` in {} block",
        kind.as_str()
    ))
}

fn is_gherkin_section_start(trimmed: &str) -> bool {
    trimmed.starts_with('@')
        || matches!(
            trimmed,
            t if t.starts_with("Feature:")
                || t.starts_with("Truth:")
                || t.starts_with("Background:")
                || t.starts_with("Scenario:")
                || t.starts_with("Rule:")
                || t.starts_with("Example:")
                || t.starts_with("Examples:")
        )
}

fn is_governance_boundary_start(trimmed: &str) -> bool {
    trimmed.starts_with('@')
        || matches!(
            trimmed,
            t if t.starts_with("Background:")
                || t.starts_with("Scenario:")
                || t.starts_with("Rule:")
                || t.starts_with("Example:")
                || t.starts_with("Examples:")
        )
}

fn is_unknown_block_header(trimmed: &str) -> bool {
    trimmed.ends_with(':')
        && !is_gherkin_section_start(trimmed)
        && !matches!(
            trimmed,
            "Intent:" | "Authority:" | "Constraint:" | "Evidence:" | "Exception:"
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_governance_blocks_and_strips_them_from_gherkin() {
        let content = r#"Truth: Release readiness

  Intent:
    Outcome: "Ship safely"
    Goal: "No critical regressions"

  Authority:
    Actor: release_manager
    May: approve_deployment
    Requires Approval: security_owner

  Evidence:
    Requires: test_report

  @invariant @acceptance
  Scenario: Release requires evidence
    Given a release candidate exists
    When the system converges
    Then deployment MUST NOT occur without test_report
"#;

        let doc = parse_truth_document(content).unwrap();
        assert!(doc.gherkin.contains("Feature: Release readiness"));
        assert!(doc.gherkin.contains("Scenario: Release requires evidence"));
        assert!(!doc.gherkin.contains("Intent:"));
        assert_eq!(
            doc.governance.intent.unwrap().outcome.as_deref(),
            Some("\"Ship safely\"")
        );
        assert_eq!(
            doc.governance.authority.unwrap().requires_approval,
            vec!["security_owner".to_string()]
        );
    }

    #[test]
    fn rejects_unknown_block_headers() {
        let content = r"Truth: Test

  Trigger:
    When: event_happens

  Scenario: Works
    Given precondition
    Then result
";

        let err = parse_truth_document(content).unwrap_err();
        assert!(
            err.to_string()
                .contains("unknown Converge Truths block header")
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let content = r"Truth: Test

  Authority:
    Owner: system

  Scenario: Works
    Given precondition
    Then result
";

        let err = parse_truth_document(content).unwrap_err();
        assert!(err.to_string().contains("unknown field `Owner`"));
    }

    #[test]
    fn rejects_duplicate_governance_blocks() {
        let content = r#"Truth: Test

  Intent:
    Outcome: "First"

  Intent:
    Outcome: "Second"

  Scenario: Works
    Given precondition
    When action happens
    Then result is produced
"#;

        let err = parse_truth_document(content).unwrap_err();
        assert!(
            err.to_string()
                .contains("duplicate Converge Truths block: Intent")
        );
    }

    #[test]
    fn rejects_malformed_declaration_lines() {
        let content = r"Truth: Test

  Evidence:
    Requires

  Scenario: Works
    Given precondition
    When action happens
    Then result is produced
";

        let err = parse_truth_document(content).unwrap_err();
        assert!(err.to_string().contains("invalid declaration line"));
    }

    proptest! {
        #[test]
        fn parse_truth_document_never_panics(s in "\\PC*") {
            let _ = parse_truth_document(&s);
        }

        #[test]
        fn plain_gherkin_round_trips_without_governance(
            name in "[A-Za-z][A-Za-z0-9 _-]{0,30}"
        ) {
            let content = format!(
                "Truth: {name}\n\n  Scenario: Example\n    Given a precondition exists\n    When an action occurs\n    Then a result is observed\n"
            );

            let doc = parse_truth_document(&content).unwrap();
            prop_assert!(doc.governance == TruthGovernance::default());
            prop_assert!(doc.gherkin.contains("Feature:"));
            prop_assert!(doc.gherkin.contains("Scenario: Example"));
        }
    }
}
