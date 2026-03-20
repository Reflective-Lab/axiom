# Converge crate standard Justfile
# Copy to your crate root and adjust as needed.
# Minimum targets: default, install, build, test, lint, check, fmt, doc, audit

default: check

# Fetch dependencies
install:
    cargo fetch

# Build all workspace members
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Clippy + format check
lint:
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Build docs
doc:
    cargo doc --workspace --no-deps

# Security audit (requires cargo-audit)
audit:
    cargo audit

# Full pre-commit check: format, lint, test, doc
check: lint test doc

# --- jj workflow ---

# Start a new spec-driven feature branch
start name:
    jj new main -m "{{name}}"
    jj bookmark create {{name}} -r @

# Commit current work
commit message:
    jj commit -m "{{message}}"

# Push current feature bookmark
push bookmark:
    jj git push --bookmark {{bookmark}}

# Land a feature bookmark onto main
land bookmark:
    jj git fetch
    jj rebase -d main
    jj bookmark set main -r @-
    jj git push --bookmark main
    jj bookmark delete {{bookmark}}

# Sync with remote main
sync:
    jj git fetch
    jj rebase -d main

# Show current work status
status:
    jj log --limit 10

# Publish a crate
publish crate:
    cargo publish -p {{crate}}
