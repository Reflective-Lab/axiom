// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Context model for Converge.
//!
//! Context is the shared, typed, evolving representation of a job.
//! It is append-only in meaning and monotonically evolving.

use crate::error::ConvergeError;
use std::collections::HashMap;
use strum::EnumIter;

use serde::{Deserialize, Serialize};

/// A key identifying a category of facts in context.
///
/// Agents declare dependencies on `ContextKey`s to enable
/// data-driven eligibility (only re-run when relevant data changes).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumIter, Serialize, Deserialize,
)]
pub enum ContextKey {
    /// Initial seed facts from the `RootIntent`.
    Seeds,
    /// Hypotheses under consideration.
    Hypotheses,
    /// Evaluated strategies or solutions.
    Strategies,
    /// Constraints that must be satisfied.
    Constraints,
    /// Signals from external sources.
    Signals,
    /// Competitor profiles and analysis.
    Competitors,
    /// Evaluations and scores for strategies.
    Evaluations,
    /// Internal storage for proposed facts before validation.
    Proposals,
    /// Diagnostics and errors emitted by the engine.
    Diagnostic,
}

/// A typed assertion added to context.
///
/// Facts are immutable once created. They carry provenance
/// for auditability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// The category this fact belongs to.
    pub key: ContextKey,
    /// Unique identifier within the context.
    pub id: String,
    /// The fact's content (simplified for MVP).
    pub content: String,
}

impl Fact {
    /// Creates a new fact.
    ///
    /// # Example
    ///
    /// ```
    /// use converge_core::{Fact, ContextKey};
    ///
    /// let fact = Fact::new(ContextKey::Seeds, "seed-1", "initial value");
    /// assert_eq!(fact.key, ContextKey::Seeds);
    /// assert_eq!(fact.id, "seed-1");
    /// ```
    #[must_use]
    pub fn new(key: ContextKey, id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            key,
            id: id.into(),
            content: content.into(),
        }
    }
}

/// A suggested fact from a non-authoritative source (e.g., LLM).
///
/// `ProposedFact` is compile-time separated from `Fact` to enforce
/// that LLM outputs cannot accidentally become trusted facts.
/// Promotion requires explicit validation via `TryFrom`.
///
/// # Decision Reference
/// See DECISIONS.md §3: "If something is dangerous, make it impossible to misuse."
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedFact {
    /// The category this proposed fact would belong to.
    pub key: ContextKey,
    /// Suggested identifier.
    pub id: String,
    /// The proposed content.
    pub content: String,
    /// Confidence hint from the source (0.0 - 1.0).
    pub confidence: f64,
    /// Provenance information (e.g., model ID, prompt hash).
    pub provenance: String,
}

/// Error when a `ProposedFact` fails validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Reason the proposal was rejected.
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "validation failed: {}", self.reason)
    }
}

impl std::error::Error for ValidationError {}

impl TryFrom<ProposedFact> for Fact {
    type Error = ValidationError;

    /// Converts a `ProposedFact` to a `Fact` after validation.
    ///
    /// This is the ONLY way to promote a proposal to a fact.
    /// In production, this would include schema validation,
    /// constraint checks, and governance rules.
    fn try_from(proposed: ProposedFact) -> Result<Self, Self::Error> {
        // Confidence must be a finite number in [0.0, 1.0].
        // NaN and infinity are rejected (NaN bypasses range checks).
        if !proposed.confidence.is_finite()
            || proposed.confidence < 0.0
            || proposed.confidence > 1.0
        {
            return Err(ValidationError {
                reason: "confidence must be a finite number between 0.0 and 1.0".into(),
            });
        }

        // MVP: Require non-empty content
        if proposed.content.trim().is_empty() {
            return Err(ValidationError {
                reason: "content cannot be empty".into(),
            });
        }

        Ok(Fact {
            key: proposed.key,
            id: proposed.id,
            content: proposed.content,
        })
    }
}

/// The shared context for a Converge job.
///
/// Agents receive `&Context` (immutable) during execution.
/// Only the engine holds `&mut Context` during the merge phase.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Facts stored by their key category.
    facts: HashMap<ContextKey, Vec<Fact>>,
    /// Tracks which keys changed in the last merge cycle.
    dirty_keys: Vec<ContextKey>,
    /// Monotonic version counter for convergence detection.
    version: u64,
}

impl Context {
    /// Creates a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all facts for a given key.
    #[must_use]
    pub fn get(&self, key: ContextKey) -> &[Fact] {
        self.facts.get(&key).map_or(&[], Vec::as_slice)
    }

    /// Returns true if there are any facts for the given key.
    #[must_use]
    pub fn has(&self, key: ContextKey) -> bool {
        self.facts.get(&key).is_some_and(|v| !v.is_empty())
    }

    /// Returns the current version (for convergence detection).
    #[must_use]
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Returns keys that changed in the last merge cycle.
    #[must_use]
    pub fn dirty_keys(&self) -> &[ContextKey] {
        &self.dirty_keys
    }

    /// Returns all keys that currently have facts in the context.
    #[must_use]
    pub fn all_keys(&self) -> Vec<ContextKey> {
        self.facts.keys().copied().collect()
    }

    /// Clears the dirty key tracker (called at start of each cycle).
    pub fn clear_dirty(&mut self) {
        self.dirty_keys.clear();
    }

    /// Adds a fact to the context (engine-only, during merge phase).
    ///
    /// Returns `Ok(true)` if the fact was new (context changed).
    /// Returns `Ok(false)` if the fact was already present and identical.
    ///
    /// # Errors
    ///
    /// Returns `Err(ConvergeError::Conflict)` if a fact with the same ID but
    /// different content already exists. This indicates non-deterministic behavior
    /// from agents producing conflicting outputs.
    pub fn add_fact(&mut self, fact: Fact) -> Result<bool, ConvergeError> {
        let key = fact.key;
        let facts = self.facts.entry(key).or_default();

        // Check for duplicate or conflict (same id)
        if let Some(existing) = facts.iter().find(|f| f.id == fact.id) {
            if existing.content == fact.content {
                return Ok(false);
            }
            return Err(ConvergeError::Conflict {
                id: fact.id,
                existing: existing.content.clone(),
                new: fact.content,
                context: Box::new(self.clone()),
            });
        }

        facts.push(fact);
        self.dirty_keys.push(key);

        self.version += 1;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_context_has_no_facts() {
        let ctx = Context::new();
        assert!(!ctx.has(ContextKey::Seeds));
        assert_eq!(ctx.version(), 0);
    }

    #[test]
    fn adding_fact_increments_version() {
        let mut ctx = Context::new();
        let fact = Fact {
            key: ContextKey::Seeds,
            id: "seed-1".into(),
            content: "initial".into(),
        };

        let changed = ctx.add_fact(fact).expect("should add");
        assert!(changed);
        assert_eq!(ctx.version(), 1);
        assert!(ctx.has(ContextKey::Seeds));
    }

    #[test]
    fn duplicate_fact_does_not_change_context() {
        let mut ctx = Context::new();
        let fact = Fact {
            key: ContextKey::Seeds,
            id: "seed-1".into(),
            content: "initial".into(),
        };

        ctx.add_fact(fact.clone()).expect("should add first");
        let changed = ctx.add_fact(fact).expect("should not error on duplicate");
        assert!(!changed);
        assert_eq!(ctx.version(), 1);
    }

    #[test]
    fn dirty_keys_track_new_facts_and_clear() {
        let mut ctx = Context::new();
        let fact = Fact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "value".into(),
        };

        ctx.add_fact(fact).expect("should add");
        assert_eq!(ctx.dirty_keys(), &[ContextKey::Hypotheses]);

        ctx.clear_dirty();
        assert!(ctx.dirty_keys().is_empty());
    }

    #[test]
    fn duplicate_fact_does_not_dirty_again() {
        let mut ctx = Context::new();
        let fact = Fact {
            key: ContextKey::Signals,
            id: "signal-1".into(),
            content: "ping".into(),
        };

        assert!(ctx.add_fact(fact.clone()).expect("should add"));
        ctx.clear_dirty();

        assert!(!ctx.add_fact(fact).expect("should not error"));
        assert!(ctx.dirty_keys().is_empty());
    }

    #[test]
    fn get_returns_partitioned_facts() {
        let mut ctx = Context::new();
        let seed = Fact {
            key: ContextKey::Seeds,
            id: "seed-1".into(),
            content: "seed".into(),
        };
        let strategy = Fact {
            key: ContextKey::Strategies,
            id: "strat-1".into(),
            content: "strategy".into(),
        };

        ctx.add_fact(seed).expect("should add");
        ctx.add_fact(strategy).expect("should add");

        assert_eq!(ctx.get(ContextKey::Seeds).len(), 1);
        assert_eq!(ctx.get(ContextKey::Strategies).len(), 1);
        assert!(ctx.get(ContextKey::Hypotheses).is_empty());
    }

    #[test]
    fn detects_conflict() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Seeds,
            id: "fact-1".into(),
            content: "version A".into(),
        })
        .unwrap();

        let result = ctx.add_fact(Fact {
            key: ContextKey::Seeds,
            id: "fact-1".into(),
            content: "version B".into(),
        });

        match result {
            Err(ConvergeError::Conflict {
                id, existing, new, ..
            }) => {
                assert_eq!(id, "fact-1");
                assert_eq!(existing, "version A");
                assert_eq!(new, "version B");
            }
            _ => panic!("Expected Conflict error, got {result:?}"),
        }
    }

    #[test]
    fn proposed_fact_converts_to_fact_when_valid() {
        let proposed = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "market is growing".into(),
            confidence: 0.8,
            provenance: "gpt-4:abc123".into(),
        };

        let fact: Fact = proposed.try_into().expect("should convert");
        assert_eq!(fact.key, ContextKey::Hypotheses);
        assert_eq!(fact.id, "hyp-1");
        assert_eq!(fact.content, "market is growing");
    }

    #[test]
    fn proposed_fact_rejects_invalid_confidence() {
        let proposed = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "some content".into(),
            confidence: 1.5, // Invalid: > 1.0
            provenance: "test".into(),
        };

        let result: Result<Fact, ValidationError> = proposed.try_into();
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("confidence"));
    }

    #[test]
    fn proposed_fact_rejects_empty_content() {
        let proposed = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "   ".into(), // Empty after trim
            confidence: 0.5,
            provenance: "test".into(),
        };

        let result: Result<Fact, ValidationError> = proposed.try_into();
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("empty"));
    }

    #[test]
    fn proposed_fact_cannot_be_used_as_fact_directly() {
        // This test documents the compile-time separation.
        // The following would NOT compile:
        //
        // let proposed = ProposedFact { ... };
        // ctx.add_fact(proposed); // ERROR: expected Fact, found ProposedFact
        //
        // You MUST go through TryFrom to convert.
    }

    // ========================================================================
    // PROPERTY-BASED TESTS: ProposedFact validation boundary (REF-8)
    // ========================================================================

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        /// Strategy for generating arbitrary ContextKeys.
        fn arb_context_key() -> impl Strategy<Value = ContextKey> {
            prop_oneof![
                Just(ContextKey::Seeds),
                Just(ContextKey::Hypotheses),
                Just(ContextKey::Strategies),
                Just(ContextKey::Constraints),
                Just(ContextKey::Signals),
                Just(ContextKey::Competitors),
                Just(ContextKey::Evaluations),
                Just(ContextKey::Proposals),
                Just(ContextKey::Diagnostic),
            ]
        }

        proptest! {
            /// Any ProposedFact with out-of-range confidence is always rejected.
            #[test]
            fn invalid_confidence_always_rejected(
                key in arb_context_key(),
                id in "[a-z]{1,20}",
                content in ".{1,100}",
                // Generate confidence outside [0.0, 1.0]
                confidence in prop_oneof![
                    (-1000.0f64..=-0.001),
                    (1.001..=1000.0f64),
                    Just(f64::NAN),
                    Just(f64::INFINITY),
                    Just(f64::NEG_INFINITY),
                ],
                provenance in ".{1,50}",
            ) {
                let proposed = ProposedFact {
                    key, id, content, confidence, provenance,
                };
                let result: Result<Fact, ValidationError> = proposed.try_into();
                prop_assert!(result.is_err(), "confidence {} should be rejected", confidence);
            }

            /// Any ProposedFact with empty/whitespace-only content is always rejected.
            #[test]
            fn empty_content_always_rejected(
                key in arb_context_key(),
                id in "[a-z]{1,20}",
                // Generate only whitespace content
                content in "[ \t\n\r]{0,20}",
                confidence in 0.0..=1.0f64,
                provenance in ".{1,50}",
            ) {
                let proposed = ProposedFact {
                    key, id, content, confidence, provenance,
                };
                let result: Result<Fact, ValidationError> = proposed.try_into();
                prop_assert!(result.is_err(), "whitespace-only content should be rejected");
            }

            /// Valid ProposedFacts always promote successfully and preserve content.
            #[test]
            fn valid_proposal_always_promotes(
                key in arb_context_key(),
                id in "[a-z]{1,20}",
                content in "[a-zA-Z0-9][a-zA-Z0-9 ]{0,99}",
                confidence in 0.0..=1.0f64,
                provenance in ".{1,50}",
            ) {
                let proposed = ProposedFact {
                    key,
                    id: id.clone(),
                    content: content.clone(),
                    confidence,
                    provenance,
                };
                let result: Result<Fact, ValidationError> = proposed.try_into();
                prop_assert!(result.is_ok(), "valid proposal should promote");
                let fact = result.unwrap();
                prop_assert_eq!(fact.key, key);
                prop_assert_eq!(fact.id, id);
                prop_assert_eq!(fact.content, content);
            }

            /// Determinism: same ProposedFact always produces same validation result.
            #[test]
            fn validation_is_deterministic(
                key in arb_context_key(),
                id in "[a-z]{1,20}",
                content in ".{0,100}",
                confidence in -2.0..=2.0f64,
                provenance in ".{1,50}",
            ) {
                let p1 = ProposedFact {
                    key, id: id.clone(), content: content.clone(),
                    confidence, provenance: provenance.clone(),
                };
                let p2 = ProposedFact {
                    key, id, content, confidence, provenance,
                };
                let r1: Result<Fact, ValidationError> = p1.try_into();
                let r2: Result<Fact, ValidationError> = p2.try_into();
                prop_assert_eq!(r1.is_ok(), r2.is_ok(), "same input must produce same result");
            }
        }
    }
}
