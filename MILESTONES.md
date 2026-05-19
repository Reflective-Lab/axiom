# Milestones

> See `~/dev/reflective/stack/bedrock-platform/EPIC.md` for the coarse-grained outcomes these milestones advance.

## Completed: v0.8.1 — Runtime Intent Boundary
**Deadline:** 2026-05-15 | **Epic:** Foundation (Axiom is well-structured and reliable)
**Completed:** 2026-05-17 — published `axiom-truth` 0.8.1 to crates.io after package verification passed

- [x] Create AGENTS.md (canonical project entrypoint)
- [x] Refactor justfile (remove Converge-specific tasks, add axiom focus)
- [x] Fix missing Cedar policy file (policy_lens test fixture)
- [x] Lint cleanup — resolve clippy pedantic warnings in code generation
- [x] Add `intent` module (`TruthDocument` → `organism_pack::IntentPacket`)
- [x] Document API surfaces and runtime intent dependency direction
- [x] Build against `converge-provider` 3.9.1, published `converge-manifold-adapters` 1.1.1, and `organism-pack` 1.9.0
- [x] Document truth validation pipeline contract (kb/Architecture/Validation Pipeline.md)
- [x] Implement deterministic simulation (reproducible across runs)
- [x] Add integration tests against mock Converge types
- [x] Publish axiom-truth 0.8.1 to crates.io
  - Upstream release train verified in crates.io search: `organism-pack` 1.9.0 and `converge-manifold-adapters` 1.1.1 are available.
  - Verified 2026-05-17: `cargo check --all-targets`, `cargo test`, `cargo package --allow-dirty`, `cargo package --list --allow-dirty`, and `cargo publish --dry-run --allow-dirty` passed after the manifest bump.
  - Published 2026-05-17 with `cargo publish --allow-dirty`.

## Current: v0.9 — Truth-to-Formation Run Proof
**Target:** prove one `.truths` source can drive Organism to a Formation and Converge to a fixed point.

- [x] Document the direction in README and KB
- [x] Add an `AxiomRunReport` design note or skeleton type
- [x] Add fixture proof test: Truth -> IntentPacket -> Organism formation -> Converge fixed point
- [x] Use Organism's public runtime surfaces for selection, compilation, instantiation, and execution
- [x] Keep first proof fixture-backed; introduce Mosaic-backed providers after the fixed-point path is green
- [ ] Surface stop reason, promoted facts, and integrity proof in the report

### Today Plan — 2026-05-17

1. [x] Publish `axiom-truth` 0.8.1.
2. [x] Finish this README/KB/milestone documentation pass.
3. [x] Start the v0.9 fixture proof test and stop at the first missing public stack surface.
4. [x] Defer real Mosaic-backed providers until the fixture path reaches `StopReason::Converged`.

## Next: v0.10 — Job-To-Truth Package
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Prove a structured JTBD becomes a reviewable Truth Package whose generated `.truths`, `IntentPacket`, and verifier spec all trace back to the originating job clauses.

Inverts the input boundary: the human writes a JTBD; `.truths` becomes an auditable intermediate; the Truth Package is the typed, reproducible bundle downstream layers consume. Doctrine: `kb/Architecture/Axiom as Verifier.md`.

### Foundation — Truth Package model

- [ ] Define `JTBDClause` model with stable clause IDs (actor, functional_job, so_that, evidence_required, failure_modes)
- [ ] Define `TruthPackage` manifest type: source JTBD, generated `.truths`, scenarios, predicates, policy/evidence/simulation/invariant expectations, `IntentPacket`, proof obligations, verifier spec, lineage map, replay profile, package ID, truth version
- [ ] Define `LineageMap` with clause→artifact and artifact→clause edges

### Decoder — JTBD as the source

- [ ] Implement `decode_jtbd(JTBDInput) -> TruthPackage`
- [ ] Generated `.truths` is an auditable projection; humans may inspect, override, and version it
- [ ] Determinism: regenerating from the same JTBD yields an identical Truth Package
- [ ] Lineage closure: every artifact carries originating clause IDs; every clause is used, explicitly deferred, or explicitly rejected — no orphan artifacts, no unused clauses

### Verifier spec — the post-run judge (types only)

- [ ] Define `VerifierSpec`: expected stop reason, required evidence, forbidden actions, satisfaction conditions
- [ ] Define `AxiomRunReport` verdict enum: `Satisfied`, `Blocked`, `Exhausted`, `Invalid`
- [ ] Roll forward the open v0.9 item — surface stop reason, promoted facts, and integrity proof in the report shape
- [ ] Wiring to a live Organism/Converge run deferred to v0.11

### Provenance — chain of custody to Converge

- [ ] Implement `AXIOM_PROVENANCE` ZST + const per Mosaic's Upstream Adoption Brief (`converge_pack::ProvenanceSource`)
- [ ] Route proposed facts seeded from a Truth Package through `AXIOM_PROVENANCE.proposed_fact(...)`
- [ ] Fixture test: a Truth-Package-seeded fact flows through Converge's evidence chain with Axiom provenance preserved

### Documentation

- [ ] `kb/Concepts/Truth Package.md` — schema reference
- [ ] `kb/Concepts/JTBD.md` — reframe JTBD as source, not `.truths` comment metadata
- [ ] Cross-links from `kb/Architecture/Axiom as Verifier.md`

### Out of scope (deferred)

- One marquee job end-to-end with verdict computation — v0.11
- Decoder calibration / clause-shape priors (the learning loop) — v0.12
- Three marquee proofs (irreversible / ambiguous / governed) — v1.0

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
