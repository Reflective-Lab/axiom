---
tags: [architecture]
---
# API Surfaces

Converge exposes six external contracts. If a type is not reachable through one of these, it is an implementation detail.

The canonical reference is `architecture/API_SURFACES.md` in the repo root. Stabilization decision: `architecture/adr/ADR-004-contract-stabilization.md`.

## The Six Public Crates

| Crate | Purpose | May depend on |
|---|---|---|
| `converge-pack` | Author packs, suggestors, invariants | nothing internal |
| `converge-provider-api` | Backend identity, capability routing | nothing internal |
| `converge-model` | Curated semantic types | pack |
| `converge-kernel` | In-process embedding API | core, pack |
| `converge-protocol` | Generated `converge.v1` wire types | nothing internal |
| `converge-client` | Remote Rust SDK | protocol |

## Who Uses What

| Consumer | Allowed Dependencies |
|---|---|
| Pack/module authors | `converge-pack`, `converge-model` |
| Embedded applications | `converge-kernel`, `converge-model`, `converge-pack` |
| Provider adapters | `converge-provider-api` |
| Remote Rust consumers | `converge-client`, `converge-protocol` |
| Non-Rust consumers | `converge.v1` protobuf/gRPC |

## Downstream Mapping

| Project | Target API |
|---|---|
| organism | `converge-pack` + `converge-model` |
| saas-killer | `converge-kernel` + `converge-model` |
| wolfgang | `converge-provider-api` + `converge-client` |

## Contract Status

The breaking cut is complete:
- `Suggestor` is the public authoring trait (replaces `Agent`)
- `AgentEffect` is proposal-only (no `facts` field)
- `Fact` has no public constructor — `Fact::new()` and `Fact::with_promotion()` are gated behind `kernel-authority` and not re-exported
- `SubmitObservationRequest` replaces `InjectFactRequest` in the wire protocol

`converge-traits` is deprecated (`publish = false`). It re-exports from `converge-pack` and `converge-provider-api` for compatibility. No new code may depend on it.

## Regression Gate

Any PR touching pack/core/protocol/client/runtime must pass:
```bash
cargo test -p converge-pack --test compile_fail
cargo test -p converge-core --test compile_fail --test truth_pipeline --test negative --test properties
cargo test -p converge-client --test messages
```

## Semver Scope

Semver promises apply only to the six public crates and the `converge.v1` wire protocol. Everything else is internal and may change without notice.

See also: [[Architecture/Crate Map]], [[Architecture/Hexagonal Architecture]], [[Architecture/Known Drift]]
