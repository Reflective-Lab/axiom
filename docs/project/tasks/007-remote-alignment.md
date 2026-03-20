# Task: converge-remote — Verify as Access Path

## Wave

- **Wave:** 5 — Experience
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 006 (converge-runtime with module registry)

## Branch

- **Repo:** `converge-remote`
- **Branch:** `feature/access-path-alignment`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "access-path-alignment"
jj bookmark create feature/access-path-alignment -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** gRPC access path into converge-runtime. Run jobs, watch, inject facts, approve/reject proposals, pause/resume. This is NOT an application — it is one way to talk to the backend.
- **Existing code:** Full gRPC client with run/watch/inject/approve/reject/pause/resume/cancel/capabilities commands
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

converge-remote is well-built but needs verification against the current runtime API and alignment with the "access path, not application" framing.

## Goal

Verified gRPC client that works against current converge-runtime. Proto definitions match. Integration tests pass (feature-gated).

## Scope

- [ ] Verify proto definitions match converge-runtime
- [ ] Verify all commands work (run, watch, inject, approve, reject, pause, resume)
- [ ] Add Justfile with standard targets
- [ ] Update README to frame as "access path" not "application"
- **Out of scope:**
  - Do not add new commands
  - Do not modify converge-runtime proto

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | verify existing | Proto serialization, command parsing |
| Integration tests | ≥ 1 | Against running runtime (feature-gated) |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] `cargo test` passes without running runtime

## Status: draft

## Log

- **2026-03-11** — Created task
