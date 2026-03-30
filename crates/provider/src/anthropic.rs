// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Anthropic Claude API provider.

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use serde::{Deserialize, Serialize};

/// Anthropic Claude API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::AnthropicProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = AnthropicProvider::new(
///     "your-api-key",
///     "claude-sonnet-4-6"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct AnthropicProvider {
    api_key: SecretString,
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl AnthropicProvider {
    /// Creates a new Anthropic provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://api.anthropic.com".into(),
        }
    }

    /// Creates a provider using the `ANTHROPIC_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        Self::from_secret_provider(&EnvSecretProvider, model)
    }

    /// Creates a provider by loading the API key from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns error if the secret cannot be loaded.
    pub fn from_secret_provider(
        secrets: &dyn SecretProvider,
        model: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let api_key = secrets
            .get_secret("ANTHROPIC_API_KEY")
            .map_err(|e| LlmError::auth(format!("ANTHROPIC_API_KEY: {e}")))?;
        Ok(Self {
            api_key,
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://api.anthropic.com".into(),
        })
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

fn slice_is_empty(s: &&[String]) -> bool {
    s.is_empty()
}

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    stop_sequences: &'a [String],
    temperature: f64,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    model: String,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        let body = AnthropicRequest {
            model: &self.model,
            max_tokens: request.max_tokens,
            messages: vec![Message {
                role: "user",
                content: &request.prompt,
            }],
            system: request.system.as_deref(),
            stop_sequences: &request.stop_sequences,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            let error_body: AnthropicError = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

            return match error_body.error.error_type.as_str() {
                "authentication_error" => Err(LlmError::auth(error_body.error.message)),
                "rate_limit_error" => Err(LlmError::rate_limit(error_body.error.message)),
                _ => Err(LlmError::provider(error_body.error.message)),
            };
        }

        let api_response: AnthropicResponse = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        let content = api_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<String>();

        let finish_reason = match api_response.stop_reason.as_deref() {
            Some("max_tokens") => FinishReason::MaxTokens,
            _ => FinishReason::Stop,
        };

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: TokenUsage {
                prompt_tokens: api_response.usage.input_tokens,
                completion_tokens: api_response.usage.output_tokens,
                total_tokens: api_response.usage.input_tokens + api_response.usage.output_tokens,
            },
            finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_has_correct_name() {
        let provider = AnthropicProvider::new("test-key", "claude-3");
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-3");
    }

    // Note: Testing from_env() requires manipulating environment variables,
    // which is unsafe in Rust 2024 (can cause data races). The from_env()
    // function itself is tested implicitly when AnthropicProvider is used
    // in integration tests with real API keys.
}
