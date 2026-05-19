use axiom_truth::{
    AXIOM_PROVENANCE, ClauseInput, JtbdInput, PromotedFactRecord, TruthPackageSeedPayload,
    decode_jtbd, truth_package_seed_fact,
};
use converge_kernel::{
    AgentEffect, Context, ContextKey, ContextState, Engine, ProposedFact, StopReason, Suggestor,
};
use converge_pack::{FactEvidenceRef, ProvenanceSource};

struct AxiomSeedSuggestor {
    proposal: ProposedFact,
}

#[async_trait::async_trait]
impl Suggestor for AxiomSeedSuggestor {
    fn name(&self) -> &'static str {
        "axiom-seed-fixture"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        !ctx.has(ContextKey::Seeds)
    }

    async fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
        AgentEffect::with_proposal(self.proposal.clone())
    }

    fn provenance(&self) -> &'static str {
        AXIOM_PROVENANCE.as_str()
    }
}

#[tokio::test]
async fn truth_package_seed_fact_preserves_axiom_provenance_through_converge_promotion() {
    let package = decode_jtbd(JtbdInput {
        key: "Vendor Commitment".to_string(),
        actor: "finance controller".to_string(),
        functional_job: "approve a vendor commitment".to_string(),
        so_that: "spend is traceable and policy-compliant".to_string(),
        evidence_required: vec![ClauseInput::new("vendor assessment")],
        failure_modes: vec![ClauseInput::new("bypassed approval")],
        time_budget: None,
    })
    .expect("JTBD decodes to a Truth Package");
    let clause = package
        .source_jtbd
        .clauses
        .first()
        .expect("package has source clauses");
    let proposal = truth_package_seed_fact(&package, clause);
    let proposal_id = proposal.id.clone();

    assert_eq!(proposal.provenance(), AXIOM_PROVENANCE.as_str());

    let mut engine = Engine::new();
    engine.register_suggestor(AxiomSeedSuggestor { proposal });
    let result = engine
        .run(ContextState::new())
        .await
        .expect("Converge promotes the Axiom seed proposal");

    assert!(matches!(result.stop_reason, StopReason::Converged));
    assert!(result.integrity.fact_count >= 1);

    let facts = result.context.get(ContextKey::Seeds);
    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    let payload = fact
        .require_payload::<TruthPackageSeedPayload>()
        .expect("promoted fact preserves the typed Axiom payload");
    assert_eq!(payload.package_id, package.package_id);
    assert_eq!(payload.truth_version, package.truth_version);
    assert_eq!(payload.clause_id, clause.id);
    assert_eq!(payload.clause_fingerprint, clause.fingerprint);

    let promotion = fact.promotion_record();
    assert!(promotion.is_replay_eligible());
    assert!(matches!(
        promotion.evidence_refs(),
        [FactEvidenceRef::Observation(id)] if id.as_str() == format!("obs:{proposal_id}")
    ));

    let record = PromotedFactRecord::from_context_fact(fact, vec![clause.id.clone()]);
    assert_eq!(record.context_key, "Seeds");
    assert_eq!(record.fact_id, proposal_id.as_str());
    assert_eq!(record.summary, "axiom.truth_package_seed v1");
    assert_eq!(record.source_clause_ids, vec![clause.id.clone()]);
    assert_eq!(record.evidence_refs[0].source, "observation");
    assert!(record.trace_link.expect("trace link").replayable);
}
