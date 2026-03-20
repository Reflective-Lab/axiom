# Task: [SHORT TITLE]

## Wave

- **Wave:** [1-5] — [Foundation / Instantiation / Tooling / Infrastructure / Experience]
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** [list completed tasks or "none"]

## Branch

- **Repo:** `converge-[name]`
- **Branch:** `[feature/fix]-[short-name]`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "[short-name]"
jj branch create [branch-name]
```

## Context for Agent

> This section carries everything an independent agent needs to execute without asking questions.

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md`
- **This crate's role:** [1-2 sentences — what this crate is and is not]
- **Key traits/types to implement or use:** [list from converge-traits / converge-core]
- **Reference examples:** [files or crates to follow as patterns]
- **Justfile:** Copy from `converge-project/templates/Justfile.template` if missing

## Problem

[1-3 sentences. What's broken or missing?]

## Goal

[1-3 sentences. What does done look like?]

## Scope

- [ ] [File or module to change]
- [ ] [File or module to change]
- **Out of scope:**
  - Do not modify other converge-* crates
  - Do not change published trait signatures unless listed above
  - Do not add documentation files outside this crate
  - [task-specific exclusions]

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | — |

## Acceptance Criteria

### Tests

| Category | Target | Notes |
|----------|--------|-------|
| Unit tests | ≥ N new | [specific modules] |
| Negative tests | ≥ N new | [error paths, invalid inputs] |
| Property tests | ≥ N new | [invariants to verify] |
| Integration tests | ≥ N new | [cross-boundary checks] |

### Performance

| Metric | Baseline | Target |
|--------|----------|--------|
| [e.g., latency p99] | [current] | [goal] |
| [e.g., throughput] | [current] | [goal] |

### Code Quality

- [ ] `cargo clippy` — zero warnings
- [ ] `cargo fmt` — clean
- [ ] No new `unwrap()` outside tests
- [ ] No new `todo!()` or `unimplemented!()`
- [ ] Public API has doc comments
- [ ] No increase in unsafe blocks

## Constraints

- Do not change public API signatures unless listed in scope
- Do not add new dependencies without noting them in New Dependencies
- Preserve existing test coverage — no deletions

## Workflow

1. Create branch from main
2. Implement changes in small, logical commits
3. Run full check before marking done:
   ```sh
   just check
   ```
4. Squash or rebase as needed:
   ```sh
   jj squash
   ```
5. Mark task status: `draft` → `active` → `review` → `done`

## Status: draft

## Log

<!-- Append progress notes here as work proceeds -->
- **[DATE]** — Created task
