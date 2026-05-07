# Milestones

> See `~/dev/work/EPIC.md` for the coarse-grained outcomes these milestones advance.

## Current: v0.8 — Runtime Intent Boundary
**Deadline:** 2026-05-15 | **Epic:** Foundation (Axiom is well-structured and reliable)
**Session:** 2026-05-07 — local 0.8.1 release build green; runtime intent bridge moved into Axiom

- [x] Create AGENTS.md (canonical project entrypoint)
- [x] Refactor justfile (remove Converge-specific tasks, add axiom focus)
- [x] Fix missing Cedar policy file (policy_lens test fixture)
- [x] Lint cleanup — resolve clippy pedantic warnings in code generation
- [x] Add `intent` module (`TruthDocument` → `organism_pack::IntentPacket`)
- [x] Document API surfaces and runtime intent dependency direction
- [x] Build against `converge-provider` 3.8.1 and `organism-pack` 1.5.1
- [ ] Document truth validation pipeline contract (kb/Architecture/Validation Pipeline.md)
- [ ] Implement deterministic simulation (reproducible across runs)
- [ ] Add integration tests against mock Converge types
- [ ] Publish axiom-truth 0.8.1 to crates.io

## Completed: v0.4.1 — Initial Release
Completed: 2026-04-15

- [x] gherkin validation module (LLM + heuristic checks)
- [x] codegen module (Gherkin → Rust invariant skeletons)
- [x] compile module (Rust → WASM pipeline)
- [x] predicate module (step → semantic extraction)
- [x] simulation module (pre-flight analysis)
- [x] guidance module (spec quality feedback)
- [x] policy_lens module (Cedar coverage analysis)
- [x] jtbd module (Jobs-to-be-Done metadata)
- [x] cz CLI tool (orchestrator)
- [x] kb/ structure (philosophy, concepts, architecture, building, workflow)
