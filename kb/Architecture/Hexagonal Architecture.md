---
tags: [architecture, philosophy]
---
# Hexagonal Architecture

Converge follows hexagonal architecture (ports and adapters). The core engine has no knowledge of the outside world. External systems plug in through traits.

## The Hexagon

```
                         ┌─────────────────────┐
                         │   Desktop (Tauri)    │
                         │   CLI / HTTP / gRPC  │
                         └──────────┬──────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
              ┌─────┴─────┐  ┌─────┴─────┐  ┌─────┴─────┐
              │  Driving   │  │  Driving   │  │  Driving   │
              │  (REST)    │  │  (gRPC)    │  │  (SSE)     │
              └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
                    │              │              │
        ┌───────────┴──────────────┴──────────────┴───────────┐
        │                                                      │
        │                    CORE HEXAGON                      │
        │                                                      │
        │   Engine ─── Context ─── Facts ─── Invariants        │
        │       │                                              │
        │   Agents ─── Proposals ─── Promotion Gate            │
        │       │                                              │
        │   RootIntent ─── Budget ─── Criteria                 │
        │                                                      │
        │            ── PORTS (traits) ──                       │
        │   Backend · LlmProvider · ExperienceStore            │
        │   BackendSelector · VectorRecall · Embedding         │
        │   Invariant · Agent · StreamingCallback              │
        │                                                      │
        └──┬──────────┬──────────┬──────────┬─────────────┬───┘
           │          │          │          │             │
     ┌─────┴───┐ ┌───┴────┐ ┌──┴───┐ ┌───┴────┐  ┌────┴─────┐
     │  LLM    │ │Storage │ │Search│ │Optimize│  │Analytics │
     │Providers│ │Adapters│ │      │ │        │  │          │
     └─────────┘ └────────┘ └──────┘ └────────┘  └──────────┘
     Anthropic    SurrealDB   LanceDB  OR-Tools    Burn
     OpenAI       LanceDB     Qdrant              Polars
     Ollama       S3/GCS
     Gemini       Local FS
     vLLM
```

## The Rule

The core hexagon depends on **nothing** outside itself. It defines [[Architecture/Ports|ports]] (traits). External systems implement those traits as [[Architecture/Providers|providers]] (adapters). The core never imports a provider. Providers import the core.

This is [[Philosophy/Nine Axioms#5. Safety by Construction|Axiom 5]] applied to architecture: invalid dependencies are unrepresentable because the core crate literally cannot see the adapter crates.

## Three Ways to Understand It

### For Business

Think of Converge as a decision-making engine with pluggable sensors and actuators. The engine itself is pure logic — it doesn't know whether it's talking to Claude, GPT, a local model, or a spreadsheet. You can swap any external system without changing how decisions are made or governed.

### For Users

You author packs against `converge-pack`. Your suggestor receives a `&dyn ReadContext` and returns proposals. If it needs an LLM, it takes an `Arc<dyn LlmProvider>` — not an `Arc<AnthropicProvider>`. At construction time, you inject the concrete provider. Your code never changes when you switch providers. See [[Architecture/API Surfaces]] for which crate to depend on.

### For Contributors

The dependency arrow always points inward. `converge-pack` and `converge-provider-api` are the leaf contracts (zero internal deps). `converge-core` depends on `converge-pack`. `converge-provider` depends on `converge-pack` + `converge-provider-api`. If you find yourself importing an adapter from a contract crate, you've broken the architecture. See [[Architecture/Purity Rules]].

## Driving vs Driven

**Driving adapters** (left side) — things that call into Converge:
- HTTP/REST API (Axum)
- gRPC bidirectional streaming (Tonic)
- SSE fallback transport
- Tauri command layer
- CLI

**Driven adapters** (right side) — things Converge calls out to:
- LLM providers (cloud and local)
- Storage backends (SurrealDB, LanceDB, S3)
- Search engines (vector, full-text)
- Optimization solvers (OR-Tools)
- Analytics engines (Burn, Polars)

The core doesn't know which side is which. It only knows traits.

See also: [[Architecture/Ports]], [[Architecture/Providers]], [[Architecture/Purity Rules]]
