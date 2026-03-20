# Engineering Plan v2

> From crate alignment to pilot-ready product. Wave-by-wave execution with dependency tracking, risk management, and pilot milestone mapping.

**Owner:** Ren Akiyama, VP Engineering
**Status:** Draft — awaiting Kenneth's review
**Date:** 2026-03-12
**Input from:** Eli Marsh (core/traits), Kira Novak (provider), Jules Carrera (frontend), Sam Okafor (QA), Dex Tanaka (DevOps), Ava Petrov (security), Nadia Reeves (PRD)

---

## 1. Strategic Context

### 1.1 Company Goal

Land 3-4 paying design partners with a working pilot product that demonstrates convergence on real business processes.

### 1.2 What "Pilot Ready" Means

Per the [Pilot Program PRD](../agents/product-manager/deliverables/PILOT-PROGRAM-PRD.md), the pilot product must satisfy 7 MVP requirements:

| ID | Requirement | Engineering Owner |
|----|------------|-------------------|
| MVP-1 | Convergence engine runs end-to-end (3+ agents, <60s) | Eli Marsh |
| MVP-2 | Customer can observe convergence (web UI) | Jules Carrera |
| MVP-3 | At least one integration works (webhook/API) | Eli Marsh + Leo Marin |
| MVP-4 | Human-in-the-loop gate (pause/approve/reject) | Eli Marsh |
| MVP-5 | Audit trail exists (reviewable convergence log) | Eli Marsh (InMemory store) |
| MVP-6 | Telemetry captures pilot metrics | Sam Okafor |
| MVP-7 | No customer data leaves their environment | Ava Petrov |

### 1.3 Critical Path to Pilot

```
converge-core proof examples (MVP-1)          ← NOW
  → HITL gate in converge-core (MVP-4)
    → Observation UI on converge-www (MVP-2)
      → First webhook integration (MVP-3)
        → Telemetry exporter wired (MVP-6)
          → Data isolation verified (MVP-7)
            → PILOT READY
```

Everything else (Wave 2-5 crates, GTM, content) is parallel work that does not gate pilot readiness.

## 2. Timeline — Converge Weeks

All estimates use [Converge Weeks (cw)](./CONVERGE_WEEK.md): 1 cw = 1 calendar week of full team output.

### 2.1 Phase 1: Pilot Critical Path (cw-2 through cw-5)

This is the only path that matters for the first design partner.

| cw | Dates | Milestone | Owner | MVP |
|----|-------|-----------|-------|-----|
| cw-2 | Mar 16-20 | **converge-core proof examples complete** — 5-6 concepts, mock agents, property-based tests, business-readable docs | Eli Marsh | MVP-1 |
| cw-2 | Mar 16-20 | **LlmAgent idempotency bug fixed** — convergence engine produces deterministic results | Eli Marsh | MVP-1 |
| cw-3 | Mar 23-27 | **HITL gate v1 in converge-core** — pause/resume/approve/reject API; Slack notification for pilot customers (Option 2 per PRD Q4) | Eli Marsh | MVP-4 |
| cw-3 | Mar 23-27 | **Audit trail verified** — InMemoryExperienceStore captures full run provenance | Eli Marsh | MVP-5 |
| cw-3 | Mar 23-27 | **Observation UI v1** — converge-www page showing live agent activity, proposals, converged state (build on Jules' demo work) | Jules Carrera | MVP-2 |
| cw-4 | Mar 30-Apr 3 | **Webhook integration framework** — inbound webhook triggers convergence; outbound API calls to CRM/external system | Eli Marsh + Leo Marin | MVP-3 |
| cw-4 | Mar 30-Apr 3 | **Telemetry exporter v1** — cycle time, convergence time, agent execution time captured per run | Sam Okafor | MVP-6 |
| cw-4 | Mar 30-Apr 3 | **Data isolation architecture** — dedicated workspace per customer, no shared state, secrets management verified | Ava Petrov | MVP-7 |
| cw-5 | Apr 6-10 | **Pilot integration testing** — end-to-end: webhook → convergence → HITL gate → observation UI → telemetry → audit trail | Sam Okafor (QA) | All |
| cw-5 | Apr 6-10 | **Pilot runbook** — setup checklist, troubleshooting, monitoring | Dex Tanaka | Ops |

**Decision point (end of cw-5):** Kenneth approves pilot readiness. If yes, first design partner engagement begins cw-6.

### 2.2 Phase 2: Platform Foundation — Waves 1-2 (cw-2 through cw-5, parallel)

This work runs in parallel with the pilot critical path and does not block it.

#### Wave 1: Foundation (cw-2 to cw-3)

| Task | Owner | Status | Duration | Dependencies |
|------|-------|--------|----------|--------------|
| converge-traits 1.0 freeze | Eli Marsh | **DONE** (v0.3.0) | — | — |
| converge-core proof examples | Eli Marsh | **In progress** | 1 cw | traits freeze |
| converge-business story audit | Blake Harmon | **DONE** | — | — |

**Quality gate:** converge-core proof examples pass. A new developer reads them and understands convergence in 15 minutes. Property-based tests pass for all core invariants.

#### Wave 2: Instantiation (cw-3 to cw-5)

All 5 crates can proceed in parallel once traits are frozen.

| Crate | Owner | Duration | Dependencies | Notes |
|-------|-------|----------|--------------|-------|
| converge-provider | Kira Novak | 1.5 cw | traits freeze (DONE) | **Already started.** Anthropic + OpenAI. Wiremock tests. |
| converge-llm | Kira Novak | 1 cw | traits freeze | After provider. Burn inference. |
| converge-analytics | Unassigned | 1 cw | traits freeze | LanceDB vector search. Can wait. |
| converge-policy | Unassigned | 1 cw | traits freeze | Cedar SDK. Aligns with SOC 2 story. |
| converge-optimization | Unassigned | 1 cw | traits freeze | CP-SAT. Lowest priority in Wave 2. |

**Quality gate:** `cargo test` passes with no API keys in all Wave 2 crates. Each crate has at least one example agent participating in convergence.

### 2.3 Phase 3: Waves 3-5 (cw-6+, after pilot launch)

These waves are post-pilot. Sequencing depends on pilot feedback.

| Wave | Crates | Est. Duration | Earliest Start |
|------|--------|---------------|----------------|
| Wave 3: Tooling | converge-tool, converge-domain, converge-experience | 2-3 cw | cw-6 |
| Wave 4: Infrastructure | converge-runtime | 2 cw | cw-8 (after Wave 3) |
| Wave 5: Experience | converge-remote, converge-application, converge-personas | 2-3 cw | cw-10 (after Wave 4) |

**These estimates are deliberately coarse.** We will refine them after pilot feedback tells us which capabilities customers actually need.

## 3. Per-Engineer Workload

### 3.1 cw-2 (Mar 16-20)

| Agent | Primary Task | Secondary Task | Load |
|-------|-------------|----------------|------|
| **Eli Marsh** | converge-core proof examples + LlmAgent bug | — | FULL (critical path) |
| **Kira Novak** | converge-provider implementation | — | FULL |
| **Jules Carrera** | Observation UI v1 (build on demo) | — | FULL |
| **Sam Okafor** | Telemetry exporter design + data model | QA review of proof examples | 70% |
| **Dex Tanaka** | CI/CD pipeline for converge-core + traits | Git push fix (REF-33) | 70% |
| **Ava Petrov** | SOC 2 policies (continuing) | Data isolation architecture design | 80% |
| **Leo Marin** | Webhook integration research + design | — | 50% (ramping) |

### 3.2 cw-3 (Mar 23-27)

| Agent | Primary Task | Secondary Task | Load |
|-------|-------------|----------------|------|
| **Eli Marsh** | HITL gate v1 + audit trail verification | — | FULL (critical path) |
| **Kira Novak** | converge-provider completion | — | FULL |
| **Jules Carrera** | Observation UI integration with converge-core | — | FULL |
| **Sam Okafor** | Telemetry exporter implementation | QA review of HITL gate | 80% |
| **Dex Tanaka** | CI/CD for converge-provider | Staging environment setup | 80% |
| **Ava Petrov** | SOC 2 policies (continuing) | Data isolation implementation | 80% |
| **Leo Marin** | Webhook integration prototype | — | 70% |

### 3.3 cw-4 (Mar 30-Apr 3)

| Agent | Primary Task | Secondary Task | Load |
|-------|-------------|----------------|------|
| **Eli Marsh** | Webhook integration framework (with Leo) | Edge case hardening | FULL |
| **Kira Novak** | converge-llm (if provider done) or provider polish | — | FULL |
| **Jules Carrera** | Observation UI polish + HITL UI integration | — | FULL |
| **Sam Okafor** | Telemetry exporter completion + integration tests | — | FULL |
| **Dex Tanaka** | Deployment pipeline for pilot | Monitoring setup | FULL |
| **Ava Petrov** | Data isolation verification + security review | — | FULL |
| **Leo Marin** | Webhook integration (with Eli) | — | FULL |

### 3.4 cw-5 (Apr 6-10)

| Agent | Primary Task | Load |
|-------|-------------|------|
| **Sam Okafor** | End-to-end pilot integration testing | FULL |
| **Dex Tanaka** | Pilot runbook + deployment verification | FULL |
| **All engineers** | Bug fixes from integration testing | As needed |
| **Ren Akiyama** | Pilot readiness assessment for Kenneth | — |

## 4. Dependency Graph

```
converge-traits v0.3.0 (DONE)
  ├── converge-core proof examples (Eli, cw-2) ← CRITICAL PATH
  │     ├── HITL gate v1 (Eli, cw-3) ← CRITICAL PATH
  │     │     └── Observation UI integration (Jules, cw-3)
  │     │           └── Webhook integration (Eli+Leo, cw-4)
  │     │                 └── Telemetry wired (Sam, cw-4)
  │     │                       └── Data isolation verified (Ava, cw-4)
  │     │                             └── Integration testing (Sam, cw-5)
  │     │                                   └── PILOT READY (cw-5)
  │     └── REF-8 LLM validation boundary (Eli, cw-3, parallel)
  │
  ├── converge-provider (Kira, cw-2-3) ← PARALLEL, not on critical path
  ├── converge-llm (Kira, cw-4-5) ← PARALLEL
  └── CI/CD pipeline (Dex, cw-2-3) ← PARALLEL
```

**Critical path length: 4 cw (cw-2 through cw-5)**

The critical path runs exclusively through Eli Marsh for cw-2 and cw-3. This is the single biggest risk.

## 5. Risk Register

| # | Risk | Severity | Probability | Mitigation |
|---|------|----------|-------------|------------|
| R1 | **Eli Marsh is a single point of failure on critical path** | CRITICAL | HIGH | Kira Novak is backup for core work if Eli is blocked. Ren reviews all core PRs to maintain context. |
| R2 | **LlmAgent idempotency bug harder than expected** | HIGH | MEDIUM | Time-box to 2 days in cw-2. If not fixed, implement deterministic seed workaround for pilot. |
| R3 | **HITL gate scope creep** | HIGH | MEDIUM | Strict MVP: Slack notification + approve/reject API. No web UI for HITL in v1 — web UI is observation only. |
| R4 | **Git push still blocked (REF-33)** | MEDIUM | MEDIUM | Dex resolving. Workaround: share commits via local copy or patch files. |
| R5 | **No design partner identified yet** | HIGH | HIGH | Morgan + Blake working this (REF-41). Engineering plan assumes partner identified by cw-4. |
| R6 | **Observation UI requires converge-core API changes** | MEDIUM | MEDIUM | Jules and Eli align on API contract in cw-2. Define interface before building. |
| R7 | **Webhook integration is custom per pilot** | MEDIUM | LOW | Build generic framework first. First pilot integration may need custom adapter. |
| R8 | **SOC 2 tooling (Vanta) budget not approved** | LOW | MEDIUM | Escalated to Morgan. Manual compliance tracking is fallback. |

## 6. Decision Points for Kenneth

| # | Decision | When | Options | Ren's Recommendation |
|---|----------|------|---------|---------------------|
| D1 | **HITL gate implementation approach** | cw-2 | (1) Full web UI, (2) Slack/email + API, (3) CLI only | **Option 2** — Slack notification for pilot, web UI later. Lowest risk, widest compatibility. |
| D2 | **Observation UI: separate app or converge-www page?** | cw-2 | (1) New pilot dashboard app, (2) Extend converge-www demo | **Option 2** — Jules already has demo infrastructure. Extend, don't rebuild. |
| D3 | **Webhook: generic framework or custom per pilot?** | cw-3 | (1) Generic webhook framework, (2) Custom integration per pilot | **Option 1 first**, with escape hatch for custom adapters. |
| D4 | **Pilot readiness gate** | End of cw-5 | Ship pilot / delay / scope down | Depends on integration testing results. |
| D5 | **Wave 2 crate staffing** | cw-3 | Assign analytics/policy/optimization to existing team or hire | **Defer hiring.** Kira handles provider + llm. Policy can wait until pilot proves Cedar demand. |
| D6 | **SOC 2 tooling budget (Vanta)** | cw-2 | Approve $15-25K/yr for Vanta or manual compliance | **Recommend Vanta** — manual tracking is unsustainable with 16 agents. |

## 7. Quality Gates

### 7.1 Wave 1 Exit Criteria (end of cw-3)

- [ ] converge-core: 5-6 proof examples pass, property-based tests green
- [ ] converge-core: LlmAgent idempotency bug fixed or deterministic workaround in place
- [ ] converge-core: HITL gate API implemented and tested
- [ ] converge-core: InMemoryExperienceStore captures full provenance
- [ ] converge-traits: v0.3.0 published (or v1.0 if ready)
- [ ] converge-core + converge-traits: CI/CD pipeline green
- [ ] A new developer can read proof examples and understand convergence in 15 minutes

### 7.2 Pilot Readiness Exit Criteria (end of cw-5)

- [ ] All 7 MVP requirements verified (MVP-1 through MVP-7)
- [ ] End-to-end integration test passes: webhook → convergence → HITL → UI → telemetry → audit
- [ ] Pilot runbook exists and has been dry-run
- [ ] Data isolation architecture reviewed by Ava
- [ ] At least one design partner identified (Morgan/Blake)
- [ ] Pricing tiers finalized (Blake/Morgan)
- [ ] Security one-pager ready for customer distribution

### 7.3 Wave 2 Exit Criteria (end of cw-5)

- [ ] converge-provider: `cargo test` passes with no API keys; Anthropic + OpenAI work with wiremock
- [ ] Each Wave 2 crate has at least one example agent in convergence

## 8. Testing Strategy

### 8.1 Unit & Property-Based Tests

| Scope | Owner | Framework | When |
|-------|-------|-----------|------|
| converge-core invariants | Eli Marsh | proptest | cw-2 |
| converge-traits contract compliance | Eli Marsh | standard #[test] | Done (v0.3.0) |
| converge-provider mock responses | Kira Novak | wiremock | cw-3 |
| HITL gate state machine | Eli Marsh | proptest | cw-3 |

### 8.2 Integration Tests

| Scope | Owner | When |
|-------|-------|------|
| Core → Experience store round-trip | Sam Okafor | cw-3 |
| Core → HITL → Observation UI | Sam Okafor | cw-4 |
| Webhook → Core → External API | Sam Okafor | cw-5 |
| Full pilot flow (end-to-end) | Sam Okafor | cw-5 |

### 8.3 Adversarial Tests

| Scenario | Owner | When |
|----------|-------|------|
| Non-converging agent loop (budget exhaustion) | Sam Okafor | cw-3 |
| Malformed proposals (ProposedFact validation) | Ava Petrov | cw-3 |
| Concurrent HITL approvals (race condition) | Sam Okafor | cw-4 |
| Data leakage between workspaces | Ava Petrov | cw-4 |

### 8.4 LlmAgent Idempotency Bug

**Bug:** LlmAgent produces non-deterministic convergence results due to LLM response variance.

**Fix approach (Eli, cw-2):**
1. Seed-based prompt construction for reproducible LLM calls
2. Fact normalization: canonicalize ProposedFact content before comparison
3. Convergence tolerance: fixed-point detection uses semantic similarity, not exact match
4. Test: same input produces same convergence outcome across 10 runs

**Fallback:** If full fix is too complex, implement deterministic seed + retry with temperature=0 for pilot.

## 9. Open Questions (Engineering Input Needed)

| # | Question | Needs Answer From | By When |
|---|----------|-------------------|---------|
| Q1 | Can InMemoryExperienceStore handle pilot telemetry volume, or do we need SurrealDB? | Eli + Sam | cw-3 |
| Q2 | Does HITL gate need its own crate or is it a module in converge-core? | Eli | cw-2 |
| Q3 | What's the observation UI data contract? SSE, WebSocket, or polling? | Jules + Eli | cw-2 |
| Q4 | Can converge-provider tests run in CI without API keys? (wiremock sufficiency) | Kira + Dex | cw-3 |
| Q5 | Multi-tenant isolation: container-level, process-level, or logical? | Ava | cw-3 |

## 10. What This Plan Does NOT Cover

- **Wave 3-5 detailed scheduling** — deferred until pilot feedback (cw-6+)
- **Hiring plan** — not needed yet; current team can deliver pilot
- **Mobile (converge-android, converge-ios)** — not on pilot critical path
- **Marketing/GTM execution** — Blake owns separately
- **Pricing/commercial terms** — Priya + Morgan + Blake own separately

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2026-03-12 | 0.1 | Initial draft — Ren Akiyama |
