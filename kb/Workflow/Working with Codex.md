---
tags: [workflow, codex]
---
# Working with Codex

Start from the root `CODEX.md` entrypoint, then use this page for workflow guidance. Codex does not use Claude slash commands directly. Instead, it uses `AGENTS.md`, the knowledgebase, the shared `just` recipes, and plain-language workflow requests.

## What to Read First

1. `AGENTS.md` — shared project rules, architecture, and public API
2. `kb/Home.md` — index, follow one relevant link at a time
3. The specific `kb/` page your task needs

Do not bulk-read the whole knowledgebase.

## Shared Automation

```bash
just focus     # Session opener — repo health + recent activity
just sync      # Team sync — PRs, issues, recent commits
just status    # Build health, test results
```

Use those when you want deterministic output from the repo itself.

## Workflow Equivalents

| Claude workflow | Use with Codex |
|---|---|
| `/focus` | "Run the focus workflow" or `just focus` |
| `/sync` | "Run a team sync" or `just sync` |
| `/status` | "Run the project status workflow" or `just status` |
| `/fix 42` | "Fix issue 42 end to end: read issue, make smallest safe change, run `just check && just test && just lint`, prepare PR" |
| `/ticket add risk agent` | "Create an agent-ready GitHub issue for adding the risk agent" |
| `/review 17` | "Review PR 17; findings first, with blockers, suggestions, and questions" |
| `/checkpoint` | "Write a session checkpoint: what moved, what kb pages changed, what the next teammate needs to know" |

## Knowledgebase Discipline

When Codex learns something that should outlive the session:
- Code changes go in code
- Architecture and process knowledge go in `kb/`

See also: [[Workflow/Daily Journey]], [[Workflow/Working with Claude]], [[Workflow/Working with Gemini]]
