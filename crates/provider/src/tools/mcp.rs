// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! MCP (Model Context Protocol) client implementation.

use super::{InputSchema, ToolCall, ToolDefinition, ToolError, ToolResult, ToolSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP protocol version.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// MCP transport configuration.
#[derive(Debug, Clone)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    Http {
        base_url: String,
        auth_header: Option<String>,
    },
}

impl McpTransport {
    #[must_use]
    pub fn stdio(command: impl Into<String>, args: &[&str]) -> Self {
        Self::Stdio {
            command: command.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            env: HashMap::new(),
        }
    }

    #[must_use]
    pub fn stdio_with_env(
        command: impl Into<String>,
        args: &[&str],
        env: HashMap<String, String>,
    ) -> Self {
        Self::Stdio {
            command: command.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            env,
        }
    }

    #[must_use]
    pub fn http(base_url: impl Into<String>) -> Self {
        Self::Http {
            base_url: base_url.into(),
            auth_header: None,
        }
    }

    #[must_use]
    pub fn http_with_auth(base_url: impl Into<String>, auth_header: impl Into<String>) -> Self {
        Self::Http {
            base_url: base_url.into(),
            auth_header: Some(auth_header.into()),
        }
    }

    #[must_use]
    pub fn to_uri(&self) -> String {
        match self {
            Self::Stdio { command, args, .. } => format!("stdio:{} {}", command, args.join(" ")),
            Self::Http { base_url, .. } => base_url.clone(),
        }
    }
}

/// MCP server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub protocol_version: String,
}

/// MCP client for communicating with MCP servers.
#[derive(Debug)]
pub struct McpClient {
    name: String,
    transport: McpTransport,
    server_info: Option<McpServerInfo>,
    cached_tools: Vec<ToolDefinition>,
    connected: bool,
}

impl McpClient {
    #[must_use]
    pub fn new(name: impl Into<String>, transport: McpTransport) -> Self {
        Self {
            name: name.into(),
            transport,
            server_info: None,
            cached_tools: Vec::new(),
            connected: false,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn transport(&self) -> &McpTransport {
        &self.transport
    }

    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn connect(&mut self) -> Result<&McpServerInfo, ToolError> {
        // Stub implementation - real impl needs async runtime
        self.server_info = Some(McpServerInfo {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        });
        self.connected = true;
        Ok(self.server_info.as_ref().unwrap())
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        self.server_info = None;
        self.cached_tools.clear();
    }

    pub fn list_tools(&mut self) -> Result<&[ToolDefinition], ToolError> {
        if !self.connected {
            return Err(ToolError::connection_failed("Not connected"));
        }
        Ok(&self.cached_tools)
    }

    pub fn call_tool(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        if !self.connected {
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
            server_name: self.name.clone(),
            server_uri: self.transport.to_uri(),
        });
        self.cached_tools.push(tool);
    }

    #[must_use]
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.cached_tools
    }
}

/// Builder for MCP clients.
#[derive(Debug, Default)]
pub struct McpClientBuilder {
    name: Option<String>,
    transport: Option<McpTransport>,
}

impl McpClientBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn stdio(mut self, command: impl Into<String>, args: &[&str]) -> Self {
        self.transport = Some(McpTransport::stdio(command, args));
        self
    }

    #[must_use]
    pub fn http(mut self, base_url: impl Into<String>) -> Self {
        self.transport = Some(McpTransport::http(base_url));
        self
    }

    pub fn build(self) -> Result<McpClient, ToolError> {
        let name = self
            .name
            .ok_or_else(|| ToolError::invalid_arguments("name required"))?;
        let transport = self
            .transport
            .ok_or_else(|| ToolError::invalid_arguments("transport required"))?;
        Ok(McpClient::new(name, transport))
    }
}

/// MCP tool definition from server.
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
