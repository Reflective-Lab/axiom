// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! Deterministic fake provider for testing.
//!
//! This provider returns pre-configured responses based on prompt content,
//! making tests deterministic and reproducible.
//!
//! # Example
//!
//! ```ignore
//! use converge_provider::FakeProvider;
//! use crate::provider_api::{LlmProvider, LlmRequest};
//!
//! let provider = FakeProvider::new()
//!     .with_response("hello", "Hello back!")
//!     .with_response("analyze", "Analysis: looks good");
//!
//! let response = provider.complete(&LlmRequest::new("hello world"))?;
//! assert_eq!(response.content, "Hello back!");
//! ```

use crate::provider_api::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::contract::{CallTimer, ProviderObservation, canonical_hash};

/// A deterministic provider for testing.
///
/// Returns pre-configured responses based on prompt content matching.
/// If no match is found, returns a default response.
#[derive(Debug, Clone)]
pub struct FakeProvider {
    /// Model name
    model: String,
    /// Mapping from prompt substring to response
    responses: HashMap<String, FakeResponse>,
    /// Default response when no match found
    default_response: String,
    /// Simulated latency in milliseconds
    latency_ms: u64,
    /// Call counter for testing
    call_count: std::sync::Arc<AtomicU64>,
    /// Whether to fail all calls
    should_fail: bool,
    /// Error to return when failing
    fail_error: Option<String>,
}

/// A configured fake response.
#[derive(Debug, Clone)]
struct FakeResponse {
    content: String,
    tokens_in: u32,
    tokens_out: u32,
}

impl Default for FakeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeProvider {
    /// Create a new fake provider.
    #[must_use]
    pub fn new() -> Self {
        Self {
            model: "fake-model".to_string(),
            responses: HashMap::new(),
            default_response: "This is a fake response for testing.".to_string(),
            latency_ms: 0,
            call_count: std::sync::Arc::new(AtomicU64::new(0)),
            should_fail: false,
            fail_error: None,
        }
    }

    /// Set the model name.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Add a response for prompts containing the given substring.
    #[must_use]
    pub fn with_response(mut self, prompt_contains: &str, response: &str) -> Self {
        self.responses.insert(
            prompt_contains.to_lowercase(),
            FakeResponse {
                content: response.to_string(),
                tokens_in: 10,
                tokens_out: 20,
            },
        );
        self
    }

    /// Add a response with token counts.
    #[must_use]
    pub fn with_response_tokens(
        mut self,
        prompt_contains: &str,
        response: &str,
        tokens_in: u32,
        tokens_out: u32,
    ) -> Self {
        self.responses.insert(
            prompt_contains.to_lowercase(),
            FakeResponse {
                content: response.to_string(),
                tokens_in,
                tokens_out,
            },
        );
        self
    }

    /// Set the default response when no match is found.
    #[must_use]
    pub fn with_default_response(mut self, response: impl Into<String>) -> Self {
        self.default_response = response.into();
        self
    }

    /// Set simulated latency in milliseconds.
    #[must_use]
    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Configure the provider to fail all calls.
    #[must_use]
    pub fn with_failure(mut self, error_message: &str) -> Self {
        self.should_fail = true;
        self.fail_error = Some(error_message.to_string());
        self
    }

    /// Get the number of calls made to this provider.
    #[must_use]
    pub fn call_count(&self) -> u64 {
        self.call_count.load(Ordering::Relaxed)
    }

    /// Reset the call counter.
    pub fn reset_call_count(&self) {
        self.call_count.store(0, Ordering::Relaxed);
    }

    /// Find matching response for a prompt.
    fn find_response(&self, prompt: &str) -> FakeResponse {
        let prompt_lower = prompt.to_lowercase();
        for (key, response) in &self.responses {
            if prompt_lower.contains(key) {
                return response.clone();
            }
        }
        FakeResponse {
            content: self.default_response.clone(),
            tokens_in: 10,
            tokens_out: 10,
        }
    }
}

impl LlmProvider for FakeProvider {
    fn name(&self) -> &'static str {
        "fake"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let timer = CallTimer::start();

        // Increment call count
        self.call_count.fetch_add(1, Ordering::Relaxed);

        // Simulate latency
        if self.latency_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.latency_ms));
        }

        // Check if configured to fail
        if self.should_fail {
            return Err(LlmError::provider(
                self.fail_error
                    .as_deref()
                    .unwrap_or("Fake provider configured to fail"),
            ));
        }

        // Find matching response
        let fake_response = self.find_response(&request.prompt);

        // Build observation
        let observation = ProviderObservation::new(
            "fake",
            &self.model,
            fake_response.content.clone(),
            timer.elapsed_ms().max(self.latency_ms),
        )
        .with_request_hash(canonical_hash(&request.prompt))
        .with_tokens(fake_response.tokens_in, fake_response.tokens_out)
        .with_cost(0.0); // Fake provider is free

        // Convert to LlmResponse
        Ok(LlmResponse {
            content: observation.content,
            model: self.model.clone(),
            usage: TokenUsage {
                prompt_tokens: fake_response.tokens_in,
                completion_tokens: fake_response.tokens_out,
                total_tokens: fake_response.tokens_in + fake_response.tokens_out,
            },
            finish_reason: FinishReason::Stop,
        })
    }

    fn provenance(&self, request_id: &str) -> String {
        format!("fake:{}:{}", self.model, request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_provider_default() {
        let provider = FakeProvider::new();
        assert_eq!(provider.name(), "fake");
        assert_eq!(provider.model(), "fake-model");
    }

    #[test]
    fn test_fake_provider_custom_model() {
        let provider = FakeProvider::new().with_model("test-model-v1");
        assert_eq!(provider.model(), "test-model-v1");
    }

    #[test]
    fn test_fake_provider_response_matching() {
        let provider = FakeProvider::new()
            .with_response("hello", "Hi there!")
            .with_response("goodbye", "See you later!");

        let request = LlmRequest::new("Say hello to me");
        let response = provider.complete(&request).unwrap();
        assert_eq!(response.content, "Hi there!");

        let request = LlmRequest::new("Time to say goodbye");
        let response = provider.complete(&request).unwrap();
        assert_eq!(response.content, "See you later!");
    }

    #[test]
    fn test_fake_provider_default_response() {
        let provider = FakeProvider::new().with_default_response("Default!");

        let request = LlmRequest::new("Something random");
        let response = provider.complete(&request).unwrap();
        assert_eq!(response.content, "Default!");
    }

    #[test]
    fn test_fake_provider_case_insensitive() {
        let provider = FakeProvider::new().with_response("hello", "Matched!");

        let request = LlmRequest::new("HELLO WORLD");
        let response = provider.complete(&request).unwrap();
        assert_eq!(response.content, "Matched!");
    }

    #[test]
    fn test_fake_provider_call_count() {
        let provider = FakeProvider::new();
        assert_eq!(provider.call_count(), 0);

        provider.complete(&LlmRequest::new("test")).unwrap();
        assert_eq!(provider.call_count(), 1);

        provider.complete(&LlmRequest::new("test")).unwrap();
        assert_eq!(provider.call_count(), 2);

        provider.reset_call_count();
        assert_eq!(provider.call_count(), 0);
    }

    #[test]
    fn test_fake_provider_failure() {
        let provider = FakeProvider::new().with_failure("Test failure");

        let result = provider.complete(&LlmRequest::new("test"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(format!("{error}").contains("Test failure"));
    }

    #[test]
    fn test_fake_provider_tokens() {
        let provider = FakeProvider::new().with_response_tokens("test", "Response", 50, 100);

        let response = provider.complete(&LlmRequest::new("test")).unwrap();
        assert_eq!(response.usage.prompt_tokens, 50);
        assert_eq!(response.usage.completion_tokens, 100);
        assert_eq!(response.usage.total_tokens, 150);
    }

    #[test]
    fn test_fake_provider_provenance() {
        let provider = FakeProvider::new().with_model("test-v1");
        let prov = provider.provenance("req-123");
        assert_eq!(prov, "fake:test-v1:req-123");
    }

    #[test]
    fn test_fake_provider_latency() {
        let provider = FakeProvider::new().with_latency_ms(50);

        let start = std::time::Instant::now();
        provider.complete(&LlmRequest::new("test")).unwrap();
        let elapsed = start.elapsed().as_millis();

        assert!(elapsed >= 50);
    }
}
