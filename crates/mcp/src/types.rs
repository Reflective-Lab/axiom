//! MCP protocol types.
//!
//! JSON-RPC request/response types and MCP-specific structures
//! shared by both client and server implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP protocol version.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// MCP JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version string (always `"2.0"`).
    pub jsonrpc: String,
    /// Request identifier; `None` for notifications.
    pub id: Option<serde_json::Value>,
    /// Method name to invoke.
    pub method: String,
    /// Optional method parameters.
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// MCP JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version string (always `"2.0"`).
    pub jsonrpc: String,
    /// Mirrors the `id` from the originating request.
    pub id: Option<serde_json::Value>,
    /// Successful result payload; mutually exclusive with `error`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error payload; mutually exclusive with `result`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Build a successful response wrapping `result`.
    pub fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response with the given `code` and `message`.
    pub fn error(id: Option<serde_json::Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

/// MCP JSON-RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code as defined by JSON-RPC / MCP.
    pub code: i32,
    /// Human-readable error description.
    pub message: String,
    /// Optional additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP server capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability advertised by the server.
    pub tools: Option<ToolsCapability>,
    /// Resources capability advertised by the server.
    pub resources: Option<ResourcesCapability>,
    /// Prompts capability advertised by the server.
    pub prompts: Option<PromptsCapability>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            tools: Some(ToolsCapability {}),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
            }),
            prompts: None,
        }
    }
}

/// Marker type indicating the server exposes tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {}

/// Marker type indicating the server exposes resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether the server supports resource-change subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
}

/// Marker type indicating the server exposes prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {}

/// MCP server info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Human-readable server name.
    pub name: String,
    /// Server version string.
    pub version: String,
}

/// MCP initialize result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// MCP protocol version in use.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Capabilities the server supports.
    pub capabilities: ServerCapabilities,
    /// Basic information about this server.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

/// MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique tool identifier.
    pub name: String,
    /// Human-readable description of the tool.
    pub description: String,
    /// JSON Schema describing the tool's input parameters.
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// MCP tools list result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// All tools available on this server.
    pub tools: Vec<Tool>,
}

/// MCP tool call request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments to pass to the tool.
    #[serde(default)]
    pub arguments: HashMap<String, serde_json::Value>,
}

/// MCP tool call result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content items returned by the tool.
    pub content: Vec<ToolContent>,
    /// Set to `true` when the tool call produced an error.
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// MCP tool content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    /// Plain-text content.
    #[serde(rename = "text")]
    Text {
        /// The text payload.
        text: String,
    },
    /// Base64-encoded image.
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type of the image (e.g. `"image/png"`).
        mime_type: String,
    },
    /// Embedded resource reference.
    #[serde(rename = "resource")]
    Resource {
        /// The referenced resource content.
        resource: ResourceContent,
    },
}

/// MCP resource content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// URI identifying the resource.
    pub uri: String,
    /// MIME type of the content, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content of the resource, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// MCP resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// URI that uniquely identifies this resource.
    pub uri: String,
    /// Human-readable resource name.
    pub name: String,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP resources list result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// All resources available on this server.
    pub resources: Vec<Resource>,
}

/// MCP read resource request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceRequest {
    /// URI of the resource to read.
    pub uri: String,
}

/// MCP read resource result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Content items that make up the resource.
    pub contents: Vec<ResourceContent>,
}
