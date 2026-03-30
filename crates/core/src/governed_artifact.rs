// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Governed artifact lifecycle management.
//!
//! This module provides types for managing the lifecycle of governed artifacts
//! in the Converge platform. A governed artifact is anything that can change
//! execution outcomes and therefore requires:
//!
//! - Explicit lifecycle states with audit trails
//! - Approval workflows before production use
//! - Rollback capability with impact tracking
//! - Replay integrity verification
//!
//! # Governed Artifact Examples
//!
//! - LoRA adapters (model behavior modification)
//! - Prompt/contract versions
//! - Recall corpora snapshots
//! - Domain packs / eval packs
//! - Any "thing that can change outcomes"
//!
//! # Axiom Compliance
//!
//! - **No Hidden State**: Lifecycle state is explicit and auditable
//! - **Safety by Construction**: Invalid state transitions are rejected
//! - **Explicit Authority**: State changes require actor and justification
//! - **System Tells Truth**: Rollback captures 'why' for forensic audit
//!
//! # Example
//!
//! ```
//! use converge_core::governed_artifact::{
//!     GovernedArtifactState, LifecycleEvent, validate_transition,
//! };
//!
//! // Start in Draft
//! let mut state = GovernedArtifactState::Draft;
//!
//! // Approve for production
//! validate_transition(state, GovernedArtifactState::Approved).unwrap();
//! state = GovernedArtifactState::Approved;
//!
//! // Activate
//! validate_transition(state, GovernedArtifactState::Active).unwrap();
//! state = GovernedArtifactState::Active;
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Governed Artifact State
// ============================================================================

/// Lifecycle state of a governed artifact.
///
/// Governed artifacts progress through explicit lifecycle states:
/// - Draft → Approved → Active → Deprecated | RolledBack | Quarantined
///
/// # State Semantics
///
/// | State | Can Use in Production | Accepts New Runs | Notes |
/// |-------|----------------------|------------------|-------|
/// | Draft | No | No | Development/testing |
/// | Approved | Yes | Yes | Passed review |
/// | Active | Yes | Yes | Currently deployed |
/// | Quarantined | No | No | Stopped for investigation |
/// | Deprecated | No | No | Superseded, migrate away |
/// | RolledBack | No | No | Issues discovered |
///
/// # Tenant Scoping
///
/// "Active" is typically tenant-scoped to avoid cross-tenant authority leakage.
/// An artifact can be Active for tenant A but still Draft for tenant B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GovernedArtifactState {
    /// Artifact is in development/testing - not approved for production
    #[default]
    Draft,
    /// Artifact has been reviewed and approved for production use
    Approved,
    /// Artifact is actively deployed and in use
    Active,
    /// Artifact is quarantined - stopped for investigation but preserved for audit
    /// Allowed: replay old traces. Blocked: new runs.
    Quarantined,
    /// Artifact is deprecated - should be migrated away from
    Deprecated,
    /// Artifact has been rolled back due to issues
    RolledBack,
}

impl GovernedArtifactState {
    /// Check if this state allows production use.
    #[must_use]
    pub fn allows_production_use(&self) -> bool {
        matches!(self, Self::Approved | Self::Active)
    }

    /// Check if this state accepts new runs.
    #[must_use]
    pub fn accepts_new_runs(&self) -> bool {
        matches!(self, Self::Approved | Self::Active)
    }

    /// Check if this state allows replaying old traces.
    ///
    /// Quarantined artifacts can replay old traces for forensic analysis
    /// but cannot be used for new runs.
    #[must_use]
    pub fn allows_replay(&self) -> bool {
        matches!(self, Self::Approved | Self::Active | Self::Quarantined)
    }

    /// Check if this state is terminal (no further transitions allowed).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Deprecated | Self::RolledBack)
    }

    /// Check if this state requires investigation.
    #[must_use]
    pub fn requires_investigation(&self) -> bool {
        matches!(self, Self::Quarantined | Self::RolledBack)
    }

    /// Get human-readable description of this state.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Draft => "In development/testing, not approved for production",
            Self::Approved => "Reviewed and approved, ready for production",
            Self::Active => "Currently deployed and in use",
            Self::Quarantined => "Stopped for investigation, replay allowed",
            Self::Deprecated => "Superseded, should migrate away",
            Self::RolledBack => "Rolled back due to issues",
        }
    }
}

impl std::fmt::Display for GovernedArtifactState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Approved => write!(f, "approved"),
            Self::Active => write!(f, "active"),
            Self::Quarantined => write!(f, "quarantined"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::RolledBack => write!(f, "rolled_back"),
        }
    }
}

// ============================================================================
// State Transition Validation
// ============================================================================

/// Validate a state transition.
///
/// This is a pure function that checks if a transition is allowed
/// without modifying any state.
///
/// # Valid Transitions
///
/// ```text
/// Draft → Approved (after review)
/// Draft → Deprecated (abandoned)
/// Approved → Active (deployment)
/// Approved → Quarantined (issues found before activation)
/// Approved → RolledBack (issues found before activation)
/// Active → Quarantined (immediate stop for investigation)
/// Active → Deprecated (superseded by new version)
/// Active → RolledBack (issues discovered)
/// Quarantined → Active (investigation complete, cleared)
/// Quarantined → RolledBack (investigation confirms issues)
/// Quarantined → Deprecated (decided to replace)
/// ```
///
/// # Terminal States
///
/// No transitions allowed from Deprecated or RolledBack.
///
/// # Errors
///
/// Returns `InvalidStateTransition` if the transition is not allowed.
pub fn validate_transition(
    from: GovernedArtifactState,
    to: GovernedArtifactState,
) -> Result<(), InvalidStateTransition> {
    use GovernedArtifactState::*;

    let valid = match (from, to) {
        // From Draft
        (Draft, Approved) => true,
        (Draft, Deprecated) => true, // Abandoned

        // From Approved
        (Approved, Active) => true,
        (Approved, Quarantined) => true,
        (Approved, RolledBack) => true,

        // From Active
        (Active, Quarantined) => true, // Immediate stop
        (Active, Deprecated) => true,
        (Active, RolledBack) => true,

        // From Quarantined (investigation outcomes)
        (Quarantined, Active) => true,     // Cleared
        (Quarantined, RolledBack) => true, // Confirmed issues
        (Quarantined, Deprecated) => true, // Replace

        // No transitions from terminal states
        (Deprecated, _) => false,
        (RolledBack, _) => false,

        // All other transitions invalid
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(InvalidStateTransition { from, to })
    }
}

/// Error when attempting an invalid state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidStateTransition {
    pub from: GovernedArtifactState,
    pub to: GovernedArtifactState,
}

impl std::fmt::Display for InvalidStateTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid artifact state transition: {} → {} (from '{}' to '{}')",
            self.from,
            self.to,
            self.from.description(),
            self.to.description()
        )
    }
}

impl std::error::Error for InvalidStateTransition {}

// ============================================================================
// Lifecycle Events
// ============================================================================

/// Record of a lifecycle state change.
///
/// Every state transition is recorded with:
/// - Who/what initiated the change
/// - Why the change was made
/// - When it occurred
/// - Optional reference to approval/review ticket
///
/// # Audit Trail
///
/// LifecycleEvents form an append-only audit trail that captures
/// the full history of an artifact's governance journey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Previous state
    pub from_state: GovernedArtifactState,
    /// New state
    pub to_state: GovernedArtifactState,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Who or what initiated the change (user, system, policy engine)
    pub actor: String,
    /// Reason for the state change
    pub reason: String,
    /// Optional link to approval/review ticket
    pub ticket_ref: Option<String>,
    /// Optional tenant scope (if transition is tenant-specific)
    pub tenant_id: Option<String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event with current timestamp.
    ///
    /// Note: For portability, this uses a simple ISO 8601 string.
    /// Production systems should inject the timestamp from a trusted source.
    pub fn new(
        from_state: GovernedArtifactState,
        to_state: GovernedArtifactState,
        actor: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            from_state,
            to_state,
            // Portable timestamp - production should inject from trusted source
            timestamp: Self::now_iso8601(),
            actor: actor.into(),
            reason: reason.into(),
            ticket_ref: None,
            tenant_id: None,
        }
    }

    /// Add a ticket reference.
    #[must_use]
    pub fn with_ticket(mut self, ticket: impl Into<String>) -> Self {
        self.ticket_ref = Some(ticket.into());
        self
    }

    /// Add tenant scope.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set explicit timestamp (for replay/testing).
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = timestamp.into();
        self
    }

    /// Generate ISO 8601 timestamp.
    ///
    /// Note: This returns a placeholder. Production systems should use
    /// `with_timestamp()` to inject a timestamp from a trusted source.
    fn now_iso8601() -> String {
        // Portable placeholder - production should inject timestamp via with_timestamp()
        // This avoids adding chrono as a dependency to converge-core
        "1970-01-01T00:00:00Z".to_string()
    }
}

// ============================================================================
// Rollback Types
// ============================================================================

/// Severity of the issue causing a rollback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RollbackSeverity {
    /// Minor issue - inconvenience but no incorrect outputs
    #[default]
    Low,
    /// Moderate issue - some outputs may be suboptimal
    Medium,
    /// Serious issue - outputs may be incorrect
    High,
    /// Critical issue - must rollback immediately, potential harm
    Critical,
}

impl RollbackSeverity {
    /// Check if this severity requires immediate action.
    #[must_use]
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Get human-readable description.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Minor issue, no incorrect outputs",
            Self::Medium => "Some outputs may be suboptimal",
            Self::High => "Outputs may be incorrect",
            Self::Critical => "Critical, potential harm, immediate action required",
        }
    }
}

impl std::fmt::Display for RollbackSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Impact assessment for a rollback.
///
/// Captures the scope and nature of impact from rolling back an artifact.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RollbackImpact {
    /// Number of requests/runs affected (if known)
    pub affected_count: Option<u64>,
    /// Quality issues observed (descriptions)
    pub quality_issues: Vec<String>,
    /// Whether rollback invalidates existing outputs
    pub invalidates_outputs: bool,
    /// Severity level
    pub severity: RollbackSeverity,
    /// Affected tenant IDs (if known)
    pub affected_tenants: Vec<String>,
}

impl RollbackImpact {
    /// Create a new impact assessment.
    pub fn new(severity: RollbackSeverity) -> Self {
        Self {
            severity,
            ..Default::default()
        }
    }

    /// Add affected count.
    #[must_use]
    pub fn with_affected_count(mut self, count: u64) -> Self {
        self.affected_count = Some(count);
        self
    }

    /// Add quality issue.
    #[must_use]
    pub fn with_quality_issue(mut self, issue: impl Into<String>) -> Self {
        self.quality_issues.push(issue.into());
        self
    }

    /// Mark as invalidating outputs.
    #[must_use]
    pub fn invalidates_outputs(mut self) -> Self {
        self.invalidates_outputs = true;
        self
    }

    /// Add affected tenant.
    #[must_use]
    pub fn with_affected_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.affected_tenants.push(tenant_id.into());
        self
    }
}

/// Rollback record with full context for audit.
///
/// When an artifact is rolled back, we capture everything needed to:
/// 1. Understand why it was rolled back
/// 2. Reproduce the issue
/// 3. Prevent reactivation without addressing the issue
///
/// # Portable Shape
///
/// This type is intentionally generic - it captures rollback semantics
/// without referencing implementation-specific details like merge hashes.
/// Capability-specific rollback records can embed this and add their fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackRecord {
    /// Artifact identifier (opaque string, format depends on artifact type)
    pub artifact_id: String,
    /// Previous state before rollback
    pub previous_state: GovernedArtifactState,
    /// ISO 8601 timestamp of rollback
    pub rolled_back_at: String,
    /// Who initiated the rollback
    pub actor: String,
    /// Detailed reason for rollback
    pub reason: String,
    /// Impact assessment
    pub impact: RollbackImpact,
    /// Optional: Link to incident ticket
    pub incident_ref: Option<String>,
    /// Optional: Tenant scope
    pub tenant_id: Option<String>,
}

impl RollbackRecord {
    /// Create a new rollback record.
    pub fn new(
        artifact_id: impl Into<String>,
        previous_state: GovernedArtifactState,
        actor: impl Into<String>,
        reason: impl Into<String>,
        impact: RollbackImpact,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            previous_state,
            rolled_back_at: LifecycleEvent::now_iso8601(),
            actor: actor.into(),
            reason: reason.into(),
            impact,
            incident_ref: None,
            tenant_id: None,
        }
    }

    /// Add incident reference.
    #[must_use]
    pub fn with_incident(mut self, incident_ref: impl Into<String>) -> Self {
        self.incident_ref = Some(incident_ref.into());
        self
    }

    /// Add tenant scope.
    #[must_use]
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }
}

// ============================================================================
// Replay Integrity
// ============================================================================

/// Categories of replay integrity violations.
///
/// When verifying that a replay is valid, these are the categories
/// of mismatches that can occur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayIntegrityViolation {
    /// Artifact identifier doesn't match
    ArtifactMismatch { expected: String, actual: String },
    /// Content hash doesn't match (artifact was modified)
    ContentHashMismatch { expected: String, actual: String },
    /// Version mismatch
    VersionMismatch { expected: String, actual: String },
    /// Artifact is in a state that doesn't allow replay
    InvalidState {
        state: GovernedArtifactState,
        reason: String,
    },
    /// Required metadata is missing
    MissingMetadata { field: String },
    /// Custom violation (for capability-specific checks)
    Custom { category: String, message: String },
}

impl std::fmt::Display for ReplayIntegrityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArtifactMismatch { expected, actual } => {
                write!(
                    f,
                    "Artifact mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            Self::ContentHashMismatch { expected, actual } => {
                write!(
                    f,
                    "Content hash mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            Self::VersionMismatch { expected, actual } => {
                write!(
                    f,
                    "Version mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            Self::InvalidState { state, reason } => {
                write!(f, "Invalid state '{}' for replay: {}", state, reason)
            }
            Self::MissingMetadata { field } => {
                write!(f, "Missing required metadata field: '{}'", field)
            }
            Self::Custom { category, message } => {
                write!(f, "Replay integrity violation [{}]: {}", category, message)
            }
        }
    }
}

impl std::error::Error for ReplayIntegrityViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // State Tests
    // ========================================================================

    #[test]
    fn test_default_state_is_draft() {
        assert_eq!(
            GovernedArtifactState::default(),
            GovernedArtifactState::Draft
        );
    }

    #[test]
    fn test_allows_production_use() {
        assert!(!GovernedArtifactState::Draft.allows_production_use());
        assert!(GovernedArtifactState::Approved.allows_production_use());
        assert!(GovernedArtifactState::Active.allows_production_use());
        assert!(!GovernedArtifactState::Quarantined.allows_production_use());
        assert!(!GovernedArtifactState::Deprecated.allows_production_use());
        assert!(!GovernedArtifactState::RolledBack.allows_production_use());
    }

    #[test]
    fn test_accepts_new_runs() {
        assert!(!GovernedArtifactState::Draft.accepts_new_runs());
        assert!(GovernedArtifactState::Approved.accepts_new_runs());
        assert!(GovernedArtifactState::Active.accepts_new_runs());
        assert!(!GovernedArtifactState::Quarantined.accepts_new_runs());
        assert!(!GovernedArtifactState::Deprecated.accepts_new_runs());
        assert!(!GovernedArtifactState::RolledBack.accepts_new_runs());
    }

    #[test]
    fn test_allows_replay() {
        assert!(!GovernedArtifactState::Draft.allows_replay());
        assert!(GovernedArtifactState::Approved.allows_replay());
        assert!(GovernedArtifactState::Active.allows_replay());
        assert!(GovernedArtifactState::Quarantined.allows_replay()); // Forensic replay
        assert!(!GovernedArtifactState::Deprecated.allows_replay());
        assert!(!GovernedArtifactState::RolledBack.allows_replay());
    }

    #[test]
    fn test_is_terminal() {
        assert!(!GovernedArtifactState::Draft.is_terminal());
        assert!(!GovernedArtifactState::Approved.is_terminal());
        assert!(!GovernedArtifactState::Active.is_terminal());
        assert!(!GovernedArtifactState::Quarantined.is_terminal());
        assert!(GovernedArtifactState::Deprecated.is_terminal());
        assert!(GovernedArtifactState::RolledBack.is_terminal());
    }

    // ========================================================================
    // Transition Tests
    // ========================================================================

    #[test]
    fn test_valid_transitions_from_draft() {
        assert!(
            validate_transition(
                GovernedArtifactState::Draft,
                GovernedArtifactState::Approved
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Draft,
                GovernedArtifactState::Deprecated
            )
            .is_ok()
        );
    }

    #[test]
    fn test_valid_transitions_from_approved() {
        assert!(
            validate_transition(
                GovernedArtifactState::Approved,
                GovernedArtifactState::Active
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Approved,
                GovernedArtifactState::Quarantined
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Approved,
                GovernedArtifactState::RolledBack
            )
            .is_ok()
        );
    }

    #[test]
    fn test_valid_transitions_from_active() {
        assert!(
            validate_transition(
                GovernedArtifactState::Active,
                GovernedArtifactState::Quarantined
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Active,
                GovernedArtifactState::Deprecated
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Active,
                GovernedArtifactState::RolledBack
            )
            .is_ok()
        );
    }

    #[test]
    fn test_valid_transitions_from_quarantined() {
        assert!(
            validate_transition(
                GovernedArtifactState::Quarantined,
                GovernedArtifactState::Active
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Quarantined,
                GovernedArtifactState::RolledBack
            )
            .is_ok()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::Quarantined,
                GovernedArtifactState::Deprecated
            )
            .is_ok()
        );
    }

    #[test]
    fn test_invalid_transitions() {
        // Cannot skip states
        assert!(
            validate_transition(GovernedArtifactState::Draft, GovernedArtifactState::Active)
                .is_err()
        );
        // Cannot go backwards
        assert!(
            validate_transition(
                GovernedArtifactState::Active,
                GovernedArtifactState::Approved
            )
            .is_err()
        );
        // Cannot transition from terminal states
        assert!(
            validate_transition(
                GovernedArtifactState::Deprecated,
                GovernedArtifactState::Active
            )
            .is_err()
        );
        assert!(
            validate_transition(
                GovernedArtifactState::RolledBack,
                GovernedArtifactState::Draft
            )
            .is_err()
        );
    }

    // ========================================================================
    // Serialization Stability Tests
    // ========================================================================

    #[test]
    fn test_state_serialization_stable() {
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::Draft).unwrap(),
            "\"Draft\""
        );
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::Approved).unwrap(),
            "\"Approved\""
        );
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::Active).unwrap(),
            "\"Active\""
        );
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::Quarantined).unwrap(),
            "\"Quarantined\""
        );
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::Deprecated).unwrap(),
            "\"Deprecated\""
        );
        assert_eq!(
            serde_json::to_string(&GovernedArtifactState::RolledBack).unwrap(),
            "\"RolledBack\""
        );
    }

    #[test]
    fn test_severity_serialization_stable() {
        assert_eq!(
            serde_json::to_string(&RollbackSeverity::Low).unwrap(),
            "\"Low\""
        );
        assert_eq!(
            serde_json::to_string(&RollbackSeverity::Medium).unwrap(),
            "\"Medium\""
        );
        assert_eq!(
            serde_json::to_string(&RollbackSeverity::High).unwrap(),
            "\"High\""
        );
        assert_eq!(
            serde_json::to_string(&RollbackSeverity::Critical).unwrap(),
            "\"Critical\""
        );
    }

    #[test]
    fn test_lifecycle_event_roundtrip() {
        let event = LifecycleEvent::new(
            GovernedArtifactState::Draft,
            GovernedArtifactState::Approved,
            "reviewer@example.com",
            "Passed quality review",
        )
        .with_ticket("TICKET-123")
        .with_tenant("tenant-abc")
        .with_timestamp("2026-01-19T12:00:00Z");

        let json = serde_json::to_string(&event).unwrap();
        let restored: LifecycleEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.from_state, GovernedArtifactState::Draft);
        assert_eq!(restored.to_state, GovernedArtifactState::Approved);
        assert_eq!(restored.actor, "reviewer@example.com");
        assert_eq!(restored.reason, "Passed quality review");
        assert_eq!(restored.ticket_ref, Some("TICKET-123".to_string()));
        assert_eq!(restored.tenant_id, Some("tenant-abc".to_string()));
        assert_eq!(restored.timestamp, "2026-01-19T12:00:00Z");
    }

    #[test]
    fn test_rollback_impact_roundtrip() {
        let impact = RollbackImpact::new(RollbackSeverity::High)
            .with_affected_count(1500)
            .with_quality_issue("Incorrect grounding")
            .with_quality_issue("Missing citations")
            .invalidates_outputs()
            .with_affected_tenant("tenant-1");

        let json = serde_json::to_string(&impact).unwrap();
        let restored: RollbackImpact = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.severity, RollbackSeverity::High);
        assert_eq!(restored.affected_count, Some(1500));
        assert_eq!(restored.quality_issues.len(), 2);
        assert!(restored.invalidates_outputs);
        assert_eq!(restored.affected_tenants, vec!["tenant-1"]);
    }

    #[test]
    fn test_rollback_record_roundtrip() {
        let impact = RollbackImpact::new(RollbackSeverity::Critical);
        let record = RollbackRecord::new(
            "llm/adapter@1.0.0",
            GovernedArtifactState::Active,
            "incident-commander",
            "Critical grounding failure",
            impact,
        )
        .with_incident("INC-456")
        .with_tenant("tenant-xyz");

        let json = serde_json::to_string(&record).unwrap();
        let restored: RollbackRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.artifact_id, "llm/adapter@1.0.0");
        assert_eq!(restored.previous_state, GovernedArtifactState::Active);
        assert_eq!(restored.actor, "incident-commander");
        assert_eq!(restored.incident_ref, Some("INC-456".to_string()));
        assert_eq!(restored.tenant_id, Some("tenant-xyz".to_string()));
    }

    #[test]
    fn test_replay_integrity_violation_display() {
        let v1 = ReplayIntegrityViolation::ArtifactMismatch {
            expected: "adapter-v1".to_string(),
            actual: "adapter-v2".to_string(),
        };
        assert!(v1.to_string().contains("adapter-v1"));
        assert!(v1.to_string().contains("adapter-v2"));

        let v2 = ReplayIntegrityViolation::InvalidState {
            state: GovernedArtifactState::RolledBack,
            reason: "Artifact was rolled back".to_string(),
        };
        assert!(v2.to_string().contains("rolled_back"));
    }
}
