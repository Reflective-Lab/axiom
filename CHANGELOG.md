# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/Reflective-Lab/axiom/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/Reflective-Lab/axiom/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/Reflective-Lab/axiom/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/Reflective-Lab/axiom/releases/tag/v0.4.1
