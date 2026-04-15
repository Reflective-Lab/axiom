---
tags: [philosophy]
source: llm
---

# Why Axiom

Multi-agent systems execute business logic autonomously. If a specification is wrong — ambiguous, underspecified, or violating policy — the system will faithfully execute the wrong thing. Catching errors at runtime is expensive. Catching them before execution is cheap.

Axiom exists to **validate specifications before they reach the engine**.

## The Problem

Business specifications in Converge are expressed as `.truths` files — Gherkin-extended documents with governance declarations. These files define invariants, suggestor behavior, and policy constraints. Before Axiom, there was no structured way to check whether a truth file:

1. **Makes business sense** — does it describe a real invariant, not just a process?
2. **Is compilable** — can it be translated to a Rust WASM module?
3. **Follows conventions** — does it use the right tags, structure, and governance blocks?
4. **Has policy coverage** — are all gated actions covered by Cedar policies?
5. **Will converge** — will the engine reach a fixed point, or will it loop?

## The Solution

Axiom applies a layered validation pipeline:

1. **Parse** — extract governance blocks and Gherkin scenarios
2. **Validate** — LLM-powered business sense and compilability checks, local convention checks
3. **Simulate** — pre-flight analysis for convergence readiness
4. **Generate** — produce Rust WASM invariant code from predicates
5. **Compile** — build to `wasm32-unknown-unknown`
6. **Analyze** — Cedar policy coverage gap detection

Each layer catches a different class of error, and each runs independently.

## Design Decisions

- **LLM-powered where judgment is needed** — business sense and compilability require reasoning, not pattern matching
- **Local where rules suffice** — convention checks and predicate parsing are deterministic
- **Fallback paths everywhere** — every LLM check has an offline heuristic fallback
- **Audit trail** — findings are stored as append-only JSON artifacts in `.converge/`
