---
tags: [architecture, runtime, formation]
source: llm
---

# Truth-to-Formation Run Proof

Axiom v0.9 should prove the stack path from a human-authored `.truths` file to
a real Converge fixed point. It should not become a second runtime. It should
make the existing runtime legible.

## Target Flow

```text
.truths source
  -> Axiom parses TruthDocument
  -> Axiom validates, simulates, and checks policy coverage
  -> Axiom compiles organism_pack::IntentPacket
  -> Organism Runtime admits the intent
  -> Organism selects a Formation template from available capabilities
  -> Organism compiles and instantiates concrete Suggestors
  -> Converge Engine runs the Formation
  -> Axiom returns an auditable run proof
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

## Minimum Proof

The first v0.9 proof should be deliberately small:

1. Start with one governed `.truths` fixture.
2. Validate and simulate it in Axiom.
3. Compile it into `IntentPacket`.
4. Ask Organism for a standard formation selection from declared capabilities.
5. Compile and run the formation with fixture `Suggestor` factories.
6. Assert the returned `ConvergeResult` has a successful fixed-point stop reason.
7. Return an `AxiomRunReport`-shaped record containing the evidence.

Fixture suggestors are acceptable for the first proof because the architectural
question is whether the stack path is honest. The next pass should replace one
fixture role at a time with Mosaic-backed capabilities: Manifold LLM selection,
Prism analytics suggestors, policy gates, and knowledge/search providers.

## AxiomRunReport Shape

The report should be stable enough for Helms and other apps to render:

| Field | Source |
|---|---|
| validation | `SpecValidation` |
| simulation | `SimulationReport` |
| intent | `IntentPacket` |
| selection_trace | `organism_runtime::SelectionTrace` |
| formation_plan | `organism_runtime::CompiledFormationPlan` |
| provider_assignments | Organism compiler/provider descriptors |
| stop_reason | `converge_kernel::StopReason` |
| promoted_facts | `ConvergeResult.context` |
| integrity | `ConvergeResult.integrity` |
| replay_notes | Axiom simulation + Converge replay metadata |

## Today Plan

For 2026-05-17:

1. Publish `axiom-truth` 0.8.1, since package verification is already green.
2. Land this v0.9 architecture story in README and KB.
3. Add the first fixture-based proof test: Truth -> IntentPacket -> Organism formation -> Converge fixed point.
4. Keep real Mosaic-backed providers out of the first test; add them after the fixed-point path is proven.
5. Stop before broad feature work if the proof reveals a missing public surface in Organism or Converge.
