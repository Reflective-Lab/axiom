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

use crate::common::{ChatCompletionRequest, ChatCompletionResponse, chat_response_to_llm_response};
use crate::model_selection::ModelMetadata;
use crate::provider_api::{
    CostClass, DataSovereignty, LlmError, LlmProvider, LlmRequest, LlmResponse,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};

/// Declares what a Kong AI Gateway route provides.
///
/// Kong is a gateway, not a model. Each route maps to an upstream model
/// (Claude, GPT-4, Gemini, etc.). This config tells the model selector
/// what capabilities the route exposes so it can match against
/// `AgentRequirements`.
///
/// # Example
///
/// ```ignore
/// use converge_provider::kong::{KongRoute, KongProvider};
/// use converge_provider::provider_api::CostClass;
///
/// // Route backed by Claude Sonnet
/// let route = KongRoute::new("claude-route")
///     .upstream("anthropic", "claude-sonnet-4-6")
///     .cost(CostClass::Low)
///     .latency_ms(3000)
///     .quality(0.93)
///     .reasoning(true)
///     .tool_use(true);
///
/// let provider = KongProvider::new("https://kong.example.com", "key", route);
/// ```
#[derive(Debug, Clone)]
pub struct KongRoute {
    /// Route name (used as model identifier in the provider).
    pub name: String,
    /// Upstream provider name (e.g., "anthropic", "openai").
    pub upstream_provider: String,
    /// Upstream model name (e.g., "claude-sonnet-4-6").
    pub upstream_model: String,
    /// Cost class for this route.
    pub cost_class: CostClass,
    /// Typical latency in milliseconds (includes gateway overhead).
    pub typical_latency_ms: u32,
    /// Quality score (0.0-1.0).
    pub quality: f64,
    /// Whether the upstream model supports reasoning.
    pub has_reasoning: bool,
    /// Whether the upstream model supports tool/function calling.
    pub supports_tool_use: bool,
    /// Whether the upstream model supports vision.
    pub supports_vision: bool,
    /// Whether the upstream model supports structured output.
    pub supports_structured_output: bool,
    /// Whether the upstream model supports code generation.
    pub supports_code: bool,
    /// Whether the upstream model supports web search.
    pub supports_web_search: bool,
    /// Whether the upstream model supports multilingual content.
    pub supports_multilingual: bool,
    /// Context window in tokens.
    pub context_tokens: usize,
    /// Data sovereignty (the gateway's region, not the upstream model's).
    pub data_sovereignty: DataSovereignty,
}

impl KongRoute {
    /// Creates a new Kong route with defaults (no capabilities declared).
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            upstream_provider: "unknown".into(),
            upstream_model: "unknown".into(),
            cost_class: CostClass::Medium,
            typical_latency_ms: 5000,
            quality: 0.80,
            has_reasoning: false,
            supports_tool_use: false,
            supports_vision: false,
            supports_structured_output: false,
            supports_code: false,
            supports_web_search: false,
            supports_multilingual: false,
            context_tokens: 128_000,
            data_sovereignty: DataSovereignty::Any,
        }
    }

    /// Sets the upstream model behind this route.
    #[must_use]
    pub fn upstream(mut self, provider: impl Into<String>, model: impl Into<String>) -> Self {
        self.upstream_provider = provider.into();
        self.upstream_model = model.into();
        self
    }

    #[must_use]
    pub fn cost(mut self, cost: CostClass) -> Self {
        self.cost_class = cost;
        self
    }

    #[must_use]
    pub fn latency_ms(mut self, ms: u32) -> Self {
        self.typical_latency_ms = ms;
        self
    }

    #[must_use]
    pub fn quality(mut self, q: f64) -> Self {
        self.quality = q;
        self
    }

    #[must_use]
    pub fn reasoning(mut self, v: bool) -> Self {
        self.has_reasoning = v;
        self
    }

    #[must_use]
    pub fn tool_use(mut self, v: bool) -> Self {
        self.supports_tool_use = v;
        self
    }

    #[must_use]
    pub fn vision(mut self, v: bool) -> Self {
        self.supports_vision = v;
        self
    }

    #[must_use]
    pub fn structured_output(mut self, v: bool) -> Self {
        self.supports_structured_output = v;
        self
    }

    #[must_use]
    pub fn code(mut self, v: bool) -> Self {
        self.supports_code = v;
        self
    }

    #[must_use]
    pub fn web_search(mut self, v: bool) -> Self {
        self.supports_web_search = v;
        self
    }

    #[must_use]
    pub fn multilingual(mut self, v: bool) -> Self {
        self.supports_multilingual = v;
        self
    }

    #[must_use]
    pub fn context_tokens(mut self, tokens: usize) -> Self {
        self.context_tokens = tokens;
        self
    }

    #[must_use]
    pub fn sovereignty(mut self, ds: DataSovereignty) -> Self {
        self.data_sovereignty = ds;
        self
    }

    /// Converts this route into a `ModelMetadata` entry for the model selector.
    #[must_use]
    pub fn to_model_metadata(&self) -> ModelMetadata {
        ModelMetadata::new(
            "kong",
            &self.name,
            self.cost_class,
            self.typical_latency_ms,
            self.quality,
        )
        .with_reasoning(self.has_reasoning)
        .with_tool_use(self.supports_tool_use)
        .with_vision(self.supports_vision)
        .with_structured_output(self.supports_structured_output)
        .with_code(self.supports_code)
        .with_web_search(self.supports_web_search)
        .with_multilingual(self.supports_multilingual)
        .with_context_tokens(self.context_tokens)
        .with_data_sovereignty(self.data_sovereignty)
    }
}

/// Kong AI Gateway provider.
///
/// Routes LLM calls through Kong AI Gateway. The `KongRoute` describes
/// what the route provides, so the model selector can match against
/// `AgentRequirements`.
///
/// # Example
///
/// ```ignore
/// use converge_provider::kong::{KongRoute, KongProvider};
/// use converge_provider::provider_api::{LlmProvider, LlmRequest, CostClass};
///
/// let route = KongRoute::new("claude-route")
///     .upstream("anthropic", "claude-sonnet-4-6")
///     .cost(CostClass::Low)
///     .quality(0.93)
///     .reasoning(true);
///
/// let provider = KongProvider::new("https://kong.example.com", "key", route);
/// let response = provider.complete(&LlmRequest::new("Analyze this"))?;
/// ```
pub struct KongProvider {
    gateway_url: String,
    api_key: SecretString,
    route: KongRoute,
    client: reqwest::blocking::Client,
}

impl KongProvider {
    /// Creates a new Kong provider.
    #[must_use]
    pub fn new(
        gateway_url: impl Into<String>,
        api_key: impl Into<String>,
        route: KongRoute,
    ) -> Self {
        Self {
            gateway_url: gateway_url.into(),
            api_key: SecretString::new(api_key),
            route,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Creates a provider from environment variables.
    ///
    /// Reads `KONG_AI_GATEWAY_URL` and `KONG_API_KEY` from env.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set.
    pub fn from_env(route: KongRoute) -> Result<Self, LlmError> {
        Self::from_secret_provider(&EnvSecretProvider, route)
    }

    /// Creates a provider by loading secrets from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns error if secrets cannot be loaded.
    pub fn from_secret_provider(
        secrets: &dyn SecretProvider,
        route: KongRoute,
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
            route,
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Uses a custom HTTP client.
    #[must_use]
    pub fn with_client(mut self, client: reqwest::blocking::Client) -> Self {
        self.client = client;
        self
    }

    /// Returns the route configuration.
    #[must_use]
    pub fn route(&self) -> &KongRoute {
        &self.route
    }

    /// Returns model metadata suitable for registering in a `ProviderRegistry`.
    #[must_use]
    pub fn model_metadata(&self) -> ModelMetadata {
        self.route.to_model_metadata()
    }
}

impl LlmProvider for KongProvider {
    fn name(&self) -> &'static str {
        "kong"
    }

    fn model(&self) -> &str {
        &self.route.name
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/ai/chat/completions", self.gateway_url);
        let body = ChatCompletionRequest::from_llm_request(self.route.name.clone(), request);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Kong request failed: {e}")))?;

        if !response.status().is_success() {
            let code = response.status().as_u16();
            let text = response.text().unwrap_or_default();
            return Err(if code == 429 {
                LlmError::rate_limit(text)
            } else {
                LlmError::provider(format!("Kong returned {code}: {text}"))
            });
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .map_err(|e| LlmError::parse(format!("Failed to parse Kong response: {e}")))?;

        chat_response_to_llm_response(chat_response)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!(
            "kong:{}:{}:{}",
            self.route.name, self.route.upstream_provider, request_id
        )
    }
}

// ---------------------------------------------------------------------------
// Gateway — single configuration point for all Kong-routed access
// ---------------------------------------------------------------------------

/// A configured Kong AI Gateway instance.
///
/// Applications create one `KongGateway` and use it to build LLM providers,
/// MCP clients, and HTTP clients — all pre-configured with the gateway URL
/// and API key.
///
/// # Example
///
/// ```ignore
/// use converge_provider::kong::{KongGateway, KongRoute};
/// use converge_provider::provider_api::CostClass;
///
/// let gateway = KongGateway::from_env()?;
///
/// // LLM access
/// let llm = gateway.llm_provider(
///     KongRoute::new("claude-route")
///         .upstream("anthropic", "claude-sonnet-4-6")
///         .cost(CostClass::Low)
///         .reasoning(true)
/// );
///
/// // MCP tool access (vendor registry behind Kong)
/// let mcp_url = gateway.mcp_url("vendor-registry");
/// let api_url = gateway.api_url("vendors/v1");
/// let headers = gateway.auth_headers();
/// ```
pub struct KongGateway {
    gateway_url: String,
    api_key: SecretString,
    client: reqwest::blocking::Client,
}

impl KongGateway {
    /// Creates a new gateway configuration.
    #[must_use]
    pub fn new(gateway_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            gateway_url: gateway_url.into(),
            api_key: SecretString::new(api_key),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Creates a gateway from environment variables.
    ///
    /// Reads `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set.
    pub fn from_env() -> Result<Self, LlmError> {
        Self::from_secret_provider(&EnvSecretProvider)
    }

    /// Creates a gateway from a secret provider.
    ///
    /// # Errors
    ///
    /// Returns error if secrets cannot be loaded.
    pub fn from_secret_provider(secrets: &dyn SecretProvider) -> Result<Self, LlmError> {
        let gateway_url = secrets
            .get_secret("KONG_AI_GATEWAY_URL")
            .map_err(|e| LlmError::auth(format!("KONG_AI_GATEWAY_URL: {e}")))?;
        let api_key = secrets
            .get_secret("KONG_API_KEY")
            .map_err(|e| LlmError::auth(format!("KONG_API_KEY: {e}")))?;
        Ok(Self {
            gateway_url: gateway_url.expose().to_string(),
            api_key,
            client: reqwest::blocking::Client::new(),
        })
    }

    /// Creates an LLM provider for a specific Kong route.
    #[must_use]
    pub fn llm_provider(&self, route: KongRoute) -> KongProvider {
        KongProvider {
            gateway_url: self.gateway_url.clone(),
            api_key: self.api_key.clone(),
            route,
            client: self.client.clone(),
        }
    }

    /// Returns the full URL for an MCP server exposed through Kong.
    ///
    /// Use this with `McpClient::new(name, McpTransport::Http { url, .. })`.
    #[must_use]
    pub fn mcp_url(&self, service_name: &str) -> String {
        format!("{}/mcp/{service_name}", self.gateway_url)
    }

    /// Returns the full URL for a REST API exposed through Kong.
    #[must_use]
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/{path}", self.gateway_url)
    }

    /// Returns the authentication header pair for Kong requests.
    ///
    /// Use this when building custom HTTP clients that go through Kong.
    #[must_use]
    pub fn auth_header(&self) -> (&'static str, String) {
        ("x-api-key", self.api_key.expose().to_string())
    }

    /// Returns the gateway base URL.
    #[must_use]
    pub fn gateway_url(&self) -> &str {
        &self.gateway_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kong_provider_name_and_model() {
        let route = KongRoute::new("gpt-4-route")
            .upstream("openai", "gpt-4")
            .cost(CostClass::Medium)
            .quality(0.90)
            .reasoning(true);
        let provider = KongProvider::new("https://kong.example.com", "test-key", route);
        assert_eq!(provider.name(), "kong");
        assert_eq!(provider.model(), "gpt-4-route");
    }

    #[test]
    fn gateway_builds_urls() {
        let gw = KongGateway::new("https://kong.example.com", "key");
        assert_eq!(
            gw.mcp_url("vendor-registry"),
            "https://kong.example.com/mcp/vendor-registry"
        );
        assert_eq!(
            gw.api_url("vendors/v1/list"),
            "https://kong.example.com/vendors/v1/list"
        );
        assert_eq!(gw.auth_header().0, "x-api-key");
    }

    #[test]
    fn gateway_creates_llm_provider() {
        let gw = KongGateway::new("https://kong.example.com", "key");
        let route = KongRoute::new("test-route").cost(CostClass::Low);
        let provider = gw.llm_provider(route);
        assert_eq!(provider.name(), "kong");
        assert_eq!(provider.model(), "test-route");
    }

    #[test]
    fn route_produces_model_metadata() {
        let route = KongRoute::new("claude-route")
            .upstream("anthropic", "claude-sonnet-4-6")
            .cost(CostClass::Low)
            .latency_ms(3000)
            .quality(0.93)
            .reasoning(true)
            .tool_use(true)
            .vision(true)
            .context_tokens(200_000);

        let metadata = route.to_model_metadata();
        assert_eq!(metadata.provider, "kong");
        assert_eq!(metadata.model, "claude-route");
        assert_eq!(metadata.cost_class, CostClass::Low);
        assert!(metadata.has_reasoning);
        assert!(metadata.supports_tool_use);
        assert!(metadata.supports_vision);
        assert_eq!(metadata.context_tokens, 200_000);
    }
}
