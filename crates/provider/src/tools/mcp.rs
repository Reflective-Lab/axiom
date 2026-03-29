// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! MCP (Model Context Protocol) client integration.
//!
//! Re-exports transport types from `converge-mcp` and provides
//! Converge-specific integration (tool registration, call/result mapping).

use super::{InputSchema, ToolCall, ToolDefinition, ToolError, ToolResult, ToolSource};
use serde::Deserialize;

// Re-export transport types from converge-mcp
pub use converge_mcp::client::{McpClientBuilder, McpClientError, McpTransport};
pub use converge_mcp::MCP_PROTOCOL_VERSION;

/// MCP server information.
pub use converge_mcp::client::McpServerInfo;

/// MCP client with Converge tool integration.
///
/// Wraps [`converge_mcp::client::McpClient`] and adds integration
/// with Converge's [`ToolDefinition`] / [`ToolCall`] / [`ToolResult`] types.
#[derive(Debug)]
pub struct McpClient {
    inner: converge_mcp::client::McpClient,
    cached_tools: Vec<ToolDefinition>,
}

impl McpClient {
    #[must_use]
    pub fn new(name: impl Into<String>, transport: McpTransport) -> Self {
        Self {
            inner: converge_mcp::client::McpClient::new(name, transport),
            cached_tools: Vec::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    #[must_use]
    pub fn transport(&self) -> &McpTransport {
        self.inner.transport()
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    pub fn connect(&mut self) -> Result<&McpServerInfo, ToolError> {
        self.inner
            .connect()
            .map_err(|e| ToolError::connection_failed(e.to_string()))
    }

    pub fn disconnect(&mut self) {
        self.inner.disconnect();
        self.cached_tools.clear();
    }

    pub fn list_tools(&mut self) -> Result<&[ToolDefinition], ToolError> {
        if !self.inner.is_connected() {
            return Err(ToolError::connection_failed("Not connected"));
        }
        Ok(&self.cached_tools)
    }

    pub fn call_tool(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        if !self.inner.is_connected() {
            return Err(ToolError::connection_failed("Not connected"));
        }
        if !self.cached_tools.iter().any(|t| t.name == call.tool_name) {
            return Err(ToolError::not_found(&call.tool_name));
        }
        Ok(ToolResult::text(
            &call.call_id,
            format!("MCP tool '{}' called (stub)", call.tool_name),
        ))
    }

    pub fn register_tool(&mut self, tool: ToolDefinition) {
        let tool = tool.with_source(ToolSource::Mcp {
            server_name: self.inner.name().to_string(),
            server_uri: self.inner.transport().to_uri(),
        });
        self.cached_tools.push(tool);
    }

    #[must_use]
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.cached_tools
    }
}

/// MCP tool definition from server — converts to Converge [`ToolDefinition`].
#[derive(Debug, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

impl McpToolDefinition {
    #[must_use]
    pub fn to_tool_definition(self, server_name: &str, server_uri: &str) -> ToolDefinition {
        ToolDefinition::new(
            self.name,
            self.description.unwrap_or_default(),
            self.input_schema
                .map(InputSchema::from_json_schema)
                .unwrap_or_default(),
        )
        .with_source(ToolSource::Mcp {
            server_name: server_name.to_string(),
            server_uri: server_uri.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transport_stdio() {
        let transport = McpTransport::stdio("npx", &["-y", "mcp-server"]);
        assert!(transport.to_uri().starts_with("stdio:npx"));
    }

    #[test]
    fn test_client_connect() {
        let mut client = McpClient::new("test", McpTransport::stdio("echo", &[]));
        let info = client.connect().unwrap();
        assert_eq!(info.name, "test");
        assert!(client.is_connected());
    }

    #[test]
    fn test_register_and_call() {
        let mut client = McpClient::new("test", McpTransport::stdio("echo", &[]));
        client.connect().unwrap();
        client.register_tool(ToolDefinition::new(
            "test_tool",
            "Test",
            InputSchema::empty(),
        ));

        let call = ToolCall::new("test_tool", json!({}));
        let result = client.call_tool(&call).unwrap();
        assert!(!result.is_error);
    }
}
