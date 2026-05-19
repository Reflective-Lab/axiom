//! Scout Sourcing — fourth app probe for the Axiom/Helm contract.
//!
//! Tally proved irreversible release, Atlas proved integration candidate
//! review, Quorum proved contested sensemaking readiness, and Scout pressures
//! the same contract with procurement: source packs, ranked vendors, policy
//! gates, and HITL thresholds.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const SCOUT_SOURCE_PACK_TRUTH_KEY: &str = "source-pack-cited";
const SCOUT_INTENT_ADMITTED_TRUTH_KEY: &str = "intent-admitted";
const SCOUT_FORMATION_ASSEMBLED_TRUTH_KEY: &str = "formation-assembled";
const SCOUT_VENDORS_SCREENED_TRUTH_KEY: &str = "vendors-screened";
const SCOUT_SHORTLIST_PRODUCED_TRUTH_KEY: &str = "shortlist-produced";
const SCOUT_POLICY_AUTHORIZED_TRUTH_KEY: &str = "policy-authorized";
const SCOUT_HUMAN_APPROVAL_TRUTH_KEY: &str = "human-approval-recorded";
const SCOUT_PROVENANCE_TRUTH_KEY: &str = "decision-provenance-preserved";
const SCOUT_ADAPTER_ID: &str = "scout-sourcing.vendor-selection-to-axiom-observation";
const SCOUT_ADAPTER_VERSION: &str = "fixture.v0.1";
const SCOUT_VENDOR_SELECTION_TRANSCRIPT: &str =
    include_str!("fixtures/scout_vendor_selection_transcript.json");

fn scout_vendor_selection_jtbd() -> JtbdInput {
    JtbdInput {
        key: "scout-vendor-selection".to_string(),
        actor: "procurement lead".to_string(),
        functional_job:
            "select a preferred AI vendor with auditable rationale, bounded authority, and evidence-backed policy compliance"
                .to_string(),
        so_that:
            "the buyer can commit sourcing effort without bypassing governance, budget, or provenance requirements"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "source_pack_loaded",
                "the vendor-selection source pack, criteria, and standing facts are cited by hash or reference",
            ),
            ClauseInput::with_key(
                "intent_admitted",
                "the vendor-selection intent is admitted with procurement authority context",
            ),
            ClauseInput::with_key(
                "formation_assembled",
                "the sourcing formation names planning, compliance, cost, risk, optimization, synthesis, and policy roles",
            ),
            ClauseInput::with_key(
                "vendors_screened",
                "every candidate vendor is screened through compliance, cost, and risk evidence channels",
            ),
            ClauseInput::with_key(
                "shortlist_explained",
                "the shortlist names selected and rejected vendors with objective scoring and rejection reasons",
            ),
            ClauseInput::with_key(
                "policy_authorized",
                "Cedar policy authorizes the vendor commitment or blocks honestly with a reason",
            ),
            ClauseInput::with_key(
                "human_approval_state",
                "human approval state is recorded for commitments that cross the HITL threshold",
            ),
            ClauseInput::with_key(
                "decision_provenance",
                "the recommendation preserves source material, evidence refs, policy identity, and promotion authority",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "shortlist_without_screening",
                "a vendor shortlist is produced before every candidate has compliance, cost, and risk screening",
            ),
            ClauseInput::with_key(
                "noncompliant_recommended",
                "a non-compliant or over-risk vendor is recommended as commit-ready",
            ),
            ClauseInput::with_key(
                "hidden_rejections",
                "rejected vendors or rejection reasons are hidden from the operator",
            ),
            ClauseInput::with_key(
                "over_threshold_without_approval",
                "a vendor commitment above the HITL threshold proceeds without approval",
            ),
            ClauseInput::with_key(
                "advisory_commitment",
                "advisory authority is treated as sufficient to commit procurement spend",
            ),
            ClauseInput::with_key(
                "missing_source_provenance",
                "the recommendation cannot be traced back to source pack, evidence, and policy refs",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(45)),
    }
}

#[test]
fn scout_vendor_selection_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Select a preferred AI vendor")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        8
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        6
    );
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("Cedar policy authorizes"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("without approval"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn scout_selection_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");
    let transcript = scout_vendor_selection_transcript();

    let observation =
        adapt_scout_vendor_selection_transcript(&package, &transcript).expect("Scout adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.scout-vendor-selection")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Scout-adapted vendor decision preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 8);
    assert_eq!(audit.failure_coverage.len(), 6);
    assert_eq!(audit.facts_audited, 8);
}

#[test]
fn scout_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");
    let transcript = scout_vendor_selection_transcript();

    let first = adapt_scout_vendor_selection_transcript_with_receipt(&package, &transcript);
    let second = adapt_scout_vendor_selection_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, SCOUT_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "scout-sourcing");
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
            "scout.selection.source-pack",
            "scout.selection.intent-admitted",
            "scout.selection.formation-assembled",
            "scout.selection.vendors-screened",
            "scout.selection.shortlist-produced",
            "scout.selection.policy-authorized",
            "scout.selection.human-approval",
            "scout.selection.decision-provenance",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 14);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("Mistral"));
    assert!(!serialized.contains("examples/vendor-selection/buyer-brief.md"));
    assert!(!serialized.contains("just demo-today"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/scout-sourcing"));
}

#[test]
fn scout_job_readiness_packet_marks_missing_source_provenance() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");
    let mut transcript = scout_vendor_selection_transcript();
    transcript
        .selection
        .truth_keys
        .retain(|truth_key| truth_key != SCOUT_SOURCE_PACK_TRUTH_KEY);
    transcript.selection.source_pack.static_fact_refs.clear();
    let adapter_outcome =
        adapt_scout_vendor_selection_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let source_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "source_pack_loaded")
        .expect("source pack evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(source_status.status, EvidenceReadinessStatus::Missing);
    assert!(source_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for source_pack_loaded".to_string())
    );
}

#[test]
fn scout_job_readiness_packet_marks_high_value_commitment_without_approval() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");
    let mut transcript = scout_vendor_selection_transcript();
    transcript.selection.shortlist.selected_amount_major = 72_000;
    transcript.selection.policy.outcome = "Escalate".to_string();
    transcript.selection.policy.reason =
        "amount exceeds HITL threshold and approval is pending".to_string();
    transcript.selection.policy.human_approval_status = "Pending".to_string();
    let adapter_outcome =
        adapt_scout_vendor_selection_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "human_approval_state")
        .expect("human approval evidence is represented");
    let policy_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "policy_authorized")
        .expect("policy evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(policy_status.status, EvidenceReadinessStatus::Missing);
    assert!(approval_status.fact_ids.is_empty());
    assert!(policy_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for human_approval_state".to_string())
    );
}

#[test]
fn scout_operator_ledger_entries_are_deterministic_backlinks_without_commit_authority() {
    let package = decode_jtbd(scout_vendor_selection_jtbd()).expect("Scout JTBD decodes");
    let transcript = scout_vendor_selection_transcript();
    let adapter_outcome =
        adapt_scout_vendor_selection_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);

    let first = job_readiness_ledger_entries(&adapter_outcome.receipt, &packet);
    let second = job_readiness_ledger_entries(&adapter_outcome.receipt, &packet);

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        first[0].record_kind,
        HelmLedgerRecordKind::ObservationAdapterReceipt
    );
    assert_eq!(
        first[1].record_kind,
        HelmLedgerRecordKind::JobReadinessPacket
    );
    assert_eq!(
        first[1].backlink_ids,
        vec![adapter_outcome.receipt.receipt_id.as_str().to_string()]
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("Mistral"));
    assert!(!serialized.contains("examples/vendor-selection/buyer-brief.md"));
    assert!(!serialized.contains("vendor-selection-policy.cedar"));
    assert!(!serialized.contains("just demo-today"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/scout-sourcing"));
}

fn adapt_scout_vendor_selection_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &ScoutVendorSelectionTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_scout_vendor_selection_transcript(package, transcript) {
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
    transcript: &ScoutVendorSelectionTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: SCOUT_ADAPTER_ID.to_string(),
        adapter_version: SCOUT_ADAPTER_VERSION.to_string(),
        status,
        source_app: "scout-sourcing".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "scout://selection/{}/{}",
            transcript.source.run_id, transcript.selection.selection_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "vendors.name".to_string(),
            "shortlist.entries.rationale".to_string(),
            "source_pack.document_ref".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &ScoutVendorSelectionTranscript,
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

    JobReadinessPacket {
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!("scout://selection/{}", transcript.selection.selection_id),
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
    actions.push("route commitment through Helm HITL before procurement action".to_string());
    actions
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
) -> Vec<HelmLedgerEntry> {
    let receipt_payload_hash = sha256_json(receipt);
    let packet_payload_hash = sha256_json(packet);

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
            format!("helm://job-readiness/{}", packet.adapter_receipt_id),
            packet.package_id.clone(),
            packet.truth_version.clone(),
            packet.domain_hint.clone(),
            packet_payload_hash,
            vec![receipt.receipt_id.as_str().to_string()],
            format!("job readiness {:?} for {}", packet.verdict, packet.job_key),
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

fn adapt_scout_vendor_selection_transcript(
    package: &TruthPackage,
    transcript: &ScoutVendorSelectionTranscript,
) -> Result<AxiomRunObservation, String> {
    let selection = &transcript.selection;
    if selection.truth_key != "vendor-selection" {
        return Err("expected Scout product truth vendor-selection".to_string());
    }
    if selection.status != "Converged" {
        return Err("expected Scout selection to converge before adaptation".to_string());
    }
    if selection.vendors.is_empty() {
        return Err("expected Scout selection to carry candidate vendors".to_string());
    }
    if selection.shortlist.entries.is_empty() {
        return Err("expected Scout selection to carry a shortlist".to_string());
    }

    let source_pack_loaded = evidence_clause_id(package, "source_pack_loaded");
    let intent_admitted = evidence_clause_id(package, "intent_admitted");
    let formation_assembled = evidence_clause_id(package, "formation_assembled");
    let vendors_screened = evidence_clause_id(package, "vendors_screened");
    let shortlist_explained = evidence_clause_id(package, "shortlist_explained");
    let policy_authorized = evidence_clause_id(package, "policy_authorized");
    let human_approval_state = evidence_clause_id(package, "human_approval_state");
    let decision_provenance = evidence_clause_id(package, "decision_provenance");
    let shortlist_without_screening = failure_clause_id(package, "shortlist_without_screening");
    let noncompliant_recommended = failure_clause_id(package, "noncompliant_recommended");
    let hidden_rejections = failure_clause_id(package, "hidden_rejections");
    let over_threshold_without_approval =
        failure_clause_id(package, "over_threshold_without_approval");
    let advisory_commitment = failure_clause_id(package, "advisory_commitment");
    let missing_source_provenance = failure_clause_id(package, "missing_source_provenance");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&selection.truth_keys, SCOUT_SOURCE_PACK_TRUTH_KEY)
        && !selection.source_pack.document_ref.trim().is_empty()
        && !selection.source_pack.criteria_ref.trim().is_empty()
        && !selection.source_pack.static_fact_refs.is_empty()
        && selection.source_pack.source_hash.starts_with("sha256:")
    {
        promoted_facts.push(scout_fact(
            "SourceMaterial",
            "scout.selection.source-pack",
            "vendor-selection source pack, criteria, standing facts, and source hash are present",
            vec![source_pack_loaded, missing_source_provenance.clone()],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_INTENT_ADMITTED_TRUTH_KEY)
        && selection.intent.intent_id == "truth:vendor-selection"
        && selection.intent.domain == "procurement"
    {
        promoted_facts.push(scout_fact(
            "Intent",
            "scout.selection.intent-admitted",
            "vendor-selection intent is admitted with procurement authority context",
            vec![intent_admitted],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_FORMATION_ASSEMBLED_TRUTH_KEY)
        && selection.formation.roles.len() >= 7
    {
        promoted_facts.push(scout_fact(
            "Formation",
            "scout.selection.formation-assembled",
            "sourcing formation names planning, compliance, cost, risk, optimization, synthesis, and policy roles",
            vec![formation_assembled],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_VENDORS_SCREENED_TRUTH_KEY)
        && selection.screening.screened_vendor_count == selection.vendors.len()
        && selection.screening.evidence_channels.len() >= 4
        && !selection.screening.evidence_refs.is_empty()
    {
        promoted_facts.push(scout_fact(
            "Screening",
            "scout.selection.vendors-screened",
            "every candidate vendor has compliance, cost, risk, and policy evidence channels",
            vec![vendors_screened, shortlist_without_screening],
            &selection.promotion_authority,
        ));
    }

    let selected_vendor = selection
        .vendors
        .iter()
        .find(|vendor| vendor.name == selection.shortlist.selected_vendor);
    let selected_vendor_is_commit_ready = selected_vendor
        .is_some_and(|vendor| vendor.compliance_status == "compliant" && vendor.risk_score <= 30.0);
    if has_truth_key(&selection.truth_keys, SCOUT_SHORTLIST_PRODUCED_TRUTH_KEY)
        && selected_vendor_is_commit_ready
        && !selection.shortlist.rejected.is_empty()
        && !selection.shortlist.objective.trim().is_empty()
    {
        promoted_facts.push(scout_fact(
            "Shortlist",
            "scout.selection.shortlist-produced",
            "ranked shortlist names selected and rejected vendors with objective scoring and rejection reasons",
            vec![shortlist_explained, noncompliant_recommended, hidden_rejections],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_POLICY_AUTHORIZED_TRUTH_KEY)
        && selection.policy.outcome == "Permit"
        && selection.policy.gates_met
        && selection.intent.principal_authority != "advisory"
    {
        promoted_facts.push(scout_fact(
            "Policy",
            "scout.selection.policy-authorized",
            "Cedar policy permits the selected vendor commitment with gates met and sufficient authority",
            vec![policy_authorized, advisory_commitment],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_HUMAN_APPROVAL_TRUTH_KEY)
        && (selection.shortlist.selected_amount_major <= selection.policy.hitl_threshold_major
            || selection.policy.human_approval_status == "Approved")
    {
        promoted_facts.push(scout_fact(
            "HumanApproval",
            "scout.selection.human-approval",
            "human approval state is recorded for the selected commitment amount and HITL threshold",
            vec![human_approval_state, over_threshold_without_approval],
            &selection.promotion_authority,
        ));
    }

    if has_truth_key(&selection.truth_keys, SCOUT_PROVENANCE_TRUTH_KEY)
        && selection.source_pack.source_hash.starts_with("sha256:")
        && !selection.screening.evidence_refs.is_empty()
        && !selection.policy.policy_id.trim().is_empty()
    {
        promoted_facts.push(scout_fact(
            "Provenance",
            "scout.selection.decision-provenance",
            "recommendation preserves source material, evidence refs, policy identity, and promotion authority",
            vec![decision_provenance, missing_source_provenance],
            &selection.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:scout-vendor-selection", 27, 8),
        replay_notes: vec![
            format!(
                "adapted Scout selection {} into AxiomRunObservation",
                selection.selection_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn scout_fact(
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
            evidence_id: format!("scout.evidence.{fact_id}"),
            source: "scout-vendor-selection-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("scout.trace.{fact_id}"),
            location: Some("scout://vendor-selection".to_string()),
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
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutVendorSelectionTranscript {
    source: ScoutRunSource,
    selection: ScoutSelectionOutcome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutSelectionOutcome {
    selection_id: String,
    truth_key: String,
    status: String,
    truth_keys: Vec<String>,
    source_pack: ScoutSourcePack,
    intent: ScoutIntentOutcome,
    formation: ScoutFormationOutcome,
    vendors: Vec<ScoutVendorOutcome>,
    screening: ScoutScreeningOutcome,
    shortlist: ScoutShortlistOutcome,
    policy: ScoutPolicyOutcome,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutSourcePack {
    document_ref: String,
    criteria_ref: String,
    static_fact_refs: Vec<String>,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutIntentOutcome {
    intent_id: String,
    principal: String,
    principal_authority: String,
    domain: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutFormationOutcome {
    formation_id: String,
    roles: Vec<String>,
    strategy: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutVendorOutcome {
    name: String,
    score: f64,
    risk_score: f64,
    compliance_status: String,
    certifications: Vec<String>,
    monthly_cost_major: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutScreeningOutcome {
    screened_vendor_count: usize,
    evidence_channels: Vec<String>,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutShortlistOutcome {
    selected_vendor: String,
    selected_amount_major: u64,
    entries: Vec<ScoutShortlistEntry>,
    rejected: Vec<ScoutRejectedVendor>,
    objective: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutShortlistEntry {
    rank: u16,
    vendor_name: String,
    composite_score: f64,
    rationale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutRejectedVendor {
    vendor_name: String,
    reasons: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScoutPolicyOutcome {
    policy_id: String,
    outcome: String,
    reason: String,
    hitl_threshold_major: u64,
    human_approval_status: String,
    gates_met: bool,
}

fn scout_vendor_selection_transcript() -> ScoutVendorSelectionTranscript {
    serde_json::from_str(SCOUT_VENDOR_SELECTION_TRANSCRIPT)
        .expect("Scout vendor selection transcript parses")
}
