// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! # Converge Provider API
//!
//! This crate defines the provider-facing capability contract for Converge.
//! It is deliberately separate from pack authoring:
//!
//! - [`Backend`] and [`BackendKind`] describe provider identity
//! - [`Capability`] describes what a provider can do
//! - [`BackendRequirements`] and [`BackendSelector`] drive routing
//! - [`BackendError`] is the generic provider error surface
//!
//! Suggestor and invariant authoring do not live here. Those contracts belong to
//! `converge-pack`.

pub mod backend;
pub mod capability;
pub mod error;
pub mod selection;

pub use backend::{Backend, BackendKind};
pub use capability::Capability;
pub use error::{BackendError, BackendErrorKind};
pub use selection::{
    BackendRequirements, BackendSelector, ComplianceLevel, CostClass, DataSovereignty,
};
