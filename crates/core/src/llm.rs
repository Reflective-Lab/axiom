// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! LLM provider abstraction for Converge.
//!
//! These types were previously in `converge-traits` v0.1.0 (LLM-specific).
//! As of v0.2.0, `converge-traits` is a pure generic backend abstraction.
//! LLM-specific types now live here in `converge-core` and will eventually
//! migrate to `converge-provider`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// LLM ERROR
// =============================================================================

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
    pub fn new(kind: LlmErrorKind, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            kind,
            message: message.into(),
            retryable,
        }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Authentication, message, false)
    }

    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::RateLimit, message, true)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Network, message, true)
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::ParseError, message, false)
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::ProviderError, message, false)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::Timeout, message, true)
    }
}

// =============================================================================
// LLM REQUEST / RESPONSE
// =============================================================================

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
    /// Hit max_tokens limit.
    MaxTokens,
    /// Hit a stop sequence.
    StopSequence,
    /// Content was filtered.
    ContentFilter,
}

/// Trait for LLM providers.
pub trait LlmProvider: Send + Sync {
    /// The name of this provider (e.g., "anthropic", "openai").
    fn name(&self) -> &'static str;

    /// The model being used (e.g., "claude-3-opus", "gpt-4").
    fn model(&self) -> &str;

    /// Sends a completion request to the LLM.
    ///
    /// # Errors
    ///
    /// Returns `LlmError` if the request fails.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Returns a provenance string for tracking (e.g., "claude-3-opus:abc123").
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

// =============================================================================
// LLM AGENT
// =============================================================================

use crate::agent::Suggestor;
use crate::context::{ContextKey, ProposedFact};
use crate::effect::AgentEffect;
use std::sync::Arc;

/// Configuration for an LLM-powered agent.
#[derive(Clone)]
pub struct LlmAgentConfig {
    /// System prompt for the LLM.
    pub system_prompt: String,
    /// Template for the user prompt (use {context} for context injection).
    pub prompt_template: String,
    /// Prompt format (EDN by default for token efficiency).
    pub prompt_format: crate::prompt::PromptFormat,
    /// Target context key for generated proposals.
    pub target_key: ContextKey,
    /// Dependencies that trigger this agent.
    pub dependencies: Vec<ContextKey>,
    /// Default confidence for proposals (can be overridden by parser).
    pub default_confidence: f64,
    /// Maximum tokens for generation.
    pub max_tokens: u32,
    /// Temperature for generation.
    pub temperature: f64,
    /// Requirements for model selection (optional).
    pub requirements: Option<crate::model_selection::AgentRequirements>,
}

impl Default for LlmAgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            prompt_template: "{context}".into(),
            prompt_format: crate::prompt::PromptFormat::Edn,
            target_key: ContextKey::Hypotheses,
            dependencies: vec![ContextKey::Seeds],
            default_confidence: 0.7,
            max_tokens: 1024,
            temperature: 0.7,
            requirements: None,
        }
    }
}

/// Parser for LLM responses into proposals.
pub trait ResponseParser: Send + Sync {
    /// Parses an LLM response into proposals.
    fn parse(&self, response: &LlmResponse, target_key: ContextKey) -> Vec<ProposedFact>;
}

/// Simple parser that creates one proposal from the entire response.
pub struct SimpleParser {
    /// ID prefix for generated proposals.
    pub id_prefix: String,
    /// Default confidence.
    pub confidence: f64,
}

impl Default for SimpleParser {
    fn default() -> Self {
        Self {
            id_prefix: "llm".into(),
            confidence: 0.7,
        }
    }
}

impl ResponseParser for SimpleParser {
    fn parse(&self, response: &LlmResponse, target_key: ContextKey) -> Vec<ProposedFact> {
        let content = response.content.trim();
        if content.is_empty() {
            return Vec::new();
        }

        let id = format!("{}-{}", self.id_prefix, uuid_v4_simple());

        vec![ProposedFact {
            key: target_key,
            id,
            content: content.to_string(),
            confidence: self.confidence,
            provenance: response.model.clone(),
        }]
    }
}

/// An agent powered by an LLM provider.
pub struct LlmAgent {
    name: String,
    provider: Arc<dyn LlmProvider>,
    config: LlmAgentConfig,
    parser: Arc<dyn ResponseParser>,
    full_dependencies: Vec<ContextKey>,
}

impl LlmAgent {
    pub fn new(
        name: impl Into<String>,
        provider: Arc<dyn LlmProvider>,
        config: LlmAgentConfig,
    ) -> Self {
        let name_str = name.into();
        let mut full_dependencies = config.dependencies.clone();
        if !full_dependencies.contains(&config.target_key) {
            full_dependencies.push(config.target_key);
        }
        let parser = Arc::new(SimpleParser {
            id_prefix: name_str.clone(),
            confidence: 0.7,
        });
        Self {
            name: name_str,
            provider,
            config,
            parser,
            full_dependencies,
        }
    }
}

impl Suggestor for LlmAgent {
    fn name(&self) -> &str {
        &self.name
    }
    fn dependencies(&self) -> &[ContextKey] {
        &self.full_dependencies
    }
    fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
        let has_input = self.config.dependencies.iter().any(|k| ctx.has(*k));
        if !has_input {
            return false;
        }
        let my_prefix = format!("{}-", self.name);
        !ctx.get(self.config.target_key)
            .iter()
            .any(|f| f.id.starts_with(&my_prefix))
    }
    fn execute(&self, _ctx: &dyn crate::ContextView) -> AgentEffect {
        let request = LlmRequest::new("prompt") // Simplified for traits demonstration
            .with_max_tokens(self.config.max_tokens)
            .with_temperature(self.config.temperature);
        match self.provider.complete(&request) {
            Ok(response) => {
                let proposals = self.parser.parse(&response, self.config.target_key);
                AgentEffect::with_proposals(proposals)
            }
            Err(_) => AgentEffect::empty(),
        }
    }
}

fn uuid_v4_simple() -> String {
    "test".into()
}
