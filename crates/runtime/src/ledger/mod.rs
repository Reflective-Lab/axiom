// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Ledger client for connecting to converge-ledger.
//!
//! This module provides a gRPC client for the append-only context store.
//!
//! # Architecture
//!
//! The converge-ledger is an Elixir/Mnesia service that provides:
//! - Append-only entry storage with sequence numbers
//! - Lamport clocks for causal ordering across distributed contexts
//! - Merkle trees for tamper detection
//! - Snapshots for persistence/recovery
//!
//! This client connects to the ledger via gRPC and provides async Rust APIs.
//!
//! # Usage
//!
//! ```ignore
//! use converge_runtime::ledger::LedgerClient;
//!
//! let client = LedgerClient::connect("http://localhost:50051").await?;
//!
//! // Append an entry
//! let entry = client.append("my-context", "facts", b"payload").await?;
//!
//! // Get entries
//! let entries = client.get("my-context", None, None, None).await?;
//!
//! // Create a snapshot
//! let snapshot = client.snapshot("my-context").await?;
//!
//! // Watch for new entries
//! let mut stream = client.watch("my-context", None, None).await?;
//! while let Some(event) = stream.next().await {
//!     println!("New entry: {:?}", event?.entry);
//! }
//! ```

mod client;
#[cfg(test)]
mod tests;

pub use client::{LedgerClient, LedgerError, LedgerResult};

// Re-export generated types for convenience
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
mod generated {
    include!("generated/converge.context.v1.rs");
}

pub use generated::{
    AppendRequest, AppendResponse, Entry, GetRequest, GetResponse, LoadRequest, LoadResponse,
    SnapshotMetadata, SnapshotRequest, SnapshotResponse, WatchEvent, WatchRequest,
};
