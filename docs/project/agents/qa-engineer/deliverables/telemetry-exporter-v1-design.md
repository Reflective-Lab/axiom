# Telemetry Exporter v1 — Design Document

**Author:** Sam Okafor, QA Engineer
**Issue:** REF-55 (MVP-6)
**Status:** Draft
**Created:** 2026-03-12
**Target:** Design complete by end of cw-2 (Mar 20). Implementation cw-4.

---

## 1. Purpose

The telemetry exporter captures structured performance data from every convergence run. This data feeds the Pilot Metrics Framework (Section 4.1) and enables before/after measurement for design partner pilots. Without it, we cannot measure convergence time, iteration count, per-agent time, or HITL wait — the metrics that prove Converge works.

## 2. Requirements

From the task description, the exporter must capture:

| Signal | Source | Priority |
|--------|--------|----------|
| Cycle count | Engine loop | P0 |
| Convergence time (total) | Engine `run()` start → fixed-point | P0 |
| Per-agent execution time | `Agent::execute()` calls | P0 |
| HITL wait time | Gate request → decision timestamps | P0 |
| Final state | Convergence success/failure + fact count | P0 |
| Per-agent token/cost usage | `BackendUsage` from responses | P1 |
| Invariant violation count | Invariant check results | P1 |
| Decision step breakdown | `OutcomeRecorded` events | P2 |

## 3. Architecture

### 3.1 Design Principles

1. **Zero runtime cost when disabled** — feature-gated (`telemetry` feature flag)
2. **No external dependencies for MVP** — structured JSON to file or in-memory buffer
3. **Append-only** — telemetry events are immutable once emitted
4. **Correlation by run** — every event carries `run_id` for grouping
5. **Built on existing hooks** — extend `StreamingCallback` and `ExperienceStore`, don't replace them

### 3.2 Integration Points

The exporter plugs into three existing hooks in converge-core:

```
Engine::run()
  │
  ├─ [NEW] TelemetryCollector::on_run_start(run_id, agent_count, context_snapshot)
  │
  ├─ per cycle:
  │   ├─ StreamingCallback::on_cycle_start(cycle)           [existing]
  │   ├─ [NEW] per agent: TelemetryCollector::on_agent_start(agent_id)
  │   ├─ Agent::execute(ctx) → AgentEffect                  [existing]
  │   ├─ [NEW] per agent: TelemetryCollector::on_agent_end(agent_id, effect_summary)
  │   ├─ [NEW] if HITL gate: TelemetryCollector::on_hitl_start(gate_id)
  │   ├─ [NEW] if HITL gate: TelemetryCollector::on_hitl_end(gate_id, verdict)
  │   └─ StreamingCallback::on_cycle_end(cycle, facts_added) [existing]
  │
  └─ [NEW] TelemetryCollector::on_run_end(outcome)
         │
         └─ Emits: RunTelemetryReport (complete structured summary)
```

### 3.3 Where Code Changes

| Crate | File | Change |
|-------|------|--------|
| `converge-core` | `engine.rs` | Wrap `Agent::execute()` calls with timing. Call collector hooks. |
| `converge-core` | `gates/hitl.rs` | Emit gate timing events to collector. |
| `converge-core` | New: `telemetry.rs` | `TelemetryCollector` trait + `RunTelemetryReport` struct. |
| `converge-experience` | `lib.rs` | Implement `TelemetryCollector` as an `ExperienceStore` decorator. |
| New crate (optional) | `converge-telemetry` | Standalone exporter if we want file/Prometheus output decoupled from core. |

**Recommendation:** Start with `telemetry.rs` in converge-core behind a `telemetry` feature flag. Extract to `converge-telemetry` crate only if the module exceeds ~500 lines.

## 4. Data Model

### 4.1 RunTelemetryReport

The primary output — one per convergence run:

```rust
/// Complete telemetry for a single convergence run.
/// Emitted by TelemetryCollector::on_run_end().
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTelemetryReport {
    /// Unique identifier for this run
    pub run_id: String,

    /// ISO-8601 timestamps
    pub started_at: String,
    pub ended_at: String,

    /// Total wall-clock duration in milliseconds
    pub duration_ms: u64,

    /// Number of convergence cycles executed
    pub cycle_count: u32,

    /// Number of registered agents
    pub agent_count: usize,

    /// Per-agent timing breakdown
    pub agent_timings: Vec<AgentTiming>,

    /// HITL gate events (if any)
    pub hitl_events: Vec<HitlTiming>,

    /// Final outcome
    pub outcome: RunOutcome,

    /// Facts at convergence
    pub final_fact_count: usize,

    /// Invariant violations encountered
    pub invariant_violations: u32,

    /// Budget usage
    pub budget: BudgetUsage,

    /// Tenant/customer context (for pilot filtering)
    pub tenant_id: Option<String>,

    /// Job type label (for metric grouping)
    pub job_type: Option<String>,
}
```

### 4.2 AgentTiming

```rust
/// Per-agent execution timing across all cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTiming {
    /// Agent identifier (name, not internal ID)
    pub agent_name: String,

    /// Total time spent in execute() across all cycles
    pub total_execution_ms: u64,

    /// Number of cycles this agent was eligible and ran
    pub executions: u32,

    /// Number of facts this agent produced
    pub facts_produced: u32,

    /// Backend usage (if agent invokes LLM)
    pub backend_usage: Option<BackendUsageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendUsageSummary {
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
    pub total_latency_ms: u64,
    pub total_cost_microdollars: Option<u64>,
    pub invocation_count: u32,
}
```

### 4.3 HitlTiming

```rust
/// Timing for a single HITL gate pause/resume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitlTiming {
    pub gate_id: String,
    pub agent_name: String,
    pub cycle: u32,
    pub requested_at: String,
    pub decided_at: Option<String>,
    /// Wall-clock wait in milliseconds (requested_at → decided_at)
    pub wait_ms: Option<u64>,
    pub verdict: String,  // "approved" | "rejected" | "timed_out"
}
```

### 4.4 RunOutcome

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunOutcome {
    /// Reached fixed point — convergence succeeded
    Converged,
    /// Budget exhausted before convergence
    BudgetExhausted { cycles_used: u32, max_cycles: u32 },
    /// Invariant violation halted execution
    InvariantViolation { violation: String },
    /// HITL gate rejection halted execution
    HitlRejected { gate_id: String, reason: String },
    /// Engine error
    Error { message: String },
}
```

### 4.5 BudgetUsage

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub cycles_used: u32,
    pub cycles_max: u32,
    pub facts_produced: usize,
    pub facts_max: Option<usize>,
}
```

## 5. TelemetryCollector Trait

```rust
/// Trait for collecting telemetry during convergence runs.
/// Implementors receive events at key engine lifecycle points.
/// All methods have default no-op implementations for zero-cost
/// when telemetry is disabled.
#[cfg(feature = "telemetry")]
pub trait TelemetryCollector: Send + Sync {
    fn on_run_start(&self, run_id: &str, agent_count: usize) {}
    fn on_cycle_start(&self, cycle: u32) {}
    fn on_agent_start(&self, cycle: u32, agent_name: &str) {}
    fn on_agent_end(&self, cycle: u32, agent_name: &str, duration_ms: u64, facts_produced: u32) {}
    fn on_hitl_start(&self, gate_id: &str, agent_name: &str, cycle: u32) {}
    fn on_hitl_end(&self, gate_id: &str, verdict: &str, wait_ms: u64) {}
    fn on_backend_usage(&self, agent_name: &str, usage: &BackendUsage) {}
    fn on_invariant_violation(&self, cycle: u32, violation: &str) {}
    fn on_cycle_end(&self, cycle: u32, facts_added: usize) {}
    fn on_run_end(&self, report: RunTelemetryReport) {}
}
```

## 6. Export Formats

### 6.1 MVP: JSON Lines (file-based)

One `RunTelemetryReport` per line, appended to:

```
pilot-data/{tenant-id}/telemetry.jsonl
```

This aligns with the Pilot Metrics Framework data storage spec (Section 4.3).

### 6.2 MVP: Prometheus Exposition Format (optional)

Map to the metrics defined in PILOT_METRICS_FRAMEWORK.md Section 4.1:

```
converge_run_duration_seconds{customer="anon-001", job_type="lead_to_cash"} 12.4
converge_run_cycles_total{customer="anon-001", job_type="lead_to_cash"} 7
converge_invariant_violations_total{customer="anon-001", job_type="lead_to_cash", severity="warning"} 0
converge_agent_execution_seconds{customer="anon-001", agent_type="pricing_agent"} 3.2
converge_convergence_success{customer="anon-001", job_type="lead_to_cash"} 1
```

### 6.3 In-Memory Buffer (for tests)

An `InMemoryTelemetryCollector` that stores `Vec<RunTelemetryReport>` for assertions in integration tests. This is the primary test implementation.

## 7. Engine Integration — Minimal Diff

The key change to `engine.rs` is wrapping the agent execution loop:

```rust
// Current (engine.rs ~line 303):
for (agent_id, agent) in eligible_agents {
    let effect = agent.execute(&ctx);
    effects.push((agent_id, effect));
}

// Proposed (behind #[cfg(feature = "telemetry")]):
for (agent_id, agent) in eligible_agents {
    #[cfg(feature = "telemetry")]
    if let Some(ref collector) = self.telemetry {
        collector.on_agent_start(cycles, agent.name());
    }

    let start = Instant::now();
    let effect = agent.execute(&ctx);
    let duration_ms = start.elapsed().as_millis() as u64;

    #[cfg(feature = "telemetry")]
    if let Some(ref collector) = self.telemetry {
        let facts = effect.proposed_facts_count();
        collector.on_agent_end(cycles, agent.name(), duration_ms, facts);
    }

    effects.push((agent_id, effect));
}
```

Similarly for HITL gate in `gates/hitl.rs`, wrap the gate request/decision flow.

## 8. HITL Wait Time Capture

The HITL gate already records `requested_at` and `decided_at` timestamps in `GateEvent`. The telemetry collector hooks into these:

1. `HitlGate::request()` → emit `on_hitl_start(gate_id, agent_name, cycle)`
2. `HitlGate::decide()` → compute `wait_ms = decided_at - requested_at`, emit `on_hitl_end(gate_id, verdict, wait_ms)`

No new data structures needed — just wiring existing timestamps to the collector.

## 9. Feature Flag Strategy

```toml
# converge-core/Cargo.toml
[features]
default = []
telemetry = ["serde", "serde_json"]  # only serde needed for JSON export
```

When `telemetry` is disabled:
- `TelemetryCollector` trait doesn't exist
- All hook call sites compile to nothing (`#[cfg(feature = "telemetry")]`)
- Zero runtime overhead

When enabled:
- One `Instant::now()` + `elapsed()` per agent execution (nanosecond-level overhead)
- One JSON serialization per completed run (microsecond-level)

## 10. Relationship to Existing Systems

| Existing System | Relationship |
|-----------------|-------------|
| `StreamingCallback` | Telemetry collector is orthogonal. Callbacks are for UI/progress display; telemetry is for measurement. Both can coexist. |
| `ExperienceStore` | Telemetry events can optionally be stored as `ExperienceEventKind::TelemetryReportGenerated` for audit trail. The store is the source of truth for `OutcomeRecorded` events that feed backend usage metrics. |
| `BackendUsage` | Already captured per-response. Telemetry aggregates these per-agent across a run. |
| `tracing` spans | Debug-level instrumentation. Telemetry is structured, queryable production data. Complementary. |
| Pilot Metrics Framework | Telemetry exporter produces the raw data. The weekly aggregation script consumes it. |

## 11. Migration Path

| Phase | Scope | Timeline |
|-------|-------|----------|
| **v1 (this design)** | `RunTelemetryReport` + JSON Lines export. Per-agent timing, HITL wait, cycle count, outcome. Feature-gated in converge-core. | cw-4 implementation |
| **v1.1** | Prometheus exposition format. Optional HTTP `/metrics` endpoint. | cw-5 |
| **v2** | Extract to `converge-telemetry` crate. Add streaming export (not just end-of-run). Real-time dashboard feed. | Post-Wave 3 |

## 12. Open Questions

1. **Should telemetry include context snapshot at run start?** Including initial fact keys would help debug slow convergence but increases data volume. **Recommendation:** Include key count and names only, not values.

2. **Async agent execution timing.** If agents become async (futures), `Instant::now()` still works but wall-clock time includes await points. **Recommendation:** Track both wall-clock and CPU time when async lands.

3. **Multi-tenant isolation.** Each `RunTelemetryReport` carries `tenant_id`. File-based export segregates by directory. **Recommendation:** Enforce tenant isolation at the file path level (`pilot-data/{tenant-id}/`), same as Pilot Metrics Framework.

## 13. Acceptance Criteria

- [ ] `TelemetryCollector` trait defined in converge-core behind `telemetry` feature flag
- [ ] `RunTelemetryReport` struct captures all P0 signals (cycle count, convergence time, per-agent time, HITL wait, final state)
- [ ] Engine wraps `Agent::execute()` with timing instrumentation
- [ ] HITL gate emits wait time events to collector
- [ ] `InMemoryTelemetryCollector` exists for testing
- [ ] JSON Lines file exporter writes to `pilot-data/{tenant-id}/telemetry.jsonl`
- [ ] Feature flag compiles cleanly when disabled (zero overhead verified)
- [ ] At least 3 integration tests verify telemetry output correctness

---

**Reviewer:** Ren Akiyama (VP Engineering), Eli Marsh (engine owner)
