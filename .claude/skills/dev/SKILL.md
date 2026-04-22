---
name: dev
model: sonnet
description: Start local development environment.
user-invocable: true
argument-hint: [example <name>]
allowed-tools: Bash, Read
---
# Dev
Run the Axiom CLI or kick off a dev workflow.
## Recipes
- `just doctor` — check environment health
- `just validate` — run cz validate
- `just help-cz` — show CLI help
## Rules
- Check required tools are installed (rust, just).
- Report missing dependencies clearly.
