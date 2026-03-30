# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Platform Primitives (bb94361)
- **Truth execution framework**: `TruthDefinition`, `TruthKind { Job, Policy, Invariant }`, `TruthCatalog` trait — typed specifications for jobs, policies, and invariants that applications declare and the runtime executes
- **Criterion evaluation**: `CriterionEvaluator` trait, `CriterionResult { Met { evidence }, Blocked { reason, approval_ref }, Unmet { reason }, Indeterminate }`, `CriterionOutcome` — four-way typed results with structured evidence, replacing boolean success checks
- **Pack-scoped execution**: `engine.register_in_pack(pack_id, agent)`, `TypesRootIntent.active_packs` — agents belong to packs, intents activate specific packs per truth
- **`run_with_types_intent_and_hooks()`** — single entry point for application-level truth execution with criterion evaluation and event observation
- **`StopReason::HumanInterventionRequired { criteria, approval_refs }`** — honest stopping when the system needs human input, with typed references to what's blocked
- **`ContextStore` trait** (GAT async) — durable context persistence across runs with `load_context`/`save_context`, plus `DynContextStore` for runtime polymorphism
- **`ExperienceEventObserver`** — run-scoped event capture without requiring a full ExperienceStore

### Changed — Type System Reconciliation (5c34d8c)
- **`ProposedFact`**: added `confidence: f64` and `provenance: String` — every proposal now carries confidence and origin for the promotion gate
- **`AgentEffect`**: changed from enum (`AddFacts | Propose | Nothing`) to struct `{ facts, proposals }` — agents can emit both validated facts and proposals in a single execution
- **`TryFrom<ProposedFact> for Fact`**: type-safe promotion with validation (rejects NaN/out-of-range confidence)
- **Agent trait signatures**: `accepts(&self, ctx: &dyn ContextView)` instead of `&Context` — agents work against a trait, not a concrete type
- **converge-traits** is now the canonical type owner; converge-core re-exports and provides concrete implementations

### Infrastructure
- Dockerfile, compose.yaml, deploy scripts for GCP Cloud Run
- GitHub CI, security workflows, dependabot
- Copyright updated to Reflective Labs (kenneth@reflective.se)
- All SPDX headers normalized to MIT
- Version bumped to 2.0.0 (breaking: ProposedFact, AgentEffect, Agent trait signatures)

## [1.1.0] - 2024-03-20

### Added
- Initial public release of Converge.zone
- All 15 crates with basic functionality
- CI/CD pipeline configuration
- Documentation and examples

### Changed
- N/A (initial release)

### Fixed
- N/A (initial release)

## [1.0.0] - 2024-01-15

### Added
- Initial private development version
- Basic agent runtime
- Core traits and abstractions

[Unreleased]: https://github.com/Reflective-Labs/converge.zone/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/Reflective-Labs/converge.zone/releases/tag/v1.1.0
[1.0.0]: https://github.com/Reflective-Labs/converge.zone/releases/tag/v1.0.0