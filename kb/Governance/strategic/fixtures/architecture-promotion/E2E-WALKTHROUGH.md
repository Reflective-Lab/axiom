# Architecture Promotion Gate: E2E Walkthrough

## Overview

This gate prevents governance drift, hidden coupling, and audit trace degradation. It triggers when changing invariants, authority models, determinism guarantees, or core type signatures. The primary protection is against audit trace regression - the pattern where performance optimizations silently remove accountability.

## Decision Flow

```
Architecture Change -> Lens Pack Checks -> Decision
```

- **All PASS** -> Proceed without friction (audit integrity preserved)
- **Any WARN** -> Acknowledge with Christensen counter-voice consultation, then proceed
- **Any STOP** -> Requires rewrite (async logging) or two-person override with compensating controls

## What This Gate Protects Against: Audit Trace Regression

Audit trace regression occurs when changes make incorrect behavior easier than correct:

| Regression Pattern | Why It's Dangerous |
|-------------------|-------------------|
| "Skip logging for performance" | Incorrect behavior becomes invisible |
| "Optional audit for hot paths" | The easy path becomes the silent path |
| "Developer decides what to log" | Documentation replaces enforcement |

The Thaler/Ostrom tension:
- **Thaler:** "Defaults and friction shape behavior more than policy"
- **Ostrom:** "Effective governance requires explicit authority paths"

When BOTH checks fire STOP, you have the worst-case: fast, silent, unaccountable actions.

## Scenario Demonstrations

### PASS: Clean Architecture Review

- **Artifact:** architecture/policy-engine-v2.md
- **Key checks:** Authority model explicit, audit logging mandatory, type-system enforcement
- **Counter-voice:** Christensen confirms real governance latency pain justifies refactor
- **Decision:** PASS - proceed without friction
- **Fixture:** [pass.clean-review.yaml](pass.clean-review.yaml)

### WARN: Serialization Format Change (Audit Implications)

- **Artifact:** architecture/event-schema-v3.md
- **Triggering checks:** `vogels.operational_simplicity.v1` flags dual-format complexity; `howitt.feedback_reinvestment.v1` flags replay determinism impact
- **Why concerning:** Migration period affects ability to replay audit logs for investigation
- **Counter-voice consulted:** Christensen - performance gain earned through benchmarks
- **Mitigation:** 90-day migration timeline; converter tool provided; runtime format detection added
- **Decision:** WARN - acknowledged with bounds, proceeding
- **Fixtures:** [warn.serialization-change.yaml](warn.serialization-change.yaml), [acks/serialization-change.yaml](acks/serialization-change.yaml)

### STOP: Audit Trace Regression (Thaler/Ostrom Tension)

- **Artifact:** architecture/hot-path-optimization.md
- **Triggering checks:** BOTH Class A checks fire STOP:
  - `thaler.default_correctness.v1`: Removing audit makes incorrect behavior EASIER
  - `ostrom.authority_path_explicit.v1`: Cannot answer "who authorized this?"
- **Why tempting:** 10x latency improvement on hot paths, "just for high-frequency operations"
- **Why blocked:** Audit logging is governance invariant - gaps create unaccountable actions
- **The Thaler/Ostrom tension:**
  - Thaler: "When shortcuts are invisible, they become the default"
  - Ostrom: "Silent authority is no authority"
- **Resolution:** Rewrite to use async audit logging - the job is "fast AND auditable"
- **Decision:** STOP - requires rewrite, not override
- **Fixture:** [stop.audit-trace-regression.yaml](stop.audit-trace-regression.yaml)

## Key Takeaways

- Audit integrity is a governance invariant - cannot be traded for performance
- Class A STOP from Thaler means "incorrect behavior becomes easier"
- Class A STOP from Ostrom means "authority becomes unclear"
- Both firing together is worst-case: fast, silent, unaccountable
- The solution to latency is better logging (async), not silent gaps
- architecture-promotion is HIGH risk: overrides require two signatures + compensating controls
