use axiom_truth::{
    AxiomRunObservation, AxiomRunReport, AxiomRunStageRecord, AxiomRunVerdict, ClauseId,
    ClauseInput, EvidenceRefRecord, JtbdClauseKind, JtbdInput, ObservedStopReason,
    PromotedFactRecord, RunIntegrityProof, TraceLinkRecord, TruthPackage, decode_jtbd,
};

fn round_driven_jtbd() -> JtbdInput {
    JtbdInput {
        key: "round-driven-formation-design".to_string(),
        actor: "formation host".to_string(),
        functional_job:
            "select and run a policy-and-anomaly audit Formation for a candidate plan".to_string(),
        so_that:
            "dynamic Formation design is traceable, executable, and converges before the plan audit is trusted"
                .to_string(),
        evidence_required: vec![
            ClauseInput::with_key("round_signals", "round signals and batch sentinels"),
            ClauseInput::with_key(
                "proposer_evolution",
                "prior-round exclusions derived from blocked drafts",
            ),
            ClauseInput::with_key(
                "critic_verdicts",
                "mechanical and LLM critic verdicts with confidence",
            ),
            ClauseInput::with_key("adversarial_findings", "adversarial findings per draft"),
            ClauseInput::with_key(
                "scorecard",
                "evidence-weighted scorecard and shortlist",
            ),
            ClauseInput::with_key(
                "compile_handoff",
                "compile handoff to real catalog and factory-covered descriptors",
            ),
            ClauseInput::with_key(
                "design_convergence",
                "design huddle stop reason and integrity proof",
            ),
            ClauseInput::with_key(
                "work_convergence",
                "work Formation stop reason and integrity proof",
            ),
        ],
        failure_modes: vec![
            ClauseInput::with_key(
                "non_evolving_rounds",
                "rounds do not evolve after blocked rosters",
            ),
            ClauseInput::with_key(
                "premature_shortlist",
                "shortlist is emitted before critic and scorer sentinels",
            ),
            ClauseInput::with_key(
                "blocked_descriptor_reuse",
                "blocked-only descriptors are reused without explanation",
            ),
            ClauseInput::with_key(
                "not_instantiable",
                "selected draft cannot be compiled or instantiated",
            ),
            ClauseInput::with_key("hidden_llm_failure", "LLM parse or chat failures are hidden"),
            ClauseInput::with_key("work_nonconvergence", "work Formation fails to converge"),
            ClauseInput::with_key(
                "missing_lineage",
                "promoted facts lack package clause lineage",
            ),
        ],
    }
}

#[test]
fn round_driven_marquee_jtbd_decodes_to_truth_package() {
    let package = decode_jtbd(round_driven_jtbd()).expect("round-driven JTBD decodes");

    assert!(
        package
            .generated_truths
            .contains("Truth: Select and run a policy-and-anomaly audit Formation")
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .count(),
        8
    );
    assert_eq!(
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .count(),
        7
    );
    assert_eq!(package.proof_obligations.len(), 15);
    assert!(
        package
            .verifier_spec
            .required_evidence
            .iter()
            .any(|evidence| evidence == "evidence-weighted scorecard and shortlist")
    );
    assert!(
        package
            .verifier_spec
            .forbidden_actions
            .iter()
            .any(|action| action.action == "selected draft cannot be compiled or instantiated")
    );
    assert!(
        package
            .lineage
            .validate_closure(&package.source_jtbd)
            .is_ok()
    );
}

#[test]
fn round_driven_marquee_report_preserves_design_and_work_stage_boundaries() {
    let package = decode_jtbd(round_driven_jtbd()).expect("round-driven JTBD decodes");
    let round_signals = clause_id(&package, "round_signals");
    let proposer_evolution = clause_id(&package, "proposer_evolution");
    let scorecard = clause_id(&package, "scorecard");
    let design_convergence = clause_id(&package, "design_convergence");
    let work_convergence = clause_id(&package, "work_convergence");
    let not_instantiable = clause_id(&package, "not_instantiable");

    let design_facts = vec![
        fact(
            "Signals",
            "design-round-1",
            "round 1 opened for policy-and-anomaly audit formation design",
            vec![round_signals.clone()],
        ),
        fact(
            "Diagnostic",
            "organism-catalog-proposer-exclusions-64657369676e2d726f756e642d32",
            "organism-catalog-proposer: exclusions for design-round-2 (blocked-minus-passed) = [disagreement-mapper]",
            vec![proposer_evolution.clone()],
        ),
        fact(
            "Diagnostic",
            "evidence-scorecard-design-round-2",
            "scenario-evidence-weighted-scorer: candidate-0 scored 182 while blocked candidates scored i32::MIN",
            vec![scorecard.clone()],
        ),
        fact(
            "Proposals",
            "evidence-shortlist-design-round-2-0",
            "latest completed batch shortlisted an executable policy/anomaly roster",
            vec![scorecard.clone(), not_instantiable.clone()],
        ),
        fact(
            "Hypotheses",
            "design-convergence:2",
            "round 1->2: CONVERGED - stable policy/anomaly coverage and preferred executable roster",
            vec![design_convergence.clone(), scorecard.clone()],
        ),
        fact(
            "Diagnostic",
            "design-convergence-reached",
            "scenario-convergence-judge: convergence reached at round 2",
            vec![design_convergence.clone()],
        ),
    ];
    let work_facts = vec![fact(
        "Evaluations",
        "candidate-plan-audit-evaluation",
        "candidate plan ALPHA-1 audited by the selected work Formation",
        vec![work_convergence.clone()],
    )];

    let observation = AxiomRunObservation::from_stages(
        ObservedStopReason::Converged,
        RunIntegrityProof::sha256_merkle("sha256:overall", 41, 28),
        vec!["two Converge boundaries reported".to_string()],
        vec![
            AxiomRunStageRecord {
                stage_id: "design_huddle".to_string(),
                formation_id: Some("round-driven-design-huddle".to_string()),
                observed_stop_reason: ObservedStopReason::Converged,
                promoted_facts: design_facts,
                integrity: RunIntegrityProof::sha256_merkle("sha256:design", 29, 21),
                replay_notes: vec![
                    "latest_completed_batch selected design-round-2".to_string(),
                    "round 3 skipped after design-convergence-reached halt marker".to_string(),
                ],
            },
            AxiomRunStageRecord {
                stage_id: "work_formation".to_string(),
                formation_id: Some("policy-and-anomaly-audit".to_string()),
                observed_stop_reason: ObservedStopReason::Converged,
                promoted_facts: work_facts,
                integrity: RunIntegrityProof::sha256_merkle("sha256:work", 12, 7),
                replay_notes: vec!["candidate plan ALPHA-1 reached fixed point".to_string()],
            },
        ],
    );

    let report =
        AxiomRunReport::from_observation(&package, AxiomRunVerdict::Satisfied, observation);

    assert!(report.expected_stop_reason_matched());
    assert_eq!(report.run_stages.len(), 2);
    assert_eq!(
        report
            .stage("design_huddle")
            .expect("design stage present")
            .integrity
            .merkle_root,
        "sha256:design"
    );
    assert_eq!(
        report
            .stage("work_formation")
            .expect("work stage present")
            .observed_stop_reason
            .expectation_kind(),
        axiom_truth::ExpectedStopReason::Converged
    );
    assert!(
        report
            .promoted_facts
            .iter()
            .flat_map(|fact| fact.source_clause_ids.iter())
            .any(|id| id == &design_convergence)
    );
    assert!(
        report
            .stage("design_huddle")
            .unwrap()
            .promoted_facts
            .iter()
            .any(|fact| fact.fact_id == "design-convergence-reached")
    );
    assert!(
        report
            .stage("design_huddle")
            .unwrap()
            .promoted_facts
            .iter()
            .any(|fact| fact.source_clause_ids.contains(&not_instantiable))
    );
    assert!(
        report
            .stage("design_huddle")
            .unwrap()
            .replay_notes
            .iter()
            .any(|note| note.contains("round 3 skipped"))
    );
}

fn clause_id(package: &TruthPackage, key: &str) -> ClauseId {
    package
        .source_jtbd
        .clauses
        .iter()
        .find(|clause| clause.key == key)
        .map_or_else(
            || panic!("missing clause key {key}"),
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
            source: "atelier-round-driven-design-scenario".to_string(),
        }],
        trace_link: Some(TraceLinkRecord {
            trace_id: format!("trace.{fact_id}"),
            location: Some("atelier-showcase:just show-round-driven".to_string()),
            replayable: true,
        }),
    }
}
