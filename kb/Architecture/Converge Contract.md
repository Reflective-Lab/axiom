---
tags: [architecture]
source: llm
---

# Converge Contract

Axiom is a **client of Converge**. It depends on two Converge crates:

| Crate | What Axiom uses |
|---|---|
| `converge-core` | `DynChatBackend` trait, `ChatRequest`, `ChatResponse`, `ChatRole`, `ChatMessage`, `ResponseFormat` |
| `converge-provider` | LLM backend implementations (Anthropic, OpenAI, etc.) |

## Boundary

Axiom **produces** artifacts that Converge **consumes**:

- WASM invariant modules (ABI v1)
- Manifests embedded in WASM
- Policy requirements mapped to Cedar

Axiom does **not**:
- Run the Converge engine
- Own the convergence loop
- Execute invariants
- Manage context or facts

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
