---
tags: [building]
---
# Context Keys

Facts are partitioned by `ContextKey`. Agents declare which keys they depend on — the engine only wakes agents when their dependencies change.

| Key | Purpose | Example facts |
|---|---|---|
| `Seeds` | Initial evidence, screening results | `compliance:screen:acme` |
| `Hypotheses` | Tentative conclusions, intermediate analysis | `hypothesis:acme-best-fit` |
| `Strategies` | Action plans, recommendations | `strategy:phased-rollout` |
| `Constraints` | Limitations, boundary conditions | `constraint:budget-cap-50k` |
| `Signals` | Observations, environmental data | `signal:market-trend-q2` |
| `Competitors` | Competitive intelligence | `competitor:beta-ml-pricing` |
| `Evaluations` | Scored assessments, rankings, decisions | `evaluation:cost:acme`, `decision:recommendation` |
| `Proposals` | LLM-generated suggestions awaiting validation | `proposal:vendor-shortlist` |
| `Diagnostic` | Debugging info | `diagnostic:cycle-timing` |

## Key Rules

- `Diagnostic` never blocks convergence — it's for debugging only
- Context keys are **plural**: `ContextKey::Seeds`, not `ContextKey::Seed`
- Agents should write to keys that match their output semantics — screening results go in `Seeds`, not `Evaluations`

## How the Cascade Works

```
Cycle 1: Agent with no dependencies runs → writes to Seeds
Cycle 2: Agent watching Seeds wakes up → writes to Evaluations
Cycle 3: Agent watching Evaluations wakes up → writes to Strategies
Cycle 4: No changes → convergence
```

This is how agents coordinate without calling each other. The engine manages the cascade through key dependencies.

See also: [[Concepts/Context and Facts]], [[Building/Writing Agents]]
