---
tags: [workflow, claude]
---
# Working with Claude

This project has two layers of automation: **Claude Code skills** (slash commands) and **Justfile recipes** (shell commands). They do different things. Use the right one.

## When to Use Which

| I want to... | Use | Why |
|---|---|---|
| Build the project | `just build` | Deterministic shell command |
| Run tests | `just test` | Deterministic shell command |
| Run clippy | `just lint` | Deterministic shell command |
| Orient myself at session start | `/focus` | Reads kb, checks build, shows team activity |
| Fix a GitHub issue end-to-end | `/fix 42` | Multi-step: read issue, branch, code, test, PR |
| Create a well-defined ticket | `/ticket add risk agent` | Needs to explore code, write requirements |
| Review a PR | `/review 17` | Reads diff, reasons about security/correctness |
| Run 3 tasks in parallel | `/parallel a \| b \| c` | Launches agents in worktrees |
| Save and push WIP | `/wip` | Multi-step git workflow |
| Capture end-of-session state | `/checkpoint` | Reads git state, updates kb, writes summary |

**Rule of thumb:** if it's a single deterministic command, use `just`. If it requires reading, thinking, or multi-step orchestration, use a skill.

## The Knowledgebase and Claude

Claude reads `kb/` pages when it needs context. The `/focus` skill starts by reading `kb/Home.md`. The `/ticket` skill reads relevant kb pages to write better issues.

When Claude learns something during a session that should be preserved:
- Code changes go in code
- Everything else goes in `kb/`

The kb is for humans AND agents.

See also: [[Workflow/Daily Journey]], [[Workflow/Working with Codex]], [[Workflow/Working with Gemini]]
