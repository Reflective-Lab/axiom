//! MCP server implementation.
//!
//! Provides transport layers (stdio and HTTP) for serving MCP requests.
//! Implement [`McpRequestHandler`] to provide your own tool/resource logic.

mod transport;

pub use transport::{McpRequestHandler, McpServer};
