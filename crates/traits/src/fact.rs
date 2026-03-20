// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Facts and proposed facts — the type boundary.
//!
//! This is the most important design decision in Converge: LLMs suggest,
//! the engine validates. `ProposedFact` is not `Fact`. There is no implicit
//! conversion between them.
//!
//! # Why two types?
//!
//! A `Fact` is an authoritative assertion in the context. It was either
//! provided by a human (seed), produced by a deterministic agent (policy,
//! optimizer), or explicitly promoted from a `ProposedFact` by a
//! `ValidationAgent`.
//!
//! A `ProposedFact` is a suggestion from a non-authoritative source (typically
//! an LLM). It lives in `ContextKey::Proposals` until validated. This
//! separation is what makes Converge trustworthy: you can always distinguish
//! between "an LLM said this" and "the system asserts this."
//!
//! # Security implications
//!
//! Any weakening of this boundary — implicit promotion, auto-validation,
//! or type coercion — is a correctness and security issue. Treat changes
//! to these types as requiring security review.

use serde::{Deserialize, Serialize};

use crate::context::ContextKey;

/// A validated, authoritative assertion in the context.
///
/// Facts are append-only. Once added to the context, they are never
/// mutated or removed (within a convergence run). History is preserved.
///
/// # Identity
///
/// The `id` field uniquely identifies this fact within its key namespace.
/// Convention: `"{agent_name}-{uuid}"` for agent-produced facts,
/// `"seed:{name}"` for initial seeds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// Unique identifier within the context key namespace.
    pub id: String,
    /// Which context key this fact belongs to.
    pub key: ContextKey,
    /// The fact's content as a string. Interpretation is key-dependent.
    pub content: String,
}

/// An unvalidated suggestion from a non-authoritative source.
///
/// Proposed facts live in `ContextKey::Proposals` until a `ValidationAgent`
/// promotes them to `Fact` by writing to the target key. The proposal
/// tracks its origin for audit trail.
///
/// # ID Convention
///
/// Proposal IDs encode their lineage:
/// `"proposal:{target_key}:{agent_name}-{uuid}"`
///
/// After promotion, the resulting `Fact` gets ID:
/// `"{agent_name}-{uuid}"` (the proposal prefix is stripped).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedFact {
    /// Unique identifier encoding origin and target.
    pub id: String,
    /// The context key this proposal targets (where it would go if validated).
    pub target_key: ContextKey,
    /// The proposed content.
    pub content: String,
    /// Which agent produced this proposal.
    pub source_agent: String,
}
