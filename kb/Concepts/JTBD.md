---
tags: [concepts]
source: llm
---

# Jobs-to-be-Done (JTBD)

The `jtbd` module extracts Jobs-to-be-Done metadata from comments in truth files, connecting specifications to user outcomes.

## Format

YAML (recommended):

```gherkin
# JTBD:
#   actor: Founder
#   job_functional: "Invoice customers and collect payment"
#   job_emotional: "Feel confident that every invoice gets sent"
#   job_relational: "Be seen as professional and reliable"
#   so_that: "Cash flows predictably"
```

Plain text:

```gherkin
# JTBD
# As: Founder
# Functional: Invoice customers and collect payment
# So that: Cash flows predictably
```

## Metadata

`JTBDMetadata` captures:
- **actor** — who has the job
- **job_functional** — what they need to accomplish
- **job_emotional** — how they want to feel (optional)
- **job_relational** — how they want to be perceived (optional)
- **so_that** — the outcome
- **success_metrics** — how success is measured
- **failure_modes** — what can go wrong
- **evidence_required** — what proof is needed
- **audit_requirements** — what must be logged

## Purpose

JTBD metadata connects technical invariants back to human outcomes. It flows into manifests and can inform prioritization and impact analysis.
