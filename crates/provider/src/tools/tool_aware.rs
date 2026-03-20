// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! Tool-aware LLM provider wrapper.

use super::{ToolCall, ToolDefinition, ToolError, ToolRegistry, ToolResult};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A provider that wraps an LLM provider and adds tool calling capabilities.
pub struct ToolAwareProvider {
    inner: Arc<dyn LlmProvider>,
    registry: ToolRegistry,
    format: ToolFormat,
}

impl std::fmt::Debug for ToolAwareProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolAwareProvider")
            .field("inner", &self.inner.name())
            .field("registry", &self.registry)
            .field("format", &self.format)
            .finish()
    }
}

/// Format for tool definitions.
#[derive(Debug, Clone, Copy, Default)]
pub enum ToolFormat {
    #[default]
    Anthropic,
    OpenAi,
    Generic,
}

impl ToolAwareProvider {
    pub fn new(provider: impl LlmProvider + 'static, registry: ToolRegistry) -> Self {
        Self {
            inner: Arc::new(provider),
            registry,
            format: ToolFormat::default(),
        }
    }

    pub fn with_shared(provider: Arc<dyn LlmProvider>, registry: ToolRegistry) -> Self {
        Self {
            inner: provider,
            registry,
            format: ToolFormat::default(),
        }
    }

    #[must_use]
    pub fn with_format(mut self, format: ToolFormat) -> Self {
        self.format = format;
        self
    }

    #[must_use]
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }

    pub fn register_tool(&mut self, tool: ToolDefinition) {
        self.registry.register(tool);
    }

    pub fn complete_with_tools(&self, request: &LlmRequest) -> Result<ToolAwareResponse, LlmError> {
        let enhanced_request = self.enhance_request(request);
        let response = self.inner.complete(&enhanced_request)?;
        let tool_calls = self.parse_tool_calls(&response);
        Ok(ToolAwareResponse {
            response,
            tool_calls,
        })
    }

    pub(crate) fn enhance_request(&self, request: &LlmRequest) -> LlmRequest {
        if self.registry.is_empty() {
            return request.clone();
        }

        let tools_json = match self.format {
            ToolFormat::Anthropic => serde_json::to_string(&self.registry.to_anthropic_tools()),
            ToolFormat::OpenAi | ToolFormat::Generic => {
                serde_json::to_string(&self.registry.to_llm_tools())
            }
        };

        let tools_section = match tools_json {
            Ok(json) => format!("\n\n<tools>\n{json}\n</tools>"),
            Err(_) => String::new(),
        };

        let enhanced_system = match &request.system {
            Some(system) => Some(format!(
                "{system}\n\nYou have access to tools. To use a tool:\n<tool_use>\n{{\"name\": \"tool_name\", \"arguments\": {{}}}}\n</tool_use>{tools_section}"
            )),
            None => Some(format!(
                "You have access to tools. To use a tool:\n<tool_use>\n{{\"name\": \"tool_name\", \"arguments\": {{}}}}\n</tool_use>{tools_section}"
            )),
        };

        LlmRequest {
            prompt: request.prompt.clone(),
            system: enhanced_system,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop_sequences: request.stop_sequences.clone(),
        }
    }

    fn parse_tool_calls(&self, response: &LlmResponse) -> Vec<ParsedToolCall> {
        let mut calls = Vec::new();
        let content = &response.content;

        let mut search_start = 0;
        while let Some(start) = content[search_start..].find("<tool_use>") {
            let abs_start = search_start + start + "<tool_use>".len();
            if let Some(end) = content[abs_start..].find("</tool_use>") {
                let tool_json = content[abs_start..abs_start + end].trim();
                if let Ok(call) = serde_json::from_str::<ToolCallJson>(tool_json) {
                    calls.push(ParsedToolCall {
                        call: ToolCall::new(&call.name, call.arguments),
                        raw: tool_json.to_string(),
                    });
                }
                search_start = abs_start + end + "</tool_use>".len();
            } else {
                break;
            }
        }

        calls
    }

    pub fn execute_tool(&self, call: &ToolCall) -> Result<ToolResult, ToolError> {
        self.registry.call_tool(call)
    }

    #[must_use]
    pub fn format_tool_result(&self, result: &ToolResult) -> String {
        if result.is_error {
            format!(
                "<tool_result call_id=\"{}\" error=\"true\">\n{}\n</tool_result>",
                result.call_id,
                result.as_text().unwrap_or("Error")
            )
        } else {
            format!(
                "<tool_result call_id=\"{}\">\n{}\n</tool_result>",
                result.call_id,
                result.as_text().unwrap_or("")
            )
        }
    }
}

/// Response from a tool-aware completion.
#[derive(Debug)]
pub struct ToolAwareResponse {
    pub response: LlmResponse,
    pub tool_calls: Vec<ParsedToolCall>,
}

impl ToolAwareResponse {
    #[must_use]
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// A parsed tool call from an LLM response.
#[derive(Debug)]
pub struct ParsedToolCall {
    pub call: ToolCall,
    pub raw: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ToolCallJson {
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeProvider;
    use crate::tools::InputSchema;

    fn create_test_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        registry.register(ToolDefinition::new(
            "get_weather",
            "Get weather",
            InputSchema::empty(),
        ));
        registry
    }

    #[test]
    fn test_tool_aware_creation() {
        let provider = FakeProvider::new();
        let registry = create_test_registry();
        let tool_provider = ToolAwareProvider::new(provider, registry);
        assert_eq!(tool_provider.registry().len(), 1);
    }

    #[test]
    fn test_enhance_request() {
        let provider = FakeProvider::new();
        let registry = create_test_registry();
        let tool_provider = ToolAwareProvider::new(provider, registry);

        let request = LlmRequest::new("Hello");
        let enhanced = tool_provider.enhance_request(&request);
        assert!(enhanced.system.is_some());
        assert!(enhanced.system.unwrap().contains("get_weather"));
    }

    #[test]
    fn test_parse_tool_calls() {
        let provider = FakeProvider::new();
        let registry = create_test_registry();
        let tool_provider = ToolAwareProvider::new(provider, registry);

        let response = LlmResponse {
            content: r#"<tool_use>{"name": "test", "arguments": {}}</tool_use>"#.to_string(),
            model: "test".to_string(),
            finish_reason: crate::provider_api::FinishReason::Stop,
            usage: crate::provider_api::TokenUsage::default(),
        };

        let calls = tool_provider.parse_tool_calls(&response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].call.tool_name, "test");
    }

    #[test]
    fn test_complete_with_tools() {
        let provider = FakeProvider::new();
        let registry = create_test_registry();
        let tool_provider = ToolAwareProvider::new(provider, registry);

        let request = LlmRequest::new("What's the weather?");
        let result = tool_provider.complete_with_tools(&request);
        assert!(result.is_ok());
    }
}
