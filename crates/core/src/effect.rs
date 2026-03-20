// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Agent effects for Converge.
//!
//! Agents emit effects instead of mutating context directly.
//! This enables transactional semantics, conflict detection,
//! and deterministic ordering.

use crate::context::{ContextKey, Fact, ProposedFact};

/// The output of an agent's execution.
///
/// Effects are:
/// - Immutable once created
/// - Self-contained
/// - Merged serially by the engine
#[derive(Debug, Default)]
pub struct AgentEffect {
    /// New facts to add to context.
    pub facts: Vec<Fact>,
    /// New proposals to be validated by the engine.
    pub proposals: Vec<ProposedFact>,
    // Future: intents, evaluations, trace events
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
    ///
    /// Used by the engine to determine which agents to re-evaluate
    /// after merging this effect (dependency-indexed eligibility).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_effect_is_empty() {
        let effect = AgentEffect::empty();
        assert!(effect.is_empty());
    }

    #[test]
    fn effect_with_fact_is_not_empty() {
        let fact = Fact {
            key: ContextKey::Seeds,
            id: "test".into(),
            content: "value".into(),
        };
        let effect = AgentEffect::with_fact(fact);
        assert!(!effect.is_empty());
        assert_eq!(effect.facts.len(), 1);
    }

    #[test]
    fn effect_with_proposal_is_not_empty() {
        let proposal = ProposedFact {
            key: ContextKey::Hypotheses,
            id: "prop-1".into(),
            content: "value".into(),
            confidence: 0.8,
            provenance: "test".into(),
        };
        let effect = AgentEffect::with_proposal(proposal);
        assert!(!effect.is_empty());
        assert_eq!(effect.proposals.len(), 1);
    }

    #[test]
    fn affected_keys_returns_unique_keys() {
        let facts = vec![Fact {
            key: ContextKey::Seeds,
            id: "a".into(),
            content: "1".into(),
        }];
        let proposals = vec![ProposedFact {
            key: ContextKey::Hypotheses,
            id: "c".into(),
            content: "3".into(),
            confidence: 0.9,
            provenance: "test".into(),
        }];
        let mut effect = AgentEffect::with_facts(facts);
        effect.proposals = proposals;
        let keys = effect.affected_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&ContextKey::Seeds));
        assert!(keys.contains(&ContextKey::Hypotheses));
    }

    #[test]
    fn empty_effect_has_no_affected_keys() {
        let effect = AgentEffect::empty();
        assert!(effect.affected_keys().is_empty());
    }
}
