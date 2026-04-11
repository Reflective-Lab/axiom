---
tags: [architecture]
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
converge-domain          → core, provider       Domain packs (trust, money, delivery, ...)
converge-experience      → core                 Experience tracking
converge-knowledge       → mcp                  Knowledge management
ortools-sys              (no deps, FFI)         OR-Tools bindings
converge-optimization    → ortools-sys          Constraint solvers
converge-analytics       → core, domain         ML/analytics agents
converge-llm             → core, domain         Local LLM inference (Burn)
converge-policy          → core                 Cedar policy engine
converge-tool            → core, provider       Gherkin validation, spec tools
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
9. converge-provider
10. converge-experience
11. converge-knowledge
12. ortools-sys
13. converge-optimization
14. converge-domain
15. converge-tool

Internal crates (`publish = false`): traits, analytics, llm, policy, storage, runtime, remote, application.

## Ownership

- `converge-pack` owns the pack authoring contract (Suggestor, Context, Invariant)
- `converge-provider-api` owns the backend capability contract (Backend, Capability, Selection)
- `converge-core` owns the engine implementation (internal, not a public contract)

See also: [[Architecture/API Surfaces]], [[Architecture/Dependency Rules]], [[Architecture/Purity Rules]]
