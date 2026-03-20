// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! `DeepSeek` AI API provider.

use crate::common::{HttpProviderConfig, OpenAiCompatibleProvider};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// `DeepSeek` AI API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::DeepSeekProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = DeepSeekProvider::new(
///     "your-api-key",
///     "deepseek-chat"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct DeepSeekProvider {
    config: HttpProviderConfig,
}

impl DeepSeekProvider {
    /// Creates a new `DeepSeek` provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://api.deepseek.com"),
        }
    }

    /// Creates a provider using the `DEEPSEEK_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| LlmError::auth("DEEPSEEK_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for DeepSeekProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/v1/chat/completions"
    }
}

impl LlmProvider for DeepSeekProvider {
    fn name(&self) -> &'static str {
        "deepseek"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_openai_compatible(request)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("deepseek:{}:{}", self.config.model, request_id)
    }
}
