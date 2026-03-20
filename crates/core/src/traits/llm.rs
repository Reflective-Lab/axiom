// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! # LLM Capability Boundary Traits
//!
//! This module defines the capability boundary traits for Large Language Model
//! operations. These traits abstract LLM functionality, allowing `converge-core`
//! to remain dependency-free while capability crates provide implementations.
//!
//! ## Design Philosophy
//!
//! - **Split traits by capability:** [`ChatBackend`] for chat completions,
//!   [`EmbedBackend`] for embeddings. Many providers support only subsets,
//!   so split traits make boundaries explicit and enforceable.
//!
//! - **GAT async pattern:** Uses generic associated types (GATs) for zero-cost
//!   async without proc macros or `async_trait`. This keeps the core dependency-free.
//!
//! - **Dyn-safe wrappers:** [`DynChatBackend`] and [`DynEmbedBackend`] provide
//!   `dyn Trait` compatibility when runtime polymorphism is needed.
//!
//! - **Thread safety required:** All traits require `Send + Sync` bounds for
//!   multi-threaded runtimes.
//!
//! ## Trait Hierarchy
//!
//! ```text
//! ChatBackend (GAT async)          EmbedBackend (GAT async)
//!      |                                |
//!      +------- LlmBackend -------------+  (umbrella trait)
//!
//! DynChatBackend (BoxFuture)       DynEmbedBackend (BoxFuture)
//!      |                                |
//!      +------ (for dyn Trait) ---------+
//! ```
//!
//! ## Error Handling
//!
//! [`LlmError`] implements [`CapabilityError`](super::error::CapabilityError) for
//! uniform error classification. This enables generic retry/circuit breaker logic.

use super::error::{CapabilityError, ErrorCategory};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Boxed future type for dyn-safe trait variants.
///
/// Used by [`DynChatBackend`] and [`DynEmbedBackend`] for runtime polymorphism.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request for chat completion.
///
/// Contains the conversation messages and generation parameters.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// The conversation messages (system, user, assistant).
    pub messages: Vec<ChatMessage>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative).
    pub temperature: Option<f32>,
    /// Model identifier (e.g., "gpt-4", "claude-3-opus").
    pub model: Option<String>,
}

/// A single message in a chat conversation.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Role of the message sender.
    pub role: ChatRole,
    /// Content of the message.
    pub content: String,
}

/// Role of a chat message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    /// System instructions.
    System,
    /// User input.
    User,
    /// Assistant (model) response.
    Assistant,
}

/// Response from chat completion.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// Generated message content.
    pub content: String,
    /// Token usage statistics.
    pub usage: Option<TokenUsage>,
    /// Model that generated the response.
    pub model: Option<String>,
    /// Reason the generation stopped.
    pub finish_reason: Option<FinishReason>,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy)]
pub struct TokenUsage {
    /// Tokens in the prompt.
    pub prompt_tokens: u32,
    /// Tokens in the completion.
    pub completion_tokens: u32,
    /// Total tokens used.
    pub total_tokens: u32,
}

/// Reason generation stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    /// Natural completion.
    Stop,
    /// Hit max_tokens limit.
    Length,
    /// Content filter triggered.
    ContentFilter,
    /// Tool/function call requested.
    ToolCalls,
}

/// Request for embedding generation.
#[derive(Debug, Clone)]
pub struct EmbedRequest {
    /// Input text(s) to embed.
    pub inputs: Vec<String>,
    /// Model identifier (e.g., "text-embedding-3-small").
    pub model: Option<String>,
    /// Embedding dimensions (for models that support variable dimensions).
    pub dimensions: Option<u32>,
}

/// Response from embedding generation.
#[derive(Debug, Clone)]
pub struct EmbedResponse {
    /// Generated embeddings (one per input).
    pub embeddings: Vec<Vec<f32>>,
    /// Token usage statistics.
    pub usage: Option<TokenUsage>,
    /// Model that generated the embeddings.
    pub model: Option<String>,
}

// ============================================================================
// Error Type
// ============================================================================

/// Error type for LLM operations.
///
/// Implements [`CapabilityError`] for uniform error classification.
#[derive(Debug, Clone)]
pub enum LlmError {
    /// Rate limited by provider.
    RateLimited {
        /// Suggested delay before retry.
        retry_after: Duration,
        /// Provider's rate limit message.
        message: Option<String>,
    },
    /// Operation timed out.
    Timeout {
        /// Time elapsed before timeout.
        elapsed: Duration,
        /// Configured deadline.
        deadline: Duration,
    },
    /// Authentication or authorization denied.
    AuthDenied {
        /// Error message from provider.
        message: String,
    },
    /// Invalid request parameters.
    InvalidRequest {
        /// Description of what's invalid.
        message: String,
    },
    /// Model not found or unavailable.
    ModelNotFound {
        /// Requested model identifier.
        model: String,
    },
    /// Context length exceeded.
    ContextLengthExceeded {
        /// Maximum allowed tokens.
        max_tokens: u32,
        /// Tokens in the request.
        request_tokens: u32,
    },
    /// Content filter triggered.
    ContentFiltered {
        /// Reason for filtering.
        reason: String,
    },
    /// Provider returned an error.
    ProviderError {
        /// Error message from provider.
        message: String,
        /// Provider-specific error code.
        code: Option<String>,
    },
    /// Network or connectivity error.
    NetworkError {
        /// Error description.
        message: String,
    },
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited {
                retry_after,
                message,
            } => {
                write!(f, "rate limited (retry after {:?})", retry_after)?;
                if let Some(msg) = message {
                    write!(f, ": {}", msg)?;
                }
                Ok(())
            }
            Self::Timeout { elapsed, deadline } => {
                write!(f, "timeout after {:?} (deadline: {:?})", elapsed, deadline)
            }
            Self::AuthDenied { message } => write!(f, "authentication denied: {}", message),
            Self::InvalidRequest { message } => write!(f, "invalid request: {}", message),
            Self::ModelNotFound { model } => write!(f, "model not found: {}", model),
            Self::ContextLengthExceeded {
                max_tokens,
                request_tokens,
            } => {
                write!(
                    f,
                    "context length exceeded: {} tokens (max: {})",
                    request_tokens, max_tokens
                )
            }
            Self::ContentFiltered { reason } => write!(f, "content filtered: {}", reason),
            Self::ProviderError { message, code } => {
                write!(f, "provider error: {}", message)?;
                if let Some(c) = code {
                    write!(f, " (code: {})", c)?;
                }
                Ok(())
            }
            Self::NetworkError { message } => write!(f, "network error: {}", message),
        }
    }
}

impl std::error::Error for LlmError {}

impl CapabilityError for LlmError {
    fn category(&self) -> ErrorCategory {
        match self {
            Self::RateLimited { .. } => ErrorCategory::RateLimit,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::AuthDenied { .. } => ErrorCategory::Auth,
            Self::InvalidRequest { .. } => ErrorCategory::InvalidInput,
            Self::ModelNotFound { .. } => ErrorCategory::NotFound,
            Self::ContextLengthExceeded { .. } => ErrorCategory::InvalidInput,
            Self::ContentFiltered { .. } => ErrorCategory::InvalidInput,
            Self::ProviderError { .. } => ErrorCategory::Internal,
            Self::NetworkError { .. } => ErrorCategory::Unavailable,
        }
    }

    fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. } | Self::Timeout { .. } | Self::NetworkError { .. }
        )
    }

    fn is_retryable(&self) -> bool {
        // Transient errors are retryable
        // ProviderError may also be retryable (temporary backend issues)
        self.is_transient() || matches!(self, Self::ProviderError { .. })
    }

    fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }
}

// ============================================================================
// Static Dispatch Traits (GAT Async Pattern)
// ============================================================================

/// Chat completion capability.
///
/// Provides chat completion functionality using the GAT async pattern for
/// zero-cost static dispatch without runtime overhead.
///
/// # Example Implementation
///
/// ```ignore
/// struct OpenAIChatBackend { /* ... */ }
///
/// impl ChatBackend for OpenAIChatBackend {
///     type ChatFut<'a> = impl Future<Output = Result<ChatResponse, LlmError>> + Send + 'a
///     where
///         Self: 'a;
///
///     fn chat<'a>(&'a self, req: ChatRequest) -> Self::ChatFut<'a> {
///         async move {
///             // Make API call...
///             Ok(ChatResponse { /* ... */ })
///         }
///     }
/// }
/// ```
pub trait ChatBackend: Send + Sync {
    /// Associated future type for chat completion.
    ///
    /// Must be `Send` to work with multi-threaded runtimes.
    type ChatFut<'a>: Future<Output = Result<ChatResponse, LlmError>> + Send + 'a
    where
        Self: 'a;

    /// Send a chat completion request.
    ///
    /// # Arguments
    ///
    /// * `req` - The chat request containing messages and parameters.
    ///
    /// # Returns
    ///
    /// A future that resolves to the chat response or an error.
    fn chat<'a>(&'a self, req: ChatRequest) -> Self::ChatFut<'a>;
}

/// Embedding generation capability.
///
/// Provides embedding generation functionality using the GAT async pattern for
/// zero-cost static dispatch without runtime overhead.
///
/// # Example Implementation
///
/// ```ignore
/// struct OpenAIEmbedBackend { /* ... */ }
///
/// impl EmbedBackend for OpenAIEmbedBackend {
///     type EmbedFut<'a> = impl Future<Output = Result<EmbedResponse, LlmError>> + Send + 'a
///     where
///         Self: 'a;
///
///     fn embed<'a>(&'a self, req: EmbedRequest) -> Self::EmbedFut<'a> {
///         async move {
///             // Make API call...
///             Ok(EmbedResponse { /* ... */ })
///         }
///     }
/// }
/// ```
pub trait EmbedBackend: Send + Sync {
    /// Associated future type for embedding generation.
    ///
    /// Must be `Send` to work with multi-threaded runtimes.
    type EmbedFut<'a>: Future<Output = Result<EmbedResponse, LlmError>> + Send + 'a
    where
        Self: 'a;

    /// Generate embeddings for input text(s).
    ///
    /// # Arguments
    ///
    /// * `req` - The embedding request containing inputs and parameters.
    ///
    /// # Returns
    ///
    /// A future that resolves to the embedding response or an error.
    fn embed<'a>(&'a self, req: EmbedRequest) -> Self::EmbedFut<'a>;
}

/// Umbrella trait combining chat and embedding capabilities.
///
/// Provides a convenience trait for backends that support both chat completions
/// and embeddings. This is an optional umbrella—implementations can choose to
/// implement only [`ChatBackend`] or [`EmbedBackend`] if they don't support both.
///
/// # Automatic Implementation
///
/// Any type implementing both [`ChatBackend`] and [`EmbedBackend`] automatically
/// implements [`LlmBackend`] via a blanket impl.
pub trait LlmBackend: ChatBackend + EmbedBackend {}

// Blanket implementation: any type with both traits gets LlmBackend
impl<T: ChatBackend + EmbedBackend> LlmBackend for T {}

// ============================================================================
// Dyn-Safe Wrappers (Runtime Polymorphism)
// ============================================================================

/// Dyn-safe chat backend for runtime polymorphism.
///
/// Use this trait when you need `dyn Trait` compatibility, such as:
/// - Storing multiple backend types in a collection
/// - Runtime routing between different providers
/// - Plugin systems with dynamic loading
///
/// For static dispatch (better performance, no allocation), use [`ChatBackend`].
///
/// # Blanket Implementation
///
/// Any type implementing [`ChatBackend`] automatically implements [`DynChatBackend`]
/// via a blanket impl that boxes the future.
pub trait DynChatBackend: Send + Sync {
    /// Send a chat completion request.
    ///
    /// Returns a boxed future for dyn-safety.
    fn chat(&self, req: ChatRequest) -> BoxFuture<'_, Result<ChatResponse, LlmError>>;
}

// Blanket implementation: ChatBackend -> DynChatBackend
impl<T: ChatBackend> DynChatBackend for T {
    fn chat(&self, req: ChatRequest) -> BoxFuture<'_, Result<ChatResponse, LlmError>> {
        Box::pin(ChatBackend::chat(self, req))
    }
}

/// Dyn-safe embed backend for runtime polymorphism.
///
/// Use this trait when you need `dyn Trait` compatibility, such as:
/// - Storing multiple backend types in a collection
/// - Runtime routing between different providers
/// - Plugin systems with dynamic loading
///
/// For static dispatch (better performance, no allocation), use [`EmbedBackend`].
///
/// # Blanket Implementation
///
/// Any type implementing [`EmbedBackend`] automatically implements [`DynEmbedBackend`]
/// via a blanket impl that boxes the future.
pub trait DynEmbedBackend: Send + Sync {
    /// Generate embeddings for input text(s).
    ///
    /// Returns a boxed future for dyn-safety.
    fn embed(&self, req: EmbedRequest) -> BoxFuture<'_, Result<EmbedResponse, LlmError>>;
}

// Blanket implementation: EmbedBackend -> DynEmbedBackend
impl<T: EmbedBackend> DynEmbedBackend for T {
    fn embed(&self, req: EmbedRequest) -> BoxFuture<'_, Result<EmbedResponse, LlmError>> {
        Box::pin(EmbedBackend::embed(self, req))
    }
}
