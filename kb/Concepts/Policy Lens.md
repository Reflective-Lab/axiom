---
tags: [concepts]
source: llm
---

# Policy Lens

The `policy_lens` module maps truth governance to Cedar policy concepts, identifying gaps between what specs require and what policies enforce.

## How It Works

1. Extract `PolicyRequirements` from governance blocks — gated actions, authority level, spending limits, escalation targets
2. Parse Cedar policy text into `PolicyRule` items
3. Cross-reference: which actions are covered by policy, which are not

## Output

`PolicyCoverageReport` contains:
- `requirements` — what the spec demands
- `rules` — what Cedar policies provide
- `covered_actions` — actions with matching policy rules
- `uncovered_actions` — gaps that need policy authoring
- `observations` — advisory notes

## Integration

Used by the `cz validate --enforce` flag to check Cedar policy coverage during validation. Uncovered actions are reported as findings.
