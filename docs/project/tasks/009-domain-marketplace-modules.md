# Task: converge-domain — Reusable Marketplace Modules

## Wave

- **Wave:** 3 — Tooling
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 001 (converge-core), 002 (converge-traits)

## Branch

- **Repo:** `converge-domain`
- **Branch:** `feature/marketplace-modules`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "marketplace-modules"
jj bookmark create feature/marketplace-modules -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** Standard library of convergent building blocks. Reusable modules that me and customers compose into JTBD applications. Each module is a tested, versioned agent implementation with marketplace metadata.
- **Existing code:** MarketSignalAgent, CompetitorAgent, StrategyAgent, EvaluationAgent, etc.
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

Existing domain agents need to be framed as reusable marketplace modules with metadata, documentation, and example JTBD usage. Each should be composable by converge-tool.

## Goal

At least 5 modules exist, each with: trait impl, tests, example JTBD usage, marketplace metadata (name, description, category, dependencies).

## Scope

- [ ] Define module metadata format (name, description, category, inputs, outputs)
- [ ] Migrate existing agents to module format
- [ ] Each module has tests proving convergence behavior
- [ ] Each module has example JTBD spec showing usage
- [ ] Module registry/index for discovery
- [ ] Justfile with standard targets
- **Out of scope:**
  - Do not build the marketplace UI (that's converge-application)
  - Do not modify converge-core or converge-traits

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | Should use existing converge-core + converge-provider |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ 5 new | One per module, proving convergence |
| Property tests | ≥ 2 new | Module composition invariants |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] Each module has doc comments + metadata

## Status: draft

## Log

- **2026-03-11** — Created task
