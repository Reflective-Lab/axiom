// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Common abstractions for LLM providers.
//!
//! This module provides shared types and utilities to reduce code duplication
//! across provider implementations.

use crate::provider_api::{
    FinishReason, LlmError, LlmErrorKind, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{SecretProvider, SecretString};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/// Base configuration for HTTP-based LLM providers.
#[derive(Clone)]
pub struct HttpProviderConfig {
    /// API key for authentication (wrapped for redaction and optional zeroize).
    pub api_key: SecretString,
    /// Model identifier.
    pub model: String,
    /// Base URL for the API.
    pub base_url: String,
    /// HTTP client.
    pub client: Client,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for HttpProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpProviderConfig")
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl HttpProviderConfig {
    /// Creates a new HTTP provider configuration.
    #[must_use]
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            model: model.into(),
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    /// Creates a configuration by loading the API key from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns `LlmError::auth` if the secret cannot be loaded.
    pub fn from_secret_provider(
        secret_provider: &dyn SecretProvider,
        secret_key: &str,
        model: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let api_key = secret_provider
            .get_secret(secret_key)
            .map_err(|e| LlmError::auth(format!("{secret_key}: {e}")))?;
        Ok(Self {
            api_key,
            model: model.into(),
            base_url: base_url.into(),
            client: Client::new(),
        })
    }

    /// Uses a custom HTTP client.
    #[must_use]
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }
}

/// OpenAI-compatible message format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    /// Message role (system, user, assistant).
    pub role: String,
    /// Message content.
    pub content: String,
}

impl ChatMessage {
    /// Creates a system message.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    /// Creates a user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// OpenAI-compatible chat completion request.
#[derive(Serialize, Debug)]
pub struct ChatCompletionRequest {
    /// Model identifier.
    pub model: String,
    /// Messages in the conversation.
    pub messages: Vec<ChatMessage>,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Temperature (0.0-2.0).
    pub temperature: f64,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop: Vec<String>,
}

impl ChatCompletionRequest {
    /// Creates a request from an `LlmRequest`.
    #[must_use]
    pub fn from_llm_request(model: impl Into<String>, request: &LlmRequest) -> Self {
        let mut messages = Vec::new();

        if let Some(ref system) = request.system {
            messages.push(ChatMessage::system(system));
        }

        messages.push(ChatMessage::user(&request.prompt));

        Self {
            model: model.into(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_sequences.clone(),
        }
    }
}

/// OpenAI-compatible choice in response.
#[derive(Deserialize, Debug)]
pub struct ChatChoice {
    /// Message from the choice.
    pub message: ChatChoiceMessage,
    /// Finish reason.
    pub finish_reason: Option<String>,
}

/// OpenAI-compatible message in choice.
#[derive(Deserialize, Debug)]
pub struct ChatChoiceMessage {
    /// Message content.
    pub content: String,
}

/// OpenAI-compatible usage statistics.
#[derive(Deserialize, Debug)]
pub struct ChatUsage {
    /// Prompt tokens.
    pub prompt_tokens: u32,
    /// Completion tokens.
    pub completion_tokens: u32,
    /// Total tokens.
    pub total_tokens: u32,
}

/// OpenAI-compatible chat completion response.
#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    /// Model used.
    pub model: String,
    /// Choices in the response.
    pub choices: Vec<ChatChoice>,
    /// Usage statistics.
    pub usage: ChatUsage,
}

/// Converts a finish reason string to `FinishReason` enum.
#[must_use]
pub fn parse_finish_reason(reason: Option<&str>) -> FinishReason {
    match reason {
        Some("length" | "max_tokens") => FinishReason::MaxTokens,
        Some("content_filter") => FinishReason::ContentFilter,
        Some("stop_sequence") => FinishReason::StopSequence,
        _ => FinishReason::Stop, // "stop" or unknown
    }
}

/// Converts an OpenAI-compatible response to `LlmResponse`.
///
/// # Errors
///
/// Returns error if response has no choices.
pub fn chat_response_to_llm_response(
    response: ChatCompletionResponse,
) -> Result<LlmResponse, LlmError> {
    let choice = response
        .choices
        .first()
        .ok_or_else(|| LlmError::provider("No choices in response"))?;

    Ok(LlmResponse {
        content: choice.message.content.clone(),
        model: response.model,
        finish_reason: parse_finish_reason(choice.finish_reason.as_deref()),
        usage: TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        },
    })
}

/// Makes an OpenAI-compatible chat completion request.
///
/// # Errors
///
/// Returns error if the HTTP request fails or response cannot be parsed.
pub fn make_chat_completion_request(
    config: &HttpProviderConfig,
    endpoint: &str,
    request: &ChatCompletionRequest,
) -> Result<LlmResponse, LlmError> {
    let url = format!("{}{}", config.base_url, endpoint);

    let http_response = config
        .client
        .post(&url)
        .header(
            "Authorization",
            format!("Bearer {}", config.api_key.expose()),
        )
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

    let status = http_response.status();

    if !status.is_success() {
        return handle_openai_style_error(http_response);
    }

    let api_response: ChatCompletionResponse = http_response
        .json()
        .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

    chat_response_to_llm_response(api_response)
}

/// OpenAI-compatible error response format.
#[derive(Deserialize, Debug)]
pub struct OpenAiStyleError {
    /// Error details.
    pub error: OpenAiStyleErrorDetail,
}

/// Error detail in OpenAI-compatible format.
#[derive(Deserialize, Debug)]
pub struct OpenAiStyleErrorDetail {
    /// Error message.
    pub message: String,
    /// Error type (e.g., "`authentication_error`", "`rate_limit_error`").
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// Handles HTTP error responses for OpenAI-compatible providers.
///
/// Parses the error response and maps error types to appropriate `LlmError` kinds.
///
/// # Errors
///
/// Returns error if:
/// - Response cannot be parsed as JSON
/// - Error type indicates authentication failure → `LlmError::auth()`
/// - Error type indicates rate limit → `LlmError::rate_limit()`
/// - Other errors → `LlmError::provider()`
pub fn handle_openai_style_error(
    http_response: reqwest::blocking::Response,
) -> Result<LlmResponse, LlmError> {
    let http_code = http_response.status().as_u16();

    let error_body: OpenAiStyleError = http_response
        .json()
        .map_err(|e| LlmError::parse(format!("Failed to parse error: {e}")))?;

    let error_type = error_body.error.error_type.as_deref().unwrap_or("unknown");
    let message = error_body.error.message;

    // HTTP 429 and 5xx are always retryable, regardless of error body classification
    if http_code == 429 || http_code >= 500 {
        return Err(LlmError::new(
            if http_code == 429 {
                LlmErrorKind::RateLimit
            } else {
                LlmErrorKind::ProviderError
            },
            message,
            true,
        ));
    }

    let llm_error = match error_type {
        "authentication_error" => LlmError::auth(message),
        "invalid_request_error" => LlmError::new(
            LlmErrorKind::InvalidRequest,
            message,
            true, // retryable: a different provider may accept the same request
        ),
        "rate_limit_error" => LlmError::rate_limit(message),
        _ => LlmError::provider(message),
    };

    Err(llm_error)
}

/// Helper for providers that use OpenAI-compatible API.
///
/// This trait can be implemented by providers to reduce boilerplate.
pub trait OpenAiCompatibleProvider {
    /// Gets the provider configuration.
    fn config(&self) -> &HttpProviderConfig;

    /// Gets the API endpoint path (e.g., "/v1/chat/completions").
    fn endpoint(&self) -> &str;

    /// Makes a completion request.
    ///
    /// Default implementation uses `make_chat_completion_request`.
    ///
    /// # Errors
    ///
    /// Returns error if the HTTP request fails or response cannot be parsed.
    fn complete_openai_compatible(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let chat_request =
            ChatCompletionRequest::from_llm_request(self.config().model.clone(), request);
        make_chat_completion_request(self.config(), self.endpoint(), &chat_request)
    }
}
