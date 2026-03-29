// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Converge MCP — Model Context Protocol client and server.
//!
//! This crate provides a unified MCP implementation for the Converge ecosystem:
//!
//! - **Types**: JSON-RPC protocol types shared by client and server
//! - **Client** (`client` feature): Connect to external MCP servers
//! - **Server** (`server` feature): Expose tools/resources via MCP
//!
//! # Usage
//!
//! ```toml
//! # Client only
//! converge-mcp = { version = "1.1", default-features = false, features = ["client"] }
//!
//! # Server only
//! converge-mcp = { version = "1.1", default-features = false, features = ["server"] }
//! ```

mod types;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

pub use types::*;
