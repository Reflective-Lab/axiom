// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Qwen (Alibaba Cloud) API provider.

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use serde::{Deserialize, Serialize};

/// Qwen API provider (Alibaba Cloud `DashScope`).
///
/// # Example
///
/// ```ignore
/// use converge_provider::QwenProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = QwenProvider::new(
///     "your-api-key",
///     "qwen-turbo"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct QwenProvider {
    api_key: SecretString,
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl QwenProvider {
    /// Creates a new Qwen provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://dashscope.aliyuncs.com".into(),
        }
    }

    /// Creates a provider using the `QWEN_API_KEY` environment variable.
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
            .get_secret("QWEN_API_KEY")
            .map_err(|e| LlmError::auth(format!("QWEN_API_KEY: {e}")))?;
        Ok(Self {
            api_key,
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://dashscope.aliyuncs.com".into(),
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
struct QwenRequest {
    model: String,
    input: QwenInput,
    parameters: QwenParameters,
}

#[derive(Serialize)]
struct QwenInput {
    messages: Vec<QwenMessage>,
}

#[derive(Serialize)]
struct QwenMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct QwenParameters {
    max_tokens: u32,
    temperature: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop: Vec<String>,
}

#[derive(Deserialize)]
struct QwenResponse {
    output: QwenOutput,
    usage: QwenUsage,
    #[allow(dead_code)]
    request_id: String,
}

#[derive(Deserialize)]
struct QwenOutput {
    choices: Vec<QwenChoice>,
}

#[derive(Deserialize)]
struct QwenChoice {
    message: QwenChoiceMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct QwenChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)] // Fields match Qwen API JSON
struct QwenUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize)]
struct QwenError {
    code: Option<String>,
    message: String,
    #[allow(dead_code)]
    request_id: Option<String>,
}

impl LlmProvider for QwenProvider {
    fn name(&self) -> &'static str {
        "qwen"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!(
            "{}/api/v1/services/aigc/text-generation/generation",
            self.base_url
        );

        let mut messages = Vec::new();

        if let Some(ref system) = request.system {
            messages.push(QwenMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        messages.push(QwenMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        let body = QwenRequest {
            model: self.model.clone(),
            input: QwenInput { messages },
            parameters: QwenParameters {
                max_tokens: request.max_tokens,
                temperature: request.temperature,
                stop: request.stop_sequences.clone(),
            },
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
            let error_body: QwenError = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

            let code = error_body.code.as_deref().unwrap_or("unknown");
            return match code {
                "InvalidApiKey" | "InvalidParameter" => Err(LlmError::auth(error_body.message)),
                "Throttling" => Err(LlmError::rate_limit(error_body.message)),
                _ => Err(LlmError::provider(error_body.message)),
            };
        }

        let api_response: QwenResponse = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        let content = api_response
            .output
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let finish_reason = match api_response
            .output
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("length") => FinishReason::MaxTokens,
            _ => FinishReason::Stop, // "stop" or unknown
        };

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            usage: TokenUsage {
                prompt_tokens: api_response.usage.input_tokens,
                completion_tokens: api_response.usage.output_tokens,
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
        let provider = QwenProvider::new("test-key", "qwen-turbo");
        assert_eq!(provider.name(), "qwen");
        assert_eq!(provider.model(), "qwen-turbo");
    }
}
