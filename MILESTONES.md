# Milestones

## Current: v3.1 — Documentation & Contract Hardening
Deadline: 2026-04-26

- [x] Fix dead `docs/` links in README.md and SECURITY.md (8 broken refs)
- [x] Update SECURITY.md supported versions to 3.0.x
- [x] Update CHANGELOG.md through v3.0.2
- [x] Update CLAUDE.md dep graph, version, rust-version to match reality
- [x] Update DEVELOPMENT.md workspace layout and publish order
- [x] Update README.md crate table to 6 canonical crates
- [x] Add missing examples to examples/README.md (4 unlisted)
- [x] Fix proto copyright headers (Aprio One AB -> Reflective Labs, MIT license)
- [x] Add doc comments to converge-client public API
- [x] Add `//!` crate doc to converge-analytics

## Completed: v3.0 — Contract Stabilization
Completed: 2026-04-11

- [x] ADR-001: Canonical public crates (pack, provider-api, model, kernel, protocol, client)
- [x] ADR-002: Single truth pipeline
- [x] ADR-003: Pack authoring contract
- [x] ADR-004: Contract stabilization and freeze
- [x] Extract auth, ledger, nats, consensus, observability crate boundaries
- [x] 26 proof tests, regression gate frozen
- [x] Knowledge lifecycle moved to organism-domain
