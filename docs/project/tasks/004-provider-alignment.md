# Task: converge-provider — Align to Architecture

## Wave

- **Wave:** 2 — Instantiation
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** 002 (converge-traits stable)

## Branch

- **Repo:** `converge-provider`
- **Branch:** `feature/architecture-alignment`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "architecture-alignment"
jj bookmark create feature/architecture-alignment -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** Remote LLM agent instantiation. Implements converge-traits for Anthropic, OpenAI, Gemini, Perplexity, etc. Each provider is feature-gated. Tests use mock HTTP (wiremock) — no live API keys for CI.
- **Key traits to implement:** From converge-traits — the agent instantiation trait
- **Reference:** converge-core examples (01-08) show what agents look like
- **Justfile:** Copy from `converge-project/templates/Justfile.template`

## Problem

converge-provider needs to implement converge-traits cleanly, with each provider feature-gated, testable without API keys, and documented as "one way to instantiate an agent."

## Goal

Clean provider implementations that a customer can depend on to wire remote LLMs into their convergence runs. Each provider tested via wiremock mocks.

## Scope

- [ ] Verify depends on converge-traits (not converge-core for implementation)
- [ ] Each provider is feature-gated (anthropic, openai, gemini, perplexity, etc.)
- [ ] Tests use wiremock — zero API keys needed for `cargo test`
- [ ] At least one example showing a provider agent in a convergence run
- [ ] Justfile with standard targets
- [ ] Doc comments explaining "this is one instantiation strategy"
- **Out of scope:**
  - Do not modify converge-traits or converge-core
  - Do not add new providers (align existing ones)

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| wiremock | latest | dev-dependency | Mock HTTP for tests |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | verify existing | Per-provider tests |
| Integration tests | ≥ 1 new | Full convergence run with mock provider |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] `cargo test` passes with no API keys

## Status: draft

## Log

- **2026-03-11** — Created task
