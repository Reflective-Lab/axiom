// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
//
// Mock LLM provider for domain testing.
// Extracted from converge-core (removed in 1.0.2) for use in converge-domain tests.

use converge_core::llm::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    responses: Mutex<Vec<MockResponse>>,
    call_count: AtomicUsize,
}

impl MockProvider {
    /// Creates a new mock provider with pre-configured responses.
    #[must_use]
    pub fn new(responses: Vec<MockResponse>) -> Self {
        Self {
            model: "mock-model".into(),
            responses: Mutex::new(responses),
            call_count: AtomicUsize::new(0),
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
        self.call_count.load(Ordering::SeqCst)
    }
}

impl LlmProvider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);

        let mut responses = self
            .responses
            .lock()
            .map_err(|_| LlmError::provider("MockProvider: mutex poisoned"))?;

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
