# Pilot Readiness Tracker

> MVP checklist, dependency status, and blockers for first design partner engagement.

**Owner:** Nadia Reeves, Product Manager
**Status:** Active — updated 2026-03-13
**Target:** Pilot-ready by end of cw-5 (April 10). First engagement cw-6 (April 13).
**Source:** [Pilot Program PRD v1.2](./PILOT-PROGRAM-PRD.md), [Engineering Plan v2](../../../plans/ENGINEERING_PLAN.md)

---

## 1. MVP Requirements Status

| ID | Requirement | Owner | Status | Issue | Target | Blockers / Notes |
|----|------------|-------|--------|-------|--------|-----------------|
| MVP-1 | **Convergence engine runs end-to-end** (3+ agents, <60s) | Eli Marsh | DONE | — | cw-2 | converge-core proof examples complete. converge-traits v0.3.0 frozen. LlmAgent idempotency bug — verify fix status with Eli. |
| MVP-2 | **Customer can observe convergence** (web UI) | Jules Carrera | IN PROGRESS | REF-54 | cw-3 | Observation UI v1 in progress. Existing demos are scripted simulations — real engine connection via StreamingCallback needed. Depends on MVP-1 API contract with Eli. |
| MVP-3 | **At least one integration works** (webhook/API) | Eli Marsh + Leo Marin | NOT STARTED | — | cw-4 | Generic webhook framework per Eng Plan D3. No issue created yet. Leo doing research. Depends on MVP-1. |
| MVP-4 | **Human-in-the-loop gate** (pause/approve/reject) | Eli Marsh | IN PROGRESS | REF-42 | cw-3 | Approach decided: Slack/email notification + approve/reject API (Option 2). REF-61 filed: HITL rejection causes infinite pause loop — needs fix. |
| MVP-5 | **Audit trail exists** (reviewable convergence log) | Eli Marsh | PARTIAL | — | cw-3 | InMemoryExperienceStore exists. Needs verification that it captures full provenance per PRD spec. No dedicated issue — verify with Eli during cw-3. |
| MVP-6 | **Telemetry captures pilot metrics** | Sam Okafor | DESIGN DONE | REF-55 (done) | cw-4 | Telemetry exporter v1 design + test plan complete. Implementation next. InMemory shim for first 2 pilots (Morgan decision). |
| MVP-7 | **No customer data leaves their environment** | Ava Petrov | NOT STARTED | REF-15, REF-16 | cw-4 | REF-15: data isolation architecture (todo). REF-16: webhook auth + TLS (todo). Ava currently focused on SOC 2 policies. Eng Plan Q5 (container vs. process vs. logical isolation) still open. |

### Summary

| Status | Count |
|--------|-------|
| Done | 1 (MVP-1) |
| In Progress | 2 (MVP-2, MVP-4) |
| Design Done | 1 (MVP-6) |
| Partial | 1 (MVP-5) |
| Not Started | 2 (MVP-3, MVP-7) |

**Overall: 3 of 7 requirements on track. 2 not started — MVP-3 and MVP-7 need to begin by cw-3 to hit cw-5 target.**

---

## 2. Critical Path Tracking

```
converge-core proof (MVP-1)       ✅ DONE
  → HITL gate v1 (MVP-4)          🔄 IN PROGRESS (REF-42, Eli, cw-3)
    → Observation UI (MVP-2)       🔄 IN PROGRESS (REF-54, Jules, cw-3)
      → Webhook integration (MVP-3) ⬜ NOT STARTED (Eli+Leo, cw-4)
        → Telemetry wired (MVP-6)   📋 DESIGN DONE (Sam, cw-4)
          → Data isolation (MVP-7)   ⬜ NOT STARTED (Ava, cw-4)
            → Integration test       ⬜ NOT STARTED (Sam, cw-5)
              → PILOT READY          🎯 Target: April 10
```

**Critical path risk: Eli Marsh is the bottleneck.** MVP-1 (done), MVP-4 (in progress), MVP-3 (not started), and MVP-5 (partial) all run through Eli. If MVP-4 slips past cw-3, everything downstream shifts.

---

## 3. Non-MVP Dependencies

These don't gate the engineering build but are required for a successful pilot engagement.

| Dependency | Owner | Status | Issue | Target | Impact |
|-----------|-------|--------|-------|--------|--------|
| **Design partner identified** | Morgan + Blake | NOT STARTED | REF-41 (critical, todo) | cw-4 (Mar 27) | **No partner = no pilot.** Blake's outreach depends on Kenneth's warm intros. Target: 3-5 candidates by Mar 27. |
| **Pricing tiers finalized** | Blake + Morgan | IN PROGRESS | — | cw-3 | Needed for pilot close / conversion. Priya flagged Professional tier ($349/mo) may be margin-negative. |
| **Pilot agreement template** | Leo Marin | IN REVIEW | REF-57 | cw-3 | v1.1 draft looks solid. Needs legal review before first use. See §5 below for product completeness notes. |
| **Security one-pager** | Ava Petrov | DONE | — | — | Available for customer distribution. |
| **Pilot-to-contract playbook** | Blake + Leo | IN PROGRESS | cecca59f | cw-4 | Leo drafted operational side. Blake owns commercial. |
| **CI/CD pipeline** | Dex Tanaka | IN PROGRESS | REF-56 | cw-3 | Needed for repeatable builds and deployment. |
| **Pilot runbook** | Dex Tanaka | NOT STARTED | — | cw-5 | Setup checklist, troubleshooting, monitoring. No issue created yet. |

---

## 4. Risks and Escalations

| # | Risk | Severity | Status | Action Required |
|---|------|----------|--------|----------------|
| R1 | **Eli is single point of failure** on critical path (MVP-1, 3, 4, 5) | CRITICAL | OPEN | Kira Novak is backup per Eng Plan. Ren reviewing all core PRs. Monitor weekly. |
| R2 | **No design partner candidate yet** (REF-41) | HIGH | OPEN | Morgan + Blake must identify candidates by Mar 27. Kenneth's warm intros are key. If no candidates by end of cw-3, escalate to Kenneth. |
| R3 | **HITL rejection bug** (REF-61) | HIGH | OPEN | Infinite pause loop on rejection. Must fix before any pilot — HITL is customer-facing. Eli to triage. |
| R4 | **MVP-7 (data isolation) not started** | HIGH | OPEN | Ava is on SOC 2 policies. REF-15 and REF-16 are todo. Architecture decision (Eng Plan Q5) still open. Must begin by cw-3 to hit cw-4 target. |
| R5 | **MVP-3 (webhook) has no issue or owner** | MEDIUM | OPEN | Engineering Plan says Eli + Leo, target cw-4. But no Paperclip issue exists. **Action: Ren should create issue and assign.** |
| R6 | **Pricing risk** — Professional tier may be margin-negative | MEDIUM | OPEN | Priya recommends raising to $499/mo or cutting included runs. Blake + Morgan to decide. Doesn't block pilot start but blocks conversion. |
| R7 | **Pilot runbook not started** | LOW | OPEN | Dex owns, target cw-5. No issue exists. Not urgent yet — create issue by end of cw-3. |

---

## 5. Pilot Agreement Template — Product Completeness Review

Reviewed Leo's pilot agreement template (v1.1) against PRD MVP requirements:

### What's covered well:
- Success criteria with measurable baselines (§3) — aligns with PRD §7
- Data handling and disposal (§6) — comprehensive, aligns with MVP-7 and security policies
- HITL controls referenced in onboarding playbook — aligns with MVP-4
- Case study consent (§8) — matches design partner selection criteria #6
- Post-pilot options and conversion path (§11) — clear

### Gaps / recommendations for Leo (REF-57):

1. **MVP-2 (Observation UI) not explicitly in agreement.** §4 says "full Converge platform access" but doesn't mention the observation dashboard. Recommend adding: "Real-time convergence observation dashboard" to §4 deliverables table. Customer expectation should be set.

2. **HITL gate mechanics not in agreement.** The onboarding playbook covers HITL training, but the agreement should set expectations: "Partner will receive approval requests via [Slack/email] and is expected to respond within [X business hours]." HITL response time affects convergence completion — if customer ignores approvals, convergence stalls.

3. **Run allowance (2,500) needs validation.** Is 2,500 runs over 4 weeks realistic? At ~90 runs/day, a busy lead-to-cash workflow could burn through this. Recommend Leo validate with Eli on expected run volume for P0 use cases, and add a clause for what happens if the allowance is exceeded.

4. **No mention of LLM cost pass-through.** The agreement says $0 to partner, but Priya's financial model shows LLM costs are significant. The agreement should note that LLM costs are absorbed by Converge during the pilot — this prevents surprises if we later need to discuss cost structure.

5. **Missing: pilot environment vs. production disclaimer.** §12 says "pilot environment may differ from production" but doesn't explain how. For trust: briefly note that pilot uses InMemory store (not production database) and that production migration is handled during conversion.

---

## 6. Weekly Review Cadence

This tracker should be reviewed weekly with Ren Akiyama (engineering status) and Blake Harmon (GTM status).

**Proposed cadence:**
- **Monday:** Nadia updates tracker with latest issue statuses from Paperclip
- **Tuesday:** Review with Ren — engineering blockers, critical path progress
- **Thursday:** Review with Blake — design partner pipeline, pricing, pilot commercial readiness
- **Friday:** Post weekly summary comment on REF-60

### Week of cw-2 (Mar 16-20) — Key questions:
- [ ] Is Eli's LlmAgent idempotency bug fixed?
- [ ] Have Jules and Eli aligned on Observation UI API contract (Eng Plan Q3)?
- [ ] Has Ava started data isolation architecture design (Eng Plan Q5)?
- [ ] Has Blake identified any warm intro candidates for design partners (REF-41)?
- [ ] Is REF-61 (HITL rejection bug) triaged and prioritized?

---

## Revision History

| Date | Changes |
|------|---------|
| 2026-03-13 | Initial tracker — Nadia Reeves. MVP-1 through MVP-7 status, critical path, risks, pilot agreement review. |
