---
tags: [building]
source: mixed
---
# Writing Suggestors

Every suggestor implements the `Suggestor` trait from `converge-pack`. Embedded applications register suggestors through `converge-kernel`. Read [[Concepts/Agents]] for the contract. This page is the practical guide.

## Minimal Suggestor

```rust
struct MySuggestor;

impl Suggestor for MySuggestor {
    fn name(&self) -> &str { "my-suggestor" }

    fn dependencies(&self) -> &[ContextKey] {
        &[]  // runs on cycle 1
    }

    fn accepts(&self, _ctx: &dyn Context) -> bool {
        true
    }

    fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
        AgentEffect::with_proposal(ProposedFact {
            key: ContextKey::Seeds,
            id: "my:fact".into(),
            content: serde_json::json!({ "result": "data" }).to_string(),
            confidence: 0.85,
            provenance: "suggestor:my-suggestor".into(),
        })
    }
}
```

## Idempotency

Check for your own facts before re-proposing. Without this, the suggestor proposes the same fact every cycle and convergence never happens.

```rust
fn execute(&self, ctx: &dyn Context) -> AgentEffect {
    let fact_id = "compliance:screen:acme";
    if ctx.get(ContextKey::Seeds).iter().any(|f| f.id == fact_id) {
        return AgentEffect::empty();
    }
    // ... propose the fact
}
```

## Service-Backed Suggestor

Inject a trait so the suggestor works with real services or mocks. See [[Integrations/External Services]].

```rust
struct ComplianceSuggestor {
    policies: Arc<dyn PolicyService>,
}
```

## LLM-Backed Suggestor

```rust
struct SynthesisSuggestor {
    provider: Arc<dyn LlmProvider>,
}

impl Suggestor for SynthesisSuggestor {
    fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let prompt = format!("Synthesize a recommendation from: {:?}",
            ctx.get(ContextKey::Evaluations));
        let response = self.provider.complete(&prompt);
        // Parse response into ProposedFact
    }
}
```

## Suggestor Packs

Register multiple suggestors in the same pack for parallel execution within a cycle:

```rust
engine.register_suggestor_in_pack("screening-pack", ComplianceSuggestor { .. });
engine.register_suggestor_in_pack("screening-pack", DataResidencySuggestor { .. });
engine.register_suggestor_in_pack("screening-pack", CertificationSuggestor { .. });
```

All three run in parallel, propose into the same context, and effects merge deterministically.

## Rules to Internalize

1. `accepts()` is pure — no I/O, no state, no randomness
2. `execute()` reads context, returns proposals — never mutates
3. Suggestors never call each other — context is the only channel
4. Check before proposing — idempotency prevents infinite loops
5. `confidence` and `provenance` are always set — the governance gate uses them

See also: [[Concepts/Agents]], [[Building/Context Keys]], [[Philosophy/Convergence Explained]]
