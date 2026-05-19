---
tags: [architecture, learning, jtbd, verifier]
source: codex
---

# Decoder Calibration

Decoder calibration is the v0.13 learning loop for Axiom. It is not the v0.12
starting point.

v0.12 should first make the verifier sharp with Tally escrow release:
`Satisfied`, `Blocked`, and `Invalid` must be concrete, traceable outcomes.
Only then should Axiom learn from those outcomes.

## Purpose

Axiom may learn as a decoder. The goal is to make future JTBD-to-Truth Package
compilation more complete and easier to verify:

- likely evidence requirements for recurring clause shapes;
- likely forbidden actions for recurring failure modes;
- scenario scaffolds that previously caught meaningful gaps;
- verifier expectations that produced clear reports;
- safer defaults based on accepted human overrides.

The learning loop must not make Axiom a runtime reasoner. It must not select
Formations, recompute authority, or host specialist execution.

## Input Signal

The calibration source is an audited `AxiomRunReport`, not a raw prompt:

- package ID and truth version;
- source JTBD clause IDs and fingerprints;
- verifier spec;
- promoted facts and evidence refs;
- lineage audit result;
- observed stop reason;
- verdict;
- explicit operator overrides or rejections.

The report needs a hard enough label to learn from. That is why Tally escrow
release precedes calibration: irreversible jobs make `Satisfied`, `Blocked`,
and `Invalid` harder to blur.

## Calibration Table

A calibration record should be keyed by structured shape, not raw prose alone:

```text
clause_kind
normalized_clause_shape
domain_hint
decoder_rule_id
fingerprint_class
```

Values may include:

```text
suggested_evidence_templates
suggested_failure_scenarios
suggested_policy_requirements
suggested_verifier_expectations
confidence
rationale
source_report_ids
```

## Query Point

Calibration is queried during JTBD decoding before the Truth Package is
finalized. Any generated artifact influenced by calibration must carry both:

1. the originating JTBD clause ID;
2. the calibration entry ID.

The JTBD clause remains the root of custody. Calibration can explain why Axiom
filled in a richer artifact, but it cannot replace the human job as the source.

## Review Rule

Learned priors are auditable inputs, not hidden behavior. Operators must be
able to review, accept, reject, or reset them. Rejected priors should become
negative calibration signals rather than disappearing silently.

## Dependency On v0.12

The first useful calibration feed should come from the Tally escrow release
proof:

- satisfied release: which evidence clauses were sufficient;
- blocked release: which unresolved authority/evidence gate stopped promotion;
- invalid release: which forbidden action made the run unacceptable.

That gives v0.13 a real feedback signal. Starting calibration before those
labels are sharp would teach the decoder from weak judgments.
