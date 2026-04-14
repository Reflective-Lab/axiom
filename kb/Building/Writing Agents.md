---
tags: [building]
source: mixed
---
# Writing Suggestors

Every suggestor implements the `Suggestor` trait from `converge-pack`. Embedded applications register suggestors through `converge-kernel`. Read [[Concepts/Agents]] for the contract. This page is the practical guide.

Suggestors are async. `accepts()` stays pure and synchronous; `execute()` is async and read-only. The host application owns the runtime.

## Minimal Suggestor

```rust
use async_trait::async_trait;

struct MySuggestor;

#[async_trait]
impl Suggestor for MySuggestor {
    fn name(&self) -> &str { "my-suggestor" }

    fn dependencies(&self) -> &[ContextKey] {
        &[]  // runs on cycle 1
    }

    fn accepts(&self, _ctx: &dyn Context) -> bool {
        true
    }

    async fn execute(&self, _ctx: &dyn Context) -> AgentEffect {
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
async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
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
    backend: Arc<dyn DynChatBackend>,
}

#[async_trait::async_trait]
impl Suggestor for SynthesisSuggestor {
    async fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: format!("Synthesize a recommendation from: {:?}", ctx.get(ContextKey::Evaluations)),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some("Be concise and propose only supported claims.".into()),
            tools: Vec::new(),
            response_format: ResponseFormat::Text,
            max_tokens: Some(256),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
            model: None,
        };

        match self.backend.chat(request).await {
            Ok(response) => {
                // Parse response.content into ProposedFact
                AgentEffect::empty()
            }
            Err(_error) => AgentEffect::empty(),
        }
    }
}
```

Handle backend errors explicitly in `execute()`. Do not invent a second sync provider contract just to avoid awaiting the canonical chat backend.

## Suggestor Packs

Register related suggestors in the same pack so truths can activate or deactivate them together:

```rust
engine.register_suggestor_in_pack("screening-pack", ComplianceSuggestor { .. });
engine.register_suggestor_in_pack("screening-pack", DataResidencySuggestor { .. });
engine.register_suggestor_in_pack("screening-pack", CertificationSuggestor { .. });
```

Pack membership controls activation and grouping. Execution and merge ordering are still owned by the engine.

## Rules to Internalize

1. `accepts()` is pure — no I/O, no state, no randomness
2. `execute()` is async, reads context, returns proposals — never mutates
3. Suggestors never call each other — context is the only channel
4. Check before proposing — idempotency prevents infinite loops
5. `confidence` and `provenance` are always set — the governance gate uses them

See also: [[Concepts/Agents]], [[Building/Context Keys]], [[Philosophy/Convergence Explained]]
