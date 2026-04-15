---
tags: [architecture]
source: llm
---

# WASM Compilation

The `compile` module builds generated Rust source into WASM binaries.

## Pipeline

```
.truth file → parse → predicates → Rust source → cargo build --target wasm32 → .wasm bytes
```

## Process

1. `WasmCompiler` creates a temporary Cargo project
2. Generated Rust source is written to `src/lib.rs`
3. `cargo build --target <target>` produces WASM
4. Result: `CompiledModule` with wasm_bytes, manifest_json, source_hash, module_name

## Targets

| Target | Use case |
|---|---|
| `wasm32-unknown-unknown` | Default — no system interface, pure invariant |
| `wasm32-wasip1` | WASI — when system access is needed |

## Optimization Levels

| Level | Flag | Use case |
|---|---|---|
| Debug | default | Development, fast compile |
| Release | `--release` | Production |
| Size | `-C opt-level=s` | Default — minimize WASM binary size |

## Output

`CompiledModule`:
- `wasm_bytes` — the compiled binary
- `manifest_json` — embedded manifest for the engine
- `source_hash` — SHA-256 of the generated source (for caching/dedup)
- `module_name` — derived from the truth heading
