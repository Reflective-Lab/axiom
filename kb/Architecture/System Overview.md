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
├── truth_package  JTBD decoding, Truth Packages, verifier reports, calibration
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

### Truth Package and verifier
```
JtbdInput
  → truth_package::decode_jtbd → TruthPackage
  → generated .truths + IntentPacket + verifier spec + lineage
  → Organism / Mosaic / Converge / app execution
  → AxiomRunObservation
  → AxiomRunReport::verify
  → LearningEpisode / CalibrationRecord candidates
```

This path is the release Axiom layer. It is a proof and calibration layer, not
a runtime. Organism owns formation selection, Mosaic owns concrete
capabilities, Converge owns the fixed-point engine and promotion authority,
Helm owns operator surfaces, and apps own domain state and raw transcripts.
See `Architecture/Axiom as Verifier.md` and `Architecture/API Surfaces.md`.

## Dependencies

- **converge-provider** — chat contracts, provider capability vocabulary, and selection types
- **converge-pack** — public provenance and fact payload contracts
- **converge-manifold-adapters** — manifold backend selection helpers
- **organism-pack** — runtime contract types (`IntentPacket`, `Reversibility`, `ForbiddenAction`, `ExpiryAction`). Required by the `intent` module to produce runtime intents.
- **organism-runtime** — dev/test fixture surface for proof recipes; not required by the library release surface.
- **gherkin** 0.15 — Gherkin parser
- **clap** 4.5 — CLI framework
- **tokio** — async runtime

## Dependency Direction

```
axiom-truth ──depends on──▶ organism-pack
            (Truth produces IntentPacket; organism never imports Truth)
```

This was inverted in 0.8.1. Prior versions had organism depending on
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
| Run proof report | serialized `AxiomRunReport` from caller-owned storage |
| Calibration table | caller-owned JSONL using `CalibrationTable` helpers |
