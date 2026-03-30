// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Perplexity AI API provider.

use crate::common::{
    ChatCompletionRequest, ChatCompletionResponse, HttpProviderConfig, OpenAiCompatibleProvider,
    chat_response_to_llm_response,
};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// Perplexity AI API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::PerplexityProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = PerplexityProvider::new(
///     "your-api-key",
///     "pplx-70b-online"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct PerplexityProvider {
    config: HttpProviderConfig,
}

impl PerplexityProvider {
    /// Creates a new Perplexity provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://api.perplexity.ai"),
        }
    }

    /// Creates a provider using the `PERPLEXITY_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("PERPLEXITY_API_KEY")
            .map_err(|_| LlmError::auth("PERPLEXITY_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for PerplexityProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/chat/completions"
    }
}

impl LlmProvider for PerplexityProvider {
    fn name(&self) -> &'static str {
        "perplexity"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // Perplexity has custom error handling and doesn't support stop sequences
        let mut chat_request =
            ChatCompletionRequest::from_llm_request(self.config.model.clone(), request);
        // Perplexity doesn't support stop sequences
        chat_request.stop = Vec::new();

        let url = format!("{}{}", self.config.base_url, self.endpoint());

        let http_response = self
            .config
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.expose()),
            )
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        let status = http_response.status();

        if !status.is_success() {
            return crate::common::handle_openai_style_error(http_response);
        }

        let api_response: ChatCompletionResponse = http_response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        chat_response_to_llm_response(api_response)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("perplexity:{}:{}", self.config.model, request_id)
    }
}
