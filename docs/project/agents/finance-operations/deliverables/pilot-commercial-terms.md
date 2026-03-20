# Pilot Commercial Terms

> The business side of every pilot engagement. Complements the Pilot Metrics Framework (measurement) with the commercial structure (money).

**Owner:** Priya Chandran, Finance & Operations
**Status:** Draft v1.0
**Created:** 2026-03-12
**Approval required:** Morgan Vale (CEO), Kenneth Pernyer (Board)
**Related:** `plans/PILOT_METRICS_FRAMEWORK.md`, `plans/FINANCIAL_MODEL.md`

---

## 1. Pilot Structure

### 1.1 Free Pilot Offer

Every design partner engagement begins with a **free, time-boxed pilot**. This is how we prove value before asking for money.

| Parameter | Value |
|-----------|-------|
| Duration | **4 weeks** (extendable to 6 weeks with CEO approval) |
| Cost to customer | **$0** |
| Workspaces | 1 |
| Domain packs | Up to 2 |
| Runs included | Up to 500/week (2,000 total) |
| Support level | Direct access to Solutions Engineer (Leo Marin) |
| Onboarding | 2-hour kickoff session + async Slack/email support |

### 1.2 What's Included in a Pilot

- Full access to the Converge platform for one workflow
- Baseline measurement (Week 0) per Pilot Metrics Framework
- Weekly check-in calls (15 min)
- Before/after performance report at pilot end
- Anonymized case study draft (with customer consent)

### 1.3 What's NOT Included

- Custom domain pack development (available at Enterprise tier)
- Multi-workflow deployments
- SLA or uptime guarantees
- Production-grade support response times
- Data migration services

---

## 2. Conversion Triggers

### 2.1 Objective Triggers

A pilot customer should be presented with a paid tier proposal when **any 2 of these signals** are observed:

| Signal | Threshold | Measurement |
|--------|-----------|-------------|
| Run volume | Customer uses >75% of pilot run allowance (>1,500 runs) | Platform telemetry |
| Success metrics | 2+ pilot success thresholds met (per Pilot Metrics Framework §8) | Before/after report |
| User expansion | Customer requests additional users or workspaces | Support request |
| Workflow expansion | Customer asks to apply Converge to a second workflow | Support request |
| Stakeholder engagement | Customer's decision-maker attends a check-in or demo | Meeting attendance |

### 2.2 Timing

- **Week 2:** Mid-pilot check-in. If early success signals are strong, introduce pricing context ("here's what it looks like after the pilot").
- **Week 4:** End-of-pilot review. Present before/after results alongside paid tier options.
- **Week 5-6:** Decision window. Customer has 2 weeks to decide. No hard sell — let the data speak.

### 2.3 Conversion Conversation Ownership

| Activity | Owner |
|----------|-------|
| Pilot results presentation | Sam Okafor (data) + Leo Marin (relationship) |
| Pricing discussion | Blake Harmon (proposal) + Priya Chandran (terms) |
| Contract negotiation | Morgan Vale (approval authority) |
| Technical questions | Eli Marsh or Kira Novak |

---

## 3. Paid Tier Activation

### 3.1 Starter → Professional Upgrade Path

The free Starter tier exists permanently for self-serve exploration. Upgrade triggers:

| Trigger | Action |
|---------|--------|
| Run limit hit (500/mo) | Prompt: "You've hit your limit. Upgrade to Professional for 5,000 runs/mo." |
| Workspace request | Professional tier unlocks 3 workspaces |
| Support request | Professional includes email support with 24h response target |

### 3.2 Professional → Enterprise Upgrade Path

| Trigger | Action |
|---------|--------|
| >3 workspaces needed | Enterprise: unlimited workspaces |
| Compliance requirements (SSO, audit, SOC 2) | Enterprise only |
| Custom domain pack request | Enterprise only |
| SLA requirement | Enterprise includes SLA |

### 3.3 Pricing Reference

| Tier | Monthly Price | Annual Price (est.) | Notes |
|------|-------------|--------------------|----|
| Starter | $0 | $0 | Permanent free tier |
| Professional | **$499/mo (recommended)** | $5,090/yr (15% annual discount) | **Finance + PM recommend $499. Pricing page currently shows $349 — must be updated before prospect exposure.** |
| Enterprise | Custom (floor: **$2,000/mo recommended**) | Custom | Raised from $1,500 per support cost model — $1,500 is margin-negative with dedicated support |

**Annual discount:** 15% off monthly price for annual commitment. This improves cash flow predictability and reduces churn risk.

---

## 4. Contract Template Outline

### 4.1 Master Service Agreement (MSA) — Key Clauses

| Clause | Professional | Enterprise |
|--------|-------------|------------|
| **Term** | Month-to-month or annual | Annual (minimum 12 months) |
| **Cancellation** | 30 days written notice | 90 days written notice |
| **Payment terms** | Net 30 | Net 30 (or Net 45 for >$50K ARR) |
| **Auto-renewal** | Yes (with 30-day opt-out) | No (explicit renewal required) |
| **Price lock** | 12 months from sign date | Contract term |
| **Overage pricing** | $0.10/run above included limit | Negotiated |

### 4.2 Data Handling Clause

Per the Pilot Metrics Framework §11 (Data Retention and Disposal Policy):

> Customer data (telemetry, workflow events, baseline measurements) is retained for 90 days following engagement completion, then permanently deleted. Anonymized, aggregated performance data is retained indefinitely for benchmarking. Integration credentials are revoked at engagement end. Customer may request early deletion at any time.

### 4.3 SLA Commitments

| SLA Parameter | Starter | Professional | Enterprise |
|---------------|---------|-------------|------------|
| Uptime target | None | 99.5% | 99.9% |
| Response time (P1) | None | 24 hours | 4 hours |
| Response time (P2) | None | 48 hours | 8 hours |
| Credits for downtime | None | Pro-rata | Negotiated |
| Maintenance windows | Unscheduled OK | 48hr notice | 1 week notice |

### 4.4 Liability & Indemnification

- Standard limitation of liability: capped at 12 months of fees paid
- Mutual indemnification for IP infringement
- No consequential damages
- Customer responsible for data accuracy and compliance with their own regulations

**Note:** These are template terms. Actual contracts will require legal review before execution.

---

## 5. Pilot-to-Paid Timeline

```
Week 0:     Baseline measurement begins
Week 1-4:   Pilot active
Week 4:     Results presentation + pricing discussion
Week 5-6:   Decision window (customer evaluates)
Week 6-7:   Contract negotiation (if converting)
Week 7-8:   Contract signed, first invoice issued
Week 8:     Paid tier activated, pilot workspace migrated
```

**Expected timeline from pilot start to first invoice: 7-8 weeks**
**Expected timeline from first contact to first invoice: 9-12 weeks** (includes 2-4 weeks of pre-pilot sales)

---

## 6. Early Termination

### 6.1 Pilot Termination (by customer)

- Customer may end the pilot at any time, no penalty
- All customer data deleted within 7 days per retention policy
- Integration credentials revoked immediately
- Brief exit survey requested (optional)

### 6.2 Pilot Termination (by Converge)

- If pilot is consuming excessive resources or customer is non-responsive for >2 weeks
- 7-day written notice to customer
- Requires Morgan Vale approval

### 6.3 Paid Tier Termination

- Per MSA cancellation terms (30 days Professional, 90 days Enterprise)
- Pro-rata refund for annual prepayment (minus months used)
- Data handled per retention policy
- 30-day grace period for data export

---

## 7. Financial Controls

### 7.1 Pilot Cost Cap

Each pilot has a maximum cost to Converge:

| Cost Component | Cap | Tracking |
|----------------|-----|----------|
| LLM API calls | $160 (2,000 runs × $0.08 max) | Platform telemetry |
| Solutions engineering time | 10 hours | Leo Marin timesheet |
| Infrastructure | Shared (no incremental cost for pilot) | N/A |
| **Total pilot cost cap** | **$250** | Priya monitors weekly |

If a pilot approaches 80% of its cost cap, Priya escalates to Morgan for approval to continue.

### 7.2 Discount Authority

| Discount Level | Authority |
|----------------|-----------|
| Up to 10% | Blake Harmon (VP Marketing & Sales) |
| 11-25% | Morgan Vale (CEO) |
| >25% | Kenneth Pernyer (Board) |

No free extensions beyond 6 weeks without Board approval.

---

## 8. Open Questions

1. **Annual discount amount:** 15% proposed. Is this too aggressive? Market range is 10-20%.
2. **Enterprise minimum:** $1,500/mo floor. Should we go higher given support costs?
3. **Payment method:** Stripe? Invoice only? Self-serve checkout for Professional?
4. **Legal review:** Who handles contract review? Do we need outside counsel for MSA template?
5. **Pilot insurance:** Should we budget for a pilot that goes badly (refund, remediation)?

---

*Draft v1.0. Requires review from Morgan Vale and Kenneth Pernyer before use with any customer.*
