//! Policy decision types aligned with converge-core's gate model.
//!
//! Maps Cedar allow/deny to the three-valued `GateDecision`:
//! Promote (allow), Reject (deny), Escalate (needs human).

use serde::{Deserialize, Serialize};

/// Outcome of a policy evaluation, aligned with converge-core's `GateDecision`.
///
/// This is intentionally compatible with `converge_optimization::gate::GateDecision`
/// so that policy decisions can flow directly into the promotion gate pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOutcome {
    /// Action is allowed — maps to `GateDecision::Promote`
    Promote,
    /// Action is denied — maps to `GateDecision::Reject`
    Reject,
    /// Action requires escalation to human authority — maps to `GateDecision::Escalate`
    Escalate,
}

impl PolicyOutcome {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Promote)
    }

    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Promote | Self::Reject)
    }
}

/// Full policy decision with rationale and audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// The outcome
    pub outcome: PolicyOutcome,
    /// How the decision was made
    pub mode: DecisionMode,
    /// Human-readable rationale (from Cedar diagnostics or delegation verification)
    pub reason: Option<String>,
    /// The principal that was evaluated
    pub principal_id: String,
    /// The action that was attempted
    pub action: String,
    /// The resource that was targeted
    pub resource_id: String,
}

/// How the decision was reached
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMode {
    /// Evaluated via Cedar policy rules
    Policy,
    /// Verified via signed delegation token
    Delegation,
}

impl PolicyDecision {
    #[must_use]
    pub fn policy(
        outcome: PolicyOutcome,
        reason: Option<String>,
        principal_id: String,
        action: String,
        resource_id: String,
    ) -> Self {
        Self {
            outcome,
            mode: DecisionMode::Policy,
            reason,
            principal_id,
            action,
            resource_id,
        }
    }

    #[must_use]
    pub fn delegation(
        outcome: PolicyOutcome,
        reason: Option<String>,
        principal_id: String,
        action: String,
        resource_id: String,
    ) -> Self {
        Self {
            outcome,
            mode: DecisionMode::Delegation,
            reason,
            principal_id,
            action,
            resource_id,
        }
    }
}
