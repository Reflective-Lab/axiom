// Copyright 2024-2026 Reflective Labs
//
// LLM Provider Abstraction - Migrated from converge-core for purity.
//
// This module provides extensions to the core LlmProvider trait,
// including agent wrappers, routers, and response parsers.
//
// The core trait (LlmProvider) remains in converge-core and is re-exported
// from this crate for convenience.

use std::fmt;
use std::sync::Arc;

// Import types from converge-core using public re-exports
use converge_core::{AgentEffect, ContextKey, ProposedFact, Suggestor};

// Re-export core LLM types - these are the canonical types
// NOTE: LlmProvider is deprecated in converge-core. We define ChatProvider below as the replacement.
pub use converge_core::llm::{
    FinishReason, LlmError, LlmErrorKind, LlmRequest, LlmResponse, TokenUsage,
};

// ============================================================================
// ChatProvider Trait (replacement for deprecated LlmProvider)
// ============================================================================

/// Synchronous chat completion provider.
///
/// This trait replaces the deprecated `LlmProvider` from converge-core.
/// It provides a simple sync interface for LLM completions.
///
/// For async usage, implement `converge_core::traits::ChatBackend` instead.
///
/// # Blanket Implementation
///
/// Any type implementing the deprecated `converge_core::llm::LlmProvider`
/// automatically gets a `ChatProvider` implementation via a blanket impl.
/// This allows existing providers (like `AnthropicProvider`, `OpenAiProvider`)
/// to work with the new trait without changes.
pub trait ChatProvider: Send + Sync {
    /// Provider name for identification.
    fn name(&self) -> &str;

    /// Model identifier.
    fn model(&self) -> &str;

    /// Complete a chat request.
    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;
}

// ============================================================================
// Blanket Implementation: LlmProvider -> ChatProvider
// ============================================================================

/// Blanket implementation that bridges deprecated LlmProvider to ChatProvider.
///
/// This allows existing provider implementations (AnthropicProvider, OpenAiProvider)
/// that implement the deprecated `converge_core::llm::LlmProvider` to automatically
/// work with the new `ChatProvider` trait.
#[allow(deprecated)]
impl<T> ChatProvider for T
where
    T: converge_core::llm::LlmProvider + Send + Sync,
{
    fn name(&self) -> &str {
        converge_core::llm::LlmProvider::name(self)
    }

    fn model(&self) -> &str {
        converge_core::llm::LlmProvider::model(self)
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        converge_core::llm::LlmProvider::complete(self, request)
    }
}

// Import prompt types from our local prompt_dsl module
use crate::prompt_dsl::{
    AgentPrompt, AgentRole, Constraint, DslOutputContract, PromptContext, PromptFormat,
};

// =============================================================================
// PROVIDER-SPECIFIC ERROR TYPES (for extended error handling)
// =============================================================================

/// Extended error type with additional provider-specific information.
/// Wraps the core LlmError with additional context.
#[derive(Debug, Clone)]
pub struct ProviderError {
    /// The underlying error.
    pub inner: LlmError,
}

impl ProviderError {
    /// Creates a new provider error from an LlmError.
    #[must_use]
    pub fn from_llm_error(error: LlmError) -> Self {
        Self { inner: error }
    }

    /// Creates an authentication error.
    #[must_use]
    pub fn auth(message: impl Into<String>) -> Self {
        Self {
            inner: LlmError::auth(message),
        }
    }

    /// Creates a rate limit error.
    #[must_use]
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self {
            inner: LlmError::rate_limit(message),
        }
    }

    /// Creates a network error.
    #[must_use]
    pub fn network(message: impl Into<String>) -> Self {
        Self {
            inner: LlmError::network(message),
        }
    }

    /// Creates a parse error.
    #[must_use]
    pub fn parse(message: impl Into<String>) -> Self {
        Self {
            inner: LlmError::parse(message),
        }
    }

    /// Creates a provider error.
    #[must_use]
    pub fn provider(message: impl Into<String>) -> Self {
        Self {
            inner: LlmError::provider(message),
        }
    }

    /// Returns whether this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.inner.retryable
    }

    /// Returns the error kind.
    #[must_use]
    pub fn kind(&self) -> LlmErrorKind {
        self.inner.kind
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::error::Error for ProviderError {}

impl From<LlmError> for ProviderError {
    fn from(error: LlmError) -> Self {
        Self::from_llm_error(error)
    }
}

// Re-export ProviderErrorKind as alias for LlmErrorKind
pub use converge_core::llm::LlmErrorKind as ProviderErrorKind;

// Re-export FinishReason with provider alias
pub use converge_core::llm::FinishReason as ProviderFinishReason;

// =============================================================================
// MOCK PROVIDER (extended version for testing)
// =============================================================================

/// Pre-configured response for `MockProvider`.
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// The content to return.
    pub content: String,
    /// Simulated confidence (used by callers to set `ProposedFact` confidence).
    pub confidence: f64,
    /// Whether this response should succeed.
    pub success: bool,
    /// Optional error to return.
    pub error: Option<LlmError>,
}

impl MockResponse {
    /// Creates a successful mock response.
    #[must_use]
    pub fn success(content: impl Into<String>, confidence: f64) -> Self {
        Self {
            content: content.into(),
            confidence,
            success: true,
            error: None,
        }
    }

    /// Creates a failing mock response.
    #[must_use]
    pub fn failure(error: LlmError) -> Self {
        Self {
            content: String::new(),
            confidence: 0.0,
            success: false,
            error: Some(error),
        }
    }
}

/// Mock LLM provider for testing.
///
/// Returns pre-configured responses in order. Useful for deterministic tests.
pub struct MockProvider {
    model: String,
    responses: std::sync::Mutex<Vec<MockResponse>>,
    call_count: std::sync::atomic::AtomicUsize,
}

impl MockProvider {
    /// Creates a new mock provider with pre-configured responses.
    #[must_use]
    pub fn new(responses: Vec<MockResponse>) -> Self {
        Self {
            model: "mock-model".into(),
            responses: std::sync::Mutex::new(responses),
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Creates a mock that always returns the same response.
    #[must_use]
    pub fn constant(content: impl Into<String>, confidence: f64) -> Self {
        let content = content.into();
        let responses = (0..100)
            .map(|_| MockResponse::success(content.clone(), confidence))
            .collect();
        Self::new(responses)
    }

    /// Returns the number of times `complete` was called.
    #[must_use]
    pub fn call_count(&self) -> usize {
        self.call_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl ChatProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut responses = self.responses.lock().expect("MockProvider mutex poisoned");

        if responses.is_empty() {
            return Err(LlmError::provider("MockProvider: no more responses"));
        }

        let response = responses.remove(0);

        if let Some(error) = response.error {
            return Err(error);
        }

        Ok(LlmResponse {
            content: response.content,
            model: self.model.clone(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            finish_reason: FinishReason::Stop,
        })
    }
}

// =============================================================================
// LLM AGENT
// =============================================================================

/// Configuration for an LLM-powered agent.
#[derive(Clone)]
pub struct LlmAgentConfig {
    /// System prompt for the LLM.
    pub system_prompt: String,
    /// Template for the user prompt (use {context} for context injection).
    pub prompt_template: String,
    /// Prompt format (EDN by default for token efficiency).
    pub prompt_format: PromptFormat,
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
}

impl Default for LlmAgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            prompt_template: "{context}".into(),
            prompt_format: PromptFormat::Edn,
            target_key: ContextKey::Hypotheses,
            dependencies: vec![ContextKey::Seeds],
            default_confidence: 0.7,
            max_tokens: 1024,
            temperature: 0.7,
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

/// Parser that splits response into multiple proposals by delimiter.
pub struct MultiLineParser {
    /// ID prefix for generated proposals.
    pub id_prefix: String,
    /// Delimiter to split on (e.g., "\n", "---").
    pub delimiter: String,
    /// Default confidence.
    pub confidence: f64,
}

impl MultiLineParser {
    /// Creates a parser that splits on newlines.
    #[must_use]
    pub fn newline(id_prefix: impl Into<String>, confidence: f64) -> Self {
        Self {
            id_prefix: id_prefix.into(),
            delimiter: "\n".into(),
            confidence,
        }
    }
}

impl ResponseParser for MultiLineParser {
    fn parse(&self, response: &LlmResponse, target_key: ContextKey) -> Vec<ProposedFact> {
        response
            .content
            .split(&self.delimiter)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .enumerate()
            .map(|(i, content)| ProposedFact {
                key: target_key,
                id: format!("{}-{}", self.id_prefix, i),
                content: content.to_string(),
                confidence: self.confidence,
                provenance: response.model.clone(),
            })
            .collect()
    }
}

/// An agent powered by an LLM provider.
pub struct ProviderAgent {
    name: String,
    provider: Arc<dyn ChatProvider>,
    config: LlmAgentConfig,
    parser: Arc<dyn ResponseParser>,
    full_dependencies: Vec<ContextKey>,
}

impl ProviderAgent {
    /// Creates a new LLM agent.
    pub fn new(
        name: impl Into<String>,
        provider: Arc<dyn ChatProvider>,
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

    /// Sets a custom response parser.
    #[must_use]
    pub fn with_parser(mut self, parser: Arc<dyn ResponseParser>) -> Self {
        self.parser = parser;
        self
    }

    /// Builds the prompt from context using the configured format.
    fn build_prompt(&self, ctx: &dyn converge_core::ContextView) -> String {
        use std::fmt::Write;

        if matches!(self.config.prompt_format, PromptFormat::Edn) {
            let prompt_ctx = PromptContext::from_context(ctx, &self.config.dependencies);
            let output_contract = DslOutputContract::new("proposed-fact", self.config.target_key);

            let objective = if self.config.prompt_template == "{context}" {
                format!("analyze-{:?}", self.config.target_key).to_lowercase()
            } else {
                self.config
                    .prompt_template
                    .replace("{context}", "")
                    .trim()
                    .to_string()
            };

            let agent_prompt =
                AgentPrompt::new(AgentRole::Proposer, objective, prompt_ctx, output_contract)
                    .with_constraint(Constraint::NoHallucinate)
                    .with_constraint(Constraint::NoInvent);

            return agent_prompt.serialize(self.config.prompt_format);
        }

        // Fallback to plain text format
        let mut context_str = String::new();

        for &key in &self.config.dependencies {
            let facts = ctx.get(key);
            if !facts.is_empty() {
                let _ = writeln!(context_str, "\n## {key:?}");
                for fact in facts {
                    let _ = writeln!(context_str, "- {}: {}", fact.id, fact.content);
                }
            }
        }

        self.config
            .prompt_template
            .replace("{context}", &context_str)
    }
}

impl Suggestor for ProviderAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> &[ContextKey] {
        &self.full_dependencies
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        let has_input = self.config.dependencies.iter().any(|k| ctx.has(*k));
        if !has_input {
            return false;
        }

        let my_prefix = format!("{}-", self.name);
        let already_contributed = ctx
            .get(self.config.target_key)
            .iter()
            .any(|f| f.id.starts_with(&my_prefix));

        !already_contributed
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let prompt = self.build_prompt(ctx);

        let request = LlmRequest::new(prompt)
            .with_system(self.config.system_prompt.clone())
            .with_max_tokens(self.config.max_tokens)
            .with_temperature(self.config.temperature);

        match self.provider.complete(&request) {
            Ok(response) => {
                let proposals = self.parser.parse(&response, self.config.target_key);
                AgentEffect::with_proposals(proposals)
            }
            Err(e) => {
                tracing::error!(agent = %self.name, error = %e, "LLM call failed");
                AgentEffect::empty()
            }
        }
    }
}

/// Generate a simple UUID-like string.
fn uuid_v4_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos % 0xFFFF_FFFF)
}

// =============================================================================
// LLM ROUTER - Model selection by role
// =============================================================================

/// The purpose/role an LLM is being used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LlmRole {
    /// Web research - gathering information from the internet.
    WebResearch,
    /// Fast analysis of structured data.
    FastAnalysis,
    /// Deep analysis requiring nuanced understanding.
    DeepAnalysis,
    /// Verification/second opinion on another model's output.
    Verification,
    /// Creative generation (strategies, ideas, hypotheses).
    Creative,
    /// Synthesis - combining multiple sources into coherent output.
    Synthesis,
    /// Code generation and analysis.
    Code,
    /// Summarization of long content.
    Summarization,
    /// Custom role for domain-specific purposes.
    Custom(&'static str),
}

impl fmt::Display for LlmRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebResearch => write!(f, "web-research"),
            Self::FastAnalysis => write!(f, "fast-analysis"),
            Self::DeepAnalysis => write!(f, "deep-analysis"),
            Self::Verification => write!(f, "verification"),
            Self::Creative => write!(f, "creative"),
            Self::Synthesis => write!(f, "synthesis"),
            Self::Code => write!(f, "code"),
            Self::Summarization => write!(f, "summarization"),
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

/// Routes LLM requests to appropriate providers based on role.
pub struct LlmRouter {
    providers: std::collections::HashMap<LlmRole, Arc<dyn ChatProvider>>,
    default: Option<Arc<dyn ChatProvider>>,
}

impl Default for LlmRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmRouter {
    /// Creates a new empty router.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            default: None,
        }
    }

    /// Registers a provider for a specific role.
    #[must_use]
    pub fn with_provider(mut self, role: LlmRole, provider: Arc<dyn ChatProvider>) -> Self {
        self.providers.insert(role, provider);
        self
    }

    /// Sets the default provider for unmapped roles.
    #[must_use]
    pub fn with_default(mut self, provider: Arc<dyn ChatProvider>) -> Self {
        self.default = Some(provider);
        self
    }

    /// Gets the provider for a role, falling back to default.
    #[must_use]
    pub fn get(&self, role: LlmRole) -> Option<Arc<dyn ChatProvider>> {
        self.providers
            .get(&role)
            .cloned()
            .or_else(|| self.default.clone())
    }

    /// Checks if a role has a registered provider.
    #[must_use]
    pub fn has_provider(&self, role: LlmRole) -> bool {
        self.providers.contains_key(&role) || self.default.is_some()
    }

    /// Lists all registered roles.
    #[must_use]
    pub fn roles(&self) -> Vec<LlmRole> {
        self.providers.keys().copied().collect()
    }

    /// Completes a request using the provider for the given role.
    pub fn complete(&self, role: LlmRole, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let provider = self.get(role).ok_or_else(|| {
            LlmError::provider(format!("No provider configured for role: {role}"))
        })?;
        provider.complete(request)
    }
}

/// Configuration for a multi-model pipeline.
#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    models: std::collections::HashMap<LlmRole, (String, String)>,
}

impl ModelConfig {
    /// Creates a new empty configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures a model for a role.
    #[must_use]
    pub fn model(
        mut self,
        role: LlmRole,
        provider: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Self {
        self.models.insert(role, (provider.into(), model_id.into()));
        self
    }

    /// Gets the configured model for a role.
    #[must_use]
    pub fn get(&self, role: LlmRole) -> Option<(&str, &str)> {
        self.models
            .get(&role)
            .map(|(p, m)| (p.as_str(), m.as_str()))
    }

    /// Checks if a role has a configured model.
    #[must_use]
    pub fn has(&self, role: LlmRole) -> bool {
        self.models.contains_key(&role)
    }

    /// Creates a preset for high-quality, diverse model selection.
    #[must_use]
    pub fn high_quality() -> Self {
        Self::new()
            .model(LlmRole::WebResearch, "perplexity", "sonar-pro")
            .model(LlmRole::FastAnalysis, "google", "gemini-2.0-flash")
            .model(LlmRole::DeepAnalysis, "anthropic", "claude-opus-4")
            .model(LlmRole::Verification, "openai", "gpt-4.5")
            .model(LlmRole::Creative, "anthropic", "claude-opus-4")
            .model(LlmRole::Synthesis, "anthropic", "claude-opus-4")
            .model(LlmRole::Code, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Summarization, "google", "gemini-2.0-flash")
    }

    /// Creates a preset optimized for speed and cost.
    #[must_use]
    pub fn fast() -> Self {
        Self::new()
            .model(LlmRole::WebResearch, "perplexity", "sonar")
            .model(LlmRole::FastAnalysis, "google", "gemini-2.0-flash")
            .model(LlmRole::DeepAnalysis, "google", "gemini-2.0-flash")
            .model(LlmRole::Verification, "anthropic", "claude-haiku-3.5")
            .model(LlmRole::Creative, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Synthesis, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Code, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Summarization, "google", "gemini-2.0-flash")
    }

    /// Creates a preset using only Anthropic models.
    #[must_use]
    pub fn anthropic_only() -> Self {
        Self::new()
            .model(LlmRole::WebResearch, "anthropic", "claude-sonnet-4")
            .model(LlmRole::FastAnalysis, "anthropic", "claude-haiku-3.5")
            .model(LlmRole::DeepAnalysis, "anthropic", "claude-opus-4")
            .model(LlmRole::Verification, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Creative, "anthropic", "claude-opus-4")
            .model(LlmRole::Synthesis, "anthropic", "claude-opus-4")
            .model(LlmRole::Code, "anthropic", "claude-sonnet-4")
            .model(LlmRole::Summarization, "anthropic", "claude-haiku-3.5")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_provider_returns_responses_in_order() {
        let provider = MockProvider::new(vec![
            MockResponse::success("First response", 0.8),
            MockResponse::success("Second response", 0.9),
        ]);

        let request = LlmRequest::new("test");

        let r1 = provider.complete(&request).unwrap();
        assert_eq!(r1.content, "First response");

        let r2 = provider.complete(&request).unwrap();
        assert_eq!(r2.content, "Second response");

        assert_eq!(provider.call_count(), 2);
    }

    #[test]
    fn mock_provider_can_return_errors() {
        let provider = MockProvider::new(vec![MockResponse::failure(LlmError::rate_limit(
            "Too many requests",
        ))]);

        let request = LlmRequest::new("test");
        let result = provider.complete(&request);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind, LlmErrorKind::RateLimit);
        assert!(err.retryable);
    }

    #[test]
    fn router_routes_by_role() {
        let gemini = Arc::new(MockProvider::new(vec![MockResponse::success(
            "Gemini response",
            0.85,
        )]));
        let claude = Arc::new(MockProvider::new(vec![MockResponse::success(
            "Claude response",
            0.90,
        )]));

        let router = LlmRouter::new()
            .with_provider(LlmRole::FastAnalysis, gemini)
            .with_provider(LlmRole::Synthesis, claude);

        let request = LlmRequest::new("test");

        let fast_response = router.complete(LlmRole::FastAnalysis, &request).unwrap();
        assert_eq!(fast_response.content, "Gemini response");

        let synth_response = router.complete(LlmRole::Synthesis, &request).unwrap();
        assert_eq!(synth_response.content, "Claude response");
    }

    #[test]
    fn model_config_stores_choices() {
        let config = ModelConfig::new()
            .model(LlmRole::WebResearch, "perplexity", "sonar-pro")
            .model(LlmRole::DeepAnalysis, "anthropic", "claude-opus-4");

        assert_eq!(
            config.get(LlmRole::WebResearch),
            Some(("perplexity", "sonar-pro"))
        );
        assert_eq!(
            config.get(LlmRole::DeepAnalysis),
            Some(("anthropic", "claude-opus-4"))
        );
        assert_eq!(config.get(LlmRole::Code), None);
    }
}
