---
tags: [workflow]
source: llm
---

# Daily Journey

## Morning

1. `cz doctor` — verify environment health
2. `cz digest` — review open findings and pending acks

## Development Cycle

1. Write or edit `.truths` files
2. `cz validate specs/` — check your work
3. Fix issues flagged by validation
4. `cz validate --enforce specs/` — verify Cedar policy coverage
5. `just test` — run tests
6. `just lint` — check formatting and clippy

## Before Committing

1. `just lint` — must pass
2. `cz validate specs/` — all specs valid
3. Review findings: `cz digest`
4. Acknowledge resolved findings: `cz ack <id>`
