# Integration Test Plan — cw-3 through cw-5

**Author:** Sam Okafor, QA Engineer
**Issue:** REF-55 (MVP-6)
**Status:** Draft
**Created:** 2026-03-12
**Scope:** Integration and end-to-end tests for the pilot critical path (MVP-1 through MVP-7)

---

## 1. Purpose

This plan defines every integration test required to declare pilot readiness at the end of cw-5. Each test maps to one or more MVP requirements. Tests are sequenced by dependency — we can only test what's been built.

## 2. Test Matrix

### 2.1 Summary

| Phase | Week | Tests | MVP Coverage |
|-------|------|-------|-------------|
| Phase 1: Core Integration | cw-3 | 8 tests | MVP-1, MVP-4, MVP-5 |
| Phase 2: Telemetry + HITL | cw-4 | 7 tests | MVP-2, MVP-3, MVP-4, MVP-6 |
| Phase 3: End-to-End Pilot | cw-5 | 6 tests | All (MVP-1 through MVP-7) |
| **Total** | | **21 tests** | |

### 2.2 Test Environment

- **Rust toolchain:** stable (same as CI)
- **Test harness:** `cargo test` + `#[tokio::test]` for async
- **Mocking:** `wiremock` for HTTP, `InMemoryExperienceStore` for storage
- **Property tests:** `proptest` for invariant verification
- **No external services required** — all tests run without API keys, databases, or network access
- **Feature flags:** Tests tagged `#[cfg(feature = "telemetry")]` for telemetry-specific assertions

---

## 3. Phase 1: Core Integration Tests (cw-3, Mar 23-27)

**Depends on:** converge-core proof examples (cw-2), HITL gate v1 (cw-3)

### IT-01: Multi-Agent Convergence Round-Trip

**MVP:** MVP-1 (convergence engine runs end-to-end)
**What:** 3 mock agents converge to a fixed point within budget. Experience store captures all events.

```
Setup: 3 agents (pricing, routing, approval), InMemoryExperienceStore
Input: Initial context with lead data
Assert:
  - Engine returns Converged outcome
  - Cycle count > 0 and <= budget.max_cycles
  - Each agent executed at least once
  - Final fact count matches expected (no phantom facts)
  - ExperienceStore contains ProposalCreated, ProposalValidated, FactPromoted events
  - Events ordered by timestamp (monotonic)
```

### IT-02: Non-Converging Agent — Budget Exhaustion

**MVP:** MVP-1
**What:** An adversarial agent that always proposes new facts. Engine must halt at budget limit.

```
Setup: 1 well-behaved agent + 1 "flip-flop" agent that alternates proposals
Input: Any valid context
Assert:
  - Engine returns BudgetExhausted
  - cycles_used == budget.max_cycles
  - No panic or hang (timeout: 10s)
  - ExperienceStore records BudgetExceeded event
```

### IT-03: HITL Gate — Approve Flow

**MVP:** MVP-4 (human-in-the-loop gate)
**What:** Agent proposal triggers HITL gate. Simulated human approves. Convergence continues.

```
Setup: 2 agents, HitlPolicy gating "high_value_proposal" kind
Input: Context triggering high-value proposal
Procedure:
  1. Engine runs until gate fires (returns GateRequest)
  2. Submit GateDecision::Approve
  3. Engine resumes and converges
Assert:
  - GateRequest contains correct proposal_id, agent_name, cycle
  - After approval, engine converges normally
  - ExperienceStore contains GateEvent::Requested + GateEvent::Approved
  - Final facts include the gated proposal's content
```

### IT-04: HITL Gate — Reject Flow

**MVP:** MVP-4
**What:** Same as IT-03 but human rejects. Convergence halts or excludes the proposal.

```
Assert:
  - After rejection, engine completes (with or without convergence)
  - GateEvent::Rejected recorded with reason
  - Rejected proposal's content NOT in final facts
```

### IT-05: HITL Gate — Timeout Flow

**MVP:** MVP-4
**What:** HITL gate fires, no human responds. Timeout policy executes.

```
Setup: TimeoutPolicy::AutoReject { after_seconds: 1 }
Assert:
  - After 1s, gate auto-rejects
  - GateEvent::TimedOut recorded
  - Engine continues without gated proposal
```

### IT-06: Experience Store — Full Provenance Chain

**MVP:** MVP-5 (audit trail)
**What:** After a convergence run, query the experience store and reconstruct the full provenance chain for any fact.

```
Setup: 3 agents, InMemoryExperienceStore
Input: Lead-to-close scenario
Assert:
  - For each final Fact, can trace back:
    ProposalCreated → ProposalValidated → FactPromoted
  - Each event has correlation_id linking to the run
  - TraceLinks are present and reference the producing agent
  - No orphaned events (every Validated has a Created, every Promoted has a Validated)
```

### IT-07: Invariant Violation Detection

**MVP:** MVP-1
**What:** Agent proposes a fact that violates a structural invariant. Engine detects and reports.

```
Setup: Agent that proposes duplicate context keys (structural invariant)
Assert:
  - Engine reports InvariantViolation
  - Violation details identify the conflicting keys
  - Violating proposal is NOT promoted to Fact
  - ExperienceStore records the violation
```

### IT-08: Determinism — Replay Produces Same Result

**MVP:** MVP-1
**What:** Run the same agents + same initial context 5 times. Compare final states.

```
Setup: 3 mock agents (no LLM — deterministic execute())
Assert:
  - All 5 runs produce identical final fact sets
  - All 5 runs take the same number of cycles
  - All 5 runs produce the same event sequence (by kind, not timestamp)
```

---

## 4. Phase 2: Telemetry + UI Integration Tests (cw-4, Mar 30-Apr 3)

**Depends on:** Phase 1 passing, telemetry exporter v1, observation UI v1, webhook integration

### IT-09: Telemetry — RunTelemetryReport Correctness

**MVP:** MVP-6 (telemetry captures pilot metrics)
**What:** After convergence, the telemetry collector emits a complete RunTelemetryReport.

```
Setup: 3 agents, InMemoryTelemetryCollector
Assert:
  - report.run_id is non-empty UUID
  - report.duration_ms > 0 and plausible (< 10s for mock agents)
  - report.cycle_count matches engine's actual cycle count
  - report.agent_timings.len() == 3 (one per agent)
  - Each AgentTiming.total_execution_ms > 0
  - Each AgentTiming.executions >= 1
  - report.outcome == RunOutcome::Converged
  - report.final_fact_count matches actual facts
  - report.invariant_violations == 0 (clean run)
```

### IT-10: Telemetry — Per-Agent Timing Accuracy

**MVP:** MVP-6
**What:** Verify per-agent timing is accurate (not double-counted, not missing).

```
Setup: 3 agents with known sleep durations (agent_a: 10ms, agent_b: 20ms, agent_c: 5ms)
Assert:
  - AgentTiming for agent_a: total_execution_ms within 10ms ± 5ms per execution
  - AgentTiming for agent_b: total_execution_ms within 20ms ± 5ms per execution
  - Sum of all agent times <= report.duration_ms (no double-counting)
  - No agent has 0ms execution time if it ran
```

### IT-11: Telemetry — HITL Wait Time Capture

**MVP:** MVP-4, MVP-6
**What:** When HITL gate fires, telemetry captures the wait duration.

```
Setup: Agent triggers HITL gate, simulated 500ms delay before approval
Assert:
  - report.hitl_events.len() == 1
  - HitlTiming.wait_ms within 500ms ± 100ms
  - HitlTiming.verdict == "approved"
  - HitlTiming.gate_id matches the GateRequest.gate_id
```

### IT-12: Telemetry — Budget Exhaustion Report

**MVP:** MVP-6
**What:** When engine hits budget, telemetry reports BudgetExhausted outcome with correct counts.

```
Setup: Non-converging agents, budget.max_cycles = 5
Assert:
  - report.outcome == RunOutcome::BudgetExhausted { cycles_used: 5, max_cycles: 5 }
  - report.cycle_count == 5
  - report.budget.cycles_used == 5
```

### IT-13: Telemetry — JSON Lines Export

**MVP:** MVP-6
**What:** File-based exporter writes valid JSON Lines.

```
Setup: Run 3 convergence runs with file exporter targeting temp directory
Assert:
  - File exists at expected path
  - File contains exactly 3 lines
  - Each line deserializes to valid RunTelemetryReport
  - Reports have distinct run_ids
  - Reports have monotonically increasing started_at timestamps
```

### IT-14: Telemetry — Feature Flag Disabled

**MVP:** MVP-6
**What:** When `telemetry` feature is disabled, engine compiles and runs without telemetry overhead.

```
Build: cargo test --no-default-features (without telemetry)
Assert:
  - Compiles successfully
  - Engine runs and converges (same as IT-01)
  - No telemetry structs or collector calls present in binary (verify via cargo build --release size comparison)
```

### IT-15: Webhook → Convergence Trigger

**MVP:** MVP-3 (integration works)
**What:** Inbound webhook triggers a convergence run. Outbound call fires on completion.

```
Setup: wiremock for inbound webhook endpoint, wiremock for outbound CRM callback
Input: POST /webhook/trigger with lead creation payload
Assert:
  - Convergence run starts within 100ms of webhook receipt
  - Engine converges
  - Outbound CRM callback fires with converged result
  - Telemetry report includes correct job_type from webhook metadata
```

---

## 5. Phase 3: End-to-End Pilot Tests (cw-5, Apr 6-10)

**Depends on:** All Phase 1 + Phase 2 passing, all MVP components deployed

### IT-16: Full Pilot Flow — Happy Path

**MVP:** All (MVP-1 through MVP-7)
**What:** Complete pilot scenario from webhook trigger through observation UI to telemetry export.

```
Scenario: Lead-to-close pilot
Flow:
  1. Inbound webhook → lead creation event
  2. Engine starts convergence (3+ agents)
  3. Agent proposes high-value action → HITL gate fires
  4. Human approves via API
  5. Engine resumes and converges
  6. Observation UI receives convergence events (via polling/SSE)
  7. Telemetry exporter writes RunTelemetryReport
  8. Audit trail in ExperienceStore is complete

Assert:
  - All 7 steps complete without error
  - Total latency < 60 seconds (MVP-1 requirement)
  - Telemetry report captures all signals (cycle, agent time, HITL wait, outcome)
  - Audit trail reconstructable from experience store
  - No customer data leaks to other tenant's workspace
```

### IT-17: Full Pilot Flow — HITL Rejection

**MVP:** All
**What:** Same as IT-16 but human rejects at HITL gate.

```
Assert:
  - Engine handles rejection gracefully
  - Observation UI shows rejection status
  - Telemetry records the rejection
  - Outbound CRM callback does NOT fire for the rejected action
```

### IT-18: Multi-Tenant Isolation

**MVP:** MVP-7 (no customer data leaves their environment)
**What:** Two concurrent pilot runs for different tenants. No data leakage.

```
Setup: Tenant A and Tenant B, each with separate workspace, agents, context
Run: Both convergence runs simultaneously (or sequentially in same process)
Assert:
  - Tenant A's experience store has zero events from Tenant B
  - Tenant B's experience store has zero events from Tenant A
  - Telemetry files written to separate directories
  - No cross-tenant correlation_ids
  - Context keys from Tenant A not visible to Tenant B's agents
```

### IT-19: Pilot Under Load — 10 Concurrent Runs

**MVP:** MVP-1, MVP-6
**What:** Simulate 10 concurrent convergence runs (represents a busy pilot day).

```
Setup: 10 independent convergence runs, same agent definitions, different initial contexts
Assert:
  - All 10 runs converge within 60s each
  - No panics, deadlocks, or data corruption
  - All 10 RunTelemetryReports written (no lost data)
  - Memory usage stays below 500MB (reasonable for 10 in-memory runs)
```

### IT-20: Telemetry → Pilot Metrics Aggregation

**MVP:** MVP-6
**What:** Telemetry output feeds the weekly aggregation pipeline.

```
Setup: Generate 20 RunTelemetryReports to JSON Lines file
Run: scripts/pilot-metrics-aggregate.sh (once built)
Assert:
  - Aggregated output matches expected averages
  - Per-agent timing summaries correct
  - HITL wait times averaged correctly
  - Output format matches Pilot Metrics Framework Section 3.2 spec
```

### IT-21: Data Disposal After Pilot

**MVP:** MVP-7
**What:** After pilot ends, disposal script correctly removes raw data while preserving anonymized copies.

```
Setup: Populated pilot-data/{tenant-id}/ with telemetry.jsonl + raw data
Run: scripts/pilot-data-dispose.sh --customer {tenant-id}
Assert:
  - Raw telemetry files deleted
  - Anonymized copies still exist
  - Disposal audit log entry written
  - scripts/pilot-pii-scan.sh --strict on remaining files passes
```

---

## 6. Test Infrastructure

### 6.1 Test Fixtures

| Fixture | Description | Location |
|---------|-------------|----------|
| `mock_agents` | 3 deterministic agents (pricing, routing, approval) | `converge-core/tests/fixtures/` |
| `adversarial_agent` | Agent that never converges | `converge-core/tests/fixtures/` |
| `hitl_scenario` | Context + agents that trigger HITL gate | `converge-core/tests/fixtures/` |
| `pilot_lead_context` | Realistic lead-to-close initial context | `converge-core/tests/fixtures/` |
| `webhook_payloads` | Sample inbound webhook JSON | Integration test crate |

### 6.2 Helper Utilities

| Utility | Purpose |
|---------|---------|
| `assert_provenance_chain(store, fact_id)` | Verify complete event chain for a fact |
| `assert_telemetry_complete(report)` | Validate all P0 fields are populated |
| `assert_no_cross_tenant(store_a, store_b)` | Verify zero overlap between tenant stores |
| `timed_agent(name, sleep_ms)` | Agent with configurable execution delay for timing tests |

### 6.3 CI Integration

```yaml
# Suggested CI job structure
integration-tests:
  runs-on: ubuntu-latest
  steps:
    - cargo test --features telemetry -p converge-core --test integration
    - cargo test --features telemetry -p converge-experience --test integration
    - cargo test --no-default-features -p converge-core  # feature flag verification
```

## 7. Test Dependencies and Sequencing

```
cw-3 (Phase 1):
  IT-01 ──┐
  IT-02 ──┤
  IT-07 ──┤── Can run in parallel (core only, no HITL)
  IT-08 ──┘
  IT-03 ──┐
  IT-04 ──┤── Can run in parallel (need HITL gate)
  IT-05 ──┤
  IT-06 ──┘

cw-4 (Phase 2):
  IT-09 ──┐
  IT-10 ──┤── Can run in parallel (telemetry only)
  IT-11 ──┤
  IT-12 ──┤
  IT-13 ──┤
  IT-14 ──┘
  IT-15 ────── Needs webhook integration

cw-5 (Phase 3):
  IT-16 ──┐
  IT-17 ──┤── Sequential (full flow)
  IT-18 ──┤
  IT-19 ──┤
  IT-20 ──┤
  IT-21 ──┘
```

## 8. Pass/Fail Criteria

### 8.1 Per-Phase Gates

| Phase | Gate | Threshold |
|-------|------|-----------|
| Phase 1 | Core integration | 8/8 pass (zero tolerance — these are foundational) |
| Phase 2 | Telemetry + UI | 6/7 pass (IT-14 feature flag can be deferred if blocking) |
| Phase 3 | End-to-end | 5/6 pass (IT-20 depends on aggregation script which may not be ready) |

### 8.2 Pilot Readiness Gate

**All of the following must be true:**

- [ ] IT-16 (full pilot happy path) passes
- [ ] IT-18 (multi-tenant isolation) passes
- [ ] IT-21 (data disposal) passes
- [ ] No critical or high severity bugs open against pilot path
- [ ] Telemetry exporter produces valid RunTelemetryReport for a real convergence run

### 8.3 Known Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| HITL gate not ready by cw-3 | IT-03/04/05/11 blocked | Test core convergence (IT-01/02/07/08) first; HITL tests can slip to early cw-4 |
| Webhook integration delayed | IT-15/IT-16 blocked | Mock webhook layer; test convergence trigger directly |
| Observation UI not testable via cargo | IT-16 UI assertion impossible | Assert event emission only; UI visual testing is manual or Playwright (separate) |
| converge-experience SurrealDB instability | IT-06 may fail | Use InMemoryExperienceStore for all integration tests; SurrealDB tested separately |

## 9. Ownership

| Activity | Owner | Reviewer |
|----------|-------|----------|
| Test plan authorship | Sam Okafor | Ren Akiyama |
| Phase 1 test implementation | Sam Okafor | Eli Marsh (fixture review) |
| Phase 2 test implementation | Sam Okafor | Eli Marsh + Jules Carrera |
| Phase 3 test execution | Sam Okafor | Ren Akiyama (readiness call) |
| Bug triage from test failures | Sam Okafor (file) | Responsible engineer (fix) |

---

**Reviewer:** Ren Akiyama (VP Engineering), Eli Marsh (core engine owner)
