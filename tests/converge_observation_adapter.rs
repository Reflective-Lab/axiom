//! Wire recipe: adapt a Converge run into an `AxiomRunObservation`.
//!
//! Axiom's `truth_package` module deliberately does **not** depend on
//! `converge-kernel` at the lib level — the AGENTS.md doctrine constrains
//! Axiom's runtime dep closure to the narrow Converge capability contract
//! (`converge-provider` + `converge-pack`). Callers that already pull in
//! `converge-kernel` (atelier-showcase scenarios, helms, integration tests)
//! supply their own thin adapter from typed Converge values into Axiom's
//! wire-free `ObservedStopReason` enum.
//!
//! This file is the canonical reference implementation. Copy the
//! `convert_stop_reason` helper into your scenario or runtime; the surface
//! is stable because both sides mirror the same vocabulary intentionally.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservedStopReason, PromotedFactRecord, RunIntegrityProof,
    TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use converge_kernel::StopReason as ConvergeStopReason;
use converge_pack::{ApprovalPointId, CriterionId, GateId, ProposalId};

/// Adapter recipe: map a Converge `StopReason` into Axiom's wire-free
/// `ObservedStopReason`. Typed Id wrappers stringify via `as_str`; the
/// `InvariantClass` and `ErrorCategory` enums stringify via Debug (variant
/// names are part of Converge's public surface).
fn convert_stop_reason(reason: &ConvergeStopReason) -> ObservedStopReason {
    match reason {
        ConvergeStopReason::Converged => ObservedStopReason::Converged,
        ConvergeStopReason::CriteriaMet { criteria } => ObservedStopReason::CriteriaMet {
            criteria: criteria.iter().map(|c| c.as_str().to_string()).collect(),
        },
        ConvergeStopReason::UserCancelled => ObservedStopReason::UserCancelled,
        ConvergeStopReason::HumanInterventionRequired {
            criteria,
            approval_refs,
        } => ObservedStopReason::HumanInterventionRequired {
            criteria: criteria.iter().map(|c| c.as_str().to_string()).collect(),
            approval_refs: approval_refs
                .iter()
                .map(|a| a.as_str().to_string())
                .collect(),
        },
        ConvergeStopReason::CycleBudgetExhausted {
            cycles_executed,
            limit,
        } => ObservedStopReason::CycleBudgetExhausted {
            cycles_executed: *cycles_executed,
            limit: *limit,
        },
        ConvergeStopReason::FactBudgetExhausted { facts_count, limit } => {
            ObservedStopReason::FactBudgetExhausted {
                facts_count: *facts_count,
                limit: *limit,
            }
        }
        ConvergeStopReason::TokenBudgetExhausted {
            tokens_consumed,
            limit,
        } => ObservedStopReason::TokenBudgetExhausted {
            tokens_consumed: *tokens_consumed,
            limit: *limit,
        },
        ConvergeStopReason::TimeBudgetExhausted {
            duration_ms,
            limit_ms,
        } => ObservedStopReason::TimeBudgetExhausted {
            duration_ms: *duration_ms,
            limit_ms: *limit_ms,
        },
        ConvergeStopReason::InvariantViolated {
            class,
            name,
            reason,
        } => ObservedStopReason::InvariantViolated {
            class: format!("{class:?}").to_lowercase(),
            name: name.clone(),
            reason: reason.clone(),
        },
        ConvergeStopReason::PromotionRejected {
            proposal_id,
            reason,
        } => ObservedStopReason::PromotionRejected {
            proposal_id: proposal_id.as_str().to_string(),
            reason: reason.clone(),
        },
        ConvergeStopReason::Error { message, category } => ObservedStopReason::RuntimeError {
            message: message.clone(),
            category: format!("{category:?}").to_lowercase(),
        },
        ConvergeStopReason::AgentRefused { agent_id, reason } => ObservedStopReason::AgentRefused {
            agent_id: agent_id.clone(),
            reason: reason.clone(),
        },
        ConvergeStopReason::HitlGatePending {
            gate_id,
            proposal_id,
            summary,
            agent_id,
            cycle,
        } => ObservedStopReason::HitlGatePending {
            gate_id: gate_id.as_str().to_string(),
            proposal_id: proposal_id.as_str().to_string(),
            summary: summary.clone(),
            agent_id: agent_id.clone(),
            cycle: *cycle,
        },
        // Converge's StopReason is `#[non_exhaustive]`. Unknown variants
        // surface as RuntimeError so the verifier still produces a verdict
        // instead of panicking.
        _ => ObservedStopReason::RuntimeError {
            message: format!("unrecognized Converge stop reason: {reason:?}"),
            category: "unknown".to_string(),
        },
    }
}

#[test]
fn converged_maps_to_axiom_converged() {
    let mapped = convert_stop_reason(&ConvergeStopReason::Converged);
    assert!(matches!(mapped, ObservedStopReason::Converged));
}

#[test]
fn criteria_met_carries_criterion_strings() {
    let reason = ConvergeStopReason::criteria_met(vec![CriterionId::new("approval-ready")]);
    let mapped = convert_stop_reason(&reason);
    match mapped {
        ObservedStopReason::CriteriaMet { criteria } => {
            assert_eq!(criteria, vec!["approval-ready".to_string()]);
        }
        other => panic!("expected CriteriaMet, got {other:?}"),
    }
}

#[test]
fn time_budget_exhausted_round_trips_through_adapter() {
    let reason = ConvergeStopReason::time_budget_exhausted(900_000, 600_000);
    let mapped = convert_stop_reason(&reason);
    match mapped {
        ObservedStopReason::TimeBudgetExhausted {
            duration_ms,
            limit_ms,
        } => {
            assert_eq!(duration_ms, 900_000);
            assert_eq!(limit_ms, 600_000);
        }
        other => panic!("expected TimeBudgetExhausted, got {other:?}"),
    }
}

#[test]
fn hitl_gate_pending_carries_gate_metadata() {
    let reason = ConvergeStopReason::HitlGatePending {
        gate_id: GateId::new("gate.escrow-release"),
        proposal_id: ProposalId::new("proposal.escrow.7"),
        summary: "buyer approval required before release".to_string(),
        agent_id: "policy-gate".to_string(),
        cycle: 3,
    };
    let mapped = convert_stop_reason(&reason);
    match mapped {
        ObservedStopReason::HitlGatePending {
            gate_id,
            proposal_id,
            summary,
            agent_id,
            cycle,
        } => {
            assert_eq!(gate_id, "gate.escrow-release");
            assert_eq!(proposal_id, "proposal.escrow.7");
            assert_eq!(summary, "buyer approval required before release");
            assert_eq!(agent_id, "policy-gate");
            assert_eq!(cycle, 3);
        }
        other => panic!("expected HitlGatePending, got {other:?}"),
    }
}

#[test]
fn human_intervention_required_maps_approval_refs() {
    let reason = ConvergeStopReason::human_intervention_required(
        vec![CriterionId::new("auditor-signoff")],
        vec![ApprovalPointId::new("audit.workflow.42")],
    );
    let mapped = convert_stop_reason(&reason);
    match mapped {
        ObservedStopReason::HumanInterventionRequired {
            criteria,
            approval_refs,
        } => {
            assert_eq!(criteria, vec!["auditor-signoff".to_string()]);
            assert_eq!(approval_refs, vec!["audit.workflow.42".to_string()]);
        }
        other => panic!("expected HumanInterventionRequired, got {other:?}"),
    }
}

#[test]
fn synthetic_converge_run_verifies_against_axiom_truth_package() {
    let package = decode_jtbd(small_jtbd()).expect("JTBD decodes");
    let evidence = evidence_clause(&package, "vendor_assessment");
    let observation = AxiomRunObservation {
        stop_reason: convert_stop_reason(&ConvergeStopReason::Converged),
        promoted_facts: vec![promoted_fact(
            "Evidence",
            "fact.vendor_assessment",
            "vendor assessment captured by the live Converge run",
            vec![evidence],
        )],
        integrity: RunIntegrityProof::sha256_merkle("sha256:converge-live", 7, 3),
        replay_notes: vec!["adapter mapped converge_kernel::StopReason::Converged".to_string()],
        run_stages: Vec::new(),
    };

    let report = AxiomRunReport::verify(&package, observation);
    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    let audit = report
        .audit_fact_lineage(&package)
        .expect("adapter-built observation passes the lineage audit");
    assert_eq!(audit.facts_audited, 1);
    assert_eq!(audit.evidence_coverage.len(), 1);
}

fn small_jtbd() -> JtbdInput {
    JtbdInput {
        key: "Adapter Smoke".to_string(),
        actor: "integration runner".to_string(),
        functional_job: "verify a converge-backed observation".to_string(),
        so_that: "the verifier can be exercised end to end against real Converge types".to_string(),
        evidence_required: vec![ClauseInput::with_key(
            "vendor_assessment",
            "vendor assessment",
        )],
        failure_modes: vec![ClauseInput::new("bypassed approval")],
        time_budget: Some(TimeBudget::from_minutes(5)),
    }
}

fn evidence_clause(package: &TruthPackage, key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .find(|clause| clause.key == key)
        .map_or_else(
            || panic!("missing evidence clause {key}"),
            |clause| clause.id.clone(),
        )
}

fn promoted_fact(
    context_key: &str,
    fact_id: &str,
    summary: &str,
    source_clause_ids: Vec<ClauseId>,
) -> PromotedFactRecord {
    PromotedFactRecord {
        context_key: context_key.to_string(),
        fact_id: fact_id.to_string(),
        summary: summary.to_string(),
        source_clause_ids,
        evidence_refs: vec![EvidenceRefRecord {
            evidence_id: format!("evidence.{fact_id}"),
            source: "converge-adapter-test".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace.{fact_id}"),
            location: Some("fixture://converge-adapter".to_string()),
            replayable: true,
        }),
        promotion_authority: None,
    }
}
