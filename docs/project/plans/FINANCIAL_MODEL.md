# Converge Financial Model

> Numbers first, narrative second. Label uncertainty. Update weekly.

**Owner:** Priya Chandran, Finance & Operations
**Status:** Draft v1.2 (added Vanta/SOC 2 cost scenario, pricing discrepancy flag)
**Created:** 2026-03-12
**Review:** Morgan Vale (CEO), Kenneth Pernyer (Board)

---

## 1. Cost Structure

### 1.1 Agent Compute (Internal — Paperclip)

Current monthly budget allocation for the 16-agent team:

| Category | Agents | Monthly Budget | Mar Spend (MTD) | Notes |
|----------|--------|---------------|-----------------|-------|
| Leadership | Morgan, Nadia, Priya | $150.00 | $3.48 | Low utilization — normal for oversight roles |
| Engineering | Ren, Eli, Kira, Jules, Sam, Dex, Ava, Leo | $290.00 | $82.23 | Highest spend category, expected |
| Marketing & Content | Blake, Rio, Caroline, Alice, Bob | $130.00 | $26.55 | Ramping as GTM work accelerates |
| **Total** | **16 agents** | **$590.00** | **$112.26** | **19.0% utilization as of 2026-03-12** |

**Projection:** At current burn rate (~$112/mo midpoint), monthly agent compute will stabilize around **$400-600/mo** as all agents hit steady state.

**Uncertainty: MEDIUM** — Agent utilization is still ramping. Actual steady-state spend is an estimate.

### 1.2 Infrastructure Costs (Estimated)

| Item | Monthly Cost | Confidence | Notes |
|------|-------------|------------|-------|
| Cloud hosting (pilot workloads) | $200-500 | Low | Depends on customer volume; WASM runtime not yet deployed |
| CI/CD (GitHub Actions) | $50-100 | Medium | Based on current repo activity |
| Domain & DNS (converge.zone) | $15 | High | Known cost |
| Monitoring / observability | $0-100 | Low | Not yet instrumented; free tier initially |
| **Subtotal** | **$265-715** | | |

### 1.3 LLM API Costs Per Customer Run

This is the key variable cost. Estimated per-run cost based on converge-provider architecture:

| Provider | Est. Cost Per Run | Tokens/Run (est.) | Notes |
|----------|------------------|-------------------|-------|
| Anthropic Claude | $0.02-0.08 | 2K-8K tokens | Depends on agent count and convergence cycles |
| OpenAI GPT-4 | $0.03-0.10 | 2K-8K tokens | Similar range, slightly higher pricing |
| Blended average | **$0.03-0.08** | | Assuming multi-provider mix |

**Uncertainty: HIGH** — No production run data yet. Token estimates based on demo workloads. Actual costs depend on convergence cycle count (typically 3-8 cycles per run) and agent count per workspace.

### 1.4 Monthly Fixed Cost Summary

| Category | Low Estimate | High Estimate |
|----------|-------------|---------------|
| Agent compute (Paperclip) | $400 | $600 |
| Infrastructure | $265 | $715 |
| Tooling & SaaS | $50 | $150 |
| **Total Fixed (without Vanta)** | **$715** | **$1,465** |

**Midpoint estimate: ~$1,100/mo** in fixed costs before any customer revenue.

### 1.5 Conditional: SOC 2 Tooling (Vanta) — Pending Approval (REF-47)

If approved (Finance recommendation: approve at $15K/yr tier):

| Category | Low Estimate | High Estimate |
|----------|-------------|---------------|
| All above | $715 | $1,465 |
| Vanta (compliance automation) | $1,250 | $2,083 |
| **Total Fixed (with Vanta)** | **$1,965** | **$3,548** |

**Midpoint with Vanta: ~$2,350/mo.** This roughly doubles fixed costs but is required for Enterprise sales (SOC 2 is table stakes for mid-market B2B). See REF-47 for full analysis.

**Impact on break-even:** Professional-only scenario shifts from 5-6 customers to 8-10. Mixed scenario (2 Pro + 1 Ent) remains at 3-4 customers — Enterprise contribution absorbs the Vanta cost.

**Uncertainty: LOW** — Vanta pricing is published. Decision is binary (approve/reject).

---

## 2. Pricing Tiers & Unit Economics

### 2.1 Current Pricing (from design spec, pending Morgan/Blake approval)

| Tier | Price | Workspaces | Domain Packs | Runs/mo | Support |
|------|-------|-----------|-------------|---------|---------|
| **Starter** | $0/mo | 1 | 2 | 500 | Community |
| **Professional** | $349/mo | 3 | 4 | 5,000 | Email |
| **Enterprise** | Custom | Unlimited | Custom | Unlimited | Dedicated |

### 2.2 Cost Per Customer Per Tier

| Cost Component | Starter | Professional | Enterprise |
|----------------|---------|-------------|------------|
| LLM API (runs) | $15-40/mo (500 runs × $0.03-0.08) | $150-400/mo (5K runs × $0.03-0.08) | Variable |
| Compute/hosting share | $5-10/mo | $20-50/mo | $50-200/mo |
| Support labor | $0 | $50-100/mo (est. 1-2 hrs/mo) | $500-2,000/mo |
| **Total COGS** | **$20-50/mo** | **$220-550/mo** | **Negotiated** |

### 2.3 Gross Margin Analysis

| Tier | Revenue | COGS (midpoint) | Gross Margin | GM% |
|------|---------|-----------------|-------------|-----|
| **Starter** | $0 | $35 | -$35 | N/A (free) |
| **Professional** | $349 | $385 | **-$36** | **-10%** |
| **Enterprise** | $1,500+ (target) | $800 | $700+ | 47%+ |

### 2.4 Pricing Risk Flag

**The Professional tier at $349/mo is margin-negative at 5,000 runs/month** if LLM costs are at the higher end ($0.08/run). Two paths to fix:

1. **Reduce runs included** — 2,500 runs/mo instead of 5,000, with overage pricing
2. **Raise price** — $499/mo brings Professional to ~15% gross margin at midpoint COGS
3. **Optimize LLM costs** — Caching, smaller models for simple convergence steps, reducing average cost/run to $0.03

**Recommendation (updated per PM review):** Raise Professional to **$499/mo** AND reduce included runs to **2,500/mo** with $0.10/run overage. Rationale:
- Pricing correctly from day 1 avoids trust erosion if we raise prices after pilots convert (Nadia's point)
- Design partners getting early access at $499 is a fair deal — offer a **3-month locked-rate incentive** to sweeten conversion
- Keep Starter at $0 but **gate it** (require signup and approval) to capture leads without open-ended free usage
- Both levers together achieve ~25-30% gross margin at midpoint COGS

---

## 2.5 Pilot-Specific Costs (NEW — per PM review)

The cost-per-customer analysis above models steady-state. The first 3-4 design partner pilots have additional one-time costs:

| Cost Component | Per Pilot | Assumption | Confidence |
|----------------|-----------|------------|------------|
| Leo Marin (Solutions Engineering) setup time | $500-1,000 | 10-20 hours × $50/hr imputed cost | Medium |
| Custom integration work (per MVP-3) | $200-500 | 4-10 hours of Eli/Kira time | Low |
| Dedicated onboarding (2hr kickoff + async) | $100-200 | Per pilot commercial terms | High |
| **Total pilot-specific cost** | **$800-1,700** | | |

**Impact:** For the first 3-4 pilots, add **$800-1,700 per pilot** in one-time costs. At 4 pilots, this is $3,200-6,800 in incremental spend during Month 1-2. This is acceptable as customer acquisition cost (CAC) if pilots convert.

**CAC payback:** At $499/mo Professional with ~$200/mo net contribution, each converting pilot pays back its setup cost in **4-9 months**. Enterprise deals pay back in **1-2 months**.

---

## 3. Break-Even Analysis

### 3.1 Monthly Fixed Costs to Cover

| Cost | Amount |
|------|--------|
| Agent compute (Paperclip) | $500 |
| Infrastructure | $490 |
| Tooling & SaaS | $100 |
| **Total fixed** | **$1,090** |

### 3.2 Break-Even by Scenario

Assumes Professional tier at **$499/mo with 2,500 runs** (recommended pricing):
- Net contribution per Professional customer: ~$200-250/mo (after COGS)
- Net contribution per Enterprise customer: ~$700+/mo

| Scenario | Mix | Net Contribution/mo | Customers to Break Even |
|----------|-----|---------------------|------------------------|
| Professional only | All Pro @ $499 | $200-250/customer | **5-6 customers** |
| Mixed (recommended) | 2 Pro + 1 Ent | $1,100-1,200/mo | **3 customers** |
| Enterprise-heavy | 1 Pro + 2 Ent | $1,600+/mo | **3 customers** |

**Key insight:** We need at least 1 Enterprise deal among the 3-4 design partners to break even in 90 days. Per Nadia's recommendation: target at least one Enterprise-scale candidate in the design partner pipeline.

### 3.3 Break-Even with Vanta (Conditional — if REF-47 approved)

Total fixed with Vanta: ~$2,350/mo

| Scenario | Mix | Net Contribution/mo | Customers to Break Even |
|----------|-----|---------------------|------------------------|
| Professional only | All Pro @ $499 | $200-250/customer | **10-12 customers** |
| Mixed (recommended) | 2 Pro + 1 Ent | $1,100-1,200/mo | **3-4 customers** |
| Enterprise-heavy | 1 Pro + 2 Ent | $1,600+/mo | **3 customers** |

**Key insight:** Vanta cost is absorbed by Enterprise deals but makes a Professional-only path much harder. This reinforces the need to target Enterprise customers — SOC 2 is both a cost and a sales enabler for the same segment.

---

## 4. Revenue Projections (90 Days)

### 4.1 Assumptions

| Assumption | Value | Confidence |
|------------|-------|------------|
| Pilot duration | 4 weeks | High (defined in Pilot Metrics Framework) |
| Pilot-to-paid conversion rate | 50-75% | Low (no data yet) |
| Time from pilot end to first invoice | 2 weeks | Medium |
| Design partner targets | 3-4 companies | High (company goal) |
| Enterprise deal close time | 6-8 weeks from first contact | Low |

### 4.2 Scenario Modeling

**Conservative (2 converting partners):**

| Month | Pilots Active | Paying Customers | MRR |
|-------|--------------|-----------------|-----|
| Month 1 (Apr) | 2-3 | 0 | $0 |
| Month 2 (May) | 1-2 | 1 Pro | $499 |
| Month 3 (Jun) | 0-1 | 2 Pro | $998 |

**Base (3 converting partners, 1 Enterprise):**

| Month | Pilots Active | Paying Customers | MRR |
|-------|--------------|-----------------|-----|
| Month 1 (Apr) | 3-4 | 0 | $0 |
| Month 2 (May) | 1-2 | 2 (1 Pro + 1 Ent) | $1,999 |
| Month 3 (Jun) | 0-1 | 3 (2 Pro + 1 Ent) | $2,498 |

**Optimistic (4 converting partners, 2 Enterprise):**

| Month | Pilots Active | Paying Customers | MRR |
|-------|--------------|-----------------|-----|
| Month 1 (Apr) | 4 | 0 | $0 |
| Month 2 (May) | 1-2 | 3 (1 Pro + 2 Ent) | $3,499 |
| Month 3 (Jun) | 0 | 4 (2 Pro + 2 Ent) | $3,998 |

### 4.3 Revenue Summary (End of 90 Days)

| Scenario | MRR at Month 3 | vs. Monthly Fixed Costs ($1,090) | Status |
|----------|----------------|----------------------------------|--------|
| Conservative | $998 | -$92 | **Near break-even** |
| Base | $2,498 | +$1,408 | **Break-even achieved** |
| Optimistic | $3,998 | +$2,908 | **Profitable** |

---

## 5. Budget Allocation Framework

### 5.1 Current Allocation

| Team | Monthly Budget | % of Total |
|------|---------------|------------|
| Engineering | $290 | 49% |
| Leadership & Ops | $150 | 25% |
| Marketing & Content | $130 | 22% |
| **Total** | **$590** (agent compute only) | |

### 5.2 Governance Rules

1. **Budget owner:** Morgan Vale (CEO) approves all allocation changes. Kenneth (Board) approves total budget increases >20%.
2. **Overspend threshold:** Any agent exceeding 80% of monthly budget triggers an alert to Priya and the agent's manager.
3. **Reallocation authority:** Priya can reallocate up to 10% between teams without CEO approval. Larger shifts require Morgan.
4. **Monthly review:** Priya publishes a budget utilization report on the 1st of each month.
5. **Quarterly reset:** Budget reviewed and adjusted quarterly based on company stage and revenue.

### 5.3 Recommended Adjustments

| Agent | Current Budget | Recommended | Rationale |
|-------|---------------|-------------|-----------|
| Kira Novak | $50.00 | $50.00 | High utilization (44%), core engineering — maintain |
| Eli Marsh | $50.00 | $50.00 | Core engine work — maintain |
| Morgan Vale | $100.00 | $50.00 | 3.5% utilization — reduce until strategy work increases |
| Nadia Reeves | $30.00 | $30.00 | New, monitor utilization |
| Leo Marin | $30.00 | $20.00 | 0% utilization — reduce until pilot customers exist |
| **Net savings** | | **$60.00/mo** | Reallocate to engineering or hold as buffer |

**Note:** These are recommendations, not directives. Morgan approves.

---

## 6. Key Risks

| Risk | Severity | Financial Impact | Mitigation |
|------|----------|-----------------|------------|
| LLM costs higher than estimated | High | +$200-500/mo per customer | Cache convergence results, use smaller models for routine steps |
| Design partners don't convert to paid | High | $0 revenue for 90 days | Tight pilot scoping, clear success criteria, early pricing discussion |
| Professional tier is margin-negative | Medium | -$36/customer/mo at current pricing | Raise to $499 or reduce included runs |
| Infrastructure costs spike at scale | Medium | +$500-2,000/mo | Capacity planning before onboarding Enterprise customers |
| Support costs underestimated | **Medium** | +$100-300/customer/mo | See `agents/finance-operations/deliverables/support-cost-model.md`. Support is ~equal to LLM costs at Professional tier. |
| Enterprise floor too low | Medium | Margin-negative at $1,500/mo with high support | Raise Enterprise floor to $2,000/mo or scope support hours in contract |

---

## 7. Open Questions for Morgan / Kenneth

1. **Pricing decision:** Finance + PM recommend $499/mo with 2,500 runs/mo. Awaiting Morgan/Blake confirmation.
2. **Runway:** What is the total available capital? How many months of burn can we sustain at $0 revenue?
3. **Enterprise pricing floor:** Support cost model suggests $2,000/mo minimum (up from $1,500). At $1,500/mo, Enterprise can be margin-negative with dedicated support.
4. **Headcount plan:** Do we anticipate adding human employees in the next 90 days? This changes the cost structure significantly.
5. **Agent compute budget ceiling:** Is the current $590/mo total appropriate? Should we plan for increases as work accelerates?

---

## Appendix: Data Sources

- Agent budget/spend: Paperclip API (`/api/companies/{id}/agents`), pulled 2026-03-12
- Pricing tiers: `agents/designer/specs/PRICING-PAGE.md` (Rio Castellan)
- LLM cost estimates: Public provider pricing (Anthropic, OpenAI), March 2026
- Pilot structure: `plans/PILOT_METRICS_FRAMEWORK.md` (Sam Okafor)

---

*This model will be updated weekly as new data becomes available. All projections are estimates — see confidence labels per section.*
