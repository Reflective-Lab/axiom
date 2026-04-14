// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! HTTP fetch provider — fetches a single URL and returns its content.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::search::{WebFetchBackend, WebFetchError, WebFetchRequest, WebFetchResponse};

/// HTTP-based web fetch provider.
///
/// Uses `reqwest` under the hood (the same HTTP stack as the search providers).
pub struct HttpFetchProvider {
    client: Client,
    user_agent: String,
}

impl std::fmt::Debug for HttpFetchProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpFetchProvider")
            .field("user_agent", &self.user_agent)
            .finish_non_exhaustive()
    }
}

impl Default for HttpFetchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpFetchProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::limited(10))
                .build()
                .expect("failed to build reqwest client"),
            user_agent: format!("converge/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
}

impl WebFetchBackend for HttpFetchProvider {
    fn provider_name(&self) -> &'static str {
        "http"
    }

    fn fetch(&self, request: &WebFetchRequest) -> Result<WebFetchResponse, WebFetchError> {
        let url = reqwest::Url::parse(&request.url)
            .map_err(|e| WebFetchError::InvalidUrl(e.to_string()))?;

        let mut headers = HeaderMap::new();
        for (name, value) in &request.headers {
            let name = HeaderName::try_from(name.as_str())
                .map_err(|e| WebFetchError::Network(format!("invalid header name: {e}")))?;
            let value = HeaderValue::from_str(value)
                .map_err(|e| WebFetchError::Network(format!("invalid header value: {e}")))?;
            headers.insert(name, value);
        }

        let response = self
            .client
            .get(url)
            .timeout(Duration::from_millis(request.timeout_ms))
            .headers(headers)
            .header("User-Agent", &self.user_agent)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    WebFetchError::Timeout(request.timeout_ms)
                } else {
                    WebFetchError::Network(e.to_string())
                }
            })?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let bytes = response
            .bytes()
            .map_err(|e| WebFetchError::Network(e.to_string()))?;

        let truncated = bytes.len() > request.max_bytes;
        let body = if truncated {
            String::from_utf8_lossy(&bytes[..request.max_bytes]).into_owned()
        } else {
            String::from_utf8_lossy(&bytes).into_owned()
        };

        Ok(WebFetchResponse {
            url: final_url,
            status,
            content_type,
            body,
            truncated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_url_returns_error() {
        let provider = HttpFetchProvider::new();
        let request = WebFetchRequest::new("not a url");
        let result = provider.fetch(&request);
        assert!(matches!(result, Err(WebFetchError::InvalidUrl(_))));
    }

    #[test]
    fn default_user_agent_contains_crate_version() {
        let provider = HttpFetchProvider::new();
        assert!(provider.user_agent.starts_with("converge/"));
    }

    #[test]
    fn builder_overrides_user_agent() {
        let provider = HttpFetchProvider::new().with_user_agent("test-agent/1.0");
        assert_eq!(provider.user_agent, "test-agent/1.0");
    }
}
