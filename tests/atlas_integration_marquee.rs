//! Atlas Integration — second app probe for the Axiom/Helm contract.
//!
//! Tally pressure-tested irreversible escrow release. Atlas tests whether the
//! same contract surfaces survive a very different app: post-acquisition
//! identity/auth consolidation. The app owns repository and candidate meaning;
//! Axiom owns the normalized observation and verifier result; Helm owns the
//! operator-facing readiness and ledger shape.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const ATLAS_CANDIDATE_EVIDENCE_TRUTH_KEY: &str = "candidate-has-reviewable-evidence";
const ATLAS_OWNER_APPROVAL_TRUTH_KEY: &str = "owner-approval-before-writeback";
const ATLAS_BOUNDED_PROOF_TRUTH_KEY: &str = "bounded-proof-language";
const ATLAS_ADAPTER_ID: &str = "atlas-integration.identity-candidate-to-axiom-observation";
const ATLAS_ADAPTER_VERSION: &str = "fixture.v0.1";
const ATLAS_IDENTITY_CANDIDATE_TRANSCRIPT: &str =
    include_str!("fixtures/atlas_identity_candidate_transcript.json");

fn atlas_identity_consolidation_jtbd() -> JtbdInput {
    JtbdInput {
        key: "atlas-identity-consolidation".to_string(),
        actor: "integration lead".to_string(),
        functional_job:
            "select the first identity/auth consolidation candidate across acquired repositories"
                .to_string(),
        so_that:
            "the 100-day integration room can capture synergy without unsafe repository writeback"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "reviewable_evidence",
                "integration candidate cites concrete reviewable evidence for the overlap claim",
            ),
            ClauseInput::with_key(
                "repository_coverage",
                "candidate spans at least two known acquired repositories",
            ),
            ClauseInput::with_key(
                "similarity_basis",
                "similarity score names the basis and remains bounded as a candidate signal",
            ),
            ClauseInput::with_key(
                "owner_approval_state",
                "writeback proposal names repository owner approval state before any provider-side action",
            ),
            ClauseInput::with_key(
                "bounded_contract_check",
                "solver or bounded check evidence names the encoded boundary and excluded behavior",
            ),
            ClauseInput::with_key(
                "rollback_plan",
                "migration sequence includes a rollback path before shared-service cutover",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "similarity_only",
                "candidate advances from similarity score alone without reviewable evidence",
            ),
            ClauseInput::with_key(
                "writeback_without_owner_approval",
                "adapter PR or repository writeback proceeds without named owner approval",
            ),
            ClauseInput::with_key(
                "universal_proof_claim",
                "bounded solver result is described as universal code equivalence",
            ),
            ClauseInput::with_key(
                "hidden_disagreement",
                "unresolved counterargument or owner disagreement is hidden from review",
            ),
            ClauseInput::with_key(
                "unsafe_cutover_without_rollback",
                "migration sequence cuts over identity traffic without rollback support",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(60)),
    }
}

#[test]
fn atlas_identity_consolidation_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Select the first identity/auth consolidation candidate")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        6
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        5
    );
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("bounded check evidence"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("repository writeback"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn atlas_candidate_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");
    let transcript = atlas_identity_candidate_transcript();

    let observation = adapt_atlas_identity_candidate_transcript(&package, &transcript)
        .expect("Atlas candidate adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.atlas-identity-candidate")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Atlas-adapted candidate preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 6);
    assert_eq!(audit.failure_coverage.len(), 3);
    assert_eq!(audit.facts_audited, 6);
}

#[test]
fn atlas_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");
    let transcript = atlas_identity_candidate_transcript();

    let first = adapt_atlas_identity_candidate_transcript_with_receipt(&package, &transcript);
    let second = adapt_atlas_identity_candidate_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, ATLAS_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "atlas-integration");
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
            "atlas.identity.reviewable-evidence",
            "atlas.identity.repository-coverage",
            "atlas.identity.similarity-basis",
            "atlas.identity.owner-gate",
            "atlas.identity.bounded-check",
            "atlas.identity.rollback-plan",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 9);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("commercial-access/auth/jwt.py"));
    assert!(!serialized.contains("hospitality-suite/auth/oidc.test.ts"));
    assert!(!serialized.contains("cargo test"));
}

#[test]
fn atlas_readiness_packet_marks_missing_bounded_proof() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");
    let mut transcript = atlas_identity_candidate_transcript();
    transcript
        .candidate
        .truth_keys
        .retain(|truth_key| truth_key != ATLAS_BOUNDED_PROOF_TRUTH_KEY);
    let adapter_outcome =
        adapt_atlas_identity_candidate_transcript_with_receipt(&package, &transcript);

    let packet = integration_readiness_packet(&package, &transcript, &adapter_outcome);
    let bounded_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "bounded_contract_check")
        .expect("bounded proof evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_writeback);
    assert_eq!(bounded_status.status, EvidenceReadinessStatus::Missing);
    assert!(bounded_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for bounded_contract_check".to_string())
    );
}

#[test]
fn atlas_readiness_packet_marks_missing_owner_approval_before_writeback() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");
    let mut transcript = atlas_identity_candidate_transcript();
    transcript
        .candidate
        .truth_keys
        .retain(|truth_key| truth_key != ATLAS_OWNER_APPROVAL_TRUTH_KEY);
    let adapter_outcome =
        adapt_atlas_identity_candidate_transcript_with_receipt(&package, &transcript);

    let packet = integration_readiness_packet(&package, &transcript, &adapter_outcome);
    let owner_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "owner_approval_state")
        .expect("owner approval evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_writeback);
    assert_eq!(owner_status.status, EvidenceReadinessStatus::Missing);
    assert!(owner_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for owner_approval_state".to_string())
    );
}

#[test]
fn atlas_operator_ledger_entries_are_deterministic_backlinks_without_writeback_authority() {
    let package = decode_jtbd(atlas_identity_consolidation_jtbd()).expect("Atlas JTBD decodes");
    let transcript = atlas_identity_candidate_transcript();
    let adapter_outcome =
        adapt_atlas_identity_candidate_transcript_with_receipt(&package, &transcript);
    let packet = integration_readiness_packet(&package, &transcript, &adapter_outcome);

    let first = integration_readiness_ledger_entries(&adapter_outcome.receipt, &packet);
    let second = integration_readiness_ledger_entries(&adapter_outcome.receipt, &packet);

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        first[0].record_kind,
        HelmLedgerRecordKind::ObservationAdapterReceipt
    );
    assert_eq!(
        first[1].record_kind,
        HelmLedgerRecordKind::IntegrationReadinessPacket
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
    assert!(!serialized.contains("fixture://"));
    assert!(!serialized.contains("cargo test"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/atlas-integration"));
    assert!(!serialized.contains("shared-identity-core-shadow-traffic"));
}

fn adapt_atlas_identity_candidate_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &AtlasCandidateTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_atlas_identity_candidate_transcript(package, transcript) {
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
    transcript: &AtlasCandidateTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: ATLAS_ADAPTER_ID.to_string(),
        adapter_version: ATLAS_ADAPTER_VERSION.to_string(),
        status,
        source_app: "atlas-integration".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "atlas://candidate/{}/{}",
            transcript.source.run_id, transcript.candidate.candidate_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: Vec::new(),
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn integration_readiness_packet(
    package: &TruthPackage,
    transcript: &AtlasCandidateTranscript,
    adapter_outcome: &ObservationAdapterOutcome,
) -> IntegrationReadinessPacket {
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

            IntegrationEvidenceStatus {
                clause_id: clause.id.to_string(),
                clause_key: clause.key.clone(),
                label: clause.text.clone(),
                status,
                fact_ids,
            }
        })
        .collect::<Vec<_>>();
    let operator_actions = integration_readiness_operator_actions(
        adapter_outcome.receipt.status,
        &evidence_status,
        report.as_ref(),
    );

    IntegrationReadinessPacket {
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        candidate_id: transcript.candidate.candidate_id.clone(),
        target_capability: transcript.candidate.capability.clone(),
        adapter_receipt_id: adapter_outcome.receipt.receipt_id.as_str().to_string(),
        adapter_status: adapter_outcome.receipt.status,
        verdict: report.as_ref().map(|report| report.verdict),
        authorizes_writeback: false,
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

fn integration_readiness_operator_actions(
    adapter_status: ObservationAdapterStatus,
    evidence_status: &[IntegrationEvidenceStatus],
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
    actions
}

fn integration_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &IntegrationReadinessPacket,
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
            HelmLedgerRecordKind::IntegrationReadinessPacket,
            format!("helm://integration-readiness/{}", packet.adapter_receipt_id),
            packet.package_id.clone(),
            packet.truth_version.clone(),
            packet.domain_hint.clone(),
            packet_payload_hash,
            vec![receipt.receipt_id.as_str().to_string()],
            format!(
                "integration readiness {:?} for {}",
                packet.verdict, packet.target_capability
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

fn adapt_atlas_identity_candidate_transcript(
    package: &TruthPackage,
    transcript: &AtlasCandidateTranscript,
) -> Result<AxiomRunObservation, String> {
    let candidate = &transcript.candidate;
    if candidate.phase != "EvidenceOnly" {
        return Err("expected Atlas candidate to remain EvidenceOnly".to_string());
    }
    if candidate.repositories.len() < 2 {
        return Err("expected Atlas candidate to cover at least two repositories".to_string());
    }
    if !candidate
        .truth_keys
        .iter()
        .any(|truth_key| truth_key == ATLAS_CANDIDATE_EVIDENCE_TRUTH_KEY)
    {
        return Err("expected Atlas candidate evidence truth key".to_string());
    }

    let reviewable_evidence = evidence_clause_id(package, "reviewable_evidence");
    let repository_coverage = evidence_clause_id(package, "repository_coverage");
    let similarity_basis = evidence_clause_id(package, "similarity_basis");
    let owner_approval_state = evidence_clause_id(package, "owner_approval_state");
    let bounded_contract_check = evidence_clause_id(package, "bounded_contract_check");
    let rollback_plan = evidence_clause_id(package, "rollback_plan");
    let writeback_without_owner_approval =
        failure_clause_id(package, "writeback_without_owner_approval");
    let universal_proof_claim = failure_clause_id(package, "universal_proof_claim");
    let unsafe_cutover_without_rollback =
        failure_clause_id(package, "unsafe_cutover_without_rollback");
    let mut promoted_facts = Vec::new();

    promoted_facts.push(atlas_fact(
        "Evidence",
        "atlas.identity.reviewable-evidence",
        "candidate overlap claim cites AST, contract-test, owner-attestation, bounded-check, and migration-plan evidence",
        vec![reviewable_evidence],
        &candidate.promotion_authority,
    ));
    promoted_facts.push(atlas_fact(
        "RepositoryCoverage",
        "atlas.identity.repository-coverage",
        "candidate covers two acquired repositories for the identity.jwt capability",
        vec![repository_coverage],
        &candidate.promotion_authority,
    ));
    promoted_facts.push(atlas_fact(
        "SimilarityScore",
        "atlas.identity.similarity-basis",
        &format!(
            "Prism-style similarity is {} basis points and remains a candidate signal",
            candidate.similarity_basis_points
        ),
        vec![similarity_basis],
        &candidate.promotion_authority,
    ));

    if candidate
        .truth_keys
        .iter()
        .any(|truth_key| truth_key == ATLAS_OWNER_APPROVAL_TRUTH_KEY)
    {
        promoted_facts.push(atlas_fact(
            "OwnerGate",
            "atlas.identity.owner-gate",
            "owner approval state is recorded; provider-side writeback remains blocked pending named owner approval",
            vec![owner_approval_state, writeback_without_owner_approval],
            &candidate.promotion_authority,
        ));
    }

    if candidate
        .truth_keys
        .iter()
        .any(|truth_key| truth_key == ATLAS_BOUNDED_PROOF_TRUTH_KEY)
    {
        promoted_facts.push(atlas_fact(
            "BoundedCheck",
            "atlas.identity.bounded-check",
            "bounded check names issuer, expiry, subject, missing-signature scope and excluded token-refresh behavior",
            vec![bounded_contract_check, universal_proof_claim],
            &candidate.promotion_authority,
        ));
    }

    promoted_facts.push(atlas_fact(
        "MigrationPlan",
        "atlas.identity.rollback-plan",
        "migration sequence starts with adapter draft and shadow traffic before rollback-capable cutover",
        vec![rollback_plan, unsafe_cutover_without_rollback],
        &candidate.promotion_authority,
    ));

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:atlas-identity-candidate", 18, 6),
        replay_notes: vec![
            format!(
                "adapted Atlas candidate {} into AxiomRunObservation",
                candidate.candidate_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn atlas_fact(
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
            evidence_id: format!("atlas.evidence.{fact_id}"),
            source: "atlas-identity-candidate-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("atlas.trace.{fact_id}"),
            location: Some("atlas://identity-candidate".to_string()),
            replayable: true,
        }),
        promotion_authority: Some(authority.clone()),
    }
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
struct IntegrationReadinessPacket {
    package_id: String,
    truth_version: String,
    domain_hint: String,
    candidate_id: String,
    target_capability: String,
    adapter_receipt_id: String,
    adapter_status: ObservationAdapterStatus,
    verdict: Option<AxiomRunVerdict>,
    authorizes_writeback: bool,
    evidence_status: Vec<IntegrationEvidenceStatus>,
    verifier_forbidden_actions: Vec<String>,
    operator_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct IntegrationEvidenceStatus {
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
    IntegrationReadinessPacket,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::IntegrationReadinessPacket => "integration_readiness_packet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AtlasCandidateTranscript {
    source: AtlasRunSource,
    candidate: AtlasCandidateOutcome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AtlasRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AtlasCandidateOutcome {
    candidate_id: String,
    capability: String,
    repositories: Vec<String>,
    proposed_shared_service: String,
    similarity_basis_points: u16,
    risk_band: String,
    phase: String,
    truth_keys: Vec<String>,
    evidence: Vec<AtlasEvidenceRef>,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AtlasEvidenceRef {
    kind: String,
    source: String,
    summary: String,
}

fn atlas_identity_candidate_transcript() -> AtlasCandidateTranscript {
    serde_json::from_str(ATLAS_IDENTITY_CANDIDATE_TRANSCRIPT)
        .expect("Atlas identity candidate transcript parses")
}
