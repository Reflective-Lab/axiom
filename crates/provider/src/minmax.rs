// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! `MinMax` AI API provider.

use crate::common::{
    ChatCompletionRequest, ChatCompletionResponse, HttpProviderConfig, OpenAiCompatibleProvider,
    chat_response_to_llm_response,
};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// `MinMax` AI API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::MinMaxProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = MinMaxProvider::new(
///     "your-api-key",
///     "abab5.5-chat"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct MinMaxProvider {
    config: HttpProviderConfig,
}

impl MinMaxProvider {
    /// Creates a new `MinMax` provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://api.minmax.ai"),
        }
    }

    /// Creates a provider using the `MINMAX_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("MINMAX_API_KEY")
            .map_err(|_| LlmError::auth("MINMAX_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for MinMaxProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/v1/chat/completions"
    }
}

impl LlmProvider for MinMaxProvider {
    fn name(&self) -> &'static str {
        "minmax"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // MinMax has custom error handling, so we implement it manually
        let chat_request =
            ChatCompletionRequest::from_llm_request(self.config.model.clone(), request);
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
        format!("minmax:{}:{}", self.config.model, request_id)
    }
}
