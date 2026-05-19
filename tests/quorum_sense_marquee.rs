//! Quorum Sense — third app probe for the Axiom/Helm contract.
//!
//! Tally proved irreversible release, Atlas proved integration candidate
//! review, and Quorum pressures the same contract against organizational
//! sensemaking. The app owns inquiry state and synthesis semantics; Axiom owns
//! the truth package and verifier result; Helm owns the operator-facing job
//! readiness packet and append-only ledger records.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const QUORUM_SIGNAL_CONTENT_TRUTH_KEY: &str = "signal-requires-content";
const QUORUM_SIGNAL_CONSENT_TRUTH_KEY: &str = "signal-requires-consent";
const QUORUM_HYPOTHESIS_SIGNAL_TRUTH_KEY: &str = "hypothesis-requires-signal";
const QUORUM_PROBE_HYPOTHESIS_TRUTH_KEY: &str = "probe-cites-hypothesis";
const QUORUM_THRESHOLD_TRUTH_KEY: &str = "quorum-requires-explicit-threshold";
const QUORUM_DISSENT_TRUTH_KEY: &str = "minority-hypotheses-remain-visible";
const QUORUM_OPERATOR_APPROVAL_TRUTH_KEY: &str = "operator-approval-before-synthesis-action";
const QUORUM_ADAPTER_ID: &str = "quorum-sense.release-readiness-to-axiom-observation";
const QUORUM_ADAPTER_VERSION: &str = "fixture.v0.1";
const QUORUM_RELEASE_READINESS_TRANSCRIPT: &str =
    include_str!("fixtures/quorum_release_readiness_transcript.json");

fn quorum_release_readiness_jtbd() -> JtbdInput {
    JtbdInput {
        key: "quorum-release-readiness".to_string(),
        actor: "transformation operator".to_string(),
        functional_job:
            "decide whether an organizational inquiry has enough diverse evidence to present a synthesis for action"
                .to_string(),
        so_that:
            "leaders can act on emerging change signals without collapsing minority hypotheses or violating participant boundaries"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "active_consent",
                "every promoted signal has non-blank content and active participant consent",
            ),
            ClauseInput::with_key(
                "cited_hypotheses",
                "every promoted hypothesis cites at least one supporting participant signal",
            ),
            ClauseInput::with_key(
                "traceable_probes",
                "every adaptive probe cites the hypothesis it tests or deepens",
            ),
            ClauseInput::with_key(
                "explicit_threshold",
                "quorum readiness names an explicit evidence threshold before declaring readiness",
            ),
            ClauseInput::with_key(
                "role_coverage",
                "readiness evidence spans enough distinct participant roles to avoid a single-corner consensus",
            ),
            ClauseInput::with_key(
                "dissent_preserved",
                "minority hypotheses above the dissent threshold remain visible in the synthesis",
            ),
            ClauseInput::with_key(
                "operator_approval_state",
                "operator approval is recorded before synthesis is used for organizational action",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "signal_without_consent",
                "participant signal is promoted without active consent or non-blank content",
            ),
            ClauseInput::with_key(
                "hypothesis_without_signal",
                "hypothesis is promoted without supporting signal citations",
            ),
            ClauseInput::with_key(
                "probe_without_hypothesis",
                "adaptive probe is generated without citing an existing hypothesis",
            ),
            ClauseInput::with_key(
                "scalar_quorum_only",
                "quorum is declared from confidence mass alone without threshold and role coverage",
            ),
            ClauseInput::with_key(
                "suppressed_minority",
                "minority hypothesis above the dissent threshold is hidden from synthesis review",
            ),
            ClauseInput::with_key(
                "action_without_operator_approval",
                "organizational action proceeds from synthesis without operator approval",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(90)),
    }
}

#[test]
fn quorum_release_readiness_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Decide whether an organizational inquiry")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        7
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
            .any(|evidence| evidence.contains("minority hypotheses"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("without operator approval"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn quorum_inquiry_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");
    let transcript = quorum_release_readiness_transcript();

    let observation =
        adapt_quorum_release_readiness_transcript(&package, &transcript).expect("Quorum adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority.as_ref().is_some_and(|authority| {
            authority.gate_id == "converge.gate.quorum-synthesis-readiness"
        })
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Quorum-adapted synthesis preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 7);
    assert_eq!(audit.failure_coverage.len(), 6);
    assert_eq!(audit.facts_audited, 7);
}

#[test]
fn quorum_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");
    let transcript = quorum_release_readiness_transcript();

    let first = adapt_quorum_release_readiness_transcript_with_receipt(&package, &transcript);
    let second = adapt_quorum_release_readiness_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, QUORUM_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "quorum-sense");
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
            "quorum.readiness.active-consent",
            "quorum.readiness.cited-hypotheses",
            "quorum.readiness.traceable-probes",
            "quorum.readiness.explicit-threshold",
            "quorum.readiness.role-coverage",
            "quorum.readiness.dissent-preserved",
            "quorum.readiness.operator-approval",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 13);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("Decision ownership keeps changing"));
    assert!(!serialized.contains("embassy://interview/northstar/exec-001"));
    assert!(!serialized.contains("cargo test"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/quorum-sense"));
}

#[test]
fn quorum_job_readiness_packet_marks_missing_dissent_preservation() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");
    let mut transcript = quorum_release_readiness_transcript();
    transcript
        .inquiry
        .truth_keys
        .retain(|truth_key| truth_key != QUORUM_DISSENT_TRUTH_KEY);
    let adapter_outcome =
        adapt_quorum_release_readiness_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let dissent_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "dissent_preserved")
        .expect("dissent evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(dissent_status.status, EvidenceReadinessStatus::Missing);
    assert!(dissent_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for dissent_preserved".to_string())
    );
}

#[test]
fn quorum_job_readiness_packet_marks_missing_operator_approval() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");
    let mut transcript = quorum_release_readiness_transcript();
    transcript
        .inquiry
        .truth_keys
        .retain(|truth_key| truth_key != QUORUM_OPERATOR_APPROVAL_TRUTH_KEY);
    transcript.inquiry.synthesis.operator_approval.status = "Pending".to_string();
    let adapter_outcome =
        adapt_quorum_release_readiness_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "operator_approval_state")
        .expect("operator approval evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert!(approval_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for operator_approval_state".to_string())
    );
}

#[test]
fn quorum_operator_ledger_entries_are_deterministic_backlinks_without_action_authority() {
    let package = decode_jtbd(quorum_release_readiness_jtbd()).expect("Quorum JTBD decodes");
    let transcript = quorum_release_readiness_transcript();
    let adapter_outcome =
        adapt_quorum_release_readiness_transcript_with_receipt(&package, &transcript);
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
    assert!(!serialized.contains("Decision ownership keeps changing"));
    assert!(!serialized.contains("embassy://interview/northstar/exec-001"));
    assert!(!serialized.contains("helm.approval.quorum.northstar.20260519"));
    assert!(!serialized.contains("cargo test"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/quorum-sense"));
}

fn adapt_quorum_release_readiness_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &QuorumReleaseReadinessTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_quorum_release_readiness_transcript(package, transcript) {
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
    transcript: &QuorumReleaseReadinessTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: QUORUM_ADAPTER_ID.to_string(),
        adapter_version: QUORUM_ADAPTER_VERSION.to_string(),
        status,
        source_app: "quorum-sense".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "quorum://inquiry/{}/{}",
            transcript.source.run_id, transcript.inquiry.inquiry_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "signals.content".to_string(),
            "signals.source_ref".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &QuorumReleaseReadinessTranscript,
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
        subject_ref: format!("quorum://inquiry/{}", transcript.inquiry.inquiry_id),
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
    actions.push("route synthesis through Helm HITL before organizational action".to_string());
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

fn adapt_quorum_release_readiness_transcript(
    package: &TruthPackage,
    transcript: &QuorumReleaseReadinessTranscript,
) -> Result<AxiomRunObservation, String> {
    let inquiry = &transcript.inquiry;
    if inquiry.status != "QuorumReached" {
        return Err("expected Quorum inquiry to report QuorumReached".to_string());
    }
    if inquiry.signals.is_empty() {
        return Err("expected Quorum inquiry to carry participant signals".to_string());
    }
    if inquiry.hypotheses.is_empty() {
        return Err("expected Quorum inquiry to carry hypotheses".to_string());
    }

    let active_consent = evidence_clause_id(package, "active_consent");
    let cited_hypotheses = evidence_clause_id(package, "cited_hypotheses");
    let traceable_probes = evidence_clause_id(package, "traceable_probes");
    let explicit_threshold = evidence_clause_id(package, "explicit_threshold");
    let role_coverage = evidence_clause_id(package, "role_coverage");
    let dissent_preserved = evidence_clause_id(package, "dissent_preserved");
    let operator_approval_state = evidence_clause_id(package, "operator_approval_state");
    let signal_without_consent = failure_clause_id(package, "signal_without_consent");
    let hypothesis_without_signal = failure_clause_id(package, "hypothesis_without_signal");
    let probe_without_hypothesis = failure_clause_id(package, "probe_without_hypothesis");
    let scalar_quorum_only = failure_clause_id(package, "scalar_quorum_only");
    let suppressed_minority = failure_clause_id(package, "suppressed_minority");
    let action_without_operator_approval =
        failure_clause_id(package, "action_without_operator_approval");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&inquiry.truth_keys, QUORUM_SIGNAL_CONTENT_TRUTH_KEY)
        && has_truth_key(&inquiry.truth_keys, QUORUM_SIGNAL_CONSENT_TRUTH_KEY)
        && inquiry.signals.iter().all(|signal| {
            !signal.content.trim().is_empty() && !signal.consent_ref.trim().is_empty()
        })
    {
        promoted_facts.push(quorum_fact(
            "Signals",
            "quorum.readiness.active-consent",
            "all promoted signals have non-blank content and active participant consent refs",
            vec![active_consent, signal_without_consent],
            &inquiry.promotion_authority,
        ));
    }

    if has_truth_key(&inquiry.truth_keys, QUORUM_HYPOTHESIS_SIGNAL_TRUTH_KEY)
        && inquiry
            .hypotheses
            .iter()
            .all(|hypothesis| !hypothesis.supporting_signal_ids.is_empty())
    {
        promoted_facts.push(quorum_fact(
            "Hypotheses",
            "quorum.readiness.cited-hypotheses",
            "each promoted hypothesis cites supporting participant signals",
            vec![cited_hypotheses, hypothesis_without_signal],
            &inquiry.promotion_authority,
        ));
    }

    let hypothesis_ids = inquiry
        .hypotheses
        .iter()
        .map(|hypothesis| hypothesis.hypothesis_id.as_str())
        .collect::<BTreeSet<_>>();
    if has_truth_key(&inquiry.truth_keys, QUORUM_PROBE_HYPOTHESIS_TRUTH_KEY)
        && inquiry
            .probes
            .iter()
            .all(|probe| hypothesis_ids.contains(probe.targets_hypothesis_id.as_str()))
    {
        promoted_facts.push(quorum_fact(
            "Probes",
            "quorum.readiness.traceable-probes",
            "every adaptive probe cites an existing hypothesis before operator review",
            vec![traceable_probes, probe_without_hypothesis],
            &inquiry.promotion_authority,
        ));
    }

    if has_truth_key(&inquiry.truth_keys, QUORUM_THRESHOLD_TRUTH_KEY)
        && inquiry.threshold.minimum_signal_count > 0
        && inquiry.threshold.minimum_role_count > 0
        && inquiry.threshold.confidence_threshold_basis_points > 0
    {
        promoted_facts.push(quorum_fact(
            "QuorumThreshold",
            "quorum.readiness.explicit-threshold",
            "readiness declares signal count, role coverage, confidence, and dissent thresholds",
            vec![explicit_threshold, scalar_quorum_only.clone()],
            &inquiry.promotion_authority,
        ));
    }

    let covered_roles = inquiry
        .signals
        .iter()
        .map(|signal| signal.participant_role.as_str())
        .collect::<BTreeSet<_>>();
    if inquiry.signals.len() >= usize::from(inquiry.threshold.minimum_signal_count)
        && covered_roles.len() >= usize::from(inquiry.threshold.minimum_role_count)
    {
        promoted_facts.push(quorum_fact(
            "RoleCoverage",
            "quorum.readiness.role-coverage",
            "readiness evidence spans executive, middle-management, frontline, and customer roles",
            vec![role_coverage, scalar_quorum_only],
            &inquiry.promotion_authority,
        ));
    }

    if has_truth_key(&inquiry.truth_keys, QUORUM_DISSENT_TRUTH_KEY)
        && inquiry.hypotheses.iter().any(|hypothesis| {
            hypothesis.status == "minority"
                && hypothesis.visible
                && hypothesis.confidence_basis_points
                    >= inquiry.threshold.dissent_threshold_basis_points
                && inquiry
                    .synthesis
                    .minority_hypothesis_ids
                    .contains(&hypothesis.hypothesis_id)
        })
    {
        promoted_facts.push(quorum_fact(
            "Dissent",
            "quorum.readiness.dissent-preserved",
            "minority autonomy-deficit hypothesis remains visible above the dissent threshold",
            vec![dissent_preserved, suppressed_minority],
            &inquiry.promotion_authority,
        ));
    }

    if has_truth_key(&inquiry.truth_keys, QUORUM_OPERATOR_APPROVAL_TRUTH_KEY)
        && inquiry.synthesis.operator_approval.status == "Approved"
    {
        promoted_facts.push(quorum_fact(
            "OperatorApproval",
            "quorum.readiness.operator-approval",
            "operator approval is recorded before synthesis can inform organizational action",
            vec![operator_approval_state, action_without_operator_approval],
            &inquiry.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:quorum-release-readiness", 34, 7),
        replay_notes: vec![
            format!(
                "adapted Quorum inquiry {} into AxiomRunObservation",
                inquiry.inquiry_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn quorum_fact(
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
            evidence_id: format!("quorum.evidence.{fact_id}"),
            source: "quorum-release-readiness-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("quorum.trace.{fact_id}"),
            location: Some("quorum://release-readiness".to_string()),
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
struct QuorumReleaseReadinessTranscript {
    source: QuorumRunSource,
    inquiry: QuorumInquiryOutcome,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumInquiryOutcome {
    inquiry_id: String,
    core_question: String,
    status: String,
    truth_keys: Vec<String>,
    threshold: QuorumThresholdOutcome,
    signals: Vec<QuorumSignalOutcome>,
    hypotheses: Vec<QuorumHypothesisOutcome>,
    probes: Vec<QuorumProbeOutcome>,
    synthesis: QuorumSynthesisOutcome,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumThresholdOutcome {
    minimum_signal_count: u16,
    minimum_role_count: u16,
    confidence_threshold_basis_points: u16,
    dissent_threshold_basis_points: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumSignalOutcome {
    signal_id: String,
    participant_role: String,
    consent_ref: String,
    source_ref: String,
    content: String,
    themes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumHypothesisOutcome {
    hypothesis_id: String,
    statement: String,
    confidence_basis_points: u16,
    supporting_signal_ids: Vec<String>,
    status: String,
    visible: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumProbeOutcome {
    probe_id: String,
    targets_hypothesis_id: String,
    question: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumSynthesisOutcome {
    synthesis_id: String,
    statement: String,
    minority_hypothesis_ids: Vec<String>,
    operator_approval: QuorumOperatorApproval,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct QuorumOperatorApproval {
    approval_id: String,
    approver_id: String,
    status: String,
    scope: String,
}

fn quorum_release_readiness_transcript() -> QuorumReleaseReadinessTranscript {
    serde_json::from_str(QUORUM_RELEASE_READINESS_TRANSCRIPT)
        .expect("Quorum release readiness transcript parses")
}
