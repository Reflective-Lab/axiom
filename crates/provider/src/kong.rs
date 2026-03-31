// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Kong AI Gateway provider.
//!
//! Routes LLM calls through a [Kong AI Gateway](https://konghq.com/products/kong-ai-gateway)
//! instance. Kong proxies to the configured upstream model (OpenAI, Anthropic, etc.)
//! and adds rate limiting, PII detection, token tracking, and cost governance.
//!
//! The provider uses the OpenAI-compatible chat completions API that Kong exposes,
//! regardless of which upstream model is configured behind the route.

use crate::common::{
    ChatCompletionRequest, ChatMessage, chat_response_to_llm_response,
    handle_openai_style_error,
};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};

/// Kong AI Gateway provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::KongProvider;
/// use converge_provider::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = KongProvider::new(
///     "https://kong.example.com",
///     "your-team-api-key",
///     "default",
/// );
///
/// let response = provider.complete(&LlmRequest::new("What is 2+2?"))?;
/// println!("{}", response.content);
/// ```
pub struct KongProvider {
    gateway_url: String,
    api_key: SecretString,
    route: String,
    client: reqwest::blocking::Client,
}

impl KongProvider {
    /// Creates a new Kong provider.
    ///
    /// - `gateway_url`: The Kong AI Gateway base URL (e.g., `https://kong.example.com`)
    /// - `api_key`: Team API key for Kong authentication
    /// - `route`: The Kong AI Gateway route name (determines which upstream model is used)
    #[must_use]
    pub fn new(
        gateway_url: impl Into<String>,
        api_key: impl Into<String>,
        route: impl Into<String>,
    ) -> Self {
        Self {
            gateway_url: gateway_url.into(),
            api_key: SecretString::new(api_key),
            route: route.into(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Creates a provider from environment variables.
    ///
    /// Reads:
    /// - `KONG_AI_GATEWAY_URL` — gateway base URL
    /// - `KONG_API_KEY` — team API key
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set.
    pub fn from_env(route: impl Into<String>) -> Result<Self, LlmError> {
        Self::from_secret_provider(&EnvSecretProvider, route)
    }

    /// Creates a provider by loading secrets from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns error if secrets cannot be loaded.
    pub fn from_secret_provider(
        secrets: &dyn SecretProvider,
        route: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let gateway_url = secrets
            .get_secret("KONG_AI_GATEWAY_URL")
            .map_err(|e| LlmError::auth(format!("KONG_AI_GATEWAY_URL: {e}")))?;
        let api_key = secrets
            .get_secret("KONG_API_KEY")
            .map_err(|e| LlmError::auth(format!("KONG_API_KEY: {e}")))?;
        Ok(Self {
            gateway_url: gateway_url.expose().to_string(),
            api_key,
            route: route.into(),
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Uses a custom HTTP client.
    #[must_use]
    pub fn with_client(mut self, client: reqwest::blocking::Client) -> Self {
        self.client = client;
        self
    }
}

impl LlmProvider for KongProvider {
    fn name(&self) -> &'static str {
        "kong"
    }

    fn model(&self) -> &str {
        &self.route
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/ai/chat/completions", self.gateway_url);

        let mut messages = Vec::new();
        if let Some(ref system) = request.system {
            messages.push(ChatMessage {
                role: "system".into(),
                content: system.clone(),
            });
        }
        messages.push(ChatMessage {
            role: "user".into(),
            content: request.prompt.clone(),
        });

        let body = ChatCompletionRequest {
            model: self.route.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_sequences.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Kong request failed: {e}")))?;

        if !response.status().is_success() {
            return handle_openai_style_error(response);
        }

        let chat_response = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse Kong response: {e}")))?;

        chat_response_to_llm_response(chat_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kong_provider_name_and_model() {
        let provider = KongProvider::new("https://kong.example.com", "test-key", "gpt-4-route");
        assert_eq!(provider.name(), "kong");
        assert_eq!(provider.model(), "gpt-4-route");
    }
}
