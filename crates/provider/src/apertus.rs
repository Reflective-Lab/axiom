// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Apertus (Switzerland) API provider.
//!
//! Apertus is a multilingual open LLM developed under European digital sovereignty initiatives.

use crate::common::{HttpProviderConfig, OpenAiCompatibleProvider};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// Apertus API provider (Switzerland, EU digital sovereignty).
///
/// # Example
///
/// ```ignore
/// use converge_provider::ApertusProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = ApertusProvider::new(
///     "your-api-key",
///     "apertus-v1"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct ApertusProvider {
    config: HttpProviderConfig,
}

impl ApertusProvider {
    /// Creates a new Apertus provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://api.apertus.ai"),
        }
    }

    /// Creates a provider using the `APERTUS_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("APERTUS_API_KEY")
            .map_err(|_| LlmError::auth("APERTUS_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for ApertusProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/v1/chat/completions"
    }
}

impl LlmProvider for ApertusProvider {
    fn name(&self) -> &'static str {
        "apertus"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_openai_compatible(request)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("apertus:{}:{}", self.config.model, request_id)
    }
}
