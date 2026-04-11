---
tags: [concepts]
---
# Human-in-the-Loop Gates

Human review is a first-class concept, not a workaround ([[Philosophy/Nine Axioms#7. Human Authority First-Class|Axiom 7]]). The engine pauses convergence when a decision requires human judgment.

## How Gating Works

```rust
let hitl = HitlPolicy::gate_all()
    .with_timeout(TimeoutPolicy {
        duration_secs: 3600,
        action: TimeoutAction::Reject,
    });
```

When a gated proposal arrives, the engine pauses and emits a `GateRequest`. Your application presents it to a reviewer and collects a `GateDecision`:

```rust
GateDecision {
    verdict: GateVerdict::Approved,
    reason: "CFO reviewed and approved".into(),
    reviewer: "jane.doe@company.com".into(),
}
```

The engine resumes with `engine.resume()`.

## Criterion-Level Blocking

Signal that a criterion itself needs human input:

```rust
CriterionResult::Blocked {
    reason: "requires procurement approval above $50k".into(),
    approval_ref: Some("PROC-2026-0412".into()),
}
```

This is an honest outcome. The system doesn't pretend it can decide — it tells you exactly what it needs.

## Integration Patterns

| Pattern | Flow | Best for |
|---|---|---|
| Desktop UI | Engine pauses -> Tauri frontend -> Svelte review panel -> resume | Local-first, offline |
| Slack | Engine pauses -> Block Kit message -> button click -> resume | Team visibility, async |
| Email | Engine pauses -> email with approve/reject link -> resume | Escalation, not primary |

## Timeout Policies

- `TimeoutAction::Reject` — no review in time, proposal rejected. Safe default.
- `TimeoutAction::Approve` — no review in time, auto-approved. Only for low-risk.

## Selective Gating

Not every proposal needs review:
- Auto-approve rule-based screening (deterministic, low risk)
- Gate financial decisions above a threshold
- Always gate the final recommendation

Filter in `HitlPolicy::requires_approval()` by checking proposal content, agent, or confidence.

See also: [[Philosophy/Nine Axioms]], [[Concepts/Proposals and Promotion]]
