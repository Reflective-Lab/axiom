---
tags: [architecture, doctrine, intent, verifier]
source: llm
---

# Axiom as Verifier

Axiom is the typed translator from a human Job-to-be-Done into a governed
runtime contract. It is the layer where human intent enters the Reflective
stack in a form that can be audited, compiled, simulated, and later judged
against a real run.

This page is the doctrine anchor for E7: Axiom translates human jobs into
governed runtime contracts.

## Doctrine

Axiom is not "spec validation" as an end in itself. Validation is one of the
checks Axiom performs while compiling intent.

Axiom is also not an orchestrator, formation selector, authority engine, or
specialist host. Those responsibilities belong to other layers.

The central claim is:

```text
Human JTBD
  -> Axiom decodes and compiles intent
  -> Truth Package + IntentPacket + verifier spec + proof obligations
  -> Organism admits, reasons, selects, and runs formations
  -> Converge promotes facts under current authority and stop contracts
  -> ExperienceStore records outcomes and lineage
  -> Axiom tightens future decoding priors
```

Axiom succeeds when the downstream system can answer: which human job did this
runtime fact serve, which evidence requirement justified it, which failure mode
was being guarded, which truth version was active, and which run verdict proved
or rejected satisfaction.

## Source Of Intent

The primary human input should be a compact JTBD, not a hand-authored `.truths`
file.

Minimum shape:

| Field | Purpose |
|---|---|
| `actor` | The human, role, or organization trying to make progress. |
| `functional_job` | The concrete job the actor wants done. |
| `so_that` | The downstream value or risk reduction the job serves. |
| `evidence_required` | What must be observed before the system may treat the job as satisfied. |
| `failure_modes` | What must not happen, or what must be detected and blocked. |

The `.truths` file becomes an auditable projection of the job. Humans may
inspect, override, and version that projection, but it is no longer the only
place where intent is expressed.

## Truth Package

A Truth Package is the immutable bundle produced by decoding a JTBD. It should
contain, at minimum:

| Artifact | Role |
|---|---|
| Source JTBD | Root human job with stable clause IDs. |
| Generated `.truths` | Auditable intermediate projection for validation and versioning. |
| Gherkin scenarios | Executable behavioral examples and invariant surfaces. |
| Predicates | Semantic claims extracted from scenarios. |
| Policy requirements | Cedar requirements or draft policy coverage expectations. |
| Invariant artifacts | Compiled invariant expectations and any generated WASM-facing artifacts. |
| Simulation cases | Deterministic cases used to test convergence readiness. |
| Replay profile | Inputs required to reproduce verifier-relevant behavior. |
| IntentPacket | Runtime contract consumed by Organism. |
| Proof obligations | Required evidence, forbidden actions, provenance duties, and expected stop reasons. |
| Verifier spec | The expected conditions an `AxiomRunReport` will judge after a run. |
| Lineage map | Clause-to-artifact and artifact-to-clause chain of custody. |

Every generated artifact must point back to the JTBD clause it serves. Every
JTBD clause must be used, explicitly deferred, or explicitly rejected. Orphan
artifacts and unused clauses are verifier failures, not cosmetic lint.

The v0.10 schema reference lives in [[../Concepts/Truth Package]].

## AxiomRunReport

`AxiomRunReport` is a verifier, not a dashboard summary.

It compares the Truth Package's verifier spec against an Organism/Converge run
and emits a verdict:

| Verdict | Meaning |
|---|---|
| `Satisfied` | Required evidence appeared, forbidden actions did not occur, authority checks passed at promotion time, and the stop reason matches the verifier spec. |
| `Blocked` | The run correctly stopped before satisfaction because required approval, evidence, policy, or human intervention was missing. |
| `Exhausted` | The run consumed a declared budget, retry, time, token, or search limit without satisfying the job. |
| `Invalid` | The run violated the Truth Package: missing lineage, unexpected stop reason, invariant violation, forbidden action, stale evidence, or unverifiable promotion. |

The report should preserve enough structure for Helms or a marquee app to show
why the run was judged that way without reinterpreting raw traces.

## Chain Of Custody

The differentiation is not that Axiom can generate specs from prose. That is
generic. The differentiation is the guarantee that intent remains typed and
traceable.

Three custody rules are mandatory:

1. Every decoded artifact carries the package ID, truth version, and source
   JTBD clause IDs it serves.
2. Decoding is constrained and reproducible enough to review. A generated
   package must explain which decoder rule, calibration entry, or explicit
   human override produced each artifact.
3. The root job survives runtime. Any fact promoted from a Truth Package run
   must be traceable back to the job, evidence requirement, failure mode, and
   truth version it served.

This is where Axiom should adopt an `AXIOM_PROVENANCE` source compatible with
Mosaic's `ProvenanceSource` contract. Axiom provenance describes where a
proposed fact or obligation came from. It does not promote the fact and does
not grant authority.

## Authority Boundary

Axiom declares requirements. Converge enforces the world's current answer.

A Truth Package may say:

- this job requires spend authority;
- this evidence must exist before promotion;
- this actor must not perform a forbidden action;
- this run should halt, escalate, or be invalid under these stop reasons.

Those are requirements, not delegated authority. Converge still recomputes
authority at promotion time against current policy, current facts, and current
evidence. If policy changes after the Truth Package was created, Converge's
promotion-time answer wins.

## Formation Boundary

Axiom can make plausible runtime needs explicit. It can say the job likely
needs vendor assessment, policy review, risk analysis, or evidence retrieval.
It can compile those needs into an `IntentPacket` and proof obligations.

Axiom must not select the Formation.

Organism owns admission, problem classification, Formation selection, planning,
competition, multi-step dynamics, and learning from run outcomes. Axiom may
feed Organism a verifier result after the run, but it must not become a
formation optimizer.

## Specialist And Sandbox Boundary

Axiom does not host specialists.

Mosaic owns concrete specialist capabilities, adapters, classifiers, solvers,
and fact-emitting suggestors. Helm owns executable WASM plugin hosting,
sandbox policy, signing, quotas, and lifecycle. Converge owns the governed
promotion boundary and decides whether plugin output can become fact.

Axiom may generate or reference invariant artifacts, manifests, hashes, and
lineage, but it does not execute them as a specialist runtime.

This sandbox boundary is mirrored in the neighboring KBs:

- `../helms/kb/Architecture/Foundation Contracts.md` names
  `helm-plugin-runtime` as the application plugin execution surface.
- `../converge/kb/Architecture/Golden Path Matrix.md` says Converge consumes
  adapted proposals, invariant verdicts, evidence refs, and trace links; it
  does not load Axiom plugins or embed Wasmtime/Cranelift.
- `../converge/kb/Architecture/Lean Packaging and Embedding.md` records the
  2026-05-18 move of sandboxed WASM execution out of Converge and into Helm.

The rule is simple: Axiom compiles the contract; other layers provide the
capabilities that try to satisfy it.

## Learning Boundary

Axiom may learn as a decoder.

It may persist calibration such as:

- clause shape -> likely evidence requirements;
- failure mode -> useful scenario scaffolding;
- domain phrase -> policy obligation template;
- prior override -> safer default projection;
- verifier result -> future decoder confidence.

Axiom must not learn which Formation should win. That loop belongs to
Organism. Axiom's learning should make future Truth Packages more complete,
better traced, and easier to verify, not more operationally clever.

This is a v0.13 milestone, not the v0.12 starting point. v0.12 should first
make verdict labels sharp with Tally escrow release. Once Axiom can distinguish
`Satisfied`, `Blocked`, and `Invalid` on an irreversible job with clause-level
lineage, those reports become useful calibration input. See
[[Architecture/Decoder Calibration]].

## Falsifiable Signals

This doctrine is real only if it produces testable signals:

1. A structured JTBD can deterministically regenerate the same Truth Package.
2. The lineage map closes with no orphan artifacts and no unused clauses.
3. Axiom facts or obligations carry `AXIOM_PROVENANCE` through the evidence
   chain.
4. `AxiomRunReport` judges an actual run against the verifier spec.
5. One consequential marquee job flows end to end with lineage preserved.
6. Decoder calibration improves package completeness for repeated clause
   shapes without selecting formations.
7. The boundaries stay explicit: no formation selection, no authority
   recompute, no specialist hosting.

## Implementation Posture

Build this forward in the smallest honest order:

1. Define the JTBD clause model and stable clause IDs.
2. Define the Truth Package manifest and lineage map.
3. Treat generated `.truths` as a projection with explicit overrides.
4. Split intent from satisfaction conditions in the compiled runtime contract.
5. Add verifier expectations before expanding generation breadth.
6. Wire `AxiomRunReport` to Organism and Converge run traces.
7. Feed verifier outcomes into decoder calibration, not formation strategy.
