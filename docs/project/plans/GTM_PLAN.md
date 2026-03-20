# Marketing & Go-to-Market Execution Plan v2

**Date:** March 12, 2026
**Owner:** Blake Harmon, VP of Marketing & Sales
**Reviewer:** Kenneth Pernyer (Board)
**Status:** Draft for review

---

## Executive Summary

Converge is a semantic governance platform that makes multi-agent business workflows verifiably correct through executable convergence proofs. Our GTM motion is **narrow-then-expand**: land 3-4 paying design partners through a specific workflow wedge (Lead-to-Cash), prove measurable ROI, then use case studies to open the broader market.

This plan covers the full revenue funnel from awareness to expansion, with specific timelines, budgets, and metrics. Everything is sequenced — we don't try to reach everyone at once.

**Goal:** 3-4 signed production contracts by end of Converge Week 12 (June 5, 2026).

---

## 1. Target Customer Profiles

### 1.1 Beachhead Persona: The Ops-Burdened SMB

Our first customers are **SMB operators (10-100 employees) drowning in manual handoffs between SaaS tools.** They are the "human glue" between their CRM, billing, and delivery systems. They feel the pain daily — dropped leads, slow quotes, mismatched invoices — but don't have the engineering team to build automation themselves.

**Why them first:**
- Acute, quantifiable pain (we can measure before/after)
- Short sales cycle (founder/owner decides)
- Low integration complexity (HubSpot + Stripe + Slack)
- High willingness to try new tools (survival mode)
- Their success stories translate directly to the next tier

### 1.2 Target Design Partner Profiles

We need 3-4 design partners. Here are the four profiles we're pursuing, in priority order:

#### Partner Profile A: "The Scaling Services Firm" (Primary)

| Attribute | Detail |
|-----------|--------|
| **Industry** | B2B professional services (consulting, agency, staffing) |
| **Size** | 15-60 employees |
| **Revenue** | $2M-$15M ARR |
| **Tech stack** | HubSpot/Pipedrive + Stripe/QuickBooks + Slack + Google Workspace |
| **Pain** | Founder still manually routes leads, quotes take 2+ days, invoices don't match contracts, no audit trail |
| **Buyer** | Founder/CEO or VP Operations |
| **Decision process** | 1-2 people, 1-2 weeks |
| **Pilot workflow** | Lead-to-Cash (full pipeline) |
| **Example companies** | Regional staffing agencies, boutique consulting firms, B2B marketing agencies |
| **Where they hang out** | LinkedIn, Pavilion/Revenue Collective, local Vistage groups, SaaS-focused Slack communities |

**Why this is Partner A:** Services firms have the highest density of manual handoffs per employee. Every dropped lead or slow quote directly costs revenue. The founder feels it personally.

#### Partner Profile B: "The B2B SaaS with Messy RevOps" (Primary)

| Attribute | Detail |
|-----------|--------|
| **Industry** | B2B SaaS (vertical or horizontal) |
| **Size** | 20-80 employees |
| **Revenue** | $3M-$20M ARR |
| **Tech stack** | Salesforce/HubSpot + Stripe + PandaDoc + Slack |
| **Pain** | Pipeline leakage between stages, no single source of truth for deal status, quote-to-cash takes 5+ handoffs |
| **Buyer** | VP Revenue Operations or Head of Sales Ops |
| **Decision process** | RevOps recommends → VP Sales approves, 2-4 weeks |
| **Pilot workflow** | Lead-to-Cash (with pricing optimization focus) |
| **Example companies** | Series A-B SaaS companies with 500+ deals/quarter |
| **Where they hang out** | RevGenius, Pavilion, Modern Sales Pros, /r/salesops |

**Why this is Partner B:** SaaS RevOps teams already understand pipeline metrics. They can measure ROI themselves. Their success stories resonate across the entire SaaS ecosystem.

#### Partner Profile C: "The Engineering Team with Release Pain" (Secondary)

| Attribute | Detail |
|-----------|--------|
| **Industry** | Developer tools, infrastructure, security |
| **Size** | 20-150 engineers |
| **Revenue** | $5M-$50M ARR |
| **Tech stack** | GitHub + CI/CD + Slack + Jira/Linear |
| **Pain** | Release signoff fragmented across CI, security, perf dashboards. No auditable go/no-go record |
| **Buyer** | VP Engineering or Engineering Manager |
| **Decision process** | Eng Manager tries → VP approves, 2-3 weeks |
| **Pilot workflow** | Release Readiness |
| **Example companies** | See target list in `converge-business/gtm/release-readiness-target-list-and-outreach-script.md` |
| **Where they hang out** | Hacker News, dev tool communities, engineering blogs |

**Why this is Partner C:** Different wedge, same platform. Proves Converge isn't a one-trick pony. Engineering buyers have budget authority and short procurement cycles.

#### Partner Profile D: "The Franchise/Multi-Location Operator" (Stretch)

| Attribute | Detail |
|-----------|--------|
| **Industry** | Franchise, retail, hospitality |
| **Size** | 3-20 locations |
| **Revenue** | $5M-$30M |
| **Tech stack** | Square/Toast + QuickBooks + Google Sheets + email |
| **Pain** | Each location operates differently, no visibility into cross-location performance, reconciliation is manual nightmare |
| **Buyer** | Owner or Director of Operations |
| **Decision process** | Owner decides, 1 week |
| **Pilot workflow** | Money Pack (financial reconciliation across locations) |

**Why this is a stretch:** Lower tech sophistication means more hand-holding. But the pain is extreme and the case study would be highly differentiated.

### 1.3 Anti-Personas (Who We Don't Sell To Yet)

- **Enterprise (500+ employees):** Too slow, too many stakeholders, needs SSO/SOC2 Type II we don't have yet
- **Solopreneurs / micro-businesses (<5 people):** Not enough workflow complexity to show ROI
- **Companies without SaaS tools:** No integration surface for Converge
- **"AI curious" buyers:** They want to experiment, not solve a specific problem. We need problem-aware buyers.

---

## 2. Outreach Strategy

### 2.1 Channel Prioritization

| Channel | Priority | Expected Yield | Timeline |
|---------|----------|---------------|----------|
| **Warm intros (Kenneth's network)** | P0 | 2-3 qualified conversations | cw1-cw2 |
| **Targeted LinkedIn outreach (Blake)** | P0 | 3-5 qualified conversations | cw1-cw4 |
| **Community engagement (RevGenius, Pavilion)** | P1 | 1-2 qualified conversations | cw2-cw6 |
| **Content-driven inbound (blog + podcast)** | P1 | 1-3 leads/month (post-publish) | cw4+ |
| **Cold email (release readiness targets)** | P2 | 1-2 qualified conversations | cw3-cw6 |

### 2.2 Warm Intro Playbook (Highest Priority)

**Kenneth's role:** Make 3-5 warm introductions to founders/ops leaders in his network. Blake handles everything after the intro.

**Intro template for Kenneth:**

> Hey [Name] — I'm working with a company called Converge that's building something I think you'd find interesting. They're looking for 3-4 design partners to pilot a system that automates the handoffs between your CRM, billing, and ops tools — with guaranteed audit trails and no dropped leads. The pilot is free, 3-4 weeks, and they measure everything. Would you be open to a 20-minute call with Blake Harmon, their VP of Marketing & Sales?

**After the intro, Blake runs the discovery call** (script in `converge-business/gtm/release-readiness-target-list-and-outreach-script.md`, adapted for Lead-to-Cash).

### 2.3 LinkedIn Outreach Playbook

**Target:** VP Operations, Head of RevOps, Founder/CEO at companies matching Partner Profiles A and B.

**Sequence:**
1. **Connect request** (no pitch): "Hi [Name] — I saw you're running ops at [Company]. I'm working on something in the ops automation space and would love to connect."
2. **Day 3 — Value message:** Share a relevant content piece (Signals article, Business Voices episode) with a 1-line comment.
3. **Day 7 — Soft ask:** "I'm looking for 3-4 design partners to pilot a system that makes handoffs between CRM/billing/ops tools provably reliable. Free pilot, 3-4 weeks, measured outcomes. Would you be open to a 20-minute look?"
4. **Day 14 — Follow-up or move on.**

**Volume:** 20-30 new connections/week. Expected response rate: 10-15%. Expected booking rate: 15-25% of responses.

### 2.4 Community Engagement

- **RevGenius Slack:** Participate in #revenue-operations, #sales-ops channels. Share insights, not pitches. Offer to do a community talk on "Why your handoffs break and what to do about it."
- **Pavilion (if accessible):** Request speaking slot or AMA. Position: "Converge for revenue ops teams."
- **Indie Hackers / SaaStr community:** For Partner Profile A (founder-operators).

### 2.5 Cold Email (Release Readiness)

Use the existing outreach script and target list in `converge-business/gtm/release-readiness-target-list-and-outreach-script.md`. Kenneth signs the emails. Blake manages the pipeline.

---

## 3. Content Strategy

### 3.1 Content-to-Pipeline Mapping

Every piece of content has a defined role in the sales funnel:

```
AWARENESS               CONSIDERATION              DECISION
(know we exist)         (understand what we do)     (ready to pilot)
────────────────────    ────────────────────────    ─────────────────
Blog: Signals series    Interactive demos           Pricing page
Blog: Business Voices   Business-buyer landing      Pilot charter
LinkedIn posts          Case studies (post-pilot)   Security one-pager
Community talks         Public roadmap              1:1 demo call
                        About Us page
```

### 3.2 Content Calendar (cw1-cw12)

| Week | Content | Owner | Channel | Funnel Stage |
|------|---------|-------|---------|--------------|
| cw1 (Mar 16) | Business Voices Ep. 1 publish | Bob + Caroline | converge.zone/blog | Awareness |
| cw2 (Mar 23) | Signals: ProposedFact trust boundary | Alice + Caroline | converge.zone/blog | Consideration |
| cw3 (Mar 30) | Business Voices Ep. 2 publish | Bob + Caroline | converge.zone/blog | Awareness |
| cw4 (Apr 6) | LinkedIn: "Why your handoffs break" post series (3 posts) | Blake | LinkedIn | Awareness |
| cw5 (Apr 13) | Case study draft (if Pilot 1 completes) | Blake + Sam | converge.zone | Decision |
| cw6 (Apr 20) | Tech Voices Ep. 1 (when converge-core proofs ready) | Alice + Caroline | converge.zone/blog | Consideration |
| cw7 (Apr 27) | LinkedIn: Pilot results teaser (anonymized) | Blake | LinkedIn | Awareness |
| cw8 (May 4) | Business Voices Ep. 3 | Bob + Caroline | converge.zone/blog | Awareness |
| cw9 (May 11) | "Converge for RevOps" landing page copy | Blake + Rio | converge.zone | Consideration |
| cw10 (May 18) | Community talk (RevGenius or Pavilion) | Blake | Community | Awareness |
| cw11 (May 25) | Case study #2 (if Pilot 2 completes) | Blake + Sam | converge.zone | Decision |
| cw12 (Jun 1) | Signals: Architecture deep dive | Alice + Caroline | converge.zone/blog | Consideration |

### 3.3 Content Production Model

- **Caroline Ashford** (Editor-in-Chief) owns editorial quality. Nothing publishes without her review.
- **Alice Mercer** writes technical/systems content (Signals series, Tech Voices).
- **Bob Calder** writes demo-forward, builder content (Business Voices, experiments).
- **Blake** writes GTM copy (landing pages, LinkedIn, email templates, case study narratives).
- **Rio** designs all visual assets.

**Cadence:** One blog post every 1-2 weeks. One LinkedIn post series per month. Case studies as pilots complete.

### 3.4 Blog Structure on converge.zone

| Section | Description | Audience |
|---------|-------------|----------|
| **Backstage** | Internal engineering decisions, architecture notes | Developers, platform builders |
| **Signals** | Technical analysis, system design, trust boundaries | Engineers, technical evaluators |
| **Mechanics** | How Converge works, tutorials, walkthroughs | Developers, ops engineers |
| **Voices** | Business Voices (outcomes), Tech Voices (architecture) | Business buyers, technical leaders |

---

## 4. Pilot Program Details

### 4.1 What We Offer

| Pilot Component | Detail |
|----------------|--------|
| **Duration** | 3-4 weeks (extendable by 2 weeks if mixed results) |
| **Cost** | Free (Professional-tier access) |
| **Packs included** | Money + Customers + Pricing + Delivery (4 packs) |
| **Runs** | Up to 2,500/month |
| **Support** | Named Converge contact (Blake), weekly 30-60 min working session, shared Slack channel |
| **Setup** | 90-minute workshop to map the partner's workflow to Converge concepts (Jobs-to-Be-Done identification, success criteria definition, integration mapping) |
| **Instrumentation** | Full telemetry from day 1 (see Pilot Metrics Framework in `plans/PILOT_METRICS_FRAMEWORK.md`) |

### 4.2 What We Measure

Per the Pilot Metrics Framework, every pilot measures:

- **Cycle time** (end-to-end and per-stage)
- **Lead response time** (from creation to first action)
- **Manual steps eliminated** (count and percentage)
- **Automation rate** (% of steps handled by Converge)
- **Throughput** (items processed per time unit)
- **Output accuracy** (human-rated quality, 1-5 scale)

Baseline data collected for 2 weeks before the pilot starts. Weekly snapshots during pilot. Before/after analysis within 5 days of pilot end.

### 4.3 Success Criteria

Defined at kickoff with each partner. A pilot is a **success** if it demonstrates improvement on at least 2 of:

| Metric | Minimum Improvement |
|--------|-------------------|
| End-to-end cycle time | -30% |
| Lead response time | -50% |
| Manual steps eliminated | -40% |
| Automation rate | +20 percentage points |
| Throughput | +25% |

### 4.4 Partner Commitment

Before the pilot starts, the partner agrees to:

1. Name a pilot owner
2. Run 2-3 real workflow executions (not hypotheticals)
3. Attend weekly 30-60 min working sessions
4. Allow us to measure success metrics
5. Enter production contract conversation if success criteria are met
6. Sign Data Processing Agreement (DPA) if pilot involves PII
7. Pass Converge security pre-flight review (Ava sign-off)

Formalized in the pilot charter + LOI (`converge-business/gtm/release-readiness-design-partner-pilot-charter-loi.md`).

### 4.5 Data Handling

- Raw pilot data retained 90 days post-pilot, then permanently deleted
- Anonymized/aggregated data retained indefinitely for benchmarking
- Integration credentials revoked and deleted at pilot end
- Customer may request early deletion at any time
- Full details in Pilot Metrics Framework §11

---

## 5. Conversion Funnel

### 5.1 Funnel Stages and Expected Timelines

```
Stage                    Timeline        Conversion Rate (target)
─────────────────────    ──────────      ────────────────────────
Outreach / Inbound       cw1-cw4         —
    ↓
Discovery Call           Week 1-2        15-25% of outreach → call
    ↓
Pilot Kickoff            Week 2-3        40-60% of calls → pilot
    ↓
Pilot (3-4 weeks)        Week 3-7        —
    ↓
Success Conversation     Week 7-8        60-80% of pilots → contract
    ↓
Production Contract      Week 8-10       —
    ↓
Expansion (Q2+)          Month 3+        30-50% at 6 months
```

### 5.2 Funnel Math

To get 3-4 production contracts by cw12:

| Stage | Volume Needed | Rationale |
|-------|--------------|-----------|
| Outreach touches | 80-120 | Warm + LinkedIn + cold |
| Discovery calls | 12-18 | 15% booking rate |
| Pilots started | 5-7 | 50% call-to-pilot conversion |
| Successful pilots | 4-5 | 70% pilot success rate |
| Production contracts | 3-4 | 80% success-to-contract conversion |

**This is tight but achievable** if we start outreach in cw1 and run pilots overlapping (not sequential).

### 5.3 Pipeline Tracking

Track in a simple spreadsheet (upgrade to CRM when we have 20+ prospects). **Data classification: Confidential** — store in access-controlled location (not shared drive), share raw pipeline data only with Blake + Kenneth.

| Field | Values |
|-------|--------|
| Company name | — |
| Partner profile (A/B/C/D) | — |
| Source (warm intro, LinkedIn, cold, inbound) | — |
| Stage | Outreach → Discovery → Pilot → Contract → Active |
| Next action | — |
| Next action date | — |
| Pilot start date | — |
| Pilot end date | — |
| Success criteria (Y/N per metric) | — |
| Contract value (annual) | — |
| Owner | Blake |

### 5.4 Conversion Playbook

Full conversation scripts, email templates, and expansion tactics documented in `converge-business/gtm/pilot-to-contract-playbook.md`.

---

## 6. Pricing Recommendation

### 6.1 Three Tiers

| | Starter | Professional | Enterprise |
|---|---------|-------------|------------|
| **Price** | $0/mo | $499/mo ($399/mo annual) | Starting at $2,000/mo |
| **Target** | Founders validating first workflow | Teams of 5-50 with pipeline pain | 50-500 employees, compliance needs |
| **Workspaces** | 1 | 3 | Unlimited |
| **Packs** | 2 (Money + Customers) | 4 (+ Pricing + Delivery) | All + custom |
| **Runs/month** | 500 | 2,500 | Unlimited |
| **Blueprints** | — | 1 (Lead-to-Cash) | All + custom |
| **Trace retention** | 7 days | 90 days | 1 year |
| **Providers** | Standard | Standard + Premium | All + custom |
| **Support** | Community | Email (next biz day) | Dedicated + named CSM |

### 6.2 Usage-Based Levers

- Additional runs beyond tier limit: **$0.02/run** (declining to $0.01/run at 100K+)
- Additional workspaces: **$99/month each**
- Add-on packs: **$149-$249/month** (Trust, People, Sustainability)
- Provider pass-through (LLM, payment processing): **cost + 10%**
- Annual discount on Professional: **20%** ($399/month billed annually)

### 6.3 Revenue Projections (Conservative)

| Quarter | Design Partners | Paying Customers | MRR |
|---------|----------------|-----------------|-----|
| Q1 2026 (cw1-cw12) | 3-4 pilots → 3 contracts | 3 | $1,497 (3 × $499) |
| Q2 2026 | 2 new pilots → 2 contracts | 5 | $2,495 (5 × $499) |
| Q3 2026 | Inbound starts, 3-4 new | 8-9 | $3,992-$4,491 |
| Q4 2026 | First Enterprise deal | 10-12 | $6,000-$10,000+ |

**Year 1 target:** $60K-$90K ARR. This is not a revenue play — it's a validation play. The goal is 3-4 referenceable customers with published case studies.

### 6.4 Open Questions for Kenneth

1. ~~**$349/mo anchor**~~ **Resolved: $499/mo approved per Financial Model review (REF-37). Sustainable at 32% gross margin.**
2. **Starter pack scope:** Should Starter include the Pricing pack to drive engagement, or keep it behind Professional to drive upgrades?
3. **Provider margin:** Is cost + 10% enough margin on LLM pass-through? Should we absorb provider costs in the tier price for simplicity?

---

## 7. Brand and Website Plan

### 7.1 converge.zone Site Map

| Page | Status | Owner | Purpose |
|------|--------|-------|---------|
| **/** (homepage) | Live | Jules | Developer/builder entry point |
| **/pricing** | Built, content pending CEO approval | Jules + Blake + Rio | Pricing tiers + comparison + FAQ |
| **/for/operations** (Business-buyer landing) | Built, content pending CEO approval | Jules + Blake + Rio | Ops/RevOps buyer entry point |
| **/about** | Built | Jules + Blake + Rio | Team, values, hiring signal |
| **/roadmap** | Content drafted, pending CEO approval | Blake + Rio | Public product roadmap |
| **/demo** | Live (interactive) | Jules | Lead-to-Cash + Release Readiness demos |
| **/demo/lead-to-cash** | Live (interactive) | Jules | 8-agent, 9-cycle interactive demo |
| **/blog** (Backstage, Signals, Mechanics, Voices) | Structure live, content in progress | Caroline + editorial team | Thought leadership + SEO |
| **/security** | Content ready (Ava's one-pager) | Blake + Jules | Trust signal for procurement |

### 7.2 Persona Routing

The site needs to serve multiple personas without confusion:

- **Homepage** → Developer/builder entry (current)
- **Nav: "For Business"** → `/for/operations` (ops/RevOps buyer)
- **Nav: "Pricing"** → `/pricing` (all personas)
- **Nav: "About"** → `/about` (all personas)
- **Blog nav:** Backstage + Signals + Mechanics + Voices → each section serves a different audience

**Header nav order** (current): `Backstage | Signals | Mechanics | Voices | Pricing | For Business | About`

### 7.3 Next Website Priorities

1. **CEO approves pricing numbers** → Jules publishes pricing page content
2. **CEO approves landing page** → Jules publishes /for/operations content
3. **CEO approves roadmap** → Jules builds /roadmap page
4. **Blog content starts publishing** (Business Voices Ep. 1 first)
5. **/security page** with Ava's one-pager content

### 7.4 Brand Guidelines

Documented in Rio's design system (`agents/designer/design-system/DESIGN-SYSTEM.md`). Key decisions:

- **Paper/ink palette:** No flashy colors. Professional, precise, engineered.
- **Wine (#722f37) = problem/pain.** Pine (#2d5a3d) = solution/success.
- **No rounded corners, no gradients, no decorative elements.**
- **Copy voice:** Clear over clever. Confident but honest. Persona-aware.
- **IBM Plex Mono** for headings, **Inter** for body, **Georgia** for wordmark.

---

## 8. Timeline (Converge Weeks)

### Phase 1: Launch Outreach (cw1-cw3) — Mar 16 to Apr 4

| cw | Calendar | Action | Owner |
|----|----------|--------|-------|
| cw1 (Mar 16-22) | Kenneth: 3-5 warm intros to Partner Profile A/B contacts | Kenneth + Blake |
| cw1 | Blake: Begin LinkedIn outreach (20-30 connections) | Blake |
| cw1 | Publish Business Voices Ep. 1 | Bob + Caroline |
| cw1 | **CEO reviews and approves pricing page content** | Kenneth |
| cw2 (Mar 23-29) | First discovery calls from warm intros | Blake |
| cw2 | Publish Signals: ProposedFact trust boundary | Alice + Caroline |
| cw2 | LinkedIn outreach continues | Blake |
| cw3 (Mar 30-Apr 4) | Continue discovery calls; pilot agreements signed (kickoff gated on runtime readiness, cw5) | Blake |
| cw3 | Publish Business Voices Ep. 2 | Bob + Caroline |
| cw3 | Cold email to Release Readiness targets begins | Blake (Kenneth signs) |

### Phase 2: Pilot Execution (cw5-cw10) — Apr 13 to May 22

**Hard gate:** No pilot kickoff before runtime readiness (Milestone 4, end of cw5 per Engineering Plan v2). Discovery calls and pilot agreements happen in cw1-cw4; execution starts cw5+.

| cw | Calendar | Action | Owner |
|----|----------|--------|-------|
| cw4 (Apr 6-12) | LinkedIn post series: "Why your handoffs break" | Blake |
| cw5 (Apr 13-19) | **Runtime ready (Milestone 4) — Pilot 1 kickoff** | Ren + Blake + partner |
| cw5-cw8 | Pilot 1 running (3-4 weeks) | Blake + partner |
| cw6 (Apr 20-26) | Tech Voices Ep. 1 (if converge-core proofs ready) | Alice + Caroline |
| cw6-cw9 | Pilot 2 running (overlap with Pilot 1) | Blake + partner |
| cw7 (Apr 27-May 2) | Pilot 1 success conversation | Blake |
| cw7 | Case study #1 draft (if pilot succeeds) | Blake + Sam |
| cw7 | LinkedIn: anonymized pilot results teaser | Blake |
| cw8 (May 4-10) | Publish Business Voices Ep. 3 | Bob + Caroline |
| cw8-cw11 | Pilot 3 running | Blake + partner |
| cw9 (May 11-17) | Pilot 2 success conversation | Blake |

### Phase 3: Convert and Expand (cw8-cw12) — May 15 to Jun 5

| cw | Calendar | Action | Owner |
|----|----------|--------|-------|
| cw8-cw10 | Production contracts for Pilots 1-3 | Blake |
| cw9 (May 11-17) | "Converge for RevOps" landing page | Blake + Rio |
| cw10 (May 18-24) | Community talk (RevGenius or Pavilion) | Blake |
| cw10 | Pilot 4 running (from cold email or inbound) | Blake + partner |
| cw11 (May 25-31) | Case study #2 publish | Blake |
| cw11 | Pilot 4 success conversation | Blake |
| cw12 (Jun 1-5) | 3-4 production contracts signed (target) | Blake |
| cw12 | Publish architecture deep-dive (Signals) | Alice + Caroline |
| cw12 | Retrospective: funnel metrics, what worked, what to change | Blake + Kenneth |

### Key Milestones

| Milestone | Target Date | Dependency |
|-----------|-------------|------------|
| First discovery call | cw2 (Mar 23) | Kenneth warm intros |
| Pricing page live on converge.zone | cw1-cw2 | **CEO approval** |
| Business-buyer landing page live | cw1-cw2 | **CEO approval** |
| Runtime ready (Milestone 4) | cw5 (Apr 13) | **Engineering Plan v2** |
| First pilot started | cw5 (Apr 13) | Runtime ready + signed pilot agreement |
| First case study published | cw7-cw8 (Apr 27-May 10) | Pilot 1 success |
| First production contract signed | cw8-cw10 (May 4-22) | Pilot → contract conversion |
| 3-4 contracts signed | cw12 (Jun 5) | Full funnel execution |

---

## 9. Budget Allocation

### 9.1 Monthly Marketing Budget: $2,000-$3,000/month

This is a lean, founder-stage budget. No paid ads until we have proven messaging.

| Category | Monthly | Purpose |
|----------|---------|---------|
| **LinkedIn Sales Navigator** | $100 | Targeted outreach, InMail credits |
| **Community memberships** | $200 | RevGenius pro, Pavilion (if accessible) |
| **Content production tools** | $150 | Editing tools, design assets, hosting |
| **Pilot partner travel** (if needed) | $300-500 | On-site kickoff for key partners |
| **Demo environment hosting** | $200 | Dedicated pilot infrastructure |
| **LLM costs for demos/pilots** | $300-500 | Provider pass-through during free pilots |
| **Email/outreach tooling** | $100 | Outreach sequencing |
| **Reserve** | $450-$950 | Unforeseen needs, events, content |
| **Total** | **$2,000-$3,000** | — |

### 9.2 One-Time Costs (cw1-cw2 setup)

| Item | Cost | Purpose |
|------|------|---------|
| Domain + hosting for converge.zone | $50-100/year | Already in place |
| Legal review of pilot charter + LOI | $500-$1,000 | One-time |
| Video/podcast editing setup | $200-$500 | For Business Voices / Tech Voices |

### 9.3 Cost Per Design Partner Acquisition

At $2,500/month marketing spend + 1 FTE (Blake) time:

- **Cost to acquire a design partner:** ~$2,500-$5,000 (1-2 months of spend per signed pilot)
- **Cost to convert to production:** ~$0 additional (conversion happens through pilot results)
- **LTV at Professional tier (12-month contract):** $5,988 (annual) or $4,788 (annual discount)
- **Payback period:** 1-2 months after contract start

### 9.4 What We Don't Spend On (Yet)

- Paid search / Google Ads (no proven keywords yet)
- Paid social (LinkedIn ads) (wait for proven organic messaging)
- Conference booths / sponsorships (too expensive for current stage)
- PR agency (no news to announce yet)
- Full-time content writer (Caroline's editorial team covers this)

---

## 10. Metrics: How We Know It's Working

### 10.1 Leading Indicators (Weekly)

| Metric | Target | Tracked By |
|--------|--------|------------|
| New outreach touches | 20-30/week | Blake |
| Discovery calls booked | 2-3/week | Blake |
| LinkedIn connection acceptance rate | 30-40% | Blake |
| Website traffic (converge.zone) | 100+ unique/week by cw6 | Jules (analytics) |
| Blog post engagement (reads, shares) | 50+ reads per post by cw4 | Caroline |

### 10.2 Pipeline Metrics (Bi-Weekly)

| Metric | Target | Tracked By |
|--------|--------|------------|
| Active prospects in pipeline | 10-15 by cw4 | Blake |
| Pilots in progress | 2-3 concurrent by cw5 | Blake |
| Discovery → Pilot conversion rate | 40-60% | Blake |

### 10.3 Outcome Metrics (Monthly)

| Metric | Target | Tracked By |
|--------|--------|------------|
| Pilots completed | 1-2/month starting cw5 | Blake + Sam |
| Pilot success rate | 60-80% | Blake + Sam |
| Production contracts signed | 1/month starting cw6 | Blake |
| MRR | $1,497+ by cw12 | Blake |
| Case studies published | 1 by cw6, 2 by cw12 | Blake |

### 10.4 Health Signals (Things That Tell Us It's Working)

- Partners attend weekly reviews consistently
- Partners use Converge without Converge team present
- Partners introduce us to other potential partners (referrals)
- Inbound inquiries start arriving from blog/community
- Discovery calls go from "what is Converge?" to "how soon can we start?"

### 10.5 Warning Signals (Things That Tell Us to Adjust)

- Discovery calls don't convert to pilots (messaging problem)
- Pilots don't hit success criteria (product problem)
- Successful pilots don't convert to contracts (pricing or positioning problem)
- Blog posts get no engagement (wrong audience or wrong topics)
- Warm intros don't respond (wrong persona or timing)

---

## 11. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Kenneth's warm intros don't yield qualified prospects | Medium | High | Blake runs LinkedIn outreach in parallel from day 1 |
| Pilots take longer than 4 weeks | Medium | Medium | Build 2-week extension into the plan. Start pilots overlapping. |
| Demo not ready for discovery calls | Low | Medium | Interactive demos already live on converge.zone. |
| Runtime not ready for pilot execution | Medium | Critical | Depends on Milestone 4 (7 unbuilt MVP capabilities). Coordinate closely with Ren on engineering timeline. |
| Pricing too high for early SMBs | Medium | Medium | Monitor discovery call objections. $499/mo is margin-sustainable. Prepared to offer pilot discounts if needed. |
| Content doesn't drive inbound | High (early) | Low (early) | Content is a long-term play. Outbound drives cw1-cw8. Inbound kicks in cw8+. |
| Competitor launches similar positioning | Low | Medium | Our differentiator is determinism + explainability. Hard to copy the engine. |

---

## 12. Team Responsibilities

| Person | Role in GTM | Time Commitment |
|--------|------------|-----------------|
| **Blake Harmon** | GTM owner: outreach, pipeline, copy, case studies, pilot management | Full-time |
| **Kenneth Pernyer** | Warm intros, pricing approval, strategic review | 2-3 hrs/week |
| **Morgan Vale (CEO)** | Review pricing, approve external content, strategic decisions | 2-3 hrs/week |
| **Rio Castellan** | Visual design for all pages, marketing assets | ~20% of time |
| **Caroline Ashford** | Editorial quality, content calendar, editorial team management | ~30% of time |
| **Alice Mercer** | Signals articles, technical content | ~20% of time |
| **Bob Calder** | Business Voices, demo content | ~20% of time |
| **Jules Carrera** | Website implementation, analytics setup | ~15% of time |
| **Sam Okafor** | Pilot metrics instrumentation, data collection, anonymization | ~10% of time (ramps during pilots) |
| **Ren Akiyama** | Engineering coordination, roadmap input, runtime readiness | ~5% of time |

---

## 13. Immediate Action Items

### This Week (pre-cw1)

1. **Kenneth:** Review and approve this GTM plan
2. **Kenneth:** Review and approve pricing page content (`converge-business/gtm/pricing-page.md`)
3. **Kenneth:** Review and approve business-buyer landing page (`converge-business/gtm/business-buyer-landing-page.md`)
4. **Kenneth:** Identify 3-5 warm intro candidates from network
5. **Blake:** Prepare discovery call script adapted for Lead-to-Cash
6. **Blake:** Set up LinkedIn Sales Navigator and begin connection building

### cw1 (Mar 16-22)

7. **Kenneth:** Send warm intros
8. **Blake:** First 20-30 LinkedIn connections
9. **Blake + Caroline:** Publish Business Voices Ep. 1
10. **Jules:** Publish pricing page with approved content
11. **Jules:** Publish /for/operations with approved content

---

## Appendix A: Document Index

All supporting GTM documents live in `converge-business/gtm/`:

| Document | Purpose |
|----------|---------|
| `pricing-page.md` | Pricing tiers, comparison, FAQ |
| `business-buyer-landing-page.md` | /for/operations copy |
| `pilot-to-contract-playbook.md` | Full conversion funnel scripts and templates |
| `public-roadmap.md` | Public roadmap page content |
| `about-us-page.md` | About page copy |
| `release-readiness-target-list-and-outreach-script.md` | Release Readiness cold outreach |
| `release-readiness-design-partner-pilot-charter-loi.md` | Pilot charter template |
| `release-readiness-pilot-security-data-handling-one-pager.md` | Security trust signal |
| `smb-value.md` | SMB value creation narrative |
| `business-storytelling.md` | Core business narrative |
| `mvp-plan.md` | Technical MVP plan (Lead-to-Cash Lite) |
| `pack-value-props/` | Per-pack value propositions |

## Appendix B: Key Dependencies

| Dependency | Owner | Impact if Delayed |
|------------|-------|-------------------|
| CEO pricing approval | Kenneth | Blocks pricing page publish, outreach, and contract conversations |
| CEO landing page approval | Kenneth | Blocks /for/operations publish and LinkedIn outreach messaging |
| converge-core proof examples | Eli Marsh | Done. Unblocked Tech Voices Ep. 1 (in_review) and Signals article (in_review). |
| Runtime readiness for pilot execution | Ren Akiyama | Blocks pilot kickoffs |
| Interactive demo availability | Jules | Already live — no blocker. Note: demos are scripted simulations, not connected to engine. |

---

**End of document.**

*Blake Harmon, VP of Marketing & Sales — March 12, 2026*
