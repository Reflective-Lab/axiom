---
tags: [concepts]
source: mixed
---
# Invariants

Invariants are executable guarantees, not validation functions. They define what must always be true about the context ([[Philosophy/Nine Axioms#5. Safety by Construction|Axiom 5]]).

## The Trait

```rust
pub trait Invariant: Send + Sync {
    fn name(&self) -> &str;
    fn class(&self) -> InvariantClass;
    fn check(&self, ctx: &dyn Context) -> InvariantResult;
}
```

## Invariant Classes

| Class | When checked | Failure effect | Example |
|---|---|---|---|
| `Structural` | Every merge operation | Merge rejected immediately | "No duplicate fact IDs" |
| `Semantic` | End of each cycle | Convergence blocked this cycle | "Every strategy references a signal" |
| `Acceptance` | When convergence is claimed | Result rejected even if fixed point reached | "At least 3 independent strategies exist" |

## Results

```rust
pub enum InvariantResult {
    Ok,
    Violation(String),
}
```

A violation is not a silent log entry. Structural violations reject the merge. Semantic violations prevent convergence. Acceptance violations reject the entire result. The system cannot claim success while violating its own guarantees.

## Domain Pack Invariants

[[Concepts/Domain Packs|Domain packs]] ship their own invariants:

- `AllActionsAuditedInvariant` — every access decision has an audit entry
- `AuditImmutabilityInvariant` — audit entries are marked immutable
- `ViolationsHaveRemediationInvariant` — open violations have remediation plans

See also: [[Concepts/Context and Facts]], [[Philosophy/Nine Axioms]]
