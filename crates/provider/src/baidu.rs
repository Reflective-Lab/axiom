// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Baidu ERNIE API provider.

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use crate::secret::{EnvSecretProvider, SecretProvider, SecretString};
use serde::{Deserialize, Serialize};

/// Baidu ERNIE API provider.
///
/// # Example
///
/// ```ignore
/// use converge_provider::BaiduProvider;
/// use crate::provider_api::{LlmProvider, LlmRequest};
///
/// let provider = BaiduProvider::new(
///     "your-api-key",
///     "your-secret-key",
///     "ernie-bot"
/// );
///
/// let request = LlmRequest::new("What is 2+2?");
/// let response = provider.complete(&request)?;
/// println!("{}", response.content);
/// ```
pub struct BaiduProvider {
    api_key: SecretString,
    secret_key: SecretString,
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
    access_token: Option<String>,
}

impl BaiduProvider {
    /// Creates a new Baidu provider.
    #[must_use]
    pub fn new(
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            secret_key: SecretString::new(secret_key),
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://aip.baidubce.com".into(),
            access_token: None,
        }
    }

    /// Creates a provider using environment variables.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set.
    pub fn from_env(model: impl Into<String>) -> Result<Self, LlmError> {
        Self::from_secret_provider(&EnvSecretProvider, model)
    }

    /// Creates a provider by loading keys from a `SecretProvider`.
    ///
    /// # Errors
    ///
    /// Returns error if secrets cannot be loaded.
    pub fn from_secret_provider(
        secrets: &dyn SecretProvider,
        model: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let api_key = secrets
            .get_secret("BAIDU_API_KEY")
            .map_err(|e| LlmError::auth(format!("BAIDU_API_KEY: {e}")))?;
        let secret_key = secrets
            .get_secret("BAIDU_SECRET_KEY")
            .map_err(|e| LlmError::auth(format!("BAIDU_SECRET_KEY: {e}")))?;
        Ok(Self {
            api_key,
            secret_key,
            model: model.into(),
            client: reqwest::blocking::Client::new(),
            base_url: "https://aip.baidubce.com".into(),
            access_token: None,
        })
    }

    /// Uses a custom base URL (for testing or proxies).
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Gets or refreshes the access token.
    fn get_access_token(&mut self) -> Result<String, LlmError> {
        if let Some(ref token) = self.access_token {
            return Ok(token.clone());
        }

        let url = format!(
            "{}/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
            self.base_url,
            self.api_key.expose(),
            self.secret_key.expose()
        );

        #[derive(Deserialize)]
        #[allow(clippy::items_after_statements)] // Local struct for response parsing
        struct TokenResponse {
            access_token: String,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| LlmError::network(format!("Token request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(LlmError::auth("Failed to get access token"));
        }

        let token_response: TokenResponse = response
            .json()
            .map_err(|e| LlmError::auth(format!("Failed to parse token response: {e}")))?;

        self.access_token = Some(token_response.access_token.clone());
        Ok(token_response.access_token)
    }
}

#[derive(Serialize)]
struct BaiduRequest {
    messages: Vec<BaiduMessage>,
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Serialize)]
struct BaiduMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct BaiduResponse {
    result: String,
    #[allow(dead_code)]
    error_code: Option<u32>,
    #[allow(dead_code)]
    error_msg: Option<String>,
}

impl LlmProvider for BaiduProvider {
    fn name(&self) -> &'static str {
        "baidu"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        // Note: This is a simplified implementation.
        // Baidu ERNIE API requires access token management.
        // For production, implement proper token caching and refresh.

        // Create a mutable copy to manage access token
        let mut temp_provider = BaiduProvider {
            api_key: self.api_key.clone(),
            secret_key: self.secret_key.clone(),
            model: self.model.clone(),
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            access_token: self.access_token.clone(),
        };

        let access_token = temp_provider.get_access_token()?;

        let mut messages = Vec::new();
        if let Some(system) = &request.system {
            messages.push(BaiduMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }
        messages.push(BaiduMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        });

        let api_request = BaiduRequest {
            messages,
            temperature: request.temperature,
            max_output_tokens: Some(request.max_tokens),
        };

        let url = format!(
            "{}/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{}?access_token={}",
            self.base_url, self.model, access_token
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&api_request)
            .send()
            .map_err(|e| LlmError::network(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_else(|_| format!("HTTP {status}"));
            return Err(LlmError::provider(format!(
                "Baidu API error ({status}): {error_text}"
            )));
        }

        let baidu_response: BaiduResponse = response
            .json()
            .map_err(|e| LlmError::provider(format!("Failed to parse response: {e}")))?;

        if let Some(error_code) = baidu_response.error_code {
            return Err(LlmError::provider(format!(
                "Baidu API error: {}",
                baidu_response
                    .error_msg
                    .unwrap_or_else(|| format!("Error code: {error_code}"))
            )));
        }

        Ok(LlmResponse {
            content: baidu_response.result,
            model: self.model.clone(),
            finish_reason: FinishReason::Stop,
            usage: TokenUsage {
                prompt_tokens: 0, // Baidu API doesn't always provide token usage
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("baidu:{}:{}", self.model, request_id)
    }
}
