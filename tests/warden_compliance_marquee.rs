//! Warden Compliance — fifth app probe for the Axiom/Helm contract.
//!
//! Tally proved irreversible release, Atlas proved integration writeback
//! readiness, Quorum proved contested sensemaking, and Scout proved governed
//! procurement. Warden pressures the shared operator ledger with compliance
//! registry shadow-runs, approval receipts, publication receipts, and the
//! boundary that Warden informs app gates without becoming a production gate.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

const WARDEN_REGISTRY_METADATA_TRUTH_KEY: &str = "registry-metadata-complete";
const WARDEN_IMMUTABLE_REGISTRY_TRUTH_KEY: &str = "immutable-registry-version";
const WARDEN_CORPUS_SCOPE_TRUTH_KEY: &str = "corpus-scope-named";
const WARDEN_SHADOW_DIFF_TRUTH_KEY: &str = "shadow-diff-produced";
const WARDEN_VERDICT_TRACE_TRUTH_KEY: &str = "verdicts-trace-to-rules";
const WARDEN_IMPACT_BREAKDOWN_TRUTH_KEY: &str = "impact-breakdown-by-app";
const WARDEN_APPROVAL_TRUTH_KEY: &str = "compliance-approval-recorded";
const WARDEN_PUBLICATION_TRUTH_KEY: &str = "publication-receipt-issued";
const WARDEN_PRODUCTION_BOUNDARY_TRUTH_KEY: &str = "warden-not-production-gate";
const WARDEN_ADAPTER_ID: &str = "warden-compliance.shadow-run-to-axiom-observation";
const WARDEN_ADAPTER_VERSION: &str = "fixture.v0.1";
const WARDEN_SHADOW_RUN_TRANSCRIPT: &str =
    include_str!("fixtures/warden_compliance_shadow_run_transcript.json");

fn warden_registry_shadow_run_jtbd() -> JtbdInput {
    JtbdInput {
        key: "warden-registry-shadow-run".to_string(),
        actor: "compliance operator".to_string(),
        functional_job:
            "publish a compliance rule registry only after a shadow run explains rule effects, approvals, and rollback"
                .to_string(),
        so_that:
            "applications can reload reviewed compliance policy without Warden becoming their production gate"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "registry_metadata_complete",
                "every proposed rule carries framework, field, owner, citation, effective window, and rationale",
            ),
            ClauseInput::with_key(
                "immutable_registry_version",
                "the registry publication names a new immutable version, source hash, and prior version",
            ),
            ClauseInput::with_key(
                "corpus_scope_named",
                "the shadow run names the historical corpus, time window, source applications, and exclusions",
            ),
            ClauseInput::with_key(
                "shadow_diff_produced",
                "the shadow run produces baseline versus proposed verdict counts and deltas",
            ),
            ClauseInput::with_key(
                "verdict_traceability",
                "every proposed verdict traces to a registry rule, source app, document id, field, and action",
            ),
            ClauseInput::with_key(
                "impact_breakdown",
                "operator review can inspect impact breakdown by framework, rule, and source app",
            ),
            ClauseInput::with_key(
                "compliance_approval_state",
                "compliance approval is recorded before the proposed registry is published",
            ),
            ClauseInput::with_key(
                "publication_receipt",
                "registry publication emits a receipt with publication time, subscriber count, and rollback reference",
            ),
            ClauseInput::with_key(
                "production_boundary",
                "Warden records that applications reload the registry but Warden does not run production gates",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "rule_without_owner",
                "a registry rule can be published without an accountable owner or citation",
            ),
            ClauseInput::with_key(
                "mutable_registry_publish",
                "an existing registry version is overwritten instead of publishing a new immutable version",
            ),
            ClauseInput::with_key(
                "unnamed_corpus",
                "operator review sees shadow results without the corpus and exclusions that produced them",
            ),
            ClauseInput::with_key(
                "verdict_without_rule",
                "a verdict appears in the shadow run without tracing to a proposed registry rule",
            ),
            ClauseInput::with_key(
                "hidden_impact",
                "rule impacts are summarized without framework, rule, and source-app breakdowns",
            ),
            ClauseInput::with_key(
                "unreviewed_publication",
                "a registry publication proceeds before compliance approval is recorded",
            ),
            ClauseInput::with_key(
                "missing_rollback",
                "a publication receipt lacks a rollback reference to the last known good registry",
            ),
            ClauseInput::with_key(
                "warden_as_production_gate",
                "applications treat Warden's shadow-run registry as direct production gate authority",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(30)),
    }
}

#[test]
fn warden_shadow_registry_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Publish a compliance rule registry")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        9
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        8
    );
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("historical corpus"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("production gate authority"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn warden_shadow_run_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");
    let transcript = warden_shadow_run_transcript();

    let observation =
        adapt_warden_shadow_run_transcript(&package, &transcript).expect("Warden adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.warden-shadow-run")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Warden-adapted shadow run preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 9);
    assert_eq!(audit.failure_coverage.len(), 8);
    assert_eq!(audit.facts_audited, 9);
}

#[test]
fn warden_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");
    let transcript = warden_shadow_run_transcript();

    let first = adapt_warden_shadow_run_transcript_with_receipt(&package, &transcript);
    let second = adapt_warden_shadow_run_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, WARDEN_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "warden-compliance");
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
            "warden.shadow.registry-metadata",
            "warden.shadow.immutable-registry",
            "warden.shadow.corpus-scope",
            "warden.shadow.shadow-diff",
            "warden.shadow.verdict-traceability",
            "warden.shadow.impact-breakdown",
            "warden.shadow.compliance-approval",
            "warden.shadow.publication-receipt",
            "warden.shadow.production-boundary",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 17);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("compliance@reflective.example"));
    assert!(!serialized.contains("GDPR Article 44"));
    assert!(!serialized.contains("doc-triple-violation"));
    assert!(!serialized.contains("WARDEN_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/warden-compliance"));
}

#[test]
fn warden_job_readiness_packet_marks_missing_corpus_scope() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");
    let mut transcript = warden_shadow_run_transcript();
    transcript
        .shadow_run
        .truth_keys
        .retain(|truth_key| truth_key != WARDEN_CORPUS_SCOPE_TRUTH_KEY);
    transcript.shadow_run.corpus.document_count = 0;
    transcript.shadow_run.corpus.source_apps.clear();
    let adapter_outcome = adapt_warden_shadow_run_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let corpus_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "corpus_scope_named")
        .expect("corpus scope evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(corpus_status.status, EvidenceReadinessStatus::Missing);
    assert!(corpus_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for corpus_scope_named".to_string())
    );
}

#[test]
fn warden_job_readiness_packet_marks_publication_without_approval() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");
    let mut transcript = warden_shadow_run_transcript();
    transcript
        .shadow_run
        .truth_keys
        .retain(|truth_key| truth_key != WARDEN_APPROVAL_TRUTH_KEY);
    transcript.shadow_run.approval.status = "Pending".to_string();
    let adapter_outcome = adapt_warden_shadow_run_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "compliance_approval_state")
        .expect("compliance approval evidence is represented");
    let publication_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "publication_receipt")
        .expect("publication receipt evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(publication_status.status, EvidenceReadinessStatus::Present);
    assert!(approval_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for compliance_approval_state".to_string())
    );
}

#[test]
fn warden_operator_ledger_entries_are_deterministic_backlinks_without_registry_authority() {
    let package = decode_jtbd(warden_registry_shadow_run_jtbd()).expect("Warden JTBD decodes");
    let transcript = warden_shadow_run_transcript();
    let adapter_outcome = adapt_warden_shadow_run_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_receipt = compliance_approval_receipt(&packet, &transcript);
    let publication_receipt = registry_publication_receipt(&packet, &transcript, &approval_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &approval_receipt,
        &publication_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &approval_receipt,
        &publication_receipt,
    );

    assert_eq!(first, second);
    assert_eq!(first.len(), 4);
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
        HelmLedgerRecordKind::ComplianceApprovalReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::RegistryPublicationReceipt
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
            approval_receipt.receipt_id.clone(),
        ]
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("compliance@reflective.example"));
    assert!(!serialized.contains("compliance.officer.primary"));
    assert!(!serialized.contains("GDPR Article 44"));
    assert!(!serialized.contains("doc-triple-violation"));
    assert!(!serialized.contains("WARDEN_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/warden-compliance"));
}

fn adapt_warden_shadow_run_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &WardenComplianceShadowTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_warden_shadow_run_transcript(package, transcript) {
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
    transcript: &WardenComplianceShadowTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: WARDEN_ADAPTER_ID.to_string(),
        adapter_version: WARDEN_ADAPTER_VERSION.to_string(),
        status,
        source_app: "warden-compliance".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "warden://shadow-run/{}/{}",
            transcript.source.run_id, transcript.shadow_run.shadow_run_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "registry.rules.owner".to_string(),
            "registry.rules.citation".to_string(),
            "verdicts.doc_id".to_string(),
            "source.command".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &WardenComplianceShadowTranscript,
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
        &transcript.shadow_run.shadow_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!(
            "warden://shadow-run/{}",
            transcript.shadow_run.shadow_run_id
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
    actions.push("route registry publication through Helm operator review".to_string());
    actions.push(
        "confirm apps consume registry version without delegating production gates to Warden"
            .to_string(),
    );
    actions
}

fn compliance_approval_receipt(
    packet: &JobReadinessPacket,
    transcript: &WardenComplianceShadowTranscript,
) -> ComplianceApprovalReceipt {
    let approval = &transcript.shadow_run.approval;
    ComplianceApprovalReceipt {
        receipt_id: compliance_approval_receipt_id(packet, approval),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        shadow_run_ref: packet.subject_ref.clone(),
        approval_ref_hash: sha256_lines(&[approval.approval_id.as_str()]),
        status: approval.status.clone(),
        scope_hash: sha256_lines(&[approval.scope.as_str()]),
        note_hash: approval.note_hash.clone(),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn registry_publication_receipt(
    packet: &JobReadinessPacket,
    transcript: &WardenComplianceShadowTranscript,
    approval_receipt: &ComplianceApprovalReceipt,
) -> RegistryPublicationLedgerReceipt {
    let publication = &transcript.shadow_run.publication_receipt;
    RegistryPublicationLedgerReceipt {
        receipt_id: registry_publication_receipt_id(packet, publication, approval_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        registry_version: publication.registry_version.clone(),
        publication_ref_hash: sha256_lines(&[publication.receipt_id.as_str()]),
        status: publication.status.clone(),
        published_at: publication.published_at.clone(),
        subscriber_count: publication.subscriber_count,
        rollback_ref_hash: sha256_lines(&[publication.rollback_ref.as_str()]),
        approval_receipt_id: approval_receipt.receipt_id.clone(),
        job_readiness_packet_id: packet.packet_id.clone(),
    }
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    approval_receipt: &ComplianceApprovalReceipt,
    publication_receipt: &RegistryPublicationLedgerReceipt,
) -> Vec<HelmLedgerEntry> {
    let receipt_payload_hash = sha256_json(receipt);
    let packet_payload_hash = sha256_json(packet);
    let approval_payload_hash = sha256_json(approval_receipt);
    let publication_payload_hash = sha256_json(publication_receipt);

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
            HelmLedgerRecordKind::ComplianceApprovalReceipt,
            approval_receipt.receipt_id.clone(),
            approval_receipt.package_id.clone(),
            approval_receipt.truth_version.clone(),
            approval_receipt.domain_hint.clone(),
            approval_payload_hash,
            vec![packet.packet_id.clone()],
            format!("compliance approval {}", approval_receipt.status),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::RegistryPublicationReceipt,
            publication_receipt.receipt_id.clone(),
            publication_receipt.package_id.clone(),
            publication_receipt.truth_version.clone(),
            publication_receipt.domain_hint.clone(),
            publication_payload_hash,
            vec![
                packet.packet_id.clone(),
                approval_receipt.receipt_id.clone(),
            ],
            format!(
                "registry publication {} for {}",
                publication_receipt.status, publication_receipt.registry_version
            ),
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

fn adapt_warden_shadow_run_transcript(
    package: &TruthPackage,
    transcript: &WardenComplianceShadowTranscript,
) -> Result<AxiomRunObservation, String> {
    let shadow_run = &transcript.shadow_run;
    if shadow_run.status != "Converged" {
        return Err("expected Warden shadow run to converge before adaptation".to_string());
    }
    if shadow_run.registry.rules.is_empty() {
        return Err("expected Warden shadow run to carry registry rules".to_string());
    }
    if shadow_run.verdicts.is_empty() {
        return Err("expected Warden shadow run to carry proposed verdicts".to_string());
    }

    let registry_metadata_complete = evidence_clause_id(package, "registry_metadata_complete");
    let immutable_registry_version = evidence_clause_id(package, "immutable_registry_version");
    let corpus_scope_named = evidence_clause_id(package, "corpus_scope_named");
    let shadow_diff_produced = evidence_clause_id(package, "shadow_diff_produced");
    let verdict_traceability = evidence_clause_id(package, "verdict_traceability");
    let impact_breakdown = evidence_clause_id(package, "impact_breakdown");
    let compliance_approval_state = evidence_clause_id(package, "compliance_approval_state");
    let publication_receipt = evidence_clause_id(package, "publication_receipt");
    let production_boundary = evidence_clause_id(package, "production_boundary");
    let rule_without_owner = failure_clause_id(package, "rule_without_owner");
    let mutable_registry_publish = failure_clause_id(package, "mutable_registry_publish");
    let unnamed_corpus = failure_clause_id(package, "unnamed_corpus");
    let verdict_without_rule = failure_clause_id(package, "verdict_without_rule");
    let hidden_impact = failure_clause_id(package, "hidden_impact");
    let unreviewed_publication = failure_clause_id(package, "unreviewed_publication");
    let missing_rollback = failure_clause_id(package, "missing_rollback");
    let warden_as_production_gate = failure_clause_id(package, "warden_as_production_gate");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&shadow_run.truth_keys, WARDEN_REGISTRY_METADATA_TRUTH_KEY)
        && shadow_run.registry.source_hash.starts_with("sha256:")
        && shadow_run
            .registry
            .rules
            .iter()
            .all(WardenRule::is_complete)
    {
        promoted_facts.push(warden_fact(
            "Registry",
            "warden.shadow.registry-metadata",
            "proposed registry rules carry framework, field, owner, citation, effective window, and rationale",
            vec![registry_metadata_complete, rule_without_owner],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_IMMUTABLE_REGISTRY_TRUTH_KEY)
        && shadow_run.registry.current_version != shadow_run.registry.proposed_version
        && shadow_run.registry.source_hash.starts_with("sha256:")
    {
        promoted_facts.push(warden_fact(
            "RegistryVersion",
            "warden.shadow.immutable-registry",
            "proposed registry names a new immutable version, prior version, and source hash",
            vec![immutable_registry_version, mutable_registry_publish],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_CORPUS_SCOPE_TRUTH_KEY)
        && !shadow_run.corpus.corpus_ref.trim().is_empty()
        && !shadow_run.corpus.time_window.trim().is_empty()
        && shadow_run.corpus.document_count > 0
        && !shadow_run.corpus.source_apps.is_empty()
        && !shadow_run.corpus.excluded_cases.is_empty()
    {
        promoted_facts.push(warden_fact(
            "Corpus",
            "warden.shadow.corpus-scope",
            "shadow run names corpus, time window, source applications, and exclusions",
            vec![corpus_scope_named, unnamed_corpus],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_SHADOW_DIFF_TRUTH_KEY)
        && shadow_run.shadow_diff.proposed_verdicts == shadow_run.verdicts.len()
        && shadow_run.shadow_diff.proposed_verdicts >= shadow_run.shadow_diff.baseline_verdicts
        && shadow_run.shadow_diff.newly_blocked + shadow_run.shadow_diff.newly_paused > 0
    {
        promoted_facts.push(warden_fact(
            "ShadowDiff",
            "warden.shadow.shadow-diff",
            "shadow run produces baseline and proposed verdict counts with blocked and paused deltas",
            vec![shadow_diff_produced, hidden_impact.clone()],
            &shadow_run.promotion_authority,
        ));
    }

    let registry_rule_ids = shadow_run
        .registry
        .rules
        .iter()
        .map(|rule| rule.id.as_str())
        .collect::<BTreeSet<_>>();
    if has_truth_key(&shadow_run.truth_keys, WARDEN_VERDICT_TRACE_TRUTH_KEY)
        && shadow_run.verdicts.iter().all(|verdict| {
            registry_rule_ids.contains(verdict.rule_id.as_str())
                && !verdict.verdict_id.trim().is_empty()
                && !verdict.source_app.trim().is_empty()
                && !verdict.doc_id.trim().is_empty()
                && !verdict.field.trim().is_empty()
                && !verdict.action.trim().is_empty()
        })
    {
        promoted_facts.push(warden_fact(
            "Verdicts",
            "warden.shadow.verdict-traceability",
            "every shadow verdict traces to a registry rule, source app, document id, field, and action",
            vec![verdict_traceability, verdict_without_rule],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_IMPACT_BREAKDOWN_TRUTH_KEY)
        && !shadow_run.shadow_diff.by_framework.is_empty()
        && !shadow_run.shadow_diff.by_rule.is_empty()
        && !shadow_run.shadow_diff.by_source_app.is_empty()
    {
        promoted_facts.push(warden_fact(
            "ImpactBreakdown",
            "warden.shadow.impact-breakdown",
            "operator impact review can inspect framework, rule, and source-app breakdowns",
            vec![impact_breakdown, hidden_impact],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_APPROVAL_TRUTH_KEY)
        && shadow_run.approval.status == "Approved"
        && shadow_run.approval.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(warden_fact(
            "ComplianceApproval",
            "warden.shadow.compliance-approval",
            "compliance approval is recorded before registry publication",
            vec![compliance_approval_state, unreviewed_publication],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_PUBLICATION_TRUTH_KEY)
        && shadow_run.publication_receipt.status == "Published"
        && shadow_run.publication_receipt.registry_version == shadow_run.registry.proposed_version
        && !shadow_run
            .publication_receipt
            .rollback_ref
            .trim()
            .is_empty()
        && shadow_run.publication_receipt.subscriber_count > 0
    {
        promoted_facts.push(warden_fact(
            "Publication",
            "warden.shadow.publication-receipt",
            "registry publication receipt names publication time, subscriber count, and rollback reference",
            vec![publication_receipt, missing_rollback],
            &shadow_run.promotion_authority,
        ));
    }

    if has_truth_key(&shadow_run.truth_keys, WARDEN_PRODUCTION_BOUNDARY_TRUTH_KEY)
        && !shadow_run.production_boundary.warden_runs_production_gates
        && shadow_run.production_boundary.apps_reload_registry
    {
        promoted_facts.push(warden_fact(
            "ProductionBoundary",
            "warden.shadow.production-boundary",
            "applications reload the registry while Warden remains outside production gate authority",
            vec![production_boundary, warden_as_production_gate],
            &shadow_run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:warden-shadow-run", 41, 9),
        replay_notes: vec![
            format!(
                "adapted Warden shadow run {} into AxiomRunObservation",
                shadow_run.shadow_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn warden_fact(
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
            evidence_id: format!("warden.evidence.{fact_id}"),
            source: "warden-shadow-run-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("warden.trace.{fact_id}"),
            location: Some("warden://shadow-run".to_string()),
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
    shadow_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    let digest = sha256_lines(&[
        "job_readiness_packet",
        package.package_id.as_str(),
        package.truth_version.as_str(),
        domain_hint,
        shadow_run_id,
        adapter_receipt_id,
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.job_readiness.{short_digest}")
}

fn compliance_approval_receipt_id(
    packet: &JobReadinessPacket,
    approval: &WardenApproval,
) -> String {
    let digest = sha256_lines(&[
        "compliance_approval_receipt",
        packet.packet_id.as_str(),
        approval.approval_id.as_str(),
        approval.status.as_str(),
        approval.note_hash.as_str(),
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.compliance_approval.{short_digest}")
}

fn registry_publication_receipt_id(
    packet: &JobReadinessPacket,
    publication: &WardenPublicationReceipt,
    approval_receipt: &ComplianceApprovalReceipt,
) -> String {
    let digest = sha256_lines(&[
        "registry_publication_receipt",
        packet.packet_id.as_str(),
        publication.receipt_id.as_str(),
        publication.registry_version.as_str(),
        publication.status.as_str(),
        approval_receipt.receipt_id.as_str(),
    ]);
    let short_digest = &digest
        .strip_prefix("sha256:")
        .expect("local digest has sha256 prefix")[..12];
    format!("helm.registry_publication.{short_digest}")
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
struct ComplianceApprovalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    shadow_run_ref: String,
    approval_ref_hash: String,
    status: String,
    scope_hash: String,
    note_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct RegistryPublicationLedgerReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    registry_version: String,
    publication_ref_hash: String,
    status: String,
    published_at: String,
    subscriber_count: usize,
    rollback_ref_hash: String,
    approval_receipt_id: String,
    job_readiness_packet_id: String,
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
    ComplianceApprovalReceipt,
    RegistryPublicationReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::ComplianceApprovalReceipt => "compliance_approval_receipt",
            Self::RegistryPublicationReceipt => "registry_publication_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenComplianceShadowTranscript {
    source: WardenRunSource,
    shadow_run: WardenShadowRunOutcome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenShadowRunOutcome {
    shadow_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    registry: WardenRegistry,
    corpus: WardenCorpus,
    shadow_diff: WardenShadowDiff,
    verdicts: Vec<WardenVerdict>,
    approval: WardenApproval,
    publication_receipt: WardenPublicationReceipt,
    production_boundary: WardenProductionBoundary,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenRegistry {
    current_version: String,
    proposed_version: String,
    source_hash: String,
    rules: Vec<WardenRule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenRule {
    id: String,
    framework: String,
    field: String,
    owner: String,
    citation: String,
    effective: String,
    sunset: String,
    rationale: String,
}

impl WardenRule {
    fn is_complete(&self) -> bool {
        [
            self.id.as_str(),
            self.framework.as_str(),
            self.field.as_str(),
            self.owner.as_str(),
            self.citation.as_str(),
            self.effective.as_str(),
            self.sunset.as_str(),
            self.rationale.as_str(),
        ]
        .iter()
        .all(|field| !field.trim().is_empty())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenCorpus {
    corpus_ref: String,
    time_window: String,
    document_count: usize,
    source_apps: Vec<String>,
    excluded_cases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenShadowDiff {
    baseline_verdicts: usize,
    proposed_verdicts: usize,
    newly_blocked: usize,
    newly_paused: usize,
    newly_logged: usize,
    by_framework: BTreeMap<String, usize>,
    by_source_app: BTreeMap<String, usize>,
    by_rule: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenVerdict {
    verdict_id: String,
    framework: String,
    source_app: String,
    doc_id: String,
    rule_id: String,
    field: String,
    action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenApproval {
    approval_id: String,
    approver_id: String,
    status: String,
    scope: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenPublicationReceipt {
    receipt_id: String,
    registry_version: String,
    status: String,
    published_at: String,
    rollback_ref: String,
    subscriber_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct WardenProductionBoundary {
    warden_runs_production_gates: bool,
    apps_reload_registry: bool,
    last_known_good_registry: String,
}

fn warden_shadow_run_transcript() -> WardenComplianceShadowTranscript {
    serde_json::from_str(WARDEN_SHADOW_RUN_TRANSCRIPT)
        .expect("Warden compliance shadow run transcript parses")
}
