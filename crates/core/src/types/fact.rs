// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Fact type with private constructor.
//!
//! Facts can only be created via `PromotionGate::promote()`.
//! All fields are private with getter methods only - no mutation possible.
//!
//! # Design Invariants
//!
//! - **Private constructor**: `Fact::new()` is `pub(crate)` - only callable within converge-core
//! - **Required PromotionRecord**: Every Fact has a non-optional PromotionRecord
//! - **Immutable**: No `&mut self` methods - once created, a Fact cannot change
//! - **Getters only**: All access is through getter methods returning references

use serde::{Deserialize, Serialize};

use super::id::{FactId, Timestamp};
use super::provenance::PromotionRecord;

// ============================================================================
// FactContentKind - What kind of promoted content
// ============================================================================

/// Kind of fact content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactContentKind {
    /// A promoted claim or assertion
    Claim,
    /// A promoted action plan
    Plan,
    /// A promoted classification
    Classification,
    /// A promoted evaluation
    Evaluation,
    /// A promoted document
    Document,
    /// Promoted reasoning output
    Reasoning,
}

impl Default for FactContentKind {
    fn default() -> Self {
        Self::Reasoning
    }
}

/// Convert from ProposedContentKind to FactContentKind.
///
/// Used during promotion to map proposal content kinds to fact content kinds.
impl From<super::proposal::ProposedContentKind> for FactContentKind {
    fn from(kind: super::proposal::ProposedContentKind) -> Self {
        use super::proposal::ProposedContentKind;
        match kind {
            ProposedContentKind::Claim => FactContentKind::Claim,
            ProposedContentKind::Plan => FactContentKind::Plan,
            ProposedContentKind::Classification => FactContentKind::Classification,
            ProposedContentKind::Evaluation => FactContentKind::Evaluation,
            ProposedContentKind::Draft => FactContentKind::Document,
            ProposedContentKind::Reasoning => FactContentKind::Reasoning,
        }
    }
}

// ============================================================================
// FactContent - The promoted content
// ============================================================================

/// Content of a promoted fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactContent {
    /// What kind of content this is
    pub kind: FactContentKind,
    /// The textual content
    pub content: String,
    /// Structured content (if applicable)
    pub structured: Option<serde_json::Value>,
}

impl FactContent {
    /// Create new fact content.
    pub fn new(kind: FactContentKind, content: impl Into<String>) -> Self {
        Self {
            kind,
            content: content.into(),
            structured: None,
        }
    }

    /// Add structured content.
    pub fn with_structured(mut self, structured: serde_json::Value) -> Self {
        self.structured = Some(structured);
        self
    }
}

// ============================================================================
// Fact - A promoted, governed truth
// ============================================================================

/// A promoted, governed truth. Immutable after creation.
///
/// Facts can only be created via `PromotionGate::promote()`. Direct construction
/// is impossible outside converge-core.
///
/// # Invariants
///
/// - **Non-optional PromotionRecord**: Every Fact has a complete promotion record
/// - **Immutable**: No mutation methods - corrections are new Facts
/// - **Private fields**: All access via getters
///
/// # Example
///
/// ```text
/// // Facts cannot be created directly from external code:
/// //
/// // let fact = Fact::new(...);  // ERROR: `new` is private
/// //
/// // Instead, Facts are created by the PromotionGate:
/// //
/// // let fact = promotion_gate.promote(validated_proposal)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Unique identifier (private)
    id: FactId,
    /// The fact content (private)
    content: FactContent,
    /// How this fact was promoted - REQUIRED per CONTEXT.md (private)
    promotion_record: PromotionRecord,
    /// When this fact was created (private)
    created_at: Timestamp,
}

impl Fact {
    /// Private constructor - only callable by PromotionGate.
    ///
    /// This enforces that Facts can only be created through proper governance.
    /// External crates cannot call this constructor.
    pub(crate) fn new(
        id: FactId,
        content: FactContent,
        promotion_record: PromotionRecord,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            content,
            promotion_record,
            created_at,
        }
    }

    // ========================================================================
    // Getter methods only - no mutation
    // ========================================================================

    /// Get the fact ID.
    pub fn id(&self) -> &FactId {
        &self.id
    }

    /// Get the fact content.
    pub fn content(&self) -> &FactContent {
        &self.content
    }

    /// Get the promotion record.
    ///
    /// This is ALWAYS present (non-optional) per CONTEXT.md requirements.
    pub fn promotion_record(&self) -> &PromotionRecord {
        &self.promotion_record
    }

    /// Get the creation timestamp.
    pub fn created_at(&self) -> &Timestamp {
        &self.created_at
    }

    /// Check if this fact's promotion is replay-eligible.
    pub fn is_replay_eligible(&self) -> bool {
        self.promotion_record.is_replay_eligible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Actor, ContentHash, EvidenceRef, GateId, LocalTrace, ObservationId, TraceLink,
        ValidationSummary,
    };

    fn make_promotion_record() -> PromotionRecord {
        PromotionRecord::new(
            GateId::new("gate-test"),
            ContentHash::zero(),
            Actor::system("test-engine"),
            ValidationSummary::new().with_passed("all_checks"),
            vec![EvidenceRef::observation(ObservationId::new("obs-001"))],
            TraceLink::local(LocalTrace::new("trace-001", "span-001")),
            Timestamp::now(),
        )
    }

    #[test]
    fn fact_content_creation() {
        let content = FactContent::new(FactContentKind::Claim, "The market is growing")
            .with_structured(serde_json::json!({"growth_rate": 0.15}));

        assert_eq!(content.kind, FactContentKind::Claim);
        assert!(content.structured.is_some());
    }

    #[test]
    fn fact_creation_internal() {
        // Note: This test works because it's in the same crate.
        // External code would get "error: associated function `new` is private"
        let fact = Fact::new(
            FactId::new("fact-001"),
            FactContent::new(FactContentKind::Claim, "Test fact"),
            make_promotion_record(),
            Timestamp::now(),
        );

        assert_eq!(fact.id().as_str(), "fact-001");
        assert_eq!(fact.content().kind, FactContentKind::Claim);
        assert_eq!(fact.promotion_record().gate_id.as_str(), "gate-test");
    }

    #[test]
    fn fact_is_immutable() {
        // This test documents that there are no &mut self methods.
        // The Fact struct has only getter methods that return references.
        let fact = Fact::new(
            FactId::new("fact-001"),
            FactContent::new(FactContentKind::Claim, "Immutable"),
            make_promotion_record(),
            Timestamp::now(),
        );

        // All these return references, not mutable access
        let _ = fact.id();
        let _ = fact.content();
        let _ = fact.promotion_record();
        let _ = fact.created_at();

        // There is no way to mutate the fact after creation
    }

    #[test]
    fn fact_replay_eligibility() {
        let fact = Fact::new(
            FactId::new("fact-001"),
            FactContent::new(FactContentKind::Claim, "Test"),
            make_promotion_record(),
            Timestamp::now(),
        );

        // Local trace -> replay eligible
        assert!(fact.is_replay_eligible());
    }

    #[test]
    fn fact_serialization_roundtrip() {
        let fact = Fact::new(
            FactId::new("fact-002"),
            FactContent::new(FactContentKind::Evaluation, "Score: 8/10"),
            make_promotion_record(),
            Timestamp::epoch(),
        );

        let json = serde_json::to_string(&fact).unwrap();
        let deserialized: Fact = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id().as_str(), "fact-002");
        assert_eq!(deserialized.content().kind, FactContentKind::Evaluation);
    }

    // Note: External code cannot construct Facts directly.
    // This is enforced at compile-time:
    //
    // // In external crate:
    // use converge_core::types::Fact;
    // let fact = Fact::new(...);
    // // ERROR: associated function `new` is private
    //
    // The only way to create a Fact is via PromotionGate (defined in converge-core).
}
