---
tags: [architecture]
source: mixed
---

# Converge Contract

Shared stack guidance: `~/dev/reflective/stack/bedrock-platform/converge/kb/Architecture/Golden Path Matrix.md`.

Axiom is a **client of Converge**. It depends on the public provider surface
for live LLM-backed validation and on manifold for backend selection:

| Crate | What Axiom uses |
|---|---|
| `converge-provider` | `DynChatBackend`, `ChatRequest`, `ChatResponse`, `ChatRole`, `ChatMessage`, `ResponseFormat`, `SelectionCriteria` |
| `converge-manifold-adapters` | `manifold::select_healthy_chat_backend` for concrete backend selection; enable the `llm-all` feature for chat backend helpers |

## Boundary

Axiom **produces** artifacts that Helm, Converge, and Organism **consume**
through their own boundaries:

- WASM invariant modules and manifests for Helm's plugin runtime
- Policy requirements mapped to Cedar
- `IntentPacket` values consumed by Organism's runtime
- Verifier expectations for `AxiomRunReport`

That is the key boundary:

- Axiom consumes provider capability contracts and selection helpers for validation help
- Axiom produces truth artifacts, compiled modules, and verifier expectations
- Helm hosts executable WASM artifacts in a sandbox and adapts their outputs
  into public runtime contracts
- Organism consumes intent and assembles formations
- Converge consumes proposals, invariant verdicts, evidence refs, and trace
  links through public kernel/pack contracts

Axiom does **not**:
- Own the Converge engine
- Own the convergence loop
- Execute invariants
- Manage context or facts

Helm sits above Axiom as the operator-facing truth surface and executable
plugin sandbox. Organism and Converge sit beneath it as the reasoning and
governance layers that make those truths operational. Converge does not embed
the application plugin runtime; it owns the decision about whether plugin output
can affect governed context.

For v0.9, Axiom may drive a proof run through Organism's public runtime surface.
That does not move ownership. Organism still selects and instantiates
Formations; Converge still runs the fixed-point loop and returns the stop
reason, promoted facts, and integrity proof. Axiom packages those outputs into
an auditable explanation.

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
WASM manifest tells the Converge-facing runtime adapter which context keys an
invariant cares about. Even when Helm hosts the module, Converge uses that
declaration at the decision boundary: invariant verdicts must be evaluated
against the facts they claim to depend on. This is a causal subscription —
Axiom declares "judge this only after these facts exist."

**Deterministic simulation.** The current milestone includes "Implement
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

Compiled modules export a fixed interface for Converge-facing invariant
checks. Helm may host and call this ABI inside its sandbox; Converge consumes
the adapted proposal or invariant verdict through public runtime contracts:

```rust
converge_abi_version() → "1"
converge_manifest() → JSON string
check_invariant(ptr, len) → i32
alloc(size) → ptr
dealloc(ptr, size)
```

The important contract is semantic, not hosting: the manifest declares what the
artifact needs, the invariant check returns a verdict, and Converge remains the
promotion and stop-reason authority.
