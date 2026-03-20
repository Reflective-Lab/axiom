// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Job execution module for Converge Runtime.
//!
//! This module bridges the HTTP/gRPC/SSE interfaces to the actual
//! converge-core Engine and converge-domain agents.
//!
//! # Architecture
//!
//! ```text
//! REST/gRPC/SSE Handlers
//!         │
//!         ▼
//!    JobExecutor  ◄── Templates/Packs
//!         │
//!         ▼
//!    converge-core::Engine
//!         │
//!         ▼
//!    converge-domain Agents
//! ```
//!
//! # Usage
//!
//! ```ignore
//! let executor = JobExecutor::new();
//! let result = executor
//!     .with_pack("growth-strategy")
//!     .with_seeds(seeds)
//!     .with_budget(budget)
//!     .with_streaming(callback)
//!     .execute()
//!     .await?;
//! ```

mod executor;
mod packs;
mod streaming;

pub use executor::{ExecutionResult, JobExecutor, JobExecutorBuilder};
pub use packs::{LlmConfig, PackDefinition, PackRegistry, register_pack_agents};
pub use streaming::{EventReceiver, EventSender, RuntimeStreamingCallback, StreamingEvent};
