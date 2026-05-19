//! Escrow Release — irreversible commitment marquee fixture.
//!
//! Where the round-driven marquee exercises ambiguous sensemaking (multi-stage
//! Formation design converging at round 2), this fixture exercises an
//! irreversible commitment: once funds leave escrow, they cannot be recalled.
//! The JTBD encodes the strict evidence and failure-mode obligations that the
//! verifier must enforce for a Satisfied verdict.
//!
//! v0.12 scope: prove the strict-verdict shape with a fixture before wiring
//! `/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow` as the live
//! runtime. The fixture must distinguish Satisfied, Blocked, and Invalid
//! without creating a second verifier path.

use axiom_truth::{
    ArtifactKind, AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, CalibrationStatus,
    CalibrationTable, ClauseId, ClauseInput, EvidenceRefRecord, JtbdClauseKind, JtbdInput,
    LearningEpisode, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, apply_decoder_calibration,
    calibration_records_from_learning_episode, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt::Write as _;

const TALLY_TRANSITION_SIGNATURE_TRUTH_KEY: &str = "transition-requires-signature";
const TALLY_RELEASE_CONDITIONS_TRUTH_KEY: &str = "release-requires-conditions-met";
const TALLY_RELEASE_ADAPTER_ID: &str = "tally-escrow.release-transcript-to-axiom-observation";
const TALLY_RELEASE_ADAPTER_VERSION: &str = "fixture.v0.1";
const TALLY_RELEASE_TRANSCRIPT: &str =
    include_str!("fixtures/tally_escrow_release_transcript.json");

fn escrow_release_jtbd() -> JtbdInput {
    JtbdInput {
        key: "escrow-release".to_string(),
        actor: "escrow operator".to_string(),
        functional_job: "release escrowed funds to the vendor after buyer authorization"
            .to_string(),
        so_that:
            "the buyer's payment obligation is settled and the vendor is paid for verified delivery"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "buyer_approval",
                "buyer authorization signed and on file with non-revoked status",
            ),
            ClauseInput::with_key(
                "delivery_confirmed",
                "vendor delivery confirmed by buyer attestation or trusted third-party signal",
            ),
            ClauseInput::with_key(
                "compliance_cleared",
                "policy gate cleared (sanctions screening current and KYC valid)",
            ),
            ClauseInput::with_key(
                "idempotency_key",
                "release request carries a unique idempotency key not previously promoted",
            ),
            ClauseInput::with_key(
                "disbursement_recorded",
                "disbursement transaction recorded with the payment rail and reconciled",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "double_release",
                "release proceeds despite a prior promotion of the same idempotency key",
            ),
            ClauseInput::with_key(
                "missing_buyer_approval",
                "release proceeds without an active buyer authorization on file",
            ),
            ClauseInput::with_key(
                "sanctioned_recipient",
                "release proceeds despite sanctions screening flagging the recipient",
            ),
            ClauseInput::with_key(
                "open_dispute",
                "release proceeds while the underlying transaction has an open dispute",
            ),
            ClauseInput::with_key(
                "unverified_delivery",
                "release proceeds without verified delivery evidence",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(15)),
    }
}

#[test]
fn escrow_release_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Release escrowed funds to the vendor")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        5
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        5
    );
    assert_eq!(package.proof_obligations.len(), 10);
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence.contains("idempotency key"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("sanctions screening"))
    );
    assert_eq!(package.artifacts.policy_requirements.len(), 11);
    assert!(
        package
            .artifacts
            .policy_requirements
            .iter()
            .any(|artifact| artifact
                .summary
                .contains("current Converge promotion policy"))
    );
    assert!(
        package
            .artifacts
            .policy_requirements
            .iter()
            .any(|artifact| artifact.summary.contains("buyer authorization signed"))
    );
    assert!(
        package
            .artifacts
            .policy_requirements
            .iter()
            .any(|artifact| artifact.summary.contains("sanctions screening flagging"))
    );
    assert_eq!(
        package.source_jtbd.time_budget,
        Some(TimeBudget::from_minutes(15))
    );
    assert!(
        package
            .generated_truths
            .contains("Expires: 2099-01-01T00:15:00Z")
    );
    assert_eq!(
        package.intent_packet.context["time_budget_seconds"],
        serde_json::json!(900)
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn escrow_release_satisfied_run_verifies_and_audits() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let observation = AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts: satisfied_release_facts(&package),
        integrity: RunIntegrityProof::sha256_merkle("sha256:escrow-release", 14, 5),
        replay_notes: vec![
            "buyer authorization validated against signing key registered 2026-05-12".to_string(),
            "idempotency check against last 10 minutes of disbursements found no match".to_string(),
        ],
        run_stages: Vec::new(),
    };

    let report = AxiomRunReport::verify(&package, observation);
    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());

    let audit = report
        .audit_fact_lineage(&package)
        .expect("every promoted fact must trace through the JTBD chain");
    assert_eq!(audit.evidence_coverage.len(), 5);
    assert_eq!(audit.failure_coverage.len(), 1);
    assert_eq!(audit.facts_audited, 5);
}

#[test]
fn escrow_release_blocked_when_human_gate_is_pending() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let delivery_confirmed = evidence_clause_id(&package, "delivery_confirmed");
    let compliance_cleared = evidence_clause_id(&package, "compliance_cleared");
    let idempotency_key = evidence_clause_id(&package, "idempotency_key");
    let double_release = failure_clause_id(&package, "double_release");

    let observation = AxiomRunObservation {
        stop_reason: ObservedStopReason::HitlGatePending {
            gate_id: "buyer-authorization".to_string(),
            proposal_id: "release-request-9f3a".to_string(),
            summary: "buyer authorization must be confirmed before funds leave escrow".to_string(),
            agent_id: "escrow-policy-gate".to_string(),
            cycle: 2,
        },
        promoted_facts: vec![
            fact(
                "Evidence",
                "delivery-attestation-vendor-7",
                "delivery attested by buyer; tracking number matches purchase order",
                vec![delivery_confirmed],
            ),
            fact(
                "PolicyDecision",
                "policy-gate-pass-2026-05-19",
                "policy gate cleared: sanctions screening current and KYC valid",
                vec![compliance_cleared],
            ),
            fact(
                "Diagnostic",
                "idempotency-check-pass-key-9f3a",
                "idempotency key 9f3a confirmed unique against prior promotions; double-release guard satisfied",
                vec![idempotency_key, double_release],
            ),
        ],
        integrity: RunIntegrityProof::sha256_merkle("sha256:escrow-blocked", 8, 3),
        replay_notes: vec![
            "release was not promoted while buyer authorization was pending".to_string(),
        ],
        run_stages: Vec::new(),
    };

    let report = AxiomRunReport::verify(&package, observation);
    assert_eq!(report.verdict, AxiomRunVerdict::Blocked);
    assert!(!report.expected_stop_reason_matched());

    let audit = report
        .audit_fact_lineage(&package)
        .expect("blocked runs still audit the facts they promoted");
    assert_eq!(audit.evidence_coverage.len(), 3);
    assert_eq!(audit.failure_coverage.len(), 1);
    assert_eq!(audit.facts_audited, 3);
}

#[test]
fn escrow_release_invalid_when_forbidden_release_condition_is_promoted() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let sanctioned_recipient = failure_clause_id(&package, "sanctioned_recipient");
    let mut promoted_facts = satisfied_release_facts(&package);
    promoted_facts.push(fact(
        "PolicyDecision",
        "sanctions-screening-flag-vendor-7",
        "release proceeds despite sanctions screening flagging the recipient",
        vec![sanctioned_recipient],
    ));

    let observation = AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:escrow-invalid", 15, 6),
        replay_notes: vec![
            "release attempt crossed the forbidden sanctioned-recipient condition".to_string(),
        ],
        run_stages: Vec::new(),
    };

    let report = AxiomRunReport::verify(&package, observation);
    assert_eq!(report.verdict, AxiomRunVerdict::Invalid);
    assert!(report.expected_stop_reason_matched());

    let audit = report
        .audit_fact_lineage(&package)
        .expect("invalid runs still preserve clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 5);
    assert_eq!(audit.failure_coverage.len(), 2);
    assert_eq!(audit.facts_audited, 6);
}

#[test]
fn tally_release_outcome_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();

    let observation =
        adapt_tally_release_transcript(&package, &transcript).expect("Tally release adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.tally-release")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Tally-adapted release preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 5);
    assert_eq!(audit.failure_coverage.len(), 1);
    assert_eq!(audit.facts_audited, 5);
}

#[test]
fn tally_release_outcome_missing_release_truth_key_is_invalid() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let mut transcript = tally_release_transcript();
    transcript
        .release
        .truth_keys
        .retain(|truth_key| truth_key != TALLY_RELEASE_CONDITIONS_TRUTH_KEY);

    let observation =
        adapt_tally_release_transcript(&package, &transcript).expect("Tally release adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Invalid);
    assert!(report.expected_stop_reason_matched());
}

#[test]
fn tally_release_adapter_emits_success_receipt() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();

    let first = adapt_tally_release_transcript_with_receipt(&package, &transcript);
    let second = adapt_tally_release_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, TALLY_RELEASE_ADAPTER_ID);
    assert_eq!(first.receipt.adapter_version, TALLY_RELEASE_ADAPTER_VERSION);
    assert_eq!(first.receipt.source_app, "tally-escrow");
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
            "tally.release.principal-signatures",
            "tally.release.conditions-met",
            "tally.release.current-policy",
            "tally.release.idempotency",
            "tally.release.custody-receipt",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 6);
    assert!(first.receipt.dropped_source_fields.is_empty());
    assert!(first.receipt.warnings.is_empty());
    assert!(first.receipt.errors.is_empty());
}

#[test]
fn tally_release_adapter_emits_rejection_receipt_without_observation() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let mut transcript = tally_release_transcript();
    transcript.release.to_state = "Verified".to_string();

    let result = adapt_tally_release_transcript_with_receipt(&package, &transcript);

    assert!(result.observation.is_none());
    assert_eq!(result.receipt.status, ObservationAdapterStatus::Rejected);
    assert_eq!(result.receipt.observation_hash, None);
    assert!(result.receipt.mapped_fact_ids.is_empty());
    assert!(result.receipt.mapped_clause_ids.is_empty());
    assert_eq!(
        result.receipt.errors,
        vec!["expected Tally transition Verified -> Released"]
    );
    assert!(
        result
            .receipt
            .receipt_id
            .as_str()
            .starts_with("observation_adapter_receipt.")
    );
}

#[test]
fn helm_release_readiness_packet_marks_satisfied_release_ready_to_review() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();
    let adapter_outcome = adapt_tally_release_transcript_with_receipt(&package, &transcript);

    let packet = release_readiness_packet(&package, &transcript, &adapter_outcome);

    assert_eq!(packet.package_id, package.package_id.as_str());
    assert_eq!(packet.truth_version, package.truth_version);
    assert_eq!(packet.domain_hint, "tally-escrow.release");
    assert_eq!(packet.target_transition, "Verified -> Released");
    assert_eq!(
        packet.adapter_receipt_id,
        adapter_outcome.receipt.receipt_id.as_str()
    );
    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Satisfied));
    assert!(!packet.authorizes_transition);
    assert_eq!(packet.evidence_status.len(), 5);
    assert!(
        packet
            .evidence_status
            .iter()
            .all(|status| status.status == EvidenceReadinessStatus::Present)
    );
    assert!(
        packet
            .operator_actions
            .contains(&"inspect axiom report".to_string())
    );
    assert_eq!(
        packet.verifier_forbidden_actions.len(),
        package.verifier_spec.forbidden_actions.len()
    );
}

#[test]
fn helm_release_readiness_packet_marks_missing_condition_evidence() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let mut transcript = tally_release_transcript();
    transcript
        .release
        .truth_keys
        .retain(|truth_key| truth_key != TALLY_RELEASE_CONDITIONS_TRUTH_KEY);
    let adapter_outcome = adapt_tally_release_transcript_with_receipt(&package, &transcript);

    let packet = release_readiness_packet(&package, &transcript, &adapter_outcome);
    let delivery_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "delivery_confirmed")
        .expect("delivery evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_transition);
    assert!(
        delivery_status
            .clause_id
            .ends_with(".evidence.delivery_confirmed")
    );
    assert!(delivery_status.label.contains("vendor delivery confirmed"));
    assert_eq!(delivery_status.status, EvidenceReadinessStatus::Missing);
    assert!(delivery_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for delivery_confirmed".to_string())
    );
}

#[test]
fn helm_operator_ledger_entries_are_deterministic_and_backlink_only() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();
    let adapter_outcome = adapt_tally_release_transcript_with_receipt(&package, &transcript);
    let packet = release_readiness_packet(&package, &transcript, &adapter_outcome);

    let first = release_readiness_ledger_entries(&adapter_outcome.receipt, &packet);
    let second = release_readiness_ledger_entries(&adapter_outcome.receipt, &packet);

    assert_eq!(first, second);
    assert_eq!(first.len(), 2);

    let adapter_entry = &first[0];
    assert_eq!(adapter_entry.sequence, 0);
    assert_eq!(
        adapter_entry.record_kind,
        HelmLedgerRecordKind::ObservationAdapterReceipt
    );
    assert_eq!(
        adapter_entry.source_ref,
        adapter_outcome.receipt.receipt_id.as_str()
    );
    assert_eq!(adapter_entry.package_id, package.package_id.as_str());
    assert_eq!(adapter_entry.truth_version, package.truth_version);
    assert_eq!(adapter_entry.domain_hint, transcript.source.domain_hint);
    assert!(adapter_entry.payload_hash.starts_with("sha256:"));
    assert!(adapter_entry.backlink_ids.is_empty());
    assert_eq!(
        adapter_entry.authority_effect,
        HelmLedgerAuthorityEffect::None
    );
    assert!(adapter_entry.summary.contains(TALLY_RELEASE_ADAPTER_ID));

    let readiness_entry = &first[1];
    assert_eq!(readiness_entry.sequence, 1);
    assert_eq!(
        readiness_entry.record_kind,
        HelmLedgerRecordKind::ReleaseReadinessPacket
    );
    assert_eq!(
        readiness_entry.source_ref,
        format!("helm://release-readiness/{}", packet.adapter_receipt_id)
    );
    assert_eq!(
        readiness_entry.backlink_ids,
        vec![adapter_outcome.receipt.receipt_id.as_str().to_string()]
    );
    assert_eq!(
        readiness_entry.authority_effect,
        HelmLedgerAuthorityEffect::None
    );
    assert!(readiness_entry.payload_hash.starts_with("sha256:"));
    assert!(readiness_entry.summary.contains("Verified -> Released"));

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("organism:sig"));
    assert!(!serialized.contains("cargo test"));
    assert!(!serialized.contains("attestation-namecheap-release-agreement-7"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow"));
}

#[test]
fn tally_release_adapter_rejects_non_release_transition() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let mut transcript = tally_release_transcript();
    transcript.release.to_state = "Verified".to_string();

    let err = adapt_tally_release_transcript(&package, &transcript)
        .expect_err("non-release transition is not an escrow release observation");

    assert_eq!(err, "expected Tally transition Verified -> Released");
}

#[test]
fn tally_release_report_has_learning_episode_feedstock() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();
    let observation =
        adapt_tally_release_transcript(&package, &transcript).expect("Tally release adapts");
    let report = AxiomRunReport::verify(&package, observation);
    let audit = report
        .audit_fact_lineage(&package)
        .expect("Tally release report audits");

    let episode = LearningEpisode::from_report(
        &transcript.source.run_id,
        &transcript.source.domain_hint,
        &package,
        &report,
        &audit,
    );

    assert_eq!(episode.source_run_id, transcript.source.run_id);
    assert_eq!(episode.domain_hint, "tally-escrow.release");
    assert_eq!(episode.verdict, AxiomRunVerdict::Satisfied);
    assert_eq!(episode.source_clause_signals.len(), 13);
    assert_eq!(
        episode
            .source_clause_signals
            .iter()
            .filter(|signal| signal.coverage_status.was_covered_as_evidence())
            .count(),
        5
    );
    assert_eq!(
        episode
            .source_clause_signals
            .iter()
            .filter(|signal| signal.coverage_status.was_covered_as_failure_guard())
            .count(),
        1
    );
    assert_eq!(episode.promoted_fact_ids.len(), 5);
    assert_eq!(
        episode.promotion_policy_hashes,
        vec!["sha256:tally-release-policy".to_string()]
    );
    assert!(episode.observed_stop_reason.contains("Converged"));
    assert_eq!(
        episode.verifier_required_evidence.len(),
        package.verifier_spec.required_evidence.len()
    );
    assert_eq!(
        episode.verifier_forbidden_actions.len(),
        package.verifier_spec.forbidden_actions.len()
    );
}

#[test]
fn tally_release_learning_episode_proposes_calibration_records() {
    let (package, transcript, episode) = tally_release_learning_episode();

    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("learning episode produces calibration records");

    assert_eq!(records.len(), 6);
    assert!(records.iter().all(|record| {
        record.status == CalibrationStatus::Proposed
            && record.key.domain_hint == transcript.source.domain_hint
            && record.key.decoder_rule_id == "decoder_calibration.v0.13"
            && !record.key.normalized_clause_shape.is_empty()
    }));
    assert!(records.iter().any(|record| {
        record.key.clause_kind == JtbdClauseKind::EvidenceRequired
            && !record.value.suggested_evidence_templates.is_empty()
    }));
    assert!(records.iter().any(|record| {
        record.key.clause_kind == JtbdClauseKind::FailureMode
            && !record.value.suggested_failure_scenarios.is_empty()
    }));
}

#[test]
fn accepted_tally_calibration_enriches_regenerated_truth_package() {
    let (package, transcript, episode) = tally_release_learning_episode();
    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("learning episode produces calibration records")
        .into_iter()
        .map(|record| record.accepted("operator accepted Tally release decoder prior"))
        .collect();
    let table = CalibrationTable::new(records);
    let regenerated = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD regenerates");

    let enriched = apply_decoder_calibration(regenerated, &table, &transcript.source.domain_hint)
        .expect("accepted calibration enriches package");

    assert_eq!(enriched.artifacts.calibration_suggestions.len(), 6);
    assert!(
        enriched
            .artifacts
            .calibration_suggestions
            .iter()
            .all(|artifact| {
                artifact.artifact_kind == ArtifactKind::CalibrationSuggestion
                    && artifact.summary.contains("calibration_record.")
            })
    );
    assert!(
        enriched
            .lineage
            .validate_closure(&enriched.source_jtbd)
            .is_ok()
    );
    assert!(enriched.lineage.artifacts.iter().any(|lineage| {
        lineage.artifact_kind == ArtifactKind::CalibrationSuggestion
            && lineage.decoder_rule_id.starts_with("decoder_calibration.")
    }));
}

#[test]
fn unaccepted_tally_calibration_does_not_enrich_package() {
    let (package, transcript, episode) = tally_release_learning_episode();
    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("learning episode produces calibration records");
    let table = CalibrationTable::new(
        records
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                if index % 2 == 0 {
                    record.with_status(CalibrationStatus::Rejected, "operator rejected prior")
                } else {
                    record.with_status(CalibrationStatus::Reset, "operator reset stale prior")
                }
            })
            .collect(),
    );
    let regenerated = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD regenerates");

    let enriched = apply_decoder_calibration(regenerated, &table, &transcript.source.domain_hint)
        .expect("unaccepted calibration remains a valid no-op");

    assert!(enriched.artifacts.calibration_suggestions.is_empty());
    assert!(
        enriched
            .lineage
            .artifacts
            .iter()
            .all(|lineage| lineage.artifact_kind != ArtifactKind::CalibrationSuggestion)
    );
}

#[test]
fn tally_calibration_does_not_capture_runtime_ownership_boundaries() {
    let (package, _, episode) = tally_release_learning_episode();
    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("learning episode produces calibration records");

    let serialized_records =
        serde_json::to_string(&records).expect("calibration records serialize");
    assert!(!serialized_records.contains("formation"));
    assert!(!serialized_records.contains("specialist"));
    assert!(!serialized_records.contains("approver_id"));
    assert!(!serialized_records.contains("gate_id"));
}

fn tally_release_learning_episode() -> (TruthPackage, TallyReleaseTranscript, LearningEpisode) {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let transcript = tally_release_transcript();
    let observation =
        adapt_tally_release_transcript(&package, &transcript).expect("Tally release adapts");
    let report = AxiomRunReport::verify(&package, observation);
    let audit = report
        .audit_fact_lineage(&package)
        .expect("Tally release report audits");
    let episode = LearningEpisode::from_report(
        &transcript.source.run_id,
        &transcript.source.domain_hint,
        &package,
        &report,
        &audit,
    );

    (package, transcript, episode)
}

fn satisfied_release_facts(package: &TruthPackage) -> Vec<PromotedFactRecord> {
    let buyer_approval = evidence_clause_id(package, "buyer_approval");
    let delivery_confirmed = evidence_clause_id(package, "delivery_confirmed");
    let compliance_cleared = evidence_clause_id(package, "compliance_cleared");
    let idempotency_key = evidence_clause_id(package, "idempotency_key");
    let disbursement_recorded = evidence_clause_id(package, "disbursement_recorded");
    let double_release = failure_clause_id(package, "double_release");

    vec![
        fact(
            "HumanApproval",
            "buyer-approval-signed-2026-05-19",
            "buyer authorization signed and validated against the active key",
            vec![buyer_approval],
        ),
        fact(
            "Evidence",
            "delivery-attestation-vendor-7",
            "delivery attested by buyer; tracking number matches purchase order",
            vec![delivery_confirmed],
        ),
        fact(
            "PolicyDecision",
            "policy-gate-pass-2026-05-19",
            "policy gate cleared: sanctions screening current and KYC valid",
            vec![compliance_cleared],
        ),
        fact(
            "Diagnostic",
            "idempotency-check-pass-key-9f3a",
            "idempotency key 9f3a confirmed unique against prior promotions; double-release guard satisfied",
            vec![idempotency_key, double_release],
        ),
        fact(
            "Disbursement",
            "rail-disbursement-tx-2026-05-19",
            "payment rail confirmed disbursement; transaction recorded and reconciled",
            vec![disbursement_recorded],
        ),
    ]
}

fn adapt_tally_release_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &TallyReleaseTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_tally_release_transcript(package, transcript) {
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
    transcript: &TallyReleaseTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: TALLY_RELEASE_ADAPTER_ID.to_string(),
        adapter_version: TALLY_RELEASE_ADAPTER_VERSION.to_string(),
        status,
        source_app: "tally-escrow".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "tally://release/{}/{}",
            transcript.source.run_id, transcript.release.record_id
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
        replay_notes: vec![
            format!("source command {}", transcript.source.command),
            format!("captured at {}", transcript.source.captured_at),
        ],
    })
}

fn release_readiness_packet(
    package: &TruthPackage,
    transcript: &TallyReleaseTranscript,
    adapter_outcome: &ObservationAdapterOutcome,
) -> ReleaseReadinessPacket {
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

            ReleaseEvidenceStatus {
                clause_id: clause.id.to_string(),
                clause_key: clause.key.clone(),
                label: clause.text.clone(),
                status,
                fact_ids,
            }
        })
        .collect::<Vec<_>>();
    let operator_actions = release_readiness_operator_actions(
        adapter_outcome.receipt.status,
        &evidence_status,
        report.as_ref(),
    );

    ReleaseReadinessPacket {
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        target_transition: format!(
            "{} -> {}",
            transcript.release.from_state, transcript.release.to_state
        ),
        adapter_receipt_id: adapter_outcome.receipt.receipt_id.as_str().to_string(),
        adapter_status: adapter_outcome.receipt.status,
        verdict: report.as_ref().map(|report| report.verdict),
        authorizes_transition: false,
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

fn release_readiness_operator_actions(
    adapter_status: ObservationAdapterStatus,
    evidence_status: &[ReleaseEvidenceStatus],
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

fn release_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &ReleaseReadinessPacket,
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
            HelmLedgerRecordKind::ReleaseReadinessPacket,
            format!("helm://release-readiness/{}", packet.adapter_receipt_id),
            packet.package_id.clone(),
            packet.truth_version.clone(),
            packet.domain_hint.clone(),
            packet_payload_hash,
            vec![receipt.receipt_id.as_str().to_string()],
            format!(
                "release readiness {:?} for {}",
                packet.verdict, packet.target_transition
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

fn adapt_tally_release_transcript(
    package: &TruthPackage,
    transcript: &TallyReleaseTranscript,
) -> Result<AxiomRunObservation, String> {
    let outcome = &transcript.release;
    if outcome.from_state != "Verified" || outcome.to_state != "Released" {
        return Err("expected Tally transition Verified -> Released".to_string());
    }
    if outcome.reason != "ConditionsMet" {
        return Err("expected Tally ConditionsMet release reason".to_string());
    }

    let buyer_approval = evidence_clause_id(package, "buyer_approval");
    let delivery_confirmed = evidence_clause_id(package, "delivery_confirmed");
    let compliance_cleared = evidence_clause_id(package, "compliance_cleared");
    let idempotency_key = evidence_clause_id(package, "idempotency_key");
    let disbursement_recorded = evidence_clause_id(package, "disbursement_recorded");
    let double_release = failure_clause_id(package, "double_release");
    let mut promoted_facts = Vec::new();

    if outcome.signing_policy_satisfied
        && outcome
            .truth_keys
            .iter()
            .any(|truth_key| truth_key == TALLY_TRANSITION_SIGNATURE_TRUTH_KEY)
        && outcome.has_principal_signer("Transferor")
        && outcome.has_principal_signer("Acquirer")
    {
        promoted_facts.push(tally_fact(
            "HumanApproval",
            "tally.release.principal-signatures",
            "Organism signing witnesses cover both principals for the release transition",
            vec![buyer_approval],
            &outcome.promotion_authority,
        ));
    }

    if outcome
        .truth_keys
        .iter()
        .any(|truth_key| truth_key == TALLY_RELEASE_CONDITIONS_TRUTH_KEY)
    {
        promoted_facts.push(tally_fact(
            "Evidence",
            "tally.release.conditions-met",
            "Tally release transition carried the release conditions truth key",
            vec![delivery_confirmed],
            &outcome.promotion_authority,
        ));
    }

    promoted_facts.push(tally_fact(
        "PolicyDecision",
        "tally.release.current-policy",
        "Converge promotion gate observed the current release policy hash",
        vec![compliance_cleared],
        &outcome.promotion_authority,
    ));
    promoted_facts.push(tally_fact(
        "Diagnostic",
        "tally.release.idempotency",
        "Tally transition record id is unique for this release attempt",
        vec![idempotency_key, double_release],
        &outcome.promotion_authority,
    ));
    promoted_facts.push(tally_fact(
        "CustodyReceipt",
        "tally.release.custody-receipt",
        &format!(
            "custody release receipt recorded by {} with external ref {}",
            outcome.release_receipt.adapter, outcome.release_receipt.external_ref
        ),
        vec![disbursement_recorded],
        &outcome.promotion_authority,
    ));

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:tally-release", 12, 5),
        replay_notes: vec![
            format!(
                "adapted Tally transition {} into AxiomRunObservation",
                outcome.record_id
            ),
            format!(
                "source run {} captured at {} via {}",
                transcript.source.run_id, transcript.source.captured_at, transcript.source.command
            ),
            format!("source app path {}", transcript.source.app_path),
        ],
        run_stages: Vec::new(),
    })
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

#[derive(Debug, Clone)]
struct ObservationAdapterOutcome {
    observation: Option<AxiomRunObservation>,
    receipt: ObservationAdapterReceipt,
}

#[derive(Debug, Clone, Serialize)]
struct ReleaseReadinessPacket {
    package_id: String,
    truth_version: String,
    domain_hint: String,
    target_transition: String,
    adapter_receipt_id: String,
    adapter_status: ObservationAdapterStatus,
    verdict: Option<AxiomRunVerdict>,
    authorizes_transition: bool,
    evidence_status: Vec<ReleaseEvidenceStatus>,
    verifier_forbidden_actions: Vec<String>,
    operator_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReleaseEvidenceStatus {
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
    ReleaseReadinessPacket,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::ReleaseReadinessPacket => "release_readiness_packet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TallyReleaseTranscript {
    source: TallyRunSource,
    release: TallyReleaseOutcome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TallyRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TallyReleaseOutcome {
    record_id: String,
    from_state: String,
    to_state: String,
    reason: String,
    truth_keys: Vec<String>,
    signers: Vec<TallySigner>,
    signing_policy_satisfied: bool,
    release_receipt: TallyReleaseReceipt,
    promotion_authority: PromotionAuthorityRecord,
}

impl TallyReleaseOutcome {
    fn has_principal_signer(&self, role: &str) -> bool {
        self.signers
            .iter()
            .any(|signer| signer.role == role && !signer.signature_ref.trim().is_empty())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TallySigner {
    role: String,
    signature_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TallyReleaseReceipt {
    adapter: String,
    external_ref: String,
}

fn tally_release_transcript() -> TallyReleaseTranscript {
    serde_json::from_str(TALLY_RELEASE_TRANSCRIPT).expect("Tally release transcript parses")
}

fn fact(
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
            source: "escrow-release-fixture".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace.{fact_id}"),
            location: Some("fixture://escrow-release".to_string()),
            replayable: true,
        }),
        promotion_authority: None,
    }
}

fn tally_fact(
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
            evidence_id: format!("tally.evidence.{fact_id}"),
            source: "tally-release-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("tally.trace.{fact_id}"),
            location: Some("/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow".to_string()),
            replayable: true,
        }),
        promotion_authority: Some(authority.clone()),
    }
}
