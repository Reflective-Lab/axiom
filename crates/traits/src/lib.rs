// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! # Converge Traits — The Public Contract
//!
//! This crate defines the traits and types that the entire Converge ecosystem
//! compiles against. It has two layers:
//!
//! ## Convergence Engine Contract
//!
//! The core abstractions for building convergent multi-agent systems:
//!
//! - [`Agent`] — the trait all agents implement (accepts + execute over context).
//! - [`Context`] — read-only view of shared state, keyed by [`ContextKey`].
//! - [`Fact`] / [`ProposedFact`] — the type boundary between authoritative
//!   assertions and unvalidated suggestions.
//! - [`AgentEffect`] — what agents produce; the engine merges effects
//!   deterministically.
//! - [`Invariant`] — executable guarantees checked at structural, semantic,
//!   and acceptance phases.
//!
//! ## Backend Abstraction Layer
//!
//! Generic traits for external capabilities (LLM, policy, optimization,
//! analytics). All instantiation types are first-class:
//!
//! - [`Backend`] — identity and capability declaration.
//! - [`Capability`] — fine-grained capability flags.
//! - [`BackendRequirements`] / [`BackendSelector`] — capability-based selection.
//!
//! ## Design Principles
//!
//! 1. **Agents suggest, engines decide.** `ProposedFact` is not `Fact`.
//! 2. **Context is the API.** Agents communicate through shared context, never
//!    by calling each other.
//! 3. **Minimal dependencies.** Only `serde` + `thiserror`. No async, no I/O.
//! 4. **All backend types are first-class.** No privileged instantiation type.
//!
//! ## Crate Map
//!
//! | Crate | Role | Implements |
//! |-------|------|------------|
//! | `converge-traits` | This crate — the public contract | — |
//! | `converge-core` | The convergence engine (private) | Uses all traits |
//! | `converge-provider` | Cloud LLM backends | `Backend`, `Agent` |
//! | `converge-llm` | Local LLM inference | `Backend`, `Agent` |
//! | `converge-policy` | Policy engines (Cedar, OPA) | `Backend`, `Agent` |
//! | `converge-optimization` | Constraint solvers | `Backend`, `Agent` |
//! | `converge-analytics` | ML/analytics | `Backend`, `Agent` |

// ── Convergence engine contract ──────────────────────────────────────
pub mod agent;
pub mod context;
pub mod effect;
pub mod fact;
pub mod invariant;

// ── Backend abstraction layer ────────────────────────────────────────
pub mod backend;
pub mod capability;
pub mod error;
pub mod selection;

#[cfg(test)]
mod tests;

// ── Convergence re-exports ───────────────────────────────────────────
pub use agent::Agent;
pub use context::{Context, ContextKey};
pub use effect::AgentEffect;
pub use fact::{Fact, ProposedFact, ValidationError};
pub use invariant::{Invariant, InvariantClass, InvariantResult};

// ── Backend re-exports ───────────────────────────────────────────────
pub use backend::{Backend, BackendKind};
pub use capability::Capability;
pub use error::{BackendError, BackendErrorKind};
pub use selection::{
    BackendRequirements, BackendSelector, ComplianceLevel, CostClass, DataSovereignty,
};
