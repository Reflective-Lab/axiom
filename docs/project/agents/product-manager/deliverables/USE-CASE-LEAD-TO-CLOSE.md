# Use Case Requirements: Lead-to-Close Acceleration

> What Converge must do to automate the lead-to-close workflow for a pilot customer.

**Owner:** Nadia Reeves, Product Manager
**Status:** Draft v1.0
**Date:** 2026-03-12
**Parent:** [Pilot Program PRD](PILOT-PROGRAM-PRD.md) — Section 2.3, P0 use case
**Companion:** [Lead-to-Cash Demo](../../../agents/frontend-developer/memory/) (Jules' interactive demo provides the concept; this doc defines production requirements)

---

## 1. Why This Use Case First

Lead-to-Close acceleration is our P0 pilot use case because:

1. **Universal pain.** Every B2B company with a sales team has this problem. It's not niche.
2. **CRM integration is straightforward.** Most targets use HubSpot, Salesforce, or Pipedrive — all have webhook and API support (MVP-3).
3. **Cycle time is measurable.** Lead creation timestamp → closed-won timestamp. No ambiguity about before/after (MVP-6).
4. **The demo already exists.** Jules' Lead-to-Cash interactive demo shows the concept. Customers can see it before the pilot starts.
5. **Aligns with SMB beachhead.** Small B2B SaaS companies (15-60 employees) feel this pain acutely — leads fall through cracks when the team is small.

---

## 2. Workflow Definition

### 2.1 The Problem (Customer's Current State)

A typical SMB B2B sales workflow looks like this:

```
1. Lead comes in (form fill, inbound email, referral)
2. Someone (maybe) notices within hours/days
3. Manual CRM entry (if they remember)
4. Sales rep researches the company (LinkedIn, website)
5. Rep drafts an email or makes a call
6. Follow-up falls off after 1-2 attempts
7. If the lead responds, back-and-forth scheduling
8. Discovery call happens (maybe)
9. Proposal drafted manually
10. Internal approval for pricing/discounts
11. Proposal sent, then radio silence
12. Follow-up cycle repeats
13. Deal closes or dies quietly
```

**Where it breaks:**
- Step 2: Leads sit unnoticed for hours or days. Response time kills conversion.
- Steps 3-4: Manual data entry and research eat 15-30 min per lead.
- Step 6: Follow-up discipline is inconsistent. Leads go cold.
- Step 10: Internal approvals are slow — pricing needs manager sign-off, finance review.
- Step 12: No systematic follow-up cadence. Reps rely on memory.

### 2.2 What Converge Does (Pilot State)

```
1. Lead comes in → webhook triggers Converge
2. Enrichment Agent: researches company (public data), enriches CRM record
3. Qualification Agent: scores lead against ICP criteria, flags fit/no-fit
4. Routing Agent: assigns to best-fit rep based on territory, expertise, workload
5. Draft Agent: generates personalized first-touch email using enrichment data
   → HITL GATE: Rep reviews and approves/edits the email before sending
6. Follow-up Agent: schedules follow-up sequence, adjusts based on engagement signals
7. Meeting Agent: proposes available times, handles scheduling
8. Proposal Agent: generates proposal from template + deal-specific data
   → HITL GATE: Rep/manager reviews proposal before sending
9. Approval Agent: routes pricing/discount requests through approval chain
   → HITL GATE: Manager approves discount if above threshold
10. Close Agent: tracks final negotiation steps, nudges stalled deals
```

**Convergence model:** All agents run in parallel, proposing facts. The engine converges when:
- Lead is enriched and scored
- Rep is assigned
- First touch is approved and sent
- Follow-up sequence is scheduled

Each stage is a convergence cycle. Total workflow may involve 3-5 convergence runs over the lifetime of a deal.

### 2.3 HITL Gates for This Use Case

| Gate | Trigger | What the Customer Sees | Why It's Human |
|------|---------|----------------------|----------------|
| First-touch email | Draft Agent proposes email text | Email preview with recipient, subject, body | Reps need to own their voice; wrong first email kills the deal |
| Proposal send | Proposal Agent generates proposal | Proposal PDF/doc with pricing, terms, scope | Financial commitment — must be reviewed |
| Discount approval | Approval Agent routes discount request | Discount %, justification, deal context | Pricing authority — company policy |

**Expected HITL pattern:** Gates 1 and 2 fire for every deal. Gate 3 fires only when discount > threshold (configurable per customer). Over time, as trust builds, Gate 1 may shift to auto-approve with spot checks.

---

## 3. Requirements Specific to This Use Case

### 3.1 Integration Requirements (MVP-3 Specifics)

| Integration | Direction | Method | Priority |
|------------|-----------|--------|----------|
| CRM (HubSpot or Salesforce) | Inbound: new lead webhook | Webhook listener | Required |
| CRM | Outbound: update lead/deal record | REST API | Required |
| Email (Gmail or Outlook) | Outbound: send approved email | OAuth + API | Required |
| Calendar (Google or Outlook) | Read/write: scheduling | OAuth + API | Should-have |
| Company enrichment | Outbound: lookup company data | REST API (Clearbit, Apollo, or public scraping) | Should-have |

**Minimum for first pilot:** CRM webhook (in) + CRM API (out) + Email API (out). That's 3 integrations.

### 3.2 Agent Configuration

For this use case, the convergence workspace needs these agents:

| Agent | Role | Input | Output |
|-------|------|-------|--------|
| Lead Intake | Receive webhook, create root intent | CRM webhook payload | Root intent with lead data |
| Enrichment | Research company, enrich data | Lead name, email, company | Enriched lead profile (ProposedFact) |
| Qualification | Score against ICP criteria | Enriched profile + customer's ICP rules | Qualification score + reasoning (ProposedFact) |
| Routing | Assign to best-fit rep | Qualification result + rep availability | Rep assignment (ProposedFact) |
| Draft | Generate first-touch email | Enrichment data + email templates | Email draft (ProposedFact → HITL gate) |
| Follow-up | Schedule follow-up sequence | Email sent confirmation + engagement signals | Follow-up plan (ProposedFact) |
| Proposal | Generate proposal document | Deal context + pricing templates | Proposal draft (ProposedFact → HITL gate) |
| Approval | Route pricing decisions | Proposal with discount > threshold | Approval request (ProposedFact → HITL gate) |

**Agent count:** 8 agents per workspace. This is within the MVP-1 target of 3-5 agents per convergence run (not all 8 run in every cycle — typically 3-5 are active per convergence round).

### 3.3 Metrics for This Use Case

Per Pilot Metrics Framework, these are the specific metrics we capture:

| Metric | Before (Manual) | Target (With Converge) | How to Measure |
|--------|-----------------|----------------------|----------------|
| Lead response time | Hours to days | < 15 minutes | Timestamp: lead created → first email sent |
| Lead enrichment time | 15-30 min manual | < 30 seconds | Timestamp: lead created → enrichment complete |
| Follow-up consistency | Inconsistent (rep-dependent) | 100% of leads get follow-up sequence | Count: leads with active follow-up / total leads |
| Proposal turnaround | Days (manual drafting) | < 1 hour (draft ready for review) | Timestamp: proposal requested → draft available |
| Approval cycle time | Hours to days (email/Slack back-and-forth) | < 30 minutes | Timestamp: approval requested → approved/rejected |
| Overall cycle time (lead to close) | Weeks to months | Reduce by 30-50% | CRM: lead created → deal closed-won |

**Primary headline metric:** Lead response time. This is the most dramatic improvement and the easiest to measure.

---

## 4. Acceptance Criteria for Pilot Readiness (This Use Case)

A pilot running the lead-to-close use case is ready when:

- [ ] CRM webhook fires and Converge receives it within 5 seconds
- [ ] Enrichment agent produces a valid company profile for >80% of leads
- [ ] Qualification agent scores leads correctly (verified against 10 test leads)
- [ ] Draft agent produces a reasonable first-touch email (reviewed by pilot customer)
- [ ] HITL gate pauses convergence and notifies rep via Slack/email
- [ ] Rep can approve/reject email within 3 clicks
- [ ] Approved email is sent via customer's email system
- [ ] Follow-up sequence activates after first email is sent
- [ ] All events are logged in audit trail
- [ ] Baseline metrics are captured for at least 10 historical leads

---

## 5. Risks Specific to This Use Case

| Risk | Severity | Mitigation |
|------|----------|------------|
| CRM webhook setup is customer-specific and fragile | Medium | Leo documents setup per CRM; build generic webhook adapter (Eng Plan D3) |
| Email deliverability — AI-drafted emails may trigger spam filters | High | All emails go through customer's own email system (OAuth), not ours. Draft quality reviewed at HITL gate. |
| Enrichment data quality varies | Medium | Fallback: if enrichment fails, flag for manual research. Don't block the workflow. |
| Rep adoption — reps may ignore HITL notifications | Medium | Track response time at HITL gates. If >50% timeout, switch notification channel. |
| Convergence time with 8 agents + LLM calls may exceed 60 sec | Medium | First convergence round (intake + enrich + qualify + route + draft) is the critical path. Profile and optimize this round first. |

---

## 6. What We Learn From This Pilot

This pilot is not just about closing one deal. It validates:

1. **Integration model:** Can we reliably connect to a customer's CRM and email? (Informs all future pilots)
2. **HITL pattern:** How often do reps approve vs. reject? What do they edit? (Informs auto-approve thresholds)
3. **Convergence performance:** How fast does the engine converge with real LLM agents and real data? (First production benchmark for MVP-1)
4. **Agent configuration effort:** How long does it take Leo to set up a workspace? (Informs setup time estimates for Pilot 2+)
5. **Customer trust arc:** How quickly does the customer go from "review everything" to "auto-approve most"? (Informs product roadmap for HITL Phase 2)

---

## 7. Open Questions (Use-Case Specific)

| # | Question | Who Decides |
|---|----------|-------------|
| UC-1 | Which CRM do we support first? HubSpot is more common in SMB; Salesforce in mid-market. | Depends on first design partner (REF-41) |
| UC-2 | Do we send emails directly, or just draft them for the rep to send manually? | Nadia — recommend: draft + HITL approve + auto-send for Pilot 1, with manual send as fallback |
| UC-3 | How do we handle leads that come in outside business hours? | Queue until next business day, or process immediately with HITL gate timeout? Customer decides. |
| UC-4 | What's the minimum CRM data we need per lead? (Name, email, company — anything else?) | Define during scoping session (Playbook Phase 1) |

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-03-12 | 1.0 | Initial draft — Nadia Reeves |
