// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Resilient chat: automatic format and model fallback on failure.
//!
//! Wraps a `DynChatBackend` with retry logic that:
//! 1. On parse/format failure: retries with JSON (native API enforcement)
//! 2. On model error (rate limit, auth, provider error): retries with a fallback backend
//!
//! This is the recommended way to call LLMs when you need structured output.

use std::sync::Arc;

use tracing::{info, warn};

use converge_core::traits::{
    BoxFuture, ChatBackend, ChatRequest, ChatResponse, DynChatBackend, LlmError,
};

/// A chat backend that retries with format and model fallbacks.
///
/// On the first attempt, uses the primary backend with the requested format.
/// If the response fails to parse as the requested format, retries with JSON.
/// If the primary backend errors, falls back to the secondary backend.
pub struct ResilientChatBackend {
    primary: Arc<dyn DynChatBackend>,
    fallback: Option<Arc<dyn DynChatBackend>>,
    primary_label: String,
    fallback_label: String,
}

impl ResilientChatBackend {
    #[must_use]
    pub fn new(primary: Arc<dyn DynChatBackend>, label: impl Into<String>) -> Self {
        Self {
            primary,
            fallback: None,
            primary_label: label.into(),
            fallback_label: String::new(),
        }
    }

    #[must_use]
    pub fn with_fallback(
        mut self,
        fallback: Arc<dyn DynChatBackend>,
        label: impl Into<String>,
    ) -> Self {
        self.fallback = Some(fallback);
        self.fallback_label = label.into();
        self
    }

    async fn chat_async(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        let original_format = req.response_format;

        // Attempt 1: primary backend, requested format
        match self.primary.chat(req.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) if is_retryable_with_format_change(&e) => {
                // Format-related failure — try JSON fallback
                if let Some(fallback_format) = original_format.fallback() {
                    warn!(
                        primary = %self.primary_label,
                        original_format = ?original_format,
                        fallback_format = ?fallback_format,
                        "Format failure, retrying with fallback format"
                    );

                    let mut retry_req = req.clone();
                    retry_req.response_format = fallback_format;

                    if let Ok(response) = self.primary.chat(retry_req).await {
                        return Ok(response);
                    }
                }
            }
            Err(e) if is_retryable_with_model_change(&e) => {
                // Model/provider failure — try fallback backend
                if let Some(fallback) = &self.fallback {
                    warn!(
                        primary = %self.primary_label,
                        fallback = %self.fallback_label,
                        error = %e,
                        "Model failure, retrying with fallback backend"
                    );

                    match fallback.chat(req.clone()).await {
                        Ok(response) => {
                            info!(
                                fallback = %self.fallback_label,
                                "Fallback backend succeeded"
                            );
                            return Ok(response);
                        }
                        Err(fallback_err) => {
                            // Both failed — return the original error
                            warn!(
                                fallback = %self.fallback_label,
                                error = %fallback_err,
                                "Fallback backend also failed"
                            );
                            return Err(e);
                        }
                    }
                }
                return Err(e);
            }
            Err(e) => return Err(e),
        }

        // Should not reach here, but satisfy the compiler
        Err(LlmError::ProviderError {
            message: "unexpected state in resilient backend".to_string(),
            code: None,
        })
    }
}

impl ChatBackend for ResilientChatBackend {
    type ChatFut<'a>
        = BoxFuture<'a, Result<ChatResponse, LlmError>>
    where
        Self: 'a;

    fn chat(&self, req: ChatRequest) -> Self::ChatFut<'_> {
        Box::pin(async move { self.chat_async(req).await })
    }
}

fn is_retryable_with_format_change(error: &LlmError) -> bool {
    matches!(
        error,
        LlmError::InvalidRequest { .. } | LlmError::ContentFiltered { .. }
    )
}

fn is_retryable_with_model_change(error: &LlmError) -> bool {
    matches!(
        error,
        LlmError::RateLimited { .. }
            | LlmError::ProviderError { .. }
            | LlmError::ModelNotFound { .. }
            | LlmError::NetworkError { .. }
            | LlmError::Timeout { .. }
    )
}
