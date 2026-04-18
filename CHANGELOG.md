# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- AGENTS.md — canonical agent entrypoint documenting philosophy, public surface, rules, architecture, and workflows aligned with converge/organism

### Changed
- Refactored justfile to axiom-specific tasks (removed converge infrastructure commands)
- Updated MILESTONES.md with v0.5 Foundation Hardening milestone (deadline 2026-05-15)
- Inlined Cedar policy test fixture in policy_lens.rs (was missing external file)

### Verified
- 113 tests passing (111 lib + 2 CLI), 0 failures
- Clean build with converge 3.2.1 (matches organism)
- Integration with organism verified (both on stable 3.2.1)

## [0.4.1] - 2026-04-15

### Changed
- Renamed crate from `converge-axiom` to `axiom-truth` — now a self-contained project
- Standalone single-crate repo (no longer part of converge workspace)
- Depends on `converge-core` and `converge-provider` 3.2.1 from crates.io

[Unreleased]: https://github.com/Reflective-Lab/axiom/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/Reflective-Lab/axiom/releases/tag/v0.4.1
