# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Dependency freshness gate: CI and local pre-push now run `cargo update --workspace --dry-run` and fail when compatible lockfile updates are pending.

### Changed
- Dependency analysis no longer scrapes the deps.rs HTML page; deps.rs badge status is recorded as non-blocking context while Cargo owns the blocking freshness check.

## [0.15.0] - 2026-05-20

### Added
- Release-surface Truth Package spine: deterministic `JtbdInput` decoding, stable clause IDs, clause fingerprints, generated `.truths` projections, proof obligations, verifier specs, replay profiles, and lineage closure.
- `AxiomRunObservation` and `AxiomRunReport::verify(...)` for app-neutral post-run verification with `Satisfied`, `Blocked`, `Exhausted`, and `Invalid` verdicts.
- `ObservationAdapterReceipt` as the deterministic audit envelope for app-specific transcript adapters.
- Decoder calibration records, persisted `CalibrationTable` JSONL round trips, review APIs, accepted `CalibrationSuggestion` artifacts, and typed `CalibrationConcern` artifacts for uncovered evidence clauses.
- Axiom provenance seed facts through `AXIOM_PROVENANCE` and typed `TruthPackageSeedPayload` custody.
- Marquee proof fixtures across Tally, Atlas, Quorum, Scout, Warden, Triage, Inkling, Plumb, Catalyst, Fathom, and Folio showing the same Axiom observation/report/receipt boundary without moving app or Helm responsibilities into Axiom.

### Changed
- Recentered the public release story on Axiom as the truth, lineage, verifier, and decoder-learning layer.
- Documented Helm/app contract probes as downstream boundary guidance, not additional Axiom release scope.
- Updated crate metadata for the v0.15 release surface and the current `organism-pack` 1.9.0 runtime intent dependency.

### Fixed
- Aligned `cz doctor` and generated temporary WASM crates with the repo MSRV, Rust 1.94.

### Verified
- `just status`, `just test`, `just lint`, `cargo package --allow-dirty`, and `cargo publish --dry-run --allow-dirty` passed on 2026-05-20.
- Updated yanked transitive `enumset` 1.1.12 to 1.1.13; package and publish dry-run no longer report the yanked dependency warning.
- Published `axiom-truth` 0.15.0 to crates.io on 2026-05-20.

## [0.8.1] - 2026-05-07

### Added
- `intent` module: `compile_intent(&TruthDocument) -> IntentPacket` and `compile_intent_from_source(&str)`. Axiom now owns the bridge from Truth-shaped governance to organism's runtime contract.
- `organism-pack` dependency — needed to produce `IntentPacket`. When publishing 0.8.1, a compatible `organism-pack` release must be on crates.io first.

### Changed
- Architectural inversion: `axiom-truth → organism-pack` replaces the prior `organism → axiom-truth` arrow. organism's runtime no longer parses, mentions, or knows about Truth in any form. Callers compile via axiom, then hand the resulting `IntentPacket` to `organism_runtime::Runtime::admit_intent`.

## [0.5.1] - 2026-04-19

### Added
- AGENTS.md — canonical agent entrypoint documenting philosophy, public surface, rules, architecture, and workflows

### Changed
- Aligned with Converge v3.3.1 provider API (narrowed downstream surface, new chat capability contracts)
- Resolved all clippy pedantic warnings: `format_push_string`, `assigning_clones`, `ref_option`, `collapsible_if`, `map_unwrap_or`, `vec_init_then_push`, `default_trait_access`, `unnecessary_wraps`
- Refactored justfile to axiom-specific tasks (removed converge infrastructure commands)
- Inlined Cedar policy test fixture in policy_lens.rs (was missing external file)
- Moved live LLM-facing imports from `converge-core` to `converge-provider-api`

### Verified
- 120 tests passing (111 lib + 2 CLI + 7 doctests), 0 failures
- Clean clippy pedantic, clean fmt
- Clean build against Converge v3.3.1 provider capability contract

## [0.4.1] - 2026-04-15

### Changed
- Renamed crate from `converge-axiom` to `axiom-truth` — now a self-contained project
- Standalone single-crate repo (no longer part of converge workspace)
- Depends on Converge provider surfaces rather than the engine crate

[Unreleased]: https://github.com/Reflective-Lab/axiom/compare/v0.15.0...HEAD
[0.15.0]: https://github.com/Reflective-Lab/axiom/compare/v0.8.1...v0.15.0
[0.8.1]: https://github.com/Reflective-Lab/axiom/compare/v0.5.1...v0.8.1
[0.5.1]: https://github.com/Reflective-Lab/axiom/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Reflective-Lab/axiom/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/Reflective-Lab/axiom/releases/tag/v0.4.1
