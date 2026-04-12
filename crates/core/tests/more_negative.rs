// Additional negative tests for edge cases.

use converge_core::{AgentEffect, Context, ContextKey, Engine, ProposedFact, Suggestor};

#[test]
fn confidence_exactly_zero_accepted() {
    struct ZeroConfSuggestor;
    impl Suggestor for ZeroConfSuggestor {
        fn name(&self) -> &str {
            "zero-conf"
        }
        fn dependencies(&self) -> &[ContextKey] {
            &[]
        }
        fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
            !ctx.has(ContextKey::Seeds)
        }
        fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
            AgentEffect::with_proposal(ProposedFact {
                key: ContextKey::Seeds,
                id: "zero-1".into(),
                content: "zero confidence".into(),
                confidence: 0.0,
                provenance: "zero-conf".into(),
            })
        }
    }
    let mut engine = Engine::new();
    engine.register_suggestor(ZeroConfSuggestor);
    let result = engine.run(Context::new()).expect("converges");
    assert!(result.converged);
    assert!(result.context.has(ContextKey::Seeds));
}

#[test]
fn confidence_exactly_one_accepted() {
    let mut engine = Engine::new();
    engine.register_suggestor(converge_core::suggestors::SeedSuggestor::new("s1", "v"));
    let result = engine.run(Context::new()).expect("converges");
    assert!(result.converged);
}

#[test]
fn confidence_slightly_above_one_rejected() {
    struct OverConfSuggestor;
    impl Suggestor for OverConfSuggestor {
        fn name(&self) -> &str {
            "over-conf"
        }
        fn dependencies(&self) -> &[ContextKey] {
            &[]
        }
        fn accepts(&self, _ctx: &dyn converge_core::ContextView) -> bool {
            true
        }
        fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
            AgentEffect::with_proposal(ProposedFact {
                key: ContextKey::Seeds,
                id: "over-1".into(),
                content: "over confidence".into(),
                confidence: 1.0001,
                provenance: "over-conf".into(),
            })
        }
    }
    let mut engine = Engine::new();
    engine.register_suggestor(OverConfSuggestor);
    let result = engine
        .run(Context::new())
        .expect("converges with rejection");
    assert!(!result.context.has(ContextKey::Seeds));
}

#[test]
fn suggestor_that_never_accepts_produces_no_facts() {
    struct NeverAccepts;
    impl Suggestor for NeverAccepts {
        fn name(&self) -> &str {
            "never"
        }
        fn dependencies(&self) -> &[ContextKey] {
            &[]
        }
        fn accepts(&self, _ctx: &dyn converge_core::ContextView) -> bool {
            false
        }
        fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
            panic!("should never be called")
        }
    }
    let mut engine = Engine::new();
    engine.register_suggestor(NeverAccepts);
    let result = engine.run(Context::new()).expect("converges");
    assert!(result.converged);
}

#[test]
fn empty_proposal_id_still_works() {
    struct EmptyIdSuggestor;
    impl Suggestor for EmptyIdSuggestor {
        fn name(&self) -> &str {
            "empty-id"
        }
        fn dependencies(&self) -> &[ContextKey] {
            &[]
        }
        fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
            !ctx.has(ContextKey::Seeds)
        }
        fn execute(&self, _ctx: &dyn converge_core::ContextView) -> AgentEffect {
            AgentEffect::with_proposal(ProposedFact {
                key: ContextKey::Seeds,
                id: String::new(),
                content: "has empty id".into(),
                confidence: 0.8,
                provenance: "empty-id".into(),
            })
        }
    }
    let mut engine = Engine::new();
    engine.register_suggestor(EmptyIdSuggestor);
    // Should still converge — empty ID is valid at the pack level
    let result = engine.run(Context::new()).expect("converges");
    assert!(result.converged);
}
