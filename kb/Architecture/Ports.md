---
tags: [architecture]
source: mixed
---
# Ports

Ports are the trait definitions that form the boundary of the [[Architecture/Hexagonal Architecture|hexagon]]. The core defines what it needs. Adapters provide it.

## Authoring Ports (`converge-pack`)

These are the public contract. Every port is a trait. Every trait is `Send + Sync`.

### Suggestor Execution

| Port | Purpose | Key Methods |
|---|---|---|
| `Suggestor` | Capability contract | `name()`, `dependencies()`, `accepts()`, `execute()` |
| `Context` | Read-only shared state | `has()`, `get()`, `get_proposals()`, `count()` |
| `Invariant` | Executable guarantees | `name()`, `class()`, `check()` |

## Provider Routing Ports (`converge-provider-api`)

| Port | Purpose | Key Methods |
|---|---|---|
| `Backend` | Capability declaration | `name()`, `kind()`, `capabilities()`, `has_capability()` |
| `BackendSelector` | Capability-based routing | `select(&BackendRequirements)` |

### Backend Requirements

```rust
BackendRequirements::new(BackendKind::Llm)
    .with_capability(Capability::TextGeneration)
    .with_capability(Capability::Reasoning)
    .with_max_cost(CostClass::Medium)
    .with_data_sovereignty(DataSovereignty::EU)
    .with_compliance(ComplianceLevel::GDPR)
```

Selection is by capability, not by name ([[Concepts/Backends and Capabilities]]).

## Kernel Embedding Hooks (`converge-kernel`)

| Port | Purpose | Key Methods |
|---|---|---|
| `StreamingCallback` | Real-time notifications | `on_cycle_start()`, `on_fact()`, `on_cycle_end()` |
| `ExperienceEventObserver` | Experience event taps | `on_event(...)` |

## Internal Adapter Ports

| Port | Purpose | Key Methods |
|---|---|---|
| `LlmProvider` | LLM inference | `complete(request)`, `health_check()`, `provenance()` |
| `Embedding` | Vector embeddings | Generate embeddings from text/images |
| `VectorRecall` | Similarity search | Vector-indexed retrieval |
| `Reranking` | Result re-ranking | Cross-encoder relevance scoring |

These are implementation details, not stable external contracts.

## Error Contract

All ports use a consistent error type:

```rust
BackendError {
    kind: BackendErrorKind,
    message: String,
    retryable: bool,
}
```

Retryability is declared, not guessed: `RateLimit`, `Unavailable`, `Network`, `Timeout` are retryable. `Authentication`, `InvalidRequest` are not.

See also: [[Architecture/Providers]], [[Architecture/Hexagonal Architecture]]
