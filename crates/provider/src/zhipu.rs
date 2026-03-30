// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Zhipu GLM API provider.

use crate::common::{HttpProviderConfig, OpenAiCompatibleProvider};
use crate::provider_api::{LlmError, LlmProvider, LlmRequest, LlmResponse};

/// Zhipu GLM API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::ZhipuProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = ZhipuProvider::new(
///     "your-api-key",
///     "glm-4"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct ZhipuProvider {
    config: HttpProviderConfig,
}

impl ZhipuProvider {
    /// Creates a new Zhipu provider.
    #[must_use]
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            config: HttpProviderConfig::new(api_key, model, "https://open.bigmodel.cn"),
        }
    }

    /// Creates a provider using the `ZHIPU_API_KEY` environment variable.
    ///
    /// # Errors
    ///
    /// Returns error if the environment variable is not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .map_err(|_| LlmError::auth("ZHIPU_API_KEY environment variable not set"))?;
        Ok(Self::new(api_key, model))
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }
}

impl OpenAiCompatibleProvider for ZhipuProvider {
    fn config(&self) -> &HttpProviderConfig {
        &self.config
    }

    fn endpoint(&self) -> &'static str {
        "/api/paas/v4/chat/completions"
    }
}

impl LlmProvider for ZhipuProvider {
    fn name(&self) -> &'static str {
        "zhipu"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.complete_openai_compatible(request)
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("zhipu:{}:{}", self.config.model, request_id)
    }
}
