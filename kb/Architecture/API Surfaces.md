---
tags: [architecture, api]
source: llm
---

# API Surfaces

This page is the authoritative API reference for Axiom's public boundaries.
When it conflicts with older root-level docs, this page wins.

## Release Surface v0.15

The v0.15 release surface is the Axiom layer, not the whole app/Helm stack:

| Surface | Public API |
|---|---|
| JTBD decoding | `JtbdInput`, `ClauseInput`, `decode_jtbd` |
| Truth package | `TruthPackage`, `TruthPackageId`, `TruthPackageArtifacts`, `VerifierSpec`, `ProofObligation`, `ReplayProfile`, `LineageMap` |
| Intent compilation | `compile_intent`, `compile_intent_from_source` |
| Run verification | `AxiomRunObservation`, `AxiomRunStageRecord`, `AxiomRunReport::verify`, `AxiomRunVerdict`, `ObservedStopReason`, `PromotedFactRecord`, `RunIntegrityProof` |
| Adapter audit | `ObservationAdapterReceipt`, `ObservationAdapterReceiptInput`, `ObservationAdapterStatus` |
| Decoder learning | `LearningEpisode`, `CalibrationRecord`, `CalibrationTable`, `CalibrationSignalKind`, `CalibrationStatus`, `apply_decoder_calibration` |
| Provenance | `AXIOM_PROVENANCE`, `TruthPackageSeedPayload`, `truth_package_seed_fact`, `truth_package_seed_facts` |

Axiom owns these schemas and verifier semantics. Helm owns operator displays,
package review, calibration review, ledger policy, and plugin hosting. Apps own
raw transcripts and app-specific adapters. Organism owns Formation selection.
Converge owns promotion authority and integrity.

## Runtime intent

Axiom owns Truth-shaped input. Organism owns runtime mechanism. The public flow
is:

```rust
use axiom_truth::{compile_intent, parse_truth_document};

let truth = parse_truth_document(source)?;
let intent = compile_intent(&truth)?;
let receipt = runtime.admit_intent(&intent, actor, src, &mut ctx)?;
```

Pipeline:

```text
.truths source
  -> truths::parse_truth_document
  -> TruthDocument
  -> intent::compile_intent
  -> organism_pack::IntentPacket
  -> organism_runtime::Runtime::admit_intent
```

`compile_intent_from_source(&str)` is the convenience API for callers that do
not need to inspect the intermediate `TruthDocument`.

Axiom does not re-export `IntentPacket`, `ExpiryAction`, `ForbiddenAction`, or
`Reversibility`. Those types are defined by `organism-pack`; callers that need
to name them should depend on `organism-pack` directly. The ownership story is:
Axiom produces the packet, organism-pack defines it, organism-runtime admits it.

## Truth parsing

| API | Purpose |
|---|---|
| `parse_truth_document(&str)` | Parse `.truths` source into `TruthDocument`. |
| `TruthDocument` | Parsed Gherkin body plus governance blocks. |
| `TruthGovernance` | Parsed `Intent`, `Authority`, `Constraint`, `Evidence`, and `Exception` blocks. |

## Validation and feedback

| Module | Public surface |
|---|---|
| `gherkin` | `GherkinValidator`, `ValidationConfig`, `SpecValidation`, validation issue types. |
| `guidance` | `suggest_guidance`, `GuidanceRequest`, `GuidanceResponse`. |
| `simulation` | `simulate`, `simulate_spec`, `SimulationConfig`, `SimulationReport`, optional `DomainProfile`s. |
| `policy_lens` | `check_coverage`, `PolicyCoverageReport`, Cedar coverage vocabulary. |

## Compilation

| Module | Public surface |
|---|---|
| `predicate` | Gherkin step to semantic predicate extraction. |
| `codegen` | Predicate to Rust invariant source generation. |
| `compile` | Rust invariant source to WASM module compilation. |
| `intent` | Truth governance to `organism_pack::IntentPacket` compilation. |
| `truth_package` | Structured JTBD source normalization, stable clause IDs, fingerprints, and lineage closure checks. |

The WASM path and the runtime intent path are separate outputs from the same
Truth source. WASM invariants are generated artifacts for Helm's sandbox and
Converge-facing invariant contracts. Intent packets are runtime admission
inputs for organism. Axiom does not host or execute the WASM artifacts.

The Truth Package spine starts from `JtbdInput` and `decode_jtbd` produces a
deterministic `TruthPackage`. Clause identity is held in `ClauseId`; content
custody is held in `ClauseFingerprint`; artifact custody is checked by
`LineageMap`. Human edits to generated `.truths` use
`TruthProjectionOverlay` and return a separate `TruthProjectionVersion` rather
than mutating the package.

## Run Verification

Axiom verifies normalized run observations over the existing stack. It is not a
new execution model.

```text
TruthPackage
  -> Organism / Mosaic / Converge / app execution
  -> app-specific or runtime adapter
  -> AxiomRunObservation
  -> AxiomRunReport::verify(&package, observation)
```

The report compares the package's `VerifierSpec` to the observed stop reason,
promoted facts, evidence refs, trace links, promotion-authority records, and
integrity proof. The output verdict is one of:

| Verdict | Meaning |
|---|---|
| `Satisfied` | Required evidence appeared, forbidden actions did not occur, and the observed stop reason matched the verifier spec. |
| `Blocked` | The run stopped honestly before satisfaction because approval, evidence, policy, or human intervention is still missing. |
| `Exhausted` | The run consumed a declared budget without satisfying the job. |
| `Invalid` | The run violated the package or could not be verified. |

Adapters must be deterministic for the same source transcript, package ID, and
truth version. A successful adapter returns an `AxiomRunObservation` plus an
`ObservationAdapterReceipt`; a rejected adapter returns only the receipt with
explicit errors.

## External crates

| Crate | Role |
|---|---|
| `converge-provider` | Chat backend traits, request/response types, and provider selection vocabulary used by validation and guidance. |
| `converge-manifold-adapters` | Concrete backend selection helper (`manifold::select_healthy_chat_backend`). |
| `organism-pack` | Defines `IntentPacket` and related runtime contract types consumed by organism. |
| `converge-pack` | Public provenance and fact payload contracts used for Axiom seed proposals and report extraction. |
| `organism-runtime` | Dev/test fixture dependency for proof recipes; Axiom's library release surface does not require callers to use it. |

No Axiom public API depends on Converge internal crates such as
`converge-core`, `converge-runtime`, or `converge-analytics`.
