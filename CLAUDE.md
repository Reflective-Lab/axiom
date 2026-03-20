# Converge Agent OS — Workspace Conventions

## Build Commands
```bash
make build          # cargo build --release
make build-quick    # cargo build --profile quick-release (faster iteration)
make lint           # cargo fmt --check && cargo clippy --all-targets -- -D warnings
make fix-lint       # auto-fix lint issues
make test           # cargo test --all-targets (default members only)
make test-all       # cargo test --all-targets --workspace (includes analytics, llm, runtime)
make doc            # cargo doc --no-deps --workspace
make publish-dry-run # validate crates.io readiness
```

## Workspace Layout
All crates live under `crates/`. The workspace root `Cargo.toml` centralizes:
- **Versions**: all crates at `1.1.0`
- **Dependencies**: shared dep versions in `[workspace.dependencies]`
- **Lints**: clippy pedantic with allowed exceptions in `[workspace.lints]`
- **Profiles**: quick-release, ci, release, bench

## Dependency Graph (leaf → root)
```
converge-traits          (no deps)
converge-core            (no internal deps)
converge-provider        → core, traits
converge-domain          → core, provider
converge-experience      → core
converge-knowledge       (no internal deps)
ortools-sys              (no deps, FFI)
converge-optimization    → ortools-sys (optional)
converge-analytics       → core, domain, provider
converge-llm             → core, domain, provider (optional)
converge-policy          → core
converge-tool            → core, provider
converge-remote          (no internal deps, gRPC client)
converge-runtime         → core, provider, tool
converge-application     → core, provider, domain, tool
```

## Publish Order (crates.io)
1. converge-traits
2. converge-core
3. converge-provider
4. converge-experience
5. converge-knowledge
6. ortools-sys
7. converge-optimization
8. converge-domain
9. converge-tool

Crates marked `publish = false`: analytics, llm, policy, runtime, remote, application

## Conventions
- All deps use `workspace = true` — never inline versions in crate Cargo.tomls
- All crates use `[lints] workspace = true`
- `unsafe_code = "forbid"` everywhere
- Edition 2024, rust-version 1.90
- Internal path deps include both `path` and `version` for crates.io compatibility
