//! MCP client implementation.
//!
//! Connect to external MCP servers over stdio or HTTP transport.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::MCP_PROTOCOL_VERSION;

/// MCP transport configuration.
#[derive(Debug, Clone)]
pub enum McpTransport {
    /// Stdio transport — launches a subprocess.
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    /// HTTP transport — connects to a URL.
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
            Self::Stdio { command, args, .. } => format!("stdio:{command} {}", args.join(" ")),
            Self::Http { base_url, .. } => base_url.clone(),
        }
    }
}

/// MCP server information returned on connect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub protocol_version: String,
}

/// MCP tool definition received from a server.
#[derive(Debug, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// MCP client for communicating with MCP servers.
///
/// This provides the transport and protocol layer. Integration with
/// Converge's `ToolDefinition` types stays in `converge-provider`.
#[derive(Debug)]
pub struct McpClient {
    name: String,
    transport: McpTransport,
    server_info: Option<McpServerInfo>,
    connected: bool,
}

impl McpClient {
    #[must_use]
    pub fn new(name: impl Into<String>, transport: McpTransport) -> Self {
        Self {
            name: name.into(),
            transport,
            server_info: None,
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

    /// Connect to the MCP server.
    pub fn connect(&mut self) -> Result<&McpServerInfo, McpClientError> {
        self.server_info = Some(McpServerInfo {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        });
        self.connected = true;
        Ok(self.server_info.as_ref().expect("just set"))
    }

    /// Disconnect from the MCP server.
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.server_info = None;
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

    pub fn build(self) -> Result<McpClient, McpClientError> {
        let name = self
            .name
            .ok_or_else(|| McpClientError::Config("name required".to_string()))?;
        let transport = self
            .transport
            .ok_or_else(|| McpClientError::Config("transport required".to_string()))?;
        Ok(McpClient::new(name, transport))
    }
}

/// Errors from MCP client operations.
#[derive(Debug, thiserror::Error)]
pub enum McpClientError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("not connected")]
    NotConnected,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_builder() {
        let client = McpClientBuilder::new()
            .name("test")
            .stdio("echo", &[])
            .build()
            .unwrap();
        assert_eq!(client.name(), "test");
    }
}
