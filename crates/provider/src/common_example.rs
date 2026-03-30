// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Example: Refactored provider using common abstractions.
//!
//! This shows how to implement a provider using the common abstractions
//! to reduce code duplication.
//!
//! # Benefits
//!
//! - **~50 lines** vs ~185 lines without abstractions
//! - **No manual HTTP client setup** - uses `HttpProviderConfig`
//! - **No manual request/response parsing** - uses `ChatCompletionRequest`/`ChatCompletionResponse`
//! - **Automatic error handling** - uses `handle_openai_style_error()` via `OpenAiCompatibleProvider`
//! - **Consistent behavior** - same patterns as all other OpenAI-compatible providers

use crate::common::{HttpProviderConfig, OpenAiCompatibleProvider};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// Example provider using common abstractions.
///
/// This demonstrates the fully refactored approach:
/// 1. Uses `HttpProviderConfig` for HTTP setup
/// 2. Implements `OpenAiCompatibleProvider` for default behavior
/// 3. Gets automatic error handling via `handle_openai_style_error()`
/// 4. Gets automatic request/response handling via `make_chat_completion_request()`
///
/// # Example
///
/// ```ignore
/// use converge_provider::ExampleProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = ExampleProvider::new("api-key", "model-name");
/// let response = provider.complete(&LlmRequest::new("Hello!"))?;
/// ```
pub struct ExampleProvider {
    config: HttpProviderConfig,
}

impl ExampleProvider {
    /// Creates a new provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(
                api_key,
                model,
                "https://api.example.com",
            ),
        }
    }

    /// Creates provider from environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if `EXAMPLE_API_KEY` is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("EXAMPLE_API_KEY")
            .map_err(|_| LlmError::auth("EXAMPLE_API_KEY not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for ExampleProvider {
    /// Gets the provider configuration.
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    /// Gets the API endpoint path.
    ///
    /// This is the only provider-specific detail you need to specify.
    fn endpoint(&self) -> &str {
        "/v1/chat/completions"
    }
}

impl LlmProvider for ExampleProvider {
    fn name(&self) -> &'static str {
        "example"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    /// Makes a completion request.
    ///
    /// This uses the default implementation from `OpenAiCompatibleProvider`,
    /// which automatically:
    /// - Builds the request using `ChatCompletionRequest`
    /// - Makes the HTTP call
    /// - Handles errors using `handle_openai_style_error()`
    /// - Parses the response using `chat_response_to_llm_response()`
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // One line! The trait handles everything:
        // - Request building
        // - HTTP call
        // - Error handling (automatic via handle_openai_style_error)
        // - Response parsing
        self.complete_openai_compatible(request)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("example:{}:{}", self.config.model, request_id)
    }
}

// ============================================================================
// Comparison: Before vs After Refactoring
// ============================================================================
//
// BEFORE (without abstractions): ~185 lines
// - Manual HTTP client setup (api_key, model, client, base_url)
// - Manual request structs (Request, Message, etc.)
// - Manual response structs (Response, Choice, Usage, etc.)
// - Manual error handling (~25 lines of parsing and mapping)
// - Manual response parsing (~20 lines)
// - Manual finish_reason mapping
//
// AFTER (with abstractions): ~80 lines
// - HttpProviderConfig handles HTTP setup
// - ChatCompletionRequest handles request building
// - ChatCompletionResponse handles response parsing
// - handle_openai_style_error() handles error parsing
// - OpenAiCompatibleProvider trait provides default implementation
//
// SAVINGS: ~105 lines (57% reduction)
//
// ============================================================================
// What You Get Automatically
// ============================================================================
//
// ✅ HTTP client setup and configuration
// ✅ Request building from LlmRequest
// ✅ Error handling with proper LlmError mapping
// ✅ Response parsing to LlmResponse
// ✅ Finish reason mapping
// ✅ Token usage extraction
//
// ============================================================================
// What You Still Need to Provide
// ============================================================================
//
// - Provider name (for `name()`)
// - Base URL (in `new()`)
// - Endpoint path (in `endpoint()`)
// - Environment variable name (in `from_env()`)
//
// That's it! Everything else is handled by the abstractions.

