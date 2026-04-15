---
tags: [concepts]
source: llm
---

# Truth Documents

A `.truths` file is a Gherkin-extended specification format. Axiom parses `Truth:` as an alias for `Feature:`, and adds governance declaration blocks before the Gherkin body.

## Format

```gherkin
Intent:
  Outcome: What should happen
  Goal: Why it matters

Authority:
  Actor: Who can approve
  May: Allowed actions
  Must Not: Forbidden actions
  Requires Approval: What needs gates
  Expires: When authority lapses

Constraint:
  Budget: Spending boundaries
  Cost Limit: Maximum spend
  Must Not: Forbidden patterns

Evidence:
  Requires: What must be provided
  Provenance: Where it came from
  Audit: What must be logged

Exception:
  Escalates To: Who handles edge cases
  Requires: Prerequisites for exceptions

Truth: Enterprise vendor selection is auditable and approval-gated
  @invariant @structural
  Scenario: All vendor evaluations require documented criteria
    Given a vendor evaluation request
    When the evaluation is submitted
    Then it must include field "evaluation_criteria"
    And it must include field "risk_assessment"
```

## Governance Blocks

All governance blocks are optional. They are parsed by the `truths` module into `TruthGovernance` and flow through to:

- **WASM manifests** — embedded as metadata in compiled modules
- **Cedar policy analysis** — mapped to policy requirements by `policy_lens`
- **Simulation** — checked for completeness by the simulation module

## Scenario Tags

| Tag | Purpose |
|---|---|
| `@invariant` | Marks as invariant check |
| `@structural` / `@semantic` / `@acceptance` | Invariant classification |
| `@id:name` | Named identifier for cross-referencing |
| `@llm` | Requires LLM provider at runtime |
| `@test` | Test-only, not deployed |

## File Extensions

- `.truths` — canonical format
- `.truth` — accepted alias
- `.feature` — Gherkin compatibility
