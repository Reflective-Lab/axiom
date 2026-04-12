// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Suggestor effects — what suggestors produce, the engine merges.
//!
//! Effects are proposal-only. Suggestors suggest; the engine validates and promotes.

use crate::context::ContextKey;
use crate::fact::ProposedFact;

/// The output of a suggestor's `execute()` call.
///
/// An effect describes what a suggestor wants to suggest to the context.
/// The engine collects effects from all eligible suggestors, validates them,
/// and promotes them serially in deterministic order.
#[derive(Debug, Default)]
pub struct AgentEffect {
    /// New proposals to be validated by the engine.
    pub proposals: Vec<ProposedFact>,
}

impl AgentEffect {
    /// Creates an empty effect (no contributions).
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates an effect with a single proposal.
    #[must_use]
    pub fn with_proposal(proposal: ProposedFact) -> Self {
        Self {
            proposals: vec![proposal],
        }
    }

    /// Creates an effect with multiple proposals.
    #[must_use]
    pub fn with_proposals(proposals: Vec<ProposedFact>) -> Self {
        Self { proposals }
    }

    /// Returns true if this effect contributes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.proposals.is_empty()
    }

    /// Returns the context keys affected by this effect.
    #[must_use]
    pub fn affected_keys(&self) -> Vec<ContextKey> {
        let mut keys: Vec<ContextKey> = self.proposals.iter().map(|p| p.key).collect();
        keys.sort();
        keys.dedup();
        keys
    }
}
