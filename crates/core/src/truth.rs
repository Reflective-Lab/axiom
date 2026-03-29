// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Truth catalog primitives.
//!
//! Truths describe jobs, policies, and invariants above domain packs.
//! Applications provide the catalog content; the runtime consumes a common
//! shape for intent construction, guardrails, and pack participation.

use serde::{Deserialize, Serialize};

use crate::{Context, Criterion, FactId, TypesIntentConstraint};

/// What class of truth is being described.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TruthKind {
    /// A job-to-be-done spanning multiple packs.
    Job,
    /// A cross-cutting policy or guardrail.
    Policy,
    /// A module-local or pack-local invariant.
    Invariant,
}

/// Portable truth definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthDefinition {
    /// Stable truth identifier.
    pub key: String,
    /// Truth class.
    pub kind: TruthKind,
    /// Human-readable summary.
    pub summary: String,
    /// Required or optional success criteria.
    pub success_criteria: Vec<Criterion>,
    /// Hard and soft constraints derived from the truth.
    pub constraints: Vec<TypesIntentConstraint>,
    /// Human approval points that the runtime must respect.
    pub approval_points: Vec<String>,
    /// Which packs should participate when this truth is active.
    pub participating_packs: Vec<String>,
}

/// Machine-evaluable outcome for a single criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriterionResult {
    /// The criterion was satisfied, with fact IDs that justify the result.
    Met { evidence: Vec<FactId> },
    /// The criterion is currently blocked on human intervention.
    Blocked {
        /// Why the criterion is blocked.
        reason: String,
        /// Optional approval or workflow reference the host can surface.
        approval_ref: Option<String>,
    },
    /// The criterion was evaluated and is not satisfied.
    Unmet { reason: String },
    /// The runtime could not determine whether the criterion was satisfied.
    Indeterminate,
}

/// Evaluated outcome for a specific criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionOutcome {
    /// The criterion that was evaluated.
    pub criterion: Criterion,
    /// The result of the evaluation.
    pub result: CriterionResult,
}

/// Application-provided boundary for evaluating success criteria.
pub trait CriterionEvaluator: Send + Sync {
    /// Evaluate a criterion against the converged context.
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult;
}

/// Application-provided truth catalog boundary.
pub trait TruthCatalog: Send + Sync {
    /// List all truths known to the application.
    fn list_truths(&self) -> Vec<TruthDefinition>;

    /// Resolve a truth by key.
    fn find_truth(&self, key: &str) -> Option<TruthDefinition> {
        self.list_truths()
            .into_iter()
            .find(|truth| truth.key == key)
    }
}
