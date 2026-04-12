---
tags: [building, reference]
---
# Crate Catalog

## Public Contract Crates

These are the six supported external APIs. See [[Architecture/API Surfaces]] for who should use what.

### Authoring

| Crate | What it does |
|---|---|
| `converge-pack` | Author packs, suggestors, invariants. The strict authoring contract. |
| `converge-provider-api` | Backend identity, capability declaration, routing requirements. |

### Semantic Model

| Crate | What it does |
|---|---|
| `converge-model` | Curated semantic types: governed Fact, Proposal, PromotionRecord, RootIntent, Criterion, StopReason, and all IDs/newtypes. |

### Execution

| Crate | What it does |
|---|---|
| `converge-kernel` | In-process embedding API: Engine, RunResult, Budget, HITL, streaming callbacks. |

### Remote

| Crate | What it does |
|---|---|
| `converge-protocol` | Generated `converge.v1` protobuf/gRPC types. Wire contract for remote systems. |
| `converge-client` | Idiomatic Rust SDK for remote Converge runtimes. Typed wrappers over the wire protocol. |

## Internal Crates

These are implementation crates. Not stable external contracts.

| Crate | What it does |
|---|---|
| `converge-core` | Engine implementation, context, promotion gates. Re-exports pack types. |
| `converge-domain` | Pre-built [[Concepts/Domain Packs\|agent packs]]: trust, money, delivery, knowledge, data_metrics |
| `converge-provider` | LLM provider adapters (Anthropic, OpenAI, Gemini, Ollama, and more) |
| `converge-mcp` | [[Integrations/MCP Tools\|Model Context Protocol]] server/client |
| `converge-tool` | Gherkin spec validation, truth-spec parsing |
| `converge-knowledge` | Knowledge management, signal capture |
| `converge-experience` | Experience tracking across runs |
| `converge-optimization` | Multi-criteria optimization via OR-Tools |
| `ortools-sys` | FFI bindings to Google OR-Tools |
| `converge-storage` | Object store abstraction (local, S3, GCS) |
| `converge-llm` | Local LLM inference (Burn, Gemma, Llama) |
| `converge-runtime` | HTTP/gRPC server, SSE transport |
| `converge-remote` | gRPC CLI client |
| `converge-application` | CLI/TUI distribution |

## Deprecated

| Crate | Status |
|---|---|
| `converge-traits` | Compatibility facade. `publish = false`. Use `converge-pack` + `converge-provider-api` instead. |

## Adding a Dependency

For pack authors:
```toml
[dependencies]
converge-pack = "3.0.1"
```

For embedded applications:
```toml
[dependencies]
converge-kernel = "3.0.1"
converge-model = "3.0.1"
```

For remote Rust consumers:
```toml
[dependencies]
converge-client = "3.0.1"
```

For provider adapters:
```toml
[dependencies]
converge-provider-api = "3.0.1"
```

If you need something that doesn't exist in any of these crates, say so. We patch Converge. We don't work around it.

See also: [[Architecture/API Surfaces]], [[Architecture/Crate Map]]
