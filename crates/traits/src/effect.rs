// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Agent effects — what agents produce, the engine merges.
//!
//! # Why effects instead of direct mutation?
//!
//! If agents mutated context directly, the order of execution would
//! determine the outcome. By having agents return effects and letting
//! the engine merge them in a deterministic order (sorted by agent name),
//! we guarantee that the same set of agents with the same context always
//! produces the same result — regardless of parallel execution timing.

use serde::{Deserialize, Serialize};

use crate::fact::{Fact, ProposedFact};

/// The output of an agent's `execute()` call.
///
/// An effect describes what an agent wants to contribute to the context.
/// The engine collects effects from all eligible agents, then merges them
/// serially in deterministic order.
///
/// # Variants
///
/// - `AddFacts`: deterministic agents (policy, optimizer, rules) emit
///   validated facts directly.
/// - `Propose`: LLM agents emit proposals that require validation.
/// - `Nothing`: the agent ran but had nothing to contribute (legitimate
///   for guard/gate agents that only check conditions).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentEffect {
    /// Emit validated facts into the context.
    ///
    /// Used by deterministic agents whose output is authoritative:
    /// policy engines, constraint solvers, rule evaluators, seed providers.
    AddFacts(Vec<Fact>),

    /// Emit proposals that require validation before becoming facts.
    ///
    /// Used by LLM agents and other non-authoritative sources. Proposals
    /// go to `ContextKey::Proposals` and must be promoted by a
    /// `ValidationAgent`.
    Propose(Vec<ProposedFact>),

    /// The agent executed but produced no output.
    ///
    /// This is not an error. Gate agents, for example, may check a
    /// condition and decide that no action is needed.
    Nothing,
}

impl AgentEffect {
    /// Create an effect that adds validated facts.
    #[must_use]
    pub fn with_facts(facts: Vec<Fact>) -> Self {
        Self::AddFacts(facts)
    }

    /// Create an effect that proposes facts for validation.
    #[must_use]
    pub fn with_proposals(proposals: Vec<ProposedFact>) -> Self {
        Self::Propose(proposals)
    }

    /// Whether this effect contributes anything to the context.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::AddFacts(facts) => facts.is_empty(),
            Self::Propose(proposals) => proposals.is_empty(),
            Self::Nothing => true,
        }
    }
}
