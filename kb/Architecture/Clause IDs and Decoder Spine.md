---
tags: [architecture, jtbd, truth-package]
source: codex
---

# Clause IDs And Decoder Spine

This is the v0.10 entry decision for turning a structured JTBD into the first
Truth Package spine. The goal is not full bundle generation yet. The goal is a
deterministic address space for human intent and a decoder path that can be
audited before any runtime behavior depends on it.

## Decision

Use deterministic, package-local hierarchical clause IDs with separate content
fingerprints.

Example:

```text
jtbd.vendor_commitment.actor
jtbd.vendor_commitment.functional_job
jtbd.vendor_commitment.so_that
jtbd.vendor_commitment.evidence.vendor_assessment
jtbd.vendor_commitment.failure.bypassed_approval
```

The ID is the reviewable address. The fingerprint proves the canonical text
that address referred to.

## Why Not UUIDs Or Indexes

Random UUIDs make regeneration noisy and hard to review. Array indexes make
identity depend on list order, so inserting a new evidence item would rewrite
lineage.

IDs must survive harmless reordering. Text changes should show up as changed
fingerprints. If a human wants identity to survive substantial wording changes,
they preserve the explicit clause key and accept the changed fingerprint.

## Clause ID Rules

- IDs are scoped to a Truth Package.
- IDs start with `jtbd.<job_key>`.
- Scalar fields use fixed paths: `actor`, `functional_job`, and `so_that`.
- Collection fields use stable keys: `evidence.<key>` and `failure.<key>`.
- Explicit keys win. Missing keys are derived from canonical text as slugs.
- List indexes are never valid identity.
- Duplicate explicit keys are errors.
- Duplicate implicit slugs get a short fingerprint suffix; exact duplicate
  clauses are errors.

## Fingerprint Rules

`ClauseFingerprint` is a SHA-256 hash of canonicalized clause text. The first
normalizer is deliberately small:

- trim leading and trailing whitespace;
- collapse internal whitespace to a single space;
- preserve case and punctuation.

This means formatting noise does not change the fingerprint, but meaningful
wording changes do.

## Decoder Strategy

v0.10 decoding is rule-based and deterministic:

```text
structured JTBD input
  -> canonical clauses
  -> deterministic IDs and fingerprints
  -> initial lineage map
  -> generated package spine
```

LLMs may later propose scenarios, policy hints, or evidence scaffolding, but
their output must pass through deterministic normalization before entering a
Truth Package. A generated artifact must be able to explain:

- source clause IDs;
- decoder rule ID;
- decoder version;
- input fingerprints.

## Deferred Decisions

Crate boundary: start in `axiom-truth` with separable `truth_package` types.
Extract `axiom-truth-package` only if downstream crates need the manifest
without the validation pipeline.

Overrides: use overlays, not mutation of generated `.truths`. The overlay
format can wait until the package spine exists.

Verifier spec: expected stop reasons should be a set. Forbidden actions compose
additively with Cedar policy requirements. Full verifier semantics can react to
the first generated package.

## First Proof

The first implementation should prove:

1. the same structured JTBD regenerates the same `JtbdDocument`;
2. reordering evidence or failure clauses does not change IDs;
3. every generated artifact references known clauses;
4. every clause is used, explicitly deferred, or explicitly rejected.
