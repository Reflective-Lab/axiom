# Axiom — Truth Layer Development
# Install: brew install just  |  cargo install just
# Usage:   just --list

set dotenv-load := true

# ── Build ──────────────────────────────────────────────────────────────

# Build workspace (release)
build:
    cargo build --release

# Build workspace (fast iteration)
build-quick:
    cargo build --profile quick-release

# Build workspace without release artifacts
check:
    cargo check

# ── Test ───────────────────────────────────────────────────────────────

# Run tests
test:
    cargo test --all-targets

# Run tests with output
test-verbose:
    cargo test --all-targets -- --nocapture

# Run a single test by name
test-one name:
    cargo test {{name}} -- --nocapture

# ── Lint & Format ─────────────────────────────────────────────────────

# Check formatting and clippy
lint:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings

# Check that Cargo.lock has no pending compatible dependency updates
deps-fresh:
    @output="$(CARGO_TERM_COLOR=never cargo update --workspace --dry-run 2>&1)"; \
    printf '%s\n' "$output"; \
    if ! printf '%s\n' "$output" | grep -Fq "Locking 0 packages"; then \
        echo ""; \
        echo "Dependency lockfile is stale. Run 'cargo update --workspace' and commit Cargo.lock."; \
        exit 1; \
    fi

# Auto-fix lint issues
fix-lint:
    cargo clippy --fix --allow-staged --allow-dirty --allow-no-vcs
    cargo fmt

# Format only
fmt:
    cargo fmt

# ── Docs ───────────────────────────────────────────────────────────────

# Generate workspace docs
doc:
    cargo doc --no-deps

# Open docs in browser
doc-open:
    cargo doc --no-deps --open

# ── CLI ────────────────────────────────────────────────────────────────

# Run cz help
help-cz:
    cargo run --bin cz -- --help

# Run cz doctor
doctor:
    cargo run --bin cz -- doctor

# Run cz validate
validate:
    cargo run --bin cz -- validate

# ── Clean ──────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# ── Git ────────────────────────────────────────────────────────────────

# Install git hooks (pre-commit fmt/clippy + pre-push dependency freshness)
git-hooks:
    git config core.hooksPath .githooks
    @echo "Git hooks installed — .githooks/pre-commit and .githooks/pre-push are active"

# Repo state and recent commits
sync:
    @git status --short
    @git log --oneline -5

# Build health
status:
    @cargo check

# Compatibility name for older local habits
git-sync: sync

# Compatibility name for older local habits
git-status: status

# ── Workflow ───────────────────────────────────────────────────────────

# Session opener — repo health + recent activity
focus:
    @git status --short
    @git log --oneline -5
    @just --list

# Compatibility name for older local habits
wow-focus: focus

# ── Info ───────────────────────────────────────────────────────────────

# Show module structure
modules:
    @echo "Axiom modules:"
    @echo "  gherkin         – LLM-powered validation of .truths specs"
    @echo "  truths          – Governance block parsing"
    @echo "  codegen         – WASM invariant code generation"
    @echo "  compile         – Rust → WASM compilation"
    @echo "  predicate       – Gherkin step → semantic extraction"
    @echo "  simulation      – Pre-flight convergence readiness"
    @echo "  guidance        – LLM + heuristic quality feedback"
    @echo "  policy_lens     – Cedar policy coverage analysis"
    @echo "  jtbd            – Jobs-to-be-Done metadata extraction"
