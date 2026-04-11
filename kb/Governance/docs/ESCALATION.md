# Escalation Guide

> How Extended team personas escalate concerns to Core team for blocking review.

**Cross-references:**
- For team roster and authority tiers, see [TEAM.md](../TEAM.md)
- For gate policies and participation, see [GATES.md](../GATES.md)

---

## Introduction

**What is escalation?** Escalation is the formal process for Extended team personas to surface blocking concerns to Core team for review and disposition. Extended personas (Advisory or Escalating tier) cannot block promotions directly - instead, they submit evidence-based escalation packets that trigger Core team review.

**Why structured escalation?** Unstructured "please review" comments create noise and don't enable learning. Structured escalation packets require evidence, investigation, and clear stop rules - proving that homework was done before escalating to Core. This raises escalation quality while creating a feedback loop where Core coaching helps Extended calibrate over time.

**When to escalate vs advise?**

- **Advisory comments** are for informational input that doesn't need blocking review. Example: "Noticed confusing UX in onboarding flow" from Curious Searcher. Development team considers feedback but isn't blocked.

- **Escalation packets** are for concerns that may need to block promotion until resolved. Example: "Binary size bloated from 12MB to 45MB, environmental impact quantified" from Sustainability Lead with evidence and stop rule. Core reviews and decides whether to block.

**General principle:** If you're uncertain whether issue merits blocking, check precedents first (search prior dispositions). If still uncertain, file escalation - Core will provide coaching on whether concern warrants escalation in future.

---

## When to Escalate

Use this decision tree to determine when escalation is appropriate:

### 1. Is this informational or a blocking concern?

- **Informational:** Use advisory comment (no escalation needed)
- **Blocking concern:** Continue to question 2

### 2. Does this require Core team decision?

- **Yes, Core expertise needed:** Continue to question 3
- **No, can be resolved by dev team:** File issue, no escalation needed

### 3. Is evidence available to support your concern?

- **Yes, investigation complete:** Continue to question 4
- **No, preliminary concern:** Investigate first, then return to question 3

### 4. Have you checked precedents?

- **Yes, no prior disposition on this topic:** Proceed with escalation
- **Yes, found similar prior escalation:** Review prior Core feedback, apply learnings
- **No precedents checked:** Search `.planning/` for prior dispositions first

**When escalation is appropriate:**

- Concern may warrant blocking promotion (security, legal, safety, sustainability, operational)
- Evidence exists proving concern validity (logs, metrics, test results, analysis)
- Investigation summary shows homework was done
- Clear stop rule defines what would satisfy concern
- No recent precedent provides answer (or precedent suggests escalation is warranted)

**When escalation is NOT appropriate:**

- Informational feedback that doesn't need blocking review
- Concern can be resolved by development team without Core review
- No evidence available (investigate first before escalating)
- Duplicate of recent escalation that Core already addressed

---

## Escalation Packet Structure

Escalation packets follow a structured schema ensuring Core has context to make informed decisions.

**Full schema:** See [escalation-packet.md](../schemas/escalation-packet.md) for complete field definitions.

### Quick Reference Table

| Field | Purpose | Example |
|-------|---------|---------|
| `escalation_id` | Unique tracking identifier | ESC-2026-042 |
| `severity` | P0/P1/P2 routing priority | P1 (same-day review) |
| `escalated_by` | Your persona ID from TEAM.md | sustainability-lead |
| `gate_id` | Which gate this applies to | release-candidate |
| `eval_id` | Which eval surfaced concern | sustainability-eval |
| `risk_prevented` | What bad outcome this prevents | Users waste bandwidth downloading bloated binary, environmental cost |
| `evidence` | Links, logs, test results proving concern | Binary size: 45MB (up from 12MB), CO2 footprint: +82 metric tons |
| `stop_rule` | What would satisfy this escalation | Binary size <20MB or justification for size increase |
| `confidence` | How certain are you (high/medium/low) | high (tested alternative, dependency tree analyzed) |
| `investigation_summary` | What you already checked (homework) | Analyzed dependency tree, identified unused features, tested minimal build |

### Three Required Blocks

**1. Metadata Block:** Identifies escalation (ID, severity, persona, gate, eval)

**2. Concern Block:** Core content with risk prevented, evidence, stop rule, confidence

**3. Context Block:** Investigation summary, related escalations, recommended disposition

**Key principle:** Evidence and investigation summary prove homework was done. Stop rule clarifies resolution criteria so Core knows when escalation is satisfied.

---

## Severity Levels

Severity determines review SLA (Service Level Agreement). Choose severity honestly - inflating severity for faster response degrades system trust.

| Severity | Definition | Review Target | Use When |
|----------|------------|---------------|----------|
| **P0** | Immediate blocking concern | 4 hours | Security vulnerability, legal violation, production outage imminent |
| **P1** | Same-day review needed | 24 hours | High-risk concern blocking current gate, significant user impact |
| **P2** | Weekly review acceptable | 1 week | Important strategic concern but not immediately blocking |

### Severity Guidelines

**P0 is rare.** Reserve for genuine emergencies:
- Security breach or critical vulnerability discovered
- Legal compliance violation exposing organization to liability
- Production outage imminent or in progress
- Safety issue with immediate user harm potential

**P1 is typical high-priority.** Most blocking concerns at release gates fall here:
- Binary size bloat with environmental/bandwidth impact
- Performance regression affecting user experience
- Operational readiness concerns at release candidate
- Contract deliverability concerns before customer commitment

**P2 is for strategic concerns.** Long-term issues not blocking current gate:
- Documentation gaps for future maintainability
- Sustainability improvements for future releases
- Architectural debt that should be addressed eventually
- Minor UX concerns that can be deferred

**SLA Notes:**
- SLAs are targets, not guarantees. Core prioritizes based on severity and capacity.
- If escalation is more urgent than initially assessed, Core may escalate severity.
- If escalation is less urgent than filed, Core may de-escalate severity in disposition.

---

## Disposition Types

When Core reviews your escalation, they'll respond with one of four disposition types. Each comes with coaching feedback in the learning block.

**Full schema:** See [disposition-feedback.md](../schemas/disposition-feedback.md) for complete disposition structure.

### Disposition Outcomes

| Disposition | Meaning | What Happens Next |
|-------------|---------|-------------------|
| **approved** | Concern is valid, action will be taken | Core provides next steps, timeline, and owner. Your escalation led to concrete action. |
| **denied** | Concern not valid or already addressed | Core explains why concern doesn't merit blocking, provides compensating controls showing alternative mitigations exist. |
| **need_more_info** | Evidence insufficient, strengthen and resubmit | Core provides specific guidance on what evidence or investigation would strengthen escalation. Resubmit with additions. |
| **deferred** | Valid concern but not blocking current gate | Core acknowledges concern but explains why it doesn't block this specific gate. Assigns follow-up owner for future review. |

### What to Expect from Each Disposition

**Approved Dispositions:**
- **Rationale:** Why Core agrees concern is valid
- **Next steps:** Numbered action list with timeline and responsible party
- **Learning:** What made your escalation strong, how to make future escalations even better
- **Follow-up owner:** Core persona who owns executing resolution

Example: Binary size bloat escalation approved - optimization work scheduled, binary size threshold added to CI pipeline.

**Denied Dispositions:**
- **Rationale:** Why concern doesn't merit blocking action
- **Compensating controls:** Existing mitigations showing concern is already addressed
- **Precedent references:** Links to similar prior escalations that were denied (shows consistency)
- **Learning:** What you did well (awareness, investigation), what would change decision in future

Example: GDPR concern denied because privacy policy already addresses data retention, Core references precedent DISP-2025-203.

**Need More Info Dispositions:**
- **Rationale:** Why current evidence is insufficient
- **What to improve:** Specific guidance on strengthening escalation (more evidence, clearer stop rule, deeper investigation)
- **Learning:** What you got right, what's missing
- **Invitation to resubmit:** Core signals willingness to reconsider with additions

Example: Insurance liability escalation needs more info - quantify financial exposure, provide test results showing SLA breach likelihood.

**Deferred Dispositions:**
- **Rationale:** Why valid concern doesn't block current gate
- **Follow-up plan:** When and how concern will be revisited
- **Learning:** What made concern valid but not blocking for this gate
- **Follow-up owner:** Who will track concern for future review

Example: Documentation gap deferred from release-candidate to post-release - valid concern but doesn't block release, assigned to Developer Advocate for next sprint.

---

## The Learning Loop

Every disposition includes coaching feedback in the learning block. This is how Extended team calibrates over time - learning which concerns merit escalation and how to structure evidence for maximum effectiveness.

### Learning Block Fields

**what_was_strong:**
- Always provided, even for denied escalations
- Positive reinforcement for effective elements
- Example: "Strong evidence with concrete metrics (binary size +266%), root cause analysis traced to dependency"

**what_to_improve:**
- Constructive feedback for growth
- Provided when escalation could be strengthened
- Example: "Stop rule could be more specific - suggest threshold with justification rather than open-ended optimization"

**coaching_notes:**
- Teaching moment from Core to Extended
- References documentation, suggests research, explains domain context
- Example: "For future sustainability escalations, quantify environmental impact using carbon calculator methodology documented in [link]"

### How to Use Coaching Feedback

1. **Read learning block first.** Before reacting to disposition outcome, read what Core says about your escalation quality.

2. **Apply to future escalations.** Use coaching notes to improve next escalation structure and evidence quality.

3. **Ask questions if unclear.** If coaching notes reference unfamiliar concepts, ask Core for clarification or documentation links.

4. **Track your growth.** Over time, you should see patterns - stronger evidence, clearer stop rules, better investigation summaries.

**Goal:** After 5-10 escalations with coaching feedback, you should be able to predict whether concern merits escalation and how to structure packet for Core efficiency.

---

## Examples

Study these examples to understand what strong escalations look like and how Core responds to different scenarios.

### Approved Escalation

**File:** [approved-example.md](../schemas/escalation-examples/approved-example.md)

**Scenario:** Sustainability Lead escalates binary size bloat at release-candidate gate

**What made it strong:**
- Concrete metrics (binary size: 12MB → 45MB, +266%)
- Root cause analysis (traced to llvm-symbolizer and bloated dependencies)
- Tested solution (minimal features build: 18MB)
- Quantified environmental impact (82 metric tons CO2 for 1M downloads)
- Clear stop rule (binary <20MB or justification)
- High confidence with investigation summary

**Core response:** Approved with next steps (optimization work, binary size threshold in CI)

**Key learning:** Strong evidence with measurable impact and tested alternatives leads to approval.

---

### Denied Escalation

**File:** [denied-example.md](../schemas/escalation-examples/denied-example.md)

**Scenario:** Regulator Lens escalates GDPR data retention concern at release-approval gate

**Why it was denied:**
- Concern already addressed in privacy policy
- Data retention complies with GDPR Article 5 principles
- Core references precedent (DISP-2025-203 on similar GDPR concern)

**Compensating controls provided:**
- Privacy policy documents 30-day retention for analytics
- DPO review already passed
- Data export/deletion flows implemented

**Core response:** Denied with positive feedback (good GDPR awareness) and coaching (check privacy policy first, review DPO eval results)

**Key learning:** Check existing policies and prior dispositions before escalating compliance concerns.

---

### Need More Info Escalation

**File:** [need-more-info-example.md](../schemas/escalation-examples/need-more-info-example.md)

**Scenario:** Insurance Underwriter escalates SLA liability at customer-commitment gate

**Why more info needed:**
- Financial exposure quantification missing
- Test data insufficient (10 sample requests, should be 10k)
- Stop rule unclear (needs specific SLA percentile definition)

**Core guidance:**
- Quantify financial impact (penalty clauses in contract)
- Expand load testing (10k requests at p99 latency)
- Refine stop rule (specific SLA percentile under penalty threshold)

**Extended resubmits:** V2 escalation with additions, Core approves

**Key learning:** Quantify financial/operational impact, provide comprehensive test data, make stop rules measurable.

---

## Anti-Patterns to Avoid

Learn from common escalation mistakes:

### 1. "Please review" without evidence

**Bad:** "This code looks risky, please review"

**Good:** "Binary size increased 266% (45MB from 12MB), traced to llvm-symbolizer dependency [commit link], environmental impact: 82 metric tons CO2 [calculation]"

**Why:** Evidence proves concern validity and shows investigation was done.

---

### 2. Escalating everything "to be safe"

**Bad:** Filing 20 escalations per release, most denied or deferred

**Good:** File 2-3 high-quality escalations per release with strong evidence

**Why:** Escalation fatigue degrades system. Core spends time on low-value reviews instead of genuine concerns.

---

### 3. Not checking precedents

**Bad:** Escalating GDPR concern that Core denied 3 weeks ago with same reasoning

**Good:** Search `.planning/` for prior dispositions, review Core reasoning, apply learnings

**Why:** Duplicate escalations waste time and suggest Extended didn't learn from prior feedback.

---

### 4. Missing stop rule

**Bad:** "Binary is too large" (open-ended, unclear when satisfied)

**Good:** "Binary size <20MB or architectural justification documenting necessity" (clear resolution criteria)

**Why:** Stop rule clarifies what Core needs to approve. Open-ended escalations never converge.

---

### 5. Low confidence without strengthening first

**Bad:** Filing escalation with confidence: low and minimal investigation

**Good:** If confidence is low, investigate further before escalating OR file escalation noting preliminary concern and requesting Core expertise to validate

**Why:** Low confidence escalations should either be strengthened first or explicitly request Core validation expertise.

---

### 6. Ignoring coaching feedback

**Bad:** Receiving "stop rule too vague" feedback in disposition, then filing next escalation with vague stop rule again

**Good:** Apply coaching notes to future escalations, demonstrating learning

**Why:** Coaching feedback is how Extended calibrates. Ignoring it means repeating same mistakes.

---

## Getting Started

**First escalation?** Follow this checklist:

1. **Identify concern:** What risk are you preventing?
2. **Gather evidence:** Links, logs, metrics, test results
3. **Investigate:** Check dependency trees, review documentation, test alternatives
4. **Check precedents:** Search prior dispositions for similar concerns
5. **Define stop rule:** What specific outcome would satisfy this escalation?
6. **Assess confidence:** High (thoroughly investigated), Medium (some uncertainty), Low (preliminary)
7. **Write escalation packet:** Use schema from escalation-packet.md
8. **Submit for review:** Core will provide disposition with coaching feedback
9. **Read learning block:** Apply coaching to future escalations
10. **Track your growth:** Over time, escalations should get stronger and dispositions more favorable

**Questions?** File escalation even if uncertain - Core will provide coaching on whether concern warrants future escalation.

---

**See also:**
- [TEAM.md](../TEAM.md) - Who can escalate (Extended tier personas)
- [GATES.md](../GATES.md) - Where escalations apply (all gates allow escalation)
- [escalation-packet.md](../schemas/escalation-packet.md) - Full schema for escalation packets
- [disposition-feedback.md](../schemas/disposition-feedback.md) - Full schema for Core responses

---

*Last updated: 2026-01-26*
