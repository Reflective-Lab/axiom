// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Facts and proposed facts — the type boundary.
//!
//! This is the most important design decision in Converge: LLMs suggest,
//! the engine validates. `ProposedFact` is not `Fact`. There is no implicit
//! conversion between them.

use serde::{Deserialize, Serialize};

use crate::context::ContextKey;

/// Actor kind recorded on a promoted fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FactActorKind {
    /// Human approver.
    Human,
    /// Suggestor or automated domain actor.
    Suggestor,
    /// Kernel or system component.
    System,
}

/// Read-only actor record attached to authoritative facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FactActor {
    id: String,
    kind: FactActorKind,
}

impl FactActor {
    /// Returns the actor identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the actor kind.
    #[must_use]
    pub fn kind(&self) -> FactActorKind {
        self.kind
    }

    #[cfg(feature = "kernel-authority")]
    #[doc(hidden)]
    pub fn new(id: impl Into<String>, kind: FactActorKind) -> Self {
        Self {
            id: id.into(),
            kind,
        }
    }
}

/// Summary of validation checks attached to an authoritative fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct FactValidationSummary {
    checks_passed: Vec<String>,
    checks_skipped: Vec<String>,
    warnings: Vec<String>,
}

impl FactValidationSummary {
    /// Returns validation checks that passed.
    #[must_use]
    pub fn checks_passed(&self) -> &[String] {
        &self.checks_passed
    }

    /// Returns validation checks that were skipped.
    #[must_use]
    pub fn checks_skipped(&self) -> &[String] {
        &self.checks_skipped
    }

    /// Returns validation warnings.
    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    #[cfg(feature = "kernel-authority")]
    #[doc(hidden)]
    pub fn new(
        checks_passed: Vec<String>,
        checks_skipped: Vec<String>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            checks_passed,
            checks_skipped,
            warnings,
        }
    }
}

/// Typed evidence references attached to an authoritative fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "id")]
pub enum FactEvidenceRef {
    /// Observation used as evidence.
    Observation(String),
    /// Human approval used as evidence.
    HumanApproval(String),
    /// Derived artifact used as evidence.
    Derived(String),
}

/// Local replayable trace reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FactLocalTrace {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    sampled: bool,
}

impl FactLocalTrace {
    /// Returns the trace identifier.
    #[must_use]
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Returns the span identifier.
    #[must_use]
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// Returns the parent span identifier.
    #[must_use]
    pub fn parent_span_id(&self) -> Option<&str> {
        self.parent_span_id.as_deref()
    }

    /// Returns whether the trace was sampled.
    #[must_use]
    pub fn sampled(&self) -> bool {
        self.sampled
    }

    #[cfg(feature = "kernel-authority")]
    #[doc(hidden)]
    pub fn new(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        parent_span_id: Option<String>,
        sampled: bool,
    ) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id,
            sampled,
        }
    }
}

/// Remote audit-only trace reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FactRemoteTrace {
    system: String,
    reference: String,
    retrieval_auth: Option<String>,
    retention_hint: Option<String>,
}

impl FactRemoteTrace {
    /// Returns the remote system identifier.
    #[must_use]
    pub fn system(&self) -> &str {
        &self.system
    }

    /// Returns the remote trace reference.
    #[must_use]
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Returns the retrieval auth hint.
    #[must_use]
    pub fn retrieval_auth(&self) -> Option<&str> {
        self.retrieval_auth.as_deref()
    }

    /// Returns the retention hint.
    #[must_use]
    pub fn retention_hint(&self) -> Option<&str> {
        self.retention_hint.as_deref()
    }

    #[cfg(feature = "kernel-authority")]
    #[doc(hidden)]
    pub fn new(
        system: impl Into<String>,
        reference: impl Into<String>,
        retrieval_auth: Option<String>,
        retention_hint: Option<String>,
    ) -> Self {
        Self {
            system: system.into(),
            reference: reference.into(),
            retrieval_auth,
            retention_hint,
        }
    }
}

/// Trace record attached to an authoritative fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum FactTraceLink {
    /// Local replayable trace.
    Local(FactLocalTrace),
    /// Remote audit-only trace.
    Remote(FactRemoteTrace),
}

impl FactTraceLink {
    /// Returns whether the trace is replay-eligible.
    #[must_use]
    pub fn is_replay_eligible(&self) -> bool {
        matches!(self, Self::Local(_))
    }
}

/// Read-only promotion record attached to an authoritative fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FactPromotionRecord {
    gate_id: String,
    policy_version_hash: String,
    approver: FactActor,
    validation_summary: FactValidationSummary,
    evidence_refs: Vec<FactEvidenceRef>,
    trace_link: FactTraceLink,
    promoted_at: String,
}

impl FactPromotionRecord {
    /// Returns the gate identifier that promoted the fact.
    #[must_use]
    pub fn gate_id(&self) -> &str {
        &self.gate_id
    }

    /// Returns the policy hash used during promotion.
    #[must_use]
    pub fn policy_version_hash(&self) -> &str {
        &self.policy_version_hash
    }

    /// Returns the approving actor.
    #[must_use]
    pub fn approver(&self) -> &FactActor {
        &self.approver
    }

    /// Returns the validation summary.
    #[must_use]
    pub fn validation_summary(&self) -> &FactValidationSummary {
        &self.validation_summary
    }

    /// Returns the evidence references used during promotion.
    #[must_use]
    pub fn evidence_refs(&self) -> &[FactEvidenceRef] {
        &self.evidence_refs
    }

    /// Returns the trace link for audit or replay.
    #[must_use]
    pub fn trace_link(&self) -> &FactTraceLink {
        &self.trace_link
    }

    /// Returns the promotion timestamp.
    #[must_use]
    pub fn promoted_at(&self) -> &str {
        &self.promoted_at
    }

    /// Returns whether the promotion is replay-eligible.
    #[must_use]
    pub fn is_replay_eligible(&self) -> bool {
        self.trace_link.is_replay_eligible()
    }

    #[cfg(feature = "kernel-authority")]
    #[doc(hidden)]
    pub fn new(
        gate_id: impl Into<String>,
        policy_version_hash: impl Into<String>,
        approver: FactActor,
        validation_summary: FactValidationSummary,
        evidence_refs: Vec<FactEvidenceRef>,
        trace_link: FactTraceLink,
        promoted_at: impl Into<String>,
    ) -> Self {
        Self {
            gate_id: gate_id.into(),
            policy_version_hash: policy_version_hash.into(),
            approver,
            validation_summary,
            evidence_refs,
            trace_link,
            promoted_at: promoted_at.into(),
        }
    }
}

/// A validated, authoritative assertion in the context.
///
/// Facts are append-only. Once added to the context, they are never
/// mutated or removed (within a convergence run). History is preserved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Fact {
    /// Which context key this fact belongs to.
    key: ContextKey,
    /// Unique identifier within the context key namespace.
    pub id: String,
    /// The fact's content as a string. Interpretation is key-dependent.
    pub content: String,
    /// The immutable promotion record that made this fact authoritative.
    promotion_record: FactPromotionRecord,
    /// When the authoritative fact entered context.
    created_at: String,
}

impl Fact {
    /// Returns the context key this fact belongs to.
    #[must_use]
    pub fn key(&self) -> ContextKey {
        self.key
    }

    /// Returns the immutable promotion record for this fact.
    #[must_use]
    pub fn promotion_record(&self) -> &FactPromotionRecord {
        &self.promotion_record
    }

    /// Returns the fact creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Returns whether the fact is replay-eligible.
    #[must_use]
    pub fn is_replay_eligible(&self) -> bool {
        self.promotion_record.is_replay_eligible()
    }
}

/// Kernel-only construction helpers for authoritative facts.
#[cfg(feature = "kernel-authority")]
#[doc(hidden)]
pub mod kernel_authority {
    use super::*;

    /// Creates a kernel-authoritative fact with default promotion metadata.
    #[must_use]
    pub fn new_fact(key: ContextKey, id: impl Into<String>, content: impl Into<String>) -> Fact {
        new_fact_with_promotion(
            key,
            id,
            content,
            FactPromotionRecord::new(
                "kernel-authority",
                "0000000000000000000000000000000000000000000000000000000000000000",
                FactActor::new("converge-kernel", FactActorKind::System),
                FactValidationSummary::default(),
                Vec::new(),
                FactTraceLink::Local(FactLocalTrace::new("kernel-authority", "seed", None, true)),
                "1970-01-01T00:00:00Z",
            ),
            "1970-01-01T00:00:00Z",
        )
    }

    /// Creates a kernel-authoritative fact with an explicit promotion record.
    #[must_use]
    pub fn new_fact_with_promotion(
        key: ContextKey,
        id: impl Into<String>,
        content: impl Into<String>,
        promotion_record: FactPromotionRecord,
        created_at: impl Into<String>,
    ) -> Fact {
        Fact {
            key,
            id: id.into(),
            content: content.into(),
            promotion_record,
            created_at: created_at.into(),
        }
    }
}

/// An unvalidated suggestion from a non-authoritative source.
///
/// Proposed facts live in `ContextKey::Proposals` until a `ValidationAgent`
/// promotes them to `Fact`. The proposal tracks its origin for audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedFact {
    /// The context key this proposal targets.
    pub key: ContextKey,
    /// Unique identifier encoding origin and target.
    pub id: String,
    /// The proposed content.
    pub content: String,
    /// Confidence hint from the source (0.0 - 1.0).
    pub confidence: f64,
    /// Provenance information (e.g., model ID, prompt hash).
    pub provenance: String,
}

impl ProposedFact {
    /// Create a new draft proposal with explicit provenance.
    #[must_use]
    pub fn new(
        key: ContextKey,
        id: impl Into<String>,
        content: impl Into<String>,
        provenance: impl Into<String>,
    ) -> Self {
        Self {
            key,
            id: id.into(),
            content: content.into(),
            confidence: 1.0,
            provenance: provenance.into(),
        }
    }

    /// Override the proposal confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Error when a `ProposedFact` fails validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Reason the proposal was rejected.
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "validation failed: {}", self.reason)
    }
}

impl std::error::Error for ValidationError {}
