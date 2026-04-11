// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! Invariants — the engine's guarantee system.
//!
//! # Why invariants instead of validation functions?
//!
//! Converge doesn't have features — it has guarantees. Invariants are
//! those guarantees made executable. They run at specific phases of the
//! convergence loop and can block progress, reject results, or halt
//! the engine entirely.
//!
//! The three classes correspond to when guarantees are checked:
//! - **Structural**: every merge. Catches corruption immediately.
//! - **Semantic**: end of cycle. Ensures logical consistency.
//! - **Acceptance**: convergence claim. Final gate before results.

use serde::{Deserialize, Serialize};

use crate::context::Context;

/// The class of an invariant determines when it runs.
///
/// Invariants are not all equal. Some must hold at every merge (structural
/// integrity). Others only matter at cycle boundaries (semantic consistency).
/// Acceptance invariants are the final gate — they decide whether the
/// converged result is actually acceptable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InvariantClass {
    /// Checked on every merge operation.
    ///
    /// Structural invariants protect data integrity. If they fail,
    /// the merge is rejected immediately. Example: "no duplicate fact IDs."
    Structural,

    /// Checked at the end of each convergence cycle.
    ///
    /// Semantic invariants ensure logical consistency across the full
    /// context. If they fail, convergence cannot be claimed for this
    /// cycle. Example: "every strategy must reference at least one signal."
    Semantic,

    /// Checked when the engine claims convergence (fixed point reached).
    ///
    /// Acceptance invariants are the final quality gate. If they fail,
    /// the result is rejected even though the engine reached a fixed
    /// point. Example: "at least 3 independent strategies must exist."
    Acceptance,
}

/// The result of checking an invariant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantResult {
    /// The invariant holds.
    Ok,
    /// The invariant is violated, with an explanation.
    Violation(String),
}

impl InvariantResult {
    /// Whether this result represents a passing check.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Whether this result represents a violation.
    #[must_use]
    pub fn is_violation(&self) -> bool {
        matches!(self, Self::Violation(_))
    }
}

/// An executable invariant that the engine checks during convergence.
///
/// Invariants are registered with the engine alongside suggestors. The engine
/// calls `check()` at the appropriate phase based on `class()`.
///
/// # Thread Safety
///
/// Invariants must be `Send + Sync` because the engine may check multiple
/// invariants concurrently.
pub trait Invariant: Send + Sync {
    /// Human-readable name for diagnostics and logging.
    fn name(&self) -> &str;

    /// When this invariant should be checked.
    fn class(&self) -> InvariantClass;

    /// Check whether this invariant holds for the given context.
    ///
    /// # Contract
    ///
    /// - Must be **pure**: no side effects, no I/O.
    /// - Must be **deterministic**: same context → same result.
    fn check(&self, ctx: &dyn Context) -> InvariantResult;
}
