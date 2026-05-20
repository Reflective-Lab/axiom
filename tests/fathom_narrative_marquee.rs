//! Fathom Narrative - tenth app probe for the Axiom/Helm contract.
//!
//! Catalyst closed the everyday business-ops receipt loop. Fathom pressures a
//! different boundary: time-indexed analytical claims. A defensible narrative
//! result must name the corpus snapshot, comparable periods, filing sections,
//! query plan, disagreements, HITL escalation, and recommendation boundary.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const FATHOM_SNAPSHOT_TRUTH_KEY: &str = "corpus-snapshot-bound";
const FATHOM_WINDOW_TRUTH_KEY: &str = "comparison-window-named";
const FATHOM_SECTIONS_TRUTH_KEY: &str = "filing-sections-cited";
const FATHOM_QUERY_PLAN_TRUTH_KEY: &str = "query-plan-recorded";
const FATHOM_COUNT_DELTA_TRUTH_KEY: &str = "risk-count-delta-computed";
const FATHOM_LANGUAGE_DRIFT_TRUTH_KEY: &str = "language-drift-computed";
const FATHOM_COHORT_TRUTH_KEY: &str = "peer-cohort-boundary-named";
const FATHOM_DISAGREEMENT_TRUTH_KEY: &str = "disagreement-preserved";
const FATHOM_HITL_TRUTH_KEY: &str = "hitl-escalation-recorded";
const FATHOM_PORTFOLIO_TRUTH_KEY: &str = "portfolio-selection-recorded";
const FATHOM_CLAIMS_TRUTH_KEY: &str = "narrative-claims-cited";
const FATHOM_BOUNDARY_TRUTH_KEY: &str = "recommendation-boundary-declared";
const FATHOM_ADAPTER_ID: &str = "fathom-narrative.temporal-evidence-to-axiom-observation";
const FATHOM_ADAPTER_VERSION: &str = "fixture.v0.1";
const FATHOM_TEMPORAL_TRANSCRIPT: &str =
    include_str!("fixtures/fathom_temporal_evidence_transcript.json");

fn fathom_temporal_evidence_jtbd() -> JtbdInput {
    JtbdInput {
        key: "fathom-temporal-evidence".to_string(),
        actor: "portfolio analyst".to_string(),
        functional_job:
            "screen a time-indexed disclosure corpus for narrative drift worth analyst attention"
                .to_string(),
        so_that:
            "risk-language changes can be reviewed without collapsing disagreements into an unaudited recommendation"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "corpus_snapshot_bound",
                "the analysis names the corpus snapshot id, as-of time, corpus version, fixture refs, and snapshot hash",
            ),
            ClauseInput::with_key(
                "comparison_window_named",
                "the run names current period, prior period, comparison kind, consecutiveness, and missing-data notes",
            ),
            ClauseInput::with_key(
                "filing_sections_cited",
                "every compared issuer cites filing id, form, section, current and prior section refs, and source hash",
            ),
            ClauseInput::with_key(
                "query_plan_recorded",
                "the analytical query plan records engine, plan id, sql hash, parameter hash, and replayability",
            ),
            ClauseInput::with_key(
                "risk_count_delta_computed",
                "risk factor count deltas name current and prior periods, counts, and signed delta",
            ),
            ClauseInput::with_key(
                "language_drift_computed",
                "language drift records jaccard similarity, added count, removed count, and confidence for each issuer",
            ),
            ClauseInput::with_key(
                "peer_cohort_boundary_named",
                "peer comparison names cohort id, boundary, as-of date, member ids, and rationale hash",
            ),
            ClauseInput::with_key(
                "disagreement_preserved",
                "count-vs-language disagreements are preserved with evidence refs instead of averaged away",
            ),
            ClauseInput::with_key(
                "hitl_escalation_recorded",
                "low-confidence or contradictory analytical signals record HITL gate, status, reviewer role, and note hash",
            ),
            ClauseInput::with_key(
                "portfolio_selection_recorded",
                "portfolio selection records solver, analyst budget, selected issuers, selected weight, value, and rationale hash",
            ),
            ClauseInput::with_key(
                "narrative_claims_cited",
                "each narrative claim carries claim hash, citation refs, and citation status",
            ),
            ClauseInput::with_key(
                "recommendation_boundary_declared",
                "the output declares that it is an analyst-attention artifact and not an investment recommendation",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "snapshot_missing",
                "a temporal claim is made without a fixed corpus snapshot and as-of time",
            ),
            ClauseInput::with_key(
                "nonconsecutive_period_compared",
                "year-over-year drift is claimed across non-consecutive or unnamed periods",
            ),
            ClauseInput::with_key(
                "section_without_filing",
                "a risk-language claim cites a section without filing identity and source hash",
            ),
            ClauseInput::with_key(
                "query_plan_missing",
                "an analytical fact is promoted without a replayable query plan",
            ),
            ClauseInput::with_key(
                "count_delta_without_periods",
                "a risk count delta omits comparable periods or signed count arithmetic",
            ),
            ClauseInput::with_key(
                "language_drift_averaged_away",
                "language drift is collapsed into a score without added/removed evidence",
            ),
            ClauseInput::with_key(
                "cohort_boundary_missing",
                "peer comparison is made without a cohort boundary and as-of date",
            ),
            ClauseInput::with_key(
                "disagreement_collapsed",
                "contradictory analytical signals are averaged away instead of preserved",
            ),
            ClauseInput::with_key(
                "hitl_escalation_bypassed",
                "low-confidence or contradictory signals bypass analyst review",
            ),
            ClauseInput::with_key(
                "portfolio_selected_without_budget",
                "portfolio review targets are selected without analyst-capacity budget evidence",
            ),
            ClauseInput::with_key(
                "claim_without_citation",
                "a narrative claim appears without citations to promoted facts and source sections",
            ),
            ClauseInput::with_key(
                "investment_recommendation_without_authority",
                "the output is represented as an investment recommendation without investment authority",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(45)),
    }
}

#[test]
fn fathom_temporal_evidence_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Screen a time-indexed disclosure corpus")
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
            .any(|evidence| evidence.contains("as-of time"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("investment recommendation"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn fathom_temporal_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");
    let transcript = fathom_temporal_transcript();

    let observation =
        adapt_fathom_temporal_transcript(&package, &transcript).expect("Fathom adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.fathom-temporal-evidence")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Fathom-adapted temporal run preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 12);
    assert_eq!(audit.failure_coverage.len(), 12);
    assert_eq!(audit.facts_audited, 12);
}

#[test]
fn fathom_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");
    let transcript = fathom_temporal_transcript();

    let first = adapt_fathom_temporal_transcript_with_receipt(&package, &transcript);
    let second = adapt_fathom_temporal_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, FATHOM_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "fathom-narrative");
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
            "fathom.narrative.corpus-snapshot",
            "fathom.narrative.comparison-window",
            "fathom.narrative.filing-sections",
            "fathom.narrative.query-plan",
            "fathom.narrative.risk-count-deltas",
            "fathom.narrative.language-drifts",
            "fathom.narrative.peer-cohort",
            "fathom.narrative.disagreements",
            "fathom.narrative.hitl-review",
            "fathom.narrative.portfolio-selection",
            "fathom.narrative.claims",
            "fathom.narrative.recommendation-boundary",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 24);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("0000320193"));
    assert!(!serialized.contains("Apple Inc."));
    assert!(!serialized.contains("sec://edgar"));
    assert!(!serialized.contains("FATHOM_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/fathom-narrative"));
    assert!(!serialized.contains("investment recommendation"));
}

#[test]
fn fathom_job_readiness_packet_marks_missing_comparison_window() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");
    let mut transcript = fathom_temporal_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != FATHOM_WINDOW_TRUTH_KEY);
    transcript.execution_run.comparison_window.consecutive = false;
    transcript
        .execution_run
        .comparison_window
        .missing_data_notes
        .push("FY2024 baseline was not available at the requested as-of time".to_string());
    let adapter_outcome = adapt_fathom_temporal_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let window_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "comparison_window_named")
        .expect("comparison window evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(window_status.status, EvidenceReadinessStatus::Missing);
    assert!(window_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for comparison_window_named".to_string())
    );
}

#[test]
fn fathom_job_readiness_packet_marks_missing_recommendation_boundary() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");
    let mut transcript = fathom_temporal_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != FATHOM_BOUNDARY_TRUTH_KEY);
    transcript
        .execution_run
        .recommendation_boundary
        .investment_recommendation_presented = true;
    transcript
        .execution_run
        .recommendation_boundary
        .capital_decision_authority = "unknown".to_string();
    let adapter_outcome = adapt_fathom_temporal_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let boundary_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "recommendation_boundary_declared")
        .expect("recommendation boundary evidence is represented");
    let claims_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "narrative_claims_cited")
        .expect("claim evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(boundary_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(claims_status.status, EvidenceReadinessStatus::Present);
    assert!(boundary_status.fact_ids.is_empty());
}

#[test]
fn fathom_operator_ledger_entries_are_temporal_backlinks_without_recommendation_authority() {
    let package = decode_jtbd(fathom_temporal_evidence_jtbd()).expect("Fathom JTBD decodes");
    let transcript = fathom_temporal_transcript();
    let adapter_outcome = adapt_fathom_temporal_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let snapshot_receipt = corpus_snapshot_receipt(&packet, &transcript);
    let window_receipt = evidence_window_receipt(&packet, &transcript, &snapshot_receipt);
    let disagreement_receipt = disagreement_receipt(&packet, &transcript, &window_receipt);
    let review_receipt = analyst_review_receipt(&packet, &transcript, &disagreement_receipt);
    let claim_receipt = narrative_claim_receipt(&packet, &transcript, &review_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &snapshot_receipt,
        &window_receipt,
        &disagreement_receipt,
        &review_receipt,
        &claim_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &snapshot_receipt,
        &window_receipt,
        &disagreement_receipt,
        &review_receipt,
        &claim_receipt,
    );

    assert_eq!(first, second);
    assert_eq!(first.len(), 7);
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
        HelmLedgerRecordKind::CorpusSnapshotReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::EvidenceWindowReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::DisagreementReceipt
    );
    assert_eq!(
        first[5].record_kind,
        HelmLedgerRecordKind::AnalystReviewReceipt
    );
    assert_eq!(
        first[6].record_kind,
        HelmLedgerRecordKind::NarrativeClaimReceipt
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );
    assert_eq!(
        first[6].backlink_ids,
        vec![
            packet.packet_id.clone(),
            disagreement_receipt.receipt_id.clone(),
            review_receipt.receipt_id.clone(),
        ]
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("0000320193"));
    assert!(!serialized.contains("Apple Inc."));
    assert!(!serialized.contains("sec://edgar"));
    assert!(!serialized.contains("FATHOM_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/fathom-narrative"));
    assert!(!serialized.contains("investment recommendation"));
}

fn adapt_fathom_temporal_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &FathomTemporalTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_fathom_temporal_transcript(package, transcript) {
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
    transcript: &FathomTemporalTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: FATHOM_ADAPTER_ID.to_string(),
        adapter_version: FATHOM_ADAPTER_VERSION.to_string(),
        status,
        source_app: "fathom-narrative".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "fathom://temporal-evidence/{}/{}",
            transcript.source.run_id, transcript.execution_run.analysis_run_id
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
            "corpus_snapshot.fixture_refs".to_string(),
            "corpus_snapshot.table_ref".to_string(),
            "filing_sections.cik".to_string(),
            "filing_sections.company_name".to_string(),
            "filing_sections.section_refs".to_string(),
            "peer_cohort.ciks".to_string(),
            "disagreements.evidence_refs".to_string(),
            "narrative_claims.citation_refs".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &FathomTemporalTranscript,
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
        &transcript.execution_run.analysis_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!(
            "fathom://temporal-evidence/{}",
            transcript.execution_run.analysis_run_id
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
    actions.push("review preserved disagreements before publication".to_string());
    actions.push(
        "keep output as analyst-attention evidence, not recommendation authority".to_string(),
    );
    actions
}

fn corpus_snapshot_receipt(
    packet: &JobReadinessPacket,
    transcript: &FathomTemporalTranscript,
) -> CorpusSnapshotReceipt {
    let snapshot = &transcript.execution_run.corpus_snapshot;
    CorpusSnapshotReceipt {
        receipt_id: corpus_snapshot_receipt_id(packet, snapshot),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        snapshot_ref_hash: sha256_lines(&[snapshot.snapshot_id.as_str()]),
        as_of: snapshot.as_of.clone(),
        corpus_version: snapshot.corpus_version.clone(),
        fixture_count: snapshot.fixture_refs.len(),
        snapshot_hash: snapshot.snapshot_hash.clone(),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn evidence_window_receipt(
    packet: &JobReadinessPacket,
    transcript: &FathomTemporalTranscript,
    snapshot_receipt: &CorpusSnapshotReceipt,
) -> EvidenceWindowReceipt {
    let window = &transcript.execution_run.comparison_window;
    EvidenceWindowReceipt {
        receipt_id: evidence_window_receipt_id(packet, window, snapshot_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        current_period: window.current_period.clone(),
        prior_period: window.prior_period.clone(),
        comparison_kind: window.comparison_kind.clone(),
        consecutive: window.consecutive,
        missing_data_note_count: window.missing_data_notes.len(),
        corpus_snapshot_receipt_id: snapshot_receipt.receipt_id.clone(),
    }
}

fn disagreement_receipt(
    packet: &JobReadinessPacket,
    transcript: &FathomTemporalTranscript,
    window_receipt: &EvidenceWindowReceipt,
) -> DisagreementReceipt {
    let disagreement_hashes = transcript
        .execution_run
        .disagreements
        .iter()
        .map(|disagreement| disagreement.disagreement_id.as_str())
        .collect::<Vec<_>>();
    DisagreementReceipt {
        receipt_id: disagreement_receipt_id(packet, &disagreement_hashes, window_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        disagreement_hash: sha256_lines(&disagreement_hashes),
        disagreement_count: transcript.execution_run.disagreements.len(),
        hitl_required_count: transcript
            .execution_run
            .disagreements
            .iter()
            .filter(|disagreement| disagreement.hitl_required)
            .count(),
        evidence_hash: sha256_json(&transcript.execution_run.disagreements),
        evidence_window_receipt_id: window_receipt.receipt_id.clone(),
    }
}

fn analyst_review_receipt(
    packet: &JobReadinessPacket,
    transcript: &FathomTemporalTranscript,
    disagreement_receipt: &DisagreementReceipt,
) -> AnalystReviewReceipt {
    let review = &transcript.execution_run.hitl_review;
    AnalystReviewReceipt {
        receipt_id: analyst_review_receipt_id(packet, review, disagreement_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        gate_ref_hash: sha256_lines(&[review.gate_id.as_str()]),
        status: review.status.clone(),
        reviewer_role_hash: sha256_lines(&[review.reviewer_role.as_str()]),
        note_hash: review.note_hash.clone(),
        disagreement_receipt_id: disagreement_receipt.receipt_id.clone(),
    }
}

fn narrative_claim_receipt(
    packet: &JobReadinessPacket,
    transcript: &FathomTemporalTranscript,
    review_receipt: &AnalystReviewReceipt,
) -> NarrativeClaimReceipt {
    let claims = &transcript.execution_run.narrative_claims;
    NarrativeClaimReceipt {
        receipt_id: narrative_claim_receipt_id(packet, claims, review_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        claim_count: claims.len(),
        citation_count: claims.iter().map(|claim| claim.citation_refs.len()).sum(),
        claims_hash: sha256_json(claims),
        output_kind: transcript
            .execution_run
            .recommendation_boundary
            .output_kind
            .clone(),
        recommendation_boundary_ok: !transcript
            .execution_run
            .recommendation_boundary
            .investment_recommendation_presented,
        analyst_review_receipt_id: review_receipt.receipt_id.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    snapshot_receipt: &CorpusSnapshotReceipt,
    window_receipt: &EvidenceWindowReceipt,
    disagreement_receipt: &DisagreementReceipt,
    review_receipt: &AnalystReviewReceipt,
    claim_receipt: &NarrativeClaimReceipt,
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
            HelmLedgerRecordKind::CorpusSnapshotReceipt,
            snapshot_receipt.receipt_id.clone(),
            snapshot_receipt.package_id.clone(),
            snapshot_receipt.truth_version.clone(),
            snapshot_receipt.domain_hint.clone(),
            sha256_json(snapshot_receipt),
            vec![packet.packet_id.clone()],
            format!("corpus snapshot as-of {}", snapshot_receipt.as_of),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::EvidenceWindowReceipt,
            window_receipt.receipt_id.clone(),
            window_receipt.package_id.clone(),
            window_receipt.truth_version.clone(),
            window_receipt.domain_hint.clone(),
            sha256_json(window_receipt),
            vec![
                packet.packet_id.clone(),
                snapshot_receipt.receipt_id.clone(),
            ],
            format!(
                "evidence window {} vs {}",
                window_receipt.current_period, window_receipt.prior_period
            ),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::DisagreementReceipt,
            disagreement_receipt.receipt_id.clone(),
            disagreement_receipt.package_id.clone(),
            disagreement_receipt.truth_version.clone(),
            disagreement_receipt.domain_hint.clone(),
            sha256_json(disagreement_receipt),
            vec![packet.packet_id.clone(), window_receipt.receipt_id.clone()],
            format!(
                "preserved {} disagreement(s)",
                disagreement_receipt.disagreement_count
            ),
        ),
        helm_ledger_entry(
            5,
            HelmLedgerRecordKind::AnalystReviewReceipt,
            review_receipt.receipt_id.clone(),
            review_receipt.package_id.clone(),
            review_receipt.truth_version.clone(),
            review_receipt.domain_hint.clone(),
            sha256_json(review_receipt),
            vec![
                packet.packet_id.clone(),
                disagreement_receipt.receipt_id.clone(),
            ],
            format!("analyst review {}", review_receipt.status),
        ),
        helm_ledger_entry(
            6,
            HelmLedgerRecordKind::NarrativeClaimReceipt,
            claim_receipt.receipt_id.clone(),
            claim_receipt.package_id.clone(),
            claim_receipt.truth_version.clone(),
            claim_receipt.domain_hint.clone(),
            sha256_json(claim_receipt),
            vec![
                packet.packet_id.clone(),
                disagreement_receipt.receipt_id.clone(),
                review_receipt.receipt_id.clone(),
            ],
            format!("narrative claims {}", claim_receipt.output_kind),
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

fn adapt_fathom_temporal_transcript(
    package: &TruthPackage,
    transcript: &FathomTemporalTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.execution_run;
    if run.status != "Converged" {
        return Err("expected Fathom temporal run to converge before adaptation".to_string());
    }
    if run.filing_sections.is_empty() {
        return Err("expected Fathom run to carry filing sections".to_string());
    }

    let corpus_snapshot_bound = evidence_clause_id(package, "corpus_snapshot_bound");
    let comparison_window_named = evidence_clause_id(package, "comparison_window_named");
    let filing_sections_cited = evidence_clause_id(package, "filing_sections_cited");
    let query_plan_recorded = evidence_clause_id(package, "query_plan_recorded");
    let risk_count_delta_computed = evidence_clause_id(package, "risk_count_delta_computed");
    let language_drift_computed = evidence_clause_id(package, "language_drift_computed");
    let peer_cohort_boundary_named = evidence_clause_id(package, "peer_cohort_boundary_named");
    let disagreement_preserved = evidence_clause_id(package, "disagreement_preserved");
    let hitl_escalation_recorded = evidence_clause_id(package, "hitl_escalation_recorded");
    let portfolio_selection_recorded = evidence_clause_id(package, "portfolio_selection_recorded");
    let narrative_claims_cited = evidence_clause_id(package, "narrative_claims_cited");
    let recommendation_boundary_declared =
        evidence_clause_id(package, "recommendation_boundary_declared");
    let snapshot_missing = failure_clause_id(package, "snapshot_missing");
    let nonconsecutive_period_compared =
        failure_clause_id(package, "nonconsecutive_period_compared");
    let section_without_filing = failure_clause_id(package, "section_without_filing");
    let query_plan_missing = failure_clause_id(package, "query_plan_missing");
    let count_delta_without_periods = failure_clause_id(package, "count_delta_without_periods");
    let language_drift_averaged_away = failure_clause_id(package, "language_drift_averaged_away");
    let cohort_boundary_missing = failure_clause_id(package, "cohort_boundary_missing");
    let disagreement_collapsed = failure_clause_id(package, "disagreement_collapsed");
    let hitl_escalation_bypassed = failure_clause_id(package, "hitl_escalation_bypassed");
    let portfolio_selected_without_budget =
        failure_clause_id(package, "portfolio_selected_without_budget");
    let claim_without_citation = failure_clause_id(package, "claim_without_citation");
    let investment_recommendation_without_authority =
        failure_clause_id(package, "investment_recommendation_without_authority");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&run.truth_keys, FATHOM_SNAPSHOT_TRUTH_KEY)
        && run.corpus_snapshot.snapshot_hash.starts_with("sha256:")
        && !run.corpus_snapshot.fixture_refs.is_empty()
        && !run.corpus_snapshot.as_of.trim().is_empty()
    {
        promoted_facts.push(fathom_fact(
            "CorpusSnapshot",
            "fathom.narrative.corpus-snapshot",
            "corpus snapshot names as-of time, corpus version, fixture refs, and snapshot hash",
            vec![corpus_snapshot_bound, snapshot_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_WINDOW_TRUTH_KEY)
        && run.comparison_window.consecutive
        && !run.comparison_window.current_period.trim().is_empty()
        && !run.comparison_window.prior_period.trim().is_empty()
        && run.comparison_window.comparison_kind == "year_over_year"
    {
        promoted_facts.push(fathom_fact(
            "ComparisonWindow",
            "fathom.narrative.comparison-window",
            "current and prior periods, comparison kind, consecutiveness, and missing-data notes are recorded",
            vec![comparison_window_named, nonconsecutive_period_compared],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_SECTIONS_TRUTH_KEY)
        && run.filing_sections.iter().all(|section| {
            section.form == "10-K"
                && section.section == "Item 1A"
                && !section.current_section_ref.trim().is_empty()
                && !section.prior_section_ref.trim().is_empty()
                && section.source_hash.starts_with("sha256:")
        })
    {
        promoted_facts.push(fathom_fact(
            "FilingSections",
            "fathom.narrative.filing-sections",
            "compared issuers cite filing id, form, Item 1A refs, and source hashes",
            vec![filing_sections_cited, section_without_filing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_QUERY_PLAN_TRUTH_KEY)
        && run.query_plan.replayable
        && run.query_plan.sql_hash.starts_with("sha256:")
        && run.query_plan.parameters_hash.starts_with("sha256:")
    {
        promoted_facts.push(fathom_fact(
            "QueryPlan",
            "fathom.narrative.query-plan",
            "query plan records engine, plan id, sql hash, parameter hash, and replayability",
            vec![query_plan_recorded, query_plan_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_COUNT_DELTA_TRUTH_KEY)
        && run.risk_count_deltas.iter().all(|delta| {
            delta.current_fiscal_year == delta.prior_fiscal_year + 1
                && delta.current_count as i32 - delta.prior_count as i32 == delta.delta
        })
    {
        promoted_facts.push(fathom_fact(
            "RiskCountDeltas",
            "fathom.narrative.risk-count-deltas",
            "risk factor count deltas name periods, counts, and signed arithmetic",
            vec![risk_count_delta_computed, count_delta_without_periods],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_LANGUAGE_DRIFT_TRUTH_KEY)
        && run.language_drifts.iter().all(|drift| {
            drift.current_fiscal_year == drift.prior_fiscal_year + 1
                && drift.confidence_basis_points <= 10_000
                && drift.added_count as i32 - drift.removed_count as i32
                    == risk_delta_for(
                        &run.risk_count_deltas,
                        drift.cik.as_str(),
                        drift.current_fiscal_year,
                    )
        })
    {
        promoted_facts.push(fathom_fact(
            "LanguageDrifts",
            "fathom.narrative.language-drifts",
            "language drift records jaccard similarity, added/removed counts, and confidence",
            vec![language_drift_computed, language_drift_averaged_away],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_COHORT_TRUTH_KEY)
        && !run.peer_cohort.ciks.is_empty()
        && !run.peer_cohort.boundary.trim().is_empty()
        && run.peer_cohort.rationale_hash.starts_with("sha256:")
    {
        promoted_facts.push(fathom_fact(
            "PeerCohort",
            "fathom.narrative.peer-cohort",
            "peer cohort names boundary, as-of date, member ids, and rationale hash",
            vec![peer_cohort_boundary_named, cohort_boundary_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_DISAGREEMENT_TRUTH_KEY)
        && run.disagreements.iter().all(|disagreement| {
            disagreement.status == "PreservedForReview"
                && disagreement.hitl_required
                && !disagreement.evidence_refs.is_empty()
                && disagreement.summary_hash.starts_with("sha256:")
        })
    {
        promoted_facts.push(fathom_fact(
            "Disagreements",
            "fathom.narrative.disagreements",
            "count-vs-language disagreements are preserved with evidence refs",
            vec![disagreement_preserved, disagreement_collapsed],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_HITL_TRUTH_KEY)
        && run.hitl_review.status == "ApprovedForAnalystAttention"
        && run.hitl_review.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(fathom_fact(
            "HitlReview",
            "fathom.narrative.hitl-review",
            "HITL review records gate, status, reviewer role, and note hash",
            vec![hitl_escalation_recorded, hitl_escalation_bypassed],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_PORTFOLIO_TRUTH_KEY)
        && run.portfolio_selection.budget_risk_factors > 0
        && run.portfolio_selection.selected_weight <= run.portfolio_selection.budget_risk_factors
        && !run.portfolio_selection.selected_ciks.is_empty()
        && run
            .portfolio_selection
            .rationale_hash
            .starts_with("sha256:")
    {
        promoted_facts.push(fathom_fact(
            "PortfolioSelection",
            "fathom.narrative.portfolio-selection",
            "portfolio selection records solver, analyst budget, selected issuers, weight, value, and rationale",
            vec![portfolio_selection_recorded, portfolio_selected_without_budget],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_CLAIMS_TRUTH_KEY)
        && run.narrative_claims.iter().all(|claim| {
            claim.status == "Cited"
                && claim.claim_hash.starts_with("sha256:")
                && !claim.citation_refs.is_empty()
        })
    {
        promoted_facts.push(fathom_fact(
            "NarrativeClaims",
            "fathom.narrative.claims",
            "narrative claims carry hashes, citation refs, and citation status",
            vec![narrative_claims_cited, claim_without_citation],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FATHOM_BOUNDARY_TRUTH_KEY)
        && run.recommendation_boundary.output_kind == "analyst_attention_queue"
        && !run
            .recommendation_boundary
            .investment_recommendation_presented
        && run.recommendation_boundary.capital_decision_authority == "analyst_owned"
    {
        promoted_facts.push(fathom_fact(
            "RecommendationBoundary",
            "fathom.narrative.recommendation-boundary",
            "output boundary declares analyst-attention artifact rather than recommendation authority",
            vec![
                recommendation_boundary_declared,
                investment_recommendation_without_authority,
            ],
            &run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:fathom-temporal-evidence", 43, 12),
        replay_notes: vec![
            format!(
                "adapted Fathom temporal analysis {} into AxiomRunObservation",
                run.analysis_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn risk_delta_for(deltas: &[FathomRiskCountDelta], cik: &str, current_fiscal_year: u16) -> i32 {
    deltas
        .iter()
        .find(|delta| delta.cik == cik && delta.current_fiscal_year == current_fiscal_year)
        .map_or(i32::MIN, |delta| delta.delta)
}

fn fathom_fact(
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
            evidence_id: format!("fathom.evidence.{fact_id}"),
            source: "fathom-temporal-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("fathom.trace.{fact_id}"),
            location: Some("fathom://temporal-evidence".to_string()),
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
    analysis_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    short_id(
        &sha256_lines(&[
            "job_readiness_packet",
            package.package_id.as_str(),
            package.truth_version.as_str(),
            domain_hint,
            analysis_run_id,
            adapter_receipt_id,
        ]),
        "helm.job_readiness",
    )
}

fn corpus_snapshot_receipt_id(
    packet: &JobReadinessPacket,
    snapshot: &FathomCorpusSnapshot,
) -> String {
    short_id(
        &sha256_lines(&[
            "corpus_snapshot_receipt",
            packet.packet_id.as_str(),
            snapshot.snapshot_id.as_str(),
            snapshot.as_of.as_str(),
        ]),
        "helm.corpus_snapshot",
    )
}

fn evidence_window_receipt_id(
    packet: &JobReadinessPacket,
    window: &FathomComparisonWindow,
    snapshot_receipt: &CorpusSnapshotReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "evidence_window_receipt",
            packet.packet_id.as_str(),
            window.current_period.as_str(),
            window.prior_period.as_str(),
            snapshot_receipt.receipt_id.as_str(),
        ]),
        "helm.evidence_window",
    )
}

fn disagreement_receipt_id(
    packet: &JobReadinessPacket,
    disagreement_ids: &[&str],
    window_receipt: &EvidenceWindowReceipt,
) -> String {
    let disagreement_hash = sha256_lines(disagreement_ids);
    short_id(
        &sha256_lines(&[
            "disagreement_receipt",
            packet.packet_id.as_str(),
            disagreement_hash.as_str(),
            window_receipt.receipt_id.as_str(),
        ]),
        "helm.disagreement",
    )
}

fn analyst_review_receipt_id(
    packet: &JobReadinessPacket,
    review: &FathomHitlReview,
    disagreement_receipt: &DisagreementReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "analyst_review_receipt",
            packet.packet_id.as_str(),
            review.gate_id.as_str(),
            review.status.as_str(),
            disagreement_receipt.receipt_id.as_str(),
        ]),
        "helm.analyst_review",
    )
}

fn narrative_claim_receipt_id(
    packet: &JobReadinessPacket,
    claims: &[FathomNarrativeClaim],
    review_receipt: &AnalystReviewReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "narrative_claim_receipt",
            packet.packet_id.as_str(),
            sha256_json(&claims).as_str(),
            review_receipt.receipt_id.as_str(),
        ]),
        "helm.narrative_claim",
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
struct CorpusSnapshotReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    snapshot_ref_hash: String,
    as_of: String,
    corpus_version: String,
    fixture_count: usize,
    snapshot_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct EvidenceWindowReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    current_period: String,
    prior_period: String,
    comparison_kind: String,
    consecutive: bool,
    missing_data_note_count: usize,
    corpus_snapshot_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct DisagreementReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    disagreement_hash: String,
    disagreement_count: usize,
    hitl_required_count: usize,
    evidence_hash: String,
    evidence_window_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct AnalystReviewReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    gate_ref_hash: String,
    status: String,
    reviewer_role_hash: String,
    note_hash: String,
    disagreement_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct NarrativeClaimReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    claim_count: usize,
    citation_count: usize,
    claims_hash: String,
    output_kind: String,
    recommendation_boundary_ok: bool,
    analyst_review_receipt_id: String,
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
    CorpusSnapshotReceipt,
    EvidenceWindowReceipt,
    DisagreementReceipt,
    AnalystReviewReceipt,
    NarrativeClaimReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::CorpusSnapshotReceipt => "corpus_snapshot_receipt",
            Self::EvidenceWindowReceipt => "evidence_window_receipt",
            Self::DisagreementReceipt => "disagreement_receipt",
            Self::AnalystReviewReceipt => "analyst_review_receipt",
            Self::NarrativeClaimReceipt => "narrative_claim_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomTemporalTranscript {
    source: FathomRunSource,
    execution_run: FathomExecutionRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomExecutionRun {
    analysis_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    corpus_snapshot: FathomCorpusSnapshot,
    comparison_window: FathomComparisonWindow,
    filing_sections: Vec<FathomFilingSection>,
    query_plan: FathomQueryPlan,
    risk_count_deltas: Vec<FathomRiskCountDelta>,
    language_drifts: Vec<FathomLanguageDrift>,
    peer_cohort: FathomPeerCohort,
    disagreements: Vec<FathomDisagreement>,
    hitl_review: FathomHitlReview,
    portfolio_selection: FathomPortfolioSelection,
    narrative_claims: Vec<FathomNarrativeClaim>,
    recommendation_boundary: FathomRecommendationBoundary,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomCorpusSnapshot {
    snapshot_id: String,
    as_of: String,
    corpus_version: String,
    table_ref: String,
    fixture_refs: Vec<String>,
    snapshot_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomComparisonWindow {
    current_period: String,
    prior_period: String,
    comparison_kind: String,
    consecutive: bool,
    missing_data_notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomFilingSection {
    cik: String,
    company_name: String,
    current_fiscal_year: u16,
    prior_fiscal_year: u16,
    form: String,
    section: String,
    current_section_ref: String,
    prior_section_ref: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomQueryPlan {
    plan_id: String,
    engine: String,
    sql_hash: String,
    parameters_hash: String,
    replayable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomRiskCountDelta {
    cik: String,
    current_fiscal_year: u16,
    prior_fiscal_year: u16,
    current_count: usize,
    prior_count: usize,
    delta: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomLanguageDrift {
    cik: String,
    current_fiscal_year: u16,
    prior_fiscal_year: u16,
    jaccard_similarity: f64,
    added_count: usize,
    removed_count: usize,
    confidence_basis_points: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomPeerCohort {
    cohort_id: String,
    boundary: String,
    as_of: String,
    ciks: Vec<String>,
    rationale_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomDisagreement {
    disagreement_id: String,
    cik: String,
    kind: String,
    status: String,
    hitl_required: bool,
    evidence_refs: Vec<String>,
    summary_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomHitlReview {
    gate_id: String,
    status: String,
    reviewer_role: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomPortfolioSelection {
    selection_id: String,
    solver: String,
    budget_risk_factors: usize,
    selected_ciks: Vec<String>,
    selected_weight: usize,
    selection_value: usize,
    rationale_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomNarrativeClaim {
    claim_id: String,
    claim_hash: String,
    citation_refs: Vec<String>,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FathomRecommendationBoundary {
    output_kind: String,
    investment_recommendation_presented: bool,
    capital_decision_authority: String,
    legal_review_required_before_publication: bool,
}

fn fathom_temporal_transcript() -> FathomTemporalTranscript {
    serde_json::from_str(FATHOM_TEMPORAL_TRANSCRIPT).expect("Fathom temporal transcript parses")
}
