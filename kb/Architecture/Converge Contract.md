---
tags: [architecture]
source: llm
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
