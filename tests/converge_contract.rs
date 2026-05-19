use std::sync::Arc;

use axiom_truth::{
    GherkinValidator, InvariantClassTag, ScenarioKind, SimulationConfig, StaticChatBackend,
    ValidationConfig, compile_intent_from_source, simulate_spec,
};
use converge_kernel::formation::{
    FormationTemplateQuery, ProfileSnapshot, SuggestorCapability, SuggestorRole,
};
use converge_kernel::{
    AgentEffect, Context, ContextKey, ProposedFact, StopReason, Suggestor, TextPayload,
};
use converge_provider::DynChatBackend;
use converge_provider::{BackendRequirements, CostClass, LatencyClass};
use organism_pack::{
    AdmissionController, DefaultAdmissionController, ExpiryAction, FeasibilityDimension,
    FeasibilityKind, Reversibility,
};
use organism_runtime::{
    ExecutableSuggestorCatalog, FormationCompileRequest, FormationCompilerCatalogs,
    FormationOutcomeStatus, ProviderDescriptor, Runtime, Seed, SuggestorDescriptor,
    standard_formation_catalog,
};

const GOVERNED_VENDOR_TRUTH: &str = r"Truth: Vendor approval is governed
  Vendor commitments must be evidence-backed and approval-gated.

Intent:
  Outcome: Approve a vendor with auditable rationale.
  Goal: Keep financial commitments traceable.

Authority:
  Actor: finance_controller
  May: approve_vendor
  Must Not: bypass_audit
  Requires Approval: vendor_commitment
  Expires: 2099-01-01T00:00:00Z

Constraint:
  Budget: <= 100000
  Cost Limit: reversibility: irreversible
  Must Not: bypass_audit
  Must Not: spend_without_po

Evidence:
  Requires: vendor_assessment
  Requires: decision_log
  Audit: approval_record

Exception:
  Escalates To: cfo
  Requires: exception_memo

  @invariant @acceptance @id:vendor_approval
  Scenario: Vendor commitment requires evidence
    Given vendor_assessment exists
    When the finance_controller approves the vendor
    Then decision_log records the approval rationale
";

#[tokio::test]
async fn validation_runs_against_mock_converge_provider_backend() {
    let backend: Arc<dyn DynChatBackend> = Arc::new(StaticChatBackend::queued([
        "VALID",
        "COMPILABLE: acceptance - checks the approval record and required evidence",
    ]));
    let validator = GherkinValidator::new(backend, ValidationConfig::default());

    let validation = validator
        .validate(GOVERNED_VENDOR_TRUTH, "vendor-approval.truths")
        .await
        .expect("truth validates through the mock backend");

    assert!(validation.is_valid, "{:?}", validation.issues);
    assert_eq!(validation.scenario_count, 1);
    assert_eq!(
        validation
            .governance
            .intent
            .as_ref()
            .and_then(|intent| intent.outcome.as_deref()),
        Some("Approve a vendor with auditable rationale.")
    );

    let scenario = validation
        .scenario_metas
        .first()
        .expect("scenario metadata extracted");
    assert_eq!(scenario.kind, Some(ScenarioKind::Invariant));
    assert_eq!(
        scenario.invariant_class,
        Some(InvariantClassTag::Acceptance)
    );
    assert_eq!(scenario.id.as_deref(), Some("vendor_approval"));
}

#[test]
fn truth_compiles_to_organism_intent_packet_contract() {
    let packet =
        compile_intent_from_source(GOVERNED_VENDOR_TRUTH).expect("truth compiles to intent packet");

    assert_eq!(packet.outcome, "Approve a vendor with auditable rationale.");
    assert_eq!(
        packet.authority,
        vec![
            "actor: finance_controller".to_string(),
            "approve_vendor".to_string()
        ]
    );
    assert_eq!(packet.reversibility, Reversibility::Irreversible);
    assert_eq!(packet.expiry_action, ExpiryAction::Escalate);
    assert_eq!(packet.expires.to_rfc3339(), "2099-01-01T00:00:00+00:00");

    assert!(
        packet
            .constraints
            .contains(&"budget: <= 100000".to_string())
    );
    assert!(
        packet
            .constraints
            .contains(&"cost_limit: reversibility: irreversible".to_string())
    );
    assert!(
        packet
            .constraints
            .contains(&"requires_approval: vendor_commitment".to_string())
    );

    assert_eq!(packet.forbidden.len(), 2);
    assert!(packet.forbidden.iter().any(|forbidden| {
        forbidden.action == "bypass_audit" && forbidden.reason == "authority"
    }));
    assert!(packet.forbidden.iter().any(|forbidden| {
        forbidden.action == "spend_without_po" && forbidden.reason == "constraint"
    }));
}

#[test]
fn compiled_intent_is_accepted_by_default_admission_controller() {
    let packet =
        compile_intent_from_source(GOVERNED_VENDOR_TRUTH).expect("truth compiles to intent packet");
    let admission = DefaultAdmissionController::new().evaluate(&packet);

    assert!(admission.feasible, "{admission:?}");
    assert!(admission.rejection_reason.is_none());

    let authority = admission
        .dimensions
        .iter()
        .find(|dimension| dimension.dimension == FeasibilityDimension::Authority)
        .expect("authority dimension present");
    assert_eq!(authority.kind, FeasibilityKind::Feasible);
}

#[test]
fn simulation_report_is_reproducible_at_converge_boundary() {
    let config = SimulationConfig::default();
    let first = simulate_spec(GOVERNED_VENDOR_TRUTH, &config).expect("first simulation succeeds");
    let second = simulate_spec(GOVERNED_VENDOR_TRUTH, &config).expect("second simulation succeeds");

    assert_eq!(first, second);
    assert!(first.can_converge(), "{first:?}");
    assert!(first.deterministic_trace.replayable);
    assert!(first.deterministic_trace.trace_hash.starts_with("sha256:"));
}

#[tokio::test]
async fn truth_drives_organism_formation_to_converge_fixed_point() {
    let intent =
        compile_intent_from_source(GOVERNED_VENDOR_TRUTH).expect("truth compiles to intent packet");
    let runtime = Runtime::new();
    let formation_catalog = standard_formation_catalog();
    let capabilities = [
        SuggestorCapability::LlmReasoning,
        SuggestorCapability::KnowledgeRetrieval,
        SuggestorCapability::PolicyEnforcement,
        SuggestorCapability::HumanInTheLoop,
    ];

    let selection = runtime
        .select_formation(&intent, &formation_catalog, &capabilities)
        .expect("Organism selects a formation for the truth intent");
    assert_eq!(selection.primary.id(), "organism-diligence");

    let request = FormationCompileRequest::new(
        uuid::Uuid::from_u128(1),
        uuid::Uuid::from_u128(2),
        FormationTemplateQuery::new()
            .with_keyword("diligence")
            .with_keyword("audit")
            .with_entity("evidence"),
    )
    .with_tenant_id("axiom-v0.9-proof")
    .with_domain_tag("truth-proof");

    let seed = Seed {
        key: ContextKey::Seeds,
        id: format!("intent:{}", intent.id).into(),
        content: serde_json::to_string(&intent).expect("intent serializes"),
        provenance: "axiom-truth".to_string(),
    };

    let record = runtime
        .compile_and_run_formation(
            &intent,
            &request,
            &proof_catalogs(),
            &proof_executables(),
            vec![seed],
            None,
        )
        .await
        .expect("fixture formation compiles and reaches Converge");

    assert_eq!(record.plan.template_id, selection.primary.id());
    assert_eq!(record.outcome.status, FormationOutcomeStatus::Converged);
    assert!(record.result.converge_result.converged);
    assert!(matches!(
        record.result.converge_result.stop_reason,
        StopReason::Converged
    ));
    assert!(
        record
            .result
            .converge_result
            .context
            .has(ContextKey::Signals)
    );
    assert!(
        record
            .result
            .converge_result
            .context
            .has(ContextKey::Evaluations)
    );
    assert!(
        record
            .result
            .converge_result
            .context
            .has(ContextKey::Constraints)
    );
    assert!(
        record
            .result
            .converge_result
            .context
            .has(ContextKey::Proposals)
    );
    assert!(record.result.converge_result.integrity.fact_count >= 5);
    assert!(record.result.converge_result.integrity.clock_time >= 5);
}

fn proof_catalogs() -> FormationCompilerCatalogs {
    let policy_requirements = BackendRequirements::access_policy().with_replay();

    FormationCompilerCatalogs::new(standard_formation_catalog())
        .with_suggestor(proof_descriptor(
            "truth-signal",
            SuggestorRole::Signal,
            vec![ContextKey::Signals],
            vec![SuggestorCapability::KnowledgeRetrieval],
            vec![ContextKey::Seeds],
            None,
        ))
        .with_suggestor(proof_descriptor(
            "truth-evaluator",
            SuggestorRole::Evaluation,
            vec![ContextKey::Evaluations],
            vec![SuggestorCapability::Analytics],
            vec![ContextKey::Signals],
            None,
        ))
        .with_suggestor(proof_descriptor(
            "truth-policy-gate",
            SuggestorRole::Constraint,
            vec![ContextKey::Constraints],
            vec![
                SuggestorCapability::PolicyEnforcement,
                SuggestorCapability::HumanInTheLoop,
            ],
            vec![ContextKey::Evaluations],
            Some(policy_requirements.clone()),
        ))
        .with_suggestor(proof_descriptor(
            "truth-synthesis",
            SuggestorRole::Synthesis,
            vec![ContextKey::Proposals],
            vec![SuggestorCapability::LlmReasoning],
            vec![ContextKey::Evaluations, ContextKey::Constraints],
            None,
        ))
        .with_provider(
            ProviderDescriptor::new(
                "cedar-local",
                "Cedar local policy fixture",
                policy_requirements,
            )
            .with_role_affinity(SuggestorRole::Constraint)
            .with_domain_tag("truth-proof"),
        )
}

fn proof_descriptor(
    id: &'static str,
    role: SuggestorRole,
    output_keys: Vec<ContextKey>,
    capabilities: Vec<SuggestorCapability>,
    reads: Vec<ContextKey>,
    backend_requirements: Option<BackendRequirements>,
) -> SuggestorDescriptor {
    let profile = ProfileSnapshot {
        name: id.to_string(),
        role,
        output_keys,
        cost_hint: CostClass::Low,
        latency_hint: LatencyClass::Interactive,
        capabilities,
        confidence_min: 0.7,
        confidence_max: 0.95,
    };

    let mut descriptor = SuggestorDescriptor::new(id, profile).with_domain_tag("truth-proof");
    for key in reads {
        descriptor = descriptor.with_read(key);
    }
    if let Some(requirements) = backend_requirements {
        descriptor = descriptor.with_backend_requirements(requirements);
    }
    descriptor
}

fn proof_executables() -> ExecutableSuggestorCatalog {
    let mut catalog = ExecutableSuggestorCatalog::new();
    catalog
        .register_factory("truth-signal", || {
            ProofSuggestor::new("truth-signal", vec![ContextKey::Seeds], ContextKey::Signals)
        })
        .expect("register signal proof suggestor");
    catalog
        .register_factory("truth-evaluator", || {
            ProofSuggestor::new(
                "truth-evaluator",
                vec![ContextKey::Signals],
                ContextKey::Evaluations,
            )
        })
        .expect("register evaluation proof suggestor");
    catalog
        .register_factory("truth-policy-gate", || {
            ProofSuggestor::new(
                "truth-policy-gate",
                vec![ContextKey::Evaluations],
                ContextKey::Constraints,
            )
        })
        .expect("register constraint proof suggestor");
    catalog
        .register_factory("truth-synthesis", || {
            ProofSuggestor::new(
                "truth-synthesis",
                vec![ContextKey::Evaluations, ContextKey::Constraints],
                ContextKey::Proposals,
            )
        })
        .expect("register synthesis proof suggestor");
    catalog
}

struct ProofSuggestor {
    name: &'static str,
    dependencies: Vec<ContextKey>,
    output: ContextKey,
}

impl ProofSuggestor {
    fn new(name: &'static str, dependencies: Vec<ContextKey>, output: ContextKey) -> Self {
        Self {
            name,
            dependencies,
            output,
        }
    }
}

#[async_trait::async_trait]
impl Suggestor for ProofSuggestor {
    fn name(&self) -> &str {
        self.name
    }

    fn dependencies(&self) -> &[ContextKey] {
        &self.dependencies
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        self.dependencies.iter().all(|key| ctx.has(*key)) && !ctx.has(self.output)
    }

    fn provenance(&self) -> &'static str {
        "axiom-truth"
    }

    async fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
        AgentEffect::with_proposal(ProposedFact::new(
            self.output,
            format!("{}-output", self.name),
            TextPayload::new(format!("{} produced fixture proof output", self.name)),
            "axiom-truth",
        ))
    }
}
