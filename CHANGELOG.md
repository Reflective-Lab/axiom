# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed
- **Knowledge lifecycle pack** (`packs::knowledge`) — moved to `organism-domain`. Includes all knowledge agents (`SignalCaptureAgent`, `HypothesisGeneratorAgent`, etc.), invariants (`ClaimHasProvenanceInvariant`, etc.), and evals (`ClaimProvenanceEval`, `ExperimentMetricsEval`).

## [3.0.2] - 2026-04-11

### Added
- 15 additional proof tests for contract regression gate
- Extract `converge-auth`, `converge-ledger`, `converge-nats`, `converge-consensus` crate boundaries from runtime
- Extract `converge-observability` crate boundary from runtime

### Removed
- All dead code from provider, runtime, knowledge, domain
- OCR, photos, screenshots from `converge-knowledge` (moved to organism)
- OCR source from `converge-provider` (moved to organism-intelligence)
- Billing module from `converge-runtime` (moved to organism)
- Business intelligence moved to organism

## [3.0.1] - 2026-04-10

### Changed
- Hardened runtime and policy control surfaces (fail-closed defaults, auth enforcement, logging)

## [3.0.0] - 2026-04-10

### Added
- **ADR-001**: 6 canonical public crates (`converge-pack`, `converge-provider-api`, `converge-model`, `converge-kernel`, `converge-protocol`, `converge-client`)
- **ADR-002**: Single truth pipeline (`Observation -> DraftProposal -> ValidatedProposal -> Fact`)
- **ADR-003**: Pack authoring contract (`Suggestor`, `Validator`, `Invariant`, `CriterionEvaluator`, `Pack`)
- **ADR-004**: Contract stabilization and regression gate freeze
- `converge-storage` crate — object storage abstraction (local, S3, GCS)
- `architecture/ARCHITECTURE.md` as canonical architecture reference

### Changed
- `converge-traits` deprecated — replaced by `converge-pack` and `converge-provider-api`
- `converge-remote` demoted to compatibility CLI — replaced by `converge-client`
- Rust edition 2024, rust-version 1.94

## [2.1.2] - 2026-04-08

### Added
- `converge-tool` publishing enabled for crates.io

## [2.1.1] - 2026-04-07

### Added
- `converge-storage` module and workspace integration
- Embedded Gemma GGUF inference via llama.cpp (`gemma-inference` example)

## [2.1.0] - 2026-04-05

### Added
- Kong AI Gateway provider (`KongGateway`, `KongRoute`)
- CI workflow fixes and stabilization

## [2.0.0] - 2026-04-01

### Added
- **Truth execution framework**: `TruthDefinition`, `TruthKind`, `TruthCatalog` trait
- **Criterion evaluation**: `CriterionEvaluator` trait, four-way `CriterionResult`
- **Pack-scoped execution**: `engine.register_in_pack(pack_id, agent)`, `TypesRootIntent.active_packs`
- `run_with_types_intent_and_hooks()` — single entry point for application-level truth execution
- `StopReason::HumanInterventionRequired` with typed approval references
- `ContextStore` trait for durable context persistence across runs
- `ExperienceEventObserver` for run-scoped event capture
- crates.io publishing enabled for 9 crates

### Changed
- `ProposedFact` now carries `confidence: f64` and `provenance: String`
- `AgentEffect` changed from enum to struct `{ facts, proposals }`
- `TryFrom<ProposedFact> for Fact` for type-safe promotion with validation
- Agent trait signatures: `accepts(&self, ctx: &dyn ContextView)` instead of `&Context`
- `converge-traits` is canonical type owner; `converge-core` re-exports
- Copyright updated to Reflective Labs, all SPDX headers normalized to MIT
- Version bumped to 2.0.0 (breaking: ProposedFact, AgentEffect, Agent trait)

### Infrastructure
- Dockerfile, compose.yaml, deploy scripts for GCP Cloud Run
- GitHub CI, security workflows, dependabot

## [1.1.0] - 2024-03-20

### Added
- Initial public release of Converge.zone
- All 15 crates with basic functionality
- CI/CD pipeline configuration
- Documentation and examples

## [1.0.0] - 2024-01-15

### Added
- Initial private development version
- Basic agent runtime
- Core traits and abstractions

[Unreleased]: https://github.com/Reflective-Labs/converge.zone/compare/v3.0.2...HEAD
[3.0.2]: https://github.com/Reflective-Labs/converge.zone/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/Reflective-Labs/converge.zone/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/Reflective-Labs/converge.zone/compare/v2.1.2...v3.0.0
[2.1.2]: https://github.com/Reflective-Labs/converge.zone/compare/v2.1.1...v2.1.2
[2.1.1]: https://github.com/Reflective-Labs/converge.zone/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/Reflective-Labs/converge.zone/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/Reflective-Labs/converge.zone/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/Reflective-Labs/converge.zone/releases/tag/v1.1.0
[1.0.0]: https://github.com/Reflective-Labs/converge.zone/releases/tag/v1.0.0
