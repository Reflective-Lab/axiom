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
    LearningEpisode, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, apply_decoder_calibration,
    calibration_records_from_learning_episode, decode_jtbd,
};
use serde::Deserialize;

const TALLY_TRANSITION_SIGNATURE_TRUTH_KEY: &str = "transition-requires-signature";
const TALLY_RELEASE_CONDITIONS_TRUTH_KEY: &str = "release-requires-conditions-met";
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
            .filter(|signal| signal.covered_as_evidence)
            .count(),
        5
    );
    assert_eq!(
        episode
            .source_clause_signals
            .iter()
            .filter(|signal| signal.covered_as_failure_guard)
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

#[derive(Debug, Clone, Deserialize)]
struct TallyReleaseTranscript {
    source: TallyRunSource,
    release: TallyReleaseOutcome,
}

#[derive(Debug, Clone, Deserialize)]
struct TallyRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
struct TallySigner {
    role: String,
    signature_ref: String,
}

#[derive(Debug, Clone, Deserialize)]
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
