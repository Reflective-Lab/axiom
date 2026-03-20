# SLA Tier Definitions & Support Cost Model

> Operational SLA design for Converge production customers. What support each tier gets, how fast we respond, and what it costs.

**Owner:** Leo Marin, Solutions Engineer
**Status:** v1.1 — Approved (VP Eng review incorporated)
**Date:** 2026-03-12
**Financial model by:** Priya Chandran (`agents/finance-operations/deliverables/support-cost-model.md`)
**Related:** [Support Touchpoint Instrumentation](support-touchpoint-instrumentation.md), [Pilot-to-Contract Playbook](pilot-to-contract-playbook.md), [Pilot Commercial Terms](../../finance-operations/deliverables/pilot-commercial-terms.md)

---

## 1. Tier Overview

| | Starter ($0/mo) | Professional ($499/mo) | Enterprise (Custom, ≥$2,000/mo) |
|--|-----------------|----------------------|-------------------------------|
| **Target customer** | Individual devs, evaluators | Mid-market ops teams (50-500 emp) | Large orgs with complex workflows |
| **Support model** | Self-serve only | Shared queue | Named contact |
| **Channels** | Docs, community forum | Email + async chat | Dedicated Slack/Teams + email + phone |
| **Included runs** | 100/mo | 2,500/mo | Custom |
| **Onboarding** | Self-serve | Guided (7-10 hrs Leo) | White-glove (25-51 hrs Leo + eng) |

---

## 2. SLA Definitions

### 2.1 Severity Levels

Severity levels are consistent with the [Support Touchpoint Instrumentation Spec](support-touchpoint-instrumentation.md):

| Severity | Definition | Examples |
|----------|-----------|----------|
| **P0 — Critical** | Production workflow halted. No workaround. | Engine crash, data integrity issue, convergence failure |
| **P1 — High** | Production workflow degraded. Workaround exists. | Slow convergence (>60s), HITL gate timeout, integration flapping |
| **P2 — Medium** | Non-blocking issue. Workflow runs but with friction. | UI confusion, incorrect proposal (caught by HITL), config question |
| **P3 — Low** | Enhancement request or general question. | Feature request, documentation question, best-practice advice |

### 2.2 Response Time SLAs

Response time = time from customer's initial report to first substantive reply (not an autoresponder).

| Severity | Starter | Professional | Enterprise |
|----------|---------|-------------|------------|
| **P0** | N/A | **4 hours** (business hours) | **1 hour** (business hours; 24/7 deferred to Enterprise v2) |
| **P1** | N/A | **8 hours** (business hours) | **4 hours** (business hours) |
| **P2** | N/A | **24 hours** | **8 hours** |
| **P3** | N/A | **48 hours** | **24 hours** |

**Business hours:** 9:00 AM – 6:00 PM ET, Monday–Friday, excluding US federal holidays.

### 2.3 Resolution Time Targets

Resolution targets are best-effort goals, not contractual SLAs (except where noted for Enterprise).

| Severity | Professional | Enterprise |
|----------|-------------|------------|
| **P0** | 8 hours | **8 hours** (contractual), 4 hours best-effort goal |
| **P1** | 2 business days | 1 business day |
| **P2** | 5 business days | 3 business days |
| **P3** | Best effort (14 days) | 5 business days |

### 2.4 Escalation Paths

| Escalation Level | Who | When |
|-----------------|-----|------|
| **L1 — Solutions Engineer** | Leo Marin | All inbound tickets |
| **L2 — Engineering** | Eli Marsh / Kira Novak | Integration bugs, engine issues, performance |
| **L3 — VP Engineering** | Ren Akiyama | P0 unresolved after 2 hours, resource conflicts |
| **L4 — CEO** | Morgan Vale | Customer relationship risk, contract disputes |

**Auto-escalation rules:**
- P0 not acknowledged within 30 min → page Ren
- P0 not resolved within 8 hours → notify Morgan
- P1 not resolved within 2 business days → escalate to Ren
- Customer satisfaction score <3/5 on two consecutive interactions → Nadia notified

---

## 3. Channel Matrix

### 3.1 Starter Tier

| Channel | Available | Notes |
|---------|-----------|-------|
| Documentation site | Yes | Self-serve docs, tutorials, API reference |
| Community forum | Yes | Best-effort community answers |
| Email support | No | — |
| Chat support | No | — |
| Phone | No | — |
| Dedicated Slack | No | — |

### 3.2 Professional Tier

| Channel | Available | Notes |
|---------|-----------|-------|
| Documentation site | Yes | Full access |
| Community forum | Yes | Full access |
| Email support | **Yes** | support@converge.zone — shared queue |
| Async chat | **Yes** | In-app chat widget, business hours |
| Phone | No | Available as paid add-on ($100/mo) |
| Dedicated Slack | No | Available as paid add-on ($200/mo) |

### 3.3 Enterprise Tier

| Channel | Available | Notes |
|---------|-----------|-------|
| Documentation site | Yes | Full access + early access docs |
| Community forum | Yes | Full access |
| Email support | **Yes** | Direct to named SE |
| Async chat | **Yes** | Dedicated channel |
| Phone | **Yes** | Named SE direct line, business hours |
| Dedicated Slack/Teams | **Yes** | Shared channel with customer team |

---

## 4. Support Inclusions Per Tier

### 4.1 Professional — Included Support

| Activity | Included | Overage |
|----------|----------|---------|
| Email/chat support tickets | Unlimited | — |
| Guided configuration changes | 2 per month | $150/hr |
| Quarterly business review | No | Available at $200/session |
| Integration troubleshooting | Up to 2 hrs/mo | $150/hr |
| Custom workflow consulting | No | Available at $200/hr |
| Onboarding (one-time) | 7-10 hours | — |

### 4.2 Enterprise — Included Support

| Activity | Included | Overage |
|----------|----------|---------|
| All Professional inclusions | Yes | — |
| Named Solutions Engineer | Yes | — |
| Quarterly business review | Yes (1/quarter) | Extra sessions $200 each |
| Integration troubleshooting | Up to 10 hrs/mo | $200/hr |
| Custom workflow consulting | Up to 4 hrs/mo | $200/hr |
| Priority engineering escalation | Yes | — |
| Onboarding (one-time) | 25-51 hours | — |
| Annual architecture review | Yes | — |

### 4.3 Enterprise Support Hour Caps

Per Priya's recommendation: Enterprise contracts should scope support hours to prevent unlimited drain.

**Recommended structure:**
- **Standard Enterprise:** 15 hrs/mo included support, overage at $200/hr
- **Premium Enterprise:** 30 hrs/mo included support + 24/7 P0 coverage, overage at $200/hr

This caps worst-case Enterprise support cost at a known number and makes expansion revenue possible when customers exceed their allocation.

---

## 5. Effort Estimate Validation

Priya's cost model estimates the following effort per tier. I've validated against my onboarding playbook and pilot-to-contract experience:

| Activity | Priya's Estimate | My Validation | Notes |
|----------|-----------------|--------------|-------|
| Pro support labor | 1.5-3 hrs/mo | **Agree, lean toward 2-3 hrs** | First 3 months will be higher (~4 hrs) as we debug integration patterns |
| Enterprise support labor | 10-30 hrs/mo | **Agree, narrow to 12-20 hrs** | 30 hrs assumes significant custom work; with hour caps this is controllable |
| Pro onboarding | 7-10 hrs | **Agree** | Matches my playbook Phase 2-3 estimates |
| Enterprise onboarding | 25-51 hrs | **Agree, typical case ~35 hrs** | 51 hrs is worst case (legacy systems, complex auth); plan for 35 |
| Escalation eng time (Pro) | 0.5-1 hr/mo | **Agree** | Will be higher initially, declining as we fix common integration issues |
| Escalation eng time (Ent) | 2-6 hrs/mo | **Revise to 2-4 hrs/mo** | With defined escalation paths and the L2 triage, most issues resolve at L1 |

**Net assessment:** Priya's numbers are sound. The wide ranges reflect genuine uncertainty (no production data yet). I recommend using midpoint estimates for pricing decisions and tracking actuals from Pilot 1 to narrow the ranges.

---

## 6. Recommendations to Blake (Pricing Tier Support Inclusions)

### 6.1 For the Pricing Page

| Support Feature | Starter | Professional | Enterprise |
|----------------|---------|-------------|------------|
| Documentation & community | ✓ | ✓ | ✓ |
| Email support | — | ✓ | ✓ |
| Async chat | — | ✓ | ✓ |
| Response SLA | — | Next business day | 4-hour |
| Named Solutions Engineer | — | — | ✓ |
| Dedicated Slack/Teams | — | — | ✓ |
| Quarterly business review | — | — | ✓ |
| Guided onboarding | — | ✓ (7-10 hrs) | ✓ (white-glove) |

### 6.2 Pricing Recommendations

1. **Raise Enterprise floor to $2,000/mo** (per Priya's analysis). At $1,500/mo, Enterprise can be margin-negative with dedicated support. At $2,000/mo, even high-touch accounts are margin-positive.

2. **Offer phone and dedicated Slack as Professional add-ons.** This creates upsell opportunity ($100-200/mo) and keeps base Professional support cost manageable. Some mid-market buyers will pay for it; most won't need it.

3. **Annual contracts get 15% discount.** This is already in Priya's pilot commercial terms and makes economic sense — onboarding amortization means month-to-month Pro customers are unprofitable in Month 1.

4. **Enterprise support hours should be scoped in the contract.** Recommend 15 hrs/mo standard, 30 hrs/mo premium. This is a differentiation lever and protects gross margin.

5. **Starter gets zero support.** The documentation must be good enough that Starter users are entirely self-serve. Investment in docs deflects future Professional tickets too — dual benefit.

### 6.3 Messaging Suggestions

For the pricing page, avoid listing SLA response times in hours for Professional. Instead:
- Professional: "Next business day response"
- Enterprise: "4-hour response, dedicated engineer"

This is accurate, customer-friendly, and avoids committing to specific numbers publicly that are harder to change later.

---

## 7. Capacity Planning Integration

From Priya's model, with my validation:

| Threshold | Customer Mix | Support Hrs/Mo | Status |
|-----------|-------------|---------------|--------|
| **Green** | ≤5 Pro + 1 Ent | 0-30 | Leo handles everything |
| **Yellow** | ≤10 Pro + 2 Ent | 30-80 | Invest in support tooling (Linear) |
| **Red** | >10 Pro + 3 Ent | 80+ | Hire 2nd Solutions Engineer |

**Trigger for tooling investment:** When support load exceeds 30 hrs/mo, move from manual tracking (email + spreadsheet) to Linear. Cost: ~$50/mo. This should happen around customer #5-6.

**Trigger for hiring:** When support load exceeds 80 hrs/mo or I'm spending >50% of time on support (leaving insufficient time for pre-sales, onboarding, and field intelligence). Expected around Month 9-12.

---

## 8. Pilot Period SLA

During the free pilot period (per Priya's commercial terms, REF-38):

| | Pilot SLA |
|--|-----------|
| Response time | P0: 4 hours, P1: same business day, P2-P3: next business day |
| Channels | Email + shared Slack channel |
| Support hours | Capped at pilot agreement terms (typically 10-15 hrs total) |
| Named contact | Leo Marin |
| Engineering escalation | Yes (through Ren) |

The pilot SLA is intentionally generous — it's our audition. But it's also time-boxed and capped, so a long-running pilot doesn't become a de facto free Enterprise tier.

---

## 9. Open Questions

1. **CRM tooling decision** — Manual tracking is fine for first 3 partners (per Ren). Revisit at customer #4.
2. **24/7 P0 coverage for Enterprise** — ~~listed as an add-on.~~ **Deferred to Enterprise v2** per Ren's review. Cannot staff on-call rotation at current team size.
3. **Self-serve investment timeline** — Ren will queue a docs site task for cw-5/6. Jules to build a lightweight docs site. Docs-first deflection strategy confirmed.
4. **Customer satisfaction measurement** — how do we capture CSAT per interaction? Need to decide on tooling before first pilot.

---

## 10. Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-03-12 | 1.0 | Initial draft — Leo Marin. Built on Priya's cost model, validated effort estimates. |
| 2026-03-12 | 1.1 | VP Eng review incorporated: P0 Enterprise resolution → 8 hrs contractual (4 hrs best-effort), 24/7 coverage deferred to v2, docs site queued cw-5/6, CRM deferred to customer #4. |
