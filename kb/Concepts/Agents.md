---
tags: [concepts]
---
# Suggestors

A suggestor is a unit of capability that reads the shared context and proposes new evidence. Suggestors never mutate the context directly and never call each other ([[Philosophy/Nine Axioms#2. Convergence Over Control Flow|Axiom 2]]).

## The Trait

```rust
pub trait Suggestor: Send + Sync {
    fn name(&self) -> &str;
    fn dependencies(&self) -> &[ContextKey];
    fn accepts(&self, ctx: &dyn Context) -> bool;
    fn execute(&self, ctx: &dyn Context) -> AgentEffect;
}
```

| Method | Contract |
|---|---|
| `name()` | Unique identifier. Used for provenance, logging, and deterministic scheduling. |
| `dependencies()` | Which [[Concepts/Context and Facts#Context Keys|ContextKey]] partitions this suggestor watches. Engine only wakes the suggestor when these change. |
| `accepts()` | Pure predicate. No I/O, no side effects. "Should I run this cycle?" |
| `execute()` | Read context, produce [[Concepts/Proposals and Promotion|proposals]]. Never mutate context. |

## Rules

- `accepts()` must be **pure** — same context, same answer, every time
- `execute()` is **read-only** — suggestors return `AgentEffect`, never mutate state
- Suggestors **never call each other** — all coordination through the shared context
- **Idempotency is context-based** — check for your own facts before re-proposing

## Suggestor Taxonomy

| Type | When to use | Example |
|---|---|---|
| Rule-based | Deterministic logic, policy checks | Compliance screening |
| Analytics | Computation, scoring, aggregation | Cost analysis, risk scoring |
| LLM-backed | Interpretation, synthesis, reasoning | Capability matching, decision synthesis |
| Optimization | Constraint satisfaction, scheduling | Resource allocation, slot optimization |
| Governance | Audit, access control, provenance | Session validation, compliance scanning |

No type is privileged. The engine treats all suggestors identically. An LLM-backed suggestor has no special status compared to a rule-based one.

See also: [[Building/Writing Agents]], [[Concepts/Context and Facts]], [[Philosophy/Convergence Explained]]
