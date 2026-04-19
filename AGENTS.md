# Axiom — The Truth Layer

This is the canonical agent entrypoint — all agents (Claude, Codex, Gemini, or otherwise) start here. Long-form documentation lives in `kb/`.

## Philosophy

Axiom is Layer 1.5 — between user intent and Converge's commit boundary. Read `kb/Philosophy/Why Axiom.md` and `kb/Philosophy/Truth-Driven Development.md`.

Specifications must be validated, simulated, and compiled BEFORE they reach Converge. Business rules written as `.truths` files become enforceable WASM invariants.

## The Knowledgebase

`kb/` is an Obsidian vault. It is THE documentation.

**Do NOT read the entire kb on startup.** Lazy-load:

1. Read `kb/Home.md` only when you need to find something.
2. Follow ONE wikilink to the specific page you need.
3. Never bulk-read `kb/`.

## Stack

| Layer | Technology |
|---|---|
| Truth validation & codegen | Rust (Edition 2024, rust-version 1.94) |
| Converge contract | `converge-provider-api`, `converge-provider` |
| Task runner | just |
| CLI tool | `cz` (workspace orchestrator) |

## Public Surface

| Artifact | Purpose |
|---|---|
| `axiom-truth` crate | Core validation, codegen, simulation, policy lens |
| `cz` CLI | Developer-facing orchestrator for truths workflow |
| `.truths` format | Business specification language (Gherkin + governance) |

## Build

```bash
just build      # Build (release)
just test       # Run tests
just lint       # Format + clippy pedantic
just focus      # Session opener
just sync       # Team sync
```

## Rules

These are not suggestions.

- No `unsafe` code. Ever.
- Specifications are the source of truth, code is derived.
- Generated code is read-only and always-regenerated from specs.
- No feature flags. No backwards-compat shims. Converge owns versioning.
- `.truths` files are immutable once committed; new versions are new files.
- `just lint` clean before considering work done.
- Use the narrow Converge capability contract directly (`converge-provider-api`, `converge-provider`). No wrapper layers.
- Before validating a spec type, check `kb/Concepts/Truth Documents.md` — the schema is authoritative.
- Do not depend on Converge internal crates (`converge-core`, `converge-runtime`, `converge-analytics`, etc).
- Simulation results must be deterministic and reproducible.

## Architecture

The validation pipeline has three stages:

1. **Parsing** (`gherkin`, `truths`) — extract Gherkin scenarios and governance blocks
2. **Validation** (`predicate`, `guidance`, `policy_lens`) — semantic checks, quality feedback, policy coverage
3. **Codegen** (`codegen`, `compile`) — generate WASM invariants, verify compilability

See `kb/Architecture/System Overview.md` for module responsibilities and data flows.

## Workflows

| Workflow | Purpose |
|---|---|
| `/focus` / `just focus` | Session opener — orient yourself, see team activity |
| `/sync` / `just sync` | Team sync — who did what, PRs waiting, unclaimed issues |
| `/next` | Pick next task from current milestone |
| `/dev` | Start local development environment |
| `/fix` | Fix a GitHub issue by number |
| `/check` | Run lint, check, and tests |
| `/pr` | Create a pull request |
| `/ticket` | Create an agent-ready issue |
| `/review` | Review a PR |
| `/wip` | Save and push WIP |
| `/done` | End-of-session — update milestones, record what moved |
| `/deploy` | Deploy to staging or production |
| `/audit` | Security, dependency, compliance, and drift audit |
| `/help` | Show available workflows |

### Daily habit

```
Morning:    /focus → /sync → /next
Work:       /fix, /check, /pr
Evening:    /done
Monday:     /audit
Anytime:    /help
```

## Milestones

Read `MILESTONES.md` at the start of every session. Scope all work to the current milestone. See `~/dev/work/EPIC.md` for the strategic context (no dedicated Axiom epic yet) and `~/dev/work/MILESTONES.md` for the cross-project rollup.
