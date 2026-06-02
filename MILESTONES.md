# Milestones

> See `~/dev/reflective/bedrock-platform/EPIC.md` for the coarse-grained outcomes these milestones advance.

## Completed: v0.15.0 — Axiom Layer
**Date:** 2026-05-20 | **Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Completed:** 2026-05-20 — published `axiom-truth` 0.15.0 to crates.io after package verification passed.
**Goal:** Ship the current Axiom layer as a real stack boundary now, without turning the Helm/app contract probes into Axiom release scope.

This release draws the line after v0.15:

- Axiom owns the truth layer: JTBD decoding, Truth Packages, generated `.truths`, intent compilation, verifier specs, run observations, run reports, lineage, provenance, and decoder calibration.
- Axiom does not own Helm operator UX, app domain state, app-specific adapters, Formation selection, Mosaic specialists, plugin hosting, raw run history, or Converge promotion authority.
- The Axiom-Helm-App contract work is boundary guidance for future stack thinning. For this release, only the app-neutral adapter receipt shape is public Axiom API; Helm-owned readiness packets and app-domain receipts stay outside Axiom.

### Release checklist

- [x] Public crate surface exposes the release spine: `decode_jtbd`, `TruthPackage`, `VerifierSpec`, `AxiomRunObservation`, `AxiomRunReport::verify`, `ObservationAdapterReceipt`, `LearningEpisode`, `CalibrationRecord`, and `CalibrationTable`.
- [x] Milestone history proves the spine with deterministic package generation, live-observation report adaptation, irreversible commitment verdicts, decoder calibration, persisted review, uncovered-clause concerns, and repeated marquee adapter receipts.
- [x] README, changelog, and API docs describe the v0.15 Axiom release boundary rather than the older v0.8.1-only intent boundary.
- [x] MSRV metadata is aligned on Rust 1.94 for the crate, `cz doctor`, and generated WASM crates.
- [x] `just test` — passed 2026-05-20 with 455 lib tests, 2 CLI tests, and all integration/marquee suites passing; 3 WASM-target tests ignored as expected.
- [x] `just lint` — passed 2026-05-20.
- [x] `cargo package --allow-dirty` — passed 2026-05-20 after updating yanked transitive `enumset` 1.1.12 to 1.1.13; packaged 102 files and verified `axiom-truth` 0.15.0.
- [x] `cargo publish --dry-run --allow-dirty` — passed 2026-05-20 after updating yanked transitive `enumset` 1.1.12 to 1.1.13; upload aborted because this was a dry run.
- [x] Publish `axiom-truth` 0.15.0 to crates.io after the dry run passes and upstream published dependencies are available — published 2026-05-20.

### After release

- Continue the Helm operator-control work in Helm, especially a Helm-owned `JobReadinessPacket`.
- Keep app-specific transcript adapters in apps or Helm execution surfaces; only promote app-neutral audit envelopes into Axiom.
- Use Atlas as the next vertical only after the v0.15 Axiom release boundary is shipped.

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
- Irreversible commitment strict-verdict proof — v0.12
- Decoder calibration / clause-shape priors (the learning loop) — v0.13
- Three marquee proofs (irreversible / ambiguous / governed) — v1.0

## Next: v0.11 — Marquee Job Run Verifier
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Drive one Truth Package through a live Organism/Converge run adapter and compute an `AxiomRunReport` verdict from observed stop reason, promoted facts, evidence refs, trace links, and integrity.

- [x] Select one marquee JTBD fixture and keep it narrow enough to verify end to end
- [x] Preserve multi-boundary runs with staged `AxiomRunReport` records
- [x] Replace the v0.10 deterministic expiry sentinel with JTBD-declared time budget / expiry semantics (`0cd2709`)
- [x] Restore `atelier-showcase` `just show-round-driven` so the round-driven fixture can be backed by a live run
- [x] Update the Axiom fixture to reflect the platform-API run: LLM convergence judge halts at round 2 and round 3 is skipped by halt marker
- [x] Adapt an Organism/Converge run record into `AxiomRunObservation` (`21e0d99`) — recipe-form adapter in `tests/converge_observation_adapter.rs`; lib-level dep on `converge-kernel` is precluded by AGENTS.md, so the canonical pattern lives caller-side. Live-Engine adapter test (running a real Converge engine end-to-end) remains a follow-up.
- [x] Compute `AxiomRunVerdict` from `VerifierSpec` plus observed stop reason, required evidence, forbidden actions, promoted facts, and integrity (`a991cdb`)
- [x] Prove every promoted fact in the report traces to the source job clause, evidence requirement, failure mode, and truth version it served (`6aef7ec`, plus negative-path proof in `69030d6`)
- [x] Keep formation selection in Organism, authority recompute in Converge, and specialist/plugin hosting outside Axiom (doctrine in `kb/Architecture/Axiom as Verifier.md`; no commit in this milestone moved any of these into Axiom)
- [x] Track an irreversible commitment fixture as the next strict-verdict proof after round-driven lands

## Completed: v0.12 — Irreversible Commitment Verifier
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Prove Axiom's verifier semantics against the sharpest job class: an irreversible commitment whose success, block, and invalid states have concrete policy and evidence meanings.
**Completed:** 2026-05-19 — strict escrow release fixture, recorded Tally release transcript adapter, and v0.13 learning feedstock proof landed.

v0.11 proves Axiom can verify a dynamic multi-boundary Organism run without
becoming a formation selector. v0.12 should prove the verdict engine is not
soft: `Satisfied`, `Blocked`, and `Invalid` must be distinguishable on an
irreversible job with real authority requirements, concrete satisfaction
conditions, and explicit forbidden actions. Decoder calibration is intentionally
not part of v0.12; Tally should produce hard verifier labels that v0.13 can
learn from.

### Candidate family

- Primary app: Tally escrow release at
  `/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow`.
- Current slice: a strict `escrow_release` fixture plus a recorded
  `tally-escrow` release transcript adapter for the Tally transition/custody
  shape. The packaged live Tally runner remains app-owned; Axiom owns the
  observation contract and verifier proof.
- v1.0 expansion order: Quorum sensemaking at
  `/Users/kpernyer/dev/reflective/marquee-apps/quorum-sense`, then Scout
  sourcing at `/Users/kpernyer/dev/reflective/marquee-apps/scout-sourcing`.
  They are useful, but less crisp than Tally for first verdict semantics.

### Checklist

- [x] Select the irreversible commitment candidate and document whether it is a real marquee proof or a fixture proof
- [x] Define the compact JTBD with evidence requirements and failure modes that make irreversibility explicit
- [x] Decode the JTBD into a Truth Package whose verifier spec has required evidence and forbidden actions
- [x] Add concrete policy requirement artifacts for the commitment envelope
- [x] Fixture at least three cases: satisfied release, blocked release, and invalid release attempt
- [x] Add an Axiom-side adapter recipe from Tally release transition facts, custody receipt, signing witnesses, and observed promotion authority into `AxiomRunObservation`
- [x] Replace the fixture-only proof with a recorded `tally-escrow` release transcript that adapts into `AxiomRunObservation`
- [x] Prove Axiom declares authority requirements while reports preserve Converge's observed promotion gate and policy hash
- [x] Prove promoted commitment facts trace back to the source job clause, evidence requirement, failure mode, and truth version in the fixture
- [x] Ensure the final Tally report contains enough typed outcome data to become a v0.13 `LearningEpisode`
- [x] Record the residual gap from strict irreversible proof to the v1.0 three-proof set

## Completed: v0.13 — Decoder Calibration Learning Loop
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Capture verifier outcomes as typed decoder calibration so future JTBDs produce richer, better-covered Truth Packages without turning Axiom into a reasoner, authority source, or formation selector.
**Completed:** 2026-05-19 — Tally release verifier outcomes now produce reviewable calibration records, accepted priors enrich regenerated Truth Packages, and boundary tests prove calibration remains decoder-only.

v0.13 is the "the better we get, the tighter the loop gets" milestone. It
should start after v0.12 gives Axiom hard labels from an irreversible job:
`Satisfied`, `Blocked`, and `Invalid` with clause-level evidence and
failure-mode lineage. Calibration learns from those labels as a decoder aid,
not as runtime strategy.

### Shape

- Input signal: `AxiomRunReport` plus lineage audit, verifier spec, package ID,
  truth version, JTBD clause IDs, clause fingerprints, and verdict.
- Calibration key: normalized clause shape plus clause kind, domain hints,
  decoder rule ID, and source clause fingerprint class. Do not key on raw
  natural language alone.
- Calibration value: likely evidence requirements, forbidden-action templates,
  scenario scaffolds, verifier expectations, and confidence/rationale.
- Query point: during JTBD-to-Truth Package decoding, before generated
  artifacts are finalized.
- Audit rule: every calibration-influenced artifact must name the calibration
  entry and still trace back to a JTBD clause.

### Checklist

- [x] Define `LearningEpisode` / calibration record shape without changing runtime reasoning
- [x] Decide persistence location and ownership for calibration tables
- [x] Define deterministic lookup keys for JTBD clause shape and domain hints
- [x] Feed v0.12 Tally verifier outcomes into calibration records
- [x] Use calibration to enrich a regenerated Truth Package while preserving deterministic auditability
- [x] Prove calibration does not select Formations, recompute authority, or host specialist execution
- [x] Document how operators review, accept, reject, or reset learned priors

## Completed: v0.14 — Calibration Persistence And Review
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Move calibration from in-memory test records to an operator-reviewable persisted store without changing the decoder-only boundary.
**Completed:** 2026-05-19 — `CalibrationTable` gains canonical JSONL persistence (sort-on-serialize, dedup-on-load), typed review APIs (`accept` / `reject` / `reset` with mandatory notes), and a kb doctrine page separating distilled priors from raw run history.

v0.13 proves the typed loop. v0.14 should make it operational: store proposed
records, review them, accept/reject/reset them, and replay package enrichment
from the accepted table.

### Candidate checklist

- [x] Define the serialized calibration table format and append-only record log — JSONL (one `CalibrationRecord` per line, sorted by record id; deterministic byte output).
- [x] Add load/save helpers with deterministic ordering and schema validation — `CalibrationTable::to_jsonl` / `CalibrationTable::from_jsonl` with typed `CalibrationPersistenceError` (`InvalidLine` carries 1-based line number + serde message; `DuplicateRecord` rejects collisions).
- [x] Add review APIs or CLI affordances for accept/reject/reset with required notes — `CalibrationTable::accept` / `reject` / `reset` mutate a record by id, with mandatory non-empty notes; `CalibrationReviewError` distinguishes `RecordNotFound` from `EmptyNote`. Six tests in `tests/calibration_persistence.rs` cover the three transitions, empty-note rejection across all three actions, unknown record ids, status-change-on-reviewed records, and JSONL round-trip with `review_note` preserved.
- [x] Prove rejected/reset records do not influence package enrichment — `persisted_mixed_status_table_only_enriches_via_accepted_records` (the v0.13 invariant survives the JSONL boundary).
- [x] Add a golden replay test: persisted accepted table regenerates identical calibration suggestions — `golden_replay_persisted_accepted_table_regenerates_identical_suggestions` asserts byte-identical `calibration_suggestions`, identical `lineage`, and serde-equal whole packages.
- [x] Keep raw run history in downstream stores; Axiom persists only distilled decoder priors — doctrine in `kb/Architecture/Decoder Calibration.md` "Raw History Versus Distilled Priors" section; `CalibrationRecord` carries `source_episode_ids` backlinks only, not full report bodies.
- [x] Update operator docs for calibration review workflow — `kb/Architecture/Decoder Calibration.md` "Operator Review Workflow" section walks the 6-step loop from propose → load → review → persist → reapply.

## Completed: v0.15 — Uncovered-Clause Calibration
**Epic:** E7 (Axiom translates human jobs into governed runtime contracts)
**Target:** Extend decoder calibration so the loop also learns from clauses that recurrently go uncovered. An uncovered evidence clause becomes a reviewed decoder *concern* — a typed signal that future packages may want better prompts, alternate evidence scaffolding, default expectations, or operator-facing warnings. The source JTBD remains authoritative; Axiom never silently weakens a stated requirement.
**Completed:** 2026-05-19 — `ClauseCoverageStatus` + `CalibrationSignalKind` enums, `Concern` records for uncovered `EvidenceRequired` clauses on `Invalid` / `Blocked` verdicts, separate `CalibrationConcern` artifact channel, non-weakening invariant proven, JSONL persistence preserves the new signal shape.

v0.13 + v0.14 only learn from clauses the run *covered* — cited by a
promoted fact as evidence or as a failure guard. The complementary signal
— "this clause shape is repeatedly required but never cited" — is
currently lost. An `Invalid` verdict caused by missing evidence produces
no calibration record at all, so the decoder never learns which clause
shapes are hard to satisfy. This was explicitly recorded as the v0.15+
extension path in the v0.13 "Known Limitations" section of
`kb/Architecture/Decoder Calibration.md`.

This matters for app-thinning. Once v0.15 lands, Helm can surface an
operator-facing signal like "this job keeps missing this class of
evidence; accept a decoder prior so future packages surface it earlier" —
without Axiom having to weaken the source JTBD or host any of that
operator interface itself.

v0.15 closes that gap as a decoder-only extension that proposes
*concerns*, not silent relaxations. Calibration still does not select
Formations, recompute authority, host specialists, or modify the source
JTBD's evidence requirements.

### Design decisions (settled)

- **Signal location on `LearningClauseSignal`** — replace the boolean
  `covered_as_evidence` + `covered_as_failure_guard` fields with a typed
  `coverage_status: ClauseCoverageStatus`. Variants:
  - `Uncovered`
  - `CoveredAsEvidence`
  - `CoveredAsFailureGuard`
  - `CoveredAsEvidenceAndFailureGuard`

  This is a JSON schema bump for `LearningEpisode`; document it in the
  kb. The v0.13 doctrine page already noted the persistence format
  should leave room for this axis.

- **Concern vs reinforcement is typed, not implied** — add a typed
  `signal_kind: CalibrationSignalKind` to `CalibrationValue` (or
  `CalibrationRecord` — settle during implementation). Variants:
  - `Reinforcement` — covered-clause records the decoder should keep
    reaching for (the v0.13 / v0.14 default).
  - `Concern` — uncovered-clause records the decoder should treat as
    warnings, prompts, or default-expectation candidates. Never as
    silent JTBD relaxation.

  Operator-distinguishable, machine-distinguishable, explicit.

- **Source JTBD remains authoritative** — accepting a `Concern` record
  influences decoder behavior on the *next* package generation (prompts,
  defaults, warnings, alternate scaffolding). It does not remove or
  weaken evidence requirements from the original JTBD. A regenerated
  package's `verifier_spec.required_evidence` and `forbidden_actions`
  must be byte-identical to the pre-calibration version regardless of
  which concerns were accepted.

- **Review workflow** — same `accept` / `reject` / `reset` surface as
  v0.14. Operators distinguish concerns from reinforcements via the
  `signal_kind` field; no separate API.

### Open design questions

- **Which verdicts trigger uncovered-clause records?** Almost certainly
  `Invalid` (where missing evidence is the cause). Probably also
  `Blocked` (gate hasn't opened yet; evidence might still arrive).
  Probably not `Satisfied` — if the run was satisfied, no required
  clause was truly missing. Open for `Exhausted` — budget ran out
  before evidence could arrive; the signal is real but noisier.

### Checklist

- [x] Add `ClauseCoverageStatus` enum and replace the two booleans on
      `LearningClauseSignal` with `coverage_status`. Existing covered
      clauses migrate to `CoveredAsEvidence` / `CoveredAsFailureGuard`
      / `CoveredAsEvidenceAndFailureGuard`.
- [x] Add `CalibrationSignalKind` enum (`Reinforcement` | `Concern`)
      and thread it through `CalibrationValue` with
      `#[serde(default)]` so v0.13/v0.14 persisted JSONL still parses
      cleanly. v0.13 covered-clause records explicitly carry
      `Reinforcement`; v0.13 + v0.14 test suites pass unchanged.
- [x] Emit `Concern` calibration records for uncovered
      `EvidenceRequired` clauses on `Invalid` and `Blocked` verdicts.
      `Satisfied` does not fire (no missing required evidence by
      definition); `Exhausted` is deferred (noisier signal than the
      other two — open question recorded in kb).
- [x] Fixture: an `Invalid` verifier run with missing evidence
      produces typed `Concern` records; the operator accepts them;
      `apply_decoder_calibration` adds them to the new
      `TruthPackageArtifacts.calibration_concerns` channel as
      `ArtifactKind::CalibrationConcern` artifacts (distinct from
      reinforcement `CalibrationSuggestion` artifacts).
- [x] **Non-weakening invariant** — `apply_decoder_calibration`
      captures the package's `verifier_spec.required_evidence` and
      `forbidden_actions` before enrichment and asserts byte-identical
      results after, regardless of which concerns are accepted. A
      dedicated test (`accepted_concerns_do_not_weaken_verifier_spec`)
      proves this against a real Concern-bearing fixture.
- [x] Persistence: `Concern` records survive JSONL round-trip; the
      v0.14 byte-determinism + golden-replay tests still pass with the
      new signal shape; the wire format carries
      `"signal_kind":"concern"` / `"signal_kind":"reinforcement"`.
- [x] Move the v0.13 "Known Limitations" entry on uncovered clauses
      out of limitations; add a "Concerns Versus Reinforcements"
      section to `kb/Architecture/Decoder Calibration.md` documenting
      the typed signal, verdict-trigger logic, scope-clause exclusion,
      and the non-weakening invariant.
- [x] Prove uncovered-clause learning does not select Formations,
      recompute authority, host specialists, or modify the source JTBD
      — extended boundary test on serialized records covers both
      Reinforcement and Concern signal kinds.

### Out of scope (deferred)

- Confidence decay over time (a prior loses weight if never
  re-confirmed).
- Domain-hint inheritance (a child domain inherits priors from a
  parent).
- LLM-assisted enrichment of calibration suggestions; v0.15 stays
  rule-based per the v0.10 decoder doctrine.
- Helm-side operator UI for "this job keeps missing this class of
  evidence" — Helm owns that surface; v0.15 only produces the typed
  signal Helm consumes.

### Near-term product sequence

- Continue Tally for a few more iterations as the boundary-finding loop:
  identify which release controls belong in Axiom truth packages, which
  belong in Helm operator surfaces, and which must stay in the Tally app.
- Keep `kb/Architecture/Axiom-Helm-App Contract.md` updated during that
  loop so the architecture follows real pressure from the app rather than
  abstract layering.
- After Tally clarifies the contract, use
  `/Users/kpernyer/dev/reflective/marquee-apps/atlas-integration` as the
  next app vertical and start from the clarified Axiom-Helm-App split.

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
