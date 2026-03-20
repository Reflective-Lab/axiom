# Task: converge-traits — Audit and Stabilize for 1.0

## Wave

- **Wave:** 1 — Foundation
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** none (foundational, parallel with 001)

## Branch

- **Repo:** `converge-traits`
- **Branch:** `feature/traits-1.0-audit`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "traits-1.0-audit"
jj bookmark create feature/traits-1.0-audit -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** The contract that agents implement. Pure trait definitions. No implementation code, no I/O, no async. Every instantiation crate (provider, llm, analytics, policy, optimization) depends on this.
- **Key traits to audit:** Agent-facing traits that converge-provider, converge-llm, converge-analytics, converge-policy, and converge-optimization will implement
- **Reference:** `converge-core` uses these traits — check what it imports from `converge-traits`
- **Justfile:** Ensure Justfile exists with at least the targets from `converge-project/templates/Justfile.template`

## Problem

converge-traits is the dependency root for all instantiation crates. Before Wave 2 can start (5 crates in parallel), traits must be audited, documented, and declared stable. Currently unclear which traits belong here vs in converge-core.

## Goal

A minimal, stable set of trait definitions that:
1. Every instantiation crate can implement without pulling in converge-core
2. Each trait has "why" documentation, not just "what"
3. No implementation code leaks in
4. Dependencies are minimal (std + serde only)
5. Ready to declare 1.0 — breaking changes require RFC after this

## Scope

- [ ] Audit all trait definitions — identify which belong here vs converge-core
- [ ] Document each trait with "why" (not just "what")
- [ ] Verify no implementation code exists (only trait defs + associated types)
- [ ] Verify dependencies are minimal (std, serde — nothing else)
- [ ] Ensure converge-core compiles against it
- [ ] Ensure Justfile has: check, test, lint, fmt, doc, audit
- [ ] Add property tests for any associated type constraints
- **Out of scope:**
  - Do not modify other converge-* crates
  - Do not add implementation code
  - Do not add I/O, async, or network dependencies

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | Should need nothing beyond std + serde |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | verify existing | Trait-level compile tests |
| Property tests | ≥ 2 new | Associated type constraints |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] No implementation code (only traits, associated types, marker types)
- [ ] Every public trait has doc comment explaining "why"
- [ ] Dependencies: only std + serde

## Constraints

- Traits must be implementable without converge-core
- No breaking changes to traits already implemented by converge-core
- Preserve existing test coverage

## Workflow

1. Read converge-core's Cargo.toml to see what it imports from converge-traits
2. Read all trait definitions in converge-traits
3. Audit: does each trait belong here or in converge-core?
4. Add "why" documentation to each trait
5. Verify minimal dependencies
6. Run `just check`
7. Mark done

## Status: in-progress

## Log

- **2026-03-11** — Created task
- **2026-03-11** — Audit complete. Redesigned from LLM-only to pure generic architecture. v0.2.0 written: Backend trait, BackendKind (7 variants), Capability (30+ variants), BackendError/BackendErrorKind, BackendRequirements + BackendSelector, CostClass (with Free tier), DataSovereignty, ComplianceLevel. Old LLM types (LlmProvider, LlmRequest, LlmResponse) removed — they stay in converge-core's own modules which already superseded them. Deps reduced from 6 (serde, thiserror, uuid, chrono, async-trait, futures) to 2 (serde, thiserror). Tests written for all backend kinds, capability matching, error classification, serialization. Pending: delete old llm.rs/selection.rs files, add Justfile, cargo test, update converge-core dependency.
