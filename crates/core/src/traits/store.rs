// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! # ExperienceStore Capability Traits
//!
//! This module defines the capability boundary traits for ExperienceStore
//! (event sourcing). The experience store is an append-only ledger for
//! audit, replay, and provenance.
//!
//! ## Split Trait Pattern
//!
//! ExperienceStore is split by operation type:
//!
//! - [`ExperienceAppender`]: Append-only event storage for governance
//! - [`ExperienceReplayer`]: Streaming replay access for audit/debugging
//!
//! This separation ensures that replay/audit contexts cannot accidentally
//! append events, and that append authority is a hard governance boundary.
//!
//! ## GAT Async Pattern
//!
//! All traits use Generic Associated Types (GATs) for zero-cost async:
//!
//! ```ignore
//! pub trait ExperienceAppender: Send + Sync {
//!     type AppendFut<'a>: Future<Output = Result<(), StoreError>> + Send + 'a
//!     where
//!         Self: 'a;
//!
//!     fn append<'a>(&'a self, events: &'a [ExperienceEventEnvelope]) -> Self::AppendFut<'a>;
//! }
//! ```
//!
//! This enables static dispatch async without `async_trait` proc macros or
//! tokio runtime dependency in converge-core.
//!
//! ## Thread Safety
//!
//! All traits require `Send + Sync` to enable use in concurrent contexts.
//!
//! ## Error Handling
//!
//! [`StoreError`] implements [`CapabilityError`] for generic retry logic.
//! It provides `is_transient()` and `is_retryable()` classification.

use std::future::Future;
use std::time::Duration;

use super::error::{CapabilityError, ErrorCategory};
use crate::context::Context;
use crate::experience_store::{EventQuery, ExperienceEventEnvelope, TimeRange};

// ============================================================================
// Store Error
// ============================================================================

/// Error type for experience store operations.
///
/// All variants implement [`CapabilityError`] for generic error handling.
#[derive(Debug, Clone)]
pub enum StoreError {
    /// Storage backend is temporarily unavailable.
    Unavailable {
        /// Human-readable description.
        message: String,
    },
    /// Event serialization or deserialization failed.
    SerializationFailed {
        /// Human-readable error message.
        message: String,
    },
    /// Event ID already exists (duplicate append).
    Conflict {
        /// The conflicting event ID.
        event_id: String,
    },
    /// Query was malformed or invalid.
    InvalidQuery {
        /// Description of what was invalid.
        message: String,
    },
    /// Authentication with the store backend failed.
    AuthFailed {
        /// Human-readable error message.
        message: String,
    },
    /// Rate limit exceeded; retry after delay.
    RateLimited {
        /// Suggested delay before retry.
        retry_after: Duration,
    },
    /// Operation timed out.
    Timeout {
        /// How long the operation ran before timing out.
        elapsed: Duration,
        /// The configured deadline.
        deadline: Duration,
    },
    /// Event or record not found.
    NotFound {
        /// Description of what was not found.
        message: String,
    },
    /// Invariant violation in event store (should not happen).
    InvariantViolation {
        /// Description of the violated invariant.
        message: String,
    },
    /// Internal error with no specific category.
    Internal {
        /// Human-readable error message.
        message: String,
    },
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable { message } => {
                write!(f, "store unavailable: {}", message)
            }
            Self::SerializationFailed { message } => {
                write!(f, "serialization failed: {}", message)
            }
            Self::Conflict { event_id } => {
                write!(f, "event already exists: {}", event_id)
            }
            Self::InvalidQuery { message } => {
                write!(f, "invalid query: {}", message)
            }
            Self::AuthFailed { message } => {
                write!(f, "store auth failed: {}", message)
            }
            Self::RateLimited { retry_after } => {
                write!(f, "rate limited, retry after {:?}", retry_after)
            }
            Self::Timeout { elapsed, deadline } => {
                write!(
                    f,
                    "store operation timed out after {:?} (deadline: {:?})",
                    elapsed, deadline
                )
            }
            Self::NotFound { message } => {
                write!(f, "not found: {}", message)
            }
            Self::InvariantViolation { message } => {
                write!(f, "invariant violation: {}", message)
            }
            Self::Internal { message } => {
                write!(f, "internal store error: {}", message)
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl CapabilityError for StoreError {
    fn category(&self) -> ErrorCategory {
        match self {
            Self::Unavailable { .. } => ErrorCategory::Unavailable,
            Self::SerializationFailed { .. } => ErrorCategory::InvalidInput,
            Self::Conflict { .. } => ErrorCategory::Conflict,
            Self::InvalidQuery { .. } => ErrorCategory::InvalidInput,
            Self::AuthFailed { .. } => ErrorCategory::Auth,
            Self::RateLimited { .. } => ErrorCategory::RateLimit,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::NotFound { .. } => ErrorCategory::NotFound,
            Self::InvariantViolation { .. } => ErrorCategory::InvariantViolation,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }

    fn is_transient(&self) -> bool {
        match self {
            Self::Unavailable { .. } => true,
            Self::SerializationFailed { .. } => false,
            Self::Conflict { .. } => false,
            Self::InvalidQuery { .. } => false,
            Self::AuthFailed { .. } => false,
            Self::RateLimited { .. } => true,
            Self::Timeout { .. } => true,
            Self::NotFound { .. } => false,
            Self::InvariantViolation { .. } => false,
            Self::Internal { .. } => false,
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            // Transient errors are retryable
            Self::Unavailable { .. } => true,
            Self::RateLimited { .. } => true,
            Self::Timeout { .. } => true,
            // Conflict can be retryable with different event ID or idempotency handling
            Self::Conflict { .. } => true,
            // Non-retryable
            Self::SerializationFailed { .. } => false,
            Self::InvalidQuery { .. } => false,
            Self::AuthFailed { .. } => false,
            Self::NotFound { .. } => false,
            Self::InvariantViolation { .. } => false,
            Self::Internal { .. } => false,
        }
    }

    fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after } => Some(*retry_after),
            _ => None,
        }
    }
}

// ============================================================================
// Replay Cursor (for streaming replay)
// ============================================================================

/// Cursor for streaming event replay.
///
/// This type is returned by [`ExperienceReplayer::replay`] to enable efficient
/// iteration over large event ranges without loading everything into memory.
#[derive(Debug, Clone)]
pub struct ReplayCursor {
    /// Current position in the event stream (opaque token).
    pub position: String,
    /// Whether there are more events after the current batch.
    pub has_more: bool,
}

impl ReplayCursor {
    /// Create a cursor at the beginning of the stream.
    #[must_use]
    pub fn start() -> Self {
        Self {
            position: "".to_string(),
            has_more: true,
        }
    }

    /// Create a cursor with a specific position.
    #[must_use]
    pub fn at(position: impl Into<String>) -> Self {
        Self {
            position: position.into(),
            has_more: true,
        }
    }

    /// Create a cursor indicating end of stream.
    #[must_use]
    pub fn end() -> Self {
        Self {
            position: "".to_string(),
            has_more: false,
        }
    }
}

/// Result of a replay batch operation.
#[derive(Debug, Clone)]
pub struct ReplayBatch {
    /// Events in this batch.
    pub events: Vec<ExperienceEventEnvelope>,
    /// Cursor for fetching the next batch.
    pub cursor: ReplayCursor,
}

// ============================================================================
// Replay Options
// ============================================================================

/// Options for controlling replay behavior.
#[derive(Debug, Clone, Default)]
pub struct ReplayOptions {
    /// Batch size for streaming replay.
    pub batch_size: Option<usize>,
    /// Time range filter.
    pub time_range: Option<TimeRange>,
    /// Tenant filter.
    pub tenant_id: Option<String>,
    /// Correlation ID filter.
    pub correlation_id: Option<String>,
}

impl ReplayOptions {
    /// Create default replay options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set batch size.
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Set time range filter.
    #[must_use]
    pub fn with_time_range(mut self, range: TimeRange) -> Self {
        self.time_range = Some(range);
        self
    }

    /// Set tenant filter.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set correlation filter.
    #[must_use]
    pub fn with_correlation(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Convert to an EventQuery for compatibility with existing code.
    #[must_use]
    pub fn to_event_query(&self) -> EventQuery {
        EventQuery {
            tenant_id: self.tenant_id.clone(),
            time_range: self.time_range.clone(),
            kinds: Vec::new(),
            correlation_id: self.correlation_id.clone(),
            chain_id: None,
            limit: self.batch_size,
        }
    }
}

// ============================================================================
// ExperienceStore Traits
// ============================================================================

/// Append-only event storage capability trait.
///
/// This trait provides append access to the experience store. It is designed
/// for ingestion pipelines and kernel event emission.
///
/// # Authority Boundary
///
/// Append is a governance boundary. Only authorized components should hold
/// references to `ExperienceAppender`. Audit/replay contexts should use only
/// [`ExperienceReplayer`].
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in concurrent contexts.
///
/// # GAT Async Pattern
///
/// The `AppendFut` associated type enables static dispatch async:
///
/// ```ignore
/// impl ExperienceAppender for MyStore {
///     type AppendFut<'a> = impl Future<Output = Result<(), StoreError>> + Send + 'a;
///
///     fn append<'a>(&'a self, events: &'a [ExperienceEventEnvelope]) -> Self::AppendFut<'a> {
///         async move {
///             // ... implementation
///         }
///     }
/// }
/// ```
pub trait ExperienceAppender: Send + Sync {
    /// Future type for append operations.
    type AppendFut<'a>: Future<Output = Result<(), StoreError>> + Send + 'a
    where
        Self: 'a;

    /// Append events to the experience store.
    ///
    /// # Arguments
    ///
    /// * `events` - The events to append (processed in order)
    ///
    /// # Atomicity
    ///
    /// Implementations should provide best-effort atomicity for batches.
    /// If partial failure occurs, the implementation must document which
    /// events were persisted.
    ///
    /// # Idempotency
    ///
    /// Appending an event with a duplicate `event_id` should return
    /// `StoreError::Conflict`. Implementations may choose to make this
    /// idempotent (silently succeed) if documented.
    fn append<'a>(&'a self, events: &'a [ExperienceEventEnvelope]) -> Self::AppendFut<'a>;
}

/// Streaming replay capability trait.
///
/// This trait provides streaming access to the experience store for audit,
/// debugging, and deterministic re-execution.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in concurrent contexts.
///
/// # Streaming Pattern
///
/// The replay method returns batches with cursors for efficient iteration:
///
/// ```ignore
/// let mut cursor = ReplayCursor::start();
/// while cursor.has_more {
///     let batch = replayer.replay(&options, &cursor).await?;
///     for event in batch.events {
///         process(event);
///     }
///     cursor = batch.cursor;
/// }
/// ```
pub trait ExperienceReplayer: Send + Sync {
    /// Future type for replay operations.
    type ReplayFut<'a>: Future<Output = Result<ReplayBatch, StoreError>> + Send + 'a
    where
        Self: 'a;

    /// Future type for query operations (count, exists checks).
    type QueryFut<'a>: Future<Output = Result<Vec<ExperienceEventEnvelope>, StoreError>> + Send + 'a
    where
        Self: 'a;

    /// Replay events from the experience store with streaming cursor.
    ///
    /// # Arguments
    ///
    /// * `options` - Replay options (filters, batch size)
    /// * `cursor` - Current position in the stream (use `ReplayCursor::start()` initially)
    ///
    /// # Returns
    ///
    /// A batch of events and a cursor for the next batch.
    fn replay<'a>(
        &'a self,
        options: &'a ReplayOptions,
        cursor: &'a ReplayCursor,
    ) -> Self::ReplayFut<'a>;

    /// Query events matching criteria (non-streaming, loads all into memory).
    ///
    /// # Arguments
    ///
    /// * `query` - Event query with filters
    ///
    /// # Returns
    ///
    /// All matching events. For large result sets, use [`replay`](Self::replay) instead.
    fn query<'a>(&'a self, query: &'a EventQuery) -> Self::QueryFut<'a>;
}

/// Durable context snapshot storage.
///
/// Applications with state that spans multiple runs need a place to persist
/// and reconstruct the engine context. This trait defines that boundary
/// without prescribing a storage backend.
pub trait ContextStore: Send + Sync {
    /// Future type for loading a context snapshot.
    type LoadFut<'a>: Future<Output = Result<Option<Context>, StoreError>> + Send + 'a
    where
        Self: 'a;

    /// Future type for saving a context snapshot.
    type SaveFut<'a>: Future<Output = Result<(), StoreError>> + Send + 'a
    where
        Self: 'a;

    /// Load the latest snapshot for a run, tenant, or application-defined scope.
    fn load_context<'a>(&'a self, scope_id: &'a str) -> Self::LoadFut<'a>;

    /// Persist the latest snapshot for a run, tenant, or application-defined scope.
    fn save_context<'a>(&'a self, scope_id: &'a str, context: &'a Context) -> Self::SaveFut<'a>;
}

// ============================================================================
// Dyn-Safe Wrappers (for runtime polymorphism)
// ============================================================================

/// Boxed future type for dyn-safe trait objects.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Dyn-safe experience appender trait for runtime polymorphism.
///
/// Use this when you need `dyn ExperienceAppender` (e.g., plugin systems).
/// The cost is one heap allocation per call.
pub trait DynExperienceAppender: Send + Sync {
    /// Append events to the experience store.
    fn append<'a>(
        &'a self,
        events: &'a [ExperienceEventEnvelope],
    ) -> BoxFuture<'a, Result<(), StoreError>>;
}

impl<T: ExperienceAppender> DynExperienceAppender for T {
    fn append<'a>(
        &'a self,
        events: &'a [ExperienceEventEnvelope],
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(ExperienceAppender::append(self, events))
    }
}

/// Dyn-safe experience replayer trait for runtime polymorphism.
pub trait DynExperienceReplayer: Send + Sync {
    /// Replay events from the experience store.
    fn replay<'a>(
        &'a self,
        options: &'a ReplayOptions,
        cursor: &'a ReplayCursor,
    ) -> BoxFuture<'a, Result<ReplayBatch, StoreError>>;

    /// Query events matching criteria.
    fn query<'a>(
        &'a self,
        query: &'a EventQuery,
    ) -> BoxFuture<'a, Result<Vec<ExperienceEventEnvelope>, StoreError>>;
}

impl<T: ExperienceReplayer> DynExperienceReplayer for T {
    fn replay<'a>(
        &'a self,
        options: &'a ReplayOptions,
        cursor: &'a ReplayCursor,
    ) -> BoxFuture<'a, Result<ReplayBatch, StoreError>> {
        Box::pin(ExperienceReplayer::replay(self, options, cursor))
    }

    fn query<'a>(
        &'a self,
        query: &'a EventQuery,
    ) -> BoxFuture<'a, Result<Vec<ExperienceEventEnvelope>, StoreError>> {
        Box::pin(ExperienceReplayer::query(self, query))
    }
}

/// Dyn-safe context store for runtime polymorphism.
pub trait DynContextStore: Send + Sync {
    /// Load a stored context snapshot.
    fn load_context<'a>(
        &'a self,
        scope_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Context>, StoreError>>;

    /// Save a context snapshot.
    fn save_context<'a>(
        &'a self,
        scope_id: &'a str,
        context: &'a Context,
    ) -> BoxFuture<'a, Result<(), StoreError>>;
}

impl<T: ContextStore> DynContextStore for T {
    fn load_context<'a>(
        &'a self,
        scope_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Context>, StoreError>> {
        Box::pin(ContextStore::load_context(self, scope_id))
    }

    fn save_context<'a>(
        &'a self,
        scope_id: &'a str,
        context: &'a Context,
    ) -> BoxFuture<'a, Result<(), StoreError>> {
        Box::pin(ContextStore::save_context(self, scope_id, context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_error_display() {
        let err = StoreError::Conflict {
            event_id: "evt-123".to_string(),
        };
        assert!(err.to_string().contains("evt-123"));
    }

    #[test]
    fn store_error_category_classification() {
        assert_eq!(
            StoreError::Unavailable {
                message: "test".to_string()
            }
            .category(),
            ErrorCategory::Unavailable
        );
        assert_eq!(
            StoreError::Conflict {
                event_id: "test".to_string()
            }
            .category(),
            ErrorCategory::Conflict
        );
        assert_eq!(
            StoreError::InvariantViolation {
                message: "test".to_string()
            }
            .category(),
            ErrorCategory::InvariantViolation
        );
        assert_eq!(
            StoreError::RateLimited {
                retry_after: Duration::from_secs(60)
            }
            .category(),
            ErrorCategory::RateLimit
        );
    }

    #[test]
    fn store_error_transient_classification() {
        assert!(
            StoreError::Unavailable {
                message: "test".to_string()
            }
            .is_transient()
        );
        assert!(
            StoreError::RateLimited {
                retry_after: Duration::from_secs(60)
            }
            .is_transient()
        );
        assert!(
            StoreError::Timeout {
                elapsed: Duration::from_secs(30),
                deadline: Duration::from_secs(30),
            }
            .is_transient()
        );

        assert!(
            !StoreError::Conflict {
                event_id: "test".to_string()
            }
            .is_transient()
        );
        assert!(
            !StoreError::SerializationFailed {
                message: "test".to_string()
            }
            .is_transient()
        );
        assert!(
            !StoreError::InvariantViolation {
                message: "test".to_string()
            }
            .is_transient()
        );
    }

    #[test]
    fn store_error_retryable_classification() {
        // Transient errors are retryable
        assert!(
            StoreError::Unavailable {
                message: "test".to_string()
            }
            .is_retryable()
        );
        assert!(
            StoreError::RateLimited {
                retry_after: Duration::from_secs(60)
            }
            .is_retryable()
        );
        assert!(
            StoreError::Timeout {
                elapsed: Duration::from_secs(30),
                deadline: Duration::from_secs(30),
            }
            .is_retryable()
        );

        // Conflict is retryable (can retry with different ID or idempotency)
        assert!(
            StoreError::Conflict {
                event_id: "test".to_string()
            }
            .is_retryable()
        );

        // Non-retryable
        assert!(
            !StoreError::SerializationFailed {
                message: "test".to_string()
            }
            .is_retryable()
        );
        assert!(
            !StoreError::InvariantViolation {
                message: "test".to_string()
            }
            .is_retryable()
        );
        assert!(
            !StoreError::AuthFailed {
                message: "test".to_string()
            }
            .is_retryable()
        );
    }

    #[test]
    fn store_error_retry_after() {
        let err = StoreError::RateLimited {
            retry_after: Duration::from_secs(60),
        };
        assert_eq!(err.retry_after(), Some(Duration::from_secs(60)));

        let err2 = StoreError::Unavailable {
            message: "test".to_string(),
        };
        assert_eq!(err2.retry_after(), None);
    }

    #[test]
    fn replay_cursor_factories() {
        let start = ReplayCursor::start();
        assert!(start.has_more);
        assert!(start.position.is_empty());

        let at = ReplayCursor::at("pos-123");
        assert!(at.has_more);
        assert_eq!(at.position, "pos-123");

        let end = ReplayCursor::end();
        assert!(!end.has_more);
    }

    #[test]
    fn replay_options_builder() {
        let opts = ReplayOptions::new()
            .with_batch_size(100)
            .with_tenant("tenant-1")
            .with_correlation("corr-1");

        assert_eq!(opts.batch_size, Some(100));
        assert_eq!(opts.tenant_id, Some("tenant-1".to_string()));
        assert_eq!(opts.correlation_id, Some("corr-1".to_string()));
    }

    #[test]
    fn replay_options_to_event_query() {
        let opts = ReplayOptions::new()
            .with_batch_size(50)
            .with_tenant("tenant-2");

        let query = opts.to_event_query();
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.tenant_id, Some("tenant-2".to_string()));
    }
}
