---
tags: [architecture]
source: mixed
---
# Crate Map

All crates live under `crates/`. The workspace root `Cargo.toml` centralizes versions, dependencies, lints, and profiles.

## Public Contract Crates

Six crates form the supported external API. See [[Architecture/API Surfaces]] for the full contract.

```
converge-pack            (no internal deps)     Pack authoring: Suggestor, Invariant
converge-provider-api    (no internal deps)     Backend identity, capability routing
converge-model           → core, pack           Curated semantic types
converge-kernel          → core, pack           In-process embedding API
converge-protocol        (no internal deps)     Generated converge.v1 wire types
converge-client          → protocol             Remote Rust SDK
```

## Deprecated

```
converge-traits          → pack, provider-api   Compatibility facade (publish = false)
```

No new code may depend on `converge-traits`. Use `converge-pack` and `converge-provider-api` instead.

## Internal Crates

```
converge-core            → pack                 Engine, context, promotion gates
converge-mcp             (no internal deps)     MCP server/client
converge-provider        → core, pack,          LLM provider adapters
                           provider-api, mcp
converge-domain          → core, policy,        Domain packs (trust, money, delivery, ...)
                           provider             including governed flow transitions
converge-experience      → core                 Experience tracking
converge-knowledge       → mcp                  Knowledge management
ortools-sys              (no deps, FFI)         OR-Tools bindings
converge-optimization    → ortools-sys          Constraint solvers
converge-analytics       → core, domain         ML/analytics agents
converge-llm             → core, domain         Local LLM inference (Burn)
converge-policy          → core                 Cedar policy engine and default FlowGateAuthorizer
converge-axiom            → core, provider       Gherkin validation, spec tools
converge-auth            (no internal deps)     Authentication, authorization, cryptography
converge-consensus       (no internal deps)     Raft consensus adapter
converge-ledger          (no internal deps)     Append-only context ledger
converge-nats            (no internal deps)     NATS messaging adapter
converge-observability   (no internal deps)     Audit, telemetry, and metrics
converge-storage         (no internal deps)     Object store abstraction
converge-remote          → client, protocol     gRPC CLI client
converge-runtime         → core, provider,      HTTP/gRPC server, SSE
                           tool, protocol
converge-application     → core, provider,      CLI/TUI distribution
                           domain, tool, mcp
```

## Publish Order (crates.io)

1. converge-pack
2. converge-provider-api
3. converge-core
4. converge-mcp
5. converge-model
6. converge-kernel
7. converge-protocol
8. converge-client
9. converge-storage
10. converge-provider
11. converge-experience
12. converge-knowledge
13. ortools-sys
14. converge-optimization
15. converge-domain
16. converge-axiom

Internal crates (`publish = false`): traits, analytics, auth, consensus, ledger, llm, nats, observability, policy, runtime, remote, application.

## Ownership

- `converge-pack` owns the pack authoring contract (Suggestor, Context, Invariant)
- `converge-provider-api` owns the backend capability contract (Backend, Capability, Selection)
- `converge-core` owns the engine implementation (internal, not a public contract)
- `converge-core` also owns the neutral flow gate contract (`FlowGateAuthorizer`, `FlowGateInput`)
- `converge-policy` owns the default Cedar implementation of that contract
- `converge-domain` applies that contract to built-in governed pack actions

See also: [[Architecture/API Surfaces]], [[Architecture/Dependency Rules]], [[Architecture/Purity Rules]]
