---
tags: [workflow, cheat-sheet]
---
# Daily Journey

Your day, start to finish. Each phase has a skill or script.

## Morning

```
/focus              Orient yourself — kb, build health, team activity
/sync               What did the team do? PRs waiting? Unclaimed issues?
```

If it's your first session, `/focus` will point you to the key kb pages. Read them.

## Working

```
/ticket <desc>      Create an issue any teammate can pick up
/fix <issue#>       Pick up an issue, branch, fix, PR
/parallel a | b | c Run independent tasks in parallel worktrees
```

### Build loop
```bash
just check          Fast compile check (no tests)
just test           Full test suite
just lint           Clippy — must be clean before you stop
```

## Reviewing

```
/review <pr#>       Security, correctness, style review
/merge <pr#>        Squash-merge, sync main, clean up
/pr [title]         Create a PR from current branch
```

## Capturing Knowledge

```
/feedback <notes>   Turn observations into GitHub issues
/checkpoint         End-of-session — what moved, what's left for the team
```

When you learn something that isn't in the code:
1. Find the right page in `kb/`
2. Update it
3. If no page fits, create one and link it from `kb/Home.md`

The kb is shared. Keep it current. Your teammates and their agents read it too.

## End of Day

```
/checkpoint         What moved? What's open? KB updated?
/wip                Save and push everything
```

## Quick Reference Card

| I want to... | Do this |
|---|---|
| Start my session | `/focus` |
| See what the team did | `/sync` |
| Fix a bug | `/fix 42` |
| Create a task anyone can grab | `/ticket add risk scoring agent` |
| Run 3 things at once | `/parallel task a \| task b \| task c` |
| Save and go | `/wip` |
| Review a teammate's PR | `/review 17` |
| Ship a reviewed PR | `/merge 17` |
| File feedback | `/feedback the desktop crashes when...` |
| Check project health | `/status` |
| Security scan | `/audit` |
| Quality trends | `/quality check` |
| End the day | `/checkpoint` |

See also: [[Workflow/Working with Claude]], [[Workflow/Working with Codex]], [[Workflow/Working with Gemini]]
