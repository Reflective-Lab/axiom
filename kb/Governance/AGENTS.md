---
source: mixed
---
# AGENTS - converge-personas

Short entrypoint for agent work. Load deeper docs only when needed.

## Quick Start

- Read `README.md`
- Keep contract/persona edits scoped and internally consistent
- Single eval test: `./evals/run-eval.sh system-architect`

## Load On Demand

- Shared workflow: `../../AGENTS.md`
- Governance docs index: `README.md`
- Eval/contract files: `contracts/`, `evals/`, `personas/`

## Local Rules

- If present, load `.cursorrules`, `.cursor/rules/`, and `.github/copilot-instructions.md`

## Version Control

- Use `jj` for day-to-day operations (commits, branches, status, log)
- Fall back to `git` only when `jj` cannot handle it (e.g. specific remote operations)
