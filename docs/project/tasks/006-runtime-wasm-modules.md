# Task: converge-runtime — WASM Module Registry and Domain Extraction

## Wave

- **Wave:** 4 — Infrastructure
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 008 (converge-tool), 009 (converge-domain)

## Branch

- **Repo:** `converge-runtime`
- **Branch:** `feature/wasm-module-registry`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "wasm-module-registry"
jj bookmark create feature/wasm-module-registry -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** The hosting substrate. Loads WASM modules dynamically, provides protocol surface (REST/gRPC/SSE), auth, billing, observability. The runtime owns NO domain logic — domain arrives as WASM modules.
- **WASM code already exists:** `src/wasm/` has contract, adapter, engine, host, integration, store
- **Templates to migrate:** `templates/*.yaml` should move to converge-tool as JTBD spec examples
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

Runtime currently has domain-specific templates and some hardcoded domain logic. It needs a proper module registry for dynamic WASM loading/unloading/versioning. Templates should migrate to converge-tool.

## Goal

A clean WASM module registry where compiled `.wasm` modules from converge-tool are loaded, versioned, and executed. No domain logic in the runtime.

## Scope

- [ ] Module registry: load/unload/version WASM modules
- [ ] Verify WASM host↔guest ABI is sufficient for converge-domain modules
- [ ] Migrate templates to converge-tool (or flag for migration)
- [ ] Remove any domain-specific logic from runtime
- [ ] Wire module registry to existing protocol surface
- [ ] Justfile with standard targets
- **Out of scope:**
  - Do not modify WASM ABI (yet)
  - Do not rewrite existing HTTP/gRPC handlers
  - Do not touch auth/billing/observability

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | Existing WASM deps should suffice |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ 3 new | Module registry load/unload/version |
| Integration tests | ≥ 1 new | Load .wasm, run convergence, get result |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] No domain-specific logic remains in runtime

## Status: draft

## Log

- **2026-03-11** — Created task
