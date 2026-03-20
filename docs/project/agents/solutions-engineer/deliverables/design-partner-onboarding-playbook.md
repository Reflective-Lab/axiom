# Design Partner Onboarding Playbook

> Repeatable kickoff-to-live runbook. Pilot 2 should take half the effort of Pilot 1.

**Owner:** Leo Marin, Solutions Engineer
**Reviewers:** Ren Akiyama (engineering), Ava Petrov (security)
**Status:** Draft v1.1 (updated with PRD v1.2 decisions)
**Created:** 2026-03-12
**Updated:** 2026-03-12

---

## Overview

This playbook covers everything from signed pilot agreement to verified first production convergence run. It is organized into 5 phases with explicit checklists, owners, and time targets.

**Total target:** Agreement signed → first convergence run in ≤ 5 business days.

```
Phase 1: Pre-Kickoff (Day -5 to 0)
Phase 2: Environment Provisioning (Day 0-1)
Phase 3: Integration Setup (Day 1-3)
Phase 4: Customer Training & First Run (Day 3-5)
Phase 5: Steady State & 30-Day Check-In (Day 5-30)
```

---

## Phase 1: Pre-Kickoff (Day -5 to Day 0)

**Owner:** Leo Marin
**Goal:** Everything is ready before the customer shows up on Day 1.

### 1.1 Internal Readiness Checklist

- [ ] Pilot agreement signed (see `pilot-to-contract-playbook.md` §2.3)
- [ ] Discovery brief completed and shared with engineering (1-page)
- [ ] Success criteria documented and agreed with customer (2-3 measurable outcomes)
- [ ] Baseline metrics collection started (≥ 2 weeks of historical data; see Pilot Metrics Framework §3.1)
- [ ] Customer technical lead identified and responsive
- [ ] Internal Slack channel created: `#pilot-{customer-name}`
- [ ] Ava Petrov security review completed (see §1.3 below)

### 1.2 Engineering Handoff

Share the following with Ren Akiyama and assigned engineer:

| Item | Source |
|------|--------|
| Customer workflow map | Discovery brief |
| Systems to integrate | Discovery brief |
| API documentation for target systems | Customer technical lead |
| Authentication method for each integration | Customer technical lead |
| Expected data volume and cadence | Discovery brief |
| Latency requirements | Success criteria |

**Output:** Engineering confirms feasibility and estimates integration effort.

### 1.3 Security Review (Ava Petrov)

Before any customer data enters our system:

- [ ] Data classification confirmed (what data types will flow through Converge?)
- [ ] Customer compliance requirements documented (GDPR, HIPAA, SOC 2, etc.)
- [ ] Data residency requirements identified (per Ava's data classification policy)
- [ ] Secrets management plan approved (no API keys in code/logs/env; see security one-pager)
- [ ] Data retention period agreed (default: 90 days post-pilot, per Pilot Metrics Framework §11.5)
- [ ] Incident response contacts exchanged (our side: Ava; their side: named contact)
- [ ] MVP-7 verified: no customer data leaves their environment or dedicated tenant

**Gate:** Ava signs off before proceeding to Phase 2.

### 1.4 Customer Pre-Work Request

Send to customer technical lead before Day 1:

```
Subject: Converge Pilot — Pre-Kickoff Checklist

Hi {Technical Lead},

To ensure a smooth kickoff, please have the following ready:

1. API credentials for {System A, System B} (we'll provide a secure upload link)
2. A test account or sandbox environment for {integration target}
3. Sample data file (CSV or JSON) showing 10-20 representative workflow instances
4. Names and emails of team members who need dashboard access (max 5 for pilot)
5. Your preferred async channel (Slack Connect, Teams, or email)

We'll set up everything on Day 1 so your team can see the first convergence run by Day 3.

Best,
Leo
```

---

## Phase 2: Environment Provisioning (Day 0-1)

**Owner:** Leo Marin + assigned engineer
**Goal:** Dedicated workspace running, ready for integration.

### 2.1 Workspace Setup

- [ ] Create dedicated Converge workspace for this customer
- [ ] Workspace name: `pilot-{customer-name}-{yyyymmdd}`
- [ ] Configure workspace-level isolation (no shared context with other pilots; MVP-7)
- [ ] Set convergence timeout to 60s (per MVP-1)
- [ ] Enable telemetry via InMemory store shim (per MVP-6; Morgan decision: InMemory for first 2 pilots, Sam builds lightweight shim)
- [ ] Configure experience store for audit trail (per MVP-5; InMemory store for initial pilots)

### 2.2 Agent Configuration

- [ ] Map customer workflow steps to Converge agents
- [ ] Create agent definitions with customer-specific JTBD prompts
- [ ] Configure agent context keys (each agent writes only to assigned keys)
- [ ] Set up HITL gates at customer-specified decision points (per MVP-4; HITL v1 = Slack/email notification + approve/reject API — no web UI)
- [ ] Configure HITL notification channel: Slack Connect (preferred) or email for customer approve/reject flow
- [ ] Test convergence with synthetic data (3+ agents → fixed point in < 60s; note: real LLM agents add ~3s/call latency)

**Target:** < 4 hours for workspace + agent configuration (per Pilot PRD user story 4.1).

### 2.3 Credential Management

- [ ] Receive customer API credentials via secure channel (1Password share or encrypted upload)
- [ ] Store credentials in secret management system (Google Secret Manager / HashiCorp Vault)
- [ ] Verify: no credentials in source code, config files, logs, or environment dumps
- [ ] Create credential rotation reminder (rotate at pilot end or every 30 days, whichever comes first)
- [ ] Document credential inventory: which secrets exist, where they're stored, who has access

### 2.4 Monitoring Setup

- [ ] Configure alerting for convergence failures (notify Leo within 5 minutes; per Pilot PRD NFR)
- [ ] Set up uptime monitoring (99% availability during business hours)
- [ ] Enable budget tracking for LLM API usage
- [ ] Create customer-facing pilot view at `converge.zone/pilot/{id}` (extending converge-www demo page; Ren's recommendation for first 2 partners)

---

## Phase 3: Integration Setup (Day 1-3)

**Owner:** Assigned engineer (Leo validates)
**Goal:** At least one integration receiving real events and producing real actions.

### 3.1 Inbound Integration (Events → Converge)

- [ ] Configure webhook receiver or API polling for customer's source system
- [ ] Validate event schema: correct fields, data types, timestamps
- [ ] Test with 5+ real events from customer's system
- [ ] Confirm events appear in convergence context within 5 seconds
- [ ] Error handling: failed events are logged, not silently dropped

### 3.2 Outbound Integration (Converge → Actions)

- [ ] Configure API client for customer's target system (CRM, ticketing, etc.)
- [ ] Implement action mapping: convergence output → target system API calls
- [ ] Test with sandbox/test account first (never production without customer approval)
- [ ] HITL gate review: customer approves before any action hits their production system
- [ ] Verify: actions are idempotent or have deduplication (no double-sends)

### 3.3 End-to-End Verification

- [ ] Trigger a real event → watch convergence run → verify action in target system
- [ ] Record end-to-end latency (target: < 60s for convergence + action)
- [ ] Customer technical lead observes and confirms correct behavior
- [ ] Document any deviations from expected behavior for engineering

**Gate:** Customer technical lead confirms integration works before moving to Phase 4.

---

## Phase 4: Customer Training & First Run (Day 3-5)

**Owner:** Leo Marin
**Goal:** Customer team can observe convergence and use HITL controls.

### 4.1 Customer Training Session (60 minutes)

**Attendees:** Customer pilot lead + 2-3 team members + executive sponsor (optional)

**Agenda:**

| Time | Topic | Materials |
|------|-------|-----------|
| 0-10 min | What Converge does with your workflow | Workflow diagram with agent mapping |
| 10-25 min | Live walkthrough: pilot observation page (`/pilot/{id}`) | Screen share of real convergence run |
| 25-35 min | HITL controls: Slack/email approve & reject flow | Hands-on with test scenario — customer receives real notification and approves |
| 35-45 min | Reading the dashboard: metrics and audit trail | Dashboard tour |
| 45-55 min | Q&A | — |
| 55-60 min | Next steps: pilot cadence and support channels | Weekly check-in calendar invite |

**Materials to prepare:**

- [ ] Workflow diagram: their process → Converge agents
- [ ] Quick-start guide (see §4.2)
- [ ] Dashboard access for all attendees
- [ ] Test scenario they can trigger during training

### 4.2 Customer Quick-Start Guide

Provide to customer after training. One page, scannable.

```markdown
# Converge Pilot — Quick Start

## Your Dashboard
URL: {dashboard-url}
Login: {auth method}

## What You'll See
- **Agent Activity:** Each step in your workflow is handled by a named agent.
  Agents propose facts — you'll see these as "Proposed" in the activity feed.
- **Convergence:** When agents agree, the state converges. You'll see "Converged"
  with the final outcome.
- **HITL Gates:** When Converge needs your approval, you'll get a Slack message
  (or email). Click "Approve" or "Reject" with an optional reason.

## Key Actions
- **Approve/Reject:** Click the link in your Slack/email notification → review the proposal → approve or reject via the API link.
- **View Audit Trail:** Click any convergence run → see which agents ran, what they
  proposed, and what was accepted.
- **Ask a Question:** Post in {async channel} — Leo responds within 4 hours (business hours).

## Support
- **Async channel:** {Slack/Teams/Email}
- **Response time:** 4 hours during business hours
- **Escalation:** Email leo@converge.zone with subject "[URGENT] {Customer Name}"
```

### 4.3 First Production Run

- [ ] Customer triggers a real workflow event (not test data)
- [ ] Leo and customer observe convergence together (screen share or same room)
- [ ] Customer uses HITL gate to approve first real action
- [ ] Verify action executed correctly in target system
- [ ] Celebrate the moment — this is the proof point

**Gate:** First successful production convergence run verified by customer.

---

## Phase 5: Steady State & 30-Day Check-In (Day 5-30)

**Owner:** Leo Marin
**Goal:** Pilot runs smoothly, metrics are captured, customer is engaged.

### 5.1 Weekly Cadence

Follow the weekly cadence from `pilot-to-contract-playbook.md` §3.2:

- **Monday:** Review prior week's metrics, flag anomalies
- **Tuesday-Thursday:** Available for customer questions (4hr response SLA)
- **Friday:** Send weekly status update (template in playbook §3.2)

### 5.2 Metrics Collection

Confirm these are being captured from Day 1 (per Pilot Metrics Framework):

- [ ] Cycle time per convergence run
- [ ] Agent execution time per step
- [ ] Convergence success/failure rate
- [ ] HITL gate response time (customer latency)
- [ ] Customer-defined success metrics (from pilot agreement)
- [ ] LLM API cost per run

### 5.3 Health Checks

Weekly internal review (Leo + assigned engineer):

- [ ] Convergence success rate > 90%?
- [ ] Any errors or timeouts in the past week?
- [ ] Customer engagement: are they using HITL gates? Checking the dashboard?
- [ ] Budget: LLM API spend within expected range?
- [ ] Security: any anomalies in data access patterns?

### 5.4 Stall Detection

If customer engagement drops (no HITL responses, no check-in attendance):

1. Day 1 of silence: Send async message — "Just checking in, anything we can help with?"
2. Day 3 of silence: Call customer pilot lead directly
3. Day 5 of silence: Escalate to customer's executive sponsor + inform Blake
4. Day 7+ of silence: Assess whether pilot should be paused or terminated

### 5.5 30-Day Check-In

At Day 30 (or pilot end, whichever comes first):

- [ ] Generate final metrics report (Sam Okafor runs analysis)
- [ ] Schedule conversion call (see `pilot-to-contract-playbook.md` §4)
- [ ] Prepare customer results summary (before/after metrics)
- [ ] Internal retrospective: what worked, what didn't, what to improve for next pilot
- [ ] Update this playbook with lessons learned

---

## Security Checklist (All Phases)

Cross-reference with Ava Petrov's policies. These are non-negotiable:

- [ ] No customer data in git commits, PR descriptions, or Slack messages
- [ ] No API keys in code, logs, environment dumps, or screenshots
- [ ] Customer data stays in their workspace — no cross-tenant data leakage
- [ ] All data handling follows the classification policy
- [ ] Credential rotation at pilot end (revoke pilot keys, issue production keys if converting)
- [ ] Data disposal at retention deadline (90 days post-pilot unless contract signed)
- [ ] Any security incident → immediately follow IRP (Ava's incident response plan)

---

## Pilot Closeout

### If Converting to Production

- [ ] Follow `pilot-to-contract-playbook.md` §5 for production transition
- [ ] Rotate all credentials to production keys
- [ ] Migrate from pilot workspace to production workspace
- [ ] Update SLA from best-effort to contractual
- [ ] Transfer support from Leo (direct) to support model (tier-dependent)

### If Not Converting

- [ ] Conduct exit interview with customer — why not? (document for field intelligence)
- [ ] Trigger data disposal process per retention policy
- [ ] Revoke all customer credentials and access
- [ ] Archive workspace (read-only, for internal analysis)
- [ ] Share field intelligence with Blake (messaging), Ren (product), and Nadia (PRD)
- [ ] Update playbook with lessons learned

---

## Repeatability Targets

| Metric | Pilot 1 Target | Pilot 2+ Target |
|--------|---------------|----------------|
| Pre-kickoff prep | 5 days | 3 days |
| Environment provisioning | 1 day | 4 hours |
| Integration setup | 2 days | 1 day |
| Customer training | 60 min live | 30 min live + recorded walkthrough |
| First convergence run | Day 5 | Day 3 |
| **Total kickoff-to-live** | **5 days** | **3 days** |

After each pilot, update this playbook:
1. What manual steps can be automated?
2. What questions did the customer ask that should be in the quick-start guide?
3. What integration patterns can be templated?
4. What security review steps can be pre-approved for common data types?

---

## Appendix: Templates Referenced

| Template | Location | Status |
|----------|----------|--------|
| Pilot agreement | `templates/pilot-agreement.md` | To be created |
| Discovery brief | `templates/discovery-brief.md` | To be created |
| Weekly status update | `templates/pilot-weekly-update.md` | To be created |
| Quick-start guide | Embedded in §4.2 above | Draft |
| Customer pre-work email | Embedded in §1.4 above | Draft |
| Production onboarding checklist | `pilot-to-contract-playbook.md` §5.2 | Draft |

---

## Open Dependencies

| Dependency | Owner | Status (per PRD v1.2) | Impact on Playbook |
|-----------|-------|----------------------|-------------------|
| HITL gate implementation (MVP-4) | Eli (REF-42) | **Decided:** Slack/email + approve/reject API. Not started. Target: cw-3. | Playbook updated with Slack/email flow. Needs implementation before first pilot. |
| Observation UI (MVP-2) | Jules | **Decided:** Extend converge-www demo with `/pilot/{id}` route. Partial. | Playbook updated. Note: existing demos are scripted — real engine connection needed. |
| Webhook integration framework (MVP-3) | TBD | **Decided:** Generic first with escape hatch for custom adapters (Eng Plan D3). Not started. Target: cw-4. | Phase 3 may still need per-pilot custom work for first pilot. |
| Data isolation architecture (MVP-7) | Ava (REF-15) | **Open:** Container vs. process vs. logical isolation (Eng Plan Q5). | Phase 2 workspace setup placeholder — will update when Ava/Ren decide. |
| Pricing tiers (Blake) | Blake | In progress. | Phase 5 conversion depends on finalized pricing. |
| Pilot telemetry (MVP-6) | Sam | **Decided:** InMemory store for first 2 pilots. Sam builds lightweight shim (Morgan decision). | Playbook updated. Reduced automation accepted for initial pilots. |

**Timeline (per Engineering Plan v2):** Pilot-ready target: end of cw-5 (April 10). First design partner engagement: cw-6 (April 13). First pilot agreement target: April 10 (REF-41).
