---
tags: [concepts]
source: llm
---

# Code Generation

The `codegen` module transforms validated predicates and governance metadata into Rust WASM modules that implement the Converge ABI v1.

## Pipeline

```
ScenarioMeta + Predicates + Governance → ManifestBuilder → manifest JSON
                                        → generate_invariant_module() → Rust source
```

## Generated Module Shape

Each generated module exports:

| Export | Purpose |
|---|---|
| `converge_abi_version()` | Returns `"1"` |
| `converge_manifest()` | Returns manifest JSON (name, kind, class, deps, capabilities) |
| `alloc()` / `dealloc()` | WASM memory management |
| `check_invariant()` | The generated check expression from predicates |

## Manifest Contents

- **name** — derived from truth heading
- **kind** — Invariant or Suggestor (from scenario tags)
- **invariant_class** — Structural, Semantic, or Acceptance
- **dependencies** — context keys referenced by predicates
- **capabilities** — ReadContext, Log
- **requires_human_approval** — from Authority block
- **metadata** — governance blocks encoded as strings

## Compilation

Generated Rust source is compiled to WASM by the `compile` module. See [[Architecture/WASM Compilation]] for build details.
