---
tags: [architecture]
source: mixed
---
# Type Protocol

The rules for core types. These are not conventions — they are the contract. Breaking them breaks downstream consumers.

## Rules

- `ProposedFact` carries `confidence: f64` and `provenance: String` — always
- `AgentEffect` is a struct `{ proposals }` — suggestors do not emit authoritative facts
- `Suggestor::accepts` and `Suggestor::execute` take `&dyn Context`, not `&Context`
- `Suggestor::dependencies` returns `&[ContextKey]`, not `Vec<&str>`
- `Fact` is read-only for external code and is created only by kernel-authority helpers
- `CriterionResult` is four-way: `Met { evidence }`, `Blocked { reason, approval_ref }`, `Unmet { reason }`, `Indeterminate`
- `CriterionEvaluator::evaluate` takes one `&Criterion` at a time, not a list
- `TypesRootIntent` uses the builder pattern: `TypesRootIntent::builder().id(...).kind(...).build()`
- Context keys are plural: `ContextKey::Seeds`, `ContextKey::Evaluations`, `ContextKey::Diagnostic` — not `Seed`, `Evaluation`

## Type Ownership

- `converge-pack` owns the canonical authoring types
- `converge-provider-api` owns backend identity and routing contracts
- `converge-model` owns curated governed semantic types
- `converge-kernel` is the supported in-process embedding surface
- Downstream crates import from the six public crates, not from `converge-core`

## ID Newtypes

All IDs are newtypes, not raw strings: `FactId`, `ProposalId`, `ObservationId`, `FrameId`, `TensionId`, `GateId`.

## Provenance

Two provenance types:
- `LocalTrace` — replay-eligible, deterministic, reproducible
- `RemoteRef` — audit-only, not replayable (e.g., external API call)

This distinction is explicit ([[Philosophy/Nine Axioms#6. Transparent Determinism|Axiom 6]]).

See also: [[Concepts/Proposals and Promotion]], [[Architecture/Purity Rules]]
