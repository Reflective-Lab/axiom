# Converge — Company Strategy

**Author:** Morgan Vale, CEO
**Date:** March 12, 2026
**Status:** Draft for Kenneth's approval
**Audience:** Board (Kenneth Pernyer, Founder)

---

## 1. Mission

Make multi-agent business workflows provably correct.

Converge is a semantic governance platform. It takes messy, multi-step business processes — the ones held together by spreadsheets, Slack threads, and a person who "just knows" — and turns them into convergent, auditable, deterministic pipelines. Every decision has a trace. Every outcome can be replayed. Every policy is enforced, not hoped for.

## 2. Vision

Within 18 months, Converge is the default way mid-market companies orchestrate complex workflows across teams and tools. "Did it converge?" replaces "Did anyone check?"

## 3. Target Market and ICP

### Who we sell to first

**Beachhead:** SMB operators (10-100 employees) drowning in manual handoffs between SaaS tools. They are the human glue between CRM, billing, and delivery. They feel dropped leads, slow quotes, and mismatched invoices daily — but lack the engineering team to fix it.

**Why them:**
- Acute, quantifiable pain (measurable before/after)
- Short sales cycles (founder/owner decides)
- Low integration complexity (HubSpot + Stripe + Slack)
- Their success stories translate directly to the next market tier

### Design partner profiles (priority order)

| Profile | Industry | Size | Pilot Workflow | Decision Maker |
|---------|----------|------|----------------|----------------|
| **A: Scaling Services Firm** | B2B professional services | 15-60 employees | Lead-to-Cash | Founder/CEO or VP Ops |
| **B: B2B SaaS with Messy RevOps** | B2B SaaS (Series A-B) | 20-80 employees | Lead-to-Cash | VP RevOps / Head of Sales Ops |
| **C: Engineering Team with Release Pain** | Dev tools, infrastructure | 20-150 engineers | Release Readiness | VP Engineering |
| **D: Multi-Location Operator** (stretch) | Franchise, retail | 3-20 locations | Money Pack | Owner / Director of Ops |

### Who we don't sell to yet

- Enterprise (500+): too slow, needs SSO/SOC 2 Type II we don't have
- Solopreneurs (<5): not enough workflow complexity for ROI
- Companies without SaaS tools: no integration surface
- "AI curious" buyers who want to experiment, not solve a problem

## 4. Business Model

### Pricing tiers

| | Starter | Professional | Enterprise |
|---|---------|-------------|------------|
| **Price** | $0/mo | $349/mo ($279/mo annual) | Custom |
| **Target** | Founders validating first workflow | Teams of 5-50 with pipeline pain | 50-500 employees, compliance needs |
| **Workspaces** | 1 | 3 | Unlimited |
| **Packs** | 2 (Money + Customers) | 4 (+ Pricing + Delivery) | All + custom |
| **Runs/month** | 500 | 5,000 | Unlimited |
| **Support** | Community | Email (next biz day) | Dedicated CSM |

### Usage-based levers

- Additional runs: $0.02/run (declining at scale)
- Additional workspaces: $99/month each
- Add-on packs: $149-$249/month
- Provider pass-through (LLM, payments): cost + 10%

### Revenue projections (conservative)

| Quarter | Paying Customers | MRR |
|---------|-----------------|-----|
| Q1 2026 | 3 (from design partners) | $1,047 |
| Q2 2026 | 5 | $1,745 |
| Q3 2026 | 8-9 | $2,800-$3,150 |
| Q4 2026 | 10-12 | $4,000-$6,000+ |

**Year 1 target: $40K-$60K ARR.** This is a validation play, not a revenue play. The goal is 3-4 referenceable customers with published case studies.

### Open pricing questions for Kenneth

1. Is $349/mo the right Professional anchor? Could feel high for early SMBs. Alternative: $249/mo with 3K runs.
2. Should Starter include the Pricing pack to drive engagement, or hold it back to drive upgrades?
3. Is cost + 10% enough margin on LLM pass-through? Should we absorb provider costs in the tier price?

## 5. Competitive Positioning

### What we do that no one else does

**Convergence.** Other platforms orchestrate agents linearly (A → B → C). Converge runs agents in parallel cycles until they reach a mathematically fixed point — like a compiler resolving types, not a conveyor belt moving parts. This gives us three things competitors can't match:

1. **Determinism:** Same inputs produce same converged outputs, even with LLM nondeterminism in individual steps. Outcomes are replayable and auditable.
2. **Invariant enforcement:** Business rules (Cedar policies) are checked every cycle, not just at the end. Violations are caught mid-flight, not in postmortems.
3. **Explainability:** Every fact in the converged state has a provenance chain back to the agent that proposed it and the policy that approved it. No black boxes.

### Competitive landscape

| Category | Examples | Converge Difference |
|----------|----------|-------------------|
| Workflow automation | Zapier, Make, n8n | Linear triggers, no convergence, no governance |
| AI agent frameworks | LangChain, CrewAI, AutoGen | Agent orchestration without correctness guarantees |
| Process mining | Celonis, UiPath | Observes existing processes, doesn't replace them |
| BPM/low-code | ServiceNow, Appian | Enterprise-heavy, no AI-native architecture |

**Our wedge:** "Provably correct multi-agent workflows." No one else in the market can make that claim with a real engine behind it.

## 6. Product Architecture (Summary)

Converge ships as 15 Rust crates organized in 5 waves:

| Wave | What | Status |
|------|------|--------|
| **Wave 1: Foundation** | converge-core (proof engine), converge-traits (contracts), converge-business (narrative) | Active — traits audited, core examples done |
| **Wave 2: Instantiation** | Providers (LLM, analytics, policy, optimization) | Planned — converge-provider in progress |
| **Wave 3: Tooling** | JTBD compiler, domain modules, experience store | Backlog |
| **Wave 4: Infrastructure** | WASM runtime with dynamic module loading | Backlog |
| **Wave 5: Experience** | gRPC client, reference app, persona evals | Backlog |

**End-to-end milestone:** User writes a JTBD spec → compiler produces WASM → runtime executes convergence → results are traced and auditable.

**Current engineering velocity:** 32 issues completed, 8 in progress, 41 open. Wave 1 foundation work is largely done. Wave 2 is starting.

## 7. Go-to-Market Strategy

### Motion: narrow-then-expand

Land 3-4 paying design partners through a specific workflow wedge (Lead-to-Cash), prove measurable ROI, then use case studies to open the broader market.

### Channels (priority order)

1. **Warm intros from Kenneth's network** (P0) — 2-3 qualified conversations
2. **LinkedIn outreach** (P0) — 3-5 qualified conversations
3. **Community engagement** (P1) — RevGenius, Pavilion
4. **Content-driven inbound** (P1) — blog + podcast (kicks in cw4+)
5. **Cold email** (P2) — Release Readiness targets

### Funnel math

80-120 outreach touches → 12-18 discovery calls → 5-7 pilots → 4-5 successes → **3-4 production contracts by cw12 (June 5, 2026).**

### Pilot structure

- 3-4 weeks, free (Professional tier access)
- Full telemetry instrumented from day 1
- Weekly working sessions with the partner
- Success = improvement on 2+ of: cycle time (-30%), lead response (-50%), manual steps (-40%), automation rate (+20pp), throughput (+25%)
- Anonymized case study published within 5 days of pilot end

### GTM budget

$2,000-$3,000/month. No paid ads until messaging is proven. Detailed allocation in `plans/GTM_PLAN.md`.

## 8. Key Risks and Mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| 1 | **No design partners materialize** | Medium | Critical | Parallel outreach channels. Kenneth's network is primary, LinkedIn is insurance. |
| 2 | **Pilots don't hit success thresholds** | Medium | High | Thresholds are calibration targets, not hard gates. First pilot recalibrates. Extend by 2 weeks if needed. |
| 3 | **Wave 1 delays block pilot runtime** | Low | Critical | Interactive demos already live. First pilots can run against demo infrastructure while runtime hardens. |
| 4 | **Blake is a GTM bottleneck** | High | High | Blake owns pipeline, content, and pilot management. If he stalls, redistribute content to Caroline's team and consider a dedicated SDR hire. |
| 5 | **Pricing is wrong** | Medium | Medium | $349/mo is a starting point. Prepared to drop to $249/mo. Monitor discovery call objections. |
| 6 | **Content doesn't drive inbound** | High (early) | Low (early) | Content is a long game. Outbound drives cw1-cw8. Inbound matters cw8+. |
| 7 | **Security/compliance gaps slow enterprise expansion** | Medium | Medium | SOC 2 Type I readiness underway. Security one-pager done. Policies drafted. |

## 9. Success Metrics — Next 90 Days

| Metric | Target | Owner |
|--------|--------|-------|
| Design partners signed (LOI) | 3-4 | Blake |
| Production contracts signed | 3-4 | Blake |
| First case study published | By cw6 (Apr 26) | Blake + Sam |
| converge-core proof examples complete | Done | Eli |
| converge-provider working (Anthropic + OpenAI) | By cw4 | Kira |
| converge.zone pricing page live | By cw2 (Mar 29) | Jules |
| converge.zone /for/operations live | By cw2 (Mar 29) | Jules |
| Blog posts published | 4+ by cw8 | Caroline's team |
| Monthly marketing spend | <$3,000/mo | Blake |
| ARR at 90 days | $1,047+ (3 × $349) | Blake |

## 10. Capital and Budget Allocation

### Current spend

- Agent compute (Paperclip runs): $113.56 to date
- Marketing budget: $2,000-$3,000/month
- Infrastructure: minimal (demo hosting, domain)
- Legal (one-time): $500-$1,000 (pilot charter review)

### Team allocation

| Team | Focus | % of Capacity |
|------|-------|---------------|
| **Engineering (Ren, Eli, Kira, Jules, Sam, Dex, Ava, Leo)** | Wave 1 completion → Wave 2 start, pilot runtime readiness | 70% product, 20% security, 10% infrastructure |
| **GTM (Blake, Rio)** | Outreach, content, pilot management, website | 100% GTM |
| **Editorial (Caroline, Alice, Bob)** | Blog content, case studies, thought leadership | 100% content (blocked on proof examples for technical content) |
| **Finance (Priya)** | Financial model, unit economics, cost structure | 100% finance |

### Hiring plan

No new hires until we have 2+ paying customers. Current team of 16 agents is sufficient for the validation phase. If Blake becomes an unsustainable bottleneck, the first hire is an SDR/BDR.

## 11. Board Cadence

Proposed (pending Kenneth's approval in GOVERNANCE.md):

- **Weekly async update:** Morgan posts a status summary (metrics, blockers, decisions needed) — Kenneth reviews at his pace
- **Bi-weekly deep dive:** 30-minute sync on strategy, pipeline, and key decisions
- **Milestone reviews:** Kenneth signs off on major milestones (first pilot, first contract, pricing changes, new hires)

### Kenneth's decision rights (proposed)

- Strategy approval (this document)
- Capital allocation changes >$5,000
- New agent hires
- Pricing changes
- External-facing content approval (pricing page, landing page)
- Milestone sign-off

### Morgan's operating authority

- Day-to-day task assignment and prioritization
- Agent workload management
- Content production (within approved messaging)
- Pilot execution decisions
- Bug fixes and technical decisions (delegated to Ren)

---

**This is the document Kenneth reads before saying "go."**

Next steps:
1. Kenneth reviews and approves (or revises) this strategy
2. Kenneth reviews and approves pricing page content
3. Kenneth identifies 3-5 warm intro targets
4. We start cw1 outreach on March 16

*Morgan Vale, CEO — March 12, 2026*
