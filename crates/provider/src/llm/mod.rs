// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! LLM Backend implementations for the unified `LlmBackend` trait.
//!
//! This module provides implementations of `converge_core::backend::LlmBackend`
//! for remote providers. These backends produce `RemoteTraceLink` artifacts
//! for audit (not replay).
//!
//! # Architecture
//!
//! ```text
//! converge-core::backend
//!     │
//!     │  LlmBackend trait (pure interface)
//!     ▼
//! converge-provider::llm
//!     │
//!     ├── AnthropicBackend  → RemoteTraceLink (audit-only)
//!     ├── OpenAIBackend     → RemoteTraceLink (audit-only)
//!     └── ...               → RemoteTraceLink (audit-only)
//! ```
//!
//! # Provider vs Backend
//!
//! - `LlmProvider` (simple): prompt → content (no trace semantics)
//! - `LlmBackend` (traced): request → response with `TraceLink`, proposals, contracts
//!
//! The `LlmBackend` interface is used by converge-llm's kernel and supports
//! both local (replay-eligible) and remote (audit-only) backends.

mod anthropic;

pub use anthropic::AnthropicBackend;
