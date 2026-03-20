// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! `OpenRouter` API provider (multi-provider aggregator).

use crate::common::{
    ChatCompletionRequest, ChatCompletionResponse, HttpProviderConfig, OpenAiCompatibleProvider,
    chat_response_to_llm_response,
};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use serde::Deserialize;

/// `OpenRouter` API provider.
///
/// `OpenRouter` provides access to multiple LLM providers through a single API.
/// Model names use the format: `provider/model-name` (e.g., `anthropic/claude-3-opus`)
///
/// # Example
///
/// ```ignore
/// use converge_provider::OpenRouterProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = OpenRouterProvider::new(
///     "your-api-key",
///     "anthropic/claude-3-opus"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct OpenRouterProvider {
    config: HttpProviderConfig,
}

impl OpenRouterProvider {
    /// Creates a new `OpenRouter` provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://openrouter.ai/api/v1"),
        }
    }

    /// Creates a provider using the `OPENROUTER_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| LlmError::auth("OPENROUTER_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for OpenRouterProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/chat/completions"
    }
}

impl LlmProvider for OpenRouterProvider {
    fn name(&self) -> &'static str {
        "openrouter"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // OpenRouter has custom error handling and optional headers
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
            .header("HTTP-Referer", "https://github.com/converge-hey-sh") // Optional: for analytics
            .header("X-Title", "Converge") // Optional: for analytics
            .json(&chat_request)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        let status = http_response.status();

        if !status.is_success() {
            #[derive(Deserialize)]
            struct OpenRouterError {
                error: OpenRouterErrorDetail,
            }
            #[derive(Deserialize)]
            struct OpenRouterErrorDetail {
                message: String,
                #[serde(rename = "type")]
                error_type: Option<String>,
            }

            let error_body: OpenRouterError = http_response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

            let http_code = status.as_u16();
            // HTTP 429 and 5xx are always retryable
            if http_code == 429 || http_code >= 500 {
                return Err(LlmError::new(
                    if http_code == 429 {
                        crate::provider_api::LlmErrorKind::RateLimit
                    } else {
                        crate::provider_api::LlmErrorKind::ProviderError
                    },
                    error_body.error.message,
                    true,
                ));
            }
            let error_type = error_body.error.error_type.as_deref().unwrap_or("unknown");
            return match error_type {
                "authentication_error" => Err(LlmError::auth(error_body.error.message)),
                "invalid_request_error" => Err(LlmError::new(
                    crate::provider_api::LlmErrorKind::InvalidRequest,
                    error_body.error.message,
                    true,
                )),
                "rate_limit_error" => Err(LlmError::rate_limit(error_body.error.message)),
                _ => Err(LlmError::provider(error_body.error.message)),
            };
        }

        let api_response: ChatCompletionResponse = http_response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        chat_response_to_llm_response(api_response)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("openrouter:{}:{}", self.config.model, request_id)
    }
}
