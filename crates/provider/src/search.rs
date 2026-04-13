// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Generic web search request/response types for search-capable providers.

use serde::{Deserialize, Serialize};

/// Error type for web search operations.
#[derive(Debug, thiserror::Error)]
pub enum WebSearchError {
    /// Network/HTTP failure.
    #[error("network error: {0}")]
    Network(String),
    /// Authentication failure.
    #[error("authentication error: {0}")]
    Auth(String),
    /// Rate limit exceeded.
    #[error("rate limit exceeded: {0}")]
    RateLimit(String),
    /// Response parsing failure.
    #[error("parse error: {0}")]
    Parse(String),
    /// Provider-specific API failure.
    #[error("api error: {0}")]
    Api(String),
}

/// Search topic hint for providers that support topic routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchTopic {
    #[default]
    General,
    News,
    Finance,
}

/// Search depth hint for providers that expose relevance vs. latency tradeoffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchDepth {
    #[default]
    Basic,
    Advanced,
    Fast,
    UltraFast,
}

/// Provider-agnostic web search request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebSearchRequest {
    /// Query text.
    pub query: String,
    /// Maximum results to return.
    pub max_results: Option<u32>,
    /// Country bias.
    pub country: Option<String>,
    /// Language bias.
    pub language: Option<String>,
    /// Relative freshness or time range hint.
    pub time_range: Option<String>,
    /// Topic/category hint.
    pub topic: SearchTopic,
    /// Search depth / quality hint.
    pub search_depth: SearchDepth,
    /// Whether to include an answer summary if supported.
    pub include_answer: bool,
    /// Whether to include raw/extended content if supported.
    pub include_raw_content: bool,
    /// Whether to include images if supported.
    pub include_images: bool,
    /// Whether to include favicon URLs if supported.
    pub include_favicon: bool,
    /// Optional allowlist of domains.
    pub include_domains: Vec<String>,
    /// Optional denylist of domains.
    pub exclude_domains: Vec<String>,
}

impl WebSearchRequest {
    /// Create a new web search request.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Self::default()
        }
    }

    /// Set the maximum number of results.
    #[must_use]
    pub fn with_max_results(mut self, max_results: u32) -> Self {
        self.max_results = Some(max_results);
        self
    }

    /// Set the country bias.
    #[must_use]
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Set the language bias.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the time range or freshness hint.
    #[must_use]
    pub fn with_time_range(mut self, time_range: impl Into<String>) -> Self {
        self.time_range = Some(time_range.into());
        self
    }

    /// Set the topic/category.
    #[must_use]
    pub fn with_topic(mut self, topic: SearchTopic) -> Self {
        self.topic = topic;
        self
    }

    /// Set the depth/latency tradeoff.
    #[must_use]
    pub fn with_search_depth(mut self, search_depth: SearchDepth) -> Self {
        self.search_depth = search_depth;
        self
    }

    /// Include an answer summary if supported.
    #[must_use]
    pub fn with_answer(mut self, include: bool) -> Self {
        self.include_answer = include;
        self
    }

    /// Include raw content if supported.
    #[must_use]
    pub fn with_raw_content(mut self, include: bool) -> Self {
        self.include_raw_content = include;
        self
    }

    /// Include image results if supported.
    #[must_use]
    pub fn with_images(mut self, include: bool) -> Self {
        self.include_images = include;
        self
    }

    /// Include favicon URLs if supported.
    #[must_use]
    pub fn with_favicon(mut self, include: bool) -> Self {
        self.include_favicon = include;
        self
    }

    /// Restrict search to the given domains.
    #[must_use]
    pub fn with_include_domains(mut self, domains: Vec<String>) -> Self {
        self.include_domains = domains;
        self
    }

    /// Exclude the given domains.
    #[must_use]
    pub fn with_exclude_domains(mut self, domains: Vec<String>) -> Self {
        self.exclude_domains = domains;
        self
    }
}

/// Generic image result metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebSearchImage {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Generic text result metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
}

/// Generic web search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResponse {
    pub provider: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    pub results: Vec<WebSearchResult>,
    pub images: Vec<WebSearchImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time: Option<f64>,
}

/// Executable contract for provider-local web search adapters.
pub trait WebSearchBackend: Send + Sync {
    /// Stable provider identifier.
    fn provider_name(&self) -> &'static str;

    /// Execute a search request.
    fn search_web(&self, request: &WebSearchRequest) -> Result<WebSearchResponse, WebSearchError>;
}
