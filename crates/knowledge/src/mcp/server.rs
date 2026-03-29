//! MCP server implementation — delegates to converge-mcp transport.

use super::handlers::KnowledgeHandler;
use crate::core::KnowledgeBase;
use crate::error::Result;

use converge_mcp::server::McpServer as GenericMcpServer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// MCP server for Claude Desktop integration.
///
/// Wraps the generic `converge_mcp::McpServer` with a knowledge-specific handler.
pub struct McpServer {
    inner: GenericMcpServer<KnowledgeHandler>,
}

impl McpServer {
    /// Create a new MCP server backed by the given knowledge base.
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
        let handler = KnowledgeHandler::new(kb);
        Self {
            inner: GenericMcpServer::new(handler),
        }
    }

    /// Run the MCP server over stdio (for Claude Desktop).
    pub async fn run_stdio(&self) -> Result<()> {
        info!("Starting Knowledge MCP server on stdio");
        self.inner
            .run_stdio()
            .await
            .map_err(|e| crate::error::Error::storage(e.to_string()))
    }

    /// Run the MCP server over HTTP (for SSE transport).
    pub async fn run_http(&self, addr: &str) -> Result<()> {
        info!("Starting Knowledge MCP HTTP server on {addr}");
        self.inner
            .run_http(addr)
            .await
            .map_err(|e| crate::error::Error::storage(e.to_string()))
    }
}
