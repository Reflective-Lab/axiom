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
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservedStopReason, PromotedFactRecord, RunIntegrityProof,
    TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};

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
