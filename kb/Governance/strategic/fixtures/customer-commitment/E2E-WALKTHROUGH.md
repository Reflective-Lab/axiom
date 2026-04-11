# Customer Commitment Gate: E2E Walkthrough

## Overview

This gate prevents authority leakage and exception drift in customer agreements (SOWs, contracts, sales commitments). It ensures governance invariants are not eroded by business pressure, even when deals are tempting.

## Decision Flow

```
Artifact (SOW/Contract) -> Lens Pack Checks -> Decision
```

- **All PASS** -> Proceed without friction
- **Any WARN** -> Acknowledge with counter-voice consultation, then proceed
- **Any STOP** -> Requires rewrite or two-person override (HIGH risk gate)

## Scenario Demonstrations

### PASS: Clean SOW

- **Artifact:** deals/acme-corp-sow-2026-01.md
- **Key checks:** All primaries (Ostrom, Hemingway, SELL, Thaler) pass, no authority concessions
- **Counter-voice:** Vogels notes acceptable operational surface
- **Decision:** PASS - proceed without friction
- **Fixture:** [pass.clean-sow.yaml](pass.clean-sow.yaml)

### WARN: Timeline Pressure

- **Artifact:** deals/bigco-sow-2026-01.md
- **Triggering check:** `hemingway.exception_phrase_detect.v1` detected "quick patch" language
- **Why concerning:** Exception language often precedes scope creep and drift
- **Counter-voice consulted:** Vogels - operational risk acceptable if scoped to read-only
- **Mitigation:** Integration scope bounded to API read endpoints; full review scheduled Q2
- **Decision:** WARN - acknowledged with bounds, proceeding
- **Fixtures:** [warn.timeline-pressure.yaml](warn.timeline-pressure.yaml), [acks/timeline-pressure.yaml](acks/timeline-pressure.yaml)

### STOP: Authority Bypass (Tempting Exception)

- **Artifact:** deals/megacorp-msa-addendum.md
- **Triggering check:** `ostrom.authority_concession_scan.v1` detected audit log bypass request
- **Why tempting:** $2M ARR potential, "performance" sounds reasonable, sales pressure is real
- **Why blocked:** Audit logging is a governance invariant - cannot be disabled regardless of pressure
- **The Hemingway/Ostrom tension:**
  - Hemingway: "Systems fail gradually, then suddenly, through silent drift"
  - Ostrom: "Local rules must be enforceable and verifiable"
  - Disabling audit is exactly the pattern that erodes institutional integrity
- **Decision:** STOP - requires rewrite (offer async logging) or two-person override
- **Fixture:** [stop.authority-bypass.yaml](stop.authority-bypass.yaml)

## Key Takeaways

- Authority concessions are governance invariants - never negotiable for individual deals
- "Just for this customer" is a red flag phrase
- Counter-voice (Vogels) consultation is required for all WARN acknowledgments
- STOP on authority bypass requires rewrite, not just override
- customer-commitment is HIGH risk: overrides require two signatures
- The alternative to bypass is always available - async logging addresses real performance concerns
