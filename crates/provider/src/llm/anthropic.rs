// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Anthropic Claude Backend — implements `ChatBackend` for Claude models.
//!
//! # Usage
//!
//! ```ignore
//! use converge_provider::llm::AnthropicBackend;
//! use converge_core::traits::{ChatBackend, ChatRequest, ChatMessage, ChatRole};
//!
//! let backend = AnthropicBackend::new("your-api-key")
//!     .with_model("claude-sonnet-4-6");
//!
//! let req = ChatRequest { messages: vec![ChatMessage { role: ChatRole::User, content: "Hi".into() }], ..Default::default() };
//! let response = backend.chat(req).await?;
//! ```

use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use converge_core::backend::{BackendError, BackendResult};
use converge_core::traits::{
    ChatBackend, ChatRequest, ChatResponse, ChatRole, FinishReason as ChatFinishReason,
    LlmError as ChatLlmError, TokenUsage as ChatTokenUsage,
};

// ============================================================================
// AnthropicBackend
// ============================================================================

/// Anthropic Claude backend implementing the unified `LlmBackend` interface.
///
/// Produces `RemoteTraceLink` for audit purposes. Not replay-eligible.
pub struct AnthropicBackend {
    api_key: SecretString,
    model: String,
    base_url: String,
    client: Client,
    temperature: f32,
    top_p: f32,
    max_retries: usize,
}

impl AnthropicBackend {
    /// Create a new Anthropic backend with the given API key.
    ///
    /// Uses claude-sonnet-4-6 by default.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: "claude-sonnet-4-6".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            client: Client::new(),
            temperature: 0.0, // Deterministic by default for audit
            top_p: 1.0,
            max_retries: 3,
        }
    }

    /// Create a new Anthropic backend from the `ANTHROPIC_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env() -> BackendResult<Self> {
        Self::from_secret_provider(&EnvSecretProvider)
    }

    /// Create a new Anthropic backend by loading the API key from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns error if the secret cannot be loaded.
    pub fn from_secret_provider(secrets: &dyn SecretProvider) -> BackendResult<Self> {
        let api_key =
            secrets
                .get_secret("ANTHROPIC_API_KEY")
                .map_err(|e| BackendError::Unavailable {
                    message: format!("ANTHROPIC_API_KEY: {e}"),
                })?;
        Ok(Self {
            api_key,
            model: "claude-sonnet-4-6".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            client: Client::new(),
            temperature: 0.0,
            top_p: 1.0,
            max_retries: 3,
        })
    }

    /// Set the model to use.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the temperature for sampling.
    #[must_use]
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set top-p for nucleus sampling.
    #[must_use]
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    /// Set max retries for transient failures.
    #[must_use]
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Build the HTTP headers for Anthropic API.
    fn build_headers(&self) -> BackendResult<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(self.api_key.expose()).map_err(|e| {
                BackendError::InvalidRequest {
                    message: format!("Invalid API key: {e}"),
                }
            })?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        Ok(headers)
    }

    /// Convert `ChatRequest` to Anthropic message format.
    fn convert_chat_request(&self, req: &ChatRequest) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system = None;
        let mut messages = Vec::new();
        for msg in &req.messages {
            match msg.role {
                ChatRole::System => system = Some(msg.content.clone()),
                ChatRole::User => messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: msg.content.clone(),
                }),
                ChatRole::Assistant => messages.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: msg.content.clone(),
                }),
            }
        }
        (system, messages)
    }

    /// Synchronous chat execution — called from the `ChatBackend` future.
    fn chat_sync(&self, req: ChatRequest) -> Result<ChatResponse, ChatLlmError> {
        let (system, messages) = self.convert_chat_request(&req);
        let model = req.model.clone().unwrap_or_else(|| self.model.clone());
        let max_tokens = req.max_tokens.map(|t| t as usize).unwrap_or(4096);
        let temperature = req.temperature.or(Some(self.temperature));

        let anthropic_req = AnthropicRequest {
            model,
            max_tokens,
            temperature,
            top_p: Some(self.top_p),
            system,
            messages,
        };

        let (response, _, _) =
            self.execute_with_retries(&anthropic_req)
                .map_err(|e| ChatLlmError::ProviderError {
                    message: e.to_string(),
                    code: None,
                })?;

        let content = response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn" | "stop_sequence") => Some(ChatFinishReason::Stop),
            Some("max_tokens") => Some(ChatFinishReason::Length),
            _ => None,
        };

        Ok(ChatResponse {
            content,
            usage: Some(ChatTokenUsage {
                prompt_tokens: response.usage.input_tokens as u32,
                completion_tokens: response.usage.output_tokens as u32,
                total_tokens: (response.usage.input_tokens + response.usage.output_tokens) as u32,
            }),
            model: Some(response.model),
            finish_reason,
        })
    }

    /// Compute fingerprint of request for audit.
    #[allow(dead_code)]
    fn request_fingerprint(&self, request: &AnthropicRequest) -> String {
        let canonical = serde_json::to_string(request).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute fingerprint of response for audit.
    #[allow(dead_code)]
    fn response_fingerprint(&self, response: &AnthropicResponse) -> String {
        let canonical = serde_json::to_string(response).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Execute with retries.
    fn execute_with_retries(
        &self,
        request: &AnthropicRequest,
    ) -> BackendResult<(AnthropicResponse, bool, Vec<String>)> {
        let url = format!("{}/v1/messages", self.base_url);
        let headers = self.build_headers()?;

        let mut last_error = None;
        let mut retry_reasons = Vec::new();
        let mut retried = false;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                retried = true;
                // Exponential backoff
                std::thread::sleep(std::time::Duration::from_millis(
                    100 * 2_u64.pow(attempt as u32),
                ));
            }

            let result = self
                .client
                .post(&url)
                .headers(headers.clone())
                .json(request)
                .send();

            match result {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        match response.json::<AnthropicResponse>() {
                            Ok(parsed) => return Ok((parsed, retried, retry_reasons)),
                            Err(e) => {
                                retry_reasons.push(format!("Parse error: {e}"));
                                last_error = Some(BackendError::ExecutionFailed {
                                    message: format!("Failed to parse response: {e}"),
                                });
                            }
                        }
                    } else if status.as_u16() == 429 || status.as_u16() >= 500 {
                        // Retryable errors
                        retry_reasons.push(format!("HTTP {}", status.as_u16()));
                        last_error = Some(BackendError::ExecutionFailed {
                            message: format!("API error: HTTP {}", status.as_u16()),
                        });
                    } else {
                        // Non-retryable error
                        let body = response.text().unwrap_or_default();
                        return Err(BackendError::ExecutionFailed {
                            message: format!("API error: HTTP {} - {}", status.as_u16(), body),
                        });
                    }
                }
                Err(e) => {
                    retry_reasons.push(format!("Network error: {e}"));
                    last_error = Some(BackendError::ExecutionFailed {
                        message: format!("Network error: {e}"),
                    });
                }
            }
        }

        Err(last_error.unwrap_or_else(|| BackendError::ExecutionFailed {
            message: "Unknown error".to_string(),
        }))
    }
}

impl ChatBackend for AnthropicBackend {
    type ChatFut<'a>
        = std::future::Ready<Result<ChatResponse, ChatLlmError>>
    where
        Self: 'a;

    fn chat(&self, req: ChatRequest) -> Self::ChatFut<'_> {
        std::future::ready(self.chat_sync(req))
    }
}

// ============================================================================
// Anthropic API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicResponse {
    id: Option<String>,
    model: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

// ============================================================================
// Cost Estimation
// ============================================================================

/// Estimate cost in microdollars based on model and usage.
///
/// Prices as of 2025 (subject to change):
/// - claude-opus-4-6: $15/1M input, $75/1M output
/// - claude-sonnet-4-6: $3/1M input, $15/1M output
/// - claude-haiku-4-5: $0.80/1M input, $4/1M output
#[allow(dead_code)]
fn estimate_cost(model: &str, usage: &AnthropicUsage) -> u64 {
    let (input_per_m, output_per_m) = if model.contains("opus") {
        (15_000_000, 75_000_000) // microdollars per million tokens
    } else if model.contains("sonnet") {
        (3_000_000, 15_000_000)
    } else if model.contains("haiku") {
        (250_000, 1_250_000)
    } else {
        (3_000_000, 15_000_000) // Default to sonnet pricing
    };

    let input_cost = (usage.input_tokens as u64 * input_per_m) / 1_000_000;
    let output_cost = (usage.output_tokens as u64 * output_per_m) / 1_000_000;

    input_cost + output_cost
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::traits::{ChatMessage, ChatRequest, ChatRole};

    #[test]
    fn test_anthropic_backend_creation() {
        let backend = AnthropicBackend::new("test-key")
            .with_model("claude-haiku-4-5-20251001")
            .with_temperature(0.5);

        assert_eq!(backend.model, "claude-haiku-4-5-20251001");
        assert_eq!(backend.temperature, 0.5);
        assert_eq!(backend.api_key.expose(), "test-key");
    }

    #[test]
    fn test_convert_text_prompt() {
        let backend = AnthropicBackend::new("test-key");
        let req = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "Hello".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            model: None,
        };

        let (system, messages) = backend.convert_chat_request(&req);

        assert!(system.is_none());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
    }

    #[test]
    fn test_convert_messages_prompt() {
        let backend = AnthropicBackend::new("test-key");
        let req = ChatRequest {
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: "You are helpful.".to_string(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: "Hi".to_string(),
                },
            ],
            max_tokens: None,
            temperature: None,
            model: None,
        };

        let (system, messages) = backend.convert_chat_request(&req);

        assert_eq!(system, Some("You are helpful.".to_string()));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
    }

    #[test]
    fn test_cost_estimation() {
        let usage = AnthropicUsage {
            input_tokens: 1000,
            output_tokens: 500,
        };

        // Sonnet pricing: $3/1M input, $15/1M output
        let cost = estimate_cost("claude-sonnet-4-6", &usage);

        // 1000 * 3_000_000 / 1_000_000 + 500 * 15_000_000 / 1_000_000
        // = 3000 + 7500 = 10500 microdollars = $0.0105
        assert_eq!(cost, 10500);
    }

    #[test]
    fn test_request_fingerprint_deterministic() {
        let backend = AnthropicBackend::new("test-key");
        let request = AnthropicRequest {
            model: "claude-3-sonnet".to_string(),
            max_tokens: 100,
            temperature: Some(0.0),
            top_p: None,
            system: None,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "test".to_string(),
            }],
        };

        let fp1 = backend.request_fingerprint(&request);
        let fp2 = backend.request_fingerprint(&request);

        assert_eq!(fp1, fp2);
        assert!(!fp1.is_empty());
    }

    #[test]
    fn test_replayability_based_on_temperature() {
        // With temperature=0, best effort replayability
        let backend = AnthropicBackend::new("test-key").with_temperature(0.0);
        assert_eq!(backend.temperature, 0.0);

        // With temperature>0, no replayability
        let backend = AnthropicBackend::new("test-key").with_temperature(0.7);
        assert_eq!(backend.temperature, 0.7);
    }
}
