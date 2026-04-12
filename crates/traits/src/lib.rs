// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! Deprecated compatibility facade.
//!
//! New code should depend on:
//! - `converge-pack` for suggestor, context, and invariant authoring
//! - `converge-provider-api` for backend identity and capability routing

pub use converge_pack::{
    AgentEffect, Context, ContextKey, Fact, ProposedFact, Suggestor, ValidationError,
};
pub use converge_provider_api::{
    Backend, BackendError, BackendErrorKind, BackendKind, BackendRequirements, BackendSelector,
    Capability, ComplianceLevel, CostClass, DataSovereignty,
};
