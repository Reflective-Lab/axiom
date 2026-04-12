// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Kernel boundary types - Constitutional interfaces.
//!
//! This module formalizes the types that define the contract between
//! reasoning kernels (converge-llm) and the Converge platform.
//!
//! # Constitutional Properties
//!
//! 1. Kernels emit proposals, never facts
//! 2. All proposals have trace links
//! 3. Human authority is first-class
//! 4. Explicit authority grants (no defaults)
//!
//! # Re-exports
//!
//! Key types are re-exported from `kernel_boundary.rs` and organized
//! here for gate pattern integration.

/// Constitutional types for kernel-platform boundaries.
///
/// These types are "constitutional" in that they encode fundamental
/// axioms of the Converge system:
///
/// - Kernels cannot create Facts directly
/// - All proposals must have provenance
/// - Authority is explicit, never implicit
pub mod constitutional {
    pub use crate::kernel_boundary::{
        // Supporting types
        AdapterTrace,
        // Output types (kernel -> platform)
        ContentKind,
        // Input types (platform -> kernel)
        ContextFact,
        ContractResult,
        // Routing types
        DataClassification,
        // Decision flow
        DecisionStep,
        ExecutionEnv,
        KernelContext,
        KernelIntent,
        KernelPolicy,
        KernelProposal,
        // Tracing types
        LocalReplayTrace,
        ProposalKind,
        ProposedContent,
        RecallTrace,
        RemoteReplayTrace,
        ReplayTrace,
        Replayability,
        ReplayabilityDowngradeReason,
        RiskTier,
        RoutingPolicy,
        SamplerParams,
    };
}

/// Authority grant - explicit permission for promotion.
///
/// Encodes REQ-GATE-03: "No defaults that grant authority".
/// Any code path that promotes must receive an explicit AuthorityGrant.
///
/// # Design
///
/// AuthorityGrant can only be created by:
/// - System authority (for automated gates)
/// - Human approval (for human-in-the-loop)
/// - Policy delegation (for policy-defined auto-promotion)
///
/// External code cannot construct arbitrary authority grants.
#[derive(Debug, Clone)]
pub struct AuthorityGrant {
    /// Who/what granted the authority
    grantor: AuthorityGrantor,
    /// When the grant was issued
    granted_at: crate::types::Timestamp,
    /// Optional scope limitation
    scope: Option<AuthorityScope>,
}

#[allow(dead_code)]
impl AuthorityGrant {
    /// Create a system authority grant (pub(crate) - internal use only).
    pub(crate) fn system() -> Self {
        Self {
            grantor: AuthorityGrantor::System,
            granted_at: crate::types::Timestamp::now(),
            scope: None,
        }
    }

    /// Create a human authority grant (pub(crate) - requires human approval flow).
    pub(crate) fn human(approver_id: impl Into<String>) -> Self {
        Self {
            grantor: AuthorityGrantor::Human {
                approver_id: approver_id.into(),
            },
            granted_at: crate::types::Timestamp::now(),
            scope: None,
        }
    }

    /// Create a policy-delegated authority grant.
    pub(crate) fn policy(policy_id: impl Into<String>) -> Self {
        Self {
            grantor: AuthorityGrantor::Policy {
                policy_id: policy_id.into(),
            },
            granted_at: crate::types::Timestamp::now(),
            scope: None,
        }
    }

    /// Get the grantor.
    pub fn grantor(&self) -> &AuthorityGrantor {
        &self.grantor
    }

    /// Get the grant timestamp.
    pub fn granted_at(&self) -> &crate::types::Timestamp {
        &self.granted_at
    }

    /// Get the scope limitation, if any.
    pub fn scope(&self) -> Option<&AuthorityScope> {
        self.scope.as_ref()
    }

    /// Add a scope limitation.
    pub(crate) fn with_scope(mut self, scope: AuthorityScope) -> Self {
        self.scope = Some(scope);
        self
    }
}

/// Who/what granted promotion authority.
#[derive(Debug, Clone)]
pub enum AuthorityGrantor {
    /// System granted authority (automated gates).
    System,
    /// Human explicitly approved.
    Human {
        /// ID of the human approver
        approver_id: String,
    },
    /// Policy delegated authority.
    Policy {
        /// ID of the policy that granted authority
        policy_id: String,
    },
}

/// Scope limitation for authority grants.
#[derive(Debug, Clone)]
pub struct AuthorityScope {
    /// Limited to specific proposal kinds
    pub proposal_kinds: Option<Vec<crate::kernel_boundary::ProposalKind>>,
    /// Limited to specific gate IDs
    pub gate_ids: Option<Vec<crate::types::GateId>>,
    /// Time-limited (expires after this timestamp)
    pub expires_at: Option<crate::types::Timestamp>,
}

impl AuthorityScope {
    /// Create an empty scope (no limitations).
    pub fn new() -> Self {
        Self {
            proposal_kinds: None,
            gate_ids: None,
            expires_at: None,
        }
    }

    /// Limit to specific proposal kinds.
    pub fn with_proposal_kinds(mut self, kinds: Vec<crate::kernel_boundary::ProposalKind>) -> Self {
        self.proposal_kinds = Some(kinds);
        self
    }

    /// Limit to specific gates.
    pub fn with_gate_ids(mut self, ids: Vec<crate::types::GateId>) -> Self {
        self.gate_ids = Some(ids);
        self
    }

    /// Set expiration time.
    pub fn with_expiration(mut self, expires_at: crate::types::Timestamp) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

impl Default for AuthorityScope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_authority_grant() {
        let grant = AuthorityGrant::system();
        assert!(matches!(grant.grantor(), AuthorityGrantor::System));
        assert!(grant.scope().is_none());
    }

    #[test]
    fn human_authority_grant() {
        let grant = AuthorityGrant::human("user@example.com");
        match grant.grantor() {
            AuthorityGrantor::Human { approver_id } => {
                assert_eq!(approver_id, "user@example.com");
            }
            _ => panic!("Expected Human grantor"),
        }
    }

    #[test]
    fn policy_authority_grant_with_scope() {
        let scope = AuthorityScope::new()
            .with_proposal_kinds(vec![crate::kernel_boundary::ProposalKind::Claims]);

        let grant = AuthorityGrant::policy("auto-promote-claims").with_scope(scope);

        assert!(matches!(grant.grantor(), AuthorityGrantor::Policy { .. }));
        assert!(grant.scope().is_some());
        assert!(grant.scope().unwrap().proposal_kinds.is_some());
    }

    #[test]
    fn authority_scope_builder() {
        let scope = AuthorityScope::new()
            .with_proposal_kinds(vec![
                crate::kernel_boundary::ProposalKind::Claims,
                crate::kernel_boundary::ProposalKind::Plan,
            ])
            .with_gate_ids(vec![crate::types::GateId::new("gate-1")])
            .with_expiration(crate::types::Timestamp::new("2025-01-01T00:00:00Z"));

        assert_eq!(scope.proposal_kinds.as_ref().unwrap().len(), 2);
        assert_eq!(scope.gate_ids.as_ref().unwrap().len(), 1);
        assert!(scope.expires_at.is_some());
    }

    #[test]
    fn authority_scope_default() {
        let scope = AuthorityScope::default();
        assert!(scope.proposal_kinds.is_none());
        assert!(scope.gate_ids.is_none());
        assert!(scope.expires_at.is_none());
    }
}
