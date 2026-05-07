---
tags: [architecture]
source: llm
---

# System Overview

Axiom is a single Rust crate (`axiom-truth`) with a library and a CLI binary (`cz`).

## Module Map

```
axiom-truth
├── gherkin        Validation (LLM + local)
├── truths         Governance block parsing
├── intent         TruthDocument → IntentPacket compilation
├── predicate      Step → predicate extraction
├── codegen        Predicate → Rust WASM source
├── compile        Rust → WASM binary
├── simulation     Pre-flight convergence analysis
├── guidance       Heading quality feedback
├── policy_lens    Cedar policy coverage
├── jtbd           Jobs-to-be-Done metadata
├── validation_view  UI-friendly result views
└── mock_llm       Test backend
```

## Pipelines

### Validation / WASM
```
.truths file
  → truths (parse governance blocks)
  → gherkin (validate: business sense + compilability + conventions)
  → simulation (pre-flight analysis)
  → predicate (extract semantic predicates from steps)
  → codegen (generate Rust WASM module)
  → compile (build to wasm32-unknown-unknown)
  → policy_lens (check Cedar coverage)
```

### Runtime intent
```
.truths file
  → truths::parse_truth_document → TruthDocument
  → intent::compile_intent       → organism_pack::IntentPacket
  → (caller hands off to organism_runtime::Runtime::admit_intent)
```

The runtime intent pipeline is the boundary with organism. Axiom owns Truth
and the conversion. Organism owns mechanism (admission gate + Converge kernel
staging) and consumes only the typed `IntentPacket`. See `Architecture/Intent
Compilation.md` for field mapping details.

## Dependencies

- **converge-provider** — chat contracts, provider capability vocabulary, and selection types
- **converge-manifold-adapters** — manifold backend selection helpers
- **organism-pack** — runtime contract types (`IntentPacket`, `Reversibility`, `ForbiddenAction`, `ExpiryAction`). Required by the `intent` module to produce runtime intents.
- **gherkin** 0.15 — Gherkin parser
- **clap** 4.5 — CLI framework
- **tokio** — async runtime

## Dependency Direction

```
axiom-truth ──depends on──▶ organism-pack
            (Truth produces IntentPacket; organism never imports Truth)
```

This was inverted in 0.8.0. Prior versions had organism depending on
axiom-truth via a bridge in `organism-intent`. That bridge moved here, taking
Truth-shaped types out of organism entirely.

## Artifacts

| Artifact | Stored at |
|---|---|
| Findings | `.converge/findings/<id>.json` |
| Acknowledgements | `.converge/acks/` |
| Escalations | `.converge/escalations/` |
| Assignments | `.converge/assignments/` |
| Compiled WASM | build output directory |
