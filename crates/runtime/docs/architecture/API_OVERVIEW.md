# Converge Core — API Overview

This document provides a high-level overview of the Converge Core public API.

## Core Types

### Engine

The `Engine` is the convergence runtime that coordinates agent execution.

**Key Methods:**
- `new()` — Create a new engine
- `register(agent)` — Register an agent
- `run(context)` — Execute until convergence

**Properties:**
- Deterministic execution
- Parallel agent execution with serialized effect merging
- Automatic convergence detection
- Budget enforcement for termination

### Context

The `Context` is the shared, typed state visible to all agents.

**Key Methods:**
- `new()` — Create empty context
- `has(key)` — Check if a context key exists
- `get(key)` — Retrieve facts for a key

**Properties:**
- Append-only in meaning
- Typed facts
- Full provenance tracking
- Monotonic evolution

### Agent

The `Agent` trait defines the interface for implementing capabilities.

**Required Methods:**
- `accepts(ctx)` — Determine if agent should run
- `dependencies()` — Declare context dependencies
- `execute(ctx)` — Produce effects

**Constraints:**
- Agents never call other agents
- Agents never mutate context directly
- Agents only emit effects

### AgentEffect

Buffered output from an agent execution.

**Components:**
- Facts — New facts to add to context
- Traces — Execution traces for observability

**Properties:**
- Effects are buffered, not immediately applied
- Merged deterministically by the engine
- Subject to validation and conflict detection

## Execution Model

### High-Level Flow

1. **Initialization** — Context created from Root Intent
2. **Eligibility** — Agents declare dependencies, engine determines eligibility
3. **Execution** — Eligible agents run in parallel
4. **Merge** — Effects merged serially into context
5. **Convergence Check** — If no new facts, execution halts

### Guarantees

- **Determinism**: Same input always produces same output
- **Termination**: Budgets prevent infinite loops
- **Isolation**: Agents cannot affect each other's execution
- **Auditability**: All changes are traceable with provenance

## Error Handling

All operations return `Result` types. Errors are domain-specific and use `thiserror` for structured error handling.

**Error Categories:**
- Validation errors — Invalid facts or effects
- Budget errors — Execution limits exceeded
- Invariant violations — Business rules violated

## Observability

The library uses `tracing` for structured logging. All operations emit spans and events that can be captured by your tracing subscriber.

