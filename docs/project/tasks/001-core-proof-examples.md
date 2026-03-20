# Task: converge-core — The Proof Examples

## Wave

- **Wave:** 1 — Foundation
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** none (this is the starting point)

## Branch

- **Repo:** `converge-core`
- **Branch:** `feature/proof-examples`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "proof-examples"
jj branch create feature/proof-examples
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** The proof that convergence works. Pure, no I/O, no async, no persistence. Implements the engine, context, agents, invariants, and convergence loop. Every other crate builds on this.
- **Key types:** `Context`, `Agent`, `Proposal`, `Fact`, `Invariant`, `Budget`, `Engine`, `RootIntent`
- **Reference files:**
  - `converge-core/src/engine.rs` — the convergence loop
  - `converge-core/src/context.rs` — shared state
  - `converge-core/src/agent.rs` — agent trait
  - `converge-core/src/invariant.rs` — invariant enforcement
  - `converge-core/src/types.rs` — Observation → Proposal → Fact hierarchy
  - `converge-core/src/gates.rs` — promotion gates
  - `converge-core/src/root_intent.rs` — intent declaration
- **Justfile:** Already exists with: build, test, lint, publish

## Problem

converge-core has the engine and types but lacks clear, self-contained examples that prove convergence to a predefined truth. A new developer cannot currently read a single file and understand the 5-6 core concepts. The storytelling is scattered across module docs.

## Goal

A set of example programs (in `examples/`) and supporting tests that each demonstrate one core concept converging to a known truth. A developer reads these in order and understands convergence in 15 minutes. A business person reads the one-paragraph descriptions and gets the intuition.

### The 5-6 Core Concepts (to name and prove)

1. **Context** — The shared, typed, evolving state. All agents read from and write to context. There is no other communication channel.
2. **Agent** — A function from context to proposals. Agents never call each other. They only see context.
3. **Convergence** — Agents run in cycles. When no agent has new proposals, the system has converged. The fixed point is the answer.
4. **Invariant** — A rule that must hold at all times. If a proposal violates an invariant, it is rejected. The system never enters an invalid state.
5. **Budget** — Convergence must terminate. Budget caps cycles, facts, time, or cost. Exhausting budget is a valid outcome, not a failure.
6. **Root Intent** — The starting truth. Seeds the context with what we know. Everything converges relative to this.

## Scope

- [ ] `examples/01_context.rs` — Two agents, shared context, facts accumulate
- [ ] `examples/02_convergence.rs` — Agents converge to a known fixed point (assert the truth)
- [ ] `examples/03_invariant.rs` — Agent proposes a violating fact, invariant rejects it, system still converges correctly
- [ ] `examples/04_budget.rs` — Tunable budget, show resource requirements for convergence
- [ ] `examples/05_root_intent.rs` — Different root intents lead to different converged states
- [ ] `examples/06_mock_agents.rs` — Mock LLM + mock policy + mock optimizer, all converging together
- [ ] `examples/README.md` — Reading guide: concept name, one-paragraph explanation, link to example
- [ ] Tests mirroring each example as assertions (in `tests/` or inline)
- **Out of scope:**
  - Do not modify other converge-* crates
  - Do not change existing public API
  - Do not add I/O, async, or network dependencies
  - Do not modify converge-mcp or experience-store

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | No new deps expected — examples use only converge-core |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ 6 new | One per core concept, asserting convergence to known truth |
| Negative tests | ≥ 3 new | Invariant violation, budget exhaustion, empty context |
| Property tests | ≥ 3 new | "Any agent set with these invariants converges within budget" |
| Integration tests | 0 | Pure crate, no external dependencies |

### Performance

| Metric | Baseline | Target |
|--------|----------|--------|
| Example 02 convergence | — | < 10ms for 3 agents, 5 cycles |
| Example 04 budget sweep | — | Linear scaling with budget size |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] No new `unwrap()` outside tests
- [ ] No new `todo!()` or `unimplemented!()`
- [ ] Public API has doc comments
- [ ] No increase in unsafe blocks
- [ ] Each example runs standalone: `cargo run --example 01_context`
- [ ] Each example prints a human-readable narrative, not just data

## Constraints

- Do not change public API signatures
- Do not add new dependencies
- Preserve existing test coverage — no deletions
- Examples must be pure Rust, no macros that hide the concepts
- Examples should be readable top-to-bottom, no jumping between files

## Workflow

1. Create branch from main
2. Read existing engine.rs, context.rs, agent.rs, types.rs to understand current API
3. Implement examples in order (01 through 06)
4. Write matching tests
5. Write examples/README.md with the 6 concepts
6. Run full check before marking done:
   ```sh
   just check
   ```
7. Squash or rebase as needed:
   ```sh
   jj squash
   ```
8. Mark task status: `draft` → `active` → `review` → `done`

## Status: draft

## Log

- **2026-03-11** — Created task from CRATE_ALIGNMENT.md Wave 1
