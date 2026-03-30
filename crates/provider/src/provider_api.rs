// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! LLM provider invocation types.
//!
//! These types define the invocation API for LLM providers — how you call them
//! and what you get back. This is the provider-specific boundary; the
//! platform-wide contract (`Backend`, `Agent`, `AgentEffect`) lives in
//! `converge-traits`.
//!
//! # Relationship to converge-traits
//!
//! | This module             | converge-traits          |
//! |-------------------------|--------------------------|
//! | `LlmProvider` (invoke)  | `Backend` (identity)     |
//! | `LlmRequest/Response`   | `AgentEffect` (effects)  |
//! | `LlmError`              | `BackendError` (generic) |

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Request / Response ───────────────────────────────────────────────

/// Request to an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// The user prompt.
    pub prompt: String,
    /// Optional system prompt.
    pub system: Option<String>,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: f64,
    /// Optional stop sequences.
    pub stop_sequences: Vec<String>,
}

impl LlmRequest {
    #[must_use]
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            system: None,
            max_tokens: 1024,
            temperature: 0.7,
            stop_sequences: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    #[must_use]
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    #[must_use]
    pub fn with_stop_sequence(mut self, stop: impl Into<String>) -> Self {
        self.stop_sequences.push(stop.into());
        self
    }
}

/// Response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The generated content.
    pub content: String,
    /// The model that generated this response.
    pub model: String,
    /// Token usage statistics.
    pub usage: TokenUsage,
    /// Finish reason.
    pub finish_reason: FinishReason,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens in the completion.
    pub completion_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
}

/// Reason the generation stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural end of response.
    Stop,
    /// Hit `max_tokens` limit.
    MaxTokens,
    /// Hit a stop sequence.
    StopSequence,
    /// Content was filtered.
    ContentFilter,
}

// ── Error types ──────────────────────────────────────────────────────

/// Error from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{kind:?}: {message}")]
pub struct LlmError {
    /// Error kind.
    pub kind: LlmErrorKind,
    /// Human-readable message.
    pub message: String,
    /// Whether the request can be retried.
    pub retryable: bool,
}

/// Kind of LLM error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmErrorKind {
    /// Invalid API key.
    Authentication,
    /// Rate limit exceeded.
    RateLimit,
    /// Invalid request parameters.
    InvalidRequest,
    /// Model not available.
    ModelNotFound,
    /// Network or connection error.
    Network,
    /// Provider returned an error.
    ProviderError,
    /// Response couldn't be parsed.
    ParseError,
    /// Request timed out.
    Timeout,
}

impl LlmError {
    #[must_use]
    pub fn new(kind: LlmErrorKind, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            kind,
            message: message.into(),
            retryable,
        }
    }

    #[must_use]
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Authentication, message, false)
    }

    #[must_use]
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::RateLimit, message, true)
    }

    #[must_use]
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Network, message, true)
    }

    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::ParseError, message, false)
    }

    #[must_use]
    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::ProviderError, message, false)
    }

    #[must_use]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Timeout, message, true)
    }
}

// ── Provider trait ───────────────────────────────────────────────────

/// Trait for LLM providers — the invocation API.
///
/// This is one instantiation strategy for converge agents. A provider
/// wraps a remote LLM API (Anthropic, `OpenAI`, etc.) and exposes a
/// simple prompt-in/response-out interface.
///
/// Providers also implement [`converge_traits::Backend`] for identity
/// and capability declaration, enabling the platform's capability-based
/// selection.
pub trait LlmProvider: Send + Sync {
    /// The name of this provider (e.g., "anthropic", "openai").
    fn name(&self) -> &'static str;

    /// The model being used (e.g., "claude-sonnet-4-6", "gpt-4").
    fn model(&self) -> &str;

    /// Sends a completion request to the LLM.
    ///
    /// # Errors
    ///
    /// Returns `LlmError` if the request fails.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Returns a provenance string for tracking.
    fn provenance(&self, request_id: &str) -> String {
        format!("{}:{}", self.model(), request_id)
    }

    /// Quick health check — sends a minimal request to verify the provider is reachable
    /// and the API key/quota is valid.
    ///
    /// The default implementation sends a trivial completion request. Providers can
    /// override this with a lighter-weight check if available.
    ///
    /// # Errors
    ///
    /// Returns `LlmError` if the provider is unreachable, the key is invalid,
    /// or the quota is exhausted.
    fn health_check(&self) -> Result<(), LlmError> {
        let request = LlmRequest::new("Say OK").with_max_tokens(1);
        self.complete(&request).map(|_| ())
    }
}

// ── Selection types (local copies, compatible with converge-traits) ──

/// Requirements for model selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequirements {
    pub max_cost_class: CostClass,
    pub max_latency_ms: u32,
    pub requires_reasoning: bool,
    pub requires_web_search: bool,
    pub min_quality: f64,
    pub data_sovereignty: DataSovereignty,
    pub compliance: ComplianceLevel,
    pub requires_multilingual: bool,
}

impl AgentRequirements {
    #[must_use]
    pub fn new(max_cost_class: CostClass, max_latency_ms: u32, requires_reasoning: bool) -> Self {
        Self {
            max_cost_class,
            max_latency_ms,
            requires_reasoning,
            requires_web_search: false,
            min_quality: 0.0,
            data_sovereignty: DataSovereignty::Any,
            compliance: ComplianceLevel::None,
            requires_multilingual: false,
        }
    }

    #[must_use]
    pub fn fast_cheap() -> Self {
        Self::new(CostClass::VeryLow, 2000, false)
    }

    #[must_use]
    pub fn balanced() -> Self {
        Self::new(CostClass::Low, 5000, false).with_quality(0.8)
    }

    #[must_use]
    pub fn powerful() -> Self {
        Self::new(CostClass::High, 10000, true).with_quality(0.9)
    }

    #[must_use]
    pub fn with_quality(mut self, quality: f64) -> Self {
        self.min_quality = quality;
        self
    }

    #[must_use]
    pub fn with_web_search(mut self, required: bool) -> Self {
        self.requires_web_search = required;
        self
    }

    #[must_use]
    pub fn with_data_sovereignty(mut self, sovereignty: DataSovereignty) -> Self {
        self.data_sovereignty = sovereignty;
        self
    }

    #[must_use]
    pub fn with_compliance(mut self, compliance: ComplianceLevel) -> Self {
        self.compliance = compliance;
        self
    }

    #[must_use]
    pub fn with_multilingual(mut self, required: bool) -> Self {
        self.requires_multilingual = required;
        self
    }
}

/// Trait for model selection.
pub trait ModelSelectorTrait: Send + Sync {
    /// Selects a model (provider, `model_id`) satisfying the requirements.
    fn select(&self, requirements: &AgentRequirements) -> Result<(String, String), LlmError>;
}

// Re-export converge-traits selection types that are still compatible.
pub use converge_traits::{ComplianceLevel, CostClass, DataSovereignty};
