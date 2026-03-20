// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Agent effects — what agents produce, the engine merges.
//!
//! Effects allow mixed contributions: an agent can emit both validated
//! facts and proposals in a single execution. The engine collects effects
//! from all eligible agents, then merges them in deterministic order.

use crate::context::ContextKey;
use crate::fact::{Fact, ProposedFact};

/// The output of an agent's `execute()` call.
///
/// An effect describes what an agent wants to contribute to the context.
/// The engine collects effects from all eligible agents, then merges them
/// serially in deterministic order.
#[derive(Debug, Default)]
pub struct AgentEffect {
    /// New facts to add to context.
    pub facts: Vec<Fact>,
    /// New proposals to be validated by the engine.
    pub proposals: Vec<ProposedFact>,
}

impl AgentEffect {
    /// Creates an empty effect (no contributions).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates an effect with a single fact.
    #[must_use]
    pub fn with_fact(fact: Fact) -> Self {
        Self {
            facts: vec![fact],
            proposals: Vec::new(),
        }
    }

    /// Creates an effect with multiple facts.
    #[must_use]
    pub fn with_facts(facts: Vec<Fact>) -> Self {
        Self {
            facts,
            proposals: Vec::new(),
        }
    }

    /// Creates an effect with a single proposal.
    #[must_use]
    pub fn with_proposal(proposal: ProposedFact) -> Self {
        Self {
            facts: Vec::new(),
            proposals: vec![proposal],
        }
    }

    /// Returns true if this effect contributes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty() && self.proposals.is_empty()
    }

    /// Returns the context keys affected by this effect.
    #[must_use]
    pub fn affected_keys(&self) -> Vec<ContextKey> {
        let mut keys: Vec<ContextKey> = self
            .facts
            .iter()
            .map(|f| f.key)
            .chain(self.proposals.iter().map(|p| p.key))
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }
}
