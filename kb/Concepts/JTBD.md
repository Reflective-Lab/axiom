---
tags: [concepts]
source: llm
---

# Jobs-to-be-Done (JTBD)

The release direction is: JTBD is the source input for a Truth Package, not
only metadata embedded inside a `.truths` file. The existing `jtbd` module still
extracts legacy comment metadata. The `truth_package` module owns the
source-first path with `JtbdInput`, deterministic `ClauseId`s, text
fingerprints, lineage closure, verifier specs, run reports, and calibration
feedstock.

The `jtbd` module extracts Jobs-to-be-Done metadata from comments in truth
files, connecting specifications to user outcomes. That legacy comment format
is now a migration source, not the destination format for new package work.
`JtbdInput::from_metadata(...)` lifts parsed `JTBDMetadata` into structured
`JtbdInput`; from there the package spine owns clause IDs, fingerprints,
lineage, generated `.truths`, and verifier expectations.

See [[Architecture/Clause IDs and Decoder Spine]] for the clause identity
decision.

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

For new package work, use the source-first spine:

```text
JtbdInput
  -> JtbdDocument
  -> ClauseId + ClauseFingerprint
  -> LineageMap
  -> Truth Package artifacts
  -> AxiomRunReport + calibration records after execution
```
