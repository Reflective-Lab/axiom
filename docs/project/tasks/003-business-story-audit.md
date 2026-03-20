# Task: converge-business — Verify the Story is Right

## Wave

- **Wave:** 1 — Foundation
- **Plan:** See `plans/CRATE_ALIGNMENT.md`
- **Depends on:** none (parallel with 001, 002)

## Branch

- **Repo:** `converge-business`
- **Branch:** `feature/story-audit`
- **Base:** `main`
- **VCS:** jj

```sh
jj new main -m "story-audit"
jj bookmark create feature/story-audit -r @
```

## Context for Agent

- **Architecture:** See `converge-business/docs/PLATFORM_ARCHITECTURE.md` (just written)
- **This crate's role:** Single source of truth for all documentation. Strategy, business model, packs, GTM, architecture. All other repos reference this. No code.
- **Reference:** README.md already has structure, reading orders, and index
- **Key question:** Does the existing content align with the architecture we just defined?

## Problem

We just wrote `PLATFORM_ARCHITECTURE.md` defining the full platform architecture with clear crate roles, the WASM compilation flow, and the "applications as intent decoders" vision. The existing docs in converge-business predate this architecture. They may describe outdated structures, missing crates, or wrong relationships.

## Goal

Every document in converge-business tells the same story as `PLATFORM_ARCHITECTURE.md`. A stranger reads the repo and understands:
1. Why convergence matters (strategy)
2. What the platform does (architecture)
3. How the crates relate (dependency graph)
4. How customers use it (JTBD → WASM → runtime)
5. The 5-6 core concepts (from converge-core examples)

## Scope

- [ ] Cross-reference `PLATFORM_ARCHITECTURE.md` against existing strategy docs
- [ ] Verify `strategy/STRATEGY.md` aligns with architecture
- [ ] Verify `strategy/AXIOMS.md` matches the 9 axioms in converge-core lib.rs
- [ ] Verify `docs/ARCHITECTURE_OVERVIEW.md` matches new architecture
- [ ] Verify `docs/SYSTEM_OVERVIEW.md` mentions all crates correctly
- [ ] Update README.md reading order to include PLATFORM_ARCHITECTURE.md
- [ ] Verify pack descriptions are consistent
- [ ] Flag contradictions or gaps (do not invent — flag for human review)
- **Out of scope:**
  - Do not modify other converge-* crates
  - Do not delete existing docs without flagging
  - Do not write new strategy (flag gaps instead)

## New Dependencies

| Crate | Version | Feature-gated? | Justification |
|-------|---------|----------------|---------------|
| — | — | — | No code, no deps |

## Acceptance Criteria

### Tests

Not applicable — this is a documentation audit.

### Quality

- [ ] No contradictions between PLATFORM_ARCHITECTURE.md and other docs
- [ ] All crates mentioned in architecture appear correctly in SYSTEM_OVERVIEW
- [ ] README reading order includes architecture doc
- [ ] Gaps flagged in a `REVIEW_NOTES.md` for human decision

## Constraints

- Do not delete content — flag for review
- Do not invent new strategy or business content
- Do not modify code in any repo

## Workflow

1. Read PLATFORM_ARCHITECTURE.md (the new truth)
2. Read each existing doc and compare
3. Fix obvious alignment issues (crate names, relationships)
4. Create `REVIEW_NOTES.md` listing gaps and contradictions
5. Update README reading order
6. Mark done

## Status: review

## Log

- **2026-03-11** — Created task
- **2026-03-11** — Audit complete. REVIEW_NOTES.md written with P0/P1/P2 findings. README reading order updated. AXIOMS.md confirmed aligned. 5 crates missing from most docs, access paths and WASM flow undocumented outside PLATFORM_ARCHITECTURE.md.
