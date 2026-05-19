---
tags: [marquee, jtbd, verifier, organism]
source: codex
---

# Round-Driven Formation Design

`atelier-showcase`'s `just show-round-driven` scenario is the primary v0.11
marquee proof for Axiom because it is not a straight-line run.

It has two governed run boundaries:

1. a dynamic design huddle Formation proposes, criticizes, scores, synthesizes,
   and evolves candidate rosters across rounds;
2. the selected work Formation is compiled, instantiated, and run to Converge.

That means the `AxiomRunReport` cannot flatten the proof into one final stop
reason. It needs an overall verdict and per-stage records for the design huddle
and the work Formation.

## JTBD

```text
Actor: formation host
Functional job: select and run a policy-and-anomaly audit Formation for a candidate plan
So that: dynamic Formation design is traceable, executable, and converges before the plan audit is trusted
Evidence required:
  - round signals and batch sentinels
  - prior-round exclusions derived from blocked drafts
  - mechanical and LLM critic verdicts with confidence
  - adversarial findings per draft
  - evidence-weighted scorecard and shortlist
  - compile handoff to real catalog and factory-covered descriptors
  - design huddle stop reason and integrity proof
  - work Formation stop reason and integrity proof
Failure modes:
  - rounds do not evolve after blocked rosters
  - shortlist is emitted before critic and scorer sentinels
  - blocked-only descriptors are reused without explanation
  - selected draft cannot be compiled or instantiated
  - LLM parse or chat failures are hidden
  - work Formation fails to converge
  - promoted facts lack package clause lineage
```

## Report Shape Learning

The fixture added in `tests/round_driven_marquee.rs` keeps:

- top-level `AxiomRunReport.verdict = Satisfied`;
- top-level `observed_stop_reason = Converged`;
- `run_stages[design_huddle]` with the huddle Formation ID, huddle stop
  reason, promoted design facts, trace links, and huddle integrity proof;
- `run_stages[work_formation]` with the selected work Formation ID, work stop
  reason, promoted audit facts, trace links, and work integrity proof.

This is the boundary-preserving shape Axiom needs for dynamic Formations:
Organism still owns formation selection and execution dynamics, Converge still
owns promotion and integrity, and Axiom verifies that each promoted fact can be
traced back to the job clause it served.

An irreversible commitment fixture should follow as the next strict-verdict
proof. It will sharpen `Satisfied` / `Blocked` / `Invalid` semantics, but it
does not replace this staged Formation proof: this fixture proves Axiom can
verify dynamic Organism behavior without becoming a formation selector.

## Live Run Shape

The live showcase blocker is resolved in `atelier-showcase` origin/main:
`c20e59b -> 7dac4b7 -> 037845b`.

The confirmed run shape is now:

- round 1 explores candidate rosters;
- round 2 reads prior verdicts and drops `disagreement-mapper`;
- evidence scoring ranks by `100 * mechanical_pass + llm_confidence - 10 *
  adversarial_findings`, producing `182` for the surviving shortlist and
  `i32::MIN` for blocked candidates;
- the LLM convergence judge returns `CONVERGED` at round 2;
- the halt marker fires, so round 3 is skipped with no drafts proposed;
- the selected work Formation runs on the converged shortlist.

v0.11 should adapt those real design and work `ConvergeResult`s into
`AxiomRunObservation`, then compute the report verdict from the observed stage
facts instead of maintaining a hand-authored report fixture.

## Axiom Fixture Alignment

The Axiom fixture now mirrors the platform-API markers from that run:

- proposer evolution uses
  `organism-catalog-proposer-exclusions-64657369676e2d726f756e642d32`, the
  platform `proposer_exclusions_marker("design-round-2")` diagnostic;
- scorecard evidence is represented by `evidence-scorecard-design-round-2`;
- convergence judgment is represented by `design-convergence:2`;
- early halt is represented by `design-convergence-reached`;
- skipped round 3 is recorded as a stage replay note, not a promoted fact,
  because the live scenario prints the skipped-round condition from the halt
  marker and absent drafts.

`PromotedFactRecord::from_context_fact(...)` is the first v0.11 adapter seam:
it converts Converge's public promoted `ContextFact` projection into Axiom's
report fact record while preserving evidence refs and local/remote trace links.
`AxiomRunObservation::from_stages(...)` aggregates staged report records without
collapsing the design huddle and work Formation boundaries.
