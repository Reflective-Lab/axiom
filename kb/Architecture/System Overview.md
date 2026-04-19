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

## Pipeline

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

## Dependencies

- **converge-provider-api** — chat capability contracts and selection vocabulary
- **converge-provider** — LLM provider implementations and selection helpers
- **gherkin** 0.14 — Gherkin parser
- **clap** 4.5 — CLI framework
- **tokio** — async runtime

## Artifacts

| Artifact | Stored at |
|---|---|
| Findings | `.converge/findings/<id>.json` |
| Acknowledgements | `.converge/acks/` |
| Escalations | `.converge/escalations/` |
| Assignments | `.converge/assignments/` |
| Compiled WASM | build output directory |
