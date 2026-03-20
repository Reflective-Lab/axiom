# Roadmap Promotion Gate: E2E Walkthrough

## Overview

This gate prevents non-compounding commitments and adoption-chain failures. It triggers when moving items from "idea" to "committed" in the roadmap. The primary protection is against platform creep - the drift from job-focused features to category-language "platforms" that lack clear customer jobs.

## Decision Flow

```
Roadmap Item -> Lens Pack Checks -> Decision
```

- **All PASS** -> Proceed without friction (compounding confirmed)
- **Any WARN** -> Acknowledge with SELL counter-voice consultation, then proceed
- **Any STOP** -> Requires rewrite (add job statement) or single-person override

## What This Gate Protects Against: Platform Creep

Platform creep occurs when roadmap language shifts from specific job statements to universal platform language:

| Platform Creep Signs | Job-Focused Alternative |
|---------------------|------------------------|
| "Strategic governance OS" | "Compliance officers proving coverage to auditors" |
| "Enterprise AI infrastructure" | "Developers need agent memory that persists across sessions" |
| "Analytics platform" | "PMs tracking which governance checks fail most often" |

The Christensen check (`job_statement_required.v1`) is Class A - deterministic and blocking.

## Scenario Demonstrations

### PASS: Job Wedge with Compounding Path

- **Artifact:** roadmap/agent-memory-persistence.md
- **Key checks:** Job statement clear, compounding explicit, adoption chain bounded
- **Counter-voice:** SELL notes positive revenue implications
- **Decision:** PASS - proceed without friction
- **Fixture:** [pass.job-wedge.yaml](pass.job-wedge.yaml)

### WARN: Partner Dependency (Adoption-Chain Risk)

- **Artifact:** roadmap/partner-connector-framework.md
- **Triggering checks:** `adner.adoption_chain.v1` flags external complement risk; `aghion.compounding_path.v1` flags partner-dependent compounding
- **Why concerning:** Adoption chain depends on partners we don't control
- **Counter-voice consulted:** SELL - partner deals justify bounded exploration
- **Mitigation:** 90-day pilot scope; internal compounding path defined as fallback
- **Decision:** WARN - acknowledged with bounds, proceeding
- **Fixtures:** [warn.partner-dependency.yaml](warn.partner-dependency.yaml), [acks/partner-dependency.yaml](acks/partner-dependency.yaml)

### STOP: Platform Creep (Christensen vs SELL)

- **Artifact:** roadmap/strategic-governance-os.md
- **Triggering check:** `christensen.job_statement_required.v1` - no job statement
- **Why tempting:** $2M pipeline, three enterprise prospects, "governance OS" sounds strategic
- **Why blocked:** "Strategic governance OS" is category language, not job language
- **The Christensen vs SELL tension:**
  - SELL: "Three prospects want this, $2M pipeline at risk"
  - Christensen: "What job does it do? Rewrite with specific persona and pain."
- **Resolution:** Rewrite roadmap item to specify the job (e.g., "Compliance officers proving coverage")
- **Decision:** STOP - requires rewrite, not override
- **Fixture:** [stop.platform-creep.yaml](stop.platform-creep.yaml)

## Key Takeaways

- Job statements are non-negotiable - missing job statement is Class A STOP
- WARN is acceptable with bounded scope + counter-voice consultation
- STOP for missing job statement requires rewrite, not override
- SELL pressure is real but cannot override Christensen's job requirement
- "Platform" language is a red flag - transform to job language before commitment
- roadmap-promotion is Medium risk: single-person override possible, but inappropriate for missing job statement
