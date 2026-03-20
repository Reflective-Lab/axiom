# Strategy Gates

Strategic validation gates for high-consequence company decisions. These gates invoke strategic lenses at decision points where drift would compound.

**Relationship to Delivery Gates:**
- Delivery gates (GATES.md) govern *how* software ships
- Strategy gates govern *what* we commit to externally and *why*

**Semantics:**
- **PASS** — Proceed without friction
- **WARN** — Requires acknowledgment before proceeding (recorded)
- **STOP** — Requires rewrite or explicit two-person override

---

## Gate Taxonomy

| Gate | Trigger | Risk Level | Override |
|------|---------|------------|----------|
| content-publish | Website, press, landing pages, docs | Medium | Single-person + rationale |
| customer-commitment | SOW, contract terms, roadmap promises | High | Two-person required |
| roadmap-promotion | Idea → Committed scope | Medium | Single-person + rationale |
| architecture-promotion | Invariant changes, authority model changes | High | Two-person required |
| funding-narrative | Pitch deck, investor memo, term sheet response | High | Two-person required |

---

## Gate Definitions

### 1. content-publish

**Purpose:** Validate external messaging aligns with current phase and intent principles.

**Trigger:** Any content going public: website copy, landing pages, press releases, documentation claims, blog posts.

**Primary Lenses:** Christensen, Thaler, Mollick
**Counter Lenses:** Altman (bounded — vision is allowed but not unbounded)

#### Pass Criteria
- Claims are job-focused, not platform-focused
- Evidence exists for any capability claim
- Messaging matches current phase (per LENSES.md lifecycle mapping)
- No system-level guarantees (secure, compliant, safe)

#### Warn Criteria (requires acknowledgment)
- Platform-adjacent language without explicit scope bounds
- Forward-looking statements without "intent" framing
- Comparisons to competitors without evidence
- Phase-forward messaging (speaking like Phase 4 when in Phase 2)

#### Stop Criteria (requires rewrite or override)
- System-level guarantees: "ensures compliance", "guarantees security", "safe by design"
- Unbounded autonomy claims: "fully autonomous", "no human oversight needed"
- Unverifiable superlatives: "best in class", "industry leading" without citation
- Claims contradicting documented limitations

#### Evidence Required
- [ ] Claim-evidence mapping (each claim linked to proof)
- [ ] Phase alignment check (current phase from PROJECT.md)
- [ ] Legal review sign-off (for regulated claims)

#### Stop Rule
All required checks pass. Warn items acknowledged. Max 2 iterations. 48h timeout.

---

### 2. customer-commitment

**Purpose:** Prevent authority leakage and implicit guarantees in customer agreements.

**Trigger:** SOW signature, contract terms, security questionnaire responses, roadmap promises in sales calls, SLA commitments.

**Primary Lenses:** Ostrom, Hemingway, Benioff (SELL)
**Counter Lenses:** Christensen (keeps it grounded in actual jobs)

#### Pass Criteria
- Commitments match documented capabilities
- No authority granted beyond current governance model
- Timeline commitments have buffer or explicit uncertainty
- Pricing matches published model (no hidden discounts)

#### Warn Criteria (requires acknowledgment)
- Custom terms not in standard agreement
- Timeline commitments tighter than engineering estimate
- Scope adjacent to but not explicitly in product
- Discount exceeds standard threshold

#### Stop Criteria (requires two-person override)
- Commitments to unbuilt features as if they exist
- Authority grants that bypass governance model
- Compliance guarantees for uncertified capabilities
- "Just for this customer" exceptions to architectural invariants
- SLA commitments exceeding operational capacity

#### Evidence Required
- [ ] Capability mapping (commitment → existing feature)
- [ ] Engineering capacity check (for timeline commits)
- [ ] Legal review (for non-standard terms)
- [ ] Founder sign-off (for strategic exceptions)

#### Stop Rule
All required checks pass. Warn items acknowledged with rationale. Max 2 iterations. 72h timeout. Founder decides on timeout.

---

### 3. roadmap-promotion

**Purpose:** Prevent scope creep and ensure work compounds toward institutional value.

**Trigger:** Moving any item from "idea" to "committed" in roadmap. Adding phases to active milestone. Accepting feature requests into scope.

**Primary Lenses:** Aghion, Mokyr, Howitt
**Counter Lenses:** Benioff (SELL) — advisory only, cannot override

#### Pass Criteria
- Work strengthens reuse across customers (not single-customer feature)
- Builds on existing institutional knowledge (compounds)
- Fits within current phase capacity
- Has clear "done" criteria

#### Warn Criteria (requires acknowledgment)
- Single-customer origin (may still be valid if generalizable)
- Requires new dependency addition
- Extends timeline of current phase
- Touches governance invariants

#### Stop Criteria (requires rewrite or override)
- Pure single-customer feature with no reuse path
- Contradicts documented architectural decisions
- Adds scope without removing scope (capacity violation)
- "Nice to have" without clear job-to-be-done

#### Evidence Required
- [ ] Reuse analysis (which other customers benefit)
- [ ] Capacity check (fits in current phase)
- [ ] Architectural alignment (doesn't violate invariants)

#### Stop Rule
All required checks pass. Warn items acknowledged. Max 1 iteration. 24h timeout.

---

### 4. architecture-promotion

**Purpose:** Prevent premature abstraction, scale paralysis, and governance drift.

**Trigger:** Changes to: invariant definitions, authority model, determinism guarantees, core type signatures, dependency additions to converge-core.

**Primary Lenses:** Vogels (SCALE), Thaler, Ostrom
**Counter Lenses:** Christensen (prevents over-engineering for hypothetical scale)

#### Pass Criteria
- Change addresses proven pain (not hypothetical future)
- Maintains or improves determinism guarantees
- Authority model unchanged or explicitly tightened
- No new dependencies in core

#### Warn Criteria (requires acknowledgment)
- Changes authority semantics (even if tightening)
- Adds abstraction layer
- Changes serialization format
- Affects replay determinism

#### Stop Criteria (requires two-person override)
- Loosens authority model ("helpful defaults")
- Adds runtime dependency to core
- Removes determinism guarantee
- Changes invariant without migration path
- "Refactor" that resets institutional memory

#### Evidence Required
- [ ] Pain evidence (concrete examples of current limitation)
- [ ] Migration path (how existing code adapts)
- [ ] Determinism impact analysis
- [ ] Authority model diff

#### Stop Rule
All required checks pass. Warn items acknowledged with architectural rationale. Max 2 iterations. 72h timeout. Founder + Security/Ethics decides on timeout.

---

### 5. funding-narrative

**Purpose:** Maintain narrative-reality alignment and prevent FOMO-driven capital decisions.

**Trigger:** Pitch deck creation/update, investor memo, term sheet response, board deck, major fundraising conversation.

**Primary Lenses:** Marks (CAPITAL), Altman
**Counter Lenses:** Ostrom (prevents governance concessions to capital)

#### Pass Criteria
- Claims match operational reality
- Metrics are actual, not projected (or clearly labeled)
- Governance model presented accurately
- Timeline projections have explicit assumptions

#### Warn Criteria (requires acknowledgment)
- Forward-looking projections without assumption documentation
- Competitive positioning without evidence
- Valuation discussion before operational proof
- Platform narrative in Phase 1-2

#### Stop Criteria (requires two-person override)
- Metrics presented as actual when projected
- Governance capabilities overstated
- Commitments that would require architectural violation
- "Inevitable" framing without institutional proof
- Accepting terms that compromise governance invariants

#### Evidence Required
- [ ] Metric source documentation (actual vs projected)
- [ ] Capability-claim mapping
- [ ] Term sheet governance impact analysis (if applicable)

#### Stop Rule
All required checks pass. Warn items acknowledged with investor context. Max 2 iterations. 48h timeout. Founder decides.

---

## Lens Pack Mapping

Which lenses run at which gates.

| Gate | Primary Pack | Secondary Pack | Counter Pack |
|------|--------------|----------------|--------------|
| content-publish | messaging (Christensen, Thaler, Mollick) | — | vision (Altman) |
| customer-commitment | governance (Ostrom, Hemingway) | commercial (Benioff) | jobs (Christensen) |
| roadmap-promotion | compounding (Aghion, Mokyr, Howitt) | — | commercial (Benioff) |
| architecture-promotion | systems (Vogels, Thaler, Ostrom) | — | jobs (Christensen) |
| funding-narrative | capital (Marks, Altman) | — | governance (Ostrom) |

### Pack Definitions

**messaging** — Voices focused on external communication clarity
- Christensen: Job-focused, not platform-focused
- Thaler: Frictionless understanding, no policy-by-reading
- Mollick: Human bottleneck awareness, adoption realism

**governance** — Voices focused on authority and institutional integrity
- Ostrom: Local rules, enforceable, explicit authority
- Hemingway: Drift prevention, gradual-then-sudden failure

**commercial** — Voices focused on market reality
- Benioff (SELL): Enterprise buying motion, packaging, GTM

**compounding** — Voices focused on institutional knowledge growth
- Aghion: Innovation compounds through institutions
- Mokyr: Knowledge becomes institutional
- Howitt: Closed-loop diffusion

**systems** — Voices focused on operational reality
- Vogels (SCALE): Scale as first-class constraint
- Thaler: Defaults shape behavior
- Ostrom: Local governance

**capital** — Voices focused on funding discipline
- Marks (CAPITAL): Second-level thinking, risk control
- Altman: Macro inevitability (bounded by proof)

**jobs** — Counter-voice for over-engineering
- Christensen: Start with real jobs, not abstractions

**vision** — Counter-voice for excessive caution
- Altman: Inevitability framing (bounded, not unbounded)

---

## Determinism Classification

Strategy gate checks have determinism classes like delivery evals:

| Class | Description | Can Block? |
|-------|-------------|------------|
| A (Deterministic) | Pattern match, banned phrase detection | Yes (STOP) |
| B (Bounded) | LLM with rubric, checklist with judgment | Yes (WARN) |
| C (Nondeterministic) | Open-ended assessment, intuition | No (Advisory only) |

**Class A checks (can produce STOP):**
- Banned phrase detection ("ensures compliance", "fully autonomous", etc.)
- Claim-evidence link verification (claim exists without linked proof)
- Phase mismatch detection (Phase 4 language in Phase 1)

**Class B checks (can produce WARN):**
- Tone/positioning alignment (LLM with phase rubric)
- Competitive claim assessment
- Timeline realism check

**Class C checks (advisory only):**
- Strategic intuition ("this feels off")
- Counter-voice application
- Long-term impact assessment

---

## Override Semantics

### Medium-Risk Gates (content-publish, roadmap-promotion)

**Override:** Single-person with rationale

```yaml
override:
  gate_id: content-publish
  approver: [founder | legal-counsel | marketing-lead]
  rationale_required: true
  compensating_controls: optional
  audit_fields:
    - timestamp
    - approver_id
    - stopped_checks
    - override_rationale
```

### High-Risk Gates (customer-commitment, architecture-promotion, funding-narrative)

**Override:** Two-person required

```yaml
override:
  gate_id: customer-commitment
  approvers: [founder, (legal-counsel | security-auditor | ethics-safety)]
  rationale_required: true
  compensating_controls: required
  audit_fields:
    - timestamp
    - approver_ids
    - stopped_checks
    - override_rationale
    - compensating_controls_applied
    - post_override_review_date
```

---

## Integration with Delivery Gates

Strategy gates and delivery gates are orthogonal:

| Artifact Type | Delivery Gate | Strategy Gate |
|---------------|---------------|---------------|
| Code PR | pr-merge | — |
| Release | release-approval | — |
| Website copy | — | content-publish |
| SOW/Contract | — | customer-commitment |
| Roadmap change | — | roadmap-promotion |
| Core invariant change | release-approval | architecture-promotion |
| Pitch deck | — | funding-narrative |

**Intersection:** Core invariant changes require BOTH release-approval (delivery) AND architecture-promotion (strategy).

---

## Weekly Alignment Digest

Strategy gates produce a weekly summary for organizational alignment.

**Digest Sections:**

1. **Where We Are** — Current phase, active lenses, recent gate decisions
2. **What to Do** — Primary voice guidance for this phase
3. **What Not to Do** — Suppressed voices, common mistakes to avoid
4. **What to Say** — Approved messaging patterns
5. **What Not to Say** — Banned phrases, premature claims
6. **Drift Signals** — Detected patterns from monitoring (if enabled)

**Distribution:** All employees, weekly (Monday).

**Source:** Aggregated from gate decisions, phase status, LENSES.md lifecycle mapping.

---

*Strategy gates v3.0 — Draft*
*Last updated: 2026-01-26*
