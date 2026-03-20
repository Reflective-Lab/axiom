# Funding Narrative Gate: E2E Walkthrough

## Overview

This gate prevents narrative-reality gap and capital-driven distortion. It triggers when creating pitch decks, investor memos, term sheet responses, or board decks. The primary protection is against claim-to-proof gap - presenting capability claims without evidence mapping.

## Decision Flow

```
Funding Artifact -> Lens Pack Checks -> Decision
```

- **All PASS** -> Proceed without friction (claims substantiated)
- **Any WARN** -> Acknowledge with Ostrom counter-voice consultation, then proceed
- **Any STOP** -> Requires rewrite (add evidence mapping) - not overridable

## What This Gate Protects Against: Claim-to-Proof Gap

| Gap Pattern | Why It's Dangerous |
|-------------|-------------------|
| "Enterprise governance capabilities" without customer reference | Claims become commitments |
| "Deterministic enforcement" without technical documentation | Narrative outpaces reality |
| Traction metrics without source | Numbers become fictional |

The Marks skepticism:
- **Marks:** "The biggest errors come from psychological ones, not analytical ones"
- Funding pressure creates urgency that overrides evidence discipline

## Scenario Demonstrations

### PASS: Substantiated Claims

- **Artifact:** funding/series-b-deck-v3.md
- **Key checks:** All 12 claims have evidence links, risk section present, moat articulated
- **Counter-voice:** Ostrom confirms capital story preserves governance integrity
- **Decision:** PASS - proceed without friction
- **Fixture:** [pass.substantiated-claims.yaml](pass.substantiated-claims.yaml)

### WARN: Cycle Risk Language

- **Artifact:** funding/board-deck-q1-2026.md
- **Triggering checks:** `marks.cycle_risk_language.v1` flags "inevitable" framing; `lessin.moat_economics.v1` flags undemonstrated platform economics
- **Why concerning:** Board context allows aggressive framing but requires downside articulation
- **Counter-voice consulted:** Ostrom - growth narrative does not compromise governance
- **Mitigation:** Risk section added to appendix; language qualified to "emerging network effects"
- **Decision:** WARN - acknowledged with bounds, proceeding
- **Fixtures:** [warn.cycle-risk-language.yaml](warn.cycle-risk-language.yaml), [acks/cycle-risk-language.yaml](acks/cycle-risk-language.yaml)

### STOP: Claim-to-Proof Gap (Marks Skepticism)

- **Artifact:** funding/series-b-draft-v1.md
- **Triggering check:** Class A check fires STOP:
  - `marks.claim_proof_map_required.v1`: 8 claims, 0 proof links
- **Why tempting:** Series B timeline pressure, claims sound reasonable, "we'll add evidence later"
- **Why blocked:** Claim-to-proof gap is the core Marks failure mode - psychological pressure overrides discipline
- **The Marks skepticism:**
  - "The biggest errors come from psychological ones, not analytical ones"
  - "First-level thinkers chase opportunity. Second-level thinkers survive it"
- **Resolution:** Rewrite to add claim-to-proof mapping - claims without evidence must be removed or marked "roadmap"
- **Decision:** STOP - requires rewrite, not override
- **Fixture:** [stop.claim-to-proof-gap.yaml](stop.claim-to-proof-gap.yaml)

## Key Takeaways

- Claim-to-proof mapping is non-negotiable for funding artifacts
- "Inevitable" framing requires "what if we're wrong" section
- Counter-voice (Ostrom) protects governance from capital pressure
- funding-narrative is HIGH risk: overrides require two signatures + compensating controls
- Claim-proof gap requires REWRITE, not override - this is documentation failure, not risk acceptance
