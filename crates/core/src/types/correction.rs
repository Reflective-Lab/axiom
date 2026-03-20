// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Correction types for append-only fact corrections.
//!
//! Per CONTEXT.md:
//! - Facts are immutable - corrections create new Facts that supersede old ones
//! - "Current truth" = latest promoted Fact not superseded within scope
//!
//! # CorrectionEvent
//!
//! Event recording that one Fact supersedes another:
//! ```ignore
//! CorrectionEvent {
//!     new_fact_id,
//!     supersedes_fact_id,
//!     reason_code,
//!     reason_text,
//!     scope,
//!     actor,
//!     policy_version,
//!     timestamp
//! }
//! ```

use serde::{Deserialize, Serialize};

use super::id::{ContentHash, FactId, Timestamp};
use super::provenance::Actor;

// ============================================================================
// CorrectionReason - Why a correction was made
// ============================================================================

/// Why a correction was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionReason {
    /// Original data was incorrect.
    DataError,
    /// Policy changed, requiring re-evaluation.
    PolicyChange,
    /// Source retracted original claim.
    SourceRetraction,
    /// Human explicitly overrode.
    ManualOverride,
    /// System reconciliation process.
    SystemReconciliation,
}

impl Default for CorrectionReason {
    fn default() -> Self {
        Self::DataError
    }
}

// ============================================================================
// CorrectionScope - Where the correction applies
// ============================================================================

/// Scope of the correction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionScope {
    /// Applies globally across all tenants/sessions.
    Global,
    /// Applies to a specific tenant.
    Tenant(String),
    /// Applies to a specific session.
    Session(String),
}

impl Default for CorrectionScope {
    fn default() -> Self {
        Self::Global
    }
}

impl CorrectionScope {
    /// Create a global scope.
    pub fn global() -> Self {
        Self::Global
    }

    /// Create a tenant-scoped correction.
    pub fn tenant(tenant_id: impl Into<String>) -> Self {
        Self::Tenant(tenant_id.into())
    }

    /// Create a session-scoped correction.
    pub fn session(session_id: impl Into<String>) -> Self {
        Self::Session(session_id.into())
    }

    /// Check if this is a global scope.
    pub fn is_global(&self) -> bool {
        matches!(self, Self::Global)
    }
}

// ============================================================================
// CorrectionEvent - Event recording fact supersession
// ============================================================================

/// Event recording that one Fact supersedes another.
///
/// Facts are immutable - corrections create new Facts that supersede old ones.
/// "Current truth" = latest promoted Fact not superseded within scope.
///
/// # Example
///
/// ```
/// use converge_core::types::{
///     CorrectionEvent, CorrectionReason, CorrectionScope,
///     FactId, ContentHash, Actor,
/// };
///
/// let correction = CorrectionEvent::new(
///     FactId::new("fact-v2"),
///     FactId::new("fact-v1"),
///     CorrectionReason::DataError,
///     "Original data source was outdated",
///     CorrectionScope::global(),
///     Actor::human("reviewer@example.com"),
///     ContentHash::zero(),
/// );
///
/// assert_eq!(correction.new_fact_id.as_str(), "fact-v2");
/// assert_eq!(correction.supersedes_fact_id.as_str(), "fact-v1");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionEvent {
    /// ID of the new fact that supersedes the old one.
    pub new_fact_id: FactId,
    /// ID of the fact being superseded.
    pub supersedes_fact_id: FactId,
    /// Structured reason code.
    pub reason_code: CorrectionReason,
    /// Human-readable explanation.
    pub reason_text: String,
    /// Where this correction applies.
    pub scope: CorrectionScope,
    /// Who made the correction.
    pub actor: Actor,
    /// Hash of the policy version governing this correction.
    pub policy_version: ContentHash,
    /// When the correction was made.
    pub timestamp: Timestamp,
}

impl CorrectionEvent {
    /// Create a new correction event.
    pub fn new(
        new_fact_id: FactId,
        supersedes_fact_id: FactId,
        reason_code: CorrectionReason,
        reason_text: impl Into<String>,
        scope: CorrectionScope,
        actor: Actor,
        policy_version: ContentHash,
    ) -> Self {
        Self {
            new_fact_id,
            supersedes_fact_id,
            reason_code,
            reason_text: reason_text.into(),
            scope,
            actor,
            policy_version,
            timestamp: Timestamp::now(),
        }
    }

    /// Create a data error correction.
    pub fn data_error(
        new_fact_id: FactId,
        supersedes_fact_id: FactId,
        reason_text: impl Into<String>,
        actor: Actor,
        policy_version: ContentHash,
    ) -> Self {
        Self::new(
            new_fact_id,
            supersedes_fact_id,
            CorrectionReason::DataError,
            reason_text,
            CorrectionScope::Global,
            actor,
            policy_version,
        )
    }

    /// Create a manual override correction.
    pub fn manual_override(
        new_fact_id: FactId,
        supersedes_fact_id: FactId,
        reason_text: impl Into<String>,
        actor: Actor,
        policy_version: ContentHash,
    ) -> Self {
        Self::new(
            new_fact_id,
            supersedes_fact_id,
            CorrectionReason::ManualOverride,
            reason_text,
            CorrectionScope::Global,
            actor,
            policy_version,
        )
    }

    /// Check if this correction applies globally.
    pub fn is_global(&self) -> bool {
        self.scope.is_global()
    }

    /// Check if this correction applies to a specific tenant.
    pub fn applies_to_tenant(&self, tenant_id: &str) -> bool {
        match &self.scope {
            CorrectionScope::Global => true,
            CorrectionScope::Tenant(id) => id == tenant_id,
            CorrectionScope::Session(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correction_reason_default() {
        let reason = CorrectionReason::default();
        assert!(matches!(reason, CorrectionReason::DataError));
    }

    #[test]
    fn correction_scope_helpers() {
        let global = CorrectionScope::global();
        assert!(global.is_global());

        let tenant = CorrectionScope::tenant("tenant-123");
        assert!(!tenant.is_global());
        assert!(matches!(tenant, CorrectionScope::Tenant(_)));
    }

    #[test]
    fn correction_event_creation() {
        let correction = CorrectionEvent::new(
            FactId::new("fact-v2"),
            FactId::new("fact-v1"),
            CorrectionReason::PolicyChange,
            "Policy updated to require additional validation",
            CorrectionScope::global(),
            Actor::system("policy-engine"),
            ContentHash::zero(),
        );

        assert_eq!(correction.new_fact_id.as_str(), "fact-v2");
        assert_eq!(correction.supersedes_fact_id.as_str(), "fact-v1");
        assert!(matches!(
            correction.reason_code,
            CorrectionReason::PolicyChange
        ));
        assert!(correction.is_global());
    }

    #[test]
    fn correction_tenant_scope() {
        let correction = CorrectionEvent::new(
            FactId::new("fact-v2"),
            FactId::new("fact-v1"),
            CorrectionReason::ManualOverride,
            "Customer-specific correction",
            CorrectionScope::tenant("acme-corp"),
            Actor::human("admin@acme.com"),
            ContentHash::zero(),
        );

        assert!(!correction.is_global());
        assert!(correction.applies_to_tenant("acme-corp"));
        assert!(!correction.applies_to_tenant("other-corp"));
    }

    #[test]
    fn correction_helper_constructors() {
        let data_error = CorrectionEvent::data_error(
            FactId::new("new"),
            FactId::new("old"),
            "Bad data",
            Actor::system("validator"),
            ContentHash::zero(),
        );
        assert!(matches!(
            data_error.reason_code,
            CorrectionReason::DataError
        ));

        let override_correction = CorrectionEvent::manual_override(
            FactId::new("new"),
            FactId::new("old"),
            "User override",
            Actor::human("user"),
            ContentHash::zero(),
        );
        assert!(matches!(
            override_correction.reason_code,
            CorrectionReason::ManualOverride
        ));
    }

    #[test]
    fn correction_serialization() {
        let correction = CorrectionEvent::new(
            FactId::new("fact-v2"),
            FactId::new("fact-v1"),
            CorrectionReason::SourceRetraction,
            "Source retracted claim",
            CorrectionScope::global(),
            Actor::agent("correction-agent"),
            ContentHash::zero(),
        );

        let json = serde_json::to_string(&correction).unwrap();
        assert!(json.contains("\"new_fact_id\":\"fact-v2\""));
        assert!(json.contains("\"supersedes_fact_id\":\"fact-v1\""));
        assert!(json.contains("\"reason_code\":\"SourceRetraction\""));

        let deserialized: CorrectionEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.new_fact_id.as_str(), "fact-v2");
    }
}
