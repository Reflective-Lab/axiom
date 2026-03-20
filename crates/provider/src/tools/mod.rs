// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Tool abstractions for MCP, `OpenAPI`, and GraphQL integration.
//!
//! This module provides a unified interface for tool discovery, definition,
//! and execution across multiple sources:
//!
//! - **MCP (Model Context Protocol)**: Connect to MCP servers via stdio/HTTP
//! - **`OpenAPI`**: Convert `OpenAPI` specs to tool definitions
//! - **GraphQL**: Introspect GraphQL schemas for tool discovery
//!
//! # Core Types
//!
//! - [`ToolDefinition`]: Describes a tool's interface (name, schema, source)
//! - [`ToolCall`]: A request to invoke a tool
//! - [`ToolResult`]: The outcome of a tool invocation
//! - [`ToolSource`]: Where the tool came from (MCP, `OpenAPI`, GraphQL, inline)

mod definition;
mod error;
mod registry;

// Integration modules
pub mod config;
pub mod graphql;
pub mod mcp;
pub mod openapi;
pub mod tool_aware;

// Re-exports
pub use definition::{
    GraphQlOperationType, InputSchema, ToolCall, ToolDefinition, ToolResult, ToolResultContent,
    ToolSource,
};
pub use error::{ToolError, ToolErrorKind};
pub use registry::{SourceFilter, ToolHandler, ToolRegistry};

// Convenience re-exports from submodules
pub use config::{
    GraphQlConfig, InlineToolConfig, McpServerConfig, McpTransportType, OpenApiConfig, ToolsConfig,
    ToolsConfigError, build_registry_from_config, load_tools_config, parse_tools_config,
};
pub use graphql::GraphQlConverter;
pub use mcp::{McpClient, McpClientBuilder, McpTransport};
pub use openapi::OpenApiConverter;
pub use tool_aware::{ParsedToolCall, ToolAwareProvider, ToolAwareResponse, ToolFormat};
