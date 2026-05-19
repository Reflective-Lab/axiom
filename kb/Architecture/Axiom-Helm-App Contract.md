---
tags: [architecture, helm, apps, contract]
source: codex
---

# Axiom-Helm-App Contract

This page defines how Axiom, Helm, and applications should play together when
apps become thinner Helm-operated experiences rather than bespoke workflow
islands.

The goal is not to make Axiom run the app. The goal is to make the app's
important jobs expressible as Truth Packages that Helm can show, operate,
review, and replay while Organism, Mosaic, and Converge keep their runtime
responsibilities.

## Product Posture

The long-term product shape is:

```text
App domain event or operator intent
  -> Helm captures or selects a JTBD
  -> Axiom decodes a Truth Package
  -> Helm presents the package, generated .truths, obligations, and priors
  -> Organism selects and runs Formations
  -> Mosaic supplies concrete capabilities and suggestors
  -> Converge reaches a fixed point or an honest stop reason
  -> App adapter emits an AxiomRunObservation
  -> Axiom verifies the run and emits an AxiomRunReport
  -> Helm shows the verdict and reviews calibration reinforcements/concerns
```

Apps should keep domain ownership. Helm should absorb operator/control-plane
repetition. Axiom should absorb truth, lineage, verifier, and decoder-learning
repetition.

## Ownership

| Layer | Owns | Produces | Must not own |
|---|---|---|---|
| Axiom | JTBD decoding, Truth Package schema, generated `.truths`, verifier spec, proof obligations, lineage, reports, and decoder calibration priors. | `TruthPackage`, `IntentPacket`, WASM artifacts and manifests, `AxiomRunReport`, `LearningEpisode`, `CalibrationRecord`, `CalibrationTable`, `CalibrationSuggestion` and `CalibrationConcern` artifacts. | Operator UX, app domain state, formation selection, authority recompute, plugin hosting, specialist execution, raw run history. |
| Helm | Operator surface, package review, truth projection review, calibration review, plugin sandbox lifecycle, run-history display, and app-facing orchestration screens. | Reviewed package versions, accepted/rejected/reset calibration decisions, sandbox execution events, operator audit trails. | Truth semantics, source JTBD mutation, Converge promotion authority, Organism formation strategy, app business state. |
| App | Domain model, domain state transitions, user workflows, domain adapters, custody/payment/external integrations, and app-specific transcript shape. | Domain events, app transcripts, evidence handles, adapter output as `AxiomRunObservation`. | Generic truth review, generic calibration review, generic verifier logic, generic operator-control flows. |
| Organism | Admission, problem classification, Formation selection, planning, runtime dynamics, and learning from Formation outcomes. | Formation plans, stage records, runtime traces, selected work formations. | Truth parsing, authority recompute, app domain persistence. |
| Mosaic | Concrete providers, retrieval, analytics, algorithms, adapters, and fact-emitting suggestors. | Capability-backed suggestors, provider outputs, evidence candidates. | Truth semantics, Helm operator policy, Converge promotion decisions. |
| Converge | Fixed-point loop, fact promotion, authority recompute, stop reasons, evidence refs, trace links, and integrity proof. | Promoted facts, stop reason, integrity proof, promotion authority evidence. | App plugin hosting, Axiom decoding, Helm operator UX. |

## Three Loops

### Authoring Loop

```text
operator/app intent
  -> JTBD
  -> Axiom decode_jtbd(...)
  -> Truth Package
  -> Helm review of package, .truths projection, obligations, and calibration suggestions
  -> frozen package version for runtime
```

Axiom owns deterministic package construction. Helm owns the review surface.
The app should not hand-roll a second package review model.

### Runtime Loop

```text
frozen package version
  -> IntentPacket admitted by Organism
  -> Formation run with Mosaic-backed suggestors
  -> Converge promotes facts or stops honestly
  -> app/domain adapter maps run output to AxiomRunObservation
  -> AxiomRunReport verifies satisfaction, blockage, exhaustion, or invalidity
```

Axiom verifies after the run. It does not select the Formation or decide
promotion authority.

### Learning Loop

```text
AxiomRunReport + lineage audit
  -> LearningEpisode
  -> CalibrationRecord(s)
  -> Helm operator review
  -> accepted CalibrationTable
  -> future Truth Packages enriched by reviewed reinforcements and concerns
```

Axiom persists distilled decoder priors. Raw transcripts and operational
history remain downstream in Helm, the app, or ExperienceStore.

v0.15 splits accepted calibration into two operator-visible signals:

- `Reinforcement` records become `CalibrationSuggestion` artifacts: the
  decoder should keep reaching for evidence, failure, policy, or verifier
  templates that have worked before.
- `Concern` records become `CalibrationConcern` artifacts: the decoder should
  surface prompts, warnings, default evidence expectations, or alternate
  scaffolding for clause shapes that repeatedly go uncovered.

Both signals are decoder-only. Neither signal weakens the source JTBD,
changes verifier requirements, selects a Formation, or grants promotion
authority.

## Tally As Boundary-Finding Loop

Tally should stay the next proving ground because escrow release is crisp:
once funds leave escrow, the system cannot treat a bad outcome as harmless.

Use each Tally iteration to classify repeated code:

| Repeated shape | Destination |
|---|---|
| Evidence clauses, failure modes, verifier expectations, lineage, and verdict computation | Axiom |
| Package display, operator acceptance/rejection/reset, calibration reinforcement/concern review, run verdict display | Helm |
| Escrow agreement state, release transition, custody receipt, payment rail, app transcript | Tally |
| Signing, capability-backed evidence retrieval, analytics, policy suggestors | Organism/Mosaic/Converge through their public contracts |

Tally's current adapter already maps a release transcript into
`AxiomRunObservation`. The next useful iterations should ask:

1. Which Tally release requirements can be expressed as JTBD clauses and
   generated verifier obligations?
2. Which hand-built app controls are really Helm operator controls?
3. Which evidence-mapping code is app-specific adapter logic versus generic
   Axiom report plumbing?
4. Which runtime choices belong to Organism/Mosaic instead of Tally or Axiom?
5. Which accepted calibration priors make future escrow packages more complete
   without weakening the source JTBD?
6. Which uncovered evidence concerns should Helm show before an operator
   attempts another release?

### Boundary Probe: Release Conditions Met

First pressure-tested control:
`release-requires-conditions-met`.

This control currently appears in Tally as:

- a `TruthSpec` constant in `tally-truths`;
- a `TransitionReason::ConditionsMet { satisfied }` domain payload;
- a `tally-kernel::Kernel::apply_transition` guard that rejects release when
  agreement conditions are not fully satisfied;
- a release transcript truth key adapted into Axiom's escrow-release
  observation;
- an Axiom evidence requirement and verifier expectation in the escrow-release
  Truth Package.

The contract split is:

| Layer | Boundary decision |
|---|---|
| Tally app/kernel | Owns the agreement state machine, condition indices, `ConditionsMet` payload, and hard rejection of `Released` when conditions are not satisfied. This is domain law and must stay close to the agreement aggregate. |
| Axiom | Owns the truth expression: release requires satisfied conditions before the job can be judged `Satisfied`; missing condition evidence yields `Invalid` or `Blocked`, can produce uncovered-clause `Concern` records, and must never weaken the source JTBD. |
| Helm | Should show the operator which release-condition evidence is present, missing, disputed, or accepted as a calibration concern. Helm owns the review/decision surface, not the transition law. |
| Organism/Mosaic | May assemble readiness Formations and suggestors to gather condition evidence, counterexamples, policy proofs, anomaly scores, or custody facts before the release attempt. They do not authorize release by themselves. |
| Converge | Promotes readiness facts, evidence refs, stop reasons, and integrity. It remains the authority boundary for promoted facts and policy gates. |

Immediate implication: do not move the release transition guard out of Tally's
kernel. Move repeated *explanation* and *review* work outward:

- Axiom should continue turning this control into verifier obligations,
  lineage, `AxiomRunReport` verdicts, and calibration concerns.
- Helm should eventually render the release-readiness packet: required
  condition evidence, missing evidence, accepted concerns, and the last
  verifier verdict.
- Tally should emit a clean release observation/transcript and keep the
  transition guard strict.

This is the template for future probes: if the logic mutates domain state, it
stays in the app/kernel; if it explains whether a job contract was satisfied,
it moves toward Axiom; if it lets an operator inspect or review the contract,
it moves toward Helm.

## Expression Boundary

Axiom should be able to express:

- the actor and job;
- evidence required before satisfaction;
- failure modes and forbidden observations;
- time budget and expected stop reasons;
- policy requirements as obligations;
- package artifacts and lineage;
- post-run verifier expectations;
- reviewed decoder reinforcements and concerns.

Axiom should not express:

- the app's domain state machine implementation;
- payment, custody, or external API execution;
- who signs or hosts plugins at runtime;
- which Formation wins;
- which Mosaic suggestor implementation is best;
- whether a fact is promoted under current authority.

If an app repeats verifier logic, move it toward Axiom. If an app repeats
operator review or truth package display, move it toward Helm. If Axiom starts
encoding runtime strategy, move it back toward Organism, Mosaic, or Converge.

## Atlas Next

After the Tally boundary loop is clearer, the next app vertical should be
`/Users/kpernyer/dev/reflective/marquee-apps/atlas-integration`.

Atlas should not start by copying Tally's app-specific scaffolding. It should
start by consuming the clarified contract:

1. identify one customer-relevant Atlas JTBD;
2. decode it into a Truth Package;
3. decide what Helm should render and operate;
4. define the Atlas adapter from domain outcome to `AxiomRunObservation`;
5. verify the run with Axiom;
6. feed the report into calibration only after the verdict labels are sharp.

The point of doing Tally first is to make Atlas thinner on purpose, not by
wishful abstraction.

## Falsifiable Signals

The contract is working when:

1. A new app can reuse Helm package review instead of inventing a review UI.
2. A new app can emit `AxiomRunObservation` rather than implement its own
   satisfaction verdict.
3. Axiom can verify app outcomes without importing app crates.
4. Accepted calibration priors improve future packages without changing the
   source JTBD, generated `.truths`, or runtime `IntentPacket`.
5. Accepted concerns produce operator-visible warnings or scaffolding without
   removing required evidence from the verifier spec.
6. Raw app transcripts stay outside Axiom while Axiom keeps durable backlinks
   to distilled learning episodes.
7. Formation selection, authority recompute, and specialist hosting remain out
   of Axiom.
