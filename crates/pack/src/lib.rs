// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! # Converge Pack
//!
//! This crate is the strict Rust authoring contract for Converge packs.
//! External modules implement these traits to participate in convergence:
//!
//! - [`Suggestor`] for pure suggestors
//! - [`Context`] for read-only context access
//! - [`AgentEffect`] for buffered proposal output
//! - [`Fact`] / [`ProposedFact`] for the current context boundary
//! - [`Invariant`] for executable guarantees
//!
//! Provider selection and backend capability routing do not live here.
//! Those contracts belong to `converge-provider-api`.

mod agent;
pub mod context;
pub mod effect;
pub mod fact;
pub mod invariant;

pub mod suggestor {
    pub use super::agent::Suggestor;
}

pub use agent::Suggestor;
pub use context::{Context, ContextKey};
pub use effect::AgentEffect;
pub use fact::{
    Fact, FactActor, FactActorKind, FactEvidenceRef, FactLocalTrace, FactPromotionRecord,
    FactRemoteTrace, FactTraceLink, FactValidationSummary, ProposedFact, ValidationError,
};
pub use invariant::{Invariant, InvariantClass, InvariantResult};
