---
tags: [architecture]
source: mixed
---

# Converge Contract

Shared stack guidance: `~/dev/work/converge/kb/Architecture/Golden Path Matrix.md`.

Axiom is a **client of Converge**. It depends on two Converge crates for live
LLM-backed validation:

| Crate | What Axiom uses |
|---|---|
| `converge-provider-api` | `DynChatBackend`, `ChatRequest`, `ChatResponse`, `ChatRole`, `ChatMessage`, `ResponseFormat`, `SelectionCriteria` |
| `converge-provider` | LLM backend implementations and selection helpers |

## Boundary

Axiom **produces** artifacts that Converge **consumes**:

- WASM invariant modules (ABI v1)
- Manifests embedded in WASM
- Policy requirements mapped to Cedar

That is the key boundary:

- Axiom consumes provider capability contracts for validation help
- Axiom produces truth artifacts and compiled modules
- Converge consumes those artifacts at runtime

Axiom does **not**:
- Run the Converge engine
- Own the convergence loop
- Execute invariants
- Manage context or facts

Helm sits above Axiom as the operator-facing truth surface. Organism and
Converge sit beneath it as the reasoning and governance layers that make those
truths operational.

## Causal Semantics Dependency

Axiom does not import Converge's logical clock or integrity primitives — they
live in `converge-core`, which is off-limits. But the **semantics** those
primitives enforce are load-bearing for Axiom's design. This section documents
what we depend on and why.

### What Converge guarantees

| Guarantee | Mechanism | Source |
|---|---|---|
| Causal ordering of facts | Lamport clock — if A caused B, `clock(A) < clock(B)` | `converge-core::integrity::LamportClock` |
| Deterministic effect merging | Effects sorted by `SuggestorId`, promoted serially | `converge-core::engine::merge_effects` |
| Immutable, append-only facts | Facts are never mutated; corrections are new facts | Axiom 3 |
| Explicit replay contract | Every fact carries a `TraceLink` — `Local` (replay-eligible) or `Remote` (audit-only) | `converge-core::types::provenance` |
| Merkle integrity | `TrackedContext` computes a root over all facts; same inputs → same root | `converge-core::integrity::MerkleRoot` |
| Three-tier replayability | `Deterministic`, `BestEffort`, `None` — system documents its own limitations | `converge-core::kernel_boundary::Replayability` |

### Why Axiom depends on these guarantees

**Invariant ordering.** Axiom's codegen produces `check_invariant()` functions
that assume all relevant facts have been promoted before the check runs. A
`CountAtLeast { key: "Strategies", min: 2 }` predicate only works if the engine
has already promoted strategy facts into the context. The Lamport clock is what
makes this ordering real — without it, invariants would race against fact
promotion.

**Cross-reference predicates.** Codegen emits checks like "for each source fact,
a corresponding target fact must exist." This assumes the target facts were
promoted in a prior causal step. If the engine merged effects in arbitrary order,
these checks would be non-deterministic.

**Manifest dependencies as causal subscriptions.** The `dependencies` array in a
WASM manifest tells the Converge engine which context keys an invariant cares
about. The engine uses this to schedule invariant evaluation *after* those keys
are populated. This is a causal subscription — Axiom declares "run me after
these facts exist."

**Deterministic simulation.** The v0.5 milestone includes "Implement
deterministic simulation (reproducible across runs)." For this to mean anything
real, Axiom must model what Converge means by "deterministic": same context +
same effects → same merge order → same Merkle root. Axiom's simulation should
validate that a spec's step graph can produce a deterministic trace under these
rules.

**Replay eligibility.** When Axiom validates a truth spec, it should warn if the
spec's design requires facts that are inherently non-replayable (e.g., a Then
step that depends on a remote API call with no local fallback). Converge's
`Local`/`Remote` distinction is the contract Axiom should validate against.

### What this means for Axiom's future

- **Simulation** should validate that a spec's causal graph has a valid
  topological ordering — every consumed fact has a producer, no cycles.
- **Codegen** should emit manifests that declare causal dependencies between
  context keys, not just flat key subscriptions.
- **Policy lens** should check that escalation paths respect the causal chain —
  an escalation target must be reachable from the current governance state.

## WASM ABI v1

Compiled modules export a fixed interface that the Converge engine calls:

```rust
converge_abi_version() → "1"
converge_manifest() → JSON string
check_invariant(ptr, len) → i32
alloc(size) → ptr
dealloc(ptr, size)
```

The engine loads the WASM, reads the manifest for metadata, and calls `check_invariant` with serialized context.
