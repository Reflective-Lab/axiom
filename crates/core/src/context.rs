// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Context model for Converge.
//!
//! Context is the shared, typed, evolving representation of a job.
//! Types are defined in `converge-traits`; this module provides the
//! concrete `Context` struct that the engine uses.

use crate::error::ConvergeError;
use std::collections::HashMap;

// Re-export canonical types from converge-traits
pub use converge_traits::context::ContextKey;
pub use converge_traits::fact::{Fact, ProposedFact, ValidationError};

/// The shared context for a Converge job.
///
/// Agents receive `&dyn converge_traits::Context` (immutable) during execution.
/// Only the engine holds `&mut Context` during the merge phase.
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Context {
    /// Facts stored by their key category.
    facts: HashMap<ContextKey, Vec<Fact>>,
    /// Tracks which keys changed in the last merge cycle.
    dirty_keys: Vec<ContextKey>,
    /// Monotonic version counter for convergence detection.
    version: u64,
}

/// Implement the converge-traits Context trait for the concrete Context struct.
/// This allows agents to use `&dyn converge_traits::Context`.
impl converge_traits::Context for Context {
    fn has(&self, key: ContextKey) -> bool {
        self.facts.get(&key).is_some_and(|v| !v.is_empty())
    }

    fn get(&self, key: ContextKey) -> &[Fact] {
        self.facts.get(&key).map_or(&[], Vec::as_slice)
    }
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
    pub fn add_fact(&mut self, fact: Fact) -> Result<bool, ConvergeError> {
        let key = fact.key;
        let facts = self.facts.entry(key).or_default();

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

// TryFrom<ProposedFact> for Fact is defined in converge-traits

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
        let fact = Fact::new(ContextKey::Seeds, "seed-1", "initial value");

        let changed = ctx.add_fact(fact).expect("should add");
        assert!(changed);
        assert_eq!(ctx.version(), 1);
        assert!(ctx.has(ContextKey::Seeds));
    }

    #[test]
    fn duplicate_fact_does_not_change_context() {
        let mut ctx = Context::new();
        let fact = Fact::new(ContextKey::Seeds, "seed-1", "initial");

        ctx.add_fact(fact.clone()).expect("should add first");
        let changed = ctx.add_fact(fact).expect("should not error on duplicate");
        assert!(!changed);
        assert_eq!(ctx.version(), 1);
    }

    #[test]
    fn dirty_keys_track_new_facts_and_clear() {
        let mut ctx = Context::new();
        let fact = Fact::new(ContextKey::Hypotheses, "hyp-1", "value");

        ctx.add_fact(fact).expect("should add");
        assert_eq!(ctx.dirty_keys(), &[ContextKey::Hypotheses]);

        ctx.clear_dirty();
        assert!(ctx.dirty_keys().is_empty());
    }

    #[test]
    fn detects_conflict() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact::new(ContextKey::Seeds, "fact-1", "version A"))
            .unwrap();

        let result = ctx.add_fact(Fact::new(ContextKey::Seeds, "fact-1", "version B"));

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
    }

    #[test]
    fn proposed_fact_rejects_invalid_confidence() {
        let proposed = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "hyp-1".into(),
            content: "some content".into(),
            confidence: 1.5,
            provenance: "test".into(),
        };

        let result: Result<Fact, ValidationError> = proposed.try_into();
        assert!(result.is_err());
    }

    /// Test that Context implements the converge_traits::Context trait.
    #[test]
    fn context_implements_trait() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact::new(ContextKey::Seeds, "s1", "hello"))
            .unwrap();

        // Use via trait object
        let dyn_ctx: &dyn converge_traits::Context = &ctx;
        assert!(dyn_ctx.has(ContextKey::Seeds));
        assert_eq!(dyn_ctx.get(ContextKey::Seeds).len(), 1);
    }
}
