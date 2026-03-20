// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT

//! Backend selection based on requirements.
//!
//! Selection is capability-based, not type-based. You declare what you need
//! (capabilities, cost, latency, compliance) and the selector finds backends
//! that match.
//!
//! # Design
//!
//! Requirements are orthogonal dimensions:
//! 1. **Kind** — Which category of backend (LLM, policy, optimizer, etc.)
//! 2. **Capabilities** — What the backend must be able to do
//! 3. **Cost** — Budget preference
//! 4. **Data sovereignty** — Where data can legally reside
//! 5. **Compliance** — Regulatory requirements

use serde::{Deserialize, Serialize};

use crate::backend::BackendKind;
use crate::capability::Capability;
use crate::error::BackendError;

/// Requirements for backend selection.
///
/// Describes what an agent needs from a backend. The selector matches
/// these requirements against available backends.
///
/// # Example
///
/// ```
/// use converge_traits::{BackendRequirements, BackendKind, Capability, CostClass};
///
/// let reqs = BackendRequirements::new(BackendKind::Llm)
///     .with_capability(Capability::TextGeneration)
///     .with_capability(Capability::Reasoning)
///     .with_max_cost(CostClass::Medium);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendRequirements {
    /// Required backend kind.
    pub kind: BackendKind,
    /// Required capabilities (all must be satisfied).
    pub required_capabilities: Vec<Capability>,
    /// Maximum cost class.
    pub max_cost_class: CostClass,
    /// Maximum acceptable latency in milliseconds (0 = no limit).
    pub max_latency_ms: u32,
    /// Data sovereignty requirement.
    pub data_sovereignty: DataSovereignty,
    /// Compliance requirement.
    pub compliance: ComplianceLevel,
    /// Whether deterministic replay is required.
    pub requires_replay: bool,
    /// Whether offline operation is required.
    pub requires_offline: bool,
}

impl BackendRequirements {
    /// Creates requirements for a specific backend kind.
    #[must_use]
    pub fn new(kind: BackendKind) -> Self {
        Self {
            kind,
            required_capabilities: Vec::new(),
            max_cost_class: CostClass::VeryHigh,
            max_latency_ms: 0,
            data_sovereignty: DataSovereignty::Any,
            compliance: ComplianceLevel::None,
            requires_replay: false,
            requires_offline: false,
        }
    }

    /// Add a required capability.
    #[must_use]
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    /// Set maximum cost class.
    #[must_use]
    pub fn with_max_cost(mut self, cost: CostClass) -> Self {
        self.max_cost_class = cost;
        self
    }

    /// Set maximum latency in milliseconds.
    #[must_use]
    pub fn with_max_latency_ms(mut self, ms: u32) -> Self {
        self.max_latency_ms = ms;
        self
    }

    /// Set data sovereignty requirement.
    #[must_use]
    pub fn with_data_sovereignty(mut self, sovereignty: DataSovereignty) -> Self {
        self.data_sovereignty = sovereignty;
        self
    }

    /// Set compliance requirement.
    #[must_use]
    pub fn with_compliance(mut self, compliance: ComplianceLevel) -> Self {
        self.compliance = compliance;
        self
    }

    /// Require deterministic replay.
    #[must_use]
    pub fn with_replay(mut self) -> Self {
        self.requires_replay = true;
        self
    }

    /// Require offline operation.
    #[must_use]
    pub fn with_offline(mut self) -> Self {
        self.requires_offline = true;
        self
    }

    // ── Preset constructors ───────────────────────────────────────────

    /// Fast, cheap LLM for simple extraction tasks.
    #[must_use]
    pub fn fast_llm() -> Self {
        Self::new(BackendKind::Llm)
            .with_capability(Capability::TextGeneration)
            .with_max_cost(CostClass::Low)
            .with_max_latency_ms(2000)
    }

    /// Powerful LLM for reasoning tasks.
    #[must_use]
    pub fn reasoning_llm() -> Self {
        Self::new(BackendKind::Llm)
            .with_capability(Capability::TextGeneration)
            .with_capability(Capability::Reasoning)
            .with_max_cost(CostClass::High)
            .with_max_latency_ms(30_000)
    }

    /// Policy engine for access control.
    #[must_use]
    pub fn access_policy() -> Self {
        Self::new(BackendKind::Policy)
            .with_capability(Capability::AccessControl)
            .with_max_latency_ms(100)
    }

    /// Constraint solver for optimization.
    #[must_use]
    pub fn constraint_solver() -> Self {
        Self::new(BackendKind::Optimization).with_capability(Capability::ConstraintSolving)
    }

    /// Analytics pipeline for embeddings.
    #[must_use]
    pub fn embedding_pipeline() -> Self {
        Self::new(BackendKind::Analytics).with_capability(Capability::Embedding)
    }

    /// Vector search for semantic recall.
    #[must_use]
    pub fn vector_search() -> Self {
        Self::new(BackendKind::Search).with_capability(Capability::VectorSearch)
    }
}

/// Trait for selecting a backend that satisfies requirements.
///
/// Implementations live in runtime or orchestration crates that have
/// access to the full backend registry.
pub trait BackendSelector: Send + Sync {
    /// Select a backend name that satisfies the requirements.
    ///
    /// Returns the backend name (matching [`Backend::name()`](crate::Backend::name)).
    ///
    /// # Errors
    ///
    /// Returns error if no backend satisfies the requirements.
    fn select(&self, requirements: &BackendRequirements) -> Result<String, BackendError>;
}

// ── Selection dimensions ──────────────────────────────────────────────

/// Cost classification — how expensive is this backend to use?
///
/// Used for budget-aware routing. The selector filters backends whose
/// cost class exceeds the maximum allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CostClass {
    /// Free or negligible cost (local inference, cached results).
    Free,
    /// Very low cost (small local models, simple policy checks).
    VeryLow,
    /// Low cost (medium local models, basic API calls).
    Low,
    /// Medium cost (standard API calls, moderate compute).
    Medium,
    /// High cost (frontier models, complex optimization).
    High,
    /// Very high cost (multi-step reasoning, large-scale analytics).
    VeryHigh,
}

impl CostClass {
    /// Returns all cost classes at or below this level.
    #[must_use]
    pub fn allowed_classes(self) -> Vec<CostClass> {
        let all = [
            CostClass::Free,
            CostClass::VeryLow,
            CostClass::Low,
            CostClass::Medium,
            CostClass::High,
            CostClass::VeryHigh,
        ];
        all.iter().copied().filter(|&c| c <= self).collect()
    }
}

/// Data sovereignty requirements — where can data legally reside?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSovereignty {
    /// No restriction on data location.
    Any,
    /// Data must stay within the EU/EEA.
    EU,
    /// Data must stay within the US.
    US,
    /// Data must stay within Switzerland.
    Switzerland,
    /// Data must stay within China.
    China,
    /// Data must stay on-premises (no cloud).
    OnPremises,
}

/// Compliance level requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// No specific compliance requirements.
    None,
    /// EU General Data Protection Regulation.
    GDPR,
    /// US Health Insurance Portability and Accountability Act.
    HIPAA,
    /// Service Organization Control 2.
    SOC2,
    /// High explainability required (decisions must be auditable).
    HighExplainability,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_class_ordering() {
        assert!(CostClass::Free < CostClass::VeryLow);
        assert!(CostClass::VeryLow < CostClass::Low);
        assert!(CostClass::Low < CostClass::Medium);
        assert!(CostClass::Medium < CostClass::High);
        assert!(CostClass::High < CostClass::VeryHigh);
    }

    #[test]
    fn allowed_classes_correct() {
        assert_eq!(CostClass::Free.allowed_classes(), vec![CostClass::Free]);
        assert_eq!(
            CostClass::Low.allowed_classes(),
            vec![CostClass::Free, CostClass::VeryLow, CostClass::Low]
        );
        assert_eq!(CostClass::VeryHigh.allowed_classes().len(), 6);
    }

    #[test]
    fn requirements_builder() {
        let reqs = BackendRequirements::new(BackendKind::Llm)
            .with_capability(Capability::TextGeneration)
            .with_capability(Capability::Reasoning)
            .with_max_cost(CostClass::Medium)
            .with_max_latency_ms(5000);

        assert_eq!(reqs.kind, BackendKind::Llm);
        assert_eq!(reqs.required_capabilities.len(), 2);
        assert_eq!(reqs.max_cost_class, CostClass::Medium);
        assert_eq!(reqs.max_latency_ms, 5000);
    }

    #[test]
    fn preset_constructors() {
        let fast = BackendRequirements::fast_llm();
        assert_eq!(fast.kind, BackendKind::Llm);
        assert_eq!(fast.max_cost_class, CostClass::Low);

        let policy = BackendRequirements::access_policy();
        assert_eq!(policy.kind, BackendKind::Policy);
        assert!(
            policy
                .required_capabilities
                .contains(&Capability::AccessControl)
        );

        let solver = BackendRequirements::constraint_solver();
        assert_eq!(solver.kind, BackendKind::Optimization);
    }
}
