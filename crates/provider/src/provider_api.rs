// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! LLM provider invocation types.
//!
//! Re-exports from `converge-core` so all crates share a single `LlmProvider` trait.
//! The platform-wide contract (`Backend`) lives in `converge-provider-api`,
//! while pack authoring lives in `converge-pack`.

// ── LLM types (canonical home: converge-core::llm) ─────────────────
pub use converge_core::llm::{
    FinishReason, LlmError, LlmErrorKind, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};

// ── Selection types (canonical home: converge-core::model_selection) ──
pub use converge_core::model_selection::{AgentRequirements, ModelSelectorTrait};

// ── Routing enums (canonical home: converge-core::model_selection) ───
// These must come from core (not provider-api) so they match AgentRequirements.
pub use converge_core::model_selection::{ComplianceLevel, CostClass, DataSovereignty};
