// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Core type vocabulary for Converge.
//!
//! This module contains the domain types that form Converge's type vocabulary:
//!
//! - **ID types** (id.rs): FactId, ObservationId, ProposalId, etc.
//! - **3-tier hierarchy**: Observation -> Proposal -> Fact
//! - **Provenance** (provenance.rs): PromotionRecord, EvidenceRef, TraceLink
//! - **Six-phase flow** (frame.rs, tension.rs): Frame, Tension, Hypothesis
//! - **Intent** (intent.rs): TypesRootIntent with builder
//! - **Context** (context.rs): ContextBuilder, TypesContextSnapshot
//! - **Corrections** (correction.rs): CorrectionEvent for append-only
//! - **Errors** (error.rs): TypeError, PromotionError, etc.
//!
//! # Design Principles
//!
//! - **Type safety**: All IDs are newtypes to prevent mixing
//! - **Promotion invariant**: Facts can only be created via [`crate::gates::PromotionGate`]
//! - **Typed provenance**: EvidenceRef and TraceLink are enums, not strings
//! - **Immutability**: Facts have private fields, no `&mut` methods
//! - **Builder patterns**: Complex types use typed-builder for ergonomic construction
//! - **Error handling**: All errors use thiserror for derive macros
//!
//! # Design Tenets Alignment
//!
//! This module directly supports these tenets from [`crate`]:
//!
//! | Tenet | How This Module Supports It |
//! |-------|----------------------------|
//! | **Safety by Construction** | Newtype IDs, type-state on Proposal, private Fact fields |
//! | **Append-Only Truth** | Fact immutability, CorrectionEvent for changes |
//! | **Explicit Authority** | PromotionRecord traces who approved each Fact |
//! | **Transparent Determinism** | TraceLink distinguishes Local (replay) from Remote (audit) |
//! | **Human Authority First-Class** | Actor/ActorKind distinguish human vs automated |
//!
//! # Cross-Module References
//!
//! - **Gates**: [`crate::gates::PromotionGate`] is the only path to create [`Fact`]
//! - **Traits**: [`crate::traits::Validator`] validates Proposal, [`crate::traits::Promoter`] promotes to Fact

pub mod context;
pub mod correction;
pub mod error;
pub mod fact;
pub mod frame;
pub mod id;
pub mod intent;
pub mod observation;
pub mod proposal;
pub mod provenance;
pub mod tension;

// Re-export everything for convenience
pub use context::*;
pub use correction::*;
pub use error::*;
pub use fact::*;
pub use frame::*;
pub use id::*;
pub use intent::*;
pub use observation::*;
pub use proposal::*;
pub use provenance::*;
pub use tension::*;
