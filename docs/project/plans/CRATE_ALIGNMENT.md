# Crate Alignment Plan

> Bring every converge-* crate into alignment with the platform architecture.
> Each task is independent and can be executed by a separate agent in parallel.

Reference: `converge-business/docs/PLATFORM_ARCHITECTURE.md`

## Prerequisites

Before parallel work begins:

- [ ] converge-traits and converge-core are stable and published
- [ ] Justfile template exists (copy from converge-core as baseline)
- [ ] jj initialized in all repos

---

## 1. converge-core — The Proof

**Goal:** 5-6 concepts, exhaustive tests, undeniable clarity.

- [ ] Identify and name the 5-6 core concepts (Context, Agent, Proposal, Invariant, Convergence, Budget?)
- [ ] Write mock-agent examples: mock LLM, mock optimizer, mock Cedar policy, mock gate, root intent
- [ ] Each example converges to a predefined truth — the test asserts the truth
- [ ] Add tunable examples showing resource requirements (cycles, facts, time)
- [ ] Ensure MCP connector works for interactive exploration from Claude Desktop
- [ ] Storytelling: each concept gets a one-paragraph explanation a business person can follow
- [ ] Property-based tests with proptest for all core invariants

**Done when:** A new developer reads the examples and understands convergence in 15 minutes.

## 2. converge-traits — The Contract

**Goal:** Minimal, stable trait definitions.

- [ ] Audit: only traits that agents need to implement belong here
- [ ] No implementation code, no dependencies beyond std + serde
- [ ] Document each trait with "why" not just "what"
- [ ] Ensure converge-core, all instantiation crates, and converge-experience compile against it
- [ ] Justfile with: check, test, clippy, fmt, doc, audit

**Done when:** Traits are frozen for 1.0. Breaking changes require RFC.

## 3. converge-provider — Remote LLM Agents

**Goal:** Clean implementation of converge-traits for remote LLM providers.

- [ ] Depends only on converge-traits (+ converge-core for testing)
- [ ] Each provider is feature-gated
- [ ] Tests use mock HTTP (wiremock) — no live API keys needed for CI
- [ ] Justfile aligned to converge-core targets
- [ ] Examples showing a provider agent participating in convergence

**Done when:** `cargo test` passes with no API keys. At least Anthropic + OpenAI work.

## 4. converge-llm — Near/Local LLM Agents

**Goal:** On-device / edge LLM inference as converge agents.

- [ ] Depends on converge-traits
- [ ] Burn or similar for inference
- [ ] Feature-gated model backends
- [ ] Example: local model agent converging with a mock policy agent
- [ ] Justfile aligned

**Done when:** A small local model can participate in a convergence run.

## 5. converge-analytics — Near ML/Analytics Agents

**Goal:** ML and analytics as converge agents.

- [ ] Depends on converge-traits
- [ ] Burn for ML, LanceDB for vector search
- [ ] Feature-gated backends
- [ ] Example: embedding similarity agent contributing facts to context
- [ ] Justfile aligned

**Done when:** An analytics agent can propose facts based on vector similarity.

## 6. converge-policy — Cedar Policy Agents

**Goal:** Policy evaluation as converge agents.

- [ ] Depends on converge-traits
- [ ] Cedar SDK integration
- [ ] Policies expressed as Cedar, evaluated as agent proposals
- [ ] Example: policy agent that gates convergence on compliance rules
- [ ] Justfile aligned

**Done when:** A Cedar policy can participate in convergence as a first-class agent.

## 7. converge-optimization — CP-SAT / Constraint Agents

**Goal:** Constraint solving and optimization as converge agents.

- [ ] Depends on converge-traits
- [ ] Polar for authorization logic, CP-SAT for constraint solving
- [ ] Example: optimization agent that proposes resource allocation facts
- [ ] Justfile aligned

**Done when:** A constraint solver can propose optimal configurations during convergence.

## 8. converge-tool — The Compiler Toolchain

**Goal:** JTBD specs (Gherkin/Truths) compile to WASM modules.

- [ ] Gherkin validation (already exists — verify and extend)
- [ ] Define JTBD spec format (.jtbd or extended .feature)
- [ ] Parser: spec -> intermediate representation
- [ ] Resolver: map spec requirements to converge-domain modules
- [ ] Compiler: IR + modules -> Rust -> wasm32-wasi
- [ ] CLI: `converge-tool compile jobs/my-job.jtbd -> my-job.wasm`
- [ ] Justfile aligned

**Done when:** A .jtbd file compiles to a .wasm module that converge-runtime can load.

## 9. converge-domain — Reusable Modules / Marketplace

**Goal:** Standard library of convergent building blocks.

- [ ] Each module is a tested, versioned agent implementation
- [ ] Modules are composable — can be wired together by converge-tool
- [ ] Existing agents (MarketSignal, Competitor, Strategy, etc.) migrated here
- [ ] Each module has: trait impl, tests, example JTBD usage, documentation
- [ ] Marketplace metadata: name, description, category, dependencies
- [ ] Justfile aligned

**Done when:** At least 5 modules exist, each usable from a .jtbd spec.

## 10. converge-experience — The Memory

**Goal:** Event-sourced audit trail, production-ready.

- [ ] InMemoryExperienceStore (already exists — verify)
- [ ] SurrealDbExperienceStore (already exists — verify and harden)
- [ ] Add LanceDB store for vector-indexed experience retrieval
- [ ] Property-based tests for store implementations
- [ ] Justfile aligned

**Done when:** Both stores pass comprehensive tests. LanceDB store is prototyped.

## 11. converge-runtime — The Hosting Substrate

**Goal:** Production-ready WASM host with dynamic module loading.

- [ ] WASM host loads/unloads modules dynamically (already started)
- [ ] Module registry with versioning
- [ ] Context ledger wired to converge-experience stores
- [ ] Protocol surface: REST, gRPC, SSE (already exists — verify)
- [ ] Auth, billing, observability (already exists — verify)
- [ ] Remove any domain-specific logic — domain arrives as WASM only
- [ ] Templates migrate to converge-tool as JTBD spec examples
- [ ] Justfile aligned

**Done when:** A .wasm module compiled by converge-tool loads and runs in converge-runtime.

## 12. converge-remote — gRPC Access Path

**Goal:** Clean gRPC client for converge-runtime.

- [ ] Run, watch, inject, approve/reject, pause/resume (already exists)
- [ ] Verify proto definitions match converge-runtime
- [ ] Integration tests against runtime (feature-gated)
- [ ] Justfile aligned

**Done when:** All commands work against a running converge-runtime.

## 13. converge-application — Reference App / JTBD Builder

**Goal:** Show the end-to-end story. Low-friction entry point.

- [ ] Example JTBD specs (growth-strategy, patent-research, etc.)
- [ ] Pre-compiled example WASM modules
- [ ] Local marketplace UI to browse converge-domain modules
- [ ] Svelte + TypeScript frontend (SSE/WS, no SSR)
- [ ] TUI as alternative access path into runtime (not standalone app)
- [ ] CLI as alternative access path
- [ ] Justfile aligned

**Done when:** A new user can browse modules, pick them, write a spec, compile, deploy, and see results.

## 14. converge-business — The Story

**Goal:** Single source of truth, always telling the story right.

- [ ] Platform architecture doc (done: `docs/PLATFORM_ARCHITECTURE.md`)
- [ ] Verify all strategy, packs, GTM docs are current
- [ ] Cross-reference: every crate's README points here for "the why"
- [ ] Reading order for business owners, developers, platform builders (exists — verify)

**Done when:** A stranger reads this repo and understands why Converge matters.

## 15. converge-personas — Business-Level Evals

**Goal:** Evaluate convergence through different user persona lenses.

- [ ] Define key personas (SMB owner, developer, platform builder, compliance officer)
- [ ] Eval scenarios per persona
- [ ] Scoring rubrics for convergence quality from each perspective
- [ ] Lives near converge-business conceptually

**Done when:** Each persona has at least 3 eval scenarios with pass/fail criteria.

---

## Execution Strategy

These tasks are **independent by design**. The dependency graph means:

1. **Wave 1** (foundation): converge-traits, converge-core, converge-business
2. **Wave 2** (instantiation): converge-provider, converge-llm, converge-analytics, converge-policy, converge-optimization — all in parallel
3. **Wave 3** (tooling): converge-tool, converge-domain, converge-experience — all in parallel
4. **Wave 4** (infrastructure): converge-runtime
5. **Wave 5** (experience): converge-remote, converge-application, converge-personas

Within each wave, every task can run as an independent agent.

## End-to-End Milestone

The platform is ready when this flow works:

```
1. User browses converge-domain modules in converge-application
2. User writes a .jtbd spec referencing chosen modules
3. converge-tool compiles spec -> .wasm
4. .wasm deploys to converge-runtime
5. User accesses via Svelte UI / converge-remote / CLI
6. Convergence runs, traced by converge-experience
7. Result: converged state, auditable, replayable
```
