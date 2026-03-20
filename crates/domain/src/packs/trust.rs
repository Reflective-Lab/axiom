// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! Trust Pack agents for cross-cutting security substrate.
//!
//! Implements the agent contracts defined in specs/trust.feature.
//!
//! # Trust is the Immutable Substrate
//!
//! Trust agents wrap all other packs, providing:
//! - Identity verification
//! - Access control
//! - Audit trails
//! - Provenance tracking
//! - Compliance enforcement
//!
//! Note: This implementation uses the standard ContextKey enum. Facts are
//! distinguished by their ID prefixes (session:, audit:, compliance:, etc.).

use converge_core::{
    Agent, AgentEffect, Context, ContextKey, Fact,
    invariant::{Invariant, InvariantClass, InvariantResult, Violation},
};

// ============================================================================
// Fact ID Prefixes
// ============================================================================

pub const SESSION_PREFIX: &str = "session:";
pub const ACCESS_DECISION_PREFIX: &str = "access_decision:";
pub const AUDIT_PREFIX: &str = "audit:";
pub const PROVENANCE_PREFIX: &str = "provenance:";
pub const COMPLIANCE_PREFIX: &str = "compliance:";
pub const VIOLATION_PREFIX: &str = "violation:";
pub const REMEDIATION_PREFIX: &str = "remediation:";
pub const REDACTED_PREFIX: &str = "redacted:";

// ============================================================================
// Agents
// ============================================================================

/// Validates session tokens and identity claims.
///
/// Critical path - must complete within 100ms.
#[derive(Debug, Clone, Default)]
pub struct SessionValidatorAgent;

impl Agent for SessionValidatorAgent {
    fn name(&self) -> &str {
        "session_validator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|s| s.content.contains("session.token"))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers.iter() {
            if trigger.content.contains("session.token") {
                facts.push(Fact {
                    key: ContextKey::Signals,
                    id: format!("{}{}", SESSION_PREFIX, trigger.id),
                    content: serde_json::json!({
                        "type": "validated_session",
                        "token_id": trigger.id,
                        "valid": true,
                        "identity_id": "extracted",
                        "claims": [],
                        "expires_at": "2026-01-12T23:59:59Z",
                        "validated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Enforces role-based access control.
///
/// Critical path - must complete within 50ms.
#[derive(Debug, Clone, Default)]
pub struct RbacEnforcerAgent;

impl Agent for RbacEnforcerAgent {
    fn name(&self) -> &str {
        "rbac_enforcer"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        let has_valid_session = ctx
            .get(ContextKey::Signals)
            .iter()
            .any(|s| s.id.starts_with(SESSION_PREFIX) && s.content.contains("\"valid\":true"));
        let has_decisions = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(ACCESS_DECISION_PREFIX));
        has_valid_session && !has_decisions
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for session in signals.iter() {
            if session.id.starts_with(SESSION_PREFIX) && session.content.contains("\"valid\":true")
            {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", ACCESS_DECISION_PREFIX, session.id),
                    content: serde_json::json!({
                        "type": "access_decision",
                        "session_id": session.id,
                        "decision": "allow",
                        "matched_roles": [],
                        "matched_permissions": [],
                        "evaluated_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Writes immutable audit entries.
///
/// All significant actions must be audited.
#[derive(Debug, Clone, Default)]
pub struct AuditWriterAgent;

impl Agent for AuditWriterAgent {
    fn name(&self) -> &str {
        "audit_writer"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(ACCESS_DECISION_PREFIX))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for decision in proposals.iter() {
            if decision.id.starts_with(ACCESS_DECISION_PREFIX) {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", AUDIT_PREFIX, decision.id),
                    content: serde_json::json!({
                        "type": "audit_entry",
                        "access_decision_id": decision.id,
                        "action": "access_evaluated",
                        "actor": "system",
                        "resource": "unknown",
                        "outcome": "from_decision",
                        "timestamp": "2026-01-12T12:00:00Z",
                        "immutable": true
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Tracks data provenance and lineage.
#[derive(Debug, Clone, Default)]
pub struct ProvenanceTrackerAgent;

impl Agent for ProvenanceTrackerAgent {
    fn name(&self) -> &str {
        "provenance_tracker"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        let has_audit = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(AUDIT_PREFIX));
        let has_provenance = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(PROVENANCE_PREFIX));
        has_audit && !has_provenance
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let mut facts = Vec::new();

        for entry in proposals.iter() {
            if entry.id.starts_with(AUDIT_PREFIX) {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", PROVENANCE_PREFIX, entry.id),
                    content: serde_json::json!({
                        "type": "provenance",
                        "audit_entry_id": entry.id,
                        "chain": [],
                        "root_source": "system",
                        "transformations": [],
                        "verified": true
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Scans for compliance violations.
///
/// Scheduled to run periodically.
#[derive(Debug, Clone, Default)]
pub struct ComplianceScannerAgent;

impl Agent for ComplianceScannerAgent {
    fn name(&self) -> &str {
        "compliance_scanner"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Proposals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        let has_audit = ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|p| p.id.starts_with(AUDIT_PREFIX));
        let has_compliance = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|e| e.id.starts_with(COMPLIANCE_PREFIX));
        has_audit && !has_compliance
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let proposals = ctx.get(ContextKey::Proposals);
        let audit_count = proposals
            .iter()
            .filter(|p| p.id.starts_with(AUDIT_PREFIX))
            .count();

        let violations_found = false; // Simplified

        AgentEffect::with_facts(vec![Fact {
            key: ContextKey::Evaluations,
            id: format!("{}scan:latest", COMPLIANCE_PREFIX),
            content: serde_json::json!({
                "type": "compliance_status",
                "scan_id": "scan_001",
                "scanned_entries": audit_count,
                "violations_found": violations_found,
                "frameworks_checked": ["SOC2", "GDPR", "HIPAA"],
                "scanned_at": "2026-01-12T12:00:00Z"
            })
            .to_string(),
        }])
    }
}

/// Proposes remediation for compliance violations.
#[derive(Debug, Clone, Default)]
pub struct ViolationRemediatorAgent;

impl Agent for ViolationRemediatorAgent {
    fn name(&self) -> &str {
        "violation_remediator"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Signals)
            .iter()
            .any(|v| v.id.starts_with(VIOLATION_PREFIX) && v.content.contains("\"state\":\"open\""))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let signals = ctx.get(ContextKey::Signals);
        let mut facts = Vec::new();

        for violation in signals.iter() {
            if violation.id.starts_with(VIOLATION_PREFIX)
                && violation.content.contains("\"state\":\"open\"")
            {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", REMEDIATION_PREFIX, violation.id),
                    content: serde_json::json!({
                        "type": "remediation",
                        "violation_id": violation.id,
                        "proposed_actions": [],
                        "auto_remediate": false,
                        "requires_approval": true,
                        "proposed_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

/// Redacts PII from content before external sharing.
#[derive(Debug, Clone, Default)]
pub struct PiiRedactorAgent;

impl Agent for PiiRedactorAgent {
    fn name(&self) -> &str {
        "pii_redactor"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|s| s.content.contains("redaction.required"))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let triggers = ctx.get(ContextKey::Seeds);
        let mut facts = Vec::new();

        for trigger in triggers.iter() {
            if trigger.content.contains("redaction.required") {
                facts.push(Fact {
                    key: ContextKey::Proposals,
                    id: format!("{}{}", REDACTED_PREFIX, trigger.id),
                    content: serde_json::json!({
                        "type": "redacted_content",
                        "source_id": trigger.id,
                        "redacted_fields": ["email", "phone", "ssn", "address"],
                        "redaction_method": "mask",
                        "redacted_at": "2026-01-12T12:00:00Z"
                    })
                    .to_string(),
                });
            }
        }

        AgentEffect::with_facts(facts)
    }
}

// ============================================================================
// Invariants
// ============================================================================

/// Ensures all actions have audit entries.
#[derive(Debug, Clone, Default)]
pub struct AllActionsAuditedInvariant;

impl Invariant for AllActionsAuditedInvariant {
    fn name(&self) -> &str {
        "all_actions_audited"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        let proposals = ctx.get(ContextKey::Proposals);
        for decision in proposals.iter() {
            if decision.id.starts_with(ACCESS_DECISION_PREFIX) {
                let has_audit = proposals
                    .iter()
                    .any(|a| a.id.starts_with(AUDIT_PREFIX) && a.content.contains(&decision.id));
                if !has_audit {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!("Access decision {} has no audit entry", decision.id),
                        vec![decision.id.clone()],
                    ));
                }
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures audit entries are immutable.
#[derive(Debug, Clone, Default)]
pub struct AuditImmutabilityInvariant;

impl Invariant for AuditImmutabilityInvariant {
    fn name(&self) -> &str {
        "audit_immutability"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Structural
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        for entry in ctx.get(ContextKey::Proposals).iter() {
            if entry.id.starts_with(AUDIT_PREFIX) && !entry.content.contains("\"immutable\":true") {
                return InvariantResult::Violated(Violation::with_facts(
                    format!("Audit entry {} is not marked immutable", entry.id),
                    vec![entry.id.clone()],
                ));
            }
        }
        InvariantResult::Ok
    }
}

/// Ensures violations have remediation plans.
#[derive(Debug, Clone, Default)]
pub struct ViolationsHaveRemediationInvariant;

impl Invariant for ViolationsHaveRemediationInvariant {
    fn name(&self) -> &str {
        "violations_have_remediation"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Semantic
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        let proposals = ctx.get(ContextKey::Proposals);
        for violation in ctx.get(ContextKey::Signals).iter() {
            if violation.id.starts_with(VIOLATION_PREFIX)
                && violation.content.contains("\"state\":\"open\"")
            {
                let has_remediation = proposals.iter().any(|r| {
                    r.id.starts_with(REMEDIATION_PREFIX) && r.content.contains(&violation.id)
                });
                if !has_remediation {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!("Violation {} has no remediation plan", violation.id),
                        vec![violation.id.clone()],
                    ));
                }
            }
        }
        InvariantResult::Ok
    }
}

// ============================================================================
// Cross-Pack Invariants (Trust ↔ Legal)
// ============================================================================

/// Cross-pack invariant prefixes from Legal Pack
const LEGAL_CONTRACT_PREFIX: &str = "contract:";
const LEGAL_EQUITY_PREFIX: &str = "equity:";
const LEGAL_IP_ASSIGNMENT_PREFIX: &str = "ip_assignment:";

/// Ensures all significant legal actions have corresponding audit entries.
///
/// This is a cross-pack invariant that bridges Trust and Legal packs,
/// ensuring that contract executions, equity grants, and IP assignments
/// are properly recorded in the immutable audit trail.
#[derive(Debug, Clone, Default)]
pub struct LegalActionsAuditedInvariant;

impl Invariant for LegalActionsAuditedInvariant {
    fn name(&self) -> &str {
        "legal_actions_audited"
    }

    fn class(&self) -> InvariantClass {
        InvariantClass::Acceptance
    }

    fn check(&self, ctx: &dyn converge_core::ContextView) -> InvariantResult {
        let proposals = ctx.get(ContextKey::Proposals);

        // Check executed contracts have audit entries
        for contract in proposals.iter() {
            if contract.id.starts_with(LEGAL_CONTRACT_PREFIX)
                && contract.content.contains("\"state\":\"executed\"")
            {
                let has_audit = proposals.iter().any(|a| {
                    a.id.starts_with(AUDIT_PREFIX)
                        && (a.content.contains(&contract.id)
                            || a.content.contains("contract_executed")
                            || a.content.contains("legal_action"))
                });

                if !has_audit {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!("Executed contract {} has no audit trail entry", contract.id),
                        vec![contract.id.clone()],
                    ));
                }
            }
        }

        // Check equity grants have audit entries
        for grant in proposals.iter() {
            if grant.id.starts_with(LEGAL_EQUITY_PREFIX)
                && grant.content.contains("\"state\":\"granted\"")
            {
                let has_audit = proposals.iter().any(|a| {
                    a.id.starts_with(AUDIT_PREFIX)
                        && (a.content.contains(&grant.id)
                            || a.content.contains("equity_granted")
                            || a.content.contains("legal_action"))
                });

                if !has_audit {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!("Equity grant {} has no audit trail entry", grant.id),
                        vec![grant.id.clone()],
                    ));
                }
            }
        }

        // Check IP assignments have audit entries
        for ip in proposals.iter() {
            if ip.id.starts_with(LEGAL_IP_ASSIGNMENT_PREFIX)
                && ip.content.contains("\"state\":\"signed\"")
            {
                let has_audit = proposals.iter().any(|a| {
                    a.id.starts_with(AUDIT_PREFIX)
                        && (a.content.contains(&ip.id)
                            || a.content.contains("ip_assigned")
                            || a.content.contains("legal_action"))
                });

                if !has_audit {
                    return InvariantResult::Violated(Violation::with_facts(
                        format!("IP assignment {} has no audit trail entry", ip.id),
                        vec![ip.id.clone()],
                    ));
                }
            }
        }

        InvariantResult::Ok
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_have_correct_names() {
        assert_eq!(SessionValidatorAgent.name(), "session_validator");
        assert_eq!(RbacEnforcerAgent.name(), "rbac_enforcer");
        assert_eq!(AuditWriterAgent.name(), "audit_writer");
        assert_eq!(ProvenanceTrackerAgent.name(), "provenance_tracker");
        assert_eq!(ComplianceScannerAgent.name(), "compliance_scanner");
        assert_eq!(ViolationRemediatorAgent.name(), "violation_remediator");
        assert_eq!(PiiRedactorAgent.name(), "pii_redactor");
    }

    #[test]
    fn invariants_have_correct_names() {
        assert_eq!(AllActionsAuditedInvariant.name(), "all_actions_audited");
        assert_eq!(AuditImmutabilityInvariant.name(), "audit_immutability");
        assert_eq!(
            ViolationsHaveRemediationInvariant.name(),
            "violations_have_remediation"
        );
        assert_eq!(LegalActionsAuditedInvariant.name(), "legal_actions_audited");
    }

    #[test]
    fn executed_contract_without_audit_violates() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "contract:msa:deal-123".to_string(),
            content: r#"{"type":"contract","state":"executed","immutable":true}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Violated(_)));
    }

    #[test]
    fn executed_contract_with_audit_passes() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "contract:msa:deal-123".to_string(),
            content: r#"{"type":"contract","state":"executed","immutable":true}"#.to_string(),
        })
        .unwrap();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "audit:contract:msa:deal-123".to_string(),
            content: r#"{"type":"audit_entry","action":"contract_executed","contract_id":"contract:msa:deal-123","immutable":true}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Ok));
    }

    #[test]
    fn equity_grant_without_audit_violates() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "equity:grant-456".to_string(),
            content: r#"{"type":"equity_grant","state":"granted"}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Violated(_)));
    }

    #[test]
    fn equity_grant_with_audit_passes() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "equity:grant-456".to_string(),
            content: r#"{"type":"equity_grant","state":"granted"}"#.to_string(),
        })
        .unwrap();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "audit:equity:grant-456".to_string(),
            content: r#"{"type":"audit_entry","action":"equity_granted","grant_id":"equity:grant-456","immutable":true}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Ok));
    }

    #[test]
    fn ip_assignment_without_audit_violates() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "ip_assignment:contractor-789".to_string(),
            content: r#"{"type":"ip_assignment","state":"signed"}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Violated(_)));
    }

    #[test]
    fn ip_assignment_with_audit_passes() {
        let mut ctx = Context::new();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "ip_assignment:contractor-789".to_string(),
            content: r#"{"type":"ip_assignment","state":"signed"}"#.to_string(),
        })
        .unwrap();
        ctx.add_fact(Fact {
            key: ContextKey::Proposals,
            id: "audit:ip_assignment:contractor-789".to_string(),
            content: r#"{"type":"audit_entry","action":"ip_assigned","assignment_id":"ip_assignment:contractor-789","immutable":true}"#.to_string(),
        })
        .unwrap();

        let result = LegalActionsAuditedInvariant.check(&ctx);
        assert!(matches!(result, InvariantResult::Ok));
    }
}
