//! Triage Keeper — sixth app probe for the Axiom/Helm contract.
//!
//! The earlier probes proved release, integration, sensemaking, sourcing, and
//! compliance registry publication. Triage Keeper pressures the same contract
//! with sustaining-care work: dependency alerts, recurring defects, SLA/risk
//! ranking, patch plans, checks, approvals, deferrals, and the boundary that
//! maintenance triage does not become production deploy authority.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const TRIAGE_DEPENDENCY_ALERT_TRUTH_KEY: &str = "dependency-alert-cited";
const TRIAGE_RECURRING_DEFECT_TRUTH_KEY: &str = "recurring-defect-linked";
const TRIAGE_SERVICE_IMPACT_TRUTH_KEY: &str = "service-impact-cited";
const TRIAGE_SLA_RISK_TRUTH_KEY: &str = "sla-risk-ranked";
const TRIAGE_PATCH_PLAN_TRUTH_KEY: &str = "patch-plan-with-rollback";
const TRIAGE_CHECKS_TRUTH_KEY: &str = "checks-recorded";
const TRIAGE_CLIENT_APPROVAL_TRUTH_KEY: &str = "client-approval-recorded";
const TRIAGE_EMERGENCY_POLICY_TRUTH_KEY: &str = "emergency-policy-named";
const TRIAGE_DEFERRED_RISK_TRUTH_KEY: &str = "deferred-risk-recorded";
const TRIAGE_PRODUCTION_BOUNDARY_TRUTH_KEY: &str = "triage-not-deploy-authority";
const TRIAGE_ADAPTER_ID: &str = "triage-keeper.maintenance-cycle-to-axiom-observation";
const TRIAGE_ADAPTER_VERSION: &str = "fixture.v0.1";
const TRIAGE_MAINTENANCE_TRANSCRIPT: &str =
    include_str!("fixtures/triage_keeper_maintenance_transcript.json");

fn triage_weekly_maintenance_jtbd() -> JtbdInput {
    JtbdInput {
        key: "triage-weekly-maintenance".to_string(),
        actor: "maintenance lead".to_string(),
        functional_job:
            "turn dependency alerts and recurring defects into a governed weekly maintenance plan"
                .to_string(),
        so_that:
            "the client application stays secure and stable without surprise production changes"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "dependency_alert_cited",
                "the maintenance plan cites affected package, versions, advisory or exploit evidence, and impacted service",
            ),
            ClauseInput::with_key(
                "recurring_defect_linked",
                "the plan links a recurring defect to incident history, failure signature, and affected service",
            ),
            ClauseInput::with_key(
                "service_impact_cited",
                "the plan names the client service impacted by the alert or recurring defect",
            ),
            ClauseInput::with_key(
                "sla_risk_ranked",
                "dependency and defect work is ranked against SLA, severity, exploit evidence, and regression risk",
            ),
            ClauseInput::with_key(
                "patch_plan_with_rollback",
                "the patch plan names scope, maintenance window, deploy action, and rollback reference",
            ),
            ClauseInput::with_key(
                "checks_recorded",
                "relevant unit, integration, smoke, or stated-gap checks are recorded before safe-update language",
            ),
            ClauseInput::with_key(
                "client_approval_state",
                "client approval state is recorded before any production change is treated as ready",
            ),
            ClauseInput::with_key(
                "emergency_policy_state",
                "emergency work names the policy and authority that allowed escalation",
            ),
            ClauseInput::with_key(
                "deferred_risk_recorded",
                "deferred work records residual risk, reason, owner, and next review date",
            ),
            ClauseInput::with_key(
                "production_boundary",
                "Triage Keeper records that it recommends, drafts, and schedules but does not deploy production changes without client authority",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "alert_without_package",
                "a dependency alert is recommended without affected package, version, advisory, or impacted service",
            ),
            ClauseInput::with_key(
                "defect_without_history",
                "a recurring defect is prioritized without incident history or failure signature",
            ),
            ClauseInput::with_key(
                "hidden_service_impact",
                "maintenance priority is assigned without naming the impacted client service",
            ),
            ClauseInput::with_key(
                "sla_breach_hidden",
                "an SLA-relevant item is deprioritized without visible risk and due-date rationale",
            ),
            ClauseInput::with_key(
                "patch_without_rollback",
                "a patch plan is marked ready without rollback support",
            ),
            ClauseInput::with_key(
                "unsafe_update_marked_safe",
                "a dependency update is marked safe without relevant checks or an explicit stated gap",
            ),
            ClauseInput::with_key(
                "production_change_without_approval",
                "a production patch proceeds or is presented as deploy-ready without client approval",
            ),
            ClauseInput::with_key(
                "emergency_without_policy",
                "emergency maintenance is escalated without naming the governing policy",
            ),
            ClauseInput::with_key(
                "deferred_work_without_risk",
                "work is deferred without residual risk and next review date",
            ),
            ClauseInput::with_key(
                "triage_as_deploy_authority",
                "Triage Keeper is treated as the production deploy authority instead of a sustaining-care planner",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(35)),
    }
}

#[test]
fn triage_weekly_maintenance_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Turn dependency alerts and recurring defects")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        10
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        10
    );
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("affected package"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("production deploy authority"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn triage_maintenance_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");
    let transcript = triage_maintenance_transcript();

    let observation =
        adapt_triage_maintenance_transcript(&package, &transcript).expect("Triage adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.triage-maintenance-cycle")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Triage-adapted maintenance run preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 10);
    assert_eq!(audit.failure_coverage.len(), 10);
    assert_eq!(audit.facts_audited, 10);
}

#[test]
fn triage_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");
    let transcript = triage_maintenance_transcript();

    let first = adapt_triage_maintenance_transcript_with_receipt(&package, &transcript);
    let second = adapt_triage_maintenance_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, TRIAGE_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "triage-keeper");
    assert_eq!(first.receipt.source_run_id, transcript.source.run_id);
    assert_eq!(first.receipt.package_id, package.package_id);
    assert_eq!(first.receipt.truth_version, package.truth_version);
    assert_eq!(first.receipt.domain_hint, transcript.source.domain_hint);
    assert!(first.receipt.source_transcript_hash.starts_with("sha256:"));
    assert!(
        first
            .receipt
            .observation_hash
            .as_ref()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );
    assert_eq!(
        first.receipt.mapped_fact_ids,
        vec![
            "triage.maintenance.dependency-alert",
            "triage.maintenance.recurring-defect",
            "triage.maintenance.service-impact",
            "triage.maintenance.sla-risk-ranking",
            "triage.maintenance.patch-plan",
            "triage.maintenance.checks",
            "triage.maintenance.client-approval",
            "triage.maintenance.emergency-policy",
            "triage.maintenance.deferred-risk",
            "triage.maintenance.production-boundary",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 20);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("openssl"));
    assert!(!serialized.contains("checkout-api"));
    assert!(!serialized.contains("ticket://client-ledger/TK-481"));
    assert!(!serialized.contains("TRIAGE_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/triage-keeper"));
}

#[test]
fn triage_job_readiness_packet_marks_missing_checks() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");
    let mut transcript = triage_maintenance_transcript();
    transcript
        .maintenance_run
        .truth_keys
        .retain(|truth_key| truth_key != TRIAGE_CHECKS_TRUTH_KEY);
    transcript.maintenance_run.checks.status = "NotRun".to_string();
    transcript.maintenance_run.checks.test_refs.clear();
    transcript
        .maintenance_run
        .checks
        .missing_test_gaps
        .push("checkout-api smoke coverage missing".to_string());
    let adapter_outcome = adapt_triage_maintenance_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let checks_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "checks_recorded")
        .expect("checks evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(checks_status.status, EvidenceReadinessStatus::Missing);
    assert!(checks_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for checks_recorded".to_string())
    );
}

#[test]
fn triage_job_readiness_packet_marks_patch_without_client_approval() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");
    let mut transcript = triage_maintenance_transcript();
    transcript
        .maintenance_run
        .truth_keys
        .retain(|truth_key| truth_key != TRIAGE_CLIENT_APPROVAL_TRUTH_KEY);
    transcript.maintenance_run.approval.status = "Pending".to_string();
    let adapter_outcome = adapt_triage_maintenance_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "client_approval_state")
        .expect("client approval evidence is represented");
    let plan_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "patch_plan_with_rollback")
        .expect("patch plan evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(plan_status.status, EvidenceReadinessStatus::Present);
    assert!(approval_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for client_approval_state".to_string())
    );
}

#[test]
fn triage_operator_ledger_entries_are_deterministic_backlinks_without_deploy_authority() {
    let package = decode_jtbd(triage_weekly_maintenance_jtbd()).expect("Triage JTBD decodes");
    let transcript = triage_maintenance_transcript();
    let adapter_outcome = adapt_triage_maintenance_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let decision_receipt = operator_decision_receipt(&packet, &transcript);
    let approval_receipt = client_approval_receipt(&packet, &transcript, &decision_receipt);
    let plan_receipt =
        maintenance_plan_receipt(&packet, &transcript, &decision_receipt, &approval_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &decision_receipt,
        &approval_receipt,
        &plan_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &decision_receipt,
        &approval_receipt,
        &plan_receipt,
    );

    assert_eq!(first, second);
    assert_eq!(first.len(), 5);
    assert_eq!(
        first[0].record_kind,
        HelmLedgerRecordKind::ObservationAdapterReceipt
    );
    assert_eq!(
        first[1].record_kind,
        HelmLedgerRecordKind::JobReadinessPacket
    );
    assert_eq!(
        first[2].record_kind,
        HelmLedgerRecordKind::OperatorDecisionReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::ClientApprovalReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::MaintenancePlanReceipt
    );
    assert_eq!(
        first[1].backlink_ids,
        vec![adapter_outcome.receipt.receipt_id.as_str().to_string()]
    );
    assert_eq!(
        first[2].backlink_ids,
        vec![packet.packet_id.as_str().to_string()]
    );
    assert_eq!(
        first[3].backlink_ids,
        vec![
            packet.packet_id.as_str().to_string(),
            decision_receipt.receipt_id.clone(),
        ]
    );
    assert_eq!(
        first[4].backlink_ids,
        vec![
            packet.packet_id.as_str().to_string(),
            decision_receipt.receipt_id.clone(),
            approval_receipt.receipt_id.clone(),
        ]
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("openssl"));
    assert!(!serialized.contains("checkout-api"));
    assert!(!serialized.contains("client-ledger.ops-owner"));
    assert!(!serialized.contains("ticket://client-ledger/TK-481"));
    assert!(!serialized.contains("TRIAGE_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/triage-keeper"));
}

fn adapt_triage_maintenance_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &TriageKeeperMaintenanceTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_triage_maintenance_transcript(package, transcript) {
        Ok(observation) => {
            let observation_hash = sha256_json(&observation);
            let mapped_fact_ids = observation
                .promoted_facts
                .iter()
                .map(|fact| fact.fact_id.clone())
                .collect::<Vec<_>>();
            let mapped_clause_ids = observation
                .promoted_facts
                .iter()
                .flat_map(|fact| fact.source_clause_ids.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let receipt = observation_adapter_receipt(
                package,
                transcript,
                ObservationAdapterStatus::Succeeded,
                source_transcript_hash,
                Some(observation_hash),
                mapped_fact_ids,
                mapped_clause_ids,
                Vec::new(),
            );

            ObservationAdapterOutcome {
                observation: Some(observation),
                receipt,
            }
        }
        Err(error) => {
            let receipt = observation_adapter_receipt(
                package,
                transcript,
                ObservationAdapterStatus::Rejected,
                source_transcript_hash,
                None,
                Vec::new(),
                Vec::new(),
                vec![error],
            );

            ObservationAdapterOutcome {
                observation: None,
                receipt,
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn observation_adapter_receipt(
    package: &TruthPackage,
    transcript: &TriageKeeperMaintenanceTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: TRIAGE_ADAPTER_ID.to_string(),
        adapter_version: TRIAGE_ADAPTER_VERSION.to_string(),
        status,
        source_app: "triage-keeper".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "triage://maintenance-cycle/{}/{}",
            transcript.source.run_id, transcript.maintenance_run.maintenance_run_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "dependency_alert.package".to_string(),
            "dependency_alert.advisory_ref".to_string(),
            "recurring_defect.incident_refs".to_string(),
            "recurring_defect.owner".to_string(),
            "source.command".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &TriageKeeperMaintenanceTranscript,
    adapter_outcome: &ObservationAdapterOutcome,
) -> JobReadinessPacket {
    let report = adapter_outcome
        .observation
        .clone()
        .map(|observation| AxiomRunReport::verify(package, observation));
    let promoted_facts = report
        .as_ref()
        .map_or_else(Vec::new, |report| report.promoted_facts.clone());
    let evidence_status = package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .map(|clause| {
            let fact_ids = promoted_facts
                .iter()
                .filter(|fact| fact.source_clause_ids.contains(&clause.id))
                .map(|fact| fact.fact_id.clone())
                .collect::<Vec<_>>();
            let status = if fact_ids.is_empty() {
                EvidenceReadinessStatus::Missing
            } else {
                EvidenceReadinessStatus::Present
            };

            JobEvidenceStatus {
                clause_id: clause.id.to_string(),
                clause_key: clause.key.clone(),
                label: clause.text.clone(),
                status,
                fact_ids,
            }
        })
        .collect::<Vec<_>>();
    let operator_actions = job_readiness_operator_actions(
        adapter_outcome.receipt.status,
        &evidence_status,
        report.as_ref(),
    );
    let packet_id = job_readiness_packet_id(
        package,
        &transcript.source.domain_hint,
        &transcript.maintenance_run.maintenance_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!(
            "triage://maintenance-cycle/{}",
            transcript.maintenance_run.maintenance_run_id
        ),
        adapter_receipt_id: adapter_outcome.receipt.receipt_id.as_str().to_string(),
        adapter_status: adapter_outcome.receipt.status,
        verdict: report.as_ref().map(|report| report.verdict),
        authorizes_domain_action: false,
        evidence_status,
        verifier_forbidden_actions: package
            .verifier_spec
            .forbidden_actions
            .iter()
            .map(|action| action.action.clone())
            .collect(),
        operator_actions,
    }
}

fn job_readiness_operator_actions(
    adapter_status: ObservationAdapterStatus,
    evidence_status: &[JobEvidenceStatus],
    report: Option<&AxiomRunReport>,
) -> Vec<String> {
    let mut actions = Vec::new();
    if adapter_status == ObservationAdapterStatus::Rejected {
        actions.push("inspect adapter receipt errors".to_string());
        return actions;
    }

    actions.push("inspect axiom report".to_string());
    actions.extend(
        evidence_status
            .iter()
            .filter(|status| status.status == EvidenceReadinessStatus::Missing)
            .map(|status| format!("request missing evidence for {}", status.clause_key)),
    );
    if report.is_some_and(|report| report.verdict != AxiomRunVerdict::Satisfied) {
        actions.push("rerun verification after evidence changes".to_string());
    }
    actions.push("route maintenance plan through Helm operator review".to_string());
    actions.push("confirm deploy action stays behind client production authority".to_string());
    actions
}

fn operator_decision_receipt(
    packet: &JobReadinessPacket,
    transcript: &TriageKeeperMaintenanceTranscript,
) -> OperatorDecisionReceipt {
    let ranking = &transcript.maintenance_run.risk_ranking;
    OperatorDecisionReceipt {
        receipt_id: operator_decision_receipt_id(packet, ranking),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        maintenance_run_ref: packet.subject_ref.clone(),
        decision_ref_hash: sha256_lines(&[
            transcript.maintenance_run.maintenance_run_id.as_str(),
            ranking.top_item.as_str(),
        ]),
        top_item_kind: ranking.top_item.clone(),
        status: "ready_for_client_review".to_string(),
        ranking_hash: sha256_json(ranking),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn client_approval_receipt(
    packet: &JobReadinessPacket,
    transcript: &TriageKeeperMaintenanceTranscript,
    decision_receipt: &OperatorDecisionReceipt,
) -> ClientApprovalReceipt {
    let approval = &transcript.maintenance_run.approval;
    ClientApprovalReceipt {
        receipt_id: client_approval_receipt_id(packet, approval, decision_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        approval_ref_hash: sha256_lines(&[approval.approval_id.as_str()]),
        status: approval.status.clone(),
        scope_hash: sha256_lines(&[approval.scope.as_str()]),
        note_hash: approval.note_hash.clone(),
        job_readiness_packet_id: packet.packet_id.clone(),
        operator_decision_receipt_id: decision_receipt.receipt_id.clone(),
    }
}

fn maintenance_plan_receipt(
    packet: &JobReadinessPacket,
    transcript: &TriageKeeperMaintenanceTranscript,
    decision_receipt: &OperatorDecisionReceipt,
    approval_receipt: &ClientApprovalReceipt,
) -> MaintenancePlanReceipt {
    let plan = &transcript.maintenance_run.patch_plan;
    MaintenancePlanReceipt {
        receipt_id: maintenance_plan_receipt_id(packet, plan, approval_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        plan_ref_hash: sha256_lines(&[plan.plan_id.as_str()]),
        status: "scheduled_after_checks_and_approval".to_string(),
        target_window_hash: sha256_lines(&[plan.target_window.as_str()]),
        rollback_ref_hash: sha256_lines(&[plan.rollback_ref.as_str()]),
        job_readiness_packet_id: packet.packet_id.clone(),
        operator_decision_receipt_id: decision_receipt.receipt_id.clone(),
        approval_receipt_id: approval_receipt.receipt_id.clone(),
    }
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    decision_receipt: &OperatorDecisionReceipt,
    approval_receipt: &ClientApprovalReceipt,
    plan_receipt: &MaintenancePlanReceipt,
) -> Vec<HelmLedgerEntry> {
    let receipt_payload_hash = sha256_json(receipt);
    let packet_payload_hash = sha256_json(packet);
    let decision_payload_hash = sha256_json(decision_receipt);
    let approval_payload_hash = sha256_json(approval_receipt);
    let plan_payload_hash = sha256_json(plan_receipt);

    vec![
        helm_ledger_entry(
            0,
            HelmLedgerRecordKind::ObservationAdapterReceipt,
            receipt.receipt_id.as_str().to_string(),
            receipt.package_id.as_str().to_string(),
            receipt.truth_version.clone(),
            receipt.domain_hint.clone(),
            receipt_payload_hash,
            Vec::new(),
            format!("adapter {} {}", receipt.adapter_id, receipt.status.as_str()),
        ),
        helm_ledger_entry(
            1,
            HelmLedgerRecordKind::JobReadinessPacket,
            packet.packet_id.clone(),
            packet.package_id.clone(),
            packet.truth_version.clone(),
            packet.domain_hint.clone(),
            packet_payload_hash,
            vec![receipt.receipt_id.as_str().to_string()],
            format!("job readiness {:?} for {}", packet.verdict, packet.job_key),
        ),
        helm_ledger_entry(
            2,
            HelmLedgerRecordKind::OperatorDecisionReceipt,
            decision_receipt.receipt_id.clone(),
            decision_receipt.package_id.clone(),
            decision_receipt.truth_version.clone(),
            decision_receipt.domain_hint.clone(),
            decision_payload_hash,
            vec![packet.packet_id.clone()],
            format!("operator decision {}", decision_receipt.status),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::ClientApprovalReceipt,
            approval_receipt.receipt_id.clone(),
            approval_receipt.package_id.clone(),
            approval_receipt.truth_version.clone(),
            approval_receipt.domain_hint.clone(),
            approval_payload_hash,
            vec![
                packet.packet_id.clone(),
                decision_receipt.receipt_id.clone(),
            ],
            format!("client approval {}", approval_receipt.status),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::MaintenancePlanReceipt,
            plan_receipt.receipt_id.clone(),
            plan_receipt.package_id.clone(),
            plan_receipt.truth_version.clone(),
            plan_receipt.domain_hint.clone(),
            plan_payload_hash,
            vec![
                packet.packet_id.clone(),
                decision_receipt.receipt_id.clone(),
                approval_receipt.receipt_id.clone(),
            ],
            format!("maintenance plan {}", plan_receipt.status),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn helm_ledger_entry(
    sequence: u64,
    record_kind: HelmLedgerRecordKind,
    source_ref: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    payload_hash: String,
    backlink_ids: Vec<String>,
    summary: String,
) -> HelmLedgerEntry {
    let backlinks_for_id = backlink_ids.join("\n");
    let sequence_for_id = sequence.to_string();
    let entry_id = helm_ledger_entry_id(&[
        record_kind.as_str(),
        &sequence_for_id,
        &source_ref,
        &package_id,
        &truth_version,
        &domain_hint,
        &payload_hash,
        &backlinks_for_id,
    ]);

    HelmLedgerEntry {
        entry_id,
        sequence,
        record_kind,
        source_ref,
        package_id,
        truth_version,
        domain_hint,
        payload_hash,
        backlink_ids,
        authority_effect: HelmLedgerAuthorityEffect::None,
        summary,
    }
}

fn adapt_triage_maintenance_transcript(
    package: &TruthPackage,
    transcript: &TriageKeeperMaintenanceTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.maintenance_run;
    if run.status != "Converged" {
        return Err("expected Triage maintenance run to converge before adaptation".to_string());
    }
    if run.dependency_alert.package.trim().is_empty() {
        return Err(
            "expected Triage maintenance run to carry dependency alert package".to_string(),
        );
    }
    if run.recurring_defect.incident_refs.is_empty() {
        return Err(
            "expected Triage maintenance run to carry recurring defect history".to_string(),
        );
    }

    let dependency_alert_cited = evidence_clause_id(package, "dependency_alert_cited");
    let recurring_defect_linked = evidence_clause_id(package, "recurring_defect_linked");
    let service_impact_cited = evidence_clause_id(package, "service_impact_cited");
    let sla_risk_ranked = evidence_clause_id(package, "sla_risk_ranked");
    let patch_plan_with_rollback = evidence_clause_id(package, "patch_plan_with_rollback");
    let checks_recorded = evidence_clause_id(package, "checks_recorded");
    let client_approval_state = evidence_clause_id(package, "client_approval_state");
    let emergency_policy_state = evidence_clause_id(package, "emergency_policy_state");
    let deferred_risk_recorded = evidence_clause_id(package, "deferred_risk_recorded");
    let production_boundary = evidence_clause_id(package, "production_boundary");
    let alert_without_package = failure_clause_id(package, "alert_without_package");
    let defect_without_history = failure_clause_id(package, "defect_without_history");
    let hidden_service_impact = failure_clause_id(package, "hidden_service_impact");
    let sla_breach_hidden = failure_clause_id(package, "sla_breach_hidden");
    let patch_without_rollback = failure_clause_id(package, "patch_without_rollback");
    let unsafe_update_marked_safe = failure_clause_id(package, "unsafe_update_marked_safe");
    let production_change_without_approval =
        failure_clause_id(package, "production_change_without_approval");
    let emergency_without_policy = failure_clause_id(package, "emergency_without_policy");
    let deferred_work_without_risk = failure_clause_id(package, "deferred_work_without_risk");
    let triage_as_deploy_authority = failure_clause_id(package, "triage_as_deploy_authority");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&run.truth_keys, TRIAGE_DEPENDENCY_ALERT_TRUTH_KEY)
        && !run.dependency_alert.package.trim().is_empty()
        && !run.dependency_alert.current_version.trim().is_empty()
        && !run.dependency_alert.patched_version.trim().is_empty()
        && !run.dependency_alert.advisory_ref.trim().is_empty()
        && !run.dependency_alert.affected_service.trim().is_empty()
        && run.dependency_alert.source_hash.starts_with("sha256:")
    {
        promoted_facts.push(triage_fact(
            "DependencyAlert",
            "triage.maintenance.dependency-alert",
            "dependency alert cites affected package, versions, advisory, impacted service, and source hash",
            vec![dependency_alert_cited, alert_without_package],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_RECURRING_DEFECT_TRUTH_KEY)
        && run.recurring_defect.recurrence_count > 1
        && !run.recurring_defect.incident_refs.is_empty()
        && !run.recurring_defect.failure_signature.trim().is_empty()
        && run.recurring_defect.source_hash.starts_with("sha256:")
    {
        promoted_facts.push(triage_fact(
            "RecurringDefect",
            "triage.maintenance.recurring-defect",
            "recurring defect links incident history, failure signature, affected service, and source hash",
            vec![recurring_defect_linked, defect_without_history],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_SERVICE_IMPACT_TRUTH_KEY)
        && !run.dependency_alert.affected_service.trim().is_empty()
        && run.dependency_alert.affected_service == run.recurring_defect.affected_service
    {
        promoted_facts.push(triage_fact(
            "ServiceImpact",
            "triage.maintenance.service-impact",
            "dependency alert and recurring defect name the same impacted client service",
            vec![service_impact_cited, hidden_service_impact],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_SLA_RISK_TRUTH_KEY)
        && run.risk_ranking.severity_basis_points >= 9000
        && run.risk_ranking.regression_risk_basis_points > 0
        && !run.risk_ranking.sla_due_at.trim().is_empty()
        && !run.risk_ranking.ranked_items.is_empty()
    {
        promoted_facts.push(triage_fact(
            "RiskRanking",
            "triage.maintenance.sla-risk-ranking",
            "maintenance work is ranked against SLA, severity, exploit evidence, and regression risk",
            vec![sla_risk_ranked, sla_breach_hidden],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_PATCH_PLAN_TRUTH_KEY)
        && !run.patch_plan.scope.trim().is_empty()
        && !run.patch_plan.target_window.trim().is_empty()
        && !run.patch_plan.rollback_ref.trim().is_empty()
        && !run.patch_plan.tests_required.is_empty()
    {
        promoted_facts.push(triage_fact(
            "PatchPlan",
            "triage.maintenance.patch-plan",
            "patch plan names scope, maintenance window, deploy action, tests required, and rollback reference",
            vec![patch_plan_with_rollback, patch_without_rollback],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_CHECKS_TRUTH_KEY)
        && run.checks.status == "Passed"
        && !run.checks.test_refs.is_empty()
        && run.checks.missing_test_gaps.is_empty()
        && run
            .checks
            .relevant_to_services
            .contains(&run.dependency_alert.affected_service)
    {
        promoted_facts.push(triage_fact(
            "Checks",
            "triage.maintenance.checks",
            "relevant checks passed before the dependency update is treated as safe",
            vec![checks_recorded, unsafe_update_marked_safe],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_CLIENT_APPROVAL_TRUTH_KEY)
        && run.approval.status == "Approved"
        && run.approval.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(triage_fact(
            "ClientApproval",
            "triage.maintenance.client-approval",
            "client approval is recorded before the production patch is treated as ready",
            vec![client_approval_state, production_change_without_approval],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_EMERGENCY_POLICY_TRUTH_KEY)
        && (!run.emergency_policy.used
            || (run.emergency_policy.status == "Permit"
                && !run.emergency_policy.policy_id.trim().is_empty()
                && !run.emergency_policy.authority_ref.trim().is_empty()))
    {
        promoted_facts.push(triage_fact(
            "EmergencyPolicy",
            "triage.maintenance.emergency-policy",
            "emergency maintenance names the policy and authority that allowed escalation",
            vec![emergency_policy_state, emergency_without_policy],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_DEFERRED_RISK_TRUTH_KEY)
        && !run.deferral.deferred_items.is_empty()
        && !run.deferral.residual_risk.trim().is_empty()
        && !run.deferral.next_review_date.trim().is_empty()
        && run.deferral.reason_hash.starts_with("sha256:")
    {
        promoted_facts.push(triage_fact(
            "Deferral",
            "triage.maintenance.deferred-risk",
            "deferred maintenance records residual risk, reason hash, owner, and next review date",
            vec![deferred_risk_recorded, deferred_work_without_risk],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, TRIAGE_PRODUCTION_BOUNDARY_TRUTH_KEY)
        && !run.production_boundary.triage_keeper_deploys_production
        && run.production_boundary.client_authority_required
        && run
            .production_boundary
            .planned_deploy_requires_external_gate
    {
        promoted_facts.push(triage_fact(
            "ProductionBoundary",
            "triage.maintenance.production-boundary",
            "Triage Keeper recommends, drafts, and schedules while production deploy remains behind client authority",
            vec![production_boundary, triage_as_deploy_authority],
            &run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:triage-maintenance-cycle", 33, 10),
        replay_notes: vec![
            format!(
                "adapted Triage maintenance run {} into AxiomRunObservation",
                run.maintenance_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn triage_fact(
    context_key: &str,
    fact_id: &str,
    summary: &str,
    source_clause_ids: Vec<ClauseId>,
    authority: &PromotionAuthorityRecord,
) -> PromotedFactRecord {
    PromotedFactRecord {
        context_key: context_key.to_string(),
        fact_id: fact_id.to_string(),
        summary: summary.to_string(),
        source_clause_ids,
        evidence_refs: vec![EvidenceRefRecord {
            evidence_id: format!("triage.evidence.{fact_id}"),
            source: "triage-maintenance-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("triage.trace.{fact_id}"),
            location: Some("triage://maintenance-cycle".to_string()),
            replayable: true,
        }),
        promotion_authority: Some(authority.clone()),
    }
}

fn has_truth_key(truth_keys: &[String], needle: &str) -> bool {
    truth_keys.iter().any(|truth_key| truth_key == needle)
}

fn evidence_clause_id(package: &TruthPackage, key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .find(|clause| clause.key == key)
        .map_or_else(
            || panic!("missing evidence clause {key}"),
            |clause| clause.id.clone(),
        )
}

fn failure_clause_id(package: &TruthPackage, key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .find(|clause| clause.key == key)
        .map_or_else(
            || panic!("missing failure clause {key}"),
            |clause| clause.id.clone(),
        )
}

fn job_readiness_packet_id(
    package: &TruthPackage,
    domain_hint: &str,
    maintenance_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    let digest = sha256_lines(&[
        "job_readiness_packet",
        package.package_id.as_str(),
        package.truth_version.as_str(),
        domain_hint,
        maintenance_run_id,
        adapter_receipt_id,
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.job_readiness.{short_digest}")
}

fn operator_decision_receipt_id(
    packet: &JobReadinessPacket,
    ranking: &TriageRiskRanking,
) -> String {
    let digest = sha256_lines(&[
        "operator_decision_receipt",
        packet.packet_id.as_str(),
        ranking.top_item.as_str(),
        ranking.sla_due_at.as_str(),
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.operator_decision.{short_digest}")
}

fn client_approval_receipt_id(
    packet: &JobReadinessPacket,
    approval: &TriageApproval,
    decision_receipt: &OperatorDecisionReceipt,
) -> String {
    let digest = sha256_lines(&[
        "client_approval_receipt",
        packet.packet_id.as_str(),
        approval.approval_id.as_str(),
        approval.status.as_str(),
        approval.note_hash.as_str(),
        decision_receipt.receipt_id.as_str(),
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.client_approval.{short_digest}")
}

fn maintenance_plan_receipt_id(
    packet: &JobReadinessPacket,
    plan: &TriagePatchPlan,
    approval_receipt: &ClientApprovalReceipt,
) -> String {
    let digest = sha256_lines(&[
        "maintenance_plan_receipt",
        packet.packet_id.as_str(),
        plan.plan_id.as_str(),
        plan.deploy_action.as_str(),
        approval_receipt.receipt_id.as_str(),
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.maintenance_plan.{short_digest}")
}

fn helm_ledger_entry_id(parts: &[&str]) -> String {
    let digest = sha256_lines(parts);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm_ledger_entry.{short_digest}")
}

fn sha256_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("fixture value serializes");
    sha256_bytes(&bytes)
}

fn sha256_lines(parts: &[&str]) -> String {
    sha256_bytes(parts.join("\n").as_bytes())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut output = String::with_capacity("sha256:".len() + digest.len() * 2);
    output.push_str("sha256:");
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[derive(Debug, Clone)]
struct ObservationAdapterOutcome {
    observation: Option<AxiomRunObservation>,
    receipt: ObservationAdapterReceipt,
}

#[derive(Debug, Clone, Serialize)]
struct JobReadinessPacket {
    packet_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    job_key: String,
    subject_ref: String,
    adapter_receipt_id: String,
    adapter_status: ObservationAdapterStatus,
    verdict: Option<AxiomRunVerdict>,
    authorizes_domain_action: bool,
    evidence_status: Vec<JobEvidenceStatus>,
    verifier_forbidden_actions: Vec<String>,
    operator_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct JobEvidenceStatus {
    clause_id: String,
    clause_key: String,
    label: String,
    status: EvidenceReadinessStatus,
    fact_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceReadinessStatus {
    Present,
    Missing,
}

#[derive(Debug, Clone, Serialize)]
struct OperatorDecisionReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    maintenance_run_ref: String,
    decision_ref_hash: String,
    top_item_kind: String,
    status: String,
    ranking_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ClientApprovalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    approval_ref_hash: String,
    status: String,
    scope_hash: String,
    note_hash: String,
    job_readiness_packet_id: String,
    operator_decision_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct MaintenancePlanReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    plan_ref_hash: String,
    status: String,
    target_window_hash: String,
    rollback_ref_hash: String,
    job_readiness_packet_id: String,
    operator_decision_receipt_id: String,
    approval_receipt_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct HelmLedgerEntry {
    entry_id: String,
    sequence: u64,
    record_kind: HelmLedgerRecordKind,
    source_ref: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    payload_hash: String,
    backlink_ids: Vec<String>,
    authority_effect: HelmLedgerAuthorityEffect,
    summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerRecordKind {
    ObservationAdapterReceipt,
    JobReadinessPacket,
    OperatorDecisionReceipt,
    ClientApprovalReceipt,
    MaintenancePlanReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::OperatorDecisionReceipt => "operator_decision_receipt",
            Self::ClientApprovalReceipt => "client_approval_receipt",
            Self::MaintenancePlanReceipt => "maintenance_plan_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageKeeperMaintenanceTranscript {
    source: TriageRunSource,
    maintenance_run: TriageMaintenanceRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageMaintenanceRun {
    maintenance_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    client: TriageClient,
    dependency_alert: TriageDependencyAlert,
    recurring_defect: TriageRecurringDefect,
    risk_ranking: TriageRiskRanking,
    patch_plan: TriagePatchPlan,
    checks: TriageChecks,
    approval: TriageApproval,
    emergency_policy: TriageEmergencyPolicy,
    deferral: TriageDeferral,
    production_boundary: TriageProductionBoundary,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageClient {
    client_id: String,
    support_tier: String,
    sla_hours: u16,
    production_change_policy: String,
    authority_mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageDependencyAlert {
    alert_id: String,
    package: String,
    current_version: String,
    patched_version: String,
    severity: String,
    advisory_ref: String,
    affected_service: String,
    exploit_evidence_ref: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageRecurringDefect {
    defect_id: String,
    affected_service: String,
    incident_refs: Vec<String>,
    recurrence_count: u16,
    failure_signature: String,
    owner: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageRiskRanking {
    top_item: String,
    basis: Vec<String>,
    severity_basis_points: u16,
    regression_risk_basis_points: u16,
    sla_due_at: String,
    ranked_items: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriagePatchPlan {
    plan_id: String,
    scope: String,
    target_window: String,
    rollback_ref: String,
    tests_required: Vec<String>,
    deploy_action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageChecks {
    ci_run_id: String,
    status: String,
    test_refs: Vec<String>,
    missing_test_gaps: Vec<String>,
    relevant_to_services: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageApproval {
    approval_id: String,
    approver_id: String,
    status: String,
    scope: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageEmergencyPolicy {
    policy_id: String,
    used: bool,
    status: String,
    escalation_reason: String,
    authority_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageDeferral {
    deferred_items: Vec<String>,
    residual_risk: String,
    next_review_date: String,
    accepted_by: String,
    reason_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TriageProductionBoundary {
    triage_keeper_deploys_production: bool,
    client_authority_required: bool,
    planned_deploy_requires_external_gate: bool,
    last_safe_release_ref: String,
}

fn triage_maintenance_transcript() -> TriageKeeperMaintenanceTranscript {
    serde_json::from_str(TRIAGE_MAINTENANCE_TRANSCRIPT)
        .expect("Triage Keeper maintenance transcript parses")
}
