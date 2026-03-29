# Converge Agent OS — Workspace Conventions

## Build Commands
```bash
just build          # cargo build --release
just build-quick    # cargo build --profile quick-release (faster iteration)
just lint           # cargo fmt --check && cargo clippy --all-targets -- -D warnings
just fix-lint       # auto-fix lint issues
just test           # cargo test --all-targets (default members only)
just test-all       # cargo test --all-targets --workspace (includes analytics, llm, runtime)
just doc            # cargo doc --no-deps --workspace
just deny           # cargo deny check (supply chain audit)
just publish-dry-run # validate crates.io readiness
just example hello-convergence  # run an example
```

## Workspace Layout
All crates live under `crates/`. Examples live under `examples/`. The workspace root `Cargo.toml` centralizes:
- **Versions**: all crates at `1.1.0`
- **Dependencies**: shared dep versions in `[workspace.dependencies]`
- **Lints**: clippy pedantic with allowed exceptions in `[workspace.lints]`
- **Profiles**: quick-release, ci, release, bench

## Dependency Graph (leaf → root)
```
converge-traits          (no deps)
converge-core            (no internal deps)
converge-mcp             (no internal deps)
converge-provider        → core, traits, mcp
converge-domain          → core, provider
converge-experience      → core
converge-knowledge       → mcp (server feature)
ortools-sys              (no deps, FFI)
converge-optimization    → ortools-sys (optional)
converge-analytics       → core, domain, provider
converge-llm             → core, domain, provider (optional)
converge-policy          → core
converge-tool            → core, provider
converge-remote          (no internal deps, gRPC client)
converge-runtime         → core, provider, tool
converge-application     → core, provider, domain, tool, mcp, knowledge (feature-gated)
```

## Publish Order (crates.io)
1. converge-traits
2. converge-core
3. converge-mcp
4. converge-provider
5. converge-experience
6. converge-knowledge
7. ortools-sys
8. converge-optimization
9. converge-domain
10. converge-tool

Crates marked `publish = false`: analytics, llm, policy, runtime, remote, application

## Conventions
- All deps use `workspace = true` — never inline versions in crate Cargo.tomls
- All crates use `[lints] workspace = true`
- `unsafe_code = "forbid"` everywhere
- Edition 2024, rust-version 1.90
- Internal path deps include both `path` and `version` for crates.io compatibility
- Examples are standalone crates under `examples/` with `publish = false`

## Git Workflow
- Use `just worktree <branch>` for parallel work (git worktrees)
- Use `jj` (Jujutsu) for version control when available
- See DEVELOPMENT.md for details

## Schema & Proto Files
- All `.proto` files live in `schema/proto/` (single source of truth)
- OpenAPI specs go in `schema/openapi/`
- Build scripts reference protos via `CARGO_MANIFEST_DIR`-relative paths

## Internal Docs
- `/docs/project/` and `/docs/personas/` are gitignored (internal only)
- Partner-facing docs: README.md, DEVELOPMENT.md, CONTRIBUTING.md, SECURITY.md
- Architecture docs: `architecture/ARCHITECTURE.md` + crate-level `//!` comments in lib.rs files
