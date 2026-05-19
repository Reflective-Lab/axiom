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

## Completed: v0.9 — Truth-to-Formation Run Proof
**Target:** prove one `.truths` source can drive Organism to a Formation and Converge to a fixed point.
**Completed:** 2026-05-19 — v0.10 report shape rolls forward the remaining stop reason, promoted facts, and integrity proof item.

- [x] Document the direction in README and KB
- [x] Add an `AxiomRunReport` design note or skeleton type
- [x] Add fixture proof test: Truth -> IntentPacket -> Organism formation -> Converge fixed point
- [x] Use Organism's public runtime surfaces for selection, compilation, instantiation, and execution
- [x] Keep first proof fixture-backed; introduce Mosaic-backed providers after the fixed-point path is green
- [x] Surface stop reason, promoted facts, and integrity proof in the report

### Today Plan — 2026-05-17

1. [x] Publish `axiom-truth` 0.8.1.
2. [x] Finish this README/KB/milestone documentation pass.
3. [x] Start the v0.9 fixture proof test and stop at the first missing public stack surface.
4. [x] Defer real Mosaic-backed providers until the fixture path reaches `StopReason::Converged`.

## Completed: v0.10 — Job-To-Truth Package
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Prove a structured JTBD becomes a reviewable Truth Package whose generated `.truths`, `IntentPacket`, and verifier spec all trace back to the originating job clauses.
**Completed:** 2026-05-19 — foundation types, deterministic decoder spine, report shape, and Axiom provenance seed path landed.

Inverts the input boundary: the human writes a JTBD; `.truths` becomes an auditable intermediate; the Truth Package is the typed, reproducible bundle downstream layers consume. Doctrine: `kb/Architecture/Axiom as Verifier.md`.

### Design decisions for the first slice

- Clause IDs are deterministic, package-local hierarchical paths:
  `jtbd.<job_key>.actor`, `jtbd.<job_key>.functional_job`,
  `jtbd.<job_key>.so_that`, `jtbd.<job_key>.evidence.<key>`, and
  `jtbd.<job_key>.failure.<key>`.
- Clause IDs are readable addresses; `ClauseFingerprint` proves the canonical
  clause text. List indexes are forbidden as identity.
- Decoder v0.10 is rule-based and deterministic. LLMs may later suggest
  scaffolding, but deterministic normalization owns the manifest.
- Crate split is deferred. Start inside `axiom-truth` with separable
  `truth_package` types; extract only if Helm/Organism need the manifest
  without the validation pipeline.
- Overrides are overlays, not mutation of generated `.truths`. Full override
  mechanics are deferred until the package spine exists.
- Verifier stop reasons are sets, not a scalar. Forbidden actions compose
  additively with Cedar requirements. Full verifier semantics are deferred.

### First implementation slice

- [x] Add `kb/Architecture/Clause IDs and Decoder Spine.md`
- [x] Define `JtbdDocument`, `JtbdClause`, `ClauseId`, and `ClauseFingerprint`
- [x] Normalize structured JTBD input deterministically without array indexes
- [x] Add a lineage closure proof for generated artifacts

### Foundation — Truth Package model

- [x] Define `JtbdClause` model with stable clause IDs (actor, functional_job, so_that, evidence_required, failure_modes)
- [x] Define `TruthPackage` manifest type: source JTBD, generated `.truths`, scenarios, predicates, policy/evidence/simulation/invariant expectations, `IntentPacket`, proof obligations, verifier spec, lineage map, replay profile, package ID, truth version
- [x] Define `LineageMap` with clause→artifact and artifact→clause edges

### Decoder — JTBD as the source

- [x] Implement `decode_jtbd(JtbdInput) -> TruthPackage`
- [x] Generated `.truths` is an auditable projection
- [x] Override and version overlays for generated `.truths`
- [x] Determinism: regenerating from the same JTBD yields an identical Truth Package
- [x] Lineage closure: every artifact carries originating clause IDs; every clause is used, explicitly deferred, or explicitly rejected — no orphan artifacts, no unused clauses

### Verifier spec — the post-run judge (types only)

- [x] Define `VerifierSpec`: expected stop reason, required evidence, forbidden actions, satisfaction conditions
- [x] Define `AxiomRunReport` verdict enum: `Satisfied`, `Blocked`, `Exhausted`, `Invalid`
- [x] Roll forward the open v0.9 item — surface stop reason, promoted facts, and integrity proof in the report shape
- [x] Wiring to a live Organism/Converge report adapter deferred to v0.11

### Provenance — chain of custody to Converge

- [x] Implement `AXIOM_PROVENANCE` ZST + const per Mosaic's Upstream Adoption Brief (`converge_pack::ProvenanceSource`)
- [x] Route proposed facts seeded from a Truth Package through `AXIOM_PROVENANCE.proposed_fact(...)`
- [x] Fixture test: a Truth-Package-seeded fact flows through Converge's evidence chain with Axiom provenance preserved

### Documentation

- [x] `kb/Concepts/Truth Package.md` — schema reference
- [x] `kb/Concepts/JTBD.md` — reframe JTBD as source, not `.truths` comment metadata
- [x] Cross-links from `kb/Architecture/Axiom as Verifier.md`

### Out of scope (deferred)

- One marquee job end-to-end with verdict computation — v0.11
- Decoder calibration / clause-shape priors (the learning loop) — v0.12
- Three marquee proofs (irreversible / ambiguous / governed) — v1.0

## Next: v0.11 — Marquee Job Run Verifier
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Drive one Truth Package through a live Organism/Converge run adapter and compute an `AxiomRunReport` verdict from observed stop reason, promoted facts, evidence refs, trace links, and integrity.

- [x] Select one marquee JTBD fixture and keep it narrow enough to verify end to end
- [x] Preserve multi-boundary runs with staged `AxiomRunReport` records
- [ ] Replace the v0.10 deterministic expiry sentinel with JTBD-declared time budget / expiry semantics
- [x] Restore `atelier-showcase` `just show-round-driven` so the round-driven fixture can be backed by a live run
- [x] Update the Axiom fixture to reflect the platform-API run: LLM convergence judge halts at round 2 and round 3 is skipped by halt marker
- [ ] Adapt an Organism/Converge run record into `AxiomRunObservation`
- [ ] Compute `AxiomRunVerdict` from `VerifierSpec` plus observed stop reason, required evidence, forbidden actions, promoted facts, and integrity
- [ ] Prove every promoted fact in the report traces to the source job clause, evidence requirement, failure mode, and truth version it served
- [ ] Keep formation selection in Organism, authority recompute in Converge, and specialist/plugin hosting outside Axiom
- [ ] Track an irreversible commitment fixture as the next strict-verdict proof after round-driven lands

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
