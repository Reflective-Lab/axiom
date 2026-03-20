# Task: converge-experience — Harden and Add LanceDB

## Wave

- **Wave:** 3 — Tooling
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 001 (converge-core stable), 002 (converge-traits stable)

## Branch

- **Repo:** `converge-experience`
- **Branch:** `feature/harden-and-lancedb`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "harden-and-lancedb"
jj bookmark create feature/harden-and-lancedb -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** The memory of the platform. Event-sourced audit trail implementing ExperienceStore trait from converge-core. InMemory (dev/test) + SurrealDB (production). Adding LanceDB for vector-indexed experience retrieval.
- **Existing code:** InMemoryExperienceStore and SurrealDbExperienceStore already exist and work
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

The existing stores work but lack comprehensive property-based tests. LanceDB store is needed for vector-indexed retrieval (part of target stack). Justfile missing.

## Goal

Production-hardened experience stores with property tests. LanceDB prototype for vector retrieval.

## Scope

- [ ] Property-based tests for InMemoryExperienceStore
- [ ] Property-based tests for SurrealDbExperienceStore
- [ ] Prototype LanceDB store (feature-gated: `lancedb`)
- [ ] Justfile with standard targets
- [ ] Verify SurrealDB schema is correct and indexed
- **Out of scope:**
  - Do not modify converge-core's ExperienceStore trait
  - Do not add other storage backends

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| lancedb | latest | yes: `lancedb` | Vector-indexed experience retrieval |
| proptest | 1.4 | dev-dependency | Property-based testing |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Property tests | ≥ 5 new | append/query roundtrip, ordering, filtering |
| Unit tests | ≥ 3 new | LanceDB store basics |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] `cargo test` passes (InMemory always, SurrealDB/LanceDB behind features)

## Status: draft

## Log

- **2026-03-11** — Created task
