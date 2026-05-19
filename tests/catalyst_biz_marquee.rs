//! Catalyst Biz - ninth app probe for the Axiom/Helm contract.
//!
//! Plumb proved execution receipts for stateful strategy revision. Catalyst
//! pressures everyday business operations: qualification, consent, routing,
//! HITL approval, next action, and outcome tracking must compose without
//! making Helm or Axiom the business authority.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const CATALYST_JOB_DEFINITION_TRUTH_KEY: &str = "job-definition-bound";
const CATALYST_ACCOUNT_CONTEXT_TRUTH_KEY: &str = "account-context-loaded";
const CATALYST_INBOUND_INTENT_TRUTH_KEY: &str = "inbound-intent-captured";
const CATALYST_CONSENT_TRUTH_KEY: &str = "consent-policy-checked";
const CATALYST_FIT_TRUTH_KEY: &str = "fit-score-explained";
const CATALYST_OWNER_TRUTH_KEY: &str = "owner-capacity-checked";
const CATALYST_APPROVAL_TRUTH_KEY: &str = "hitl-approval-recorded";
const CATALYST_ROUTING_TRUTH_KEY: &str = "routing-decision-recorded";
const CATALYST_NEXT_ACTION_TRUTH_KEY: &str = "next-action-receipt-issued";
const CATALYST_OUTCOME_TRUTH_KEY: &str = "outcome-tracking-registered";
const CATALYST_PROVIDER_FACTS_TRUTH_KEY: &str = "provider-facts-cited";
const CATALYST_RESULT_TRUTH_KEY: &str = "honest-stop-or-business-result";
const CATALYST_ADAPTER_ID: &str = "catalyst-biz.inbound-account-to-axiom-observation";
const CATALYST_ADAPTER_VERSION: &str = "fixture.v0.1";
const CATALYST_INBOUND_TRANSCRIPT: &str =
    include_str!("fixtures/catalyst_inbound_account_transcript.json");

fn catalyst_inbound_account_jtbd() -> JtbdInput {
    JtbdInput {
        key: "catalyst-inbound-account".to_string(),
        actor: "revenue operator".to_string(),
        functional_job:
            "turn a messy inbound account into the right governed business action".to_string(),
        so_that:
            "the team can qualify, route, and follow up without losing consent, ownership, or outcome traceability"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "job_definition_bound",
                "the business job names the Catalyst job key, expected phases, HITL posture, and source hash",
            ),
            ClauseInput::with_key(
                "account_context_loaded",
                "the account context names lifecycle, opportunity state, recent timeline refs, and source hash",
            ),
            ClauseInput::with_key(
                "inbound_intent_captured",
                "the inbound intent captures contact role, use case, submitted time, and hashed source summary",
            ),
            ClauseInput::with_key(
                "consent_policy_checked",
                "outreach consent, suppression list state, policy id, and policy hash are recorded before follow-up",
            ),
            ClauseInput::with_key(
                "fit_score_explained",
                "fit score and tier cite provider-backed dimensions and evidence refs",
            ),
            ClauseInput::with_key(
                "owner_capacity_checked",
                "the route names an eligible owner, territory, capacity state, and routing basis",
            ),
            ClauseInput::with_key(
                "hitl_approval_recorded",
                "the HITL approval records approval id, status, scope, and note hash before the routed action is treated as ready",
            ),
            ClauseInput::with_key(
                "routing_decision_recorded",
                "the routing decision records owner, decision id, status, and rationale hash",
            ),
            ClauseInput::with_key(
                "next_action_receipt_issued",
                "the next business action records kind, status, customer visibility, action ref, and receipt hash",
            ),
            ClauseInput::with_key(
                "outcome_tracking_registered",
                "the job result registers outcome tracking with dashboard ref, status, and metrics",
            ),
            ClauseInput::with_key(
                "provider_facts_cited",
                "provider facts cite source providers, fact refs, and source hashes used by the result",
            ),
            ClauseInput::with_key(
                "honest_stop_or_business_result",
                "the run either returns a business result with no hidden stop reason or exposes the honest stop reason",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "job_without_definition",
                "a business result is produced without binding to a Catalyst job definition",
            ),
            ClauseInput::with_key(
                "account_context_missing",
                "the account is qualified or routed without current account context",
            ),
            ClauseInput::with_key(
                "outreach_without_consent",
                "customer-visible follow-up is queued without consent and suppression-list evidence",
            ),
            ClauseInput::with_key(
                "score_without_evidence",
                "a fit score is used without provider-backed dimensions and evidence refs",
            ),
            ClauseInput::with_key(
                "route_without_owner_capacity",
                "the lead is routed without an eligible owner and capacity basis",
            ),
            ClauseInput::with_key(
                "approval_bypassed",
                "a routed business action is treated as ready without the required HITL approval",
            ),
            ClauseInput::with_key(
                "routing_without_decision_record",
                "the account is assigned without a routing decision record",
            ),
            ClauseInput::with_key(
                "next_action_without_receipt",
                "a meeting, campaign, or follow-up action is queued without a receipt",
            ),
            ClauseInput::with_key(
                "outcome_untracked",
                "the business result is not registered for outcome tracking",
            ),
            ClauseInput::with_key(
                "provider_fact_untraced",
                "the result relies on provider facts without fact refs and source hashes",
            ),
            ClauseInput::with_key(
                "helm_as_business_authority",
                "Helm is treated as the authority that qualified, routed, or sent the customer-visible action",
            ),
            ClauseInput::with_key(
                "hidden_stop_reason",
                "the job fails or blocks but hides the stop reason behind a completed timeline",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(30)),
    }
}

#[test]
fn catalyst_inbound_account_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Turn a messy inbound account")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        12
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        12
    );
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("suppression list"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("Helm is treated as the authority"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn catalyst_inbound_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");
    let transcript = catalyst_inbound_transcript();

    let observation =
        adapt_catalyst_inbound_transcript(&package, &transcript).expect("Catalyst adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.catalyst-inbound-account")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Catalyst-adapted inbound run preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 12);
    assert_eq!(audit.failure_coverage.len(), 12);
    assert_eq!(audit.facts_audited, 12);
}

#[test]
fn catalyst_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");
    let transcript = catalyst_inbound_transcript();

    let first = adapt_catalyst_inbound_transcript_with_receipt(&package, &transcript);
    let second = adapt_catalyst_inbound_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, CATALYST_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "catalyst-biz");
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
            "catalyst.biz.job-definition",
            "catalyst.biz.account-context",
            "catalyst.biz.inbound-intent",
            "catalyst.biz.consent-policy",
            "catalyst.biz.fit-score",
            "catalyst.biz.owner-capacity",
            "catalyst.biz.hitl-approval",
            "catalyst.biz.routing-decision",
            "catalyst.biz.next-action",
            "catalyst.biz.outcome-tracking",
            "catalyst.biz.provider-facts",
            "catalyst.biz.business-result",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 24);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("operator.ari.growth"));
    assert!(!serialized.contains("user.eva.account-owner"));
    assert!(!serialized.contains("embassy://crm/consent"));
    assert!(!serialized.contains("reduce renewal leakage"));
    assert!(!serialized.contains("CATALYST_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/catalyst-biz"));
}

#[test]
fn catalyst_job_readiness_packet_marks_missing_consent_policy() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");
    let mut transcript = catalyst_inbound_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != CATALYST_CONSENT_TRUTH_KEY);
    transcript.execution_run.consent_policy.status = "Missing".to_string();
    transcript
        .execution_run
        .consent_policy
        .suppression_list_checked = false;
    let adapter_outcome = adapt_catalyst_inbound_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let consent_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "consent_policy_checked")
        .expect("consent evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(consent_status.status, EvidenceReadinessStatus::Missing);
    assert!(consent_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for consent_policy_checked".to_string())
    );
}

#[test]
fn catalyst_job_readiness_packet_marks_routing_without_hitl_approval() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");
    let mut transcript = catalyst_inbound_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != CATALYST_APPROVAL_TRUTH_KEY);
    transcript.execution_run.approval.status = "Pending".to_string();
    let adapter_outcome = adapt_catalyst_inbound_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "hitl_approval_recorded")
        .expect("approval evidence is represented");
    let routing_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "routing_decision_recorded")
        .expect("routing evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(routing_status.status, EvidenceReadinessStatus::Present);
    assert!(approval_status.fact_ids.is_empty());
}

#[test]
fn catalyst_operator_ledger_entries_are_deterministic_backlinks_without_business_authority() {
    let package = decode_jtbd(catalyst_inbound_account_jtbd()).expect("Catalyst JTBD decodes");
    let transcript = catalyst_inbound_transcript();
    let adapter_outcome = adapt_catalyst_inbound_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_receipt = hitl_approval_receipt(&packet, &transcript);
    let routing_receipt = routing_decision_receipt(&packet, &transcript, &approval_receipt);
    let next_action_receipt = next_action_receipt(&packet, &transcript, &routing_receipt);
    let outcome_receipt = outcome_tracking_receipt(&packet, &transcript, &next_action_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &approval_receipt,
        &routing_receipt,
        &next_action_receipt,
        &outcome_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &approval_receipt,
        &routing_receipt,
        &next_action_receipt,
        &outcome_receipt,
    );

    assert_eq!(first, second);
    assert_eq!(first.len(), 6);
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
        HelmLedgerRecordKind::HitlApprovalReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::RoutingDecisionReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::NextActionReceipt
    );
    assert_eq!(
        first[5].record_kind,
        HelmLedgerRecordKind::OutcomeTrackingReceipt
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );
    assert_eq!(
        first[5].backlink_ids,
        vec![
            packet.packet_id.clone(),
            routing_receipt.receipt_id.clone(),
            next_action_receipt.receipt_id.clone(),
        ]
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("operator.ari.growth"));
    assert!(!serialized.contains("user.eva.account-owner"));
    assert!(!serialized.contains("embassy://crm/consent"));
    assert!(!serialized.contains("reduce renewal leakage"));
    assert!(!serialized.contains("CATALYST_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/catalyst-biz"));
}

fn adapt_catalyst_inbound_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &CatalystInboundTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_catalyst_inbound_transcript(package, transcript) {
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
    transcript: &CatalystInboundTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: CATALYST_ADAPTER_ID.to_string(),
        adapter_version: CATALYST_ADAPTER_VERSION.to_string(),
        status,
        source_app: "catalyst-biz".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "catalyst://inbound-account/{}/{}",
            transcript.source.run_id, transcript.execution_run.execution_run_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "source.command".to_string(),
            "source.app_path".to_string(),
            "account_context.organization_id".to_string(),
            "inbound_intent.use_case".to_string(),
            "consent_policy.consent_evidence_ref".to_string(),
            "owner_routing.owner_user_id".to_string(),
            "approval.approver_id".to_string(),
            "next_action.action_ref".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &CatalystInboundTranscript,
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
        &transcript.execution_run.execution_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!(
            "catalyst://inbound-account/{}",
            transcript.execution_run.execution_run_id
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
    actions.push("review HITL approval before customer-visible action".to_string());
    actions.push("confirm outcome tracking before closing the job".to_string());
    actions
}

fn hitl_approval_receipt(
    packet: &JobReadinessPacket,
    transcript: &CatalystInboundTranscript,
) -> HitlApprovalReceipt {
    let approval = &transcript.execution_run.approval;
    HitlApprovalReceipt {
        receipt_id: hitl_approval_receipt_id(packet, approval),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        approval_ref_hash: sha256_lines(&[approval.approval_id.as_str()]),
        status: approval.status.clone(),
        scope_hash: sha256_lines(&[approval.scope.as_str()]),
        note_hash: approval.note_hash.clone(),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn routing_decision_receipt(
    packet: &JobReadinessPacket,
    transcript: &CatalystInboundTranscript,
    approval_receipt: &HitlApprovalReceipt,
) -> RoutingDecisionReceipt {
    let routing = &transcript.execution_run.routing_decision;
    RoutingDecisionReceipt {
        receipt_id: routing_decision_receipt_id(packet, routing, approval_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        decision_ref_hash: sha256_lines(&[routing.decision_id.as_str()]),
        owner_ref_hash: sha256_lines(&[routing.owner_user_id.as_str()]),
        status: routing.status.clone(),
        rationale_hash: routing.rationale_hash.clone(),
        hitl_approval_receipt_id: approval_receipt.receipt_id.clone(),
    }
}

fn next_action_receipt(
    packet: &JobReadinessPacket,
    transcript: &CatalystInboundTranscript,
    routing_receipt: &RoutingDecisionReceipt,
) -> NextActionReceipt {
    let next_action = &transcript.execution_run.next_action;
    NextActionReceipt {
        receipt_id: next_action_receipt_id(packet, next_action, routing_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        action_ref_hash: sha256_lines(&[next_action.action_ref.as_str()]),
        action_kind: next_action.kind.clone(),
        status: next_action.status.clone(),
        customer_visible: next_action.customer_visible,
        receipt_hash: next_action.receipt_hash.clone(),
        routing_decision_receipt_id: routing_receipt.receipt_id.clone(),
    }
}

fn outcome_tracking_receipt(
    packet: &JobReadinessPacket,
    transcript: &CatalystInboundTranscript,
    next_action_receipt: &NextActionReceipt,
) -> OutcomeTrackingReceipt {
    let outcome = &transcript.execution_run.outcome_tracking;
    OutcomeTrackingReceipt {
        receipt_id: outcome_tracking_receipt_id(packet, outcome, next_action_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        tracking_ref_hash: sha256_lines(&[outcome.tracking_id.as_str()]),
        status: outcome.status.clone(),
        metric_count: outcome.metrics.len(),
        dashboard_ref_hash: sha256_lines(&[outcome.dashboard_ref.as_str()]),
        next_action_receipt_id: next_action_receipt.receipt_id.clone(),
    }
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    approval_receipt: &HitlApprovalReceipt,
    routing_receipt: &RoutingDecisionReceipt,
    next_action_receipt: &NextActionReceipt,
    outcome_receipt: &OutcomeTrackingReceipt,
) -> Vec<HelmLedgerEntry> {
    vec![
        helm_ledger_entry(
            0,
            HelmLedgerRecordKind::ObservationAdapterReceipt,
            receipt.receipt_id.as_str().to_string(),
            receipt.package_id.as_str().to_string(),
            receipt.truth_version.clone(),
            receipt.domain_hint.clone(),
            sha256_json(receipt),
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
            sha256_json(packet),
            vec![receipt.receipt_id.as_str().to_string()],
            format!("job readiness {:?} for {}", packet.verdict, packet.job_key),
        ),
        helm_ledger_entry(
            2,
            HelmLedgerRecordKind::HitlApprovalReceipt,
            approval_receipt.receipt_id.clone(),
            approval_receipt.package_id.clone(),
            approval_receipt.truth_version.clone(),
            approval_receipt.domain_hint.clone(),
            sha256_json(approval_receipt),
            vec![packet.packet_id.clone()],
            format!("HITL approval {}", approval_receipt.status),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::RoutingDecisionReceipt,
            routing_receipt.receipt_id.clone(),
            routing_receipt.package_id.clone(),
            routing_receipt.truth_version.clone(),
            routing_receipt.domain_hint.clone(),
            sha256_json(routing_receipt),
            vec![
                packet.packet_id.clone(),
                approval_receipt.receipt_id.clone(),
            ],
            format!("routing decision {}", routing_receipt.status),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::NextActionReceipt,
            next_action_receipt.receipt_id.clone(),
            next_action_receipt.package_id.clone(),
            next_action_receipt.truth_version.clone(),
            next_action_receipt.domain_hint.clone(),
            sha256_json(next_action_receipt),
            vec![packet.packet_id.clone(), routing_receipt.receipt_id.clone()],
            format!(
                "next action {} {}",
                next_action_receipt.action_kind, next_action_receipt.status
            ),
        ),
        helm_ledger_entry(
            5,
            HelmLedgerRecordKind::OutcomeTrackingReceipt,
            outcome_receipt.receipt_id.clone(),
            outcome_receipt.package_id.clone(),
            outcome_receipt.truth_version.clone(),
            outcome_receipt.domain_hint.clone(),
            sha256_json(outcome_receipt),
            vec![
                packet.packet_id.clone(),
                routing_receipt.receipt_id.clone(),
                next_action_receipt.receipt_id.clone(),
            ],
            format!("outcome tracking {}", outcome_receipt.status),
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

fn adapt_catalyst_inbound_transcript(
    package: &TruthPackage,
    transcript: &CatalystInboundTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.execution_run;
    if run.status != "Completed" {
        return Err("expected Catalyst inbound run to complete before adaptation".to_string());
    }
    if run.job_key != "qualify-inbound-lead" || run.job_definition.key != run.job_key {
        return Err("expected Catalyst transcript to bind qualify-inbound-lead".to_string());
    }
    if run.job_definition.phases.is_empty() {
        return Err("expected Catalyst job definition to carry phases".to_string());
    }

    let job_definition_bound = evidence_clause_id(package, "job_definition_bound");
    let account_context_loaded = evidence_clause_id(package, "account_context_loaded");
    let inbound_intent_captured = evidence_clause_id(package, "inbound_intent_captured");
    let consent_policy_checked = evidence_clause_id(package, "consent_policy_checked");
    let fit_score_explained = evidence_clause_id(package, "fit_score_explained");
    let owner_capacity_checked = evidence_clause_id(package, "owner_capacity_checked");
    let hitl_approval_recorded = evidence_clause_id(package, "hitl_approval_recorded");
    let routing_decision_recorded = evidence_clause_id(package, "routing_decision_recorded");
    let next_action_receipt_issued = evidence_clause_id(package, "next_action_receipt_issued");
    let outcome_tracking_registered = evidence_clause_id(package, "outcome_tracking_registered");
    let provider_facts_cited = evidence_clause_id(package, "provider_facts_cited");
    let honest_stop_or_business_result =
        evidence_clause_id(package, "honest_stop_or_business_result");
    let job_without_definition = failure_clause_id(package, "job_without_definition");
    let account_context_missing = failure_clause_id(package, "account_context_missing");
    let outreach_without_consent = failure_clause_id(package, "outreach_without_consent");
    let score_without_evidence = failure_clause_id(package, "score_without_evidence");
    let route_without_owner_capacity = failure_clause_id(package, "route_without_owner_capacity");
    let approval_bypassed = failure_clause_id(package, "approval_bypassed");
    let routing_without_decision_record =
        failure_clause_id(package, "routing_without_decision_record");
    let next_action_without_receipt = failure_clause_id(package, "next_action_without_receipt");
    let outcome_untracked = failure_clause_id(package, "outcome_untracked");
    let provider_fact_untraced = failure_clause_id(package, "provider_fact_untraced");
    let helm_as_business_authority = failure_clause_id(package, "helm_as_business_authority");
    let hidden_stop_reason = failure_clause_id(package, "hidden_stop_reason");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&run.truth_keys, CATALYST_JOB_DEFINITION_TRUTH_KEY)
        && run.job_definition.requires_hitl
        && run.job_definition.source_hash.starts_with("sha256:")
        && run
            .job_definition
            .phases
            .iter()
            .any(|phase| phase == "HITL Gate")
    {
        promoted_facts.push(catalyst_fact(
            "JobDefinition",
            "catalyst.biz.job-definition",
            "Catalyst job definition binds key, phases, HITL posture, and source hash",
            vec![job_definition_bound, job_without_definition],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_ACCOUNT_CONTEXT_TRUTH_KEY)
        && run.account_context.lifecycle == "prospect"
        && run.account_context.source_hash.starts_with("sha256:")
        && !run.account_context.recent_timeline_refs.is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "AccountContext",
            "catalyst.biz.account-context",
            "account context names lifecycle, opportunity state, timeline refs, and source hash",
            vec![account_context_loaded, account_context_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_INBOUND_INTENT_TRUTH_KEY)
        && run.inbound_intent.contact_ref_hash.starts_with("sha256:")
        && run
            .inbound_intent
            .inbound_summary_hash
            .starts_with("sha256:")
        && !run.inbound_intent.contact_title.trim().is_empty()
        && !run.inbound_intent.use_case.trim().is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "InboundIntent",
            "catalyst.biz.inbound-intent",
            "inbound intent captures contact role, use case, submitted time, and hashed source summary",
            vec![inbound_intent_captured, score_without_evidence.clone()],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_CONSENT_TRUTH_KEY)
        && run.consent_policy.status == "Allowed"
        && run.consent_policy.suppression_list_checked
        && run.consent_policy.policy_hash.starts_with("sha256:")
        && !run.consent_policy.consent_evidence_ref.trim().is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "ConsentPolicy",
            "catalyst.biz.consent-policy",
            "outreach consent, suppression list, policy id, and policy hash are recorded before follow-up",
            vec![consent_policy_checked, outreach_without_consent],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_FIT_TRUTH_KEY)
        && run.fit_assessment.score_basis_points > 0
        && !run.fit_assessment.tier.trim().is_empty()
        && run.fit_assessment.dimensions.iter().all(|dimension| {
            !dimension.provider_id.trim().is_empty()
                && !dimension.evidence_ref.trim().is_empty()
                && dimension.source_hash.starts_with("sha256:")
        })
    {
        promoted_facts.push(catalyst_fact(
            "FitScore",
            "catalyst.biz.fit-score",
            "fit score and tier cite provider-backed dimensions and evidence refs",
            vec![fit_score_explained, score_without_evidence],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_OWNER_TRUTH_KEY)
        && run.owner_routing.capacity_status == "Available"
        && !run.owner_routing.owner_user_id.trim().is_empty()
        && run.owner_routing.routing_basis_hash.starts_with("sha256:")
    {
        promoted_facts.push(catalyst_fact(
            "OwnerCapacity",
            "catalyst.biz.owner-capacity",
            "eligible owner, territory, capacity state, and routing basis are recorded",
            vec![owner_capacity_checked, route_without_owner_capacity],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_APPROVAL_TRUTH_KEY)
        && run.approval.status == "Approved"
        && run.approval.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(catalyst_fact(
            "HitlApproval",
            "catalyst.biz.hitl-approval",
            "HITL approval records approval id, status, scope, and note hash before routed action readiness",
            vec![hitl_approval_recorded, approval_bypassed],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_ROUTING_TRUTH_KEY)
        && run.routing_decision.status == "Routed"
        && run.routing_decision.owner_user_id == run.owner_routing.owner_user_id
        && run.routing_decision.rationale_hash.starts_with("sha256:")
    {
        promoted_facts.push(catalyst_fact(
            "RoutingDecision",
            "catalyst.biz.routing-decision",
            "routing decision records owner, decision id, status, and rationale hash",
            vec![routing_decision_recorded, routing_without_decision_record],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_NEXT_ACTION_TRUTH_KEY)
        && run.next_action.status == "Queued"
        && run.next_action.customer_visible
        && run.next_action.receipt_hash.starts_with("sha256:")
        && !run.next_action.action_ref.trim().is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "NextAction",
            "catalyst.biz.next-action",
            "next business action records kind, status, customer visibility, action ref, and receipt hash",
            vec![next_action_receipt_issued, next_action_without_receipt],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_OUTCOME_TRUTH_KEY)
        && run.outcome_tracking.status == "Registered"
        && !run.outcome_tracking.dashboard_ref.trim().is_empty()
        && !run.outcome_tracking.metrics.is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "OutcomeTracking",
            "catalyst.biz.outcome-tracking",
            "job result registers outcome tracking with dashboard ref, status, and metrics",
            vec![outcome_tracking_registered, outcome_untracked],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_PROVIDER_FACTS_TRUTH_KEY)
        && run.provider_facts.iter().all(|fact| {
            !fact.provider_id.trim().is_empty()
                && !fact.fact_ref.trim().is_empty()
                && fact.source_hash.starts_with("sha256:")
        })
    {
        promoted_facts.push(catalyst_fact(
            "ProviderFacts",
            "catalyst.biz.provider-facts",
            "provider facts cite source providers, fact refs, and source hashes used by the result",
            vec![provider_facts_cited, provider_fact_untraced],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, CATALYST_RESULT_TRUTH_KEY)
        && run.business_result.status == "QualifiedAndRouted"
        && !run.business_result.helm_authorized_business_action
        && run.business_result.honest_stop_reason.trim().is_empty()
    {
        promoted_facts.push(catalyst_fact(
            "BusinessResult",
            "catalyst.biz.business-result",
            "run returns a business result while preserving authority boundary and stop reason visibility",
            vec![
                honest_stop_or_business_result,
                helm_as_business_authority,
                hidden_stop_reason,
            ],
            &run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:catalyst-inbound-account", 29, 12),
        replay_notes: vec![
            format!(
                "adapted Catalyst inbound account {} into AxiomRunObservation",
                run.execution_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn catalyst_fact(
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
            evidence_id: format!("catalyst.evidence.{fact_id}"),
            source: "catalyst-inbound-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("catalyst.trace.{fact_id}"),
            location: Some("catalyst://inbound-account".to_string()),
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
    execution_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    short_id(
        &sha256_lines(&[
            "job_readiness_packet",
            package.package_id.as_str(),
            package.truth_version.as_str(),
            domain_hint,
            execution_run_id,
            adapter_receipt_id,
        ]),
        "helm.job_readiness",
    )
}

fn hitl_approval_receipt_id(packet: &JobReadinessPacket, approval: &CatalystApproval) -> String {
    short_id(
        &sha256_lines(&[
            "hitl_approval_receipt",
            packet.packet_id.as_str(),
            approval.approval_id.as_str(),
            approval.status.as_str(),
        ]),
        "helm.hitl_approval",
    )
}

fn routing_decision_receipt_id(
    packet: &JobReadinessPacket,
    routing: &CatalystRoutingDecision,
    approval_receipt: &HitlApprovalReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "routing_decision_receipt",
            packet.packet_id.as_str(),
            routing.decision_id.as_str(),
            routing.status.as_str(),
            approval_receipt.receipt_id.as_str(),
        ]),
        "helm.routing_decision",
    )
}

fn next_action_receipt_id(
    packet: &JobReadinessPacket,
    next_action: &CatalystNextAction,
    routing_receipt: &RoutingDecisionReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "next_action_receipt",
            packet.packet_id.as_str(),
            next_action.action_id.as_str(),
            next_action.status.as_str(),
            routing_receipt.receipt_id.as_str(),
        ]),
        "helm.next_action",
    )
}

fn outcome_tracking_receipt_id(
    packet: &JobReadinessPacket,
    outcome: &CatalystOutcomeTracking,
    next_action_receipt: &NextActionReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "outcome_tracking_receipt",
            packet.packet_id.as_str(),
            outcome.tracking_id.as_str(),
            outcome.status.as_str(),
            next_action_receipt.receipt_id.as_str(),
        ]),
        "helm.outcome_tracking",
    )
}

fn helm_ledger_entry_id(parts: &[&str]) -> String {
    short_id(&sha256_lines(parts), "helm_ledger_entry")
}

fn short_id(digest: &str, prefix: &str) -> String {
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("{prefix}.{short_digest}")
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
struct HitlApprovalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    approval_ref_hash: String,
    status: String,
    scope_hash: String,
    note_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct RoutingDecisionReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    decision_ref_hash: String,
    owner_ref_hash: String,
    status: String,
    rationale_hash: String,
    hitl_approval_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct NextActionReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    action_ref_hash: String,
    action_kind: String,
    status: String,
    customer_visible: bool,
    receipt_hash: String,
    routing_decision_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct OutcomeTrackingReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    tracking_ref_hash: String,
    status: String,
    metric_count: usize,
    dashboard_ref_hash: String,
    next_action_receipt_id: String,
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
    HitlApprovalReceipt,
    RoutingDecisionReceipt,
    NextActionReceipt,
    OutcomeTrackingReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::HitlApprovalReceipt => "hitl_approval_receipt",
            Self::RoutingDecisionReceipt => "routing_decision_receipt",
            Self::NextActionReceipt => "next_action_receipt",
            Self::OutcomeTrackingReceipt => "outcome_tracking_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystInboundTranscript {
    source: CatalystRunSource,
    execution_run: CatalystExecutionRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystExecutionRun {
    execution_run_id: String,
    job_key: String,
    status: String,
    truth_keys: Vec<String>,
    job_definition: CatalystJobDefinition,
    account_context: CatalystAccountContext,
    inbound_intent: CatalystInboundIntent,
    consent_policy: CatalystConsentPolicy,
    fit_assessment: CatalystFitAssessment,
    owner_routing: CatalystOwnerRouting,
    approval: CatalystApproval,
    routing_decision: CatalystRoutingDecision,
    next_action: CatalystNextAction,
    outcome_tracking: CatalystOutcomeTracking,
    provider_facts: Vec<CatalystProviderFact>,
    business_result: CatalystBusinessResult,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystJobDefinition {
    key: String,
    title: String,
    requires_hitl: bool,
    phases: Vec<String>,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystAccountContext {
    organization_id: String,
    lifecycle: String,
    open_opportunity_count: u64,
    recent_timeline_refs: Vec<String>,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystInboundIntent {
    contact_ref_hash: String,
    contact_title: String,
    inbound_summary_hash: String,
    use_case: String,
    submitted_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystConsentPolicy {
    policy_id: String,
    status: String,
    suppression_list_checked: bool,
    consent_evidence_ref: String,
    policy_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystFitAssessment {
    assessment_id: String,
    score_basis_points: u16,
    tier: String,
    dimensions: Vec<CatalystFitDimension>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystFitDimension {
    dimension: String,
    provider_id: String,
    evidence_ref: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystOwnerRouting {
    owner_user_id: String,
    territory: String,
    capacity_status: String,
    routing_basis_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystApproval {
    approval_id: String,
    approver_id: String,
    status: String,
    scope: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystRoutingDecision {
    decision_id: String,
    status: String,
    owner_user_id: String,
    rationale_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystNextAction {
    action_id: String,
    kind: String,
    status: String,
    action_ref: String,
    customer_visible: bool,
    receipt_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystOutcomeTracking {
    tracking_id: String,
    status: String,
    dashboard_ref: String,
    metrics: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystProviderFact {
    provider_id: String,
    fact_ref: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CatalystBusinessResult {
    status: String,
    honest_stop_reason: String,
    helm_authorized_business_action: bool,
    result_ref: String,
}

fn catalyst_inbound_transcript() -> CatalystInboundTranscript {
    serde_json::from_str(CATALYST_INBOUND_TRANSCRIPT).expect("Catalyst inbound transcript parses")
}
