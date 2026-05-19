---
tags: [log]
source: llm
---

# Axiom — Mutation Log

## 2026-05-19

- Created `Architecture/Axiom as Verifier.md` as the E7 doctrine anchor: Axiom is the typed translator from JTBD to governed runtime contract, `.truths` is an auditable intermediate, `AxiomRunReport` is a verifier, and the explicit boundaries are no formation selection, no authority recompute, and no specialist hosting.
- Linked the new architecture page from `Home.md` so future Axiom work has a discoverable doctrine target before code follows.
- Expanded the WASM responsibility boundary across `Architecture/WASM Compilation.md`, `Architecture/Converge Contract.md`, `Architecture/Validation Pipeline.md`, `Architecture/API Surfaces.md`, `Philosophy/Truth-Driven Development.md`, and `Architecture/Axiom as Verifier.md`: Axiom produces compiled artifacts and manifests, Helm owns sandbox hosting and plugin lifecycle, and Converge owns governed promotion, stop reasons, and integrity.
- Added cross-references from `Architecture/Axiom as Verifier.md` to the Helm and Converge KB pages that now anchor the WASM sandbox boundary.
- Added `Architecture/Clause IDs and Decoder Spine.md` and enriched the v0.10 milestone with the first-slice decisions: deterministic hierarchical clause IDs, separate text fingerprints, rule-based decoder spine, deferred crate split, overlay-based overrides, and set-valued verifier stop reasons.
- Added the `truth_package` module with `JtbdInput`, `JtbdDocument`, `JtbdClause`, `ClauseId`, `ClauseFingerprint`, `ArtifactLineage`, and `LineageMap` closure checks; documented the public surface in README, API Surfaces, System Overview, INDEX, and JTBD concept docs.
- Extended `truth_package` with `TruthPackage`, `TruthPackageId`, `TruthPackageArtifacts`, `ProofObligation`, `VerifierSpec`, `ReplayProfile`, and deterministic `decode_jtbd(JtbdInput) -> TruthPackage`; added `Concepts/Truth Package.md` as the schema reference and marked the completed v0.10 spine items in `MILESTONES.md`.
- Added generated `.truths` override/version overlays: `TruthProjectionOverlay`, `TruthProjectionVersion`, `TruthProjectionSource`, package-level overlay application, parse/target/source-clause validation, and tests proving overlays do not mutate the deterministic generated package.
- Added the v0.10 `AxiomRunReport` surface with `AxiomRunVerdict`, `ObservedStopReason`, promoted fact records, evidence refs, trace links, and run integrity proof summaries; updated the Truth Package concept doc, KB index, and milestone state.
- Added Axiom's `converge_pack::ProvenanceSource` marker (`AxiomTruth` / `AXIOM_PROVENANCE`), typed `TruthPackageSeedPayload`, Truth Package seed proposal helpers, and an integration test proving Converge promotion preserves the typed Axiom payload while adding evidence refs, trace links, and integrity.
- Closed v0.9/v0.10 milestone state in `MILESTONES.md` and sketched v0.11 around one marquee job run adapter plus computed `AxiomRunReport` verdict.
- Added the round-driven Formation Design marquee fixture from `atelier-showcase`: a compact JTBD, a staged `AxiomRunReport` shape that preserves the design huddle and work Formation boundaries, `tests/round_driven_marquee.rs`, and `Marquee/Round-Driven Formation Design.md`. Noted that the live showcase currently fails to compile because `ConvergenceJudge` and `convergence_reached` are referenced but not defined.
- Folded v0.10 review feedback into the docs and v0.11 plan: documented `JtbdInput::from_metadata(...)` as the legacy JTBD migration bridge, recorded the deterministic expiry sentinel as a v0.11 replacement item, and kept round-driven Formation design as the primary staged verifier proof while tracking an irreversible commitment fixture as the next strict-verdict proof.
- Updated the v0.11 round-driven marquee note from the latest `atelier-showcase` run: origin/main now carries evidence-weighted LLM scoring, the LLM convergence judge, and the platform API migration; the live run converges at round 2, fires the halt marker, skips round 3, and runs the work Formation from the converged shortlist.
- Aligned the round-driven Axiom fixture with the new platform-API markers (`proposer_exclusions_marker`, scorecard, convergence judgment, and halt marker) and added the first v0.11 promoted-fact/staged-observation adapter surface.
- Added the v0.12 milestone direction: an irreversible commitment strict-verdict verifier proof comes before decoder calibration, with ambiguous sensemaking and governed commercial decisioning deferred to the v1.0 proof set.
- Started the v0.12 escrow release fixture proof: documented the irreversible JTBD, added Satisfied/Blocked/Invalid verdict cases, and recorded that the current slice is fixture-backed until a real Tally-style escrow runtime exists.
- Replaced the vague Tally placeholder with the concrete marquee-app sequence: `tally-escrow` for v0.12, followed by `quorum-sense` and `scout-sourcing` for the broader v1.0 proof set.
- Locked the sequence: v0.12 is Tally escrow release; v0.13 is decoder calibration. Added `Architecture/Decoder Calibration.md` and documented the v0.12 report data needed to become future calibration feedstock.
- Started v0.12 implementation by emitting deterministic policy requirement artifacts from the Truth Package decoder and asserting the escrow release commitment envelope covers authority, required evidence, and forbidden failure modes.
- Extended promoted fact report records with optional Converge promotion authority data (gate ID, policy hash, approver) so Axiom reports can show authority was observed at promotion without treating Axiom as the authority source.
- Added the Axiom-side Tally release observation adapter recipe: a local wire-shaped release outcome maps transition truth keys, Organism signing witnesses, custody receipt, idempotency, and observed promotion authority into `AxiomRunObservation`; missing release truth key now produces an `Invalid` verifier result.
- Completed v0.12 by replacing the fixture-only Tally proof with a recorded release transcript, proving the final report carries v0.13 learning feedstock fields, and recording the residual gap from irreversible strict-verdict proof to the v1.0 three-proof set.
- Implemented the v0.13 decoder calibration loop: `LearningEpisode`, `CalibrationKey`, `CalibrationRecord`, `CalibrationTable`, accepted calibration suggestions, and package enrichment via `apply_decoder_calibration(...)`, with tests proving Tally release outcomes produce reviewable priors without pulling Formation selection, authority recompute, or specialist hosting into Axiom.
- Updated `Architecture/Decoder Calibration.md`, `Concepts/Truth Package.md`, and `INDEX.md` to document calibration ownership, persistence shape, review statuses, lineage requirements, and the new public types.
- Added `Architecture/Axiom-Helm-App Contract.md` to make the app-thinning direction explicit: Axiom owns truth packages and verifier learning, Helm owns operator review and sandbox lifecycle, apps own domain state and adapters, and Tally remains the boundary-finding loop before Atlas Integration becomes the next app vertical.
- Ran the first Tally boundary probe against `release-requires-conditions-met`: Tally keeps the strict agreement transition guard, Axiom owns the truth/verifier/calibration expression of release-condition evidence, Helm should own the operator surface for missing evidence and accepted concerns, and Organism/Mosaic/Converge remain runtime capability and promotion boundaries.
- Ran the second Tally boundary probe against `transition-requires-signature`: Tally keeps party roles, signing policy, and transition admission; Organism owns signing execution; Axiom verifies principal authorization evidence and concerns; Helm should render signer coverage and missing-signature review without owning domain admission.
- Ran the third Tally boundary probe against custody release receipts: Tally/platform keeps asset-class custody adapters and typed receipts, Axiom verifies receipt evidence and missing-receipt concerns, Helm should render receipt provenance and review state, and real irreversible rails need Tally-side outbox/saga discipline before external side effects.
- Ran the fourth Tally boundary probe and named `ReleaseReadinessPacket` as the first converged Helm-facing read model: it composes Axiom verifier obligations and concerns, Tally agreement/transcript state, Converge promoted evidence and integrity, and Organism/Mosaic readiness outputs without authorizing release or bypassing Tally's transition guard.
- Ran the fifth Tally boundary probe against the `AxiomRunObservation` adapter: apps own raw transcript schemas and field-to-clause mapping knowledge, Axiom owns normalized observation/report/verifier types, Helm should display both raw transcript links and normalized reports, and future adapters need deterministic audit receipts before being promoted into a shared contract.
- Marked `ObservationAdapterReceipt` as the strongest common-module candidate from the Tally probes: it describes app-neutral adapter audit metadata rather than escrow domain facts, should be proven against Atlas before implementation, and if promoted should live with Axiom's verifier contract while Helm owns display/review/storage.
- Tightened the provisional `ObservationAdapterReceipt` rule so adapter rejection is first-class: successful adapters return an observation plus a receipt, rejected adapters return no observation plus explicit receipt errors, and any future receipt id should be content-derived for replay determinism.
- Recorded Helm's post-EPIC Operator Control direction: long-running jobs, HITL, and a shared append-only ledger strengthen Helm's role as control-plane journal, while Axiom artifacts remain deterministic, serializable, and backlink-oriented rather than raw-history stores or authority sources.
- Added a local Tally fixture proof for `ObservationAdapterReceipt`: the escrow release adapter now emits deterministic success receipts with transcript/observation hashes and mapped fact/clause ids, and deterministic rejection receipts with no observation when a transcript cannot be mapped.
- Added a local Helm-facing `ReleaseReadinessPacket` fixture proof: it composes Tally transcript data, the adapter receipt, Axiom verifier verdict, clause-level evidence readiness, forbidden-action summaries, and operator actions while explicitly not authorizing the Tally transition.
- Added a local Helm ledger fixture proof: release readiness produces deterministic append-only entries for the adapter receipt and readiness packet, each backlink-only and explicitly `authority_effect: none`, with assertions that raw signature refs, custody external refs, source commands, and local app paths do not leak into ledger entries.
- Started the Atlas second-app probe with `tests/atlas_integration_marquee.rs`: identity/auth consolidation now decodes from JTBD, adapts a recorded Atlas candidate transcript into `AxiomRunObservation`, emits the same app-neutral `ObservationAdapterReceipt` envelope, builds an integration readiness packet, and journals backlink-only Helm ledger entries with no writeback authority.
- Recorded the first cross-app conclusion in the contract doc: `ObservationAdapterReceipt` is now strong enough to promote first, while Tally's release readiness and Atlas' integration readiness point toward a later Helm-owned `JobReadinessPacket` rather than an Axiom-owned escrow-specific type.
- Promoted `ObservationAdapterReceipt` into the public Axiom truth-package API and refit the Tally and Atlas probes to use it while keeping app-specific transcript adapters local; added the Atlas owner-approval negative path so missing HITL/writeback evidence yields `Invalid` readiness without authorizing provider-side action.
- Added the Quorum third-app probe with `tests/quorum_sense_marquee.rs`: organizational sensemaking release-readiness now decodes from JTBD, adapts a recorded Quorum inquiry transcript into `AxiomRunObservation`, emits the shared `ObservationAdapterReceipt`, builds a generic Helm-facing `JobReadinessPacket`, and proves missing dissent preservation or operator approval yields `Invalid` readiness without authorizing organizational action.
- Updated the Axiom-Helm-App contract conclusion after Quorum: `JobReadinessPacket` is now a Helm common-module candidate, not an Axiom public type, because it composes Axiom reports and receipts with app subject refs and operator actions.
- Added the Scout fourth-app probe with `tests/scout_sourcing_marquee.rs`: governed vendor selection now decodes from JTBD, adapts a recorded Scout sourcing transcript into `AxiomRunObservation`, emits the shared `ObservationAdapterReceipt`, builds the same Helm-facing `JobReadinessPacket`, and proves missing source provenance or over-threshold commitment without approval yields `Invalid` readiness without authorizing procurement action.
- Recorded the next app-probe choice in the contract doc: Warden Compliance should come before Fathom Narrative when searching for common Helm/operator/ledger modules; Fathom is better saved for temporal evidence windows, large data sets, and cross-period comparison.

## 2026-05-17

- Documented the v0.9 direction: Axiom should become the truth-to-formation run-proof layer that validates `.truths`, compiles `IntentPacket`, calls Organism formation selection/compilation, and reports the Converge fixed-point result.
- Added `Architecture/Truth-to-Formation Run Proof.md` with ownership boundaries, best-of-stack rules, the minimum proof, `AxiomRunReport` shape, and the 2026-05-17 plan.
- Added the first v0.9 fixture proof test: a governed vendor `.truths` source compiles to `IntentPacket`, routes through Organism's `organism-diligence` formation selection, compiles and instantiates fixture Suggestors, and reaches Converge `StopReason::Converged`.
- Published `axiom-truth` 0.8.1 to crates.io after `cargo publish --dry-run --allow-dirty` verified the release package.
- Moved Axiom onto the published upstream release train after Mosaic, Converge, and Organism were advanced: `converge-provider` 3.9.1, `converge-manifold-adapters` 1.1.1, and `organism-pack` 1.9.0.
- Confirmed crates.io search exposes `organism-pack` 1.9.0 and `converge-manifold-adapters` 1.1.1, replacing the previous missing-Organism blocker.
- Reran release checks after the manifest bump: `cargo check --all-targets`, `cargo test`, `cargo package --allow-dirty`, and `cargo package --list --allow-dirty` all passed.
- `cargo package --allow-dirty` packaged 66 files and verified `axiom-truth` 0.8.1 against the published registry path, including `converge-manifold-adapters` 1.1.1 and Organism 1.9.0 crates.
- Ran release-readiness checks for `axiom-truth` 0.8.1: `cargo check --all-targets` passed; `cargo test` passed with 427 lib tests, 2 CLI tests, 4 Converge contract integration tests, and 7 doctests passing.
- Tightened `Cargo.toml` package includes so the crates.io package does not ship local agent configs, CI metadata, or Obsidian workspace state.
- Earlier in the session, adjusted the Manifold dependency to the then-published `converge-manifold-adapters` 1.1.0 requirement while keeping the `llm-all` feature.
- Earlier in the session, confirmed the previous publish blocker: `cargo package --allow-dirty` resolved Manifold but stopped because `organism-pack` 1.8.1 and its 1.8.1 phase crates were not yet published on crates.io.

## 2026-05-15

- Aligned the runtime intent boundary with the Converge 3.9.1 release train: `converge-provider` 3.9.1, published `converge-manifold-adapters` 1.1.0 with `llm-all`, and `organism-pack` 1.8.1.
- Updated the Converge contract note to make the Manifold chat helper feature requirement explicit.

## 2026-05-07

- Added `intent` module — `compile_intent(&TruthDocument) -> IntentPacket` plus `compile_intent_from_source` convenience. Bridge logic moved here from `organism-intent::bridge` (deleted) to invert the dependency arrow: `axiom-truth → organism-pack` instead of `organism → axiom-truth`. 17 tests inline. Axiom now depends on `organism-pack` 1.5.1.
- Updated Architecture/System Overview.md — added `intent` to module map, documented the Runtime Intent pipeline, recorded the new dependency direction.
- Added Architecture/API Surfaces.md — canonical public API reference, including the `.truths` source → `parse_truth_document` → `compile_intent` → `IntentPacket` flow.
- Created Architecture/Intent Compilation.md — full field mapping (Authority/Constraint/Exception → IntentPacket), error model, caller flow.
- Updated top-level `lib.rs` doc comment — added Compiling to runtime intent section with caller example.
- Aligned AGENTS.md, MILESTONES.md, and Justfile workflow recipes to the 0.8.1 surface: `converge-provider` + `converge-manifold-adapters` + `organism-pack`; `just focus`, `just sync`, and `just status` now exist.
- Bumped axiom-truth 0.7.0 → 0.8.1 (new public module + new dep is a minor bump).

## 2026-05-04

- Updated Concepts/Simulation.md — documented opt-in domain profiles so core simulation stays domain-neutral while downstream layers can enable richer checks.

## 2026-04-15

- Created kb/ structure: Home, INDEX, LOG, Philosophy/, Concepts/, Architecture/, Building/, Workflow/
- Created Philosophy/Why Axiom.md — problem statement and design rationale
- Created Philosophy/Truth-Driven Development.md — the methodology
- Created Concepts/Truth Documents.md — format specification
- Created Concepts/Validation Pipeline.md — three-check pipeline
- Created Concepts/Predicates.md — step parsing and predicate types
- Created Concepts/Simulation.md — pre-flight analysis
- Created Concepts/Code Generation.md — WASM compilation pipeline
- Created Concepts/Policy Lens.md — Cedar policy coverage
- Created Concepts/Guidance.md — heading quality feedback
- Created Concepts/JTBD.md — Jobs-to-be-Done integration
- Created Architecture/System Overview.md — module map and pipeline
- Created Architecture/Converge Contract.md — relationship to Converge
- Created Architecture/WASM Compilation.md — build pipeline details
- Created Building/Getting Started.md — setup and development
- Created Building/Writing Truths.md — authoring guide
- Created Building/CLI Reference.md — cz command reference
- Created Workflow/Daily Journey.md — development workflow

## 2026-04-17

- Created AGENTS.md — canonical agent entrypoint (philosophy, public surface, rules, architecture, workflows)
- Updated MILESTONES.md — axiom-specific milestones starting with v0.5 Foundation Hardening
- Refactored justfile — removed converge-specific tasks, focused on axiom development workflow
