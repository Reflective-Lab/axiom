// Copyright 2024-2026 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Semantic predicate parser for Gherkin steps.
//!
//! Parses Given/Then steps from `.truth` files into structured `Predicate`
//! values that can be compiled to Rust check logic or WASM invariant code.
//!
//! # Recognized Patterns
//!
//! | Step Pattern | Predicate |
//! |---|---|
//! | "contains at least N facts" | `CountAtLeast { min: N }` |
//! | "contains at most N facts" | `CountAtMost { max: N }` |
//! | "must not contain any forbidden term" | `ContentMustNotContain` |
//! | "must include" / "must contain a field" | `ContentMustContain` |
//! | "for each X there exists Y" | `CrossReference` |
//! | "any fact under key" / "facts under key" | `HasFacts` |
//! | (unrecognized) | `Custom` |
//!
//! # Architecture
//!
//! ```text
//! Gherkin Steps → parse_steps() → Vec<Predicate> → codegen (Task #3)
//! ```

use regex::Regex;

/// A semantic predicate extracted from Gherkin steps.
///
/// Represents the testable assertion that an invariant checks.
/// Each variant maps to a code pattern in the generated Rust check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Predicate {
    /// "the Context key X contains at least N facts"
    CountAtLeast { key: String, min: usize },

    /// "the Context key X contains at most N facts"
    CountAtMost { key: String, max: usize },

    /// "it must not contain any forbidden term" (with table of terms)
    ContentMustNotContain {
        key: String,
        forbidden: Vec<ForbiddenTerm>,
    },

    /// "it must include field X with a non-empty value"
    ContentMustContain { key: String, required_field: String },

    /// "for each X fact there exists a Y fact referencing it"
    CrossReference {
        source_key: String,
        target_key: String,
    },

    /// "any fact under key X" / "facts exist under key X"
    HasFacts { key: String },

    /// "must include" with a table of required fields
    RequiredFields {
        key: String,
        fields: Vec<FieldRequirement>,
    },

    /// Unrecognized step — preserved for downstream handling.
    Custom { description: String },
}

/// A forbidden term with reason (from Gherkin data tables).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenTerm {
    pub term: String,
    pub reason: String,
}

/// A required field with validation rule (from Gherkin data tables).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRequirement {
    pub field: String,
    pub rule: String,
}

/// Error during predicate parsing.
#[derive(Debug, Clone)]
pub enum PredicateError {
    /// A step references a context key that doesn't exist.
    UnknownContextKey(String),
    /// General parse error.
    ParseError(String),
}

impl std::fmt::Display for PredicateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownContextKey(key) => write!(f, "unknown context key: {key}"),
            Self::ParseError(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for PredicateError {}

/// Known context keys in Converge.
const KNOWN_KEYS: &[&str] = &[
    "Seeds",
    "Hypotheses",
    "Strategies",
    "Constraints",
    "Signals",
    "Competitors",
    "Evaluations",
];

/// Check if a context key name is valid.
fn is_valid_key(key: &str) -> bool {
    KNOWN_KEYS.contains(&key)
}

/// Parse Gherkin steps into semantic predicates.
///
/// Analyzes the text of Given/Then steps using pattern matching to
/// extract structured predicates. Steps that don't match known patterns
/// produce `Predicate::Custom` (no error).
///
/// Data tables attached to steps are parsed for forbidden terms and
/// required field specifications.
///
/// # Examples
///
/// ```
/// use converge_tool::predicate::{parse_steps, Predicate};
///
/// let steps = vec![
///     ("Then", r#"the Context key "Strategies" contains at least 2 facts"#, vec![]),
/// ];
/// let predicates = parse_steps(&steps).unwrap();
/// assert!(matches!(&predicates[0], Predicate::CountAtLeast { key, min: 2 } if key == "Strategies"));
/// ```
pub fn parse_steps(
    steps: &[(&str, &str, Vec<Vec<String>>)],
) -> Result<Vec<Predicate>, PredicateError> {
    let mut predicates = Vec::new();

    // Track the current "given" key context for Then steps
    let mut current_key: Option<String> = None;

    for (step_type, text, table) in steps {
        match *step_type {
            "Given" => {
                // "Given the engine halts" — not a context key reference
                if text.contains("engine halts") || text.contains("engine is") {
                    continue;
                }

                // Extract context key from Given steps
                if let Some(key) = extract_context_key(text) {
                    if is_valid_key(&key) {
                        current_key = Some(key.clone());
                        // "Given any fact under key X" → HasFacts
                        if text.contains("any fact") || text.contains("facts") {
                            predicates.push(Predicate::HasFacts { key });
                        }
                    }
                    // Quoted strings that aren't context keys in Given are ignored
                    // (e.g., "Converged" in 'engine halts with reason "Converged"')
                }
            }
            "Then" => {
                let pred = parse_then_step(text, table, &current_key)?;
                predicates.push(pred);
            }
            "And" => {
                // "And" continues the previous step type's context
                if text.contains("must include") || text.contains("must contain") {
                    let pred = parse_then_step(text, table, &current_key)?;
                    predicates.push(pred);
                }
            }
            _ => {} // When, But, etc. — not typically used in invariants
        }
    }

    Ok(predicates)
}

/// Parse a Then step into a predicate.
fn parse_then_step(
    text: &str,
    table: &[Vec<String>],
    current_key: &Option<String>,
) -> Result<Predicate, PredicateError> {
    // Pattern: "contains at least N facts"
    let count_at_least = Regex::new(r#"(?:contains?|at least)\s+(\d+)\s+facts?"#).unwrap();
    if let Some(caps) = count_at_least.captures(text) {
        let min: usize = caps[1].parse().unwrap_or(1);
        let key = extract_context_key(text)
            .or_else(|| current_key.clone())
            .unwrap_or_default();
        if !key.is_empty() {
            validate_key(&key)?;
        }
        return Ok(Predicate::CountAtLeast { key, min });
    }

    // Pattern: "contains at most N facts"
    let count_at_most = Regex::new(r#"at most\s+(\d+)\s+facts?"#).unwrap();
    if let Some(caps) = count_at_most.captures(text) {
        let max: usize = caps[1].parse().unwrap_or(1);
        let key = extract_context_key(text)
            .or_else(|| current_key.clone())
            .unwrap_or_default();
        if !key.is_empty() {
            validate_key(&key)?;
        }
        return Ok(Predicate::CountAtMost { key, max });
    }

    // Pattern: "must not contain any forbidden term" (with table)
    if text.contains("must not contain") {
        let key = current_key.clone().unwrap_or_default();
        let forbidden = parse_forbidden_terms(table);
        return Ok(Predicate::ContentMustNotContain { key, forbidden });
    }

    // Pattern: "for each X fact there exists a Y fact"
    let cross_ref =
        Regex::new(r#"for each\s+(\w+)\s+fact.*?exists?\s+(?:a |an )?(\w+)\s+fact"#).unwrap();
    if let Some(caps) = cross_ref.captures(text) {
        let source_key = caps[1].to_string();
        let target_key = caps[2].to_string();
        return Ok(Predicate::CrossReference {
            source_key,
            target_key,
        });
    }

    // Pattern: "must include" with table of fields
    if (text.contains("must include") || text.contains("must contain a field")) && !table.is_empty()
    {
        let key = current_key.clone().unwrap_or_default();
        let fields = parse_field_requirements(table);
        return Ok(Predicate::RequiredFields { key, fields });
    }

    // Pattern: "must contain a field X with a non-empty value"
    let field_pattern = Regex::new(r#"must contain (?:a )?field\s+"(\w+)""#).unwrap();
    if let Some(caps) = field_pattern.captures(text) {
        let key = current_key.clone().unwrap_or_default();
        return Ok(Predicate::ContentMustContain {
            key,
            required_field: caps[1].to_string(),
        });
    }

    // Fallback: Custom predicate
    Ok(Predicate::Custom {
        description: text.to_string(),
    })
}

/// Extract a context key from step text (quoted string like "Strategies").
fn extract_context_key(text: &str) -> Option<String> {
    let re = Regex::new(r#""(\w+)""#).unwrap();
    re.captures(text).map(|caps| caps[1].to_string())
}

/// Validate a context key against known keys.
fn validate_key(key: &str) -> Result<(), PredicateError> {
    if is_valid_key(key) {
        Ok(())
    } else {
        Err(PredicateError::UnknownContextKey(key.to_string()))
    }
}

/// Parse a Gherkin data table into forbidden terms.
fn parse_forbidden_terms(table: &[Vec<String>]) -> Vec<ForbiddenTerm> {
    table
        .iter()
        .filter(|row| row.len() >= 2)
        .map(|row| ForbiddenTerm {
            term: row[0].clone(),
            reason: row[1].clone(),
        })
        .collect()
}

/// Parse a Gherkin data table into field requirements.
fn parse_field_requirements(table: &[Vec<String>]) -> Vec<FieldRequirement> {
    table
        .iter()
        .filter(|row| row.len() >= 2)
        .map(|row| FieldRequirement {
            field: row[0].clone(),
            rule: row[1].clone(),
        })
        .collect()
}

/// Extract context key dependencies from a set of predicates.
///
/// Returns the set of context keys that the predicates reference,
/// which feeds into `WasmManifest.dependencies`.
pub fn extract_dependencies(predicates: &[Predicate]) -> Vec<String> {
    let mut deps = std::collections::BTreeSet::new();

    for pred in predicates {
        match pred {
            Predicate::CountAtLeast { key, .. }
            | Predicate::CountAtMost { key, .. }
            | Predicate::ContentMustNotContain { key, .. }
            | Predicate::ContentMustContain { key, .. }
            | Predicate::HasFacts { key }
            | Predicate::RequiredFields { key, .. } => {
                if !key.is_empty() {
                    deps.insert(key.clone());
                }
            }
            Predicate::CrossReference {
                source_key,
                target_key,
            } => {
                deps.insert(source_key.clone());
                deps.insert(target_key.clone());
            }
            Predicate::Custom { .. } => {}
        }
    }

    deps.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // CountAtLeast
    // =========================================================================

    #[test]
    fn parse_count_at_least() {
        let steps = vec![(
            "Then",
            r#"the Context key "Strategies" contains at least 2 facts"#,
            vec![],
        )];
        let preds = parse_steps(&steps).unwrap();
        assert_eq!(preds.len(), 1);
        assert!(matches!(
            &preds[0],
            Predicate::CountAtLeast { key, min: 2 } if key == "Strategies"
        ));
    }

    #[test]
    fn parse_count_at_least_with_given_context() {
        let steps = vec![
            (
                "Given",
                r#"the engine halts with reason "Converged""#,
                vec![],
            ),
            (
                "Then",
                r#"the Context key "Strategies" contains at least 2 facts"#,
                vec![],
            ),
        ];
        let preds = parse_steps(&steps).unwrap();
        assert!(matches!(
            &preds[0],
            Predicate::CountAtLeast { key, min: 2 } if key == "Strategies"
        ));
    }

    // =========================================================================
    // ContentMustNotContain
    // =========================================================================

    #[test]
    fn parse_forbidden_terms_with_table() {
        let steps = vec![
            ("Given", r#"any fact under key "Strategies""#, vec![]),
            (
                "Then",
                "it must not contain any forbidden term:",
                vec![
                    vec!["spam".to_string(), "illegal marketing".to_string()],
                    vec!["bot army".to_string(), "fake engagement".to_string()],
                ],
            ),
        ];
        let preds = parse_steps(&steps).unwrap();
        // HasFacts from Given + ContentMustNotContain from Then
        assert_eq!(preds.len(), 2);
        assert!(matches!(&preds[0], Predicate::HasFacts { key } if key == "Strategies"));
        match &preds[1] {
            Predicate::ContentMustNotContain { key, forbidden } => {
                assert_eq!(key, "Strategies");
                assert_eq!(forbidden.len(), 2);
                assert_eq!(forbidden[0].term, "spam");
                assert_eq!(forbidden[1].reason, "fake engagement");
            }
            _ => panic!("expected ContentMustNotContain"),
        }
    }

    // =========================================================================
    // CrossReference
    // =========================================================================

    #[test]
    fn parse_cross_reference() {
        let steps = vec![(
            "Then",
            "for each Strategy fact there exists an Evaluation fact referencing it",
            vec![],
        )];
        let preds = parse_steps(&steps).unwrap();
        assert_eq!(preds.len(), 1);
        assert!(matches!(
            &preds[0],
            Predicate::CrossReference { source_key, target_key }
            if source_key == "Strategy" && target_key == "Evaluation"
        ));
    }

    // =========================================================================
    // RequiredFields
    // =========================================================================

    #[test]
    fn parse_required_fields_with_table() {
        let steps = vec![
            ("Given", r#"any fact under key "Evaluations""#, vec![]),
            (
                "Then",
                "it must include:",
                vec![
                    vec!["score".to_string(), "integer between 0..100".to_string()],
                    vec!["rationale".to_string(), "non-empty string".to_string()],
                ],
            ),
        ];
        let preds = parse_steps(&steps).unwrap();
        assert_eq!(preds.len(), 2); // HasFacts + RequiredFields
        match &preds[1] {
            Predicate::RequiredFields { key, fields } => {
                assert_eq!(key, "Evaluations");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].field, "score");
                assert_eq!(fields[1].field, "rationale");
            }
            _ => panic!("expected RequiredFields"),
        }
    }

    // =========================================================================
    // ContentMustContain (single field)
    // =========================================================================

    #[test]
    fn parse_content_must_contain_field() {
        let steps = vec![
            ("Given", r#"any fact under key "Strategies""#, vec![]),
            (
                "Then",
                r#"it must contain a field "compliance_ref" with a non-empty value"#,
                vec![],
            ),
        ];
        let preds = parse_steps(&steps).unwrap();
        assert!(matches!(
            &preds[1],
            Predicate::ContentMustContain { key, required_field }
            if key == "Strategies" && required_field == "compliance_ref"
        ));
    }

    // =========================================================================
    // HasFacts
    // =========================================================================

    #[test]
    fn parse_has_facts() {
        let steps = vec![(
            "Given",
            r#"the Context contains facts under key "Signals""#,
            vec![],
        )];
        let preds = parse_steps(&steps).unwrap();
        assert_eq!(preds.len(), 1);
        assert!(matches!(&preds[0], Predicate::HasFacts { key } if key == "Signals"));
    }

    // =========================================================================
    // Custom fallback
    // =========================================================================

    #[test]
    fn unrecognized_step_becomes_custom() {
        let steps = vec![("Then", "something completely different happens", vec![])];
        let preds = parse_steps(&steps).unwrap();
        assert_eq!(preds.len(), 1);
        assert!(
            matches!(&preds[0], Predicate::Custom { description } if description.contains("completely different"))
        );
    }

    // =========================================================================
    // Negative tests
    // =========================================================================

    #[test]
    fn unknown_context_key_in_then_step_error() {
        let steps = vec![(
            "Then",
            r#"the Context key "Widgets" contains at least 2 facts"#,
            vec![],
        )];
        let result = parse_steps(&steps);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PredicateError::UnknownContextKey(k) if k == "Widgets"
        ));
    }

    #[test]
    fn unknown_key_in_given_is_ignored() {
        // Given steps may have non-key quoted strings (e.g., "Converged")
        let steps = vec![("Given", r#"any fact under key "Widgets""#, vec![])];
        let result = parse_steps(&steps);
        // "Widgets" is not a known key, so it's silently ignored in Given
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn empty_steps_produces_no_predicates() {
        let steps: Vec<(&str, &str, Vec<Vec<String>>)> = vec![];
        let preds = parse_steps(&steps).unwrap();
        assert!(preds.is_empty());
    }

    // =========================================================================
    // Dependency extraction
    // =========================================================================

    #[test]
    fn extract_deps_from_predicates() {
        let preds = vec![
            Predicate::CountAtLeast {
                key: "Strategies".to_string(),
                min: 2,
            },
            Predicate::CrossReference {
                source_key: "Strategies".to_string(),
                target_key: "Evaluations".to_string(),
            },
            Predicate::HasFacts {
                key: "Seeds".to_string(),
            },
        ];
        let deps = extract_dependencies(&preds);
        assert_eq!(deps, vec!["Evaluations", "Seeds", "Strategies"]);
    }

    #[test]
    fn extract_deps_deduplicates() {
        let preds = vec![
            Predicate::HasFacts {
                key: "Strategies".to_string(),
            },
            Predicate::CountAtLeast {
                key: "Strategies".to_string(),
                min: 1,
            },
        ];
        let deps = extract_dependencies(&preds);
        assert_eq!(deps, vec!["Strategies"]);
    }

    #[test]
    fn custom_predicates_have_no_deps() {
        let preds = vec![Predicate::Custom {
            description: "something".to_string(),
        }];
        let deps = extract_dependencies(&preds);
        assert!(deps.is_empty());
    }

    // =========================================================================
    // Property tests
    // =========================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn any_step_produces_predicate_or_error(text in "\\PC{1,100}") {
                let steps = vec![("Then", text.as_str(), vec![])];
                // Should never panic — either Ok or Err
                let _ = parse_steps(&steps);
            }

            #[test]
            fn count_pattern_always_parses(n in 1usize..1000, key in prop::sample::select(KNOWN_KEYS)) {
                let text = format!(r#"the Context key "{key}" contains at least {n} facts"#);
                let steps = vec![("Then", text.as_str(), vec![])];
                let preds = parse_steps(&steps).unwrap();
                assert!(matches!(&preds[0], Predicate::CountAtLeast { min, .. } if *min == n));
            }

            #[test]
            fn dependency_extraction_never_crashes(
                keys in proptest::collection::vec("[A-Z][a-z]{3,10}", 0..5)
            ) {
                let preds: Vec<Predicate> = keys.iter().map(|k| Predicate::HasFacts { key: k.clone() }).collect();
                let _ = extract_dependencies(&preds);
            }
        }
    }
}
