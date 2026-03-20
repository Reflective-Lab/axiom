# Editorial Style Rules — Converge Content Programme

**Owner:** Caroline Ashford (Editor-in-Chief)
**Date:** March 12, 2026
**Status:** Active — applies to all content in the editorial pipeline

---

## 1. Epistemic Labels (Mandatory)

Every non-obvious assertion in published content must carry one of:

| Label | Meaning | Example |
|-------|---------|---------|
| **[Observed]** | We measured it, implemented it, or can demonstrate it in code today | "The engine rejects proposals that fail verification" |
| **[Inferred]** | We reasoned it from evidence, but have not formally proven it | "This design is intended to produce the same outcome regardless of execution order" |
| **[Speculative]** | We believe it but cannot prove it yet | "Full mathematical verification of convergence across all configurations" |

Labels may be inline or in a footnote. They must be present.

## 2. Tense Discipline — The Wave Rule

**Rule:** If a feature is not in Wave 1 (Active status), it must use future tense or carry a [Speculative] label. No exceptions.

| Wave Status | Tense | Example |
|-------------|-------|---------|
| **Active** (Wave 1) | Present tense allowed | "The engine verifies proposals against rules." |
| **Planned / In Progress** (Wave 2) | Future tense required | "The engine will enforce Cedar policies when converge-policy ships." |
| **Backlog** (Wave 3-5) | Future tense + explicit caveat | "We plan to add a JTBD compiler in a future release." |

### Specific features and their current status (as of March 12, 2026):

**Present tense allowed (Wave 1 Active):**
- Proposal → verification → fact boundary
- Append-only context
- Type-level enforcement (ProposedFact ≠ Fact)
- Fixed processing order for proposals
- Basic provenance (who proposed, verified, when)

**Future tense required (Wave 2 Planned):**
- Cedar policy enforcement
- LLM provider integration (Anthropic, OpenAI)
- Analytics and optimization providers
- Per-rule provenance attribution

**Future tense + caveat required (Wave 3+ Backlog):**
- JTBD compiler
- WASM runtime with dynamic module loading
- Domain marketplace modules
- Experience store
- gRPC client / reference application

**Update this list as engineering status changes.** When a feature moves from Planned to Active, it may be described in present tense.

## 3. Prohibited Patterns

| Pattern | Why | Alternative |
|---------|-----|-------------|
| "Provably correct" without qualifier | Overstates what is proven | "Provably correct verification boundary" or qualify scope |
| "Leading" / "Best" / "Revolutionary" | Superlatives require evidence | State the specific claim with evidence |
| "AI-powered" as selling point | Everyone says this | Describe what the AI actually does |
| "End-to-end" without scope | Implies completeness | Specify which ends |
| Describing demos as production | Misleading | "In our demo environment..." or "In simulation..." |

## 4. Source Requirements

- Every factual claim needs a source: code reference, measurement, or explicit label.
- "We observed X" requires a pointer to where X was observed.
- Pilot data must cite the specific pilot (anonymised if needed).
- Competitive claims must be verifiable from public sources.

## 5. Voice

- Kenneth Pernyer is the brand voice. Edit to sharpen, never to replace.
- British spelling (organisation, colour, programme) in editorial content.
- Short paragraphs. One idea per section. Respect the reader's time.
- Questions over assertions when exploring uncertain territory.
- Warm but not casual. No clickbait. No listicles.

---

*This document governs all content passing through the editorial pipeline: Signals (blog), Converging Voices (podcast), whitepapers, and any website copy under editorial ownership. Documents outside the editorial pipeline (Strategy, SOC 2 policies, internal plans) are encouraged to adopt these standards but are not governed by this document.*
