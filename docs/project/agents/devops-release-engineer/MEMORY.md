# Dex Tanaka — Tacit Knowledge

## Paperclip API Patterns
- Checkout requires: `{"agentId": "...", "expectedStatuses": ["todo", "in_progress"]}`
- Checkout returns 409 if executionRunId already set (even for own agent in same run)
- Workaround: use PATCH to update status directly when checkout conflicts on own issues
- Comments: `POST /api/issues/{id}/comments` with `{"body": "..."}`
- Issue update: `PATCH /api/issues/{id}` with fields to change
- Company ID: `ad34cffc-6c98-47c9-b629-2aed6f694149`
- My agent ID: `825c119c-c656-4dd8-89b8-9d4be5dce5d5`

## Ecosystem Layout
- 22 converge-* repos under `/Users/kpernyer/repo/all-converge/`
- 13 are Rust crates (have Cargo.toml)
- Non-Rust repos: converge-android, converge-business, converge-ios, converge-ledger, converge-personas, converge-policy, converge-project, converge-runtime, converge-www

## Infrastructure Status (as of 2026-03-13)
- Justfile template: `converge-project/templates/Justfile.template`
- All 13 Rust crates have Justfiles
- jj: All 22/22 repos initialized (colocated)
- cargo-audit: 0.22.1 installed
- protoc: installed via Homebrew (libprotoc 34.0)
- MSRV: 1.85 base, 1.88 for provider/knowledge/llm/analytics (bumped for time fix)
- Rust toolchain: 1.94.0 installed
- Git remotes: All 22 repos use SSH (switched from HTTPS in 4th heartbeat)
- CI/CD: GitHub Actions deployed to converge-traits, converge-core, converge-provider

## Key Files
- `$AGENT_HOME` = `agents/devops-release-engineer/`
- Daily notes: `agents/devops-release-engineer/memory/YYYY-MM-DD.md`
- Knowledge graph: `agents/devops-release-engineer/life/`
- Plans go in project root `plans/` (shared with other agents)

## Open Issues (mine)
- All assigned issues completed. No open items.

## Filed for Others
- REF-30 — Replace serde_yml in converge-provider (Eli) — DONE
- REF-36 — Migrate LlmProvider deprecation in converge-application (Eli) — todo

## Audit Summary (2026-03-12, 6th heartbeat)
- 3 GREEN (core, traits, optimization)
- 7 YELLOW (warnings only — unmaintained deps, not vulnerabilities)
- 2 RED (provider: lru/upstream via lancedb, experience: 3 vulns via surrealdb)
- time CVE (RUSTSEC-2026-0009) fully resolved across all crates
- serde_yml removed from provider locally; still on crates.io published version

## Test Health (2026-03-12, 6th heartbeat)
- 12/12 crates build and pass tests
- 78 tests passing, 0 failures, 62 ignored
