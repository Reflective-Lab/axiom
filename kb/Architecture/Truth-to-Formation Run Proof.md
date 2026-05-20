---
tags: [architecture, runtime, formation]
source: llm
---

# Truth-to-Formation Run Proof

This page records the v0.9 proof target. That target has been absorbed into the
v0.15 Axiom layer release: Axiom now exposes app-neutral Truth Package,
observation, report, receipt, lineage, and calibration surfaces. It still must
not become a second runtime.

## Release Flow

```text
JtbdInput
  -> Axiom decodes TruthPackage
  -> Axiom validates, simulates, checks policy coverage, and compiles intent/invariants
  -> Organism admits the IntentPacket and selects/runs Formations
  -> Mosaic supplies concrete capabilities and suggestors
  -> Converge promotes facts or stops honestly
  -> app/runtime adapter emits AxiomRunObservation
  -> AxiomRunReport::verify returns the auditable run proof
```

The fixed point is Converge's stop contract: `StopReason::Converged` or a
domain-appropriate `StopReason::CriteriaMet`. Budget exhaustion, invariant
failure, HITL pause, and promotion rejection are also useful outcomes, but they
are not success for the first proof.

## Ownership

| Layer | Owns | Must not own |
|---|---|---|
| Axiom | Truth parsing, validation, simulation, intent compilation, run proof report | formation selection, provider credentials, fact promotion |
| Organism | admission, problem classification, Formation selection, compiled formation plans, executable suggestor catalogs | raw `.truths` parsing, Converge internals |
| Converge | `Suggestor`, `Engine`, `ContextState`, promotion, stop reasons, integrity proof | business intent parsing, formation strategy |
| Mosaic | concrete adapters, provider selection, LLM/search/fetch/storage, analytics and tool capabilities | truth semantics, promotion authority |

## Best-of-Stack Rule

Axiom shines when it forces every layer to do its own job:

- Do not hand-roll a formation runner inside Axiom.
- Do not bypass Organism's `Runtime::select_formation`, formation compiler, or executable catalog boundary.
- Do not import `converge-core` directly; use public Organism and Converge surfaces.
- Every runtime participant must enter through Converge's `Suggestor` contract.
- Provider choice should come from `converge-provider` vocabulary and Mosaic/Manifold adapters, not hardcoded model names.
- The run proof must include the Converge stop reason and integrity proof, not just an application-level success flag.

## Historical Minimum Proof

The first v0.9 proof was deliberately small:

1. Start with one governed `.truths` fixture.
2. Validate and simulate it in Axiom.
3. Compile it into `IntentPacket`.
4. Ask Organism for a standard formation selection from declared capabilities.
5. Compile and run the formation with fixture `Suggestor` factories.
6. Assert the returned `ConvergeResult` has a successful fixed-point stop reason.
7. Return an `AxiomRunReport`-shaped record containing the evidence.

Fixture suggestors were acceptable for the first proof because the
architectural question was whether the stack path is honest. Later v0.11-v0.15
work generalized the report/observation boundary and repeated it across
marquee adapters while preserving ownership boundaries.

## AxiomRunReport Shape

The release report is stable enough for Helms and apps to render:

| Field | Source |
|---|---|
| package identity | `TruthPackage` |
| verifier spec | `TruthPackage::verifier_spec` |
| observed stop reason | `AxiomRunObservation` |
| promoted facts | `PromotedFactRecord` |
| evidence refs and trace links | adapter-normalized public runtime evidence |
| promotion authority | Converge-observed authority records |
| integrity | `RunIntegrityProof` |
| run stages | optional `AxiomRunStageRecord` entries |
| replay notes | adapter/runtime replay metadata |
| verdict | `AxiomRunReport::verify` |
