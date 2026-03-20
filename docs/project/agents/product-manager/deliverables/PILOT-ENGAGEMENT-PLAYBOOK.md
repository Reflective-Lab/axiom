# Pilot Engagement Playbook

> How we run a Converge pilot from first contact to paid contract.

**Owner:** Nadia Reeves, Product Manager
**Status:** v1.1 (Approved — Ren, Blake, Alice. Technical review findings incorporated.)
**Date:** 2026-03-12
**Companion docs:** [Pilot Program PRD](PILOT-PROGRAM-PRD.md), [Pilot Metrics Framework](../../../plans/PILOT_METRICS_FRAMEWORK.md), [Pilot-to-Contract Playbook](../../solutions-engineer/deliverables/pilot-to-contract-playbook.md), [Design Partner Onboarding Playbook](../../solutions-engineer/deliverables/design-partner-onboarding-playbook.md)

> **Scope note:** This playbook is the "what" (pilot lifecycle and PM process). Leo's onboarding playbook is the "how" (technical setup details). Blake's pilot-to-contract playbook is the "commercial narrative" (conversion flow and email templates).

---

## 1. Pilot Lifecycle

```
Week -2: Qualify & Scope
Week -1: Setup & Baseline
Week 1-4: Live Pilot
Week 5:   Results & Close
```

Total engagement: ~7 weeks from first qualification call to buy/no-buy decision.

---

## 2. Phase 1: Qualify & Scope (Week -2)

### 2.1 Qualification Call (Blake + Nadia, 45 min)

**Goal:** Confirm design partner fit against ICP criteria.

Checklist — must get a "yes" on all six:

| # | Criterion | How to Verify | Red Flag |
|---|-----------|--------------|----------|
| 1 | Active pain | Ask: "Describe the last time this process broke." They should have a specific, recent example. | Vague answers, aspirational automation |
| 2 | Measurable baseline | Ask: "What does your current cycle time look like?" They should have numbers or be willing to measure. | "We don't really track that" |
| 3 | Integration surface | Ask: "What systems touch this workflow?" Need at least one we can integrate (CRM, webhook-capable tool). | Fully manual, paper-based process |
| 4 | Executive sponsor | Ask: "Who would approve a $500/mo tool purchase?" They should name a person, not a committee. | "We'd have to run it through procurement" |
| 5 | Time commitment | Ask: "Can someone spend 2 hours/week for 4 weeks reviewing Converge output?" | No dedicated person available |
| 6 | Publication consent | Ask: "Would you be open to an anonymized case study?" | Hard no on any external reference |

**Output:** Qualification scorecard (pass/fail per criterion). File in `pilots/<company-slug>/qualification.md`.

### 2.2 Scoping Session (Nadia + Leo, 60 min)

**Goal:** Define the specific workflow Converge will automate.

Must capture:
1. **Trigger event** — What starts the workflow? (e.g., new lead in CRM, approval request submitted)
2. **Steps** — List every step, who does it, what system it touches
3. **Decision points** — Where does a human currently make a judgment call?
4. **End state** — What does "done" look like?
5. **Failure modes** — Where does it break? How often? What's the cost?
6. **HITL gates** — Which decisions must stay with the human during the pilot?

**Output:** Workflow map. File in `pilots/<company-slug>/workflow-map.md`.

### 2.3 Internal Go/No-Go (Nadia + Ren + Ava, 30 min)

Before committing to a pilot, engineering and security must confirm:

- [ ] The workflow can be modeled with current converge-core capabilities
- [ ] Integration requirements are feasible (MVP-3)
- [ ] Data isolation requirements are clear (MVP-7)
- [ ] No compliance blockers for this customer's industry
- [ ] Estimated setup effort: ______ hours (Leo's time)

**Decision:** Go / No-Go / Go with conditions

---

## 3. Phase 2: Setup & Baseline (Week -1)

### 3.1 Technical Setup (Leo, ~10-20 hours)

| Task | Owner | Duration | Depends On |
|------|-------|----------|------------|
| Create customer workspace | Leo | 2 hrs | Go decision |
| Configure agents for the workflow | Leo + Eli | 4-8 hrs | Workflow map |
| Connect integration (webhook/API) | Leo | 4-8 hrs | Customer provides credentials |
| Set up HITL gates per workflow map | Leo | 2 hrs | HITL implementation (REF-42). **If REF-42 is not complete at pilot start, substitute manual review via Slack notification — Leo sends proposal summary to customer in Slack, customer replies approve/reject, Leo manually updates engine state.** |
| Verify end-to-end convergence run | Leo + Sam | 2 hrs | All above |
| Configure telemetry capture | Sam | 2-8 hrs | Pilot Metrics Framework. Note: full telemetry pipeline requires Wave 3. For first pilots, Sam builds a lightweight InMemory shim (Morgan decision). 2 hrs if shim exists; up to 8 hrs for first pilot to build it. |

### 3.2 Baseline Measurement (Sam, 3-5 days)

Before the pilot starts, we must capture "before" metrics:

| Metric | How to Capture | Who Provides Data |
|--------|---------------|-------------------|
| Current cycle time | Customer shares last 10 instances with timestamps | Customer ops lead |
| Current error/rework rate | Customer shares last 30 days of exceptions | Customer ops lead |
| Manual steps count | From workflow map (Phase 1) | Nadia |
| Current throughput | Customer shares volume data | Customer ops lead |

**Output:** Baseline report. File in `pilots/<company-slug>/baseline.md`.

### 3.3 Customer Kickoff (Blake + Nadia + Leo, 30 min)

Set expectations with the customer:

1. What Converge will do during the pilot (specific to their workflow)
2. What the customer's role is (review HITL gates, provide feedback)
3. How to observe convergence (UI access or Slack notifications)
4. Weekly check-in schedule (same day/time each week)
5. What success looks like (reference the baseline metrics)
6. What happens at the end (results presentation, pricing discussion)

**Output:** Kickoff confirmation email with above points. Template in `templates/pilot-kickoff-email.md`.

---

## 4. Phase 3: Live Pilot (Weeks 1-4)

### 4.1 Weekly Rhythm

| Day | Activity | Who | Duration |
|-----|----------|-----|----------|
| Monday | Review prior week's convergence runs, flag issues | Leo + Sam | 30 min |
| Wednesday | Customer check-in call | Nadia + Leo + Customer | 30 min |
| Friday | Internal sync: pilot health, metrics, blockers | Nadia + Leo + Ren | 15 min |

### 4.2 Customer Check-in Agenda (Wednesday)

1. **How did it go this week?** (open-ended, let them talk first)
2. **HITL decisions:** Review any approvals/rejections. Patterns? Surprises?
3. **Accuracy:** Did Converge make any wrong proposals? What happened?
4. **Friction:** Anything confusing, slow, or annoying?
5. **Metrics peek:** Share this week's numbers vs. baseline (if available)
6. **Next week:** Any workflow changes, holidays, or volume spikes to plan for?

**Rule:** Take notes. File weekly in `pilots/<company-slug>/week-N.md`.

### 4.3 Escalation Protocol

| Signal | Severity | Response | Timeline |
|--------|----------|----------|----------|
| Convergence failure (engine crash) | P0 | Leo escalates to Eli. Customer notified within 1 hour. | Fix within 4 hours (best effort — no formal SLA; monitoring is Slack-based for first pilots) |
| Wrong proposal accepted (data integrity) | P0 | Pause HITL auto-approve. Nadia calls customer. Post-mortem within 24 hours. | Same day (best effort) |
| Slow convergence (>60 sec target, TBD pending first pilot baseline) | P1 | Leo investigates. May need Ren. | Fix within 1 business day (best effort) |
| Customer confused by UI/output | P2 | Leo adjusts configuration or adds context. Note for product backlog. | Next check-in |
| Customer disengaged (missed check-in) | P2 | Nadia reaches out directly to executive sponsor. | Within 24 hours |

### 4.4 HITL Learning Loop

Track every HITL decision during the pilot:

| Data Point | Why It Matters |
|-----------|---------------|
| Total gates triggered | Volume of decisions requiring human judgment |
| Approve rate | If >90%, consider auto-approving that gate type |
| Reject reasons | Feeds back into agent tuning — what are agents getting wrong? |
| Response time | How quickly does the customer respond? Affects convergence throughput |
| Timeout rate | If gates timeout often, notification channel may be wrong |

This data feeds directly into the case study and the PRD for Phase 2 HITL (web UI).

---

## 5. Phase 4: Results & Close (Week 5)

### 5.1 Results Package (Nadia + Sam + Blake, 3 days)

Produce a one-page results summary for the executive sponsor:

**Template structure:**
1. **What we set out to do** (1 sentence — the workflow we automated)
2. **Before/after metrics** (table: baseline vs. pilot, with % improvement)
3. **Specific wins** (2-3 concrete examples of decisions Converge handled well)
4. **Honest limitations** (what Converge couldn't do or got wrong)
5. **What's next** (features coming that would improve their specific workflow)
6. **Pricing** (Professional tier pricing, locked-rate offer for early partners)

**Output:** File in `pilots/<company-slug>/results.md`.

### 5.2 Close Call (Blake + Nadia + Executive Sponsor, 30 min)

1. Walk through results (Blake leads with narrative, Nadia handles product questions)
2. Ask: "Based on what you've seen, would you continue?" (direct, no pressure)
3. If yes: present pricing, discuss contract terms (per REF-38 commercial terms)
4. If no: ask what would need to change. Document and feed back to product.
5. If maybe: offer 2-week extension with specific success criteria

### 5.3 Case Study (Blake + Sam, within 1 week of pilot end)

Per Pilot Metrics Framework Section 6:
- Anonymized version ships immediately (no customer approval needed beyond initial consent)
- Named version requires customer review and written approval
- Both versions must include honest limitations, not just wins

---

## 6. Roles Summary

| Role | Person | Responsibility |
|------|--------|---------------|
| **Pilot PM** | Nadia Reeves | Qualification, scoping, customer relationship, results |
| **Solutions Engineer** | Leo Marin | Technical setup, integration, day-to-day operations |
| **QA / Metrics** | Sam Okafor | Baseline measurement, telemetry, metrics reporting |
| **Sales Lead** | Blake Harmon | Qualification support, narrative, close call, case study |
| **Engineering Escalation** | Ren Akiyama | Blocker resolution, resource allocation |
| **Core Engine** | Eli Marsh | Engine issues, HITL gate implementation |
| **Security** | Ava Petrov | Data isolation review, compliance sign-off |

---

## 7. File Structure Per Pilot

```
pilots/
  <company-slug>/
    qualification.md     — ICP scorecard
    workflow-map.md      — Workflow steps, systems, decision points
    baseline.md          — Before-metrics
    week-1.md            — Weekly check-in notes
    week-2.md
    week-3.md
    week-4.md
    results.md           — One-page results summary
    case-study-anon.md   — Anonymized case study
    case-study-named.md  — Named case study (if approved)
```

---

## 8. Open Items

| Item | Owner | Status |
|------|-------|--------|
| Create `pilots/` directory structure | Nadia | After first design partner identified (REF-41) |
| Create pilot kickoff email template | Blake | Claimed (Blake will draft) |
| Create qualification scorecard template | Nadia | Done — see `QUALIFICATION-SCORECARD-TEMPLATE.md` |
| Define Leo's pilot allocation (hours/pilot) | Ren | Approved — 10-20 hrs/pilot, drops for Pilot 2+ |
| HITL gate implementation | Eli (REF-42) | Todo — consensus: Slack/email + approve API (Option 2) |
| Cross-reference Leo's onboarding playbook | Nadia | Done — added to companion docs |

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-03-12 | 1.0 | Initial draft — Nadia Reeves |
| 2026-03-12 | 1.1 | Approved by Ren, Blake, Alice. Incorporated Alice's technical review (4 findings): added HITL fallback for pre-REF-42 pilots, calibrated convergence threshold, adjusted telemetry setup time for shim build, labeled SLA timelines as best-effort. Added Leo's onboarding playbook cross-reference per Ren. Updated open items. — Nadia Reeves |
