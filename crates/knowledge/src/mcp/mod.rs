//! Model Context Protocol (MCP) server for Claude Desktop integration.
//!
//! This module implements an MCP server that exposes the knowledge base
//! as tools for Claude Desktop.

mod handlers;
mod server;
mod types;

pub use server::McpServer;
pub use types::*;
