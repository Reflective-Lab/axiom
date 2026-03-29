# Development

## Prerequisites

- Rust 1.90+ (`rustup update`)
- [just](https://github.com/casey/just) (`brew install just`)
- Optional: [jj (Jujutsu)](https://martinvonz.github.io/jj/) for version control
- Optional: CUDA / Vulkan / WGPU for GPU acceleration
- Optional: [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) for supply chain auditing

## Quick Start

```bash
git clone https://github.com/Reflective-Labs/converge.zone.git
cd converge.zone

just build-quick   # fast iteration build
just test          # run tests
just lint          # format + clippy
```

## Running Examples

```bash
just examples                        # list all examples
just example hello-convergence       # run the hello-convergence example
just example custom-agent            # implement your own agent
just example meeting-scheduler       # domain pack with constraint agents
```

See [examples/README.md](examples/README.md) for the full list.

## Workspace Structure

```
crates/
├── traits/        # Public contract — partners implement these
├── core/          # Convergence engine
├── provider/      # Remote LLM providers (Anthropic, OpenAI, Gemini, ...)
├── domain/        # Domain packs (scheduling, routing, drafting, ...)
├── experience/    # Event-sourced audit store
├── knowledge/     # Vector knowledge base
├── optimization/  # Constraint solving (OR-Tools)
├── policy/        # Cedar policy engine
├── llm/           # Local LLM inference (Burn)
├── analytics/     # ML/analytics agents
├── tool/          # Development toolchain (Gherkin, JTBD)
├── runtime/       # HTTP/gRPC execution service
├── remote/        # gRPC client to runtime
└── application/   # Reference distribution
```

## Build Profiles

| Profile | Use case | Command |
|---------|----------|---------|
| `quick-release` | Daily development | `just build-quick` |
| `ci` | GitHub Actions | `just build-ci` |
| `release` | Production / publish | `just build` |

## Git Workflow

### Using git worktrees for parallel work

Worktrees let you work on multiple branches simultaneously without stashing.
Each worktree is a separate checkout sharing the same `.git` directory.

```bash
# Create a worktree for a feature branch
just worktree fix-auth
# → creates ../converge-fix-auth/ on branch fix-auth

# Work in the worktree
cd ../converge-fix-auth
just test

# When done, clean up
just worktree-rm fix-auth
```

This is especially useful for:
- Running tests on one branch while developing on another
- Code review checkouts without disrupting your working tree
- Parallel agent work (each agent gets its own worktree)

### Using jj (Jujutsu) for version control

[Jujutsu](https://martinvonz.github.io/jj/) is a Git-compatible VCS with
better ergonomics for stacking changes and conflict resolution.

```bash
# Initialize jj in an existing git repo
jj git init --colocate

# Basic workflow
jj new -m "feat: add custom provider"   # start a new change
# ... edit files ...
jj status                                # see what changed
jj diff                                  # review changes
jj squash                                # fold into parent

# Stacking changes (jj's strength)
jj new -m "test: add provider tests"     # stack another change
jj log                                   # see the change graph

# Push to GitHub
jj git push
```

Key advantages over plain git:
- **No staging area** — every file save is automatically tracked
- **First-class conflicts** — resolve at your pace, not during rebase
- **Change stacking** — easy to reorder, split, and squash changes
- **Undo anything** — `jj undo` works for any operation

```bash
# Quick jj commands via just
just jj-status
just jj-log
just jj-new "feat: add something"
just jj-squash
just jj-push
```

## Supply Chain Security

```bash
# Audit dependencies for vulnerabilities and license compliance
just deny

# Advisories only (faster)
just deny-advisories

# Validate repository assurance docs and compliance declarations
just compliance-check
```

## Publishing to crates.io

Nine crates are publishable in dependency order:

1. `converge-traits`
2. `converge-core`
3. `converge-provider`
4. `converge-experience`
5. `converge-knowledge`
6. `ortools-sys`
7. `converge-optimization`
8. `converge-domain`
9. `converge-tool`

```bash
# Validate readiness
just publish-dry-run
```
