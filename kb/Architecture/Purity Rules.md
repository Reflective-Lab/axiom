---
tags: [architecture]
source: mixed
---
# Purity Rules

`converge-core` is a pure crate. If a module implies execution, I/O, network, model inference, or persistence, it does not belong in converge-core.

## Allowed in converge-core

- `thiserror` — error types
- `tracing` — structured logging (no I/O)
- `serde`, `serde_json` — serialization
- `typed-builder` — builder pattern
- `hex`, `sha2` — content hashing
- Small pure utility libraries

## Forbidden in converge-core

| Category | Examples | Why |
|---|---|---|
| Async runtime | `tokio`, `async-std` | Introduces execution model |
| Networking | `reqwest`, `axum`, `tonic` | Implies I/O |
| Model inference | `burn`, `llama-burn` | External system |
| Persistence | `lancedb`, `surrealdb`, `postgres` | I/O and state |
| Non-determinism | `rand`, `rayon` | Violates [[Philosophy/Nine Axioms#6. Transparent Determinism|Axiom 6]] |

## The Test

Before adding a dependency to converge-core, ask:

1. Does it perform I/O? → No
2. Does it introduce async? → No
3. Does it add non-determinism? → No
4. Could the engine produce different results with this dep? → No

If any answer is yes, the dependency belongs in a different crate.

See also: [[Architecture/Dependency Rules]], [[Architecture/Crate Map]]
