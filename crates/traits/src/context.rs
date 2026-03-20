// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Context keys and the shared context contract.
//!
//! Context is the API. Agents don't call each other — they read from and
//! write to shared context through typed keys. This module defines those
//! keys and the read-only view agents receive during execution.
//!
//! # Why typed keys?
//!
//! Untyped string keys invite typos and silent data loss. Typed keys make
//! the context a structured, compile-time-checked namespace. Each key
//! represents a semantic category of facts, not a storage bucket.

use serde::{Deserialize, Serialize};

/// Typed keys for the shared context namespace.
///
/// Each key represents a semantic category of facts. Agents declare their
/// dependencies as a set of keys, and the engine uses those declarations
/// to determine execution eligibility.
///
/// # Why these categories?
///
/// These map to the convergence lifecycle:
/// - **Seeds** start the process (human intent).
/// - **Hypotheses**, **Strategies**, **Signals**, **Competitors** are
///   agent-contributed analysis.
/// - **Constraints** and **Evaluations** narrow the solution space.
/// - **Proposals** hold unvalidated LLM output (the trust boundary).
/// - **Diagnostic** captures errors without polluting analysis keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextKey {
    /// Initial inputs from the root intent. Set once at initialization.
    Seeds,
    /// Proposed ideas and hypotheses from analysis agents.
    Hypotheses,
    /// Action plans and strategic recommendations.
    Strategies,
    /// Limitations, rules, and boundary conditions.
    Constraints,
    /// Observations, market data, and signals from the environment.
    Signals,
    /// Competitive intelligence and comparisons.
    Competitors,
    /// Assessments, ratings, and evaluations of other facts.
    Evaluations,
    /// LLM-generated suggestions awaiting validation.
    ///
    /// This is the trust boundary. Facts here are `ProposedFact`, not `Fact`.
    /// A `ValidationAgent` must promote them before they become authoritative.
    Proposals,
    /// Error and debugging information. Never blocks convergence.
    Diagnostic,
}

/// Read-only view of the shared context.
///
/// Agents receive this during `accepts()` and `execute()`. They cannot
/// mutate it directly — mutations happen through `AgentEffect` after
/// the engine collects all effects and merges them deterministically.
///
/// # Why read-only?
///
/// If agents could mutate context directly, execution order would matter
/// and determinism would be lost. By collecting effects and merging them
/// in a deterministic order (sorted by AgentId), the engine guarantees
/// that the same inputs always produce the same outputs.
pub trait Context: Send + Sync {
    /// Check whether any facts exist under this key.
    fn has(&self, key: ContextKey) -> bool;

    /// Get all facts under this key as serialized JSON values.
    ///
    /// Returns an empty slice if no facts exist for the key.
    fn get(&self, key: ContextKey) -> &[Fact];

    /// Get all proposed facts under this key.
    ///
    /// Proposed facts are distinct from validated facts. This method
    /// returns only the unvalidated proposals for a given key.
    fn get_proposals(&self, key: ContextKey) -> &[ProposedFact];

    /// Count of facts under a key. Useful for budget checks.
    fn count(&self, key: ContextKey) -> usize {
        self.get(key).len()
    }
}

use crate::fact::{Fact, ProposedFact};
