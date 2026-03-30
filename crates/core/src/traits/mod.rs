// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Capability Boundary Traits
//!
//! This module defines the abstraction layer for external capabilities that
//! `converge-core` depends on but does not implement. These traits define
//! the interface contract — implementations belong in capability crates like
//! `converge-runtime`.
//!
//! ## Design Philosophy
//!
//! - **converge-core defines interfaces only**: No implementation of these
//!   traits exists in this crate. This keeps the core dependency-free and
//!   focused on correctness axioms.
//!
//! - **Capability crates provide implementations**: `converge-runtime` or
//!   similar crates implement these traits using appropriate libraries
//!   (e.g., rayon for Executor, rand for Randomness, sha2/hex for Fingerprint).
//!
//! - **Thread safety required**: All traits require `Send + Sync` bounds
//!   to ensure safe use in concurrent contexts.
//!
//! ## Traits
//!
//! ### Execution & Infrastructure
//! - [`Executor`]: Abstracts parallel/sequential execution strategy
//! - [`Randomness`]: Abstracts random number generation
//! - [`Fingerprint`]: Abstracts cryptographic hashing and hex encoding
//!
//! ### Error Infrastructure
//! - [`CapabilityError`]: Shared error classification interface
//! - [`ErrorCategory`]: Error category enumeration
//!
//! ### LLM Capabilities
//! - [`ChatBackend`]: Chat completion (GAT async pattern)
//! - [`EmbedBackend`]: Embedding generation (GAT async pattern)
//! - [`LlmBackend`]: Umbrella trait combining chat + embed
//! - [`DynChatBackend`]: Dyn-safe chat wrapper for runtime polymorphism
//! - [`DynEmbedBackend`]: Dyn-safe embed wrapper for runtime polymorphism
//! - [`LlmError`]: LLM operation errors implementing [`CapabilityError`]
//!
//! ### Recall (Semantic Memory) Capabilities
//! - [`RecallReader`]: Query-only read access (validation, audit, replay)
//! - [`RecallWriter`]: Store/delete mutation access (ingestion pipelines)
//! - [`Recall`]: Umbrella trait combining RecallReader + RecallWriter
//! - [`DynRecallReader`]: Dyn-safe recall reader for runtime polymorphism
//! - [`RecallError`]: Recall errors implementing [`CapabilityError`]
//!
//! ### ExperienceStore (Event Sourcing) Capabilities
//! - [`ExperienceAppender`]: Append-only event storage (governance boundary)
//! - [`ExperienceReplayer`]: Streaming replay access (audit, debugging)
//! - [`ContextStore`]: Durable context snapshots across runs
//! - [`DynExperienceAppender`]: Dyn-safe appender for runtime polymorphism
//! - [`DynExperienceReplayer`]: Dyn-safe replayer for runtime polymorphism
//! - [`DynContextStore`]: Dyn-safe context store for runtime polymorphism
//! - [`StoreError`]: Store errors implementing [`CapabilityError`]
//!
//! ### Validation Capabilities (Type-State Aware)
//! - [`Validator`]: Validates `Proposal<Draft>` producing `ValidationReport`
//! - [`DynValidator`]: Dyn-safe validator for runtime polymorphism
//! - [`ValidatorError`]: Validation errors implementing [`CapabilityError`]
//!
//! ### Promotion Capabilities (Type-State Aware)
//! - [`Promoter`]: Promotes `Proposal<Validated>` to `Fact`
//! - [`DynPromoter`]: Dyn-safe promoter for runtime polymorphism
//! - [`PromoterError`]: Promotion errors implementing [`CapabilityError`]
//! - [`PromotionContext`]: Context for promotion operations
//!
//! # Design Tenets Alignment
//!
//! This module directly supports these tenets from [`crate`]:
//!
//! | Tenet | How This Module Supports It |
//! |-------|----------------------------|
//! | **Agents Suggest, Engine Decides** | [`Validator`] and [`Promoter`] are the decision boundary |
//! | **No Hidden Work** | All trait operations are explicit, no background effects |
//! | **Purity Declaration** | Traits only; implementations belong in capability crates |
//! | **Transparent Determinism** | [`Randomness`] abstracted for deterministic testing |
//!
//! # Cross-Module References
//!
//! - **Types**: [`Validator`] validates [`crate::types::Proposal`], [`Promoter`] creates [`crate::types::Fact`]
//! - **Gates**: [`crate::gates::PromotionGate`] uses these traits for validation/promotion lifecycle

// ============================================================================
// Submodules
// ============================================================================

mod error;
mod llm;
mod promoter;
mod recall;
mod store;
mod validator;

// ============================================================================
// Re-exports
// ============================================================================

// Error infrastructure
pub use error::{CapabilityError, ErrorCategory};

// LLM capability traits
pub use llm::{
    BoxFuture, ChatBackend, ChatMessage, ChatRequest, ChatResponse, ChatRole, DynChatBackend,
    DynEmbedBackend, EmbedBackend, EmbedRequest, EmbedResponse, FinishReason, LlmBackend, LlmError,
    TokenUsage,
};

// Recall (semantic memory) capability traits
pub use recall::{
    DynRecallReader, Recall, RecallError, RecallReader, RecallRecord, RecallRecordMetadata,
    RecallWriter,
};

// ExperienceStore (event sourcing) capability traits
pub use store::{
    ContextStore, DynContextStore, DynExperienceAppender, DynExperienceReplayer,
    ExperienceAppender, ExperienceReplayer, ReplayBatch, ReplayCursor, ReplayOptions, StoreError,
};

// Validation capability traits
pub use validator::{DynValidator, Validator, ValidatorError};

// Promotion capability traits
pub use promoter::{DynPromoter, Promoter, PromoterError, PromotionContext};

// ============================================================================
// Inline Traits (Infrastructure)
// ============================================================================

use std::fmt;

/// Error type for fingerprint operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FingerprintError {
    /// The hex string contains invalid characters.
    InvalidHex(String),
    /// The hex string has incorrect length (expected 64 characters for 32 bytes).
    InvalidLength { expected: usize, got: usize },
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FingerprintError::InvalidHex(msg) => write!(f, "invalid hex: {}", msg),
            FingerprintError::InvalidLength { expected, got } => {
                write!(
                    f,
                    "invalid length: expected {} chars, got {}",
                    expected, got
                )
            }
        }
    }
}

impl std::error::Error for FingerprintError {}

/// Abstracts parallel or sequential execution strategy.
///
/// This trait replaces direct usage of rayon's parallel iterators. By abstracting
/// the execution strategy, `converge-core` can remain dependency-free while
/// allowing runtime crates to provide optimized parallel implementations.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow sharing across threads.
///
/// # Example Implementation
///
/// ```ignore
/// // In converge-runtime:
/// use rayon::prelude::*;
///
/// pub struct RayonExecutor;
///
/// impl Executor for RayonExecutor {
///     fn execute_parallel<T, F, R>(&self, items: &[T], f: F) -> Vec<R>
///     where
///         T: Sync,
///         F: Fn(&T) -> R + Send + Sync,
///         R: Send,
///     {
///         items.par_iter().map(f).collect()
///     }
/// }
/// ```
pub trait Executor: Send + Sync {
    /// Execute a function over a slice of items, potentially in parallel.
    ///
    /// The implementation may choose to execute sequentially or in parallel
    /// based on the item count, available cores, or other heuristics.
    ///
    /// # Arguments
    ///
    /// * `items` - The slice of items to process
    /// * `f` - The function to apply to each item
    ///
    /// # Returns
    ///
    /// A vector of results in the same order as the input items.
    fn execute_parallel<T, F, R>(&self, items: &[T], f: F) -> Vec<R>
    where
        T: Sync,
        F: Fn(&T) -> R + Send + Sync,
        R: Send;
}

/// Abstracts random number generation.
///
/// This trait replaces direct usage of the `rand` crate. By abstracting
/// randomness, `converge-core` can remain dependency-free and tests can
/// use deterministic implementations for reproducibility.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow sharing across threads.
///
/// # Example Implementation
///
/// ```ignore
/// // In converge-runtime:
/// use rand::Rng;
/// use std::sync::Mutex;
///
/// pub struct ThreadRng {
///     rng: Mutex<rand::rngs::ThreadRng>,
/// }
///
/// impl Randomness for ThreadRng {
///     fn random_u32(&self) -> u32 {
///         self.rng.lock().unwrap().gen()
///     }
///
///     fn random_bytes(&self, buf: &mut [u8]) {
///         self.rng.lock().unwrap().fill(buf);
///     }
/// }
/// ```
pub trait Randomness: Send + Sync {
    /// Generate a random 32-bit unsigned integer.
    fn random_u32(&self) -> u32;

    /// Fill a buffer with random bytes.
    fn random_bytes(&self, buf: &mut [u8]);
}

/// Abstracts cryptographic fingerprinting (hashing) and hex encoding.
///
/// This trait replaces direct usage of `sha2` and `hex` crates. By abstracting
/// fingerprinting, `converge-core` can remain dependency-free and tests can
/// use mock implementations when needed.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow sharing across threads.
///
/// # Hash Size
///
/// This trait uses 32-byte (256-bit) hashes, compatible with SHA-256.
///
/// # Example Implementation
///
/// ```ignore
/// // In converge-runtime:
/// use sha2::{Sha256, Digest};
///
/// pub struct Sha256Fingerprint;
///
/// impl Fingerprint for Sha256Fingerprint {
///     fn compute(&self, data: &[u8]) -> [u8; 32] {
///         let mut hasher = Sha256::new();
///         hasher.update(data);
///         hasher.finalize().into()
///     }
///
///     fn to_hex(hash: &[u8; 32]) -> String {
///         hex::encode(hash)
///     }
///
///     fn from_hex(s: &str) -> Result<[u8; 32], FingerprintError> {
///         let bytes = hex::decode(s)
///             .map_err(|e| FingerprintError::InvalidHex(e.to_string()))?;
///         bytes.try_into()
///             .map_err(|_| FingerprintError::InvalidLength {
///                 expected: 32,
///                 got: bytes.len(),
///             })
///     }
/// }
/// ```
pub trait Fingerprint: Send + Sync {
    /// Compute a 32-byte cryptographic fingerprint of the input data.
    fn compute(&self, data: &[u8]) -> [u8; 32];

    /// Convert a 32-byte hash to a lowercase hexadecimal string.
    fn to_hex(hash: &[u8; 32]) -> String;

    /// Parse a hexadecimal string into a 32-byte hash.
    ///
    /// # Errors
    ///
    /// Returns `FingerprintError::InvalidHex` if the string contains non-hex characters.
    /// Returns `FingerprintError::InvalidLength` if the string is not exactly 64 characters.
    fn from_hex(s: &str) -> Result<[u8; 32], FingerprintError>;
}
