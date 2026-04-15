// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Minimal local LLM mocks used by converge-axiom tests and CLI fallback paths.

use std::collections::VecDeque;
use std::future::Ready;
use std::sync::Mutex;

use converge_core::traits::{ChatBackend, ChatRequest, ChatResponse, LlmError};

#[derive(Debug)]
pub struct StaticChatBackend {
    responses: Mutex<VecDeque<String>>,
}

impl StaticChatBackend {
    #[must_use]
    pub fn constant(content: impl Into<String>) -> Self {
        Self {
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
            responses: Mutex::new(responses.into_iter().map(Into::into).collect()),
        }
    }
}

impl ChatBackend for StaticChatBackend {
    type ChatFut<'a> = Ready<Result<ChatResponse, LlmError>>;

    fn chat<'a>(&'a self, _req: ChatRequest) -> Self::ChatFut<'a> {
        let result = (|| {
            let mut responses = self.responses.lock().map_err(|_| LlmError::ProviderError {
                message: "static mock backend mutex poisoned".into(),
                code: None,
            })?;

            let content = if responses.len() > 1 {
                responses.pop_front().unwrap_or_default()
            } else {
                responses.front().cloned().unwrap_or_default()
            };

            Ok(ChatResponse {
                content,
                tool_calls: Vec::new(),
                usage: None,
                model: None,
                finish_reason: None,
            })
        })();
        std::future::ready(result)
    }
}
