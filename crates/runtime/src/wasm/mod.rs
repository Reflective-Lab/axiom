// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Converge WASM Runtime
//!
//! Sandboxed execution environment for tenant-supplied WASM modules.
//!
//! This module provides the **capability surface** for running custom
//! JTBD invariants and agents compiled from Gherkin specs to WASM.
//!
//! # Architecture Role
//!
//! > `converge-runtime::wasm` owns **sandboxed guest execution**.
//!
//! It does not define domain semantics (that's `converge-domain`),
//! nor engine behavior (that's `converge-core`). It provides the
//! bridge between tenant-authored WASM modules and the Converge engine.
//!
//! # Modules
//!
//! - [`contract`] — Types and ABI definitions for the host↔guest boundary
//!
//! # Feature Gate
//!
//! This module is gated behind `#[cfg(feature = "wasm")]`.

pub mod contract;

#[cfg(feature = "wasm-runtime")]
pub mod adapter;

#[cfg(feature = "wasm-runtime")]
pub mod engine;

#[cfg(feature = "wasm-runtime")]
pub mod host;

#[cfg(feature = "wasm-runtime")]
pub mod integration;

#[cfg(feature = "wasm-runtime")]
pub mod signing;

#[cfg(feature = "wasm-runtime")]
pub mod store;
