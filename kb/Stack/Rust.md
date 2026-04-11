---
tags: [stack]
---
# Rust

Converge is Rust-first. Orchestration, domain logic, policy enforcement, and integrations are all Rust.

## Conventions

- Edition 2024, rust-version 1.90
- `unsafe_code = "forbid"` — no exceptions
- Clippy pedantic with allowed exceptions at workspace level
- All deps use `workspace = true`
- `[lints] workspace = true` in every crate

## Toolchain

- `rustup` for toolchain management
- `cargo` for builds
- `just` (task runner) wraps common cargo commands
- `cargo-deny` for supply chain audits

## Key Commands

```bash
just build          # cargo build --release
just build-quick    # cargo build --profile quick-release
just test           # cargo test --all-targets
just lint           # cargo fmt --check && cargo clippy -- -D warnings
just fix-lint       # auto-fix lint issues
just deny           # cargo deny check
```

## Style

- Concise, production-quality code
- No unnecessary comments or docstrings
- Prefer simple, direct solutions over abstractions
- Module-level `//!` docs in `lib.rs` files explain the crate's purpose and invariants

See also: [[Architecture/Dependency Rules]], [[Architecture/Purity Rules]]
