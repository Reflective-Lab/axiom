//! Domain types for policy decisions.
//!
//! These map to Converge's governance model:
//! - Principals are agent personas with authority levels
//! - Resources are flows/commitments with phase and gate state
//! - Context carries decision-relevant facts

use serde::{Deserialize, Serialize};

/// Agent persona — the principal in Converge policy decisions.
///
/// Maps to converge-personas definitions. Authority levels determine
/// what actions the agent can perform without escalation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipalIn {
    /// Agent identifier (e.g., `agent:strategic_analyst`)
    pub id: String,
    /// Authority level: advisory, supervisory, participatory, sovereign
    pub authority: String,
    /// Domains this agent operates in
    pub domains: Vec<String>,
    /// Policy version binding (e.g., `enterprise_v2.3`)
    pub policy_version: Option<String>,
}

/// Flow or commitment — the resource being acted upon.
///
/// Represents a converging flow at a specific phase, with its
/// gate evaluation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIn {
    /// Flow/commitment identifier (e.g., `flow:quote-2025-0042`)
    pub id: String,
    /// Commitment type: quote, spend, contract, invoice
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    /// Current phase: intent, framing, exploration, tension, convergence, commitment
    pub phase: Option<String>,
    /// Gates that have been passed
    pub gates_passed: Option<Vec<String>>,
}

/// Decision context — facts about the action being attempted.
///
/// The caller pre-joins these facts from the business context,
/// keeping the policy engine free of data-fetching side effects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextIn {
    /// Type of commitment (quote, spend, contract, invoice)
    pub commitment_type: Option<String>,
    /// Monetary amount (if applicable)
    pub amount: Option<i64>,
    /// Whether a human has explicitly approved this action
    pub human_approval_present: Option<bool>,
    /// Whether all required gates for the current phase are met
    pub required_gates_met: Option<bool>,
}

/// Full decision request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecideRequest {
    pub principal: PrincipalIn,
    pub resource: ResourceIn,
    /// Action: propose, validate, promote, commit, `advance_phase`
    pub action: String,
    pub context: Option<ContextIn>,
    /// If true, record the decision in the event store
    pub observe: Option<bool>,
    /// Optional delegation token for fast-path elevated authority
    pub delegation_b64: Option<String>,
}
