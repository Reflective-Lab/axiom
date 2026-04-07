// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Validation types for the Gate Pattern.
//!
//! This module provides:
//! - `ValidationToken` - Private ZST for forgery prevention
//! - `CheckResult` - Result of a single validation check
//! - `ValidationReport` - Proof object that validation occurred
//! - `ValidationPolicy` - Policy controlling validation behavior
//! - `ValidationContext` - Context for running validation
//! - `ValidationError` - Error type for validation failures
//!
//! # Key Invariant
//!
//! `ValidationReport::new()` is `pub(crate)` - external code cannot forge reports.
//! The `ValidationToken` field ensures only validators can create reports.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::types::{ContentHash, ProposalId, Timestamp};

// ============================================================================
// ValidationToken - Private ZST for forgery prevention
// ============================================================================

/// Private token preventing ValidationReport forgery.
///
/// Only validators can create this (pub(crate)).
/// This is a zero-sized type (ZST) that adds no runtime overhead.
#[derive(Clone)]
pub(crate) struct ValidationToken(());

impl ValidationToken {
    /// Create a new validation token.
    ///
    /// This is pub(crate) to prevent external code from creating tokens.
    pub(crate) fn new() -> Self {
        Self(())
    }
}

// ============================================================================
// CheckResult - Result of a single validation check
// ============================================================================

/// Result of a single validation check.
///
/// Each check has a name, pass/fail status, and optional message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Name of the check.
    pub name: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Optional message (especially useful for failures).
    pub message: Option<String>,
}

impl CheckResult {
    /// Create a passing check result.
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: None,
        }
    }

    /// Create a failing check result.
    pub fn failed(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: Some(message.into()),
        }
    }

    /// Create a passing check result with a message.
    pub fn passed_with_message(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: Some(message.into()),
        }
    }
}

// ============================================================================
// ValidationReport - Proof object that validation occurred
// ============================================================================

/// Proof object that validation occurred.
///
/// This type can only be created within the crate via `pub(crate) new()`.
/// The private `_token` field ensures external code cannot construct it.
///
/// # Invariants
///
/// - Cannot be constructed outside converge-core
/// - Contains complete validation audit trail
/// - Immutable once created
#[derive(Clone)]
pub struct ValidationReport {
    /// ID of the validated proposal.
    proposal_id: ProposalId,
    /// Results of all validation checks.
    checks: Vec<CheckResult>,
    /// Hash of the policy version used for validation.
    policy_version: ContentHash,
    /// When validation was performed.
    validated_at: Timestamp,
    /// Private token preventing external construction.
    _token: ValidationToken,
}

impl ValidationReport {
    /// Create a new validation report.
    ///
    /// This is `pub(crate)` - only callable by validators within the crate.
    pub(crate) fn new(
        proposal_id: ProposalId,
        checks: Vec<CheckResult>,
        policy_version: ContentHash,
    ) -> Self {
        Self {
            proposal_id,
            checks,
            policy_version,
            validated_at: Timestamp::now(),
            _token: ValidationToken::new(),
        }
    }

    /// Get the proposal ID.
    pub fn proposal_id(&self) -> &ProposalId {
        &self.proposal_id
    }

    /// Get the validation checks.
    pub fn checks(&self) -> &[CheckResult] {
        &self.checks
    }

    /// Get the policy version hash.
    pub fn policy_version(&self) -> &ContentHash {
        &self.policy_version
    }

    /// Get the validation timestamp.
    pub fn validated_at(&self) -> &Timestamp {
        &self.validated_at
    }

    /// Check if all validation checks passed.
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }

    /// Get the names of failed checks.
    pub fn failed_checks(&self) -> Vec<&str> {
        self.checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name.as_str())
            .collect()
    }
}

// Implement Debug manually to avoid exposing ValidationToken
impl std::fmt::Debug for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationReport")
            .field("proposal_id", &self.proposal_id)
            .field("checks", &self.checks)
            .field("policy_version", &self.policy_version)
            .field("validated_at", &self.validated_at)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// ValidationPolicy - Policy controlling validation behavior
// ============================================================================

/// Policy controlling validation behavior.
///
/// Defines which checks are required and how to handle warnings.
#[derive(Debug, Clone, Default)]
pub struct ValidationPolicy {
    /// Names of required validation checks.
    pub required_checks: Vec<String>,
    /// Whether to allow warnings (non-blocking issues).
    pub allow_warnings: bool,
    /// Hash of this policy version (for audit).
    version_hash: ContentHash,
}

impl ValidationPolicy {
    /// Create a new validation policy.
    pub fn new() -> Self {
        Self {
            required_checks: Vec::new(),
            allow_warnings: true,
            version_hash: ContentHash::zero(),
        }
    }

    /// Add a required check.
    pub fn with_required_check(mut self, check: impl Into<String>) -> Self {
        self.required_checks.push(check.into());
        self.update_version_hash();
        self
    }

    /// Set whether warnings are allowed.
    pub fn with_allow_warnings(mut self, allow: bool) -> Self {
        self.allow_warnings = allow;
        self.update_version_hash();
        self
    }

    /// Get the policy version hash.
    pub fn version_hash(&self) -> &ContentHash {
        &self.version_hash
    }

    /// Update the version hash based on policy content.
    fn update_version_hash(&mut self) {
        // Simple FNV-1a based hash (deterministic, no external deps)
        let mut hash = [0u8; 32];
        let mut fnv: u64 = 0xcbf29ce484222325;

        for check in &self.required_checks {
            for byte in check.bytes() {
                fnv ^= byte as u64;
                fnv = fnv.wrapping_mul(0x100000001b3);
            }
        }

        fnv ^= self.allow_warnings as u64;
        fnv = fnv.wrapping_mul(0x100000001b3);

        // Copy fnv into first 8 bytes
        hash[..8].copy_from_slice(&fnv.to_le_bytes());
        self.version_hash = ContentHash::new(hash);
    }
}

// ============================================================================
// ValidationContext - Context for running validation
// ============================================================================

/// Context for running validation.
///
/// Contains metadata about the validation environment.
#[derive(Debug, Clone, Default)]
pub struct ValidationContext {
    /// Optional tenant identifier.
    pub tenant_id: Option<String>,
    /// Optional session identifier.
    pub session_id: Option<String>,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ValidationContext {
    /// Create a new empty validation context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the tenant ID.
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant.into());
        self
    }

    /// Set the session ID.
    pub fn with_session(mut self, session: impl Into<String>) -> Self {
        self.session_id = Some(session.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

// ============================================================================
// ValidationError - Error type for validation failures
// ============================================================================

/// Error type for validation failures.
#[derive(Debug, Clone, Error)]
pub enum ValidationError {
    /// A validation check failed.
    #[error("check '{name}' failed: {reason}")]
    CheckFailed {
        /// Name of the failed check.
        name: String,
        /// Reason for failure.
        reason: String,
    },

    /// Policy was violated.
    #[error("policy violation: {0}")]
    PolicyViolation(String),

    /// A required check was missing.
    #[error("missing required check: {0}")]
    MissingCheck(String),

    /// Invalid input to validation.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl ValidationError {
    /// Create a check failed error.
    pub fn check_failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CheckFailed {
            name: name.into(),
            reason: reason.into(),
        }
    }

    /// Create a policy violation error.
    pub fn policy_violation(message: impl Into<String>) -> Self {
        Self::PolicyViolation(message.into())
    }

    /// Create a missing check error.
    pub fn missing_check(check: impl Into<String>) -> Self {
        Self::MissingCheck(check.into())
    }

    /// Create an invalid input error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_result_passed() {
        let check = CheckResult::passed("schema_valid");
        assert!(check.passed);
        assert_eq!(check.name, "schema_valid");
        assert!(check.message.is_none());
    }

    #[test]
    fn check_result_failed() {
        let check =
            CheckResult::failed("confidence_threshold", "confidence 0.3 below threshold 0.5");
        assert!(!check.passed);
        assert_eq!(check.name, "confidence_threshold");
        assert_eq!(
            check.message,
            Some("confidence 0.3 below threshold 0.5".to_string())
        );
    }

    #[test]
    fn validation_report_creation() {
        let report = ValidationReport::new(
            ProposalId::new("prop-001"),
            vec![
                CheckResult::passed("check_1"),
                CheckResult::passed("check_2"),
            ],
            ContentHash::zero(),
        );

        assert_eq!(report.proposal_id().as_str(), "prop-001");
        assert_eq!(report.checks().len(), 2);
        assert!(report.all_passed());
        assert!(report.failed_checks().is_empty());
    }

    #[test]
    fn validation_report_with_failures() {
        let report = ValidationReport::new(
            ProposalId::new("prop-002"),
            vec![
                CheckResult::passed("check_1"),
                CheckResult::failed("check_2", "too low"),
            ],
            ContentHash::zero(),
        );

        assert!(!report.all_passed());
        assert_eq!(report.failed_checks(), vec!["check_2"]);
    }

    #[test]
    fn validation_policy_builder() {
        let policy = ValidationPolicy::new()
            .with_required_check("schema_valid")
            .with_required_check("confidence_threshold")
            .with_allow_warnings(false);

        assert_eq!(policy.required_checks.len(), 2);
        assert!(!policy.allow_warnings);
        // Version hash should be non-zero after modifications
        assert_ne!(policy.version_hash(), &ContentHash::zero());
    }

    #[test]
    fn validation_context_builder() {
        let ctx = ValidationContext::new()
            .with_tenant("tenant-123")
            .with_session("session-456")
            .with_metadata("custom_key", serde_json::json!({"value": 42}));

        assert_eq!(ctx.tenant_id, Some("tenant-123".to_string()));
        assert_eq!(ctx.session_id, Some("session-456".to_string()));
        assert!(ctx.metadata.contains_key("custom_key"));
    }

    #[test]
    fn validation_error_display() {
        let err = ValidationError::check_failed("schema_valid", "missing required field");
        assert_eq!(
            err.to_string(),
            "check 'schema_valid' failed: missing required field"
        );

        let err = ValidationError::policy_violation("too many warnings");
        assert_eq!(err.to_string(), "policy violation: too many warnings");

        let err = ValidationError::missing_check("human_review");
        assert_eq!(err.to_string(), "missing required check: human_review");
    }

    #[test]
    fn validation_report_debug() {
        let report = ValidationReport::new(
            ProposalId::new("prop-003"),
            vec![CheckResult::passed("test")],
            ContentHash::zero(),
        );

        // Debug should work but not expose ValidationToken
        let debug = format!("{:?}", report);
        assert!(debug.contains("ValidationReport"));
        assert!(debug.contains("prop-003"));
        assert!(!debug.contains("_token"));
    }

    // Note: ValidationReport cannot be constructed outside the crate.
    // This is enforced at compile-time by pub(crate) visibility:
    //
    // // In external crate:
    // let report = ValidationReport::new(...);
    // // ERROR: associated function `new` is private
}
