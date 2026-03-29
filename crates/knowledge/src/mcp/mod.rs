//! Model Context Protocol (MCP) server for Claude Desktop integration.
//!
//! This module implements an MCP server that exposes the knowledge base
//! as tools for Claude Desktop.

mod handlers;
mod server;

pub use server::McpServer;
// Re-export MCP types for consumers that don't depend on converge-mcp directly
pub use converge_mcp::*;
