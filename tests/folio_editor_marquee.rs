//! Folio Editor - eleventh and final app probe for the Axiom/Helm contract.
//!
//! The operational probes found readiness, approval, action, outcome, and
//! temporal-evidence receipts. Folio pressures the creative/content boundary:
//! public claims, citations, editorial approval, unresolved questions, and the
//! line between package preparation and publication authority.

use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunVerdict, ClauseId, ClauseInput, EvidenceRefRecord,
    JtbdClauseKind, JtbdInput, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    RunIntegrityProof, TimeBudget, TraceLinkRecord, TruthPackage, decode_jtbd,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fmt::Write as _};

const FOLIO_SNAPSHOT_TRUTH_KEY: &str = "canonical-story-snapshot-bound";
const FOLIO_BRIEF_TRUTH_KEY: &str = "story-brief-approved";
const FOLIO_SOURCE_MAP_TRUTH_KEY: &str = "source-map-cited";
const FOLIO_CLAIMS_TRUTH_KEY: &str = "public-claims-bounded";
const FOLIO_CITATIONS_TRUTH_KEY: &str = "claim-citations-attached";
const FOLIO_OPEN_QUESTIONS_TRUTH_KEY: &str = "unresolved-questions-visible";
const FOLIO_STANDARDS_TRUTH_KEY: &str = "standards-risk-review-recorded";
const FOLIO_APPROVAL_TRUTH_KEY: &str = "editorial-approval-recorded";
const FOLIO_BOUNDARY_TRUTH_KEY: &str = "publication-boundary-declared";
const FOLIO_EXPORT_TRUTH_KEY: &str = "package-export-recorded";
const FOLIO_ADAPTER_ID: &str = "folio-editor.publication-package-to-axiom-observation";
const FOLIO_ADAPTER_VERSION: &str = "fixture.v0.1";
const FOLIO_PUBLICATION_TRANSCRIPT: &str =
    include_str!("fixtures/folio_publication_package_transcript.json");

fn folio_publication_package_jtbd() -> JtbdInput {
    JtbdInput {
        key: "folio-publication-package".to_string(),
        actor: "edition editor".to_string(),
        functional_job:
            "prepare a public editorial package from a canonical story object without publishing beyond the approved boundary"
                .to_string(),
        so_that:
            "readers can receive cited, bounded claims while editors keep unresolved questions and publication authority visible"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key(
                "canonical_story_snapshot_bound",
                "the package names a canonical story snapshot, schema version, revision, story hash, and freeze hash",
            ),
            ClauseInput::with_key(
                "story_brief_approved",
                "the story brief records desk, angle hash, approval role, and approval timestamp",
            ),
            ClauseInput::with_key(
                "source_map_cited",
                "the source map names qualified source ids, permissions, anonymity policy, and source-map hash",
            ),
            ClauseInput::with_key(
                "public_claims_bounded",
                "every public claim has promoted status, statement hash, and headline-safe bounded wording",
            ),
            ClauseInput::with_key(
                "claim_citations_attached",
                "every public claim carries at least one citation ref that resolves to a cited source hash and permission",
            ),
            ClauseInput::with_key(
                "unresolved_questions_visible",
                "unresolved questions and contradictions remain visible in the package or next-update queue",
            ),
            ClauseInput::with_key(
                "standards_risk_review_recorded",
                "standards, legal risk, right-of-response, and reviewer note hash are recorded before package approval",
            ),
            ClauseInput::with_key(
                "editorial_approval_recorded",
                "an authorized editor approval gate records status, role, approval hash, and publish window",
            ),
            ClauseInput::with_key(
                "publication_boundary_declared",
                "the output declares whether it is a preview, CMS export, or public publish and whether live publication was invoked",
            ),
            ClauseInput::with_key(
                "package_export_recorded",
                "the prepared package records channel targets, package hash, and layout candidate hash",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "publish_without_snapshot",
                "a public package is prepared without a frozen canonical story snapshot",
            ),
            ClauseInput::with_key(
                "brief_unapproved",
                "a story is packaged without an approved brief and angle",
            ),
            ClauseInput::with_key(
                "source_permission_missing",
                "source permissions or anonymity policy are missing before public packaging",
            ),
            ClauseInput::with_key(
                "headline_overstates_verified_claims",
                "headline or public wording overstates the promoted claim set",
            ),
            ClauseInput::with_key(
                "claim_without_citation",
                "a public claim lacks a resolving citation and source hash",
            ),
            ClauseInput::with_key(
                "unresolved_question_hidden",
                "unresolved questions or contradictions are hidden from the package boundary",
            ),
            ClauseInput::with_key(
                "legal_or_standards_review_missing",
                "standards, legal risk, or right-of-response review is absent before approval",
            ),
            ClauseInput::with_key(
                "editor_approval_missing",
                "a package is exported or published without authorized editorial approval",
            ),
            ClauseInput::with_key(
                "public_publish_without_gate",
                "the run invokes public publication while only preview or export authority is present",
            ),
            ClauseInput::with_key(
                "cms_export_without_package_hash",
                "a CMS or print export is prepared without package and layout hashes",
            ),
        ],
        time_budget: Some(TimeBudget::from_minutes(35)),
    }
}

#[test]
fn folio_publication_package_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Prepare a public editorial package")
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
            .any(|evidence| evidence.contains("citation ref"))
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action.contains("public publication"))
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn folio_publication_transcript_adapts_to_satisfied_axiom_observation() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");
    let transcript = folio_publication_transcript();

    let observation =
        adapt_folio_publication_transcript(&package, &transcript).expect("Folio adapts");
    let report = AxiomRunReport::verify(&package, observation);

    assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
    assert!(report.expected_stop_reason_matched());
    assert!(report.promoted_facts.iter().all(|fact| {
        fact.promotion_authority
            .as_ref()
            .is_some_and(|authority| authority.gate_id == "converge.gate.folio-publication-package")
    }));

    let audit = report
        .audit_fact_lineage(&package)
        .expect("Folio-adapted publication package preserves clause-level custody");
    assert_eq!(audit.evidence_coverage.len(), 10);
    assert_eq!(audit.failure_coverage.len(), 10);
    assert_eq!(audit.facts_audited, 10);
}

#[test]
fn folio_observation_adapter_receipt_is_deterministic_and_app_neutral() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");
    let transcript = folio_publication_transcript();

    let first = adapt_folio_publication_transcript_with_receipt(&package, &transcript);
    let second = adapt_folio_publication_transcript_with_receipt(&package, &transcript);

    assert!(first.observation.is_some());
    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.receipt.status, ObservationAdapterStatus::Succeeded);
    assert_eq!(first.receipt.adapter_id, FOLIO_ADAPTER_ID);
    assert_eq!(first.receipt.source_app, "folio-editor");
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
            "folio.editorial.canonical-story",
            "folio.editorial.story-brief",
            "folio.editorial.source-map",
            "folio.editorial.public-claims",
            "folio.editorial.claim-citations",
            "folio.editorial.unresolved-questions",
            "folio.editorial.standards-review",
            "folio.editorial.editorial-approval",
            "folio.editorial.publication-boundary",
            "folio.editorial.package-export",
        ]
    );
    assert_eq!(first.receipt.mapped_clause_ids.len(), 20);
    assert!(first.receipt.errors.is_empty());

    let serialized = serde_json::to_string(&first.receipt).expect("receipt serializes");
    assert!(!serialized.contains("story-hellevik-pier"));
    assert!(!serialized.contains("claim-001-pier-application-filed"));
    assert!(!serialized.contains("solvesborg-kommun"));
    assert!(!serialized.contains("kb://notes/sources"));
    assert!(!serialized.contains("LEDE_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/folio-editor"));
}

#[test]
fn folio_job_readiness_packet_marks_missing_citations() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");
    let mut transcript = folio_publication_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != FOLIO_CITATIONS_TRUTH_KEY);
    for claim in &mut transcript.execution_run.claim_set {
        claim.citation_refs.clear();
    }
    let adapter_outcome = adapt_folio_publication_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let citation_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "claim_citations_attached")
        .expect("citation evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(citation_status.status, EvidenceReadinessStatus::Missing);
    assert!(citation_status.fact_ids.is_empty());
    assert!(
        packet
            .operator_actions
            .contains(&"request missing evidence for claim_citations_attached".to_string())
    );
}

#[test]
fn folio_job_readiness_packet_marks_missing_editorial_approval() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");
    let mut transcript = folio_publication_transcript();
    transcript
        .execution_run
        .truth_keys
        .retain(|truth_key| truth_key != FOLIO_APPROVAL_TRUTH_KEY);
    transcript.execution_run.editorial_approval.status = "pending".to_string();
    let adapter_outcome = adapt_folio_publication_transcript_with_receipt(&package, &transcript);

    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let approval_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "editorial_approval_recorded")
        .expect("editorial approval evidence is represented");
    let boundary_status = packet
        .evidence_status
        .iter()
        .find(|status| status.clause_key == "publication_boundary_declared")
        .expect("publication boundary evidence is represented");

    assert_eq!(packet.adapter_status, ObservationAdapterStatus::Succeeded);
    assert_eq!(packet.verdict, Some(AxiomRunVerdict::Invalid));
    assert!(!packet.authorizes_domain_action);
    assert_eq!(approval_status.status, EvidenceReadinessStatus::Missing);
    assert_eq!(boundary_status.status, EvidenceReadinessStatus::Present);
    assert!(approval_status.fact_ids.is_empty());
}

#[test]
fn folio_operator_ledger_entries_are_content_backlinks_without_publication_authority() {
    let package = decode_jtbd(folio_publication_package_jtbd()).expect("Folio JTBD decodes");
    let transcript = folio_publication_transcript();
    let adapter_outcome = adapt_folio_publication_transcript_with_receipt(&package, &transcript);
    let packet = job_readiness_packet(&package, &transcript, &adapter_outcome);
    let story_receipt = canonical_story_receipt(&packet, &transcript);
    let claim_receipt = claim_review_receipt(&packet, &transcript, &story_receipt);
    let approval_receipt = editorial_approval_receipt(&packet, &transcript, &claim_receipt);
    let boundary_receipt = publication_boundary_receipt(&packet, &transcript, &approval_receipt);

    let first = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &story_receipt,
        &claim_receipt,
        &approval_receipt,
        &boundary_receipt,
    );
    let second = job_readiness_ledger_entries(
        &adapter_outcome.receipt,
        &packet,
        &story_receipt,
        &claim_receipt,
        &approval_receipt,
        &boundary_receipt,
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
        HelmLedgerRecordKind::CanonicalStoryReceipt
    );
    assert_eq!(
        first[3].record_kind,
        HelmLedgerRecordKind::ClaimReviewReceipt
    );
    assert_eq!(
        first[4].record_kind,
        HelmLedgerRecordKind::EditorialApprovalReceipt
    );
    assert_eq!(
        first[5].record_kind,
        HelmLedgerRecordKind::PublicationBoundaryReceipt
    );
    assert!(
        first
            .iter()
            .all(|entry| entry.authority_effect == HelmLedgerAuthorityEffect::None)
    );
    assert_eq!(
        first[5].backlink_ids,
        vec![
            packet.packet_id.clone(),
            claim_receipt.receipt_id.clone(),
            approval_receipt.receipt_id.clone(),
        ]
    );

    let serialized = serde_json::to_string(&first).expect("ledger entries serialize");
    assert!(!serialized.contains("story-hellevik-pier"));
    assert!(!serialized.contains("claim-001-pier-application-filed"));
    assert!(!serialized.contains("solvesborg-kommun"));
    assert!(!serialized.contains("kb://notes/sources"));
    assert!(!serialized.contains("LEDE_OUTPUT=json"));
    assert!(!serialized.contains("/Users/kpernyer/dev/reflective/marquee-apps/folio-editor"));
    assert!(!serialized.contains("public_publish_invoked"));
}

fn adapt_folio_publication_transcript_with_receipt(
    package: &TruthPackage,
    transcript: &FolioPublicationTranscript,
) -> ObservationAdapterOutcome {
    let source_transcript_hash = sha256_json(transcript);

    match adapt_folio_publication_transcript(package, transcript) {
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
    transcript: &FolioPublicationTranscript,
    status: ObservationAdapterStatus,
    source_transcript_hash: String,
    observation_hash: Option<String>,
    mapped_fact_ids: Vec<String>,
    mapped_clause_ids: Vec<ClauseId>,
    errors: Vec<String>,
) -> ObservationAdapterReceipt {
    let transcript_ref_hash = sha256_lines(&[
        transcript.source.run_id.as_str(),
        transcript.execution_run.edition_run_id.as_str(),
    ]);
    ObservationAdapterReceipt::new(ObservationAdapterReceiptInput {
        adapter_id: FOLIO_ADAPTER_ID.to_string(),
        adapter_version: FOLIO_ADAPTER_VERSION.to_string(),
        status,
        source_app: "folio-editor".to_string(),
        source_run_id: transcript.source.run_id.clone(),
        source_transcript_ref: format!("folio://publication-package/{transcript_ref_hash}"),
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
            "canonical_story.story_id".to_string(),
            "canonical_story.thread_id".to_string(),
            "brief.brief_id".to_string(),
            "source_map.qualified_source_ids".to_string(),
            "claim_set.claim_id".to_string(),
            "claim_set.citation_refs".to_string(),
            "citations.locator".to_string(),
            "citations.source_id".to_string(),
            "unresolved_questions.question_id".to_string(),
        ],
        warnings: Vec::new(),
        errors,
        replay_notes: vec![format!("captured at {}", transcript.source.captured_at)],
    })
}

fn job_readiness_packet(
    package: &TruthPackage,
    transcript: &FolioPublicationTranscript,
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
        &transcript.execution_run.edition_run_id,
        adapter_outcome.receipt.receipt_id.as_str(),
    );

    JobReadinessPacket {
        packet_id,
        package_id: package.package_id.as_str().to_string(),
        truth_version: package.truth_version.clone(),
        domain_hint: transcript.source.domain_hint.clone(),
        job_key: package.source_jtbd.key.clone(),
        subject_ref: short_id(
            &sha256_lines(&[transcript.execution_run.edition_run_id.as_str()]),
            "folio.subject",
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
    actions.push("review citations and unresolved questions before publication".to_string());
    actions.push("keep preview or export separate from public publication authority".to_string());
    actions
}

fn canonical_story_receipt(
    packet: &JobReadinessPacket,
    transcript: &FolioPublicationTranscript,
) -> CanonicalStoryReceipt {
    let story = &transcript.execution_run.canonical_story;
    CanonicalStoryReceipt {
        receipt_id: canonical_story_receipt_id(packet, story),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        story_ref_hash: sha256_lines(&[story.story_id.as_str(), story.thread_id.as_str()]),
        schema_version: story.schema_version.clone(),
        revision: story.revision,
        claim_count: transcript.execution_run.claim_set.len(),
        unresolved_question_count: transcript.execution_run.unresolved_questions.len(),
        story_object_hash: story.story_object_hash.clone(),
        adapter_receipt_id: packet.adapter_receipt_id.clone(),
    }
}

fn claim_review_receipt(
    packet: &JobReadinessPacket,
    transcript: &FolioPublicationTranscript,
    story_receipt: &CanonicalStoryReceipt,
) -> ClaimReviewReceipt {
    let claims = &transcript.execution_run.claim_set;
    ClaimReviewReceipt {
        receipt_id: claim_review_receipt_id(packet, claims, story_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        claim_count: claims.len(),
        cited_claim_count: claims
            .iter()
            .filter(|claim| !claim.citation_refs.is_empty())
            .count(),
        contradiction_count: claims
            .iter()
            .map(|claim| claim.contradiction_refs.len())
            .sum(),
        unresolved_question_count: transcript.execution_run.unresolved_questions.len(),
        claim_set_hash: sha256_json(claims),
        unresolved_questions_hash: sha256_json(&transcript.execution_run.unresolved_questions),
        canonical_story_receipt_id: story_receipt.receipt_id.clone(),
    }
}

fn editorial_approval_receipt(
    packet: &JobReadinessPacket,
    transcript: &FolioPublicationTranscript,
    claim_receipt: &ClaimReviewReceipt,
) -> EditorialApprovalReceipt {
    let approval = &transcript.execution_run.editorial_approval;
    EditorialApprovalReceipt {
        receipt_id: editorial_approval_receipt_id(packet, approval, claim_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        gate_ref_hash: sha256_lines(&[approval.gate_id.as_str()]),
        status: approval.status.clone(),
        approver_role_hash: sha256_lines(&[approval.approver_role.as_str()]),
        approval_hash: approval.approval_hash.clone(),
        publish_window_hash: sha256_lines(&[approval.publish_window.as_str()]),
        claim_review_receipt_id: claim_receipt.receipt_id.clone(),
    }
}

fn publication_boundary_receipt(
    packet: &JobReadinessPacket,
    transcript: &FolioPublicationTranscript,
    approval_receipt: &EditorialApprovalReceipt,
) -> PublicationBoundaryReceipt {
    let boundary = &transcript.execution_run.publication_boundary;
    let export = &transcript.execution_run.package_export;
    PublicationBoundaryReceipt {
        receipt_id: publication_boundary_receipt_id(packet, boundary, export, approval_receipt),
        package_id: packet.package_id.clone(),
        truth_version: packet.truth_version.clone(),
        domain_hint: packet.domain_hint.clone(),
        output_kind: boundary.output_kind.clone(),
        cms_export_prepared: boundary.cms_export_prepared,
        public_publish_invoked: boundary.public_publish_invoked,
        reader_visible_ai_label: boundary.reader_visible_ai_label,
        channel_target_count: export.channel_targets.len(),
        package_hash: export.package_hash.clone(),
        boundary_ok: !boundary.public_publish_invoked && boundary.live_url.is_none(),
        editorial_approval_receipt_id: approval_receipt.receipt_id.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn job_readiness_ledger_entries(
    receipt: &ObservationAdapterReceipt,
    packet: &JobReadinessPacket,
    story_receipt: &CanonicalStoryReceipt,
    claim_receipt: &ClaimReviewReceipt,
    approval_receipt: &EditorialApprovalReceipt,
    boundary_receipt: &PublicationBoundaryReceipt,
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
            HelmLedgerRecordKind::CanonicalStoryReceipt,
            story_receipt.receipt_id.clone(),
            story_receipt.package_id.clone(),
            story_receipt.truth_version.clone(),
            story_receipt.domain_hint.clone(),
            sha256_json(story_receipt),
            vec![packet.packet_id.clone()],
            "canonical story snapshot recorded".to_string(),
        ),
        helm_ledger_entry(
            3,
            HelmLedgerRecordKind::ClaimReviewReceipt,
            claim_receipt.receipt_id.clone(),
            claim_receipt.package_id.clone(),
            claim_receipt.truth_version.clone(),
            claim_receipt.domain_hint.clone(),
            sha256_json(claim_receipt),
            vec![packet.packet_id.clone(), story_receipt.receipt_id.clone()],
            "claims, citations, and unresolved questions reviewed".to_string(),
        ),
        helm_ledger_entry(
            4,
            HelmLedgerRecordKind::EditorialApprovalReceipt,
            approval_receipt.receipt_id.clone(),
            approval_receipt.package_id.clone(),
            approval_receipt.truth_version.clone(),
            approval_receipt.domain_hint.clone(),
            sha256_json(approval_receipt),
            vec![packet.packet_id.clone(), claim_receipt.receipt_id.clone()],
            format!("editorial approval {}", approval_receipt.status),
        ),
        helm_ledger_entry(
            5,
            HelmLedgerRecordKind::PublicationBoundaryReceipt,
            boundary_receipt.receipt_id.clone(),
            boundary_receipt.package_id.clone(),
            boundary_receipt.truth_version.clone(),
            boundary_receipt.domain_hint.clone(),
            sha256_json(boundary_receipt),
            vec![
                packet.packet_id.clone(),
                claim_receipt.receipt_id.clone(),
                approval_receipt.receipt_id.clone(),
            ],
            format!("publication boundary {}", boundary_receipt.output_kind),
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

fn adapt_folio_publication_transcript(
    package: &TruthPackage,
    transcript: &FolioPublicationTranscript,
) -> Result<AxiomRunObservation, String> {
    let run = &transcript.execution_run;
    if run.status != "Converged" {
        return Err(
            "expected Folio publication package run to converge before adaptation".to_string(),
        );
    }
    if run.claim_set.is_empty() {
        return Err("expected Folio package to carry public claims".to_string());
    }

    let canonical_story_snapshot_bound =
        evidence_clause_id(package, "canonical_story_snapshot_bound");
    let story_brief_approved = evidence_clause_id(package, "story_brief_approved");
    let source_map_cited = evidence_clause_id(package, "source_map_cited");
    let public_claims_bounded = evidence_clause_id(package, "public_claims_bounded");
    let claim_citations_attached = evidence_clause_id(package, "claim_citations_attached");
    let unresolved_questions_visible = evidence_clause_id(package, "unresolved_questions_visible");
    let standards_risk_review_recorded =
        evidence_clause_id(package, "standards_risk_review_recorded");
    let editorial_approval_recorded = evidence_clause_id(package, "editorial_approval_recorded");
    let publication_boundary_declared =
        evidence_clause_id(package, "publication_boundary_declared");
    let package_export_recorded = evidence_clause_id(package, "package_export_recorded");
    let publish_without_snapshot = failure_clause_id(package, "publish_without_snapshot");
    let brief_unapproved = failure_clause_id(package, "brief_unapproved");
    let source_permission_missing = failure_clause_id(package, "source_permission_missing");
    let headline_overstates_verified_claims =
        failure_clause_id(package, "headline_overstates_verified_claims");
    let claim_without_citation = failure_clause_id(package, "claim_without_citation");
    let unresolved_question_hidden = failure_clause_id(package, "unresolved_question_hidden");
    let legal_or_standards_review_missing =
        failure_clause_id(package, "legal_or_standards_review_missing");
    let editor_approval_missing = failure_clause_id(package, "editor_approval_missing");
    let public_publish_without_gate = failure_clause_id(package, "public_publish_without_gate");
    let cms_export_without_package_hash =
        failure_clause_id(package, "cms_export_without_package_hash");
    let mut promoted_facts = Vec::new();

    if has_truth_key(&run.truth_keys, FOLIO_SNAPSHOT_TRUTH_KEY)
        && run.canonical_story.story_object_hash.starts_with("sha256:")
        && run.canonical_story.freeze_hash.starts_with("sha256:")
        && !run.canonical_story.schema_version.trim().is_empty()
    {
        promoted_facts.push(folio_fact(
            "CanonicalStory",
            "folio.editorial.canonical-story",
            "canonical story snapshot records schema, revision, story hash, and freeze hash",
            vec![canonical_story_snapshot_bound, publish_without_snapshot],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_BRIEF_TRUTH_KEY)
        && !run.brief.angle_hash.trim().is_empty()
        && !run.brief.approved_by_role.trim().is_empty()
        && !run.brief.approved_at.trim().is_empty()
    {
        promoted_facts.push(folio_fact(
            "StoryBrief",
            "folio.editorial.story-brief",
            "story brief records desk, angle hash, approval role, and approval timestamp",
            vec![story_brief_approved, brief_unapproved],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_SOURCE_MAP_TRUTH_KEY)
        && run.source_map.permissions_recorded
        && !run.source_map.qualified_source_ids.is_empty()
        && run.source_map.source_map_hash.starts_with("sha256:")
    {
        promoted_facts.push(folio_fact(
            "SourceMap",
            "folio.editorial.source-map",
            "source map records qualified source ids, permissions, anonymity policy, and hash",
            vec![source_map_cited, source_permission_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_CLAIMS_TRUTH_KEY)
        && run
            .claim_set
            .iter()
            .all(|claim| claim.status == "promoted" && claim.statement_hash.starts_with("sha256:"))
    {
        promoted_facts.push(folio_fact(
            "PublicClaims",
            "folio.editorial.public-claims",
            "public claims are promoted, statement-hashed, and bounded to verified wording",
            vec![public_claims_bounded, headline_overstates_verified_claims],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_CITATIONS_TRUTH_KEY) && all_claim_citations_resolve(run)
    {
        promoted_facts.push(folio_fact(
            "ClaimCitations",
            "folio.editorial.claim-citations",
            "every public claim resolves to cited sources with hashes and permissions",
            vec![claim_citations_attached, claim_without_citation],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_OPEN_QUESTIONS_TRUTH_KEY)
        && run
            .unresolved_questions
            .iter()
            .all(|question| question.status.starts_with("visible"))
    {
        promoted_facts.push(folio_fact(
            "UnresolvedQuestions",
            "folio.editorial.unresolved-questions",
            "unresolved questions and contradictions remain visible before publication",
            vec![unresolved_questions_visible, unresolved_question_hidden],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_STANDARDS_TRUTH_KEY)
        && run.standards_review.status.starts_with("approved")
        && !run.standards_review.reviewer_role.trim().is_empty()
        && run.standards_review.note_hash.starts_with("sha256:")
    {
        promoted_facts.push(folio_fact(
            "StandardsReview",
            "folio.editorial.standards-review",
            "standards, legal risk, right-of-response, and reviewer note are recorded",
            vec![
                standards_risk_review_recorded,
                legal_or_standards_review_missing,
            ],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_APPROVAL_TRUTH_KEY)
        && run.editorial_approval.status == "approved"
        && run.editorial_approval.approval_hash.starts_with("sha256:")
    {
        promoted_facts.push(folio_fact(
            "EditorialApproval",
            "folio.editorial.editorial-approval",
            "editorial approval records gate, role, approval hash, and publish window",
            vec![editorial_approval_recorded, editor_approval_missing],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_BOUNDARY_TRUTH_KEY)
        && run.publication_boundary.output_kind == "editorial_package_preview"
        && !run.publication_boundary.public_publish_invoked
        && run.publication_boundary.reader_visible_ai_label
    {
        promoted_facts.push(folio_fact(
            "PublicationBoundary",
            "folio.editorial.publication-boundary",
            "publication boundary declares preview/export status without live publication authority",
            vec![publication_boundary_declared, public_publish_without_gate],
            &run.promotion_authority,
        ));
    }

    if has_truth_key(&run.truth_keys, FOLIO_EXPORT_TRUTH_KEY)
        && run.publication_boundary.cms_export_prepared
        && run.package_export.package_hash.starts_with("sha256:")
        && run
            .package_export
            .layout_candidate_hash
            .starts_with("sha256:")
    {
        promoted_facts.push(folio_fact(
            "PackageExport",
            "folio.editorial.package-export",
            "package export records channel targets, package hash, and layout candidate hash",
            vec![package_export_recorded, cms_export_without_package_hash],
            &run.promotion_authority,
        ));
    }

    let fact_count = promoted_facts.len();
    Ok(AxiomRunObservation {
        stop_reason: ObservedStopReason::Converged,
        promoted_facts,
        integrity: RunIntegrityProof::sha256_merkle(
            run.canonical_story.freeze_hash.clone(),
            run.canonical_story.revision,
            fact_count,
        ),
        replay_notes: vec![
            format!("source run {}", transcript.source.run_id),
            format!("edition run {}", run.edition_run_id),
            "Folio adapter keeps raw claims, citations, and source locators in app storage"
                .to_string(),
        ],
        run_stages: Vec::new(),
    })
}

fn all_claim_citations_resolve(run: &FolioExecutionRun) -> bool {
    let citation_ids = run
        .citations
        .iter()
        .map(|citation| citation.citation_id.as_str())
        .collect::<BTreeSet<_>>();

    !run.claim_set.is_empty()
        && run.claim_set.iter().all(|claim| {
            !claim.citation_refs.is_empty()
                && claim
                    .citation_refs
                    .iter()
                    .all(|citation_ref| citation_ids.contains(citation_ref.as_str()))
        })
        && run.citations.iter().all(|citation| {
            citation.source_hash.starts_with("sha256:")
                && !citation.locator.trim().is_empty()
                && !citation.quote_permission.trim().is_empty()
        })
}

fn folio_fact(
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
            evidence_id: format!("evidence:{fact_id}"),
            source: "folio-editor-transcript".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace:{fact_id}"),
            location: Some("folio:publication-package".to_string()),
            replayable: true,
        }),
        promotion_authority: Some(authority.clone()),
    }
}

fn evidence_clause_id(package: &TruthPackage, clause_key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .find(|clause| clause.key == clause_key)
        .map_or_else(
            || panic!("missing evidence clause {clause_key}"),
            |clause| clause.id.clone(),
        )
}

fn failure_clause_id(package: &TruthPackage, clause_key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .find(|clause| clause.key == clause_key)
        .map_or_else(
            || panic!("missing failure clause {clause_key}"),
            |clause| clause.id.clone(),
        )
}

fn has_truth_key(truth_keys: &[String], key: &str) -> bool {
    truth_keys.iter().any(|candidate| candidate == key)
}

fn folio_publication_transcript() -> FolioPublicationTranscript {
    serde_json::from_str(FOLIO_PUBLICATION_TRANSCRIPT).expect("Folio fixture is valid JSON")
}

fn job_readiness_packet_id(
    package: &TruthPackage,
    domain_hint: &str,
    edition_run_id: &str,
    adapter_receipt_id: &str,
) -> String {
    short_id(
        &sha256_lines(&[
            "job_readiness_packet",
            package.package_id.as_str(),
            package.truth_version.as_str(),
            domain_hint,
            edition_run_id,
            adapter_receipt_id,
        ]),
        "helm.job_readiness",
    )
}

fn canonical_story_receipt_id(packet: &JobReadinessPacket, story: &FolioCanonicalStory) -> String {
    short_id(
        &sha256_lines(&[
            "canonical_story_receipt",
            packet.packet_id.as_str(),
            story.story_object_hash.as_str(),
            story.freeze_hash.as_str(),
        ]),
        "helm.canonical_story",
    )
}

fn claim_review_receipt_id(
    packet: &JobReadinessPacket,
    claims: &[FolioClaim],
    story_receipt: &CanonicalStoryReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "claim_review_receipt",
            packet.packet_id.as_str(),
            sha256_json(&claims).as_str(),
            story_receipt.receipt_id.as_str(),
        ]),
        "helm.claim_review",
    )
}

fn editorial_approval_receipt_id(
    packet: &JobReadinessPacket,
    approval: &FolioEditorialApproval,
    claim_receipt: &ClaimReviewReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "editorial_approval_receipt",
            packet.packet_id.as_str(),
            approval.gate_id.as_str(),
            approval.status.as_str(),
            claim_receipt.receipt_id.as_str(),
        ]),
        "helm.editorial_approval",
    )
}

fn publication_boundary_receipt_id(
    packet: &JobReadinessPacket,
    boundary: &FolioPublicationBoundary,
    export: &FolioPackageExport,
    approval_receipt: &EditorialApprovalReceipt,
) -> String {
    short_id(
        &sha256_lines(&[
            "publication_boundary_receipt",
            packet.packet_id.as_str(),
            boundary.boundary_hash.as_str(),
            export.package_hash.as_str(),
            approval_receipt.receipt_id.as_str(),
        ]),
        "helm.publication_boundary",
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
struct CanonicalStoryReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    story_ref_hash: String,
    schema_version: String,
    revision: u64,
    claim_count: usize,
    unresolved_question_count: usize,
    story_object_hash: String,
    adapter_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ClaimReviewReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    claim_count: usize,
    cited_claim_count: usize,
    contradiction_count: usize,
    unresolved_question_count: usize,
    claim_set_hash: String,
    unresolved_questions_hash: String,
    canonical_story_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct EditorialApprovalReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    gate_ref_hash: String,
    status: String,
    approver_role_hash: String,
    approval_hash: String,
    publish_window_hash: String,
    claim_review_receipt_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct PublicationBoundaryReceipt {
    receipt_id: String,
    package_id: String,
    truth_version: String,
    domain_hint: String,
    output_kind: String,
    cms_export_prepared: bool,
    public_publish_invoked: bool,
    reader_visible_ai_label: bool,
    channel_target_count: usize,
    package_hash: String,
    boundary_ok: bool,
    editorial_approval_receipt_id: String,
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
    CanonicalStoryReceipt,
    ClaimReviewReceipt,
    EditorialApprovalReceipt,
    PublicationBoundaryReceipt,
}

impl HelmLedgerRecordKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationAdapterReceipt => "observation_adapter_receipt",
            Self::JobReadinessPacket => "job_readiness_packet",
            Self::CanonicalStoryReceipt => "canonical_story_receipt",
            Self::ClaimReviewReceipt => "claim_review_receipt",
            Self::EditorialApprovalReceipt => "editorial_approval_receipt",
            Self::PublicationBoundaryReceipt => "publication_boundary_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum HelmLedgerAuthorityEffect {
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioPublicationTranscript {
    source: FolioRunSource,
    execution_run: FolioExecutionRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioRunSource {
    run_id: String,
    app_path: String,
    command: String,
    captured_at: String,
    domain_hint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioExecutionRun {
    edition_run_id: String,
    status: String,
    truth_keys: Vec<String>,
    canonical_story: FolioCanonicalStory,
    brief: FolioBrief,
    source_map: FolioSourceMap,
    claim_set: Vec<FolioClaim>,
    citations: Vec<FolioCitation>,
    unresolved_questions: Vec<FolioUnresolvedQuestion>,
    standards_review: FolioStandardsReview,
    editorial_approval: FolioEditorialApproval,
    publication_boundary: FolioPublicationBoundary,
    package_export: FolioPackageExport,
    promotion_authority: PromotionAuthorityRecord,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioCanonicalStory {
    story_id: String,
    thread_id: String,
    schema_version: String,
    snapshot_id: String,
    story_object_hash: String,
    freeze_hash: String,
    revision: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioBrief {
    brief_id: String,
    desk: String,
    angle_hash: String,
    approved_by_role: String,
    approved_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioSourceMap {
    source_map_id: String,
    qualified_source_ids: Vec<String>,
    permissions_recorded: bool,
    anonymity_policy: String,
    source_map_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioClaim {
    claim_id: String,
    status: String,
    statement_hash: String,
    citation_refs: Vec<String>,
    contradiction_refs: Vec<String>,
    loose_ends: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioCitation {
    citation_id: String,
    source_id: String,
    locator: String,
    source_hash: String,
    quote_permission: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioUnresolvedQuestion {
    question_id: String,
    status: String,
    owner_role: String,
    evidence_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioStandardsReview {
    gate_id: String,
    status: String,
    reviewer_role: String,
    legal_risk: String,
    right_of_response_status: String,
    note_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioEditorialApproval {
    gate_id: String,
    status: String,
    approver_role: String,
    approval_hash: String,
    publish_window: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioPublicationBoundary {
    output_kind: String,
    cms_export_prepared: bool,
    public_publish_invoked: bool,
    reader_visible_ai_label: bool,
    live_url: Option<String>,
    boundary_hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FolioPackageExport {
    export_id: String,
    channel_targets: Vec<String>,
    package_hash: String,
    layout_candidate_hash: String,
}
