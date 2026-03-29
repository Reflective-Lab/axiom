//! MCP server transport (stdio + HTTP).

use crate::types::{JsonRpcRequest, JsonRpcResponse};

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Trait for handling MCP requests.
///
/// Implement this trait to provide custom tool/resource logic.
/// The server transport will route all JSON-RPC requests to this handler.
#[async_trait::async_trait]
pub trait McpRequestHandler: Send + Sync + 'static {
    /// Handle an incoming MCP JSON-RPC request and return a response.
    async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse;
}

/// MCP server with pluggable request handler.
pub struct McpServer<H: McpRequestHandler> {
    handler: Arc<H>,
}

impl<H: McpRequestHandler> McpServer<H> {
    /// Create a new MCP server with the given handler.
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    /// Create a new MCP server from a shared handler.
    pub fn from_arc(handler: Arc<H>) -> Self {
        Self { handler }
    }

    /// Run the MCP server over stdio (for Claude Desktop).
    pub async fn run_stdio(&self) -> std::io::Result<()> {
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
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                        Ok(request) => {
                            let response = self.handler.handle(request).await;
                            let response_json = serde_json::to_string(&response)
                                .expect("Failed to serialize response");

                            stdout.write_all(response_json.as_bytes()).await.ok();
                            stdout.write_all(b"\n").await.ok();
                            stdout.flush().await.ok();
                        }
                        Err(e) => {
                            let error_response =
                                JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"));
                            let response_json = serde_json::to_string(&error_response)
                                .expect("Failed to serialize error");

                            stdout.write_all(response_json.as_bytes()).await.ok();
                            stdout.write_all(b"\n").await.ok();
                            stdout.flush().await.ok();
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error reading stdin: {e}");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Run the MCP server over HTTP.
    pub async fn run_http(&self, addr: &str) -> std::io::Result<()> {
        info!("Starting MCP HTTP server on {addr}");

        let handler = self.handler.clone();

        let app = Router::new()
            .route("/", get(root))
            .route("/mcp", post(handle_mcp::<H>))
            .route("/health", get(health))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(handler);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Root handler.
async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "converge-mcp",
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
async fn handle_mcp<H: McpRequestHandler>(
    State(handler): State<Arc<H>>,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let response = handler.handle(request).await;
    Json(response)
}
