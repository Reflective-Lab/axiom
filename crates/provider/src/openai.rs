// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! `OpenAI` GPT API provider.

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use serde::{Deserialize, Serialize};

/// `OpenAI` GPT API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::OpenAiProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = OpenAiProvider::new(
///     "your-api-key",
///     "gpt-4"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct OpenAiProvider {
    api_key: SecretString,
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl OpenAiProvider {
    /// Creates a new `OpenAI` provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://api.openai.com".into(),
        }
    }

    /// Creates a provider using the `OPENAI_API_KEY` environment variable.
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
            .get_secret("OPENAI_API_KEY")
            .map_err(|e| LlmError::auth(format!("OPENAI_API_KEY: {e}")))?;
        Ok(Self {
            api_key,
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://api.openai.com".into(),
        })
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<u32>,
    temperature: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

/// Returns true if a model requires `max_completion_tokens` instead of `max_tokens`.
/// This applies to OpenAI reasoning models (o1, o3) and newer GPT models (gpt-5.x).
fn uses_max_completion_tokens(model: &str) -> bool {
    model.starts_with("o1") || model.starts_with("o3") || model.starts_with("gpt-5")
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    #[allow(dead_code)]
    id: String,
    model: String,
    choices: Vec<Choice>,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)] // Fields match OpenAI API JSON
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAiError {
    error: OpenAiErrorDetail,
}

#[derive(Deserialize)]
struct OpenAiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let mut messages = Vec::new();

        if let Some(ref system) = request.system {
            messages.push(OpenAiMessage {
                role: "system",
                content: system,
            });
        }

        messages.push(OpenAiMessage {
            role: "user",
            content: &request.prompt,
        });

        let (max_tokens, max_completion_tokens) = if uses_max_completion_tokens(&self.model) {
            (None, Some(request.max_tokens))
        } else {
            (Some(request.max_tokens), None)
        };

        let body = OpenAiRequest {
            model: &self.model,
            messages,
            max_tokens,
            max_completion_tokens,
            temperature: request.temperature,
            stop: request.stop_sequences.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key.expose()))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            let http_code = status.as_u16();

            // HTTP 429 and 5xx are always retryable, regardless of error body
            if http_code == 429 || http_code >= 500 {
                let error_body: OpenAiError = response
                    .json()
                    .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;
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

            let error_body: OpenAiError = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

            let error_type = error_body.error.error_type.as_deref().unwrap_or("unknown");
            return match error_type {
                "authentication_error" => Err(LlmError::auth(error_body.error.message)),
                "invalid_request_error" => Err(LlmError::new(
                    crate::provider_api::LlmErrorKind::InvalidRequest,
                    error_body.error.message,
                    true, // retryable: a different provider may accept the same request
                )),
                "rate_limit_error" => Err(LlmError::rate_limit(error_body.error.message)),
                _ => Err(LlmError::provider(error_body.error.message)),
            };
        }

        let api_response: OpenAiResponse = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        let content = api_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let finish_reason = match api_response
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("length") => FinishReason::MaxTokens,
            _ => FinishReason::Stop,
        };

        Ok(LlmResponse {
            content,
            model: api_response.model,
            usage: TokenUsage {
                prompt_tokens: api_response.usage.prompt_tokens,
                completion_tokens: api_response.usage.completion_tokens,
                total_tokens: api_response.usage.total_tokens,
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
        let provider = OpenAiProvider::new("test-key", "gpt-4");
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4");
    }
}
