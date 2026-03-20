# Converge Design Partner Pilot Agreement

> Template for pre-pilot agreements with design partners. Fill in bracketed fields before sending.

**Version:** 1.2
**Created by:** Leo Marin, Solutions Engineer
**Reviewed by:** [Ava Petrov (security), Priya Chandran (commercial), Morgan Vale (CEO)]
**Status:** Draft — requires legal review before first use

---

**Between:**
- **Converge** ("Provider"), represented by [Leo Marin / Blake Harmon]
- **[Customer Company Name]** ("Partner"), represented by [Name, Title]

**Effective Date:** [Date]

---

## 1. Pilot Scope

### 1.1 Workflow(s) in Scope

| # | Workflow | Description | Systems Involved |
|---|----------|-------------|------------------|
| 1 | [e.g., Lead-to-Cash] | [Brief description of the workflow] | [e.g., HubSpot, Stripe, Slack] |

### 1.2 Out of Scope

- Workflows not listed in §1.1
- Custom domain pack development beyond 2 included packs
- Integration complexity beyond standard API connections (additional scoping discussion required)
- Multi-tenant deployment
- Production-grade SLA or uptime guarantees

---

## 2. Duration

| | Date |
|--|------|
| **Pilot start** | [Date] |
| **Baseline measurement period** | [2 weeks prior to start, or specify] |
| **Pilot end** | [Date — typically 4 weeks after start] |
| **Decision window** | [2 weeks after pilot end] |

The pilot may be extended by up to 2 weeks with written agreement from both parties.

---

## 3. Success Criteria

The following metrics will be measured. The pilot is considered successful if **2 or more** thresholds are met:

| # | Metric | Baseline (measured pre-pilot) | Target | Measurement Method |
|---|--------|-------------------------------|--------|-------------------|
| 1 | [e.g., Lead response time] | [To be measured] | [e.g., <4 hours] | [e.g., CRM timestamp delta] |
| 2 | [e.g., Quote generation time] | [To be measured] | [e.g., <1 hour] | [e.g., Pipeline stage timestamps] |
| 3 | [e.g., Manual handoff errors/month] | [To be measured] | [e.g., <2 per month] | [e.g., Error log count] |

Success criteria are agreed before the pilot begins and cannot be changed mid-pilot without written consent from both parties.

---

## 4. What Partner Receives

| Item | Detail |
|------|--------|
| Platform access | Full Converge platform for workflow(s) in §1.1 |
| Observation dashboard | Real-time convergence observation dashboard |
| Workspace | 1 workspace |
| Domain packs | Up to 2 |
| Run allowance | Up to 2,500 runs over the pilot duration |
| Onboarding | 2-hour kickoff session + guided setup |
| Support | Direct access to Solutions Engineer, business hours (9 AM–6 PM ET, Mon–Fri) |
| Check-ins | Weekly 15-minute status call |
| Final report | Before/after performance report with actionable insights |

**Cost to Partner: $0** — Converge absorbs all infrastructure and LLM inference costs during the pilot period.

---

## 5. What Partner Provides

- [ ] Named pilot lead (primary contact for scheduling, feedback, and decisions)
- [ ] Read access to relevant systems for integration (API keys, OAuth consent)
- [ ] Baseline data for pre-pilot measurement period (§2)
- [ ] Availability for weekly check-in (15 min/week) and kickoff session (2 hours)
- [ ] Feedback: honest assessment of what works and what doesn't
- [ ] Timely response to human-in-the-loop (HITL) approval requests — Partner will receive approval requests via [Slack / email] and is expected to respond within [4] business hours. Delayed responses may stall workflow execution.
- [ ] Decision within 2 weeks of pilot end on whether to continue to production

---

## 6. Data Handling

### 6.1 Data Classification

All customer data processed during the pilot is classified as **Restricted** per Converge's Data Classification Policy and handled accordingly:

- Encrypted in transit (TLS 1.2+); encryption at rest will be confirmed by Security Engineer before first pilot deployment
- Access limited to named Converge team members with documented need
- No customer data committed to version control
- No customer data used in marketing without explicit written consent

### 6.2 Data Collected by Converge

| Data Type | Purpose | Retention (if converting) | Retention (if not converting) |
|-----------|---------|---------------------------|-------------------------------|
| Workflow execution logs | Measure pilot success metrics | 90 days after pilot end | Disposed per §6.4 (14 days) |
| Performance metrics (aggregated) | Before/after report | 90 days after pilot end | Disposed per §6.4 (14 days) |
| Integration metadata (no PII) | Troubleshooting and optimization | 90 days after pilot end | Disposed per §6.4 (14 days) |
| Anonymized usage statistics | Product improvement | Indefinite | Indefinite |

> **Note:** For partners who do not convert to production, data disposal follows §6.4 (14 days). The 90-day retention periods above apply only to partners who convert to a paid tier.

### 6.3 Data NOT Collected

- Customer employee PII beyond pilot lead contact info
- Financial transaction details (amounts, account numbers)
- Data from systems not listed in §1.1

### 6.4 Data Disposal

Within 14 days of pilot end (if Partner does not convert to production):
- All customer-specific data deleted from Converge systems
- Deletion confirmation sent to Partner's pilot lead
- Only anonymized, aggregated metrics retained (per §6.2)

Partner may request immediate data deletion at any time by emailing [support@converge.zone].

---

## 7. Intellectual Property

- **Partner's data remains Partner's property.** Converge claims no ownership of Partner's business data.
- **Anonymized results:** Converge may use anonymized, aggregated pilot results (no company name, no identifying details) for product improvement. Case study publication requires separate written consent (see §8).
- **Converge platform IP:** The Converge platform, including all convergence proofs, governance semantics, and domain packs, remains Converge's intellectual property.

---

## 8. Case Study Consent (Optional)

Partner **opts in / opts out** (circle one) to a co-branded case study.

If opted in:
- Converge drafts the case study and shares with Partner for review
- Partner has 10 business days to approve, request changes, or withdraw consent
- No case study is published without Partner's explicit written approval
- Partner may withdraw consent at any time before publication

---

## 9. Confidentiality

Both parties agree to keep confidential:
- Partner: Converge pricing, product roadmap, and technical architecture details shared during the pilot
- Converge: Partner's business data, workflow details, and internal processes

This obligation survives pilot termination for 2 years.

---

## 10. Termination

Either party may terminate the pilot at any time with 5 business days' written notice. Upon termination:
- Converge follows data disposal procedures (§6.4)
- No fees or penalties apply
- Partner retains the final performance report (if pilot ran long enough to generate one)

---

## 11. Post-Pilot Options

| Option | Description |
|--------|-------------|
| **Convert to Production** | Move to a paid tier — Professional ($499/mo) or Enterprise ($2,000/mo+), depending on scope. Detailed pricing presented during decision window. |
| **Extend pilot** | Up to 2 additional weeks (one time only), with specific goals for the extension. |
| **Part ways** | No commitment, no fees. Data disposed per §6.4. |

Converge can provide an internal justification template to help the pilot lead secure production budget approval within their organization.

---

## 12. Limitations

- This pilot agreement is not a production service contract
- Converge provides no uptime SLA, availability guarantee, or support SLA during the pilot
- Support response target is **best-effort, within 4 business hours** (not contractual)
- The pilot environment uses an in-memory data store and may differ from production infrastructure. Production environments use persistent, replicated storage.

---

## 13. Signatures

**Converge:**

Name: ____________________________
Title: ____________________________
Date: ____________________________
Signature: ________________________

**Partner:**

Name: ____________________________
Title: ____________________________
Company: _________________________
Date: ____________________________
Signature: ________________________

---

*This is a lightweight pilot agreement, not a full commercial contract. Production engagements are governed by a separate Master Service Agreement.*
