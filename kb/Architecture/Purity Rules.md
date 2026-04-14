---
tags: [architecture]
source: mixed
---
# Purity Rules

`converge-core` is a pure crate. If a module implies I/O, network access, model inference, persistence, hidden background execution, or runtime ownership, it does not belong in converge-core.

## Allowed in converge-core

- `thiserror` — error types
- `tracing` — structured logging (no I/O)
- `serde`, `serde_json` — serialization
- `typed-builder` — builder pattern
- `hex`, `sha2` — content hashing
- Runtime-agnostic async helpers such as `async-trait`
- Small pure utility libraries

## Forbidden in converge-core

| Category | Examples | Why |
|---|---|---|
| Async runtime / executor | `tokio`, `async-std` | Owns scheduling and task execution |
| Networking | `reqwest`, `axum`, `tonic` | Implies I/O |
| Model inference | `burn`, `llama-burn` | External system |
| Persistence | `lancedb`, `surrealdb`, `postgres` | I/O and state |
| Non-determinism | `rand`, `rayon` | Violates [[Philosophy/Nine Axioms#6. Transparent Determinism|Axiom 6]] |

Async signatures are allowed in `converge-core` as long as they are runtime-agnostic. The rule is not "must be synchronous"; the rule is "must not own or assume a runtime."

## The Test

Before adding a dependency to converge-core, ask:

1. Does it perform I/O? → No
2. Does it require or smuggle in a runtime/executor? → No
3. Does it add non-determinism? → No
4. Could the engine produce different results with this dep? → No

If any answer is yes, the dependency belongs in a different crate.

See also: [[Architecture/Dependency Rules]], [[Architecture/Crate Map]]
