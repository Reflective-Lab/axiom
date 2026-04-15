---
tags: [philosophy]
source: llm
---

# Truth-Driven Development

Truth-Driven Development (TDD — intentionally overloaded) is the methodology behind Axiom. Business invariants are expressed as `.truths` files, validated, compiled to WASM, and deployed to the Converge engine.

## The Flow

```
Business intent → .truths file → Axiom validation → WASM invariant → Converge engine
```

1. A domain expert writes a `.truths` file expressing a business invariant
2. Axiom validates the spec (business sense, compilability, conventions)
3. Axiom simulates convergence readiness
4. Axiom compiles the spec to a WASM module
5. The WASM module is deployed to Converge as an invariant or suggestor

## What Makes a Good Truth

A truth is a **governed business invariant**, not a process description.

**Good**: "Enterprise AI vendor selection is auditable, constrained, and approval-gated"
**Bad**: "AI Vendor Selection Process"

The heading should declare what must be true, not what happens. Axiom's [[Concepts/Guidance|guidance module]] helps authors improve heading quality.

## Governance Blocks

Every truth file can declare governance metadata that flows through to Cedar policies and WASM manifests. See [[Concepts/Truth Documents]] for the full format.
