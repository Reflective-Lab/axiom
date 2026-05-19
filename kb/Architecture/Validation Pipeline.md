---
tags: [architecture]
source: codex
---

# Validation Pipeline

This page is the contract for moving a `.truths` document from source text to
runtime-ready artifacts. It defines module order, inputs, outputs, determinism,
and failure boundaries.

## Contract

The validation pipeline has three stages:

1. **Parsing** — `truths` extracts governance declarations and returns a
   Gherkin-compatible body; `gherkin` parses scenarios and scenario metadata.
2. **Validation** — `gherkin`, `guidance`, `simulation`, and `policy_lens`
   produce deterministic local findings plus optional provider-backed feedback.
3. **Codegen** — `predicate`, `codegen`, and `compile` translate valid
   invariant scenarios into Rust/WASM artifacts and verify compilability.

Parsing must complete before any validation or code generation. Codegen must
only consume parsed and validated structures, never raw editor text.

## Stage 1: Parsing

Input:

- Raw `.truths`, `.truth`, or `.feature` source text.

Responsibilities:

- `truths::parse_truth_document` parses governance blocks:
  `Intent`, `Authority`, `Constraint`, `Evidence`, and `Exception`.
- `gherkin::preprocess_truths` maps `Truth:` to `Feature:` for parser
  compatibility.
- `gherkin::Feature::parse` validates Gherkin syntax.
- `gherkin::extract_all_metas` extracts scenario tags into `ScenarioMeta`.

Output:

- `TruthDocument`
- Gherkin feature/scenario structure
- `ScenarioMeta` values

Failure boundary:

- Syntax errors and malformed governance fields are hard parse failures.
- Downstream stages must not run against an unparsed document.

## Stage 2: Validation

Input:

- `TruthDocument`
- Parsed Gherkin scenarios
- `ScenarioMeta`

Responsibilities:

- `gherkin` checks scenario structure, local conventions, optional business
  sense, and optional compilability.
- `guidance` provides authoring feedback for names and headings.
- `simulation` performs deterministic pre-flight convergence analysis.
- `policy_lens` checks whether governance requirements are represented by Cedar
  policy coverage.

Output:

- `SpecValidation`
- `SimulationReport`
- `PolicyCoverageReport`
- UI-friendly summaries from `validation_view`

Failure boundary:

- Local validation findings are reported as structured issues, not panics.
- Provider failures are surfaced as validation errors outside the Gherkin issue
  categories.
- Simulation must remain reproducible: the same parsed document and config must
  produce the same verdict, findings, resources, domain profile reports, and
  deterministic trace hash.

## Stage 3: Codegen

Input:

- Validated scenario metadata
- Extracted predicates
- Governance metadata

Responsibilities:

- `predicate` extracts checkable predicates from steps.
- `codegen` emits Rust invariant source and manifests.
- `compile` verifies generated code compiles for `wasm32-unknown-unknown`.

Output:

- Generated Rust invariant source
- WASM manifest metadata
- Compiled WASM module

Failure boundary:

- Generated code is read-only and regenerated from specs.
- Compilation failures are codegen failures, not runtime failures.
- A Truth must not cross into Converge or Organism until codegen and
  compilability checks pass.

## Determinism

The deterministic contract is:

```text
same parsed TruthDocument + same SimulationConfig
  -> same normalized scenario trace
  -> same simulation findings and resource summaries
  -> same trace hash
```

Simulation models Converge replay eligibility before runtime. It must flag
non-replayable scenario language such as live network calls, current time,
randomness, or "latest" external state and suggest declaring replayable evidence
snapshots. It must also canonicalize output ordering so repeated runs do not
drift because of traversal or collection order.

## Runtime Boundary

The pipeline ends at Axiom's public artifacts:

- `TruthDocument`
- `SpecValidation`
- `SimulationReport`
- generated WASM invariants
- `IntentPacket` from `intent::compile_intent`

Helm hosts executable WASM artifacts when a sandboxed plugin is needed.
Organism consumes `IntentPacket`. Converge consumes proposals, invariant
verdicts, evidence refs, trace links, and typed provider/runtime contracts.
Neither runtime should depend on raw `.truths` source text or Axiom-internal
parser state.
