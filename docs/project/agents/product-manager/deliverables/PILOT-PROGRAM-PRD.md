# Pilot Program PRD — Product Requirements

> What the Converge pilot product must do, for whom, and by when.

**Owner:** Nadia Reeves, Product Manager
**Status:** v1.2 (Approved — Morgan, Blake, Ren. Alice's technical review findings incorporated.)
**Date:** 2026-03-12
**Reviewers:** Morgan Vale (CEO), Ren Akiyama (VP Eng), Blake Harmon (VP Marketing)

---

## 1. Problem Statement

Converge has a convergence engine, a growing set of agent traits, and a GTM strategy targeting 3-4 paying design partners. But there is no document that defines what the **pilot product** must actually do — what capabilities a customer interacts with, what integrations are required, and what "minimum viable pilot" looks like from the customer's perspective.

Without this, engineering builds what's technically interesting, GTM sells what sounds good, and the pilot fails in the gap between the two.

## 2. Ideal Customer Profile (ICP)

### 2.1 Primary ICP: Mid-Market Operations Leader

| Attribute | Definition |
|-----------|-----------|
| **Title** | VP/Director of Operations, RevOps, or Business Process |
| **Company size** | 10-500 employees (see sequencing below) |
| **Industry** | B2B SaaS, professional services, financial services, logistics |
| **Pain** | Multi-step business processes that span 3+ systems, require human coordination, and fail silently |
| **Budget authority** | $5K-$100K annual software spend decisions |
| **Technical maturity** | Has a CRM, uses Zapier/Make, may have tried RPA — not satisfied |
| **Trigger event** | Missed SLA, lost deal due to slow response, compliance audit finding, or process failure post-growth |

**Design partner sequencing:** First 2 partners from SMB beachhead (15-60 employees, $2M-$15M ARR) — shorter sales cycles, faster pilot setup, faster learning. Third partner can be from the larger mid-market range (100-500 employees). This sequences risk and aligns with the GTM Plan's outreach strategy.

### 2.2 Design Partner Selection Criteria

A design partner must meet **all** of these:

1. **Active pain** — Can describe a specific process that breaks regularly (not aspirational automation)
2. **Measurable baseline** — We can capture before-metrics within 2 weeks (see Pilot Metrics Framework)
3. **Integration surface** — Uses at least one system we can integrate with (CRM, email, webhook-capable tool)
4. **Executive sponsor** — Someone with budget authority who will champion the pilot internally
5. **Time commitment** — Willing to spend 2 hours/week for 4 weeks on the pilot
6. **Publication consent** — Willing to allow anonymized case study publication

### 2.3 Target Use Cases for First Pilots

| Priority | Use Case | Why First |
|----------|----------|-----------|
| **P0** | Lead-to-Close acceleration | Most B2B companies have this pain; CRM integration is straightforward; cycle time is measurable |
| **P0** | Multi-step approval workflows | Common in financial services and professional services; convergence model maps directly |
| **P1** | Compliance review automation | High-value pain in regulated industries; aligns with our security/governance positioning |
| **P1** | Cross-team handoff coordination | Pain scales with company size; demonstrates multi-agent value proposition |

## 3. Pilot Product Requirements

### 3.1 Minimum Viable Pilot (MVP) — What Must Work

These are **hard requirements** for any pilot engagement. If any of these fail, the pilot fails.

| ID | Requirement | Acceptance Criteria | Wave Dependency |
|----|------------|-------------------|-----------------|
| MVP-1 | **Convergence engine runs end-to-end** | Given a root intent with 3+ agents, the engine converges to a valid fixed point within 60 seconds (target, pending first pilot baseline — no production benchmarks exist yet; proof examples converge in ms with mock agents, but real LLM agents add ~3 sec/call latency) | Wave 1 (converge-core) |
| MVP-2 | **Customer can observe convergence** | Customer sees agent activity, proposals, and final converged state in a web UI or structured output. Note: existing converge-www demos (DemoTravel, DemoLeadToCash) are scripted simulations, NOT connected to the engine. The pilot observation UI must read from actual engine events via StreamingCallback or equivalent — this is a separate engineering task from the demos. | Wave 1 + converge-www |
| MVP-3 | **At least one integration works** | System receives external events (webhook) and produces external actions (API call to CRM or similar) | Custom per pilot |
| MVP-4 | **Human-in-the-loop gate** | Customer can pause convergence, review a proposal, and approve/reject before it becomes a fact | Wave 1 (converge-core HITL) |
| MVP-5 | **Audit trail exists** | Every convergence run produces a reviewable log: which agents ran, what they proposed, what was accepted, why | Wave 1 (converge-experience InMemory) |
| MVP-6 | **Telemetry captures pilot metrics** | Cycle time, agent execution time, convergence success/failure are captured per run. **Decision (Morgan):** Use InMemory store for first 2 pilots; add SurrealDB when volume justifies it. For pre-Wave-3 pilots, Sam builds a lightweight telemetry shim — accept reduced automation for first 1-2 pilots. | Pilot Metrics Framework |
| MVP-7 | **No customer data leaves their environment** | All processing happens in customer's infrastructure or a dedicated tenant; no shared compute | Security requirement |

### 3.2 Should-Have (Differentiated Pilot)

| ID | Requirement | Value | Wave Dependency |
|----|------------|-------|-----------------|
| SH-1 | **Before/after dashboard** | Customer sees quantified improvement in real-time | Pilot tooling |
| SH-2 | **Policy agent (Cedar)** | Compliance rules enforced as first-class agents | Wave 2 (converge-policy) |
| SH-3 | **Replay capability** | Customer can replay a convergence run to understand decisions | converge-experience |
| SH-4 | **Multiple LLM providers** | Failover and cost optimization across providers | Wave 2 (converge-provider) |

### 3.3 Nice-to-Have (Future Pilots)

| ID | Requirement | Wave Dependency |
|----|------------|-----------------|
| NH-1 | Self-service JTBD spec builder | Wave 3 (converge-tool) |
| NH-2 | Module marketplace browsing | Wave 3 (converge-domain) |
| NH-3 | WASM-based custom agent deployment | Wave 4 (converge-runtime) |

## 4. User Stories

### 4.1 Pilot Setup

```
As a Converge pilot lead (internal),
I want to configure a convergence workspace for a customer's use case
So that the pilot can run against their actual workflow within 1 week of kickoff.

Acceptance criteria:
- Workspace is created with customer-specific agents in < 4 hours
- At least one integration (webhook or API) is connected and tested
- Baseline metrics collection is running (per Pilot Metrics Framework)
- Customer has access to observe convergence output
```

### 4.2 Convergence Observation

```
As a pilot customer,
I want to see what Converge is doing with my data in real-time
So that I trust the system before giving it authority over my workflow.

Acceptance criteria:
- Web UI shows active agents, their proposals, and convergence status
- Customer can distinguish between "proposed" and "accepted" facts
- No jargon — labels use business language, not Converge internals
- Latency between engine action and UI update < 5 seconds
```

### 4.3 Human-in-the-Loop Decision

```
As a pilot customer,
I want to approve or reject agent proposals before they take effect
So that I maintain control over my business process during the pilot.

Acceptance criteria:
- System pauses when a HITL gate is reached
- Customer receives notification (email, Slack, or UI alert)
- Customer can see the proposal context: what agent proposed it, why, what data it used
- Customer clicks approve or reject; convergence resumes within 5 seconds
- Rejected proposals are logged with reason (optional free-text)
```

### 4.4 Pilot Results

```
As a pilot customer's executive sponsor,
I want a summary of what Converge achieved during the pilot
So that I can make a buy/no-buy decision.

Acceptance criteria:
- One-page summary with before/after metrics (per case study template)
- Specific examples of decisions Converge made correctly
- Clear statement of what Converge can't do yet (honest limitations)
- Pricing recommendation for continued use
- Conversion conversation follows pilot-to-contract playbook (see Leo's playbook)
```

**Cross-reference:** The conversion flow, email templates, and contract discussion structure are defined in Leo's [pilot-to-contract playbook](../../solutions-engineer/deliverables/pilot-to-contract-playbook.md). The PRD defines what the pilot must prove; the playbook defines how we convert proof into a contract.

## 5. Non-Functional Requirements

| Category | Requirement | Threshold |
|----------|------------|-----------|
| **Availability** | Pilot system uptime during business hours | 99% (planned maintenance excluded) |
| **Latency** | Convergence completion for typical run | < 60 seconds for 3-5 agents |
| **Security** | Data isolation between pilot customers | Dedicated workspace, no shared context |
| **Security** | Secrets management | No API keys in code, logs, or env dumps (per Ava's security issues) |
| **Compliance** | Data retention | Per Pilot Metrics Framework Section 11 |
| **Observability** | Error reporting | All failures surface to pilot lead within 5 minutes |

## 6. Dependencies & Sequencing

### 6.1 What Must Be Done Before First Pilot

| Dependency | Owner | Status | Blocks |
|-----------|-------|--------|--------|
| converge-core proof examples | Eli Marsh | Done | MVP-1 |
| converge-traits 1.0 contract | Eli Marsh | In progress (verify status) | MVP-1 |
| HITL gate implementation | Eli Marsh (REF-42, todo) | Not started — consensus: Slack/email + approve API (Option 2) | MVP-4 |
| Convergence observation UI | Jules Carrera | Partial (demo exists, but demos are scripted — real engine connection needed) | MVP-2 |
| Webhook integration framework | TBD | Not started | MVP-3 |
| Pilot telemetry exporter | Sam Okafor | Planned — lightweight InMemory shim for first 2 pilots (Morgan decision) | MVP-6 |
| Data isolation architecture | Ava Petrov | Planned | MVP-7 |
| Pricing tiers finalized | Blake + Morgan | In progress | Pilot close |

### 6.2 Critical Path

```
converge-core proof (MVP-1)
  → HITL gate (MVP-4)
    → Observation UI (MVP-2)
      → First integration (MVP-3)
        → Telemetry (MVP-6)
          → PILOT READY
```

**Estimated timeline to pilot-ready:** Per [Engineering Plan v2](../../../plans/ENGINEERING_PLAN.md), pilot-ready target is end of cw-5 (April 10). Critical path: converge-core proof (cw-2) → HITL gate (cw-3) → Observation UI (cw-3) → Webhook integration (cw-4) → Telemetry + data isolation (cw-4) → Integration testing (cw-5) → PILOT READY. Kenneth approves at end of cw-5; first design partner engagement begins cw-6 (April 13).

## 7. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to pilot-ready | Define in Eng Plan v2 | Calendar days from today to first pilot start |
| Pilot setup time | < 1 week | Days from customer agreement to first convergence run |
| Customer satisfaction (pilot) | NPS > 40 | Post-pilot survey |
| Pilot-to-contract conversion | > 50% | Pilots that convert to paid within 30 days |
| Requirements coverage | 100% MVP | All MVP-* items verified before pilot start |

## 8. Open Questions

| # | Question | Who Decides | Deadline | Status |
|---|----------|-------------|----------|--------|
| Q1 | Is the observation UI the converge-www demo page, or a separate pilot dashboard? | Ren + Jules | cw-2 | **Ren recommends Option 2** (extend converge-www demo). Blake notes separate dashboard is better for sales, but extending demo page with `/pilot/{id}` route works for first 2 partners. |
| Q2 | Do we build a generic webhook integration, or custom integrations per pilot? | Ren + Eli | cw-3 | **Ren recommends generic first** with escape hatch for custom adapters (Eng Plan D3). |
| Q3 | Can Wave 1 converge-experience (InMemory) support pilot telemetry, or do we need the Wave 3 exporter? | Sam + Eli | cw-3 | **Resolved (Morgan):** Bias toward InMemory for first 2 pilots. Don't let a database decision delay pilot readiness. Sam builds lightweight shim. |
| Q4 | What's the minimum viable HITL implementation? Full UI, or email/Slack notification with approve link? | Nadia + Ren | cw-2 | **Consensus: Option 2** (Slack/email notification + approve/reject API). Nadia, Ren, and Blake all agree. No web UI for HITL v1. |
| Q5 | Do we need multi-tenant isolation for pilots, or is single-tenant deployment acceptable for first 3-4? | Ava + Ren | cw-3 | Open — Ren's Eng Plan Q5 asks container vs. process vs. logical isolation. |
| Q6 | Who is the first design partner candidate? Do we have a named company? | Morgan + Blake | End of cw-2 (Mar 27) | **No named company yet.** Blake's GTM outreach strategy depends on Kenneth's warm intros. Target: identify 3-5 candidates by Mar 27, first pilot agreement by Apr 10. REF-41 tracks this. |

## 9. Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-03-12 | 1.0 | Initial draft — Nadia Reeves |
| 2026-03-12 | 1.1 | Incorporated Blake's GTM review: aligned ICP sizing (10-500 employees with SMB-first sequencing), added pilot-to-contract playbook cross-reference in Section 4.4, updated open questions with answers from Blake and Ren's Engineering Plan v2. Added timeline from Engineering Plan. — Nadia Reeves |
| 2026-03-12 | 1.2 | Approved by Morgan, Blake, Ren. Incorporated Alice Mercer's technical review (5 findings): calibrated MVP-1 convergence threshold as target pending baseline, clarified MVP-2 demo vs. real engine distinction, updated HITL status to REF-42 todo with approved approach, resolved Q3 per Morgan decision (InMemory for first 2 pilots), refreshed dependency statuses. — Nadia Reeves |
