---
tags: [index]
source: llm
---

# Axiom — Entity Index

## Modules

| Entity | Description | Location |
|---|---|---|
| gherkin | LLM-powered validation of `.truths` specs | `src/gherkin.rs` |
| truths | Governance block parsing (Intent, Authority, Constraint, Evidence, Exception) | `src/truths.rs` |
| codegen | WASM invariant code generation from predicates | `src/codegen.rs` |
| compile | Rust → WASM compilation pipeline | `src/compile.rs` |
| predicate | Gherkin step → semantic predicate extraction | `src/predicate.rs` |
| simulation | Pre-flight convergence readiness analysis | `src/simulation.rs` |
| guidance | LLM + heuristic heading quality feedback | `src/guidance.rs` |
| policy_lens | Cedar policy coverage analysis | `src/policy_lens.rs` |
| jtbd | Jobs-to-be-Done metadata extraction | `src/jtbd.rs` |
| validation_view | UI-friendly validation result transformation | `src/validation_view.rs` |
| mock_llm | Static chat backend for tests and offline use | `src/mock_llm.rs` |

## CLI Commands (cz)

| Command | Purpose |
|---|---|
| `cz doctor` | Environment health check |
| `cz bootstrap` | Dev environment setup |
| `cz validate` | Validate `.truths` files |
| `cz digest` | Summarize open findings |
| `cz ack` | Acknowledge a finding |
| `cz escalate` | Escalate a finding |
| `cz assign` | Assign a finding |
| `cz test` | Run tests |
| `cz fmt` | Format code |
| `cz lint` | Run clippy |
| `cz ci` | Full CI locally |
| `cz up` / `cz down` | Start/stop services |

## Key Types

| Type | Module | Purpose |
|---|---|---|
| `SpecValidation` | gherkin | Complete validation result with issues, confidence, governance |
| `ScenarioMeta` | gherkin | Parsed scenario tags (kind, invariant class, provider) |
| `TruthDocument` | truths | Parsed `.truths` file with Gherkin + governance |
| `TruthGovernance` | truths | Intent, Authority, Constraint, Evidence, Exception blocks |
| `CompiledModule` | compile | WASM bytes + manifest + source hash |
| `Predicate` | predicate | Semantic predicate extracted from Gherkin steps |
| `SimulationReport` | simulation | Pre-flight analysis with verdict and findings |
| `PolicyCoverageReport` | policy_lens | Cedar coverage: covered vs uncovered actions |
| `GuidanceResponse` | guidance | Suggested title, rewrite flag, rationale |
| `JTBDMetadata` | jtbd | Actor, jobs (functional/emotional/relational), metrics |

## Scenario Tags

| Tag | Meaning |
|---|---|
| `@invariant` | Scenario is an invariant check |
| `@structural` | Structural invariant class |
| `@semantic` | Semantic invariant class |
| `@acceptance` | Acceptance invariant class |
| `@id:name` | Named identifier |
| `@llm` | Requires LLM provider |
| `@test` | Test-only scenario |
