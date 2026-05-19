//! v0.14 — persistence layer for `CalibrationTable`.
//!
//! These tests pin the JSONL wire format used to persist proposed/accepted/
//! rejected calibration records between operator review sessions:
//!
//! - byte-deterministic serialization (sorted by record id, fields in
//!   declaration order via serde derive);
//! - lossless round-trip through `to_jsonl` / `from_jsonl`;
//! - golden replay: a persisted accepted table regenerates byte-identical
//!   calibration suggestions when re-applied to a freshly decoded package;
//! - mixed-status persistence: only `Accepted` records enrich a regenerated
//!   package even after a JSONL round-trip (the v0.13 invariant survives the
//!   persistence boundary);
//! - typed errors for malformed lines and duplicate record ids.
//!
//! v0.14 deliberately leaves review APIs / CLI affordances for accept/reject/
//! reset to a follow-up commit; this slice is the data layer.

use axiom_truth::{
    ArtifactId, AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, CalibrationPersistenceError,
    CalibrationRecord, CalibrationReviewError, CalibrationStatus, CalibrationTable, ClauseId,
    ClauseInput, EvidenceRefRecord, JtbdClauseKind, JtbdInput, LearningEpisode, ObservedStopReason,
    PromotedFactRecord, RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage,
    apply_decoder_calibration, calibration_records_from_learning_episode, decode_jtbd,
};

const REVIEW_NOTE: &str = "operator accepted Tally release decoder prior";
const DOMAIN_HINT: &str = "tally-escrow.release";

fn tally_calibration_table() -> (TruthPackage, CalibrationTable) {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let episode = synthetic_episode(&package);
    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces records")
            .into_iter()
            .map(|record| record.accepted(REVIEW_NOTE))
            .collect();
    (package, CalibrationTable::new(records))
}

#[test]
fn accepted_calibration_table_round_trips_through_jsonl() {
    let (_, table) = tally_calibration_table();

    let serialized = table.to_jsonl();
    let parsed =
        CalibrationTable::from_jsonl(&serialized).expect("persisted table parses back cleanly");

    assert_eq!(table, parsed);
}

#[test]
fn to_jsonl_is_byte_deterministic_across_invocations() {
    let (_, table) = tally_calibration_table();

    let first = table.to_jsonl();
    let second = table.to_jsonl();

    assert_eq!(first, second);
    assert!(
        first.lines().count() >= 5,
        "expected one line per record; got {}",
        first.lines().count()
    );
    assert!(
        first
            .lines()
            .all(|line| line.starts_with("{\"record_id\":\"calibration_record.")),
        "every line should serialize a CalibrationRecord starting with record_id"
    );
}

#[test]
fn golden_replay_persisted_accepted_table_regenerates_identical_suggestions() {
    let (package, table) = tally_calibration_table();

    let direct = apply_decoder_calibration(package.clone(), &table, DOMAIN_HINT)
        .expect("direct apply succeeds");

    let serialized = table.to_jsonl();
    let replayed_table = CalibrationTable::from_jsonl(&serialized).expect("table replays");
    let replayed = apply_decoder_calibration(package.clone(), &replayed_table, DOMAIN_HINT)
        .expect("replayed apply succeeds");

    assert_eq!(
        direct.artifacts.calibration_suggestions,
        replayed.artifacts.calibration_suggestions
    );
    assert_eq!(direct.lineage, replayed.lineage);
    assert_eq!(
        serde_json::to_value(&direct).expect("direct package serializes"),
        serde_json::to_value(&replayed).expect("replayed package serializes"),
    );
}

#[test]
fn persisted_mixed_status_table_only_enriches_via_accepted_records() {
    let package = decode_jtbd(escrow_release_jtbd()).expect("escrow release JTBD decodes");
    let episode = synthetic_episode(&package);
    let records: Vec<CalibrationRecord> =
        calibration_records_from_learning_episode(&package, &episode)
            .expect("episode produces records")
            .into_iter()
            .enumerate()
            .map(|(index, record)| match index % 3 {
                0 => record.accepted("operator accepted"),
                1 => record.with_status(CalibrationStatus::Rejected, "operator rejected"),
                _ => record.with_status(CalibrationStatus::Reset, "operator reset"),
            })
            .collect();
    let table = CalibrationTable::new(records);
    let accepted_count = table
        .records
        .iter()
        .filter(|record| record.status == CalibrationStatus::Accepted)
        .count();
    assert!(
        accepted_count > 0,
        "fixture must contain at least one Accepted record for the test to be meaningful",
    );

    let replayed =
        CalibrationTable::from_jsonl(&table.to_jsonl()).expect("mixed-status table round trips");
    let enriched = apply_decoder_calibration(package, &replayed, DOMAIN_HINT)
        .expect("mixed-status apply succeeds");

    assert_eq!(
        enriched.artifacts.calibration_suggestions.len(),
        accepted_count
    );
}

#[test]
fn from_jsonl_rejects_invalid_line_with_typed_error() {
    let bad_input = "{\"record_id\":\"calibration_record.x\"}\nnot json\n";

    let err = CalibrationTable::from_jsonl(bad_input)
        .expect_err("malformed JSONL surfaces as a typed error");

    match err {
        CalibrationPersistenceError::InvalidLine {
            line_number,
            message,
        } => {
            // Line 1 is itself incomplete (missing required fields), so the
            // parser fails there before reaching the garbage on line 2.
            // Either failure point is acceptable; both must carry the
            // 1-based line number.
            assert!(
                line_number == 1 || line_number == 2,
                "expected line 1 or 2, got {line_number}",
            );
            assert!(
                !message.is_empty(),
                "InvalidLine should carry the underlying parse error",
            );
        }
        other @ CalibrationPersistenceError::DuplicateRecord { .. } => {
            panic!("expected InvalidLine, got {other:?}")
        }
    }
}

#[test]
fn from_jsonl_rejects_duplicate_record_id() {
    let (_, table) = tally_calibration_table();
    let single_line = table
        .to_jsonl()
        .lines()
        .next()
        .expect("table is non-empty")
        .to_string();
    let duplicated = format!("{single_line}\n{single_line}\n");

    let err =
        CalibrationTable::from_jsonl(&duplicated).expect_err("duplicate record ids are rejected");

    assert!(matches!(
        err,
        CalibrationPersistenceError::DuplicateRecord { .. }
    ));
}

#[test]
fn to_jsonl_canonicalizes_record_order_even_when_records_were_mutated() {
    let (_, canonical) = tally_calibration_table();
    let canonical_output = canonical.to_jsonl();

    // Reverse `records` after construction to bypass the sort inside
    // `CalibrationTable::new`. A naive `to_jsonl` that just iterates
    // `self.records` would emit the reversed order; the canonical contract
    // requires the same byte output regardless of in-memory order.
    let mut mutated = canonical.clone();
    mutated.records.reverse();
    assert_ne!(
        canonical.records, mutated.records,
        "fixture must actually be reversed for the test to be meaningful",
    );
    assert_eq!(mutated.to_jsonl(), canonical_output);

    // Constructing a `CalibrationTable` directly (bypassing `new`) must also
    // produce canonical output.
    let bypassed = CalibrationTable {
        records: mutated.records.clone(),
    };
    assert_eq!(bypassed.to_jsonl(), canonical_output);
}

#[test]
fn accept_marks_record_with_status_and_note() {
    let (_, mut table) = tally_calibration_table();
    // Revert the first record to Proposed so we can prove `accept` does the
    // transition, not just leave it alone.
    table.records[0].status = CalibrationStatus::Proposed;
    table.records[0].review_note = None;
    let record_id = table.records[0].record_id.clone();

    table
        .accept(&record_id, "policy validated against the live escrow gate")
        .expect("accept succeeds for a known record");

    assert_eq!(table.records[0].status, CalibrationStatus::Accepted);
    assert_eq!(
        table.records[0].review_note.as_deref(),
        Some("policy validated against the live escrow gate"),
    );
}

#[test]
fn reject_and_reset_each_mark_records_with_their_status_and_note() {
    let (_, mut table) = tally_calibration_table();
    let first = table.records[0].record_id.clone();
    let second = table.records[1].record_id.clone();

    table
        .reject(&first, "prior duplicates an existing accepted record")
        .unwrap();
    table
        .reset(&second, "prior is stale; let the next run re-propose it")
        .unwrap();

    assert_eq!(table.records[0].status, CalibrationStatus::Rejected);
    assert_eq!(table.records[1].status, CalibrationStatus::Reset);
    assert!(
        table.records[0]
            .review_note
            .as_deref()
            .is_some_and(|note| note.contains("duplicates"))
    );
    assert!(
        table.records[1]
            .review_note
            .as_deref()
            .is_some_and(|note| note.contains("stale"))
    );
}

#[test]
fn review_actions_require_a_non_empty_note() {
    let (_, mut table) = tally_calibration_table();
    let record_id = table.records[0].record_id.clone();

    let blank = table.accept(&record_id, "   ").unwrap_err();
    let empty = table.reject(&record_id, "").unwrap_err();
    let tab_only = table.reset(&record_id, "\t\n").unwrap_err();

    for (err, expected_status) in [
        (blank, CalibrationStatus::Accepted),
        (empty, CalibrationStatus::Rejected),
        (tab_only, CalibrationStatus::Reset),
    ] {
        match err {
            CalibrationReviewError::EmptyNote { status, .. } => {
                assert_eq!(status, expected_status);
            }
            other @ CalibrationReviewError::RecordNotFound { .. } => {
                panic!("expected EmptyNote, got {other:?}")
            }
        }
    }
}

#[test]
fn review_rejects_unknown_record_id() {
    let (_, mut table) = tally_calibration_table();
    let unknown = ArtifactId::new("calibration_record.does-not-exist");

    let err = table
        .reject(&unknown, "this id is not in the table")
        .unwrap_err();

    assert!(matches!(err, CalibrationReviewError::RecordNotFound { .. }));
}

#[test]
fn review_can_change_a_previously_accepted_record() {
    let (_, mut table) = tally_calibration_table();
    let record_id = table.records[0].record_id.clone();
    // `tally_calibration_table` accepts every record up front, so the
    // baseline is Accepted.
    assert_eq!(table.records[0].status, CalibrationStatus::Accepted);

    table
        .reject(&record_id, "regression discovered after a later episode")
        .unwrap();

    assert_eq!(table.records[0].status, CalibrationStatus::Rejected);
    assert_eq!(
        table.records[0].review_note.as_deref(),
        Some("regression discovered after a later episode"),
    );
}

#[test]
fn reviewed_table_round_trips_through_jsonl_with_status_and_note_preserved() {
    let (_, mut table) = tally_calibration_table();
    let first = table.records[0].record_id.clone();
    let second = table.records[1].record_id.clone();
    table.reject(&first, "duplicate prior").unwrap();
    table.reset(&second, "stale prior").unwrap();

    let serialized = table.to_jsonl();
    let replayed = CalibrationTable::from_jsonl(&serialized).expect("reviewed table round trips");

    assert_eq!(table, replayed);
}

#[test]
fn empty_table_round_trips_through_jsonl() {
    let empty = CalibrationTable::new(Vec::new());
    assert_eq!(empty.to_jsonl(), "");

    let parsed = CalibrationTable::from_jsonl("").expect("empty input parses");
    assert_eq!(empty, parsed);

    let with_blank_lines =
        CalibrationTable::from_jsonl("\n\n   \n").expect("blank lines are ignored, not errors");
    assert_eq!(empty, with_blank_lines);
}

fn synthetic_episode(package: &TruthPackage) -> LearningEpisode {
    let observation = synthetic_satisfied_observation(package);
    let report = AxiomRunReport::verify(package, observation);
    let audit = report
        .audit_fact_lineage(package)
        .expect("synthetic observation audits cleanly");
    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);

    LearningEpisode::from_report(
        "calibration-persistence-fixture",
        DOMAIN_HINT,
        package,
        &report,
        &audit,
    )
}

fn synthetic_satisfied_observation(package: &TruthPackage) -> AxiomRunObservation {
    let mut promoted_facts: Vec<PromotedFactRecord> = package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .map(|clause| build_fact(&format!("fact.{}", clause.key), vec![clause.id.clone()]))
        .collect();
    let first_failure = package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .next()
        .expect("escrow JTBD has at least one failure mode");
    promoted_facts.push(build_fact(
        "fact.failure-guard.double-release",
        vec![first_failure.id.clone()],
    ));

    AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:persistence-fixture", 6, 6),
        replay_notes: vec!["synthetic persistence fixture".to_string()],
        run_stages: Vec::new(),
    }
}

fn build_fact(fact_id: &str, source_clause_ids: Vec<ClauseId>) -> PromotedFactRecord {
    PromotedFactRecord {
        context_key: "Evidence".to_string(),
        fact_id: fact_id.to_string(),
        summary: format!("synthetic observation for {fact_id}"),
        source_clause_ids,
        evidence_refs: vec![EvidenceRefRecord {
            evidence_id: format!("evidence.{fact_id}"),
            source: "calibration-persistence-fixture".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace.{fact_id}"),
            location: Some("fixture://calibration-persistence".to_string()),
            replayable: true,
        }),
        promotion_authority: None,
    }
}

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
