---
tags: [architecture, api]
source: llm
---

# API Surfaces

This page is the authoritative API reference for Axiom's public boundaries.
When it conflicts with older root-level docs, this page wins.

## Runtime intent

Axiom owns Truth-shaped input. Organism owns runtime mechanism. The public flow
is:

```rust
use axiom_truth::{compile_intent, parse_truth_document};

let truth = parse_truth_document(source)?;
let intent = compile_intent(&truth)?;
let receipt = runtime.admit_intent(&intent, actor, src, &mut ctx)?;
```

Pipeline:

```text
.truths source
  -> truths::parse_truth_document
  -> TruthDocument
  -> intent::compile_intent
  -> organism_pack::IntentPacket
  -> organism_runtime::Runtime::admit_intent
```

`compile_intent_from_source(&str)` is the convenience API for callers that do
not need to inspect the intermediate `TruthDocument`.

Axiom does not re-export `IntentPacket`, `ExpiryAction`, `ForbiddenAction`, or
`Reversibility`. Those types are defined by `organism-pack`; callers that need
to name them should depend on `organism-pack` directly. The ownership story is:
Axiom produces the packet, organism-pack defines it, organism-runtime admits it.

## Truth parsing

| API | Purpose |
|---|---|
| `parse_truth_document(&str)` | Parse `.truths` source into `TruthDocument`. |
| `TruthDocument` | Parsed Gherkin body plus governance blocks. |
| `TruthGovernance` | Parsed `Intent`, `Authority`, `Constraint`, `Evidence`, and `Exception` blocks. |

## Validation and feedback

| Module | Public surface |
|---|---|
| `gherkin` | `GherkinValidator`, `ValidationConfig`, `SpecValidation`, validation issue types. |
| `guidance` | `suggest_guidance`, `GuidanceRequest`, `GuidanceResponse`. |
| `simulation` | `simulate`, `simulate_spec`, `SimulationConfig`, `SimulationReport`, optional `DomainProfile`s. |
| `policy_lens` | `check_coverage`, `PolicyCoverageReport`, Cedar coverage vocabulary. |

## Compilation

| Module | Public surface |
|---|---|
| `predicate` | Gherkin step to semantic predicate extraction. |
| `codegen` | Predicate to Rust invariant source generation. |
| `compile` | Rust invariant source to WASM module compilation. |
| `intent` | Truth governance to `organism_pack::IntentPacket` compilation. |

The WASM path and the runtime intent path are separate outputs from the same
Truth source. WASM invariants are generated artifacts for Converge. Intent
packets are runtime admission inputs for organism.

## External crates

| Crate | Role |
|---|---|
| `converge-provider` | Chat backend traits, request/response types, and provider selection vocabulary used by validation and guidance. |
| `converge-manifold-adapters` | Concrete backend selection helper (`manifold::select_healthy_chat_backend`). |
| `organism-pack` | Defines `IntentPacket` and related runtime contract types consumed by organism. |

No Axiom public API depends on Converge internal crates such as
`converge-core`, `converge-runtime`, or `converge-analytics`.
