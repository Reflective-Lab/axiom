//! Plumb Execution — eighth app probe for the Axiom/Helm contract.
//!
//! Warden and Triage pushed Helm toward decision, approval, publication, and
//! plan receipts. Plumb pressures the execution boundary: strategy drift can
//! be detected and corrective revisions proposed, but the strategy anchor must
//! advance by governed versioned commit rather than silent rewrite.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const PLUMB_ANCHOR_TRUTH_KEY: &str = "strategy-anchor-versioned";
const PLUMB_SIGNALS_TRUTH_KEY: &str = "source-signals-cited";
const PLUMB_DRIFT_TRUTH_KEY: &str = "drift-verdict-traced";
const PLUMB_MATERIALITY_TRUTH_KEY: &str = "materiality-threshold-applied";
const PLUMB_PROPOSALS_TRUTH_KEY: &str = "revision-proposals-named";
const PLUMB_REJECTED_PATH_TRUTH_KEY: &str = "rejected-path-preserved";
const PLUMB_FEASIBILITY_TRUTH_KEY: &str = "feasibility-reviewed";
const PLUMB_ADVERSARIAL_TRUTH_KEY: &str = "adversarial-review-recorded";
const PLUMB_APPROVAL_TRUTH_KEY: &str = "sponsor-approval-recorded";
const PLUMB_PROMOTION_TRUTH_KEY: &str = "promotion-gate-recorded";
const PLUMB_COMMIT_TRUTH_KEY: &str = "anchor-revision-committed";
const PLUMB_RECONCILIATION_TRUTH_KEY: &str = "honest-stop-or-reconciliation";
const PLUMB_ADAPTER_ID: &str = "plumb-execution.strategy-revision-to-axiom-observation";
const PLUMB_ADAPTER_VERSION: &str = "fixture.v0.1";
const PLUMB_STRATEGY_REVISION_TRANSCRIPT: &str =
    include_str!("fixtures/plumb_strategy_revision_transcript.json");

fn plumb_strategy_revision_jtbd() -> JtbdInput {
    JtbdInput {
        key: "plumb-strategy-revision".to_string(),
        actor: "strategy sponsor".to_string(),
        functional_job:
            "close a strategy execution loop by committing a governed revision after material drift"
                .to_string(),
        so_that:
            "the organization can restore alignment without silently rewriting the strategy anchor"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "strategy_anchor_versioned",
                "the strategy anchor names current version, proposed version, objectives, assumptions, forbidden moves, and source hash",
            ),
            ClauseInput::with_key(
                "source_signals_cited",
                "operating signals cite source systems, evidence refs, source hashes, metrics, periods, and anchored targets",
            ),
            ClauseInput::with_key(
                "drift_verdict_traced",
                "the drift verdict traces to source signals, anchored target, changed assumption, and evidence refs",
            ),
            ClauseInput::with_key(
                "materiality_threshold_applied",
                "the drift verdict applies a named materiality threshold and shows the observed gap exceeds it",
            ),
            ClauseInput::with_key(
                "revision_proposals_named",
                "correction proposals name the assumption or target they change and cite supporting evidence",
            ),
            ClauseInput::with_key(
                "rejected_path_preserved",
                "at least one rejected correction path and its rejection reason remain visible",
            ),
            ClauseInput::with_key(
                "feasibility_reviewed",
                "the accepted correction path has feasibility evidence and rejected path references",
            ),
            ClauseInput::with_key(
                "adversarial_review_recorded",
                "adversarial review records counterarguments, surviving objections, and reviewer roles",
            ),
            ClauseInput::with_key(
                "sponsor_approval_state",
                "sponsor approval is recorded before the strategy revision is treated as commit-ready",
            ),
            ClauseInput::with_key(
                "promotion_gate_recorded",
                "promotion gate, policy hash, approved proposal, and integrity hash are recorded",
            ),
            ClauseInput::with_key(
                "anchor_revision_committed",
                "the strategy anchor commit receipt names previous version, new version, anchor hash, rollback ref, and mutation mode",
            ),
            ClauseInput::with_key(
                "honest_stop_or_reconciliation",
                "the run either reconciles strategy and reality visibly or records an honest stop reason",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "target_without_anchor",
                "a target or assumption is changed without being present in the versioned strategy anchor",
            ),
            ClauseInput::with_key(
                "drift_without_signal",
                "a drift verdict is produced without source signals and evidence refs",
            ),
            ClauseInput::with_key(
                "drift_without_materiality_threshold",
                "material drift is claimed without a threshold and observed gap",
            ),
            ClauseInput::with_key(
                "proposal_without_assumption",
                "a revision proposal does not name the assumption or target it changes",
            ),
            ClauseInput::with_key(
                "rejected_path_hidden",
                "a rejected correction path or rejection rationale is hidden from review",
            ),
            ClauseInput::with_key(
                "forbidden_move_reintroduced",
                "a revision reintroduces a forbidden move without explicitly amending the strategy",
            ),
            ClauseInput::with_key(
                "infeasible_revision_committed",
                "an infeasible revision is committed despite resource or roadmap review failure",
            ),
            ClauseInput::with_key(
                "revision_without_adversarial_review",
                "a revision commits without adversarial review and surviving counterarguments",
            ),
            ClauseInput::with_key(
                "commit_without_sponsor_approval",
                "a strategy revision is treated as commit-ready without sponsor approval",
            ),
            ClauseInput::with_key(
                "promotion_gate_missing",
                "a strategy revision commits without a promotion gate and integrity record",
            ),
            ClauseInput::with_key(
                "anchor_mutated_in_place",
                "the strategy anchor is overwritten in place instead of committing a new version",
            ),
            ClauseInput::with_key(
                "unreconciled_divergence_hidden",
                "reality and strategy remain divergent but the run hides the stop reason or disagreement",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(40)),
    }
}

#[test]
fn plumb_strategy_revision_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Close a strategy execution loop")
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
            .any(|evidence| evidence.contains("materiality threshold"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("without sponsor approval"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn plumb_strategy_revision_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");
    let transcript = plumb_strategy_revision_transcript();

    let observation =
        adapt_plumb_strategy_revision_transcript(&package, &transcript).expect("Plumb adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.plumb-strategy-revision")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Plumb-adapted strategy revision preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 12);
    assert_eq!(audit.failure_coverage.len(), 12);
    assert_eq!(audit.facts_audited, 12);
}

#[test]
fn plumb_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");
    let transcript = plumb_strategy_revision_transcript();

    let first = adapt_plumb_strategy_revision_transcript_with_receipt(&package, &transcript);
    let second = adapt_plumb_strategy_revision_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, PLUMB_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "plumb-execution");
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
            "plumb.execution.strategy-anchor",
            "plumb.execution.source-signals",
            "plumb.execution.drift-verdict",
            "plumb.execution.materiality",
            "plumb.execution.revision-proposals",
            "plumb.execution.rejected-path",
            "plumb.execution.feasibility",
            "plumb.execution.adversarial-review",
            "plumb.execution.sponsor-approval",
            "plumb.execution.promotion-gate",
            "plumb.execution.anchor-commit",
            "plumb.execution.reconciliation",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 24);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("strategy.sponsor.growth"));
    assert!(!serialized.contains("embassy://crm/report"));
    assert!(!serialized.contains("discount below floor"));
    assert!(!serialized.contains("PLUMB_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/plumb-execution"));
}

#[test]
fn plumb_job_readiness_packet_marks_missing_source_signals() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");
    let mut transcript = plumb_strategy_revision_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != PLUMB_SIGNALS_TRUTH_KEY);
    transcript.execution_run.signals.clear();
    let adapter_outcome =
        adapt_plumb_strategy_revision_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let signals_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "source_signals_cited")
        .expect("source signals evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(signals_status.status, EvidenceReadinessStatus::Missing);
    assert!(signals_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for source_signals_cited".to_string())
    );
}

#[test]
fn plumb_job_readiness_packet_marks_commit_without_sponsor_approval() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");
    let mut transcript = plumb_strategy_revision_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != PLUMB_APPROVAL_TRUTH_KEY);
    transcript.execution_run.approval.status = "Pending".to_string();
    let adapter_outcome =
        adapt_plumb_strategy_revision_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "sponsor_approval_state")
        .expect("sponsor approval evidence is represented");
    let commit_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "anchor_revision_committed")
        .expect("anchor commit evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(commit_status.status, EvidenceReadinessStatus::Present);
    assert!(approval_status.fact_ids.is_empty());
}

#[test]
fn plumb_operator_ledger_entries_are_deterministic_backlinks_without_strategy_authority() {
    let package = decode_jtbd(plumb_strategy_revision_jtbd()).expect("Plumb JTBD decodes");
    let transcript = plumb_strategy_revision_transcript();
    let adapter_outcome =
        adapt_plumb_strategy_revision_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let drift_receipt = drift_verdict_receipt(&packet, &transcript);
    let revision_receipt = revision_proposal_receipt(&packet, &transcript, &drift_receipt);
    let approval_receipt = sponsor_approval_receipt(&packet, &transcript, &revision_receipt);
    let commit_receipt =
        strategy_commit_receipt(&packet, &transcript, &revision_receipt, &approval_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &drift_receipt,
        &revision_receipt,
        &approval_receipt,
        &commit_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &drift_receipt,
        &revision_receipt,
        &approval_receipt,
        &commit_receipt,
    );

    assert_eq!(first, second);
    assert_eq!(first.len(), 6);
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
        HelmLedgerRecordKind::DriftVerdictReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::RevisionProposalReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::SponsorApprovalReceipt
    );
    assert_eq!(
        first[5].record_kind,
        HelmLedgerRecordKind::StrategyCommitReceipt
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );
    assert_eq!(
        first[5].backlink_ids,
        vec![
            packet.packet_id.as_str().to_string(),
            revision_receipt.receipt_id.clone(),
            approval_receipt.receipt_id.clone(),
        ]
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("strategy.sponsor.growth"));
    assert!(!serialized.contains("embassy://crm/report"));
    assert!(!serialized.contains("discount below floor"));
    assert!(!serialized.contains("PLUMB_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/plumb-execution"));
}

fn adapt_plumb_strategy_revision_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &PlumbStrategyRevisionTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_plumb_strategy_revision_transcript(package, transcript) {
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
    transcript: &PlumbStrategyRevisionTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: PLUMB_ADAPTER_ID.to_string(),
        adapter_version: PLUMB_ADAPTER_VERSION.to_string(),
        status,
        source_app: "plumb-execution".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!(
            "plumb://strategy-revision/{}/{}",
            transcript.source.run_id, transcript.execution_run.execution_run_id
        ),
        source_transcript_hash,
        package_id: package.package_id.clone(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        observation_hash,
        mapped_fact_ids,
        mapped_clause_ids,
        dropped_source_fields: vec![
            "strategy_anchor.owner".to_string(),
            "strategy_anchor.forbidden_moves".to_string(),
            "signals.evidence_ref".to_string(),
            "revision_proposals.summary".to_string(),
            "approval.approver_id".to_string(),
            "source.command".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &PlumbStrategyRevisionTranscript,
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
        &transcript.execution_run.execution_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: format!(
            "plumb://strategy-revision/{}",
            transcript.execution_run.execution_run_id
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
    actions
        .push("review rejected correction paths before committing strategy revision".to_string());
    actions.push("commit strategy only through sponsor approval and promotion gate".to_string());
    actions
}

fn drift_verdict_receipt(
    packet: &JobReadinessPacket,
    transcript: &PlumbStrategyRevisionTranscript,
) -> DriftVerdictReceipt {
    let verdict = &transcript.execution_run.drift_verdict;
    DriftVerdictReceipt {
        receipt_id: drift_verdict_receipt_id(packet, verdict),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        strategy_run_ref: packet.subject_ref.clone(),
        verdict_ref_hash: sha256_lines(&[verdict.verdict_id.as_str()]),
        status: verdict.status.clone(),
        signal_count: verdict.signal_ids.len(),
        evidence_hash: sha256_json(&verdict.evidence_refs),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn revision_proposal_receipt(
    packet: &JobReadinessPacket,
    transcript: &PlumbStrategyRevisionTranscript,
    drift_receipt: &DriftVerdictReceipt,
) -> RevisionProposalReceipt {
    let accepted = transcript
        .execution_run
        .revision_proposals
        .iter()
        .find(|proposal| proposal.status == "Accepted")
        .expect("fixture has accepted proposal");
    RevisionProposalReceipt {
        receipt_id: revision_proposal_receipt_id(packet, accepted, drift_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        accepted_proposal_ref_hash: sha256_lines(&[accepted.proposal_id.as_str()]),
        rejected_proposal_count: transcript
            .execution_run
            .revision_proposals
            .iter()
            .filter(|proposal| proposal.status == "Rejected")
            .count(),
        changed_assumption_ref_hash: sha256_lines(&[accepted.changes_assumption_id.as_str()]),
        drift_verdict_receipt_id: drift_receipt.receipt_id.clone(),
    }
}

fn sponsor_approval_receipt(
    packet: &JobReadinessPacket,
    transcript: &PlumbStrategyRevisionTranscript,
    revision_receipt: &RevisionProposalReceipt,
) -> SponsorApprovalReceipt {
    let approval = &transcript.execution_run.approval;
    SponsorApprovalReceipt {
        receipt_id: sponsor_approval_receipt_id(packet, approval, revision_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        approval_ref_hash: sha256_lines(&[approval.approval_id.as_str()]),
        status: approval.status.clone(),
        scope_hash: sha256_lines(&[approval.scope.as_str()]),
        note_hash: approval.note_hash.clone(),
        revision_proposal_receipt_id: revision_receipt.receipt_id.clone(),
    }
}

fn strategy_commit_receipt(
    packet: &JobReadinessPacket,
    transcript: &PlumbStrategyRevisionTranscript,
    revision_receipt: &RevisionProposalReceipt,
    approval_receipt: &SponsorApprovalReceipt,
) -> StrategyCommitReceipt {
    let commit = &transcript.execution_run.commit_receipt;
    StrategyCommitReceipt {
        receipt_id: strategy_commit_receipt_id(packet, commit, approval_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        previous_version: commit.previous_version.clone(),
        committed_version: commit.committed_version.clone(),
        status: commit.status.clone(),
        anchor_hash: commit.anchor_hash.clone(),
        rollback_ref_hash: sha256_lines(&[commit.rollback_ref.as_str()]),
        mutation_mode: commit.mutation_mode.clone(),
        revision_proposal_receipt_id: revision_receipt.receipt_id.clone(),
        sponsor_approval_receipt_id: approval_receipt.receipt_id.clone(),
    }
}

fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    drift_receipt: &DriftVerdictReceipt,
    revision_receipt: &RevisionProposalReceipt,
    approval_receipt: &SponsorApprovalReceipt,
    commit_receipt: &StrategyCommitReceipt,
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
            HelmLedgerRecordKind::DriftVerdictReceipt,
            drift_receipt.receipt_id.clone(),
            drift_receipt.package_id.clone(),
            drift_receipt.truth_version.clone(),
            drift_receipt.domain_hint.clone(),
            sha256_json(drift_receipt),
            vec![packet.packet_id.clone()],
            format!("drift verdict {}", drift_receipt.status),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::RevisionProposalReceipt,
            revision_receipt.receipt_id.clone(),
            revision_receipt.package_id.clone(),
            revision_receipt.truth_version.clone(),
            revision_receipt.domain_hint.clone(),
            sha256_json(revision_receipt),
            vec![packet.packet_id.clone(), drift_receipt.receipt_id.clone()],
            "revision proposal accepted with rejected paths preserved".to_string(),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::SponsorApprovalReceipt,
            approval_receipt.receipt_id.clone(),
            approval_receipt.package_id.clone(),
            approval_receipt.truth_version.clone(),
            approval_receipt.domain_hint.clone(),
            sha256_json(approval_receipt),
            vec![
                packet.packet_id.clone(),
                revision_receipt.receipt_id.clone(),
            ],
            format!("sponsor approval {}", approval_receipt.status),
        ),
        helm_ledger_entry(
            5,
            HelmLedgerRecordKind::StrategyCommitReceipt,
            commit_receipt.receipt_id.clone(),
            commit_receipt.package_id.clone(),
            commit_receipt.truth_version.clone(),
            commit_receipt.domain_hint.clone(),
            sha256_json(commit_receipt),
            vec![
                packet.packet_id.clone(),
                revision_receipt.receipt_id.clone(),
                approval_receipt.receipt_id.clone(),
            ],
            format!(
                "strategy commit {} -> {}",
                commit_receipt.previous_version, commit_receipt.committed_version
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

fn adapt_plumb_strategy_revision_transcript(
    package: &TruthPackage,
    transcript: &PlumbStrategyRevisionTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.execution_run;
    if run.status != "Converged" {
        return Err("expected Plumb execution run to converge before adaptation".to_string());
    }
    if run.strategy_anchor.objectives.is_empty() || run.strategy_anchor.assumptions.is_empty() {
        return Err(
            "expected Plumb run to carry strategy anchor objectives and assumptions".to_string(),
        );
    }
    if run.revision_proposals.is_empty() {
        return Err("expected Plumb run to carry revision proposals".to_string());
    }

    let strategy_anchor_versioned = evidence_clause_id(package, "strategy_anchor_versioned");
    let source_signals_cited = evidence_clause_id(package, "source_signals_cited");
    let drift_verdict_traced = evidence_clause_id(package, "drift_verdict_traced");
    let materiality_threshold_applied =
        evidence_clause_id(package, "materiality_threshold_applied");
    let revision_proposals_named = evidence_clause_id(package, "revision_proposals_named");
    let rejected_path_preserved = evidence_clause_id(package, "rejected_path_preserved");
    let feasibility_reviewed = evidence_clause_id(package, "feasibility_reviewed");
    let adversarial_review_recorded = evidence_clause_id(package, "adversarial_review_recorded");
    let sponsor_approval_state = evidence_clause_id(package, "sponsor_approval_state");
    let promotion_gate_recorded = evidence_clause_id(package, "promotion_gate_recorded");
    let anchor_revision_committed = evidence_clause_id(package, "anchor_revision_committed");
    let honest_stop_or_reconciliation =
        evidence_clause_id(package, "honest_stop_or_reconciliation");
    let target_without_anchor = failure_clause_id(package, "target_without_anchor");
    let drift_without_signal = failure_clause_id(package, "drift_without_signal");
    let drift_without_materiality_threshold =
        failure_clause_id(package, "drift_without_materiality_threshold");
    let proposal_without_assumption = failure_clause_id(package, "proposal_without_assumption");
    let rejected_path_hidden = failure_clause_id(package, "rejected_path_hidden");
    let forbidden_move_reintroduced = failure_clause_id(package, "forbidden_move_reintroduced");
    let infeasible_revision_committed = failure_clause_id(package, "infeasible_revision_committed");
    let revision_without_adversarial_review =
        failure_clause_id(package, "revision_without_adversarial_review");
    let commit_without_sponsor_approval =
        failure_clause_id(package, "commit_without_sponsor_approval");
    let promotion_gate_missing = failure_clause_id(package, "promotion_gate_missing");
    let anchor_mutated_in_place = failure_clause_id(package, "anchor_mutated_in_place");
    let unreconciled_divergence_hidden =
        failure_clause_id(package, "unreconciled_divergence_hidden");
    let mut promoted_facts = Vec::new();

    let target_ids = run
        .strategy_anchor
        .objectives
        .iter()
        .map(|objective| objective.target_id.as_str())
        .collect::<BTreeSet<_>>();
    let assumption_ids = run
        .strategy_anchor
        .assumptions
        .iter()
        .map(|assumption| assumption.assumption_id.as_str())
        .collect::<BTreeSet<_>>();
    let signal_ids = run
        .signals
        .iter()
        .map(|signal| signal.signal_id.as_str())
        .collect::<BTreeSet<_>>();
    let accepted_proposal_ids = run
        .revision_proposals
        .iter()
        .filter(|proposal| proposal.status == "Accepted")
        .map(|proposal| proposal.proposal_id.as_str())
        .collect::<BTreeSet<_>>();

    if has_truth_key(&run.truth_keys, PLUMB_ANCHOR_TRUTH_KEY)
        && run.strategy_anchor.current_version != run.strategy_anchor.proposed_version
        && run.strategy_anchor.source_hash.starts_with("sha256:")
        && !run.strategy_anchor.objectives.is_empty()
        && !run.strategy_anchor.assumptions.is_empty()
        && !run.strategy_anchor.forbidden_moves.is_empty()
    {
        promoted_facts.push(plumb_fact(
            "StrategyAnchor",
            "plumb.execution.strategy-anchor",
            "strategy anchor is versioned and names objectives, assumptions, forbidden moves, and source hash",
            vec![strategy_anchor_versioned, target_without_anchor],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_SIGNALS_TRUTH_KEY)
        && run.signals.iter().all(|signal| {
            target_ids.contains(signal.anchored_target_id.as_str())
                && !signal.source_system.trim().is_empty()
                && !signal.evidence_ref.trim().is_empty()
                && signal.source_hash.starts_with("sha256:")
        })
    {
        promoted_facts.push(plumb_fact(
            "SourceSignals",
            "plumb.execution.source-signals",
            "operating signals cite source systems, evidence refs, source hashes, metrics, periods, and anchored targets",
            vec![source_signals_cited, drift_without_signal.clone()],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_DRIFT_TRUTH_KEY)
        && run.drift_verdict.status == "MaterialDrift"
        && target_ids.contains(run.drift_verdict.anchored_target_id.as_str())
        && assumption_ids.contains(run.drift_verdict.changed_assumption_id.as_str())
        && run
            .drift_verdict
            .signal_ids
            .iter()
            .all(|signal_id| signal_ids.contains(signal_id.as_str()))
        && !run.drift_verdict.evidence_refs.is_empty()
    {
        promoted_facts.push(plumb_fact(
            "DriftVerdict",
            "plumb.execution.drift-verdict",
            "drift verdict traces to source signals, anchored target, changed assumption, and evidence refs",
            vec![drift_verdict_traced, drift_without_signal.clone()],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_MATERIALITY_TRUTH_KEY)
        && run.materiality.threshold_basis_points > 0
        && run.materiality.observed_gap_basis_points > run.materiality.threshold_basis_points
        && run.materiality.status == "Exceeded"
        && !run.materiality.rule_ref.trim().is_empty()
    {
        promoted_facts.push(plumb_fact(
            "Materiality",
            "plumb.execution.materiality",
            "materiality rule is named and observed gap exceeds the threshold",
            vec![
                materiality_threshold_applied,
                drift_without_materiality_threshold,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_PROPOSALS_TRUTH_KEY)
        && run.revision_proposals.iter().all(|proposal| {
            assumption_ids.contains(proposal.changes_assumption_id.as_str())
                && target_ids.contains(proposal.changes_target_id.as_str())
                && !proposal.evidence_refs.is_empty()
        })
    {
        promoted_facts.push(plumb_fact(
            "RevisionProposals",
            "plumb.execution.revision-proposals",
            "revision proposals name changed assumptions and targets and cite supporting evidence",
            vec![revision_proposals_named, proposal_without_assumption],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_REJECTED_PATH_TRUTH_KEY)
        && run
            .revision_proposals
            .iter()
            .any(|proposal| proposal.status == "Rejected" && proposal.forbidden_move_violated)
    {
        promoted_facts.push(plumb_fact(
            "RejectedPath",
            "plumb.execution.rejected-path",
            "rejected correction path and forbidden-move rationale remain visible",
            vec![
                rejected_path_preserved,
                rejected_path_hidden,
                forbidden_move_reintroduced,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_FEASIBILITY_TRUTH_KEY)
        && run.feasibility_review.status == "Feasible"
        && accepted_proposal_ids.contains(run.feasibility_review.accepted_proposal_id.as_str())
        && !run.feasibility_review.rejected_proposal_ids.is_empty()
        && !run.feasibility_review.evidence_refs.is_empty()
    {
        promoted_facts.push(plumb_fact(
            "Feasibility",
            "plumb.execution.feasibility",
            "accepted correction path has feasibility evidence and rejected path references",
            vec![feasibility_reviewed, infeasible_revision_committed],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_ADVERSARIAL_TRUTH_KEY)
        && run.adversarial_review.status == "Recorded"
        && !run.adversarial_review.counterarguments.is_empty()
        && !run.adversarial_review.surviving_counterarguments.is_empty()
        && !run.adversarial_review.reviewer_roles.is_empty()
    {
        promoted_facts.push(plumb_fact(
            "AdversarialReview",
            "plumb.execution.adversarial-review",
            "adversarial review records counterarguments, surviving objections, and reviewer roles",
            vec![
                adversarial_review_recorded,
                revision_without_adversarial_review,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_APPROVAL_TRUTH_KEY)
        && run.approval.status == "Approved"
        && run.approval.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(plumb_fact(
            "SponsorApproval",
            "plumb.execution.sponsor-approval",
            "sponsor approval is recorded before the strategy revision is commit-ready",
            vec![sponsor_approval_state, commit_without_sponsor_approval],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_PROMOTION_TRUTH_KEY)
        && run.promotion_gate.status == "Promoted"
        && run.promotion_gate.gate_id == run.promotion_authority.gate_id
        && accepted_proposal_ids.contains(run.promotion_gate.approved_proposal_id.as_str())
        && run.promotion_gate.integrity_hash.starts_with("sha256:")
    {
        promoted_facts.push(plumb_fact(
            "PromotionGate",
            "plumb.execution.promotion-gate",
            "promotion gate records policy hash, approved proposal, and integrity hash",
            vec![promotion_gate_recorded, promotion_gate_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_COMMIT_TRUTH_KEY)
        && run.commit_receipt.status == "Committed"
        && run.commit_receipt.previous_version == run.strategy_anchor.current_version
        && run.commit_receipt.committed_version == run.strategy_anchor.proposed_version
        && run.commit_receipt.anchor_hash.starts_with("sha256:")
        && !run.commit_receipt.rollback_ref.trim().is_empty()
        && run.commit_receipt.mutation_mode == "new_version"
    {
        promoted_facts.push(plumb_fact(
            "AnchorCommit",
            "plumb.execution.anchor-commit",
            "strategy commit receipt names previous version, new version, anchor hash, rollback ref, and new-version mutation mode",
            vec![anchor_revision_committed, anchor_mutated_in_place],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, PLUMB_RECONCILIATION_TRUTH_KEY)
        && !run.reconciliation.silent_override_attempted
        && run.reconciliation.unreconciled_divergence_visible
        && (run.reconciliation.status == "Reconciled"
            || !run.reconciliation.honest_stop_reason.trim().is_empty())
    {
        promoted_facts.push(plumb_fact(
            "Reconciliation",
            "plumb.execution.reconciliation",
            "run visibly reconciles strategy and reality or records an honest stop reason",
            vec![
                honest_stop_or_reconciliation,
                unreconciled_divergence_hidden,
            ],
            &run.promotion_authority,
        ));
    }

    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle("sha256:plumb-strategy-revision", 37, 12),
        replay_notes: vec![
            format!(
                "adapted Plumb strategy revision {} into AxiomRunObservation",
                run.execution_run_id
            ),
            format!(
                "source run {} captured at {}",
                transcript.source.run_id, transcript.source.captured_at
            ),
        ],
        run_stages: Vec::new(),
    })
}

fn plumb_fact(
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
            evidence_id: format!("plumb.evidence.{fact_id}"),
            source: "plumb-strategy-revision-adapter".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("plumb.trace.{fact_id}"),
            location: Some("plumb://strategy-revision".to_string()),
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
    execution_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    short_id(
        &sha256_lines(&[
            "job_readiness_packet",
            package.package_id.as_str(),
            package.truth_version.as_str(),
            domain_hint,
            execution_run_id,
            adapter_receipt_id,
        ]),
        "helm.job_readiness",
    )
}

fn drift_verdict_receipt_id(packet: &JobReadinessPacket, verdict: &PlumbDriftVerdict) -> String {
    short_id(
        &sha256_lines(&[
            "drift_verdict_receipt",
            packet.packet_id.as_str(),
            verdict.verdict_id.as_str(),
            verdict.status.as_str(),
        ]),
        "helm.drift_verdict",
    )
}

fn revision_proposal_receipt_id(
    packet: &JobReadinessPacket,
    proposal: &PlumbRevisionProposal,
    drift_receipt: &DriftVerdictReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "revision_proposal_receipt",
            packet.packet_id.as_str(),
            proposal.proposal_id.as_str(),
            drift_receipt.receipt_id.as_str(),
        ]),
        "helm.revision_proposal",
    )
}

fn sponsor_approval_receipt_id(
    packet: &JobReadinessPacket,
    approval: &PlumbApproval,
    revision_receipt: &RevisionProposalReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "sponsor_approval_receipt",
            packet.packet_id.as_str(),
            approval.approval_id.as_str(),
            approval.status.as_str(),
            revision_receipt.receipt_id.as_str(),
        ]),
        "helm.sponsor_approval",
    )
}

fn strategy_commit_receipt_id(
    packet: &JobReadinessPacket,
    commit: &PlumbCommitReceipt,
    approval_receipt: &SponsorApprovalReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "strategy_commit_receipt",
            packet.packet_id.as_str(),
            commit.receipt_id.as_str(),
            commit.committed_version.as_str(),
            approval_receipt.receipt_id.as_str(),
        ]),
        "helm.strategy_commit",
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
struct DriftVerdictReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    strategy_run_ref: String,
    verdict_ref_hash: String,
    status: String,
    signal_count: usize,
    evidence_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct RevisionProposalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    accepted_proposal_ref_hash: String,
    rejected_proposal_count: usize,
    changed_assumption_ref_hash: String,
    drift_verdict_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct SponsorApprovalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    approval_ref_hash: String,
    status: String,
    scope_hash: String,
    note_hash: String,
    revision_proposal_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct StrategyCommitReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    previous_version: String,
    committed_version: String,
    status: String,
    anchor_hash: String,
    rollback_ref_hash: String,
    mutation_mode: String,
    revision_proposal_receipt_id: String,
    sponsor_approval_receipt_id: String,
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
    DriftVerdictReceipt,
    RevisionProposalReceipt,
    SponsorApprovalReceipt,
    StrategyCommitReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::DriftVerdictReceipt => "drift_verdict_receipt",
            Self::RevisionProposalReceipt => "revision_proposal_receipt",
            Self::SponsorApprovalReceipt => "sponsor_approval_receipt",
            Self::StrategyCommitReceipt => "strategy_commit_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbStrategyRevisionTranscript {
    source: PlumbRunSource,
    execution_run: PlumbExecutionRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbExecutionRun {
    execution_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    strategy_anchor: PlumbStrategyAnchor,
    signals: Vec<PlumbSignal>,
    materiality: PlumbMateriality,
    drift_verdict: PlumbDriftVerdict,
    revision_proposals: Vec<PlumbRevisionProposal>,
    feasibility_review: PlumbFeasibilityReview,
    adversarial_review: PlumbAdversarialReview,
    approval: PlumbApproval,
    promotion_gate: PlumbPromotionGate,
    commit_receipt: PlumbCommitReceipt,
    reconciliation: PlumbReconciliation,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbStrategyAnchor {
    anchor_id: String,
    current_version: String,
    proposed_version: String,
    source_hash: String,
    owner: String,
    objectives: Vec<PlumbObjective>,
    assumptions: Vec<PlumbAssumption>,
    forbidden_moves: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbObjective {
    objective_id: String,
    target_id: String,
    target_metric: String,
    target_value: u64,
    period: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbAssumption {
    assumption_id: String,
    statement: String,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbSignal {
    signal_id: String,
    source_system: String,
    metric: String,
    observed_value: u64,
    anchored_target_id: String,
    period: String,
    evidence_ref: String,
    source_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbMateriality {
    threshold_basis_points: u16,
    observed_gap_basis_points: u16,
    rule_ref: String,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbDriftVerdict {
    verdict_id: String,
    status: String,
    signal_ids: Vec<String>,
    anchored_target_id: String,
    changed_assumption_id: String,
    rationale: String,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbRevisionProposal {
    proposal_id: String,
    status: String,
    changes_assumption_id: String,
    changes_target_id: String,
    summary: String,
    forbidden_move_violated: bool,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbFeasibilityReview {
    review_id: String,
    accepted_proposal_id: String,
    status: String,
    resource_delta: String,
    roadmap_impact: String,
    rejected_proposal_ids: Vec<String>,
    evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbAdversarialReview {
    review_id: String,
    status: String,
    counterarguments: Vec<String>,
    surviving_counterarguments: Vec<String>,
    reviewer_roles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbApproval {
    approval_id: String,
    approver_id: String,
    status: String,
    scope: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbPromotionGate {
    gate_id: String,
    policy_version_hash: String,
    status: String,
    approved_proposal_id: String,
    integrity_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbCommitReceipt {
    receipt_id: String,
    previous_version: String,
    committed_version: String,
    status: String,
    committed_at: String,
    anchor_hash: String,
    rollback_ref: String,
    mutation_mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PlumbReconciliation {
    status: String,
    honest_stop_reason: String,
    unreconciled_divergence_visible: bool,
    silent_override_attempted: bool,
}

fn plumb_strategy_revision_transcript() -> PlumbStrategyRevisionTranscript {
    serde_json::from_str(PLUMB_STRATEGY_REVISION_TRANSCRIPT)
        .expect("Plumb strategy revision transcript parses")
}
