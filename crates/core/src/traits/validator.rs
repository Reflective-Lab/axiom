// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! # Validator Capability Boundary Trait
//!
//! This module defines the capability boundary trait for proposal validation.
//! Validators examine `Proposal<Draft>` and produce `ValidationReport` evidence
//! that validation occurred.
//!
//! ## Design Philosophy
//!
//! - **Type-state enforcement:** Works with `Proposal<Draft>` from the type-state
//!   pattern established in Phase 4. Validators only accept draft proposals.
//!
//! - **Proof production:** Validators produce `ValidationReport` which serves as
//!   cryptographic proof that validation occurred. Reports cannot be forged.
//!
//! - **GAT async pattern:** Uses generic associated types for zero-cost async
//!   without proc macros or `async_trait`. Keeps core dependency-free.
//!
//! - **Split from promotion:** Validation and promotion are separate capabilities.
//!   A validator validates; a promoter promotes. This allows different authorization
//!   boundaries and audit trails.
//!
//! ## Integration with Gate Pattern
//!
//! The `Validator` trait abstracts the validation capability that `PromotionGate`
//! uses internally. This allows:
//! - Swapping validation implementations (rule-based, ML-based, hybrid)
//! - Testing with mock validators
//! - Distributed validation across services
//!
//! ## Error Handling
//!
//! [`ValidatorError`] implements [`CapabilityError`](super::error::CapabilityError)
//! for uniform error classification, enabling generic retry/circuit breaker logic.

use super::error::{CapabilityError, ErrorCategory};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use crate::gates::validation::{ValidationPolicy, ValidationReport};
use crate::types::{Draft, Proposal};

/// Boxed future type for dyn-safe trait variant.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// ============================================================================
// Error Type
// ============================================================================

/// Error type for validation operations.
///
/// Implements [`CapabilityError`] for uniform error classification.
#[derive(Debug, Clone)]
pub enum ValidatorError {
    /// Validation check failed.
    CheckFailed {
        /// Name of the failed check.
        check_name: String,
        /// Reason for failure.
        reason: String,
    },
    /// Policy violation detected.
    PolicyViolation {
        /// Policy that was violated.
        policy: String,
        /// Description of violation.
        message: String,
    },
    /// Required evidence missing.
    MissingEvidence {
        /// What evidence was expected.
        expected: String,
    },
    /// Validator service unavailable.
    Unavailable {
        /// Error message.
        message: String,
    },
    /// Operation timed out.
    Timeout {
        /// Time elapsed before timeout.
        elapsed: Duration,
        /// Configured deadline.
        deadline: Duration,
    },
    /// Internal validator error.
    Internal {
        /// Error message.
        message: String,
    },
}

impl std::fmt::Display for ValidatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CheckFailed { check_name, reason } => {
                write!(f, "validation check '{}' failed: {}", check_name, reason)
            }
            Self::PolicyViolation { policy, message } => {
                write!(f, "policy '{}' violated: {}", policy, message)
            }
            Self::MissingEvidence { expected } => {
                write!(f, "missing required evidence: {}", expected)
            }
            Self::Unavailable { message } => write!(f, "validator unavailable: {}", message),
            Self::Timeout { elapsed, deadline } => {
                write!(
                    f,
                    "validation timeout after {:?} (deadline: {:?})",
                    elapsed, deadline
                )
            }
            Self::Internal { message } => write!(f, "internal validator error: {}", message),
        }
    }
}

impl std::error::Error for ValidatorError {}

impl CapabilityError for ValidatorError {
    fn category(&self) -> ErrorCategory {
        match self {
            Self::CheckFailed { .. } => ErrorCategory::InvalidInput,
            Self::PolicyViolation { .. } => ErrorCategory::InvalidInput,
            Self::MissingEvidence { .. } => ErrorCategory::InvalidInput,
            Self::Unavailable { .. } => ErrorCategory::Unavailable,
            Self::Timeout { .. } => ErrorCategory::Timeout,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }

    fn is_transient(&self) -> bool {
        matches!(self, Self::Unavailable { .. } | Self::Timeout { .. })
    }

    fn is_retryable(&self) -> bool {
        // Transient errors are retryable
        // Internal errors may also be retryable (temporary service issues)
        self.is_transient() || matches!(self, Self::Internal { .. })
    }

    fn retry_after(&self) -> Option<Duration> {
        // No specific retry-after for validation errors
        None
    }
}

// ============================================================================
// Static Dispatch Trait (GAT Async Pattern)
// ============================================================================

/// Proposal validation capability.
///
/// Validates `Proposal<Draft>` and produces `ValidationReport` as proof.
/// This trait uses the GAT async pattern for zero-cost static dispatch.
///
/// # Type-State Integration
///
/// Works with the type-state pattern established in Phase 4:
/// - Input: `Proposal<Draft>` - publicly constructible
/// - Output: `ValidationReport` - proof that validation occurred
///
/// The report can then be used by a `Promoter` to create `Proposal<Validated>`.
///
/// # Example Implementation
///
/// ```ignore
/// struct RuleBasedValidator {
///     rules: Vec<ValidationRule>,
/// }
///
/// impl Validator for RuleBasedValidator {
///     type ValidateFut<'a> = impl Future<Output = Result<ValidationReport, ValidatorError>> + Send + 'a
///     where
///         Self: 'a;
///
///     fn validate<'a>(
///         &'a self,
///         proposal: &'a Proposal<Draft>,
///         policy: &'a ValidationPolicy,
///     ) -> Self::ValidateFut<'a> {
///         async move {
///             // Run rules against proposal...
///             Ok(report)
///         }
///     }
/// }
/// ```
pub trait Validator: Send + Sync {
    /// Associated future type for validation.
    ///
    /// Must be `Send` to work with multi-threaded runtimes.
    type ValidateFut<'a>: Future<Output = Result<ValidationReport, ValidatorError>> + Send + 'a
    where
        Self: 'a;

    /// Validate a draft proposal against the given policy.
    ///
    /// # Arguments
    ///
    /// * `proposal` - The draft proposal to validate.
    /// * `policy` - The validation policy to apply.
    ///
    /// # Returns
    ///
    /// A future that resolves to the validation report or an error.
    /// The report serves as proof that validation occurred.
    fn validate<'a>(
        &'a self,
        proposal: &'a Proposal<Draft>,
        policy: &'a ValidationPolicy,
    ) -> Self::ValidateFut<'a>;
}

// ============================================================================
// Dyn-Safe Wrapper (Runtime Polymorphism)
// ============================================================================

/// Dyn-safe validator for runtime polymorphism.
///
/// Use this trait when you need `dyn Trait` compatibility, such as:
/// - Storing multiple validator types in a collection
/// - Runtime routing between different validation strategies
/// - Plugin systems with dynamic loading
///
/// For static dispatch (better performance, no allocation), use [`Validator`].
///
/// # Blanket Implementation
///
/// Any type implementing [`Validator`] automatically implements [`DynValidator`]
/// via a blanket impl that boxes the future.
pub trait DynValidator: Send + Sync {
    /// Validate a draft proposal against the given policy.
    ///
    /// Returns a boxed future for dyn-safety.
    fn validate<'a>(
        &'a self,
        proposal: &'a Proposal<Draft>,
        policy: &'a ValidationPolicy,
    ) -> BoxFuture<'a, Result<ValidationReport, ValidatorError>>;
}

// Blanket implementation: Validator -> DynValidator
impl<T: Validator> DynValidator for T {
    fn validate<'a>(
        &'a self,
        proposal: &'a Proposal<Draft>,
        policy: &'a ValidationPolicy,
    ) -> BoxFuture<'a, Result<ValidationReport, ValidatorError>> {
        Box::pin(Validator::validate(self, proposal, policy))
    }
}
