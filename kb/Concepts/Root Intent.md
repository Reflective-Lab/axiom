---
tags: [concepts]
---
# Root Intent

A `RootIntent` defines the scope and success criteria for a convergence run. Nothing outside the root intent may exist in the context.

## Structure

Two intent types exist:

**`RootIntent`** (in `root_intent.rs`) — the primary type, constructed with builder methods:

```rust
pub struct RootIntent {
    pub id: IntentId,
    pub kind: IntentKind,
    pub objective: Option<Objective>,
    pub scope: Scope,
    pub constraints: Vec<IntentConstraint>,
    pub success_criteria: SuccessCriteria,
    pub budgets: Budgets,
}

let intent = RootIntent::new(IntentKind::Evaluation)
    .with_objective(objective)
    .with_scope(scope)
    .with_constraint(constraint);
```

**`TypesRootIntent`** (in `types/intent.rs`) — the governed type, uses `typed-builder`:

```rust
TypesRootIntent::builder()
    .id("vendor-evaluation-2026")
    .kind(IntentKind::Evaluation)
    .build()
```

## Budget

Two budget types:

**`Budget`** (engine-level) — controls the convergence loop:

```rust
pub struct Budget {
    pub max_cycles: u32,
    pub max_facts: u32,
}
```

**`Budgets`** (intent-level) — richer, includes optional token and time limits:

```rust
pub struct Budgets {
    pub max_cycles: u32,
    pub max_agents_per_cycle: Option<u32>,
    pub max_facts: u32,
    pub time_limit: Option<Duration>,
    pub max_tokens: Option<u64>,
}
```

Budget exhaustion is not failure — it is an honest report ([[Philosophy/Nine Axioms#8. No Hidden Work|Axiom 8]]).

## Scope Rule

The root intent defines the universe of discourse. Agents that propose facts outside the declared scope are rejected by the governance gate. This prevents scope creep during execution.

See also: [[Philosophy/Convergence Explained]], [[Concepts/Invariants]]
