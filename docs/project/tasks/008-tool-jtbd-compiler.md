# Task: converge-tool — JTBD Spec Format and Compilation Pipeline

## Wave

- **Wave:** 3 — Tooling
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 001 (converge-core), 002 (converge-traits)

## Branch

- **Repo:** `converge-tool`
- **Branch:** `feature/jtbd-compiler`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "jtbd-compiler"
jj bookmark create feature/jtbd-compiler -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** The compiler toolchain. JTBD specs (Gherkin/Truths) compile to WASM modules. Validates specs, resolves module dependencies, generates Rust, compiles to wasm32-wasi. Also provides Gherkin validation (already exists).
- **Existing code:** Gherkin validation already in place
- **Flow:** `.jtbd` spec → parse → resolve modules → generate Rust → compile → `.wasm`
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

The compilation pipeline from JTBD specs to WASM does not exist yet. The Gherkin validator exists but the full flow (parse → resolve → generate → compile) needs to be built.

## Goal

`converge-tool compile jobs/my-job.jtbd` produces a `.wasm` module that converge-runtime can load.

## Scope

- [ ] Define JTBD spec format (.jtbd — extended Gherkin with module references)
- [ ] Parser: spec → intermediate representation
- [ ] Resolver: map spec requirements to converge-domain modules
- [ ] Code generator: IR + modules → Rust source
- [ ] Compiler: Rust source → wasm32-wasi
- [ ] CLI: `converge-tool compile <spec>`
- [ ] At least one example spec that compiles end-to-end
- [ ] Justfile with standard targets
- **Out of scope:**
  - Do not build the runtime module loader (that's converge-runtime)
  - Do not build the marketplace (that's converge-application)

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| wasmtime | latest | no | WASM compilation target |
| syn/quote | latest | no | Rust code generation |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ 5 new | Parser, resolver, codegen |
| Integration tests | ≥ 1 new | End-to-end: .jtbd → .wasm |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] Example spec compiles successfully

## Status: draft

## Log

- **2026-03-11** — Created task
