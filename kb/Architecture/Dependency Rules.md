---
tags: [architecture]
---
# Dependency Rules

Dependencies flow downward only. Core never imports from provider. Provider never imports from domain. Violating this is a build error, not a style issue.

## Rules

1. All deps use `workspace = true` — never inline versions in crate `Cargo.toml` files
2. All crates use `[lints] workspace = true`
3. `unsafe_code = "forbid"` everywhere — no exceptions
4. Edition 2024, rust-version 1.90
5. Internal path deps include both `path` and `version` for crates.io compatibility
6. Clippy pedantic with allowed exceptions defined at workspace level

## What This Means in Practice

- Adding a dependency to a crate? Add it to `[workspace.dependencies]` in the root `Cargo.toml` first, then reference it with `workspace = true` in the crate.
- Need a new internal dependency? Check the [[Architecture/Crate Map|dependency graph]]. If the edge doesn't exist, think hard about whether it should.
- `unsafe` is forbidden. Find another way.

See also: [[Architecture/Crate Map]], [[Architecture/Purity Rules]]
