// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Provenance types - PromotionRecord, EvidenceRef, TraceLink.
//!
//! These types track how Facts were promoted and provide audit/replay support.
//!
//! # Design (per CONTEXT.md)
//!
//! - PromotionRecord is REQUIRED on Facts (not optional)
//! - EvidenceRef is a typed enum (not String)
//! - TraceLink distinguishes Local (replay-eligible) vs Remote (audit-only)

use serde::{Deserialize, Serialize};

use super::id::{ApprovalId, ArtifactId, ContentHash, GateId, ObservationId, Timestamp};

// ============================================================================
// EvidenceRef - Typed reference to supporting evidence
// ============================================================================

/// Typed reference to supporting evidence.
///
/// Evidence can be observations, human approvals, or derived artifacts.
/// Using an enum (not String) ensures type safety and enables
/// proper linking/validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum EvidenceRef {
    /// Reference to a raw observation
    Observation(ObservationId),
    /// Reference to a human approval record
    HumanApproval(ApprovalId),
    /// Reference to a derived artifact
    Derived(ArtifactId),
}

impl EvidenceRef {
    /// Create an observation evidence reference.
    pub fn observation(id: ObservationId) -> Self {
        Self::Observation(id)
    }

    /// Create a human approval evidence reference.
    pub fn human_approval(id: ApprovalId) -> Self {
        Self::HumanApproval(id)
    }

    /// Create a derived artifact evidence reference.
    pub fn derived(id: ArtifactId) -> Self {
        Self::Derived(id)
    }
}

// ============================================================================
// TraceLink - Local vs Remote trace semantics
// ============================================================================

/// Trace link for audit and replay.
///
/// The distinction between Local and Remote is constitutional:
/// - Local: replay-eligible (deterministic)
/// - Remote: audit-eligible only (bounded stochasticity)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceLink {
    /// Local trace - replay eligible
    Local(LocalTrace),
    /// Remote trace - audit eligible only
    Remote(RemoteRef),
}

impl TraceLink {
    /// Check if this trace is replay-eligible.
    pub fn is_replay_eligible(&self) -> bool {
        matches!(self, TraceLink::Local(_))
    }

    /// Create a local trace link.
    pub fn local(trace: LocalTrace) -> Self {
        Self::Local(trace)
    }

    /// Create a remote trace link.
    pub fn remote(reference: RemoteRef) -> Self {
        Self::Remote(reference)
    }
}

/// Local trace link - replay eligible.
///
/// Contains trace IDs for distributed tracing systems.
/// Per CONTEXT.md specifics, includes `sampled` flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTrace {
    /// Unique trace identifier
    pub trace_id: String,
    /// Span identifier within the trace
    pub span_id: String,
    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<String>,
    /// Whether this trace was sampled for export
    pub sampled: bool,
}

impl LocalTrace {
    /// Create a new local trace.
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            sampled: true,
        }
    }

    /// Set parent span ID.
    pub fn with_parent(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }

    /// Set sampled flag.
    pub fn with_sampled(mut self, sampled: bool) -> Self {
        self.sampled = sampled;
        self
    }
}

/// Remote trace link - audit eligible only.
///
/// References external tracing systems (Datadog, Jaeger, etc.).
/// Cannot be replayed locally but provides audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRef {
    /// External system identifier (datadog, jaeger, honeycomb, etc.)
    pub system: String,
    /// Reference/link to the trace in the external system
    pub reference: String,
    /// Authentication hint for retrieving the trace (optional)
    pub retrieval_auth: Option<String>,
    /// Hint about trace retention (optional)
    pub retention_hint: Option<String>,
}

impl RemoteRef {
    /// Create a new remote reference.
    pub fn new(system: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            reference: reference.into(),
            retrieval_auth: None,
            retention_hint: None,
        }
    }

    /// Set retrieval authentication hint.
    pub fn with_retrieval_auth(mut self, auth: impl Into<String>) -> Self {
        self.retrieval_auth = Some(auth.into());
        self
    }

    /// Set retention hint.
    pub fn with_retention_hint(mut self, hint: impl Into<String>) -> Self {
        self.retention_hint = Some(hint.into());
        self
    }
}

// ============================================================================
// Actor - Who performed an action
// ============================================================================

/// Actor who performed an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Actor identifier
    pub id: String,
    /// What kind of actor
    pub kind: ActorKind,
}

impl Actor {
    /// Create a new actor.
    pub fn new(id: impl Into<String>, kind: ActorKind) -> Self {
        Self {
            id: id.into(),
            kind,
        }
    }

    /// Create a human actor.
    pub fn human(id: impl Into<String>) -> Self {
        Self::new(id, ActorKind::Human)
    }

    /// Create an agent actor.
    pub fn agent(id: impl Into<String>) -> Self {
        Self::new(id, ActorKind::Agent)
    }

    /// Create a system actor.
    pub fn system(id: impl Into<String>) -> Self {
        Self::new(id, ActorKind::System)
    }
}

/// Kind of actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorKind {
    /// Human user
    Human,
    /// AI agent
    Agent,
    /// System/automation
    System,
}

// ============================================================================
// ValidationSummary - Summary of validation checks
// ============================================================================

/// Summary of validation checks that passed during promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Names of checks that passed
    pub checks_passed: Vec<String>,
    /// Names of checks that were skipped (with reasons)
    pub checks_skipped: Vec<String>,
    /// Warnings generated during validation
    pub warnings: Vec<String>,
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self {
            checks_passed: Vec::new(),
            checks_skipped: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationSummary {
    /// Create a new empty validation summary.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a passed check.
    pub fn with_passed(mut self, check: impl Into<String>) -> Self {
        self.checks_passed.push(check.into());
        self
    }

    /// Add a skipped check.
    pub fn with_skipped(mut self, check: impl Into<String>) -> Self {
        self.checks_skipped.push(check.into());
        self
    }

    /// Add a warning.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Check if all validation passed (no skips or warnings).
    pub fn is_clean(&self) -> bool {
        self.checks_skipped.is_empty() && self.warnings.is_empty()
    }
}

// ============================================================================
// PromotionRecord - How a Fact was promoted (REQUIRED per CONTEXT.md)
// ============================================================================

/// Record of how a Fact was promoted.
///
/// This is REQUIRED on every Fact (not optional) per CONTEXT.md.
/// It provides complete audit trail for the promotion process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRecord {
    /// Which gate approved this promotion
    pub gate_id: GateId,
    /// Hash of the policy version used (for audit/replay)
    pub policy_version_hash: ContentHash,
    /// Who approved the promotion
    pub approver: Actor,
    /// Summary of validation checks
    pub validation_summary: ValidationSummary,
    /// References to supporting evidence
    pub evidence_refs: Vec<EvidenceRef>,
    /// Trace link for audit/replay
    pub trace_link: TraceLink,
    /// When the promotion occurred
    pub promoted_at: Timestamp,
}

impl PromotionRecord {
    /// Strict constructor - all fields required.
    ///
    /// This enforces that promotion records are complete at construction time.
    pub fn new(
        gate_id: GateId,
        policy_version_hash: ContentHash,
        approver: Actor,
        validation_summary: ValidationSummary,
        evidence_refs: Vec<EvidenceRef>,
        trace_link: TraceLink,
        promoted_at: Timestamp,
    ) -> Self {
        Self {
            gate_id,
            policy_version_hash,
            approver,
            validation_summary,
            evidence_refs,
            trace_link,
            promoted_at,
        }
    }

    /// Check if the promotion is replay-eligible.
    pub fn is_replay_eligible(&self) -> bool {
        self.trace_link.is_replay_eligible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_ref_variants() {
        let obs = EvidenceRef::observation(ObservationId::new("obs-001"));
        let approval = EvidenceRef::human_approval(ApprovalId::new("approval-001"));
        let derived = EvidenceRef::derived(ArtifactId::new("artifact-001"));

        // Verify serialization tags
        let obs_json = serde_json::to_string(&obs).unwrap();
        assert!(obs_json.contains("\"type\":\"Observation\""));

        let approval_json = serde_json::to_string(&approval).unwrap();
        assert!(approval_json.contains("\"type\":\"HumanApproval\""));

        let derived_json = serde_json::to_string(&derived).unwrap();
        assert!(derived_json.contains("\"type\":\"Derived\""));
    }

    #[test]
    fn trace_link_replay_eligibility() {
        let local = TraceLink::local(LocalTrace::new("trace-001", "span-001"));
        let remote = TraceLink::remote(RemoteRef::new("datadog", "https://app.datadoghq.com/..."));

        assert!(local.is_replay_eligible());
        assert!(!remote.is_replay_eligible());
    }

    #[test]
    fn local_trace_with_parent() {
        let trace = LocalTrace::new("trace-001", "span-002")
            .with_parent("span-001")
            .with_sampled(false);

        assert_eq!(trace.parent_span_id, Some("span-001".to_string()));
        assert!(!trace.sampled);
    }

    #[test]
    fn validation_summary_builder() {
        let summary = ValidationSummary::new()
            .with_passed("schema_valid")
            .with_passed("confidence_threshold")
            .with_skipped("human_review")
            .with_warning("low_confidence_override");

        assert_eq!(summary.checks_passed.len(), 2);
        assert_eq!(summary.checks_skipped.len(), 1);
        assert_eq!(summary.warnings.len(), 1);
        assert!(!summary.is_clean());
    }

    #[test]
    fn promotion_record_creation() {
        let record = PromotionRecord::new(
            GateId::new("gate-main"),
            ContentHash::zero(),
            Actor::system("converge-engine"),
            ValidationSummary::new().with_passed("all_checks"),
            vec![EvidenceRef::observation(ObservationId::new("obs-001"))],
            TraceLink::local(LocalTrace::new("trace-001", "span-001")),
            Timestamp::now(),
        );

        assert_eq!(record.gate_id.as_str(), "gate-main");
        assert!(record.is_replay_eligible());
    }

    #[test]
    fn actor_helpers() {
        let human = Actor::human("user@example.com");
        let agent = Actor::agent("llm-agent-001");
        let system = Actor::system("converge-engine");

        assert_eq!(human.kind, ActorKind::Human);
        assert_eq!(agent.kind, ActorKind::Agent);
        assert_eq!(system.kind, ActorKind::System);
    }
}
