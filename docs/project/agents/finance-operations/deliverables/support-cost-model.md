# Support & Onboarding Cost Model

> What does it cost to support and onboard customers at each tier? Where does the model break?

**Owner:** Priya Chandran, Finance & Operations
**Status:** Draft v1.0
**Created:** 2026-03-12
**Input from:** Leo Marin (solutions engineering hours), Blake Harmon (customer expectations)
**Related:** `plans/FINANCIAL_MODEL.md`, `agents/finance-operations/deliverables/pilot-commercial-terms.md`, `agents/solutions-engineer/deliverables/pilot-to-contract-playbook.md`

---

## 1. Support Cost Per Customer Per Tier

### 1.1 Support Model by Tier

| Tier | Support Channel | Response SLA | Staffing Model |
|------|----------------|-------------|----------------|
| **Starter ($0)** | Community (docs, FAQ, forum) | None | Self-serve; no dedicated staff |
| **Professional ($499/mo)** | Email + async chat | P1: 24hr, P2: 48hr | Shared queue — Leo Marin (initially) |
| **Enterprise (Custom)** | Dedicated CSM + Slack/Teams | P1: 4hr, P2: 8hr | Named contact — Leo + escalation to engineering |

### 1.2 Cost Per Customer Per Month

| Cost Component | Starter | Professional | Enterprise |
|----------------|---------|-------------|------------|
| Support labor | $0 | $75-150/mo | $500-1,500/mo |
| Tooling (per-seat) | $0 | $5-10/mo | $10-20/mo |
| Escalation engineering time | $0 | $25-50/mo (avg) | $100-300/mo (avg) |
| Documentation maintenance | Shared across all tiers | — | — |
| **Total support cost** | **~$0** | **$105-210/mo** | **$610-1,820/mo** |

**Assumptions and uncertainty:**

- **Professional support labor ($75-150/mo):** Assumes 1.5-3 hours/month per customer at $50/hr imputed cost. Based on Leo's estimate from pilot playbook (4hr SLA during business hours, ~2 tickets/week average). **Uncertainty: MEDIUM** — no production data yet.
- **Enterprise support labor ($500-1,500/mo):** Assumes 10-30 hours/month per customer. Includes proactive monitoring, quarterly business reviews, and custom reporting. Range is wide because Enterprise scope varies significantly. **Uncertainty: HIGH.**
- **Escalation engineering time:** Professional customers occasionally need engineering help for integration issues or platform bugs. Estimated at 0.5-1 hr/mo average. Enterprise customers will have more complex integrations. **Uncertainty: MEDIUM.**

---

## 2. Onboarding Cost Per Customer

### 2.1 Onboarding Effort by Stage

| Activity | Starter | Professional | Enterprise |
|----------|---------|-------------|------------|
| Account setup | Self-serve (0 hr) | Guided (0.5 hr) | White-glove (2 hr) |
| Kickoff call | None | 30 min | 2 hr (multi-stakeholder) |
| Integration setup | Self-serve docs | Leo assists (2-4 hr) | Leo + Eli/Kira (8-20 hr) |
| Workflow configuration | Self-serve | Assisted (1-2 hr) | Custom build (4-10 hr) |
| Training | Docs + video | 1-hr session | Multi-session (3-5 hr) |
| Baseline data collection | N/A | Sam assists (2 hr) | Sam + customer IT (4-8 hr) |
| Go-live verification | N/A | Leo verifies (1 hr) | Leo + Dex (2-4 hr) |
| **Total effort** | **~0 hr** | **7-10 hr** | **25-51 hr** |

### 2.2 One-Time Onboarding Cost

| | Starter | Professional | Enterprise |
|--|---------|-------------|------------|
| Leo Marin (Solutions Eng) | $0 | $250-400 | $750-1,500 |
| Engineering support (Eli/Kira) | $0 | $0-100 | $400-1,000 |
| Sam Okafor (metrics setup) | $0 | $100 | $200-400 |
| Dex Tanaka (infra/deployment) | $0 | $0 | $100-200 |
| **Total onboarding cost** | **$0** | **$350-600** | **$1,450-3,100** |

Imputed cost: $50/hr for solutions engineering, $75/hr for senior engineering.

### 2.3 Onboarding Cost Recovery

| Tier | Onboarding Cost | Monthly Net Contribution | Months to Recover |
|------|----------------|-------------------------|-------------------|
| Starter | $0 | -$35 (free tier) | N/A |
| Professional | $350-600 | $200-250 | **1.5-3 months** |
| Enterprise | $1,450-3,100 | $700+ | **2-4.5 months** |

Onboarding cost is acceptable for both paid tiers — recovered well within the first contract term.

---

## 3. Scalability Analysis

### 3.1 Current Capacity

Leo Marin is the only Solutions Engineer. His available hours:

| Activity | Hours/mo |
|----------|----------|
| Total available (full-time agent) | ~160 hr |
| Pre-sales & qualification | -20 hr |
| Internal coordination | -10 hr |
| **Available for support + onboarding** | **~130 hr/mo** |

### 3.2 Capacity per Tier

| Tier | Support hrs/mo/customer | Onboarding hrs (one-time) | Max customers Leo can support solo |
|------|------------------------|--------------------------|-----------------------------------|
| Professional | 1.5-3 hr | 7-10 hr | **40-85** (steady state, no onboarding) |
| Enterprise | 10-30 hr | 25-51 hr | **4-13** (steady state, no onboarding) |

### 3.3 Breaking Points

**Scenario A: 3 Pro + 1 Enterprise (base case, Month 3)**
- Monthly support: (3 × 2.25 hr) + (1 × 20 hr) = **26.75 hr/mo**
- Leo utilization for support: **21%** — plenty of headroom
- **Verdict: No problem.** Leo handles this comfortably.

**Scenario B: 10 Pro + 2 Enterprise (Month 6-9 target)**
- Monthly support: (10 × 2.25 hr) + (2 × 20 hr) = **62.5 hr/mo**
- Plus onboarding new customers: ~20-30 hr/mo during ramp
- Leo utilization: **65-70%** — approaching strain
- **Verdict: Manageable but Leo can't take on new pre-sales work.** Consider hiring or tooling investment.

**Scenario C: 20 Pro + 5 Enterprise (Month 12+ target)**
- Monthly support: (20 × 2.25 hr) + (5 × 20 hr) = **145 hr/mo**
- **Verdict: Exceeds Leo's capacity.** Need 2nd Solutions Engineer or tiered support model.

### 3.4 Capacity Thresholds

| Threshold | Customer Mix | Action Required |
|-----------|-------------|-----------------|
| **Green (0-30 hr/mo)** | ≤5 Pro + 1 Ent | Leo handles everything |
| **Yellow (30-80 hr/mo)** | ≤10 Pro + 2 Ent | Invest in support tooling, reduce manual work |
| **Red (80+ hr/mo)** | >10 Pro + 3 Ent | Hire 2nd Solutions Engineer or outsource Tier 1 |

---

## 4. Build vs Buy: Support Tooling

### 4.1 Current State (MVP)

- No ticketing system
- No CRM (per Leo's open question)
- Support is ad-hoc: email, Slack, direct access to Leo
- This works for 0-4 customers. It does not scale.

### 4.2 Options

| Option | Monthly Cost | Setup Time | Best For |
|--------|-------------|------------|----------|
| **A: Stay manual** | $0 | 0 | ≤4 customers (current state) |
| **B: Lightweight SaaS** (Linear, Intercom, or Crisp) | $30-100/mo | 1-2 days | 5-15 customers |
| **C: Full platform** (Zendesk, Freshdesk) | $150-500/mo | 1-2 weeks | 15+ customers |
| **D: Build custom** on converge-experience | $0 (eng time) | 4-8 weeks | 20+ customers (eventually) |

### 4.3 Recommendation

**Phase 1 (Now → 5 customers): Option A — Stay manual.**
No tooling investment needed. Leo uses email + a shared spreadsheet tracker. Cost: $0.

**Phase 2 (5-15 customers): Option B — Lightweight SaaS.**
Move to a ticketing system with basic automation. Recommendation: **Linear** (already used for engineering via Paperclip — reduces context switching). Cost: ~$50/mo. Trigger: when Leo's support load exceeds 30 hr/mo.

**Phase 3 (15+ customers): Option C — Full platform.**
When we need knowledge base, SLA tracking, customer portal. Cost: ~$200-300/mo. Trigger: when we hire 2nd support person.

**Option D (Build custom) is not recommended** in the next 12 months. Engineering capacity is needed for the core product. Support tooling is a solved problem — buy, don't build.

---

## 5. Impact on Gross Margin Per Tier

### 5.1 Updated Unit Economics (with support costs)

Using updated pricing ($499/mo Professional, 2,500 runs/mo):

| Component | Starter | Professional | Enterprise ($1,500/mo) |
|-----------|---------|-------------|------------------------|
| Revenue | $0 | $499 | $1,500 |
| LLM API costs | $15-40 | $75-200 | Variable |
| Compute/hosting | $5-10 | $20-50 | $50-200 |
| Support (monthly) | $0 | $105-210 | $610-1,820 |
| Tooling share | $0 | $5-10 | $10-20 |
| **Total COGS** | **$20-50** | **$205-470** | **$670-2,040** |
| **Gross Margin** | **-$20 to -$50** | **$29-294** | **-$540 to +$830** |
| **GM%** | N/A | **6-59%** | **-36% to +55%** |

### 5.2 Midpoint Gross Margin (most likely)

| Tier | Revenue | COGS (midpoint) | Gross Margin | GM% |
|------|---------|-----------------|-------------|-----|
| Starter | $0 | $35 | -$35 | N/A |
| Professional | $499 | $340 | **$159** | **32%** |
| Enterprise ($1,500) | $1,500 | $1,100 | **$400** | **27%** |
| Enterprise ($2,500) | $2,500 | $1,100 | **$1,400** | **56%** |

### 5.3 Key Finding

**Support costs are the second-largest COGS component after LLM API costs.** At Professional tier, support is $105-210/mo vs LLM at $75-200/mo — they're roughly equal.

**Enterprise gross margin is highly sensitive to contract value.** At the $1,500/mo floor, Enterprise can be margin-negative if support demands are high. **Recommendation: raise Enterprise floor to $2,000/mo** or scope support hours in the contract.

This finding should feed back into `plans/FINANCIAL_MODEL.md` (Section 2).

---

## 6. Onboarding Amortization by Contract Length

| Contract | Onboarding Cost | Monthly Amortization | Impact on GM% |
|----------|----------------|---------------------|---------------|
| Pro (month-to-month) | $475 | $475 Month 1, then $0 | -95% Month 1, then 32% |
| Pro (annual) | $475 | $40/mo | Reduces GM% by 8pp → **24%** |
| Enterprise (annual) | $2,275 | $190/mo | Reduces GM% by 8-13pp |

**Implication:** Month-to-month Professional customers are unprofitable in Month 1. Annual commitments spread the cost and should be strongly encouraged via the 15% annual discount (per pilot commercial terms).

---

## 7. Recommendations Summary

| # | Recommendation | Priority | Financial Impact |
|---|---------------|----------|------------------|
| 1 | **Stay manual until 5 customers** — no tooling spend needed now | Immediate | Saves $50-500/mo |
| 2 | **Encourage annual contracts** — 15% discount pays for itself via onboarding amortization | Immediate | Improves Month 1 GM by ~$400/customer |
| 3 | **Raise Enterprise floor to $2,000/mo** — $1,500/mo is margin-risky with dedicated support | High | +$500/mo per Enterprise customer |
| 4 | **Invest in ticketing at 5 customers** — Linear or equivalent, ~$50/mo | When needed | Prevents support chaos |
| 5 | **Plan for 2nd Solutions Engineer at 10 Pro + 3 Ent** | Forward-looking | ~$3,000-5,000/mo (agent or human) |
| 6 | **Scope Enterprise support hours in contract** — prevent unlimited support drain | High | Caps worst-case COGS |

---

## 8. Open Questions

1. **CRM decision:** Leo flagged this in his playbook. Support cost model depends on tooling choice for tracking tickets and time.
2. **Enterprise support scope:** Should we offer unlimited support or cap at X hours/month with overage billing?
3. **Self-serve investment:** How much should we invest in docs, FAQ, and tutorials to deflect Professional support tickets?
4. **2nd SE hiring:** Agent or human? Agent is cheaper ($30-50/mo) but may not handle relationship-heavy Enterprise accounts.
5. **Support hours in contract:** Do we include specific support hour allocations in Enterprise MSA?

---

## Appendix: Data Sources

- Support SLAs: `agents/finance-operations/deliverables/pilot-commercial-terms.md` §4.3
- Onboarding activities: `agents/solutions-engineer/deliverables/pilot-to-contract-playbook.md` §3, §5
- Pilot metrics setup: `plans/PILOT_METRICS_FRAMEWORK.md` §3
- Financial model: `plans/FINANCIAL_MODEL.md` §2
- Imputed hourly costs: Estimated from agent budget allocations; not actual market rates

---

*Draft v1.0. Requires review from Leo Marin (solutions engineering effort validation) and Blake Harmon (customer expectations alignment).*
