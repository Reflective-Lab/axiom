---
tags: [architecture]
source: llm
---

# WASM Compilation

The `compile` module builds generated Rust source into WASM binaries and
manifests. WASM is an artifact format, not an ownership claim over runtime
execution.

The responsibility split is:

- Axiom derives, validates, compiles, hashes, and traces WASM-facing invariant
  artifacts.
- Helm hosts executable application plugins in a sandbox and owns plugin
  lifecycle policy.
- Converge owns governed promotion, stop reasons, invariant semantics, and run
  integrity. It does not host application plugins.

This aligns Axiom with Converge's Golden Path Matrix and Helm's foundation
contracts: executable plugin hosting lives above Converge, while Converge keeps
the promotion boundary.

## Responsibility Boundary

| Layer | Owns | Must not own |
|---|---|---|
| Axiom | Predicate extraction, Rust source generation, WASM compilation, manifest generation, source hashing, lineage, replay metadata, and proof obligations. | Plugin hosting, tenant runtime policy, authority recompute, fact promotion, or specialist execution. |
| Helm | Plugin install/upgrade/revoke, signing policy, sandbox host policy, quotas, tenant configuration, app-facing lifecycle, and adapters that map sandbox output into Converge-facing contracts. | Truth semantics, Axiom compilation rules, Converge promotion gates, or lower-layer specialist cores. |
| Converge | Kernel execution, proposal promotion, stop reasons, invariant decision semantics, HITL pauses, evidence refs, trace links, and integrity proof. | Wasmtime/Cranelift embedding, application plugin lifecycle, tenant plugin policy, or Axiom parser internals. |

## Artifact Flow

```text
JTBD or .truths source
  -> Axiom validates, extracts predicates, and compiles WASM + manifest
  -> Helm installs and runs the artifact in its sandbox under tenant policy
  -> Helm adapts sandbox output into Converge proposals or invariant verdicts
  -> Converge recomputes authority, checks promotion gates, and records integrity
  -> AxiomRunReport verifies the run against the Truth Package expectations
```

The old shorthand "deploy WASM to Converge" is no longer precise. The current
boundary is: Axiom produces the executable contract artifact, Helm hosts it,
and Converge decides whether any resulting proposal or invariant verdict is
allowed to affect the governed context.

## Pipeline

```
.truth file -> parse -> predicates -> Rust source -> cargo build --target wasm32 -> .wasm bytes + manifest
```

## Process

1. `WasmCompiler` creates a temporary Cargo project
2. Generated Rust source is written to `src/lib.rs`
3. `cargo build --target <target>` produces WASM
4. Result: `CompiledModule` with `wasm_bytes`, `manifest_json`,
   `source_hash`, and `module_name`

Compilation proves the generated artifact is structurally valid. It does not
activate the module, grant authority, or promote facts.

## Axiom Output Contract

| Output | Purpose |
|---|---|
| `wasm_bytes` | Portable executable artifact for the Helm sandbox. |
| `manifest_json` | Converge-facing invariant metadata: name, dependencies, ABI version, and governance hints. |
| `source_hash` | Deterministic source identity for caching, provenance, and replay checks. |
| `module_name` | Stable module identifier derived from the truth heading or package manifest. |
| lineage metadata | Truth Package ID, truth version, and JTBD clause IDs that produced the artifact. |

The manifest should declare the context keys and evidence categories the module
depends on. Axiom can validate those declarations before runtime; Converge
still decides whether promoted facts satisfy them during execution.

## Targets

| Target | Use case |
|---|---|
| `wasm32-unknown-unknown` | Default: no system interface, pure invariant. |
| `wasm32-wasip1` | WASI: only when Helm explicitly allows a WASI host policy. |

## Optimization Levels

| Level | Flag | Use case |
|---|---|---|
| Debug | default | Development, fast compile |
| Release | `--release` | Production |
| Size | `-C opt-level=s` | Default: minimize WASM binary size |

## Output

`CompiledModule`:

- `wasm_bytes` - the compiled binary
- `manifest_json` - embedded manifest for the runtime adapter
- `source_hash` - SHA-256 of the generated source for caching and deduplication
- `module_name` - derived from the truth heading

## Helm Sandbox Contract

Helm owns the executable sandbox because plugin lifecycle is product and tenant
policy:

- install, activate, suspend, upgrade, and revoke plugin artifacts;
- enforce module signing and package provenance requirements;
- enforce CPU, memory, wall-clock, network, filesystem, and host-call quotas;
- expose only approved host capabilities;
- emit audit events for plugin lifecycle and execution;
- adapt sandbox outputs into Converge-facing proposal or invariant contracts.

Helm must not treat a successful plugin execution as a promoted fact. It is
only evidence or a proposal until Converge accepts it.

## Converge Contract

Converge no longer embeds a WASM application plugin runtime. Its responsibility
is the governed decision boundary:

- receive proposals or invariant verdicts through public Converge contracts;
- recompute authority at promotion time;
- enforce HITL, policy, budget, and invariant stop contracts;
- attach evidence refs and trace links;
- record the integrity proof for the run.

If a WASM artifact produces an invalid result, Converge should reject or stop
honestly. If a WASM artifact produces a valid proposal, Converge still decides
whether that proposal becomes fact.

## Guest ABI

Generated modules export the Converge-facing invariant ABI:

```rust
converge_abi_version() -> "1"
converge_manifest() -> JSON string
check_invariant(ptr, len) -> i32
alloc(size) -> ptr
dealloc(ptr, size)
```

The ABI name is historical and semantic: it describes the invariant contract
that Converge relies on. It does not mean Converge owns the sandbox. Helm may
call this ABI inside `helm-plugin-runtime` and adapt the result into Converge's
public proposal or invariant surfaces.
