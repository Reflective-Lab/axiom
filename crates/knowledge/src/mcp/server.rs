//! MCP server implementation.

use super::handlers::McpHandler;
use super::types::JsonRpcRequest;
use crate::core::KnowledgeBase;
use crate::error::Result;

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

/// MCP server for Claude Desktop integration.
pub struct McpServer {
    handler: Arc<McpHandler>,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>) -> Self {
        Self {
            handler: Arc::new(McpHandler::new(kb)),
        }
    }

    /// Run the MCP server over stdio (for Claude Desktop).
    pub async fn run_stdio(&self) -> Result<()> {
        info!("Starting MCP server on stdio");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);

        let mut line = String::new();

        loop {
            line.clear();

            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    // Parse JSON-RPC request
                    match serde_json::from_str::<JsonRpcRequest>(line) {
                        Ok(request) => {
                            let response = self.handler.handle(request).await;
                            let response_json = serde_json::to_string(&response).unwrap();

                            stdout.write_all(response_json.as_bytes()).await.ok();
                            stdout.write_all(b"\n").await.ok();
                            stdout.flush().await.ok();
                        }
                        Err(e) => {
                            let error_response = super::types::JsonRpcResponse::error(
                                None,
                                -32700,
                                format!("Parse error: {}", e),
                            );
                            let response_json = serde_json::to_string(&error_response).unwrap();

                            stdout.write_all(response_json.as_bytes()).await.ok();
                            stdout.write_all(b"\n").await.ok();
                            stdout.flush().await.ok();
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Run the MCP server over HTTP (for SSE transport).
    pub async fn run_http(&self, addr: &str) -> Result<()> {
        info!("Starting MCP HTTP server on {}", addr);

        let handler = self.handler.clone();

        let app = Router::new()
            .route("/", get(root))
            .route("/mcp", post(handle_mcp))
            .route("/health", get(health))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(handler);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| crate::error::Error::storage(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::Error::storage(e.to_string()))?;

        Ok(())
    }
}

/// Root handler.
async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "converge-knowledge",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "mcp",
        "transports": ["stdio", "http"]
    }))
}

/// Health check handler.
async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy"
    }))
}

/// MCP request handler.
async fn handle_mcp(
    State(handler): State<Arc<McpHandler>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let response = handler.handle(request).await;
    Json(response)
}
