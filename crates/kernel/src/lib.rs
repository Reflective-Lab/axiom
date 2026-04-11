// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Converge Kernel
//!
//! This crate is the curated in-process execution API for Converge.
//! Consumers embed the kernel here; they author packs in `converge-pack`
//! and use `converge-model` for shared semantic types.

pub use converge_core::{
    SuggestorId,
    Budget,
    ConvergeError,
    ConvergeResult,
    Context,
    CriterionEvaluator,
    Engine,
    EngineHitlPolicy,
    ExperienceEvent,
    ExperienceEventEnvelope,
    ExperienceEventKind,
    ExperienceEventObserver,
    HitlPause,
    RunResult,
    StreamingCallback,
    TypesRunHooks,
};
pub use converge_pack::{
    Suggestor,
    AgentEffect,
    Context as ContextView,
    ContextKey,
    Fact,
    Invariant,
    InvariantClass,
    InvariantResult,
    ProposedFact,
    ValidationError,
};
