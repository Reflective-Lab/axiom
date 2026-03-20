# Pilot-to-Contract Playbook

> From free pilot to paid production contract. The operational runbook for converting design partners.

**Owner:** Leo Marin, Solutions Engineer
**Collaborators:** Blake Harmon (pricing/narrative), Priya Chandran (finance), Ava Petrov (security/compliance)
**Status:** v1.1 — Updated with approved SLA tiers and Blake's pricing input
**Created:** 2026-03-12

---

## 1. The Funnel

```
Discovery → Qualification → Pilot (free, 3-4 weeks) → Conversion → Production Contract (annual) → Expansion
```

| Stage | Duration | Owner | Exit Criteria |
|-------|----------|-------|---------------|
| Discovery | 1-2 weeks | Blake + Leo | Use case mapped, technical fit confirmed |
| Qualification | 1 week | Leo | Success criteria agreed, data readiness confirmed |
| Pilot (free) | 3-4 weeks | Leo (execution), Sam (metrics) | ≥2 success thresholds met (see Pilot Metrics Framework §8) |
| Conversion | 1-2 weeks | Blake (commercial), Leo (technical) | Contract signed |
| Production | Ongoing | Leo (onboarding), Ren (engineering) | Customer live on production tier |
| Expansion | Quarter 2+ | Blake + Leo | Additional packs or seats added |

---

## 2. Pre-Pilot: Discovery & Qualification

### 2.1 Discovery Call (Led by Blake, Leo attends)

**Goal:** Understand the customer's pain and assess rough fit.

Checklist:
- [ ] What process are they trying to improve? (Specific workflow, not "general AI")
- [ ] What does "success" look like in their words?
- [ ] What systems are involved? (CRM, ERP, email, internal tools)
- [ ] Who are the stakeholders? (Decision maker, technical lead, end users)
- [ ] What's their timeline? (Urgent need vs exploratory)
- [ ] Budget range and procurement process

**Output:** Discovery brief (1-page, template: `templates/discovery-brief.md`), shared with Ren for technical assessment.

### 2.2 Technical Qualification (Led by Leo)

**Goal:** Confirm Converge can deliver measurable value for this use case.

Checklist:
- [ ] Map their workflow end-to-end (all steps, all actors, all tools)
- [ ] Identify which steps are candidates for agent automation
- [ ] Assess integration complexity (APIs available? Data formats? Auth?)
- [ ] Confirm baseline metrics are measurable (see Pilot Metrics Framework §3.1)
- [ ] Identify blockers: does this need Wave 2+ crates that aren't built yet?
- [ ] Security review with Ava: customer data classification, compliance requirements
- [ ] Finance check with Priya: pilot cost estimate vs expected contract value

**Decision gate:** Do we offer a pilot? Yes/No with written rationale.

**Disqualifiers** (say no early):
- Use case requires crates that won't ship for 8+ weeks
- Customer can't provide baseline data
- Integration requires capabilities we don't have (e.g., on-prem deployment)
- Data compliance requirements exceed our current posture
- No clear decision maker or the decision maker isn't engaged

### 2.3 Pilot Agreement

Before any data collection begins, the customer signs a lightweight pilot agreement covering:

1. **Scope:** Which workflow(s) are in scope
2. **Duration:** 3-4 weeks, with specific start and end dates
3. **Success criteria:** 2-3 measurable outcomes, agreed in writing
4. **Data handling:** Retention clause (see Pilot Metrics Framework §11.5)
5. **IP:** Anonymized results may be used as case studies (opt-out available)
6. **Cost:** Free. No commitment beyond pilot duration.
7. **Contact:** Named pilot lead on each side

Template: `templates/pilot-agreement.md` (to be created)

---

## 3. During Pilot: Execution

### 3.1 Kickoff (Day 1)

- [ ] Pilot agreement signed
- [ ] Baseline data collection started (2 weeks prior, per Metrics Framework §3.1)
- [ ] Integration hooks deployed and tested
- [ ] Converge telemetry enabled and verified
- [ ] Weekly check-in scheduled (15 min, same time each week)
- [ ] Customer Slack/Teams channel created (or equivalent async channel)
- [ ] Pilot dashboard shared with customer stakeholders

### 3.2 Weekly Cadence

**Monday:** Leo reviews metrics from prior week, flags anomalies.
**Tuesday-Thursday:** Normal execution. Leo available for customer questions (SLA: 4hr response during business hours).
**Friday:** Automated metric snapshot. Leo sends weekly status update (full template: `templates/pilot-weekly-update.md`):

```
Subject: [Converge Pilot] Week {N} Update — {Customer}

## Status: 🟢 On Track / 🟡 Needs Attention / 🔴 At Risk

## Metrics This Week
| Metric | Baseline | This Week | Delta |
|--------|----------|-----------|-------|
| ...    | ...      | ...       | ...   |

## Highlights
- [1-2 wins or observations]

## Blockers
- [Any issues, or "None"]

## Next Week
- [What we're focused on]
```

### 3.3 Escalation Path

| Severity | Example | Response | Escalation To |
|----------|---------|----------|---------------|
| Low | Minor metric dip | Note in weekly update | -- |
| Medium | Integration issue affecting data collection | Fix within 24h, notify customer | Ren (if engineering help needed) |
| High | Pilot blocked, customer can't use system | Fix within 4h, customer call within 8h | Ren + Morgan |
| Critical | Data breach or security incident | Immediate response per IRP | Ava + Morgan + Ren |

---

## 4. Post-Pilot: The Conversion Window

> The 72 hours after the pilot ends are the highest-leverage moment. Move fast.

### 4.1 Day-After-Pilot Actions (Day 0-1)

1. **Generate final metrics report** (Sam runs analysis, see Metrics Framework §3.3)
2. **Leo reviews results against success criteria** — did we hit ≥2 thresholds?
3. **Leo sends day-after-pilot email** (template below)
4. **Blake prepares commercial proposal** based on usage data and tier fit

### 4.2 Day-After-Pilot Email Template

```
Subject: Your Converge Pilot Results — {Customer}

Hi {Decision Maker},

Thank you for partnering with us on the {workflow name} pilot. Here's your results summary:

**Pilot Duration:** {start} – {end} ({N} weeks)

**Key Results:**
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| {Primary metric} | X | Y | -Z% |
| {Secondary metric} | X | Y | -Z% |
| {Tertiary metric} | X | Y | +Z% |

{One sentence on the most surprising or compelling finding.}

**What's Next:**
We'd love to discuss moving to production. I've attached a proposal with three options based on your usage during the pilot. Blake Harmon (our VP of Sales) and I are available for a 30-minute call this week to walk through it.

Suggested times: {3 options within the next 5 business days}

Best,
Leo Marin
Solutions Engineer, Converge
```

### 4.3 The Conversion Call (Day 3-7)

**Attendees:** Customer decision maker + technical lead, Blake + Leo

**Agenda (30 min):**
1. (5 min) Recap pilot results — Leo presents metrics
2. (5 min) Customer feedback — what worked, what didn't
3. (10 min) Production proposal — Blake presents tiers and pricing
4. (5 min) Technical transition — Leo explains what changes from pilot to production
5. (5 min) Next steps and timeline

**Blake's materials:**
- Pricing one-pager with 3 tiers:
  - Starter: $0/mo (1 workspace, 2 packs, 500 runs) — discovery + pilot
  - Professional: $349/mo ($279/mo annual) (3 workspaces, 4 packs, 5K runs) — production
  - Enterprise: Custom (≥$2,000/mo) — 50+ employees, compliance needs
- Volume levers: additional runs at $0.02/run, workspaces at $99/mo, add-on packs at $149-$249/mo
- Annual contract draft (15% discount for annual commitment)
- *Note: Pricing pending Kenneth's final approval — do not use externally until approved*

**Leo's materials:**
- Final pilot metrics report
- Production deployment plan (what changes from pilot infra)
- Ongoing support model and SLA options

### 4.4 Objection Handling

| Objection | Response | Escalation |
|-----------|----------|------------|
| "Results were good but not enough to justify the cost" | Reframe with annual math: team spent X hrs/week on manual handoffs at $Y/hr = $Z/year. Converge Professional at $4,188/year replaces that plus audit trail. | Blake for pricing flexibility |
| "We need to run it longer" | Offer 2-week extension at no cost (one time only). Set clear criteria for what "longer" will prove. | Ren for engineering capacity |
| "Security/compliance concerns" | Share Ava's security one-pager and SOC 2 progress. Offer security review call. | Ava for deep-dive |
| "We want to start smaller" | Map to Starter tier ($0/mo). Conversion path is Starter → Professional, not pilot → nothing. Start with one workflow, expand later. | Blake for tier options |
| "Need internal approval" | Share `templates/internal-justification.md` — pre-built business case with ROI calc, risk assessment, competitive comparison, and what-happens-if-we-do-nothing. Offer to present to their leadership. | Blake for exec-to-exec call |
| "Competitor does X" | Acknowledge. Three differentiators: (1) Deterministic, (2) Explainable, (3) Human authority. Most automation tools are fast but opaque. Converge is fast AND auditable. Don't trash competitors. | Blake for positioning |

---

## 5. Production Contract: Transition

### 5.1 What Changes from Pilot to Production

| Aspect | Pilot | Production |
|--------|-------|------------|
| Cost | Free | Annual contract (tiered) |
| SLA | Best-effort, 4hr response (see [SLA Tier Definitions](sla-tier-definitions.md) §8) | Contractual SLA per tier: Professional = next business day, Enterprise = 4-hour response (see [SLA Tier Definitions](sla-tier-definitions.md) §2) |
| Data retention | 90 days post-pilot | Per production policy |
| Infrastructure | Shared pilot environment | Dedicated or isolated tenant |
| Support | Leo (direct) | Support model (tier-dependent) |
| Telemetry | Full instrumentation | Per customer preference |
| Security | Pilot controls | Full production controls per Ava's policies |

### 5.2 Production Onboarding Checklist

- [ ] Contract signed
- [ ] Production environment provisioned
- [ ] Customer credentials rotated (new production keys, pilot keys revoked)
- [ ] Integration endpoints updated to production
- [ ] Monitoring and alerting configured
- [ ] Support channel established per SLA
- [ ] Customer admin trained on dashboard/reporting
- [ ] First production run verified
- [ ] 30-day check-in scheduled

---

## 6. Expansion: Growing the Account

### 6.1 Expansion Signals

Watch for these in the first 90 days of production:

- Customer asks about additional workflows
- Usage exceeds tier allocation
- New team members requesting access
- Customer mentions related pain points in other departments
- Positive internal champion evangelizing to peers

### 6.2 Expansion Playbook

1. **Identify opportunity** — Leo notes expansion signal in CRM
2. **Propose scope** — Leo maps new workflow to existing Converge capabilities
3. **Commercial proposal** — Blake presents upgrade or add-on pricing
4. **Technical execution** — Repeat pilot-lite (1-2 weeks, focused) for new workflow
5. **Contract amendment** — Add packs or upgrade tier

---

## 7. Funnel Metrics (Leo's Scorecard)

| Metric | Target | Measurement |
|--------|--------|-------------|
| Pilot conversion rate | ≥60% | (contracts signed) / (pilots completed) |
| Time to contract | ≤14 days post-pilot | Median days from pilot end to signature |
| Pilot-to-production time | ≤21 days | Median days from contract to first production run |
| Customer satisfaction (pilot) | ≥4.0/5.0 | Post-pilot survey |
| Expansion rate (year 1) | ≥30% | Customers adding packs or upgrading tier |
| Field intelligence items | ≥3 per pilot | Actionable items fed to engineering |

---

## 8. Templates (To Be Created)

- [x] `templates/pilot-agreement.md` — Lightweight pilot agreement (REF-57: in review — Ava, Priya, Morgan)
- [x] `templates/discovery-brief.md` — One-page discovery summary
- [x] `templates/pilot-weekly-update.md` — Weekly status email (internal + support tracking)
- [x] `templates/pilot-results-report.md` — Final metrics report (before/after with weekly trends, operational impact, ROI)
- [x] `templates/internal-justification.md` — Template for customer's internal approval
- [x] `templates/production-onboarding-checklist.md` — Pilot-to-production transition checklist (6 phases)

---

## 9. Open Questions

1. ~~**Pricing tiers**~~: **Resolved.** Blake provided draft tiers (§4.3 above). Pending Kenneth's final approval.
2. **Production infrastructure**: What does "dedicated tenant" look like before Wave 4 (converge-runtime)? Need Ren's input.
3. ~~**SLA tiers**~~: **Resolved.** See [SLA Tier Definitions v1.1](sla-tier-definitions.md) — approved by Ren.
4. **Legal review**: Pilot agreement template needs legal review before first use. Do we have outside counsel?
5. **CRM tooling**: Manual tracking for first 3 partners, revisit at customer #4 (per Ren's guidance).
