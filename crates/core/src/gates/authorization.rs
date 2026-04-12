// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Neutral flow-gate authorization contract.
//!
//! Converging flows should project their current state into [`FlowGateInput`]
//! and ask a [`FlowGateAuthorizer`] for a deterministic decision. Concrete
//! implementations may use Cedar, fixed test doubles, or another governed
//! evaluator, but the flow runtime stays decoupled from those details.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Action being attempted against a converging flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowAction {
    Propose,
    Validate,
    Promote,
    Commit,
    AdvancePhase,
}

impl FlowAction {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Propose => "propose",
            Self::Validate => "validate",
            Self::Promote => "promote",
            Self::Commit => "commit",
            Self::AdvancePhase => "advance_phase",
        }
    }
}

/// Principal facts projected from the flow host or application runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGatePrincipal {
    pub id: String,
    pub authority: String,
    pub domains: Vec<String>,
    pub policy_version: Option<String>,
}

/// Resource facts projected from the current flow state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGateResource {
    pub id: String,
    pub kind: String,
    pub phase: String,
    pub gates_passed: Vec<String>,
}

/// Decision-relevant facts projected from the flow state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlowGateContext {
    pub commitment_type: Option<String>,
    pub amount: Option<i64>,
    pub human_approval_present: Option<bool>,
    pub required_gates_met: Option<bool>,
}

/// Canonical input to an authorization decision for a flow gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGateInput {
    pub principal: FlowGatePrincipal,
    pub resource: FlowGateResource,
    pub action: FlowAction,
    pub context: FlowGateContext,
}

/// Neutral outcome of a flow gate authorization decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowGateOutcome {
    Promote,
    Reject,
    Escalate,
}

impl FlowGateOutcome {
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        matches!(self, Self::Promote)
    }
}

/// Full gate decision with rationale and source attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGateDecision {
    pub outcome: FlowGateOutcome,
    pub reason: Option<String>,
    pub source: Option<String>,
}

impl FlowGateDecision {
    #[must_use]
    pub fn promote(reason: Option<String>, source: Option<String>) -> Self {
        Self {
            outcome: FlowGateOutcome::Promote,
            reason,
            source,
        }
    }

    #[must_use]
    pub fn reject(reason: Option<String>, source: Option<String>) -> Self {
        Self {
            outcome: FlowGateOutcome::Reject,
            reason,
            source,
        }
    }

    #[must_use]
    pub fn escalate(reason: Option<String>, source: Option<String>) -> Self {
        Self {
            outcome: FlowGateOutcome::Escalate,
            reason,
            source,
        }
    }
}

/// Pure error surface for flow gate authorization.
#[derive(Debug, Error)]
pub enum FlowGateError {
    #[error("authorizer failed: {0}")]
    Authorizer(String),
    #[error("invalid flow gate input: {0}")]
    InvalidInput(String),
}

/// Deterministic decision provider for consequential flow actions.
pub trait FlowGateAuthorizer: Send + Sync {
    /// Decide whether the attempted flow action should promote, reject, or escalate.
    fn decide(&self, input: &FlowGateInput) -> Result<FlowGateDecision, FlowGateError>;
}

/// Test double: always promote.
#[derive(Debug, Default, Clone, Copy)]
pub struct AllowAllFlowGateAuthorizer;

impl FlowGateAuthorizer for AllowAllFlowGateAuthorizer {
    fn decide(&self, _input: &FlowGateInput) -> Result<FlowGateDecision, FlowGateError> {
        Ok(FlowGateDecision::promote(
            Some("allow_all test authorizer".into()),
            Some("allow_all".into()),
        ))
    }
}

/// Test double: always reject.
#[derive(Debug, Default, Clone)]
pub struct RejectAllFlowGateAuthorizer {
    reason: Option<String>,
}

impl RejectAllFlowGateAuthorizer {
    #[must_use]
    pub fn with_reason(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
        }
    }
}

impl FlowGateAuthorizer for RejectAllFlowGateAuthorizer {
    fn decide(&self, _input: &FlowGateInput) -> Result<FlowGateDecision, FlowGateError> {
        Ok(FlowGateDecision::reject(
            self.reason
                .clone()
                .or_else(|| Some("reject_all test authorizer".into())),
            Some("reject_all".into()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> FlowGateInput {
        FlowGateInput {
            principal: FlowGatePrincipal {
                id: "agent:test".into(),
                authority: "supervisory".into(),
                domains: vec!["finance".into()],
                policy_version: Some("v1".into()),
            },
            resource: FlowGateResource {
                id: "expense:1".into(),
                kind: "expense".into(),
                phase: "commitment".into(),
                gates_passed: vec!["receipt".into()],
            },
            action: FlowAction::Validate,
            context: FlowGateContext {
                commitment_type: Some("expense".into()),
                amount: Some(100),
                human_approval_present: Some(false),
                required_gates_met: Some(true),
            },
        }
    }

    #[test]
    fn allow_all_authorizer_promotes() {
        let decision = AllowAllFlowGateAuthorizer
            .decide(&sample_input())
            .expect("allow_all should succeed");
        assert_eq!(decision.outcome, FlowGateOutcome::Promote);
    }

    #[test]
    fn reject_all_authorizer_rejects() {
        let decision = RejectAllFlowGateAuthorizer::with_reason("blocked")
            .decide(&sample_input())
            .expect("reject_all should succeed");
        assert_eq!(decision.outcome, FlowGateOutcome::Reject);
        assert_eq!(decision.reason.as_deref(), Some("blocked"));
    }
}
