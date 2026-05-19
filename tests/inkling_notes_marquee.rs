//! Inkling Notes — seventh app probe for the Axiom/Helm contract.
//!
//! The operator apps proved readiness, approval, publication, and plan
//! receipts. Inkling pressures a different product family: local-first private
//! corpus enrichment. The core boundary is that generated metadata and cleanup
//! suggestions may help the user, but original notes are not silently rewritten
//! and network/OCR work follows explicit permission.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const INKLING_SNAPSHOT_TRUTH_KEY: &str = "vault-snapshot-captured";
const INKLING_IMPORT_PROVENANCE_TRUTH_KEY: &str = "import-provenance-preserved";
const INKLING_PERMISSIONS_TRUTH_KEY: &str = "permissions-declared";
const INKLING_METADATA_SEPARABLE_TRUTH_KEY: &str = "derived-metadata-separable";
const INKLING_DUPLICATES_TRUTH_KEY: &str = "duplicates-evidenced";
const INKLING_FRESHNESS_TRUTH_KEY: &str = "freshness-analysis-evidenced";
const INKLING_HUBS_TRUTH_KEY: &str = "project-hubs-traced";
const INKLING_CLEANUP_TRUTH_KEY: &str = "cleanup-suggestions-proposed";
const INKLING_DESTRUCTIVE_APPROVAL_TRUTH_KEY: &str = "destructive-change-approval-recorded";
const INKLING_LOCAL_BOUNDARY_TRUTH_KEY: &str = "local-first-boundary-preserved";
const INKLING_ADAPTER_ID: &str = "inkling-notes.vault-index-to-axiom-observation";
const INKLING_ADAPTER_VERSION: &str = "fixture.v0.1";
const INKLING_VAULT_TRANSCRIPT: &str =
    include_str!("fixtures/inkling_vault_navigation_transcript.json");

fn inkling_vault_navigation_jtbd() -> JtbdInput {
    JtbdInput {
        key: "inkling-vault-navigation".to_string(),
        actor: "knowledge worker".to_string(),
        functional_job:
            "turn a messy local notes vault into a useful navigation index without losing user control"
                .to_string(),
        so_that:
            "the user can recover meaning, commitments, and structure while original notes remain trustworthy"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "vault_snapshot_captured",
                "a vault snapshot is captured before enrichment or cleanup suggestions are produced",
            ),
            ClauseInput::with_key(
                "import_provenance_preserved",
                "imports preserve source kind, source path or reference, imported count, provenance refs, and source hash",
            ),
            ClauseInput::with_key(
                "permissions_declared",
                "OCR, PDF extraction, external link fetching, and destructive edit permissions are explicit",
            ),
            ClauseInput::with_key(
                "derived_metadata_separable",
                "generated tags, links, summaries, and attachment text remain separable from original notes",
            ),
            ClauseInput::with_key(
                "duplicate_groups_evidenced",
                "duplicate groups cite evidence refs and remain proposed until accepted by the user",
            ),
            ClauseInput::with_key(
                "freshness_analysis_evidenced",
                "stale-note and cleanup candidates cite freshness evidence and a next review date",
            ),
            ClauseInput::with_key(
                "project_hubs_traced",
                "project hubs cite graph, tag, or note evidence for why they are surfaced",
            ),
            ClauseInput::with_key(
                "cleanup_suggestions_are_proposals",
                "cleanup actions are represented as proposals, not applied mutations",
            ),
            ClauseInput::with_key(
                "destructive_changes_approved_or_absent",
                "destructive note changes are either absent or backed by snapshot and user approval",
            ),
            ClauseInput::with_key(
                "local_first_boundary",
                "the run records that original notes stay local-first and are not platform memory authority",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "destructive_rewrite_without_snapshot",
                "original notes are rewritten without a prior snapshot",
            ),
            ClauseInput::with_key(
                "generated_metadata_overwrites_notes",
                "generated metadata is written into original note bodies as if it were user-authored",
            ),
            ClauseInput::with_key(
                "external_fetch_without_permission",
                "external links are fetched without explicit permission",
            ),
            ClauseInput::with_key("ocr_without_permission", "OCR runs without explicit permission"),
            ClauseInput::with_key(
                "suggestion_without_evidence",
                "an inferred tag, link, summary, hub, or cleanup suggestion lacks evidence refs",
            ),
            ClauseInput::with_key(
                "duplicate_merge_without_acceptance",
                "duplicate notes are merged before the user accepts the cleanup proposal",
            ),
            ClauseInput::with_key(
                "stale_cleanup_hidden",
                "stale-note cleanup hides residual risk or next review timing",
            ),
            ClauseInput::with_key(
                "source_provenance_lost",
                "an imported note loses its source path, source reference, or import provenance",
            ),
            ClauseInput::with_key(
                "notes_as_platform_memory_authority",
                "the app treats the private notes vault as authoritative platform memory outside user control",
            ),
            ClauseInput::with_key(
                "network_fetch_hidden",
                "the run performs network fetching without a visible count and permission trail",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(25)),
    }
}

#[test]
fn inkling_vault_navigation_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Turn a messy local notes vault")
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
            .any(|evidence| evidence.contains("vault snapshot"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("external links"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn inkling_vault_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");
    let transcript = inkling_vault_transcript();

    let observation =
        adapt_inkling_vault_transcript(&package, &transcript).expect("Inkling adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.inkling-vault-index")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Inkling-adapted vault run preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 10);
    assert_eq!(audit.failure_coverage.len(), 10);
    assert_eq!(audit.facts_audited, 10);
}

#[test]
fn inkling_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");
    let transcript = inkling_vault_transcript();

    let first = adapt_inkling_vault_transcript_with_receipt(&package, &transcript);
    let second = adapt_inkling_vault_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, INKLING_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "inkling-notes");
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
            "inkling.vault.snapshot",
            "inkling.vault.import-provenance",
            "inkling.vault.permissions",
            "inkling.vault.metadata-separable",
            "inkling.vault.duplicates",
            "inkling.vault.freshness",
            "inkling.vault.project-hubs",
            "inkling.vault.cleanup-proposals",
            "inkling.vault.destructive-change-boundary",
            "inkling.vault.local-first-boundary",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 20);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("~/Notes"));
    assert!(!serialized.contains("AppleNotes"));
    assert!(!serialized.contains("Projects/Taxes/receipt-import.md"));
    assert!(!serialized.contains("INKLING_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/inkling-notes"));
}

#[test]
fn inkling_job_readiness_packet_marks_missing_permissions() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");
    let mut transcript = inkling_vault_transcript();
    transcript
        .vault_run
        .truth_keys
        .retain(|truth_key| truth_key != INKLING_PERMISSIONS_TRUTH_KEY);
    transcript.vault_run.permissions.external_link_fetch_enabled = true;
    transcript.vault_run.navigation_index.external_fetch_count = 12;
    let adapter_outcome = adapt_inkling_vault_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let permission_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "permissions_declared")
        .expect("permissions evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(permission_status.status, EvidenceReadinessStatus::Missing);
    assert!(permission_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for permissions_declared".to_string())
    );
}

#[test]
fn inkling_job_readiness_packet_marks_destructive_change_without_snapshot() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");
    let mut transcript = inkling_vault_transcript();
    transcript
        .vault_run
        .truth_keys
        .retain(|truth_key| truth_key != INKLING_SNAPSHOT_TRUTH_KEY);
    transcript.vault_run.vault.snapshot_hash.clear();
    transcript
        .vault_run
        .destructive_change
        .applied_changes_count = 2;
    transcript.vault_run.destructive_change.user_approval_status = "Pending".to_string();
    let adapter_outcome = adapt_inkling_vault_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let snapshot_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "vault_snapshot_captured")
        .expect("snapshot evidence is represented");
    let destructive_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "destructive_changes_approved_or_absent")
        .expect("destructive-change evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(snapshot_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(destructive_status.status, EvidenceReadinessStatus::Missing);
    assert!(snapshot_status.fact_ids.is_empty());
    assert!(destructive_status.fact_ids.is_empty());
}

#[test]
fn inkling_operator_ledger_entries_are_deterministic_backlinks_without_note_authority() {
    let package = decode_jtbd(inkling_vault_navigation_jtbd()).expect("Inkling JTBD decodes");
    let transcript = inkling_vault_transcript();
    let adapter_outcome = adapt_inkling_vault_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let snapshot_receipt = vault_snapshot_receipt(&packet, &transcript);
    let permission_receipt = vault_permission_receipt(&packet, &transcript, &snapshot_receipt);
    let index_receipt =
        vault_index_receipt(&packet, &transcript, &snapshot_receipt, &permission_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &snapshot_receipt,
        &permission_receipt,
        &index_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &snapshot_receipt,
        &permission_receipt,
        &index_receipt,
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
        HelmLedgerRecordKind::VaultSnapshotReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::PermissionReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::VaultIndexReceipt
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("~/Notes"));
    assert!(!serialized.contains("AppleNotes"));
    assert!(!serialized.contains("Projects/Taxes/receipt-import.md"));
    assert!(!serialized.contains("INKLING_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/inkling-notes"));
}

fn adapt_inkling_vault_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &InklingVaultNavigationTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_inkling_vault_transcript(package, transcript) {
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
    transcript: &InklingVaultNavigationTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: INKLING_ADAPTER_ID.to_string(),
        adapter_version: INKLING_ADAPTER_VERSION.to_string(),
        status,
        source_app: "inkling-notes".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "inkling://vault-run/{}/{}",
            transcript.source.run_id, transcript.vault_run.vault_run_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "vault.root_path".to_string(),
            "import.source_path".to_string(),
            "duplicate_groups.note_ids".to_string(),
            "project_hubs.note_refs".to_string(),
            "cleanup_suggestions.target_refs".to_string(),
            "source.command".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &InklingVaultNavigationTranscript,
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
        &transcript.vault_run.vault_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!("inkling://vault-run/{}", transcript.vault_run.vault_run_id),
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
    actions.push("review vault suggestions before applying any cleanup".to_string());
    actions.push("confirm generated metadata stays separate from original notes".to_string());
    actions
}

fn vault_snapshot_receipt(
    packet: &JobReadinessPacket,
    transcript: &InklingVaultNavigationTranscript,
) -> VaultSnapshotReceipt {
    let vault = &transcript.vault_run.vault;
    VaultSnapshotReceipt {
        receipt_id: vault_snapshot_receipt_id(packet, vault),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        vault_run_ref: packet.subject_ref.clone(),
        snapshot_ref_hash: sha256_lines(&[vault.snapshot_id.as_str()]),
        snapshot_hash: vault.snapshot_hash.clone(),
        note_count: vault.note_count,
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn vault_permission_receipt(
    packet: &JobReadinessPacket,
    transcript: &InklingVaultNavigationTranscript,
    snapshot_receipt: &VaultSnapshotReceipt,
) -> PermissionReceipt {
    let permissions = &transcript.vault_run.permissions;
    PermissionReceipt {
        receipt_id: permission_receipt_id(packet, permissions, snapshot_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        permission_ref_hash: sha256_lines(&[permissions.permission_receipt_id.as_str()]),
        pdf_extraction_enabled: permissions.pdf_extraction_enabled,
        image_ocr_enabled: permissions.image_ocr_enabled,
        external_link_fetch_enabled: permissions.external_link_fetch_enabled,
        destructive_edits_allowed: permissions.destructive_edits_allowed,
        snapshot_receipt_id: snapshot_receipt.receipt_id.clone(),
    }
}

fn vault_index_receipt(
    packet: &JobReadinessPacket,
    transcript: &InklingVaultNavigationTranscript,
    snapshot_receipt: &VaultSnapshotReceipt,
    permission_receipt: &PermissionReceipt,
) -> VaultIndexReceipt {
    let index = &transcript.vault_run.navigation_index;
    VaultIndexReceipt {
        receipt_id: vault_index_receipt_id(packet, index, permission_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        index_ref_hash: sha256_lines(&[index.index_id.as_str()]),
        output_ref_hash: sha256_lines(&[index.output_ref.as_str()]),
        graph_link_count: index.graph_link_count,
        inferred_tag_count: index.inferred_tag_count,
        snapshot_receipt_id: snapshot_receipt.receipt_id.clone(),
        permission_receipt_id: permission_receipt.receipt_id.clone(),
        job_readiness_packet_id: packet.packet_id.clone(),
    }
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    snapshot_receipt: &VaultSnapshotReceipt,
    permission_receipt: &PermissionReceipt,
    index_receipt: &VaultIndexReceipt,
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
            HelmLedgerRecordKind::VaultSnapshotReceipt,
            snapshot_receipt.receipt_id.clone(),
            snapshot_receipt.package_id.clone(),
            snapshot_receipt.truth_version.clone(),
            snapshot_receipt.domain_hint.clone(),
            sha256_json(snapshot_receipt),
            vec![packet.packet_id.clone()],
            "vault snapshot captured".to_string(),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::PermissionReceipt,
            permission_receipt.receipt_id.clone(),
            permission_receipt.package_id.clone(),
            permission_receipt.truth_version.clone(),
            permission_receipt.domain_hint.clone(),
            sha256_json(permission_receipt),
            vec![
                packet.packet_id.clone(),
                snapshot_receipt.receipt_id.clone(),
            ],
            "vault permissions recorded".to_string(),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::VaultIndexReceipt,
            index_receipt.receipt_id.clone(),
            index_receipt.package_id.clone(),
            index_receipt.truth_version.clone(),
            index_receipt.domain_hint.clone(),
            sha256_json(index_receipt),
            vec![
                packet.packet_id.clone(),
                snapshot_receipt.receipt_id.clone(),
                permission_receipt.receipt_id.clone(),
            ],
            "vault index emitted".to_string(),
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

fn adapt_inkling_vault_transcript(
    package: &TruthPackage,
    transcript: &InklingVaultNavigationTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.vault_run;
    if run.status != "Converged" {
        return Err("expected Inkling vault run to converge before adaptation".to_string());
    }
    if run.vault.note_count == 0 {
        return Err("expected Inkling vault run to carry note count".to_string());
    }
    if run.cleanup_suggestions.is_empty() {
        return Err("expected Inkling vault run to carry cleanup proposals".to_string());
    }

    let vault_snapshot_captured = evidence_clause_id(package, "vault_snapshot_captured");
    let import_provenance_preserved = evidence_clause_id(package, "import_provenance_preserved");
    let permissions_declared = evidence_clause_id(package, "permissions_declared");
    let derived_metadata_separable = evidence_clause_id(package, "derived_metadata_separable");
    let duplicate_groups_evidenced = evidence_clause_id(package, "duplicate_groups_evidenced");
    let freshness_analysis_evidenced = evidence_clause_id(package, "freshness_analysis_evidenced");
    let project_hubs_traced = evidence_clause_id(package, "project_hubs_traced");
    let cleanup_suggestions_are_proposals =
        evidence_clause_id(package, "cleanup_suggestions_are_proposals");
    let destructive_changes_approved_or_absent =
        evidence_clause_id(package, "destructive_changes_approved_or_absent");
    let local_first_boundary = evidence_clause_id(package, "local_first_boundary");
    let destructive_rewrite_without_snapshot =
        failure_clause_id(package, "destructive_rewrite_without_snapshot");
    let generated_metadata_overwrites_notes =
        failure_clause_id(package, "generated_metadata_overwrites_notes");
    let external_fetch_without_permission =
        failure_clause_id(package, "external_fetch_without_permission");
    let ocr_without_permission = failure_clause_id(package, "ocr_without_permission");
    let suggestion_without_evidence = failure_clause_id(package, "suggestion_without_evidence");
    let duplicate_merge_without_acceptance =
        failure_clause_id(package, "duplicate_merge_without_acceptance");
    let stale_cleanup_hidden = failure_clause_id(package, "stale_cleanup_hidden");
    let source_provenance_lost = failure_clause_id(package, "source_provenance_lost");
    let notes_as_platform_memory_authority =
        failure_clause_id(package, "notes_as_platform_memory_authority");
    let network_fetch_hidden = failure_clause_id(package, "network_fetch_hidden");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&run.truth_keys, INKLING_SNAPSHOT_TRUTH_KEY)
        && !run.vault.snapshot_id.trim().is_empty()
        && run.vault.snapshot_hash.starts_with("sha256:")
        && run.vault.note_count > 0
    {
        promoted_facts.push(inkling_fact(
            "VaultSnapshot",
            "inkling.vault.snapshot",
            "vault snapshot exists before enrichment and cleanup proposals",
            vec![
                vault_snapshot_captured,
                destructive_rewrite_without_snapshot.clone(),
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_IMPORT_PROVENANCE_TRUTH_KEY)
        && !run.import.source_kind.trim().is_empty()
        && !run.import.source_path.trim().is_empty()
        && run.import.imported_note_count > 0
        && !run.import.provenance_refs.is_empty()
        && run.import.source_hash.starts_with("sha256:")
    {
        promoted_facts.push(inkling_fact(
            "ImportProvenance",
            "inkling.vault.import-provenance",
            "import preserves source identity, note count, provenance refs, and source hash",
            vec![import_provenance_preserved, source_provenance_lost],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_PERMISSIONS_TRUTH_KEY)
        && !run.permissions.permission_receipt_id.trim().is_empty()
        && !run.permissions.policy_ref.trim().is_empty()
        && (!run.permissions.external_link_fetch_enabled
            || run.navigation_index.external_fetch_count > 0)
    {
        promoted_facts.push(inkling_fact(
            "Permissions",
            "inkling.vault.permissions",
            "vault run declares PDF, OCR, external fetch, and destructive-edit permissions",
            vec![
                permissions_declared,
                external_fetch_without_permission,
                ocr_without_permission,
                network_fetch_hidden.clone(),
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_METADATA_SEPARABLE_TRUTH_KEY)
        && run.local_boundary.generated_metadata_separable
        && !run
            .navigation_index
            .generated_metadata_ref
            .trim()
            .is_empty()
    {
        promoted_facts.push(inkling_fact(
            "GeneratedMetadata",
            "inkling.vault.metadata-separable",
            "generated tags, links, summaries, and attachment text remain outside original note bodies",
            vec![derived_metadata_separable, generated_metadata_overwrites_notes],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_DUPLICATES_TRUTH_KEY)
        && run.duplicate_groups.iter().all(|group| {
            group.status == "Proposed"
                && !group.note_ids.is_empty()
                && !group.evidence_refs.is_empty()
        })
    {
        promoted_facts.push(inkling_fact(
            "DuplicateGroups",
            "inkling.vault.duplicates",
            "duplicate groups cite evidence refs and remain proposed",
            vec![
                duplicate_groups_evidenced,
                duplicate_merge_without_acceptance.clone(),
                suggestion_without_evidence.clone(),
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_FRESHNESS_TRUTH_KEY)
        && run.freshness.stale_candidate_count > 0
        && !run.freshness.review_date.trim().is_empty()
        && !run.freshness.evidence_refs.is_empty()
        && !run.freshness.cleanup_hidden
    {
        promoted_facts.push(inkling_fact(
            "Freshness",
            "inkling.vault.freshness",
            "stale-note candidates cite freshness evidence and next review date",
            vec![freshness_analysis_evidenced, stale_cleanup_hidden],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_HUBS_TRUTH_KEY)
        && run.project_hubs.iter().all(|hub| {
            hub.confidence_basis_points > 0
                && !hub.note_refs.is_empty()
                && !hub.evidence_refs.is_empty()
        })
    {
        promoted_facts.push(inkling_fact(
            "ProjectHubs",
            "inkling.vault.project-hubs",
            "project hubs cite graph, tag, and note evidence",
            vec![project_hubs_traced, suggestion_without_evidence],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_CLEANUP_TRUTH_KEY)
        && run.cleanup_suggestions.iter().all(|suggestion| {
            suggestion.status == "Proposed"
                && !suggestion.target_refs.is_empty()
                && !suggestion.evidence_refs.is_empty()
        })
    {
        promoted_facts.push(inkling_fact(
            "CleanupSuggestions",
            "inkling.vault.cleanup-proposals",
            "cleanup actions remain proposals with target refs and evidence refs",
            vec![
                cleanup_suggestions_are_proposals,
                duplicate_merge_without_acceptance,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_DESTRUCTIVE_APPROVAL_TRUTH_KEY)
        && run.destructive_change.snapshot_required
        && (run.destructive_change.applied_changes_count == 0
            || run.destructive_change.user_approval_status == "Approved")
    {
        promoted_facts.push(inkling_fact(
            "DestructiveChangeBoundary",
            "inkling.vault.destructive-change-boundary",
            "destructive note changes are absent or require snapshot and user approval",
            vec![
                destructive_changes_approved_or_absent,
                destructive_rewrite_without_snapshot,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, INKLING_LOCAL_BOUNDARY_TRUTH_KEY)
        && !run.local_boundary.original_notes_mutated
        && run.local_boundary.local_first_storage
        && !run.local_boundary.platform_memory_authority_claimed
        && (!run.local_boundary.network_fetches_attempted
            || run.permissions.external_link_fetch_enabled)
    {
        promoted_facts.push(inkling_fact(
            "LocalFirstBoundary",
            "inkling.vault.local-first-boundary",
            "original notes stay local-first and private vault data is not platform memory authority",
            vec![
                local_first_boundary,
                notes_as_platform_memory_authority,
                network_fetch_hidden,
            ],
            &run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:inkling-vault-index", 29, 10),
        replay_notes: vec![
            format!(
                "adapted Inkling vault run {} into AxiomRunObservation",
                run.vault_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn inkling_fact(
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
            evidence_id: format!("inkling.evidence.{fact_id}"),
            source: "inkling-vault-index-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("inkling.trace.{fact_id}"),
            location: Some("inkling://vault-run".to_string()),
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
    vault_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    short_id(
        &sha256_lines(&[
            "job_readiness_packet",
            package.package_id.as_str(),
            package.truth_version.as_str(),
            domain_hint,
            vault_run_id,
            adapter_receipt_id,
        ]),
        "helm.job_readiness",
    )
}

fn vault_snapshot_receipt_id(packet: &JobReadinessPacket, vault: &InklingVault) -> String {
    short_id(
        &sha256_lines(&[
            "vault_snapshot_receipt",
            packet.packet_id.as_str(),
            vault.snapshot_id.as_str(),
            vault.snapshot_hash.as_str(),
        ]),
        "helm.vault_snapshot",
    )
}

fn permission_receipt_id(
    packet: &JobReadinessPacket,
    permissions: &InklingPermissions,
    snapshot_receipt: &VaultSnapshotReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "permission_receipt",
            packet.packet_id.as_str(),
            permissions.permission_receipt_id.as_str(),
            snapshot_receipt.receipt_id.as_str(),
        ]),
        "helm.permission",
    )
}

fn vault_index_receipt_id(
    packet: &JobReadinessPacket,
    index: &InklingNavigationIndex,
    permission_receipt: &PermissionReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "vault_index_receipt",
            packet.packet_id.as_str(),
            index.index_id.as_str(),
            permission_receipt.receipt_id.as_str(),
        ]),
        "helm.vault_index",
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
struct VaultSnapshotReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    vault_run_ref: String,
    snapshot_ref_hash: String,
    snapshot_hash: String,
    note_count: usize,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct PermissionReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    permission_ref_hash: String,
    pdf_extraction_enabled: bool,
    image_ocr_enabled: bool,
    external_link_fetch_enabled: bool,
    destructive_edits_allowed: bool,
    snapshot_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct VaultIndexReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    index_ref_hash: String,
    output_ref_hash: String,
    graph_link_count: usize,
    inferred_tag_count: usize,
    snapshot_receipt_id: String,
    permission_receipt_id: String,
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
    VaultSnapshotReceipt,
    PermissionReceipt,
    VaultIndexReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::VaultSnapshotReceipt => "vault_snapshot_receipt",
            Self::PermissionReceipt => "permission_receipt",
            Self::VaultIndexReceipt => "vault_index_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingVaultNavigationTranscript {
    source: InklingRunSource,
    vault_run: InklingVaultRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingVaultRun {
    vault_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    vault: InklingVault,
    import: InklingImport,
    permissions: InklingPermissions,
    navigation_index: InklingNavigationIndex,
    duplicate_groups: Vec<InklingDuplicateGroup>,
    freshness: InklingFreshness,
    project_hubs: Vec<InklingProjectHub>,
    cleanup_suggestions: Vec<InklingCleanupSuggestion>,
    destructive_change: InklingDestructiveChange,
    local_boundary: InklingLocalBoundary,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingVault {
    vault_id: String,
    root_path: String,
    snapshot_id: String,
    snapshot_hash: String,
    visible_folders: Vec<String>,
    hidden_storage: Vec<String>,
    note_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingImport {
    import_id: String,
    source_kind: String,
    source_path: String,
    imported_note_count: usize,
    provenance_refs: Vec<String>,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingPermissions {
    permission_receipt_id: String,
    pdf_extraction_enabled: bool,
    image_ocr_enabled: bool,
    external_link_fetch_enabled: bool,
    destructive_edits_allowed: bool,
    policy_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingNavigationIndex {
    index_id: String,
    output_ref: String,
    generated_metadata_ref: String,
    graph_link_count: usize,
    inferred_tag_count: usize,
    attachment_text_extractions: usize,
    external_fetch_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingDuplicateGroup {
    group_id: String,
    note_ids: Vec<String>,
    evidence_refs: Vec<String>,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingFreshness {
    stale_candidate_count: usize,
    review_date: String,
    evidence_refs: Vec<String>,
    cleanup_hidden: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingProjectHub {
    hub_id: String,
    note_refs: Vec<String>,
    evidence_refs: Vec<String>,
    confidence_basis_points: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingCleanupSuggestion {
    suggestion_id: String,
    kind: String,
    target_refs: Vec<String>,
    evidence_refs: Vec<String>,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingDestructiveChange {
    snapshot_required: bool,
    user_approval_status: String,
    applied_changes_count: usize,
    approval_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InklingLocalBoundary {
    original_notes_mutated: bool,
    generated_metadata_separable: bool,
    network_fetches_attempted: bool,
    local_first_storage: bool,
    platform_memory_authority_claimed: bool,
}

fn inkling_vault_transcript() -> InklingVaultNavigationTranscript {
    serde_json::from_str(INKLING_VAULT_TRANSCRIPT)
        .expect("Inkling vault navigation transcript parses")
}
