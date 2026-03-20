# Task: converge-application — Reference App and JTBD Builder

## Wave

- **Wave:** 5 — Experience
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 006 (runtime), 008 (tool), 009 (domain)

## Branch

- **Repo:** `converge-application`
- **Branch:** `feature/reference-app`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "reference-app"
jj bookmark create feature/reference-app -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** Reference implementation + low-friction JTBD builder. Shows the end-to-end story. Includes example specs, pre-compiled WASM modules, local marketplace to browse converge-domain modules. Svelte + TypeScript frontend (SSE/WS, no SSR).
- **Existing code:** CLI/TUI with Ratatui, packs, domain agent wiring
- **Frontend stack:** Svelte, TypeScript, SSE, WebSockets — no SSR, no runtime frameworks
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

converge-application needs to transform from a standalone CLI/TUI app into a reference application showing the full end-to-end flow, plus a Svelte frontend for browsing/building.

## Goal

A new user can: browse modules, pick them, write a spec, compile, deploy, and see results — both via web UI and CLI.

## Scope

- [ ] Example JTBD specs (growth-strategy, patent-research, etc.)
- [ ] Pre-compiled example WASM modules
- [ ] Svelte + TypeScript frontend skeleton (SSE/WS connection to runtime)
- [ ] Local marketplace view (browse converge-domain modules)
- [ ] TUI refactored as access path into runtime (gRPC client)
- [ ] CLI refactored as access path into runtime
- [ ] Justfile with standard targets (+ frontend targets: dev, build-web)
- **Out of scope:**
  - Do not build production deployment tooling
  - Do not implement billing UI

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | Rust deps TBD based on access path refactor |

Frontend (package.json):
| Package | Justification |
|---------|---------------|
| svelte | Frontend framework |
| typescript | Type safety |
| vite | Build tool |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ 3 new | CLI command parsing, spec loading |
| Integration tests | ≥ 1 new | End-to-end: spec → compile → run → result |

### Code Quality

- [ ] `cargo clippy` — zero warnings (Rust)
- [ ] `cargo fmt` — clean (Rust)
- [ ] Svelte frontend builds without errors
- [ ] Example specs compile to WASM

## Status: draft

## Log

- **2026-03-11** — Created task
