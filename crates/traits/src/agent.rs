// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! The Agent trait — the contract all agent implementations satisfy.
//!
//! # Why this shape?
//!
//! Agents in Converge are not actors, not services, not workflow steps.
//! They are pure functions over context: given the current state of shared
//! context, an agent decides whether to act (`accepts`) and what to
//! contribute (`execute`).
//!
//! This design makes agents:
//! - **Deterministic**: same context → same decision.
//! - **Composable**: agents don't know about each other, only about context.
//! - **Testable**: mock the context, assert the effect.
//!
//! # Critical rules
//!
//! - `accepts()` must be **pure** — no side effects, no I/O, no mutations.
//! - `execute()` is **read-only** — it reads context and returns an effect.
//! - Agents **never call other agents** — all communication via shared context.
//! - **Idempotency is context-based** — check for existing contributions in
//!   context, not internal state. Internal `has_run` flags violate the
//!   "context is the only shared state" axiom.

use crate::context::{Context, ContextKey};
use crate::effect::AgentEffect;

/// The core agent contract.
///
/// Every agent in the Converge ecosystem implements this trait — whether
/// it wraps an LLM, a policy engine, an optimizer, or a simple rule.
///
/// The engine calls `accepts()` to determine eligibility, then `execute()`
/// to collect effects. Effects are merged by the engine in deterministic
/// order (sorted by agent name).
///
/// # Thread Safety
///
/// Agents must be `Send + Sync` because the engine executes eligible
/// agents in parallel (via Rayon). Agent state, if any, must be
/// internally synchronized.
pub trait Agent: Send + Sync {
    /// Human-readable name, used for ordering, logging, and provenance.
    ///
    /// Must be unique within a convergence run. The engine sorts agents
    /// by name to ensure deterministic merge order.
    fn name(&self) -> &str;

    /// Context keys this agent reads from.
    ///
    /// The engine uses this to determine when an agent becomes eligible:
    /// an agent is a candidate when at least one of its dependency keys
    /// has been modified since the last cycle.
    fn dependencies(&self) -> &[ContextKey];

    /// Pure predicate: should this agent execute given the current context?
    ///
    /// # Contract
    ///
    /// - Must be **pure**: no side effects, no I/O, no state mutation.
    /// - Must be **deterministic**: same context → same answer.
    /// - Must check **idempotency via context**: look for your own
    ///   contributions in context (both `Proposals` and target key),
    ///   not internal flags.
    fn accepts(&self, ctx: &dyn Context) -> bool;

    /// Produce effects given the current context.
    ///
    /// # Contract
    ///
    /// - **Read-only**: do not mutate context. Return effects instead.
    /// - Effects are collected by the engine and merged after all
    ///   eligible agents have executed.
    /// - For LLM agents: emit `ProposedFact` to `ContextKey::Proposals`,
    ///   not directly to the target key.
    fn execute(&self, ctx: &dyn Context) -> AgentEffect;
}
