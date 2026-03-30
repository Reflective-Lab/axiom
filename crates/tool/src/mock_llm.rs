// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Minimal local LLM mocks used by converge-tool tests and CLI fallback paths.

use std::collections::VecDeque;
use std::sync::Mutex;

use converge_core::llm::{
    FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};

#[derive(Debug)]
pub struct StaticLlmProvider {
    model: String,
    responses: Mutex<VecDeque<String>>,
}

impl StaticLlmProvider {
    #[must_use]
    pub fn constant(content: impl Into<String>) -> Self {
        Self {
            model: "static-mock".to_string(),
            responses: Mutex::new(VecDeque::from([content.into()])),
        }
    }

    #[must_use]
    pub fn queued<I, S>(responses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            model: "static-mock".to_string(),
            responses: Mutex::new(responses.into_iter().map(Into::into).collect()),
        }
    }
}

impl LlmProvider for StaticLlmProvider {
    fn name(&self) -> &'static str {
        "static-mock"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut responses = self
            .responses
            .lock()
            .map_err(|_| LlmError::provider("static mock provider mutex poisoned"))?;

        let content = if responses.len() > 1 {
            responses.pop_front().unwrap_or_default()
        } else {
            responses.front().cloned().unwrap_or_default()
        };

        Ok(LlmResponse {
            content,
            model: self.model.clone(),
            usage: TokenUsage::default(),
            finish_reason: FinishReason::Stop,
        })
    }
}
