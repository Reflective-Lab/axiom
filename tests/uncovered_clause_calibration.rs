//! v0.15 — calibration records for uncovered clauses.
//!
//! These tests pin the missing-evidence side of the decoder calibration
//! loop:
//!
//! - An `Invalid` verdict caused by missing evidence produces typed
//!   `Concern` calibration records, one per uncovered `EvidenceRequired`
//!   clause; covered clauses still produce `Reinforcement` records.
//! - The `Concern`/`Reinforcement` distinction is typed via
//!   `CalibrationSignalKind` and survives the JSONL persistence boundary.
//! - Operator-accepted `Concern` records produce `CalibrationConcern`
//!   artifacts (not `CalibrationSuggestion`) when a regenerated package
//!   is re-decoded with calibration applied.
//! - The **non-weakening invariant**: accepting `Concern` records must
//!   never modify the source JTBD's `verifier_spec.required_evidence`
//!   or `forbidden_actions`. Concerns propose decoder affordances, not
//!   contract relaxation.
//! - Scope clauses (`Actor`, `FunctionalJob`, `SoThat`) never generate
//!   `Concern` records even when uncovered (they're not coverage
//!   candidates). Uncovered `FailureMode` clauses also do not generate
//!   concerns — most failure modes go uncited in any single run and
//!   that is the normal case.
//! - The boundary discipline test extends v0.13's serialized-record
//!   substring check to the new `Concern` signal kind: even `Concern`
//!   records must carry no formation, specialist, approver, or gate
//!   identity.

use axiom_truth::{
    ArtifactKind, AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, CalibrationRecord,
    CalibrationSignalKind, CalibrationStatus, CalibrationTable, ClauseCoverageStatus, ClauseId,
    ClauseInput, EvidenceRefRecord, JtbdClauseKind, JtbdInput, LearningEpisode, ObservedStopReason,
    PromotedFactRecord, RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage,
    apply_decoder_calibration, calibration_records_from_learning_episode, decode_jtbd,
};

const DOMAIN_HINT: &str = "uncovered-clause.fixture";

fn minimal_jtbd() -> JtbdInput {
    JtbdInput {
        key: "uncovered-clause-fixture".to_string(),
        actor: "fixture operator".to_string(),
        functional_job: "exercise the uncovered-clause calibration path".to_string(),
        so_that: "v0.15 concerns and reinforcements stay distinguishable".to_string(),
        evidence_required: vec![
            ClauseInput::with_key("covered_evidence", "evidence the run cited"),
            ClauseInput::with_key("uncovered_evidence_a", "evidence the run never produced"),
            ClauseInput::with_key(
                "uncovered_evidence_b",
                "another evidence the run never produced",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key("known_failure", "an explicitly guarded failure mode"),
            ClauseInput::with_key("untouched_failure", "a failure mode no fact cited"),
        ],
        time_budget: Some(TimeBudget::from_minutes(5)),
    }
}

#[test]
fn invalid_run_with_missing_evidence_emits_concern_records_for_uncovered_clauses() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);

    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("invalid episode produces calibration records");

    let reinforcements: Vec<&CalibrationRecord> = records
        .iter()
        .filter(|record| record.value.signal_kind == CalibrationSignalKind::Reinforcement)
        .collect();
    let concerns: Vec<&CalibrationRecord> = records
        .iter()
        .filter(|record| record.value.signal_kind == CalibrationSignalKind::Concern)
        .collect();

    // One covered evidence clause + one covered failure clause = 2 reinforcements
    assert_eq!(
        reinforcements.len(),
        2,
        "expected one Reinforcement per covered clause (evidence + failure guard)",
    );
    // Two uncovered EvidenceRequired clauses = 2 concerns
    assert_eq!(
        concerns.len(),
        2,
        "expected one Concern per uncovered EvidenceRequired clause",
    );

    // Every concern's key should reference an EvidenceRequired clause kind.
    assert!(
        concerns
            .iter()
            .all(|record| record.key.clause_kind == JtbdClauseKind::EvidenceRequired),
        "concerns must only fire for EvidenceRequired clauses",
    );

    // Every concern's rationale should reference the missing-evidence framing.
    assert!(
        concerns.iter().all(|record| record
            .value
            .rationale
            .contains("was not cited by any promoted fact")),
        "concern rationale must explain the uncovered-clause framing",
    );
}

#[test]
fn satisfied_and_exhausted_verdicts_never_emit_concerns() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");

    for verdict in [AxiomRunVerdict::Satisfied, AxiomRunVerdict::Exhausted] {
        let episode = synthetic_episode(&package, verdict, /*cite_all_evidence=*/ false);
        let records = calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces calibration records");

        let concern_count = records
            .iter()
            .filter(|record| record.value.signal_kind == CalibrationSignalKind::Concern)
            .count();

        assert_eq!(
            concern_count, 0,
            "verdict {verdict:?} must not emit Concern records (got {concern_count})",
        );
    }
}

#[test]
fn blocked_verdict_emits_concerns_for_uncovered_evidence_clauses() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = synthetic_episode(
        &package,
        AxiomRunVerdict::Blocked,
        /*cite_all_evidence=*/ false,
    );

    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("blocked episode produces calibration records");
    let concerns: Vec<&CalibrationRecord> = records
        .iter()
        .filter(|record| record.value.signal_kind == CalibrationSignalKind::Concern)
        .collect();

    assert!(
        !concerns.is_empty(),
        "Blocked verdict must emit Concern records for uncovered EvidenceRequired clauses",
    );
    assert!(
        concerns
            .iter()
            .all(|record| record.key.clause_kind == JtbdClauseKind::EvidenceRequired),
    );
}

#[test]
fn scope_and_failure_clauses_never_generate_concern_records_when_uncovered() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);

    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("invalid episode produces calibration records");
    let concerns: Vec<&CalibrationRecord> = records
        .iter()
        .filter(|record| record.value.signal_kind == CalibrationSignalKind::Concern)
        .collect();

    // No Actor / FunctionalJob / SoThat concern records.
    for kind in [
        JtbdClauseKind::Actor,
        JtbdClauseKind::FunctionalJob,
        JtbdClauseKind::SoThat,
        JtbdClauseKind::FailureMode,
    ] {
        assert!(
            concerns.iter().all(|record| record.key.clause_kind != kind),
            "kind {kind:?} must not appear in concern records, even when uncovered",
        );
    }
}

#[test]
fn accepted_concern_produces_calibration_concern_artifact_not_suggestion() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);
    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces records")
            .into_iter()
            .map(|record| record.accepted("operator accepted prior"))
            .collect();
    let concern_count = records
        .iter()
        .filter(|record| record.value.signal_kind == CalibrationSignalKind::Concern)
        .count();
    let reinforcement_count = records.len() - concern_count;
    let table = CalibrationTable::new(records);

    let regenerated = decode_jtbd(minimal_jtbd()).expect("fixture regenerates");
    let enriched = apply_decoder_calibration(regenerated, &table, DOMAIN_HINT)
        .expect("accepted calibration enriches package");

    assert_eq!(enriched.artifacts.calibration_concerns.len(), concern_count);
    assert_eq!(
        enriched.artifacts.calibration_suggestions.len(),
        reinforcement_count,
    );
    assert!(
        enriched
            .artifacts
            .calibration_concerns
            .iter()
            .all(|artifact| artifact.artifact_kind == ArtifactKind::CalibrationConcern),
        "concern artifacts must use ArtifactKind::CalibrationConcern",
    );
    assert!(
        enriched
            .artifacts
            .calibration_suggestions
            .iter()
            .all(|artifact| artifact.artifact_kind == ArtifactKind::CalibrationSuggestion),
        "reinforcement artifacts must use ArtifactKind::CalibrationSuggestion",
    );
    // Lineage still closes after the new concern artifacts are added.
    assert!(
        enriched
            .lineage
            .validate_closure(&enriched.source_jtbd)
            .is_ok(),
    );
}

#[test]
fn accepted_concerns_do_not_weaken_verifier_spec() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let baseline_required_evidence = package.verifier_spec.required_evidence.clone();
    let baseline_forbidden_actions = serde_json::to_value(&package.verifier_spec.forbidden_actions)
        .expect("baseline forbidden actions serialize");

    let episode = invalid_episode_missing_evidence(&package);
    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces records")
            .into_iter()
            .map(|record| record.accepted("operator accepted prior"))
            .collect();
    let table = CalibrationTable::new(records);

    let regenerated = decode_jtbd(minimal_jtbd()).expect("fixture regenerates");
    let enriched = apply_decoder_calibration(regenerated, &table, DOMAIN_HINT)
        .expect("accepted calibration enriches package");

    // Non-weakening invariant — the source JTBD's evidence requirements and
    // forbidden actions remain identical after Concern acceptance.
    assert_eq!(
        enriched.verifier_spec.required_evidence, baseline_required_evidence,
        "calibration must not modify required_evidence",
    );
    assert_eq!(
        serde_json::to_value(&enriched.verifier_spec.forbidden_actions)
            .expect("post-calibration forbidden actions serialize"),
        baseline_forbidden_actions,
        "calibration must not modify forbidden_actions",
    );
}

#[test]
fn concern_records_round_trip_through_jsonl() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);
    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces records")
            .into_iter()
            .map(|record| {
                if record.value.signal_kind == CalibrationSignalKind::Concern {
                    record.with_status(CalibrationStatus::Accepted, "operator accepted concern")
                } else {
                    record.accepted("operator accepted reinforcement")
                }
            })
            .collect();
    let table = CalibrationTable::new(records);

    let serialized = table.to_jsonl();
    let replayed = CalibrationTable::from_jsonl(&serialized).expect("jsonl round trip");
    assert_eq!(table, replayed);

    // Spot-check that the wire format carries the signal_kind discriminator.
    assert!(
        serialized.contains("\"signal_kind\":\"concern\""),
        "JSONL must serialize Concern signal_kind",
    );
    assert!(
        serialized.contains("\"signal_kind\":\"reinforcement\""),
        "JSONL must serialize Reinforcement signal_kind",
    );
}

#[test]
fn concern_records_carry_no_runtime_ownership_substrings() {
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);
    let records = calibration_records_from_learning_episode(&package, &episode)
        .expect("episode produces records");

    let serialized = serde_json::to_string(&records).expect("records serialize");
    for forbidden in ["formation", "specialist", "approver_id", "gate_id"] {
        assert!(
            !serialized.contains(forbidden),
            "calibration records (including Concerns) must not capture runtime ownership: found {forbidden:?}",
        );
    }
}

fn invalid_episode_missing_evidence(package: &TruthPackage) -> LearningEpisode {
    synthetic_episode(
        package,
        AxiomRunVerdict::Invalid,
        /*cite_all_evidence=*/ false,
    )
}

/// Build a `LearningEpisode` that exercises the v0.15 paths.
///
/// - `verdict` controls which `AxiomRunVerdict` the synthetic report carries.
/// - When `cite_all_evidence` is false, only the `covered_evidence` clause
///   is cited by a promoted fact; the other two evidence clauses are left
///   uncovered. The single covered failure mode is also cited so the
///   reinforcement path stays exercised.
fn synthetic_episode(
    package: &TruthPackage,
    verdict: AxiomRunVerdict,
    cite_all_evidence: bool,
) -> LearningEpisode {
    let covered_evidence = evidence_clause_id(package, "covered_evidence");
    let known_failure = failure_clause_id(package, "known_failure");

    let mut promoted_facts = vec![
        build_fact("fact.covered_evidence", vec![covered_evidence.clone()]),
        build_fact("fact.known_failure_guard", vec![known_failure]),
    ];

    if cite_all_evidence {
        promoted_facts.push(build_fact(
            "fact.uncovered_evidence_a",
            vec![evidence_clause_id(package, "uncovered_evidence_a")],
        ));
        promoted_facts.push(build_fact(
            "fact.uncovered_evidence_b",
            vec![evidence_clause_id(package, "uncovered_evidence_b")],
        ));
    }

    let observation = AxiomRunObservation {
        // Stop reason: Satisfied uses Converged so verify() can match the
        // expected set; Invalid/Blocked use stop reasons that fail
        // verifier expectations (we bypass verify and construct the report
        // directly via from_observation so the verdict is controlled).
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:uncovered-fixture", 1, 2),
        replay_notes: vec!["synthetic uncovered-clause fixture".to_string()],
        run_stages: Vec::new(),
    };

    let report = AxiomRunReport::from_observation(package, verdict, observation);
    let audit = report
        .audit_fact_lineage(package)
        .expect("synthetic observation audits cleanly");

    LearningEpisode::from_report(
        "uncovered-fixture-run",
        DOMAIN_HINT,
        package,
        &report,
        &audit,
    )
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

fn build_fact(fact_id: &str, source_clause_ids: Vec<ClauseId>) -> PromotedFactRecord {
    PromotedFactRecord {
        context_key: "Evidence".to_string(),
        fact_id: fact_id.to_string(),
        summary: format!("synthetic observation for {fact_id}"),
        source_clause_ids,
        evidence_refs: vec![EvidenceRefRecord {
            evidence_id: format!("evidence.{fact_id}"),
            source: "uncovered-clause-fixture".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace.{fact_id}"),
            location: Some("fixture://uncovered-clause".to_string()),
            replayable: true,
        }),
        promotion_authority: None,
    }
}

#[test]
fn uncovered_clause_coverage_status_round_trip() {
    // Direct sanity check on the new enum: an uncovered EvidenceRequired
    // clause surfaces as ClauseCoverageStatus::Uncovered in the episode's
    // LearningClauseSignal, and the helper predicates classify it correctly.
    let package = decode_jtbd(minimal_jtbd()).expect("fixture decodes");
    let episode = invalid_episode_missing_evidence(&package);

    let uncovered = episode
        .source_clause_signals
        .iter()
        .find(|signal| signal.clause_id == evidence_clause_id(&package, "uncovered_evidence_a"))
        .expect("episode carries a signal for every clause");

    assert_eq!(uncovered.coverage_status, ClauseCoverageStatus::Uncovered);
    assert!(uncovered.coverage_status.is_uncovered());
    assert!(!uncovered.coverage_status.was_covered_as_evidence());
    assert!(!uncovered.coverage_status.was_covered_as_failure_guard());

    let covered = episode
        .source_clause_signals
        .iter()
        .find(|signal| signal.clause_id == evidence_clause_id(&package, "covered_evidence"))
        .expect("episode carries a signal for every clause");

    assert_eq!(
        covered.coverage_status,
        ClauseCoverageStatus::CoveredAsEvidence
    );
    assert!(covered.coverage_status.was_covered_as_evidence());
}
