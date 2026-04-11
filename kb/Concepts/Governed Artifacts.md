---
tags: [concepts]
---
# Governed Artifacts

`converge_core::governed_artifact` provides lifecycle management for system outputs that become operational artifacts.

## State Machine

```
Draft → UnderReview → Approved → Active
Active → Suspended → Retired
Any state → RolledBack (with severity and impact tracking)
```

The state machine enforces valid transitions and tracks who changed what ([[Philosophy/Nine Axioms#1. Explicit Authority|Axiom 1]]).

## When to Use

Use when agent outputs become artifacts that outlive the convergence run:

- Approved vendor lists
- Policy documents
- Compliance certificates
- Decision records with formal lifecycle

See also: [[Concepts/Experience and Recall]], [[Concepts/Proposals and Promotion]]
