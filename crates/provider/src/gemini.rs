// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Google Gemini API provider.

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use serde::{Deserialize, Serialize};

/// Google Gemini API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::GeminiProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = GeminiProvider::new(
///     "your-api-key",
///     "gemini-pro"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct GeminiProvider {
    api_key: SecretString,
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
}

impl GeminiProvider {
    /// Creates a new Gemini provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://generativelanguage.googleapis.com".into(),
        }
    }

    /// Creates a provider using the `GEMINI_API_KEY` environment variable.
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
            .get_secret("GEMINI_API_KEY")
            .map_err(|e| LlmError::auth(format!("GEMINI_API_KEY: {e}")))?;
        Ok(Self {
            api_key,
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://generativelanguage.googleapis.com".into(),
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
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    max_output_tokens: u32,
    temperature: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stop_sequences: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiCandidateContent,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiCandidatePart>,
}

#[derive(Deserialize)]
struct GeminiCandidatePart {
    text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[derive(Deserialize)]
struct GeminiError {
    error: GeminiErrorDetail,
}

#[derive(Deserialize)]
struct GeminiErrorDetail {
    message: String,
    status: Option<String>,
}

impl LlmProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url,
            self.model,
            self.api_key.expose()
        );

        let system_instruction = request.system.as_ref().map(|s| GeminiContent {
            parts: vec![GeminiPart { text: s.clone() }],
        });

        let contents = vec![GeminiContent {
            parts: vec![GeminiPart {
                text: request.prompt.clone(),
            }],
        }];

        let body = GeminiRequest {
            system_instruction,
            contents,
            generation_config: GeminiGenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
                stop_sequences: request.stop_sequences.clone(),
            },
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            // Check HTTP status first — server errors and rate limits are retryable
            // regardless of what the error body says.
            let http_code = status.as_u16();
            if http_code == 429 || http_code >= 500 {
                let error_body: GeminiError = response
                    .json()
                    .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;
                return Err(LlmError::new(
                    if http_code == 429 {
                        crate::provider_api::LlmErrorKind::RateLimit
                    } else {
                        crate::provider_api::LlmErrorKind::ProviderError
                    },
                    error_body.error.message,
                    true, // server errors are always retryable
                ));
            }

            let error_body: GeminiError = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

            let status_str = error_body.error.status.as_deref().unwrap_or("unknown");
            return match status_str {
                "UNAUTHENTICATED" | "PERMISSION_DENIED" => {
                    Err(LlmError::auth(error_body.error.message))
                }
                "RESOURCE_EXHAUSTED" => Err(LlmError::rate_limit(error_body.error.message)),
                _ => Err(LlmError::provider(error_body.error.message)),
            };
        }

        let api_response: GeminiResponse = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

        let content = api_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let finish_reason = match api_response
            .candidates
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("MAX_TOKENS") => FinishReason::MaxTokens,
            _ => FinishReason::Stop,
        };

        let usage = api_response.usage_metadata.map_or_else(
            || TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            |u| TokenUsage {
                prompt_tokens: u.prompt_token_count,
                completion_tokens: u.candidates_token_count,
                total_tokens: u.total_token_count,
            },
        );

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            usage,
            finish_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_has_correct_name() {
        let provider = GeminiProvider::new("test-key", "gemini-pro");
        assert_eq!(provider.name(), "gemini");
        assert_eq!(provider.model(), "gemini-pro");
    }
}
