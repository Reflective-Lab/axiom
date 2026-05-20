---
tags: [index]
source: llm
---

# Axiom — Entity Index

## Modules

| Entity | Description | Location |
|---|---|---|
| gherkin | LLM-powered validation of `.truths` specs | `src/gherkin.rs` |
| truths | Governance block parsing (Intent, Authority, Constraint, Evidence, Exception) | `src/truths.rs` |
| intent | TruthDocument to organism `IntentPacket` compilation | `src/intent.rs` |
| codegen | WASM invariant code generation from predicates | `src/codegen.rs` |
| compile | Rust → WASM compilation pipeline | `src/compile.rs` |
| predicate | Gherkin step → semantic predicate extraction | `src/predicate.rs` |
| simulation | Pre-flight convergence readiness analysis | `src/simulation.rs` |
| guidance | LLM + heuristic heading quality feedback | `src/guidance.rs` |
| policy_lens | Cedar policy coverage analysis | `src/policy_lens.rs` |
| jtbd | Jobs-to-be-Done metadata extraction | `src/jtbd.rs` |
| truth_package | Deterministic JTBD clause identity, fingerprints, lineage closure, verifier reports, and decoder calibration | `src/truth_package.rs` |
| validation_view | UI-friendly validation result transformation | `src/validation_view.rs` |
| mock_llm | Static chat backend for tests and offline use | `src/mock_llm.rs` |

## Architecture Pages

| Page | Purpose |
|---|---|
| Axiom-Helm-App Contract | Ownership split and operating loops for thin Helm-operated applications backed by Axiom Truth Packages |
| Decoder Calibration | v0.13 learning loop from audited verifier outcomes to richer future JTBD decoding |

## Philosophy Pages

| Page | Purpose |
|---|---|
| Why Axiom | Problem framing for Axiom as the truth layer between intent and governed runtime execution |
| Truth-Driven Development | Methodology for turning business rules into validated, simulated, compiled specifications |
| Composed Vertical AI | Strategic framing for vertical products as JTBD narratives over composed LLM, solver, graph, workflow, and human-accountability systems |

## CLI Commands (cz)

| Command | Purpose |
|---|---|
| `cz doctor` | Environment health check |
| `cz bootstrap` | Dev environment setup |
| `cz validate` | Validate `.truths` files |
| `cz digest` | Summarize open findings |
| `cz ack` | Acknowledge a finding |
| `cz escalate` | Escalate a finding |
| `cz assign` | Assign a finding |
| `cz test` | Run tests |
| `cz fmt` | Format code |
| `cz lint` | Run clippy |
| `cz ci` | Full CI locally |
| `cz up` / `cz down` | Start/stop services |

## Key Types

| Type | Module | Purpose |
|---|---|---|
| `SpecValidation` | gherkin | Complete validation result with issues, confidence, governance |
| `ScenarioMeta` | gherkin | Parsed scenario tags (kind, invariant class, provider) |
| `TruthDocument` | truths | Parsed `.truths` file with Gherkin + governance |
| `TruthGovernance` | truths | Intent, Authority, Constraint, Evidence, Exception blocks |
| `CompileError` | intent | Truth governance to runtime intent compilation errors |
| `CompileFromSourceError` | intent | Combined parse and compile errors for raw source input |
| `CompiledModule` | compile | WASM bytes + manifest + source hash |
| `Predicate` | predicate | Semantic predicate extracted from Gherkin steps |
| `SimulationReport` | simulation | Pre-flight analysis with verdict and findings |
| `PolicyCoverageReport` | policy_lens | Cedar coverage: covered vs uncovered actions |
| `GuidanceResponse` | guidance | Suggested title, rewrite flag, rationale |
| `JTBDMetadata` | jtbd | Actor, jobs (functional/emotional/relational), metrics |
| `JtbdInput` | truth_package | Structured JTBD source supplied by a human or authoring UI |
| `JtbdDocument` | truth_package | Canonical JTBD clauses with stable IDs and fingerprints |
| `ClauseId` | truth_package | Deterministic package-local JTBD clause address |
| `ClauseFingerprint` | truth_package | SHA-256 hash of canonicalized clause text |
| `LineageMap` | truth_package | Artifact-to-clause closure check for generated package artifacts |
| `TruthPackage` | truth_package | Deterministic v0.10 package manifest from JTBD to runtime contract |
| `TruthProjectionOverlay` | truth_package | Human-authored overlay for generated `.truths` projection versions |
| `TruthProjectionVersion` | truth_package | Base or overlay-applied `.truths` projection view |
| `VerifierSpec` | truth_package | Post-run expectations: stop reasons, evidence, forbidden actions, satisfaction conditions |
| `AxiomRunReport` | truth_package | Auditable verifier report with verdict, observed stop reason, promoted facts, evidence/trace links, and integrity proof |
| `AxiomRunStageRecord` | truth_package | Stage-level stop reason, promoted facts, trace links, and integrity proof for multi-boundary jobs |
| `AxiomRunVerdict` | truth_package | Report verdict: Satisfied, Blocked, Exhausted, Invalid |
| `ObservedStopReason` | truth_package | Converge-compatible stop reason shape for report observations |
| `ObservationAdapterReceipt` | truth_package | Deterministic audit receipt for app adapters that map raw transcripts to `AxiomRunObservation` |
| `ObservationAdapterStatus` | truth_package | Adapter receipt status: succeeded or rejected |
| `PromotedFactRecord` | truth_package | Promoted fact summary with source clause IDs, evidence refs, trace link, and observed promotion authority |
| `PromotionAuthorityRecord` | truth_package | Converge promotion gate, policy hash, and approver observed when a fact became authoritative |
| `RunIntegrityProof` | truth_package | Integrity proof summary captured at the Converge boundary |
| `AxiomTruth` | provenance | Zero-sized Axiom provenance marker implementing `converge_pack::ProvenanceSource` |
| `AXIOM_PROVENANCE` | provenance | Canonical Axiom Truth Package provenance constant |
| `TruthPackageSeedPayload` | provenance | Typed Converge payload for Truth-Package-seeded facts |
| `LearningEpisode` | truth_package | Distilled verifier outcome used as decoder calibration input |
| `LearningClauseSignal` | truth_package | Clause-level coverage signal inside a learning episode |
| `CalibrationKey` | truth_package | Deterministic lookup key for a JTBD clause shape and domain hint |
| `CalibrationRecord` | truth_package | Proposed/accepted/rejected/reset decoder prior with rationale and source episode |
| `CalibrationTable` | truth_package | Accepted calibration records queried during package enrichment |

## Marquee Fixtures

| Fixture | Location | Purpose |
|---|---|---|
| Round-driven Formation Design | `tests/round_driven_marquee.rs`; `kb/Marquee/Round-Driven Formation Design.md` | JTBD and staged `AxiomRunReport` fixture for a dynamic design huddle plus selected work Formation |
| Escrow Release | `tests/escrow_release_marquee.rs`; `tests/fixtures/tally_escrow_release_transcript.json`; `kb/Marquee/Escrow Release.md` | Strict-verdict fixture for irreversible commitments, recorded Tally transcript adapter, and v0.13 calibration proof |
| Atlas Integration | `tests/atlas_integration_marquee.rs`; `tests/fixtures/atlas_identity_candidate_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/atlas-integration` | Second app probe for Axiom-Helm-App contract, identity/auth consolidation readiness, and public `ObservationAdapterReceipt` promotion |
| Quorum Sense | `tests/quorum_sense_marquee.rs`; `tests/fixtures/quorum_release_readiness_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/quorum-sense` | Third app probe for Axiom-Helm-App contract, organizational sensemaking readiness, and Helm-owned `JobReadinessPacket` shape |
| Scout Sourcing | `tests/scout_sourcing_marquee.rs`; `tests/fixtures/scout_vendor_selection_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/scout-sourcing` | Fourth app probe for Axiom-Helm-App contract, vendor-selection readiness, HITL/policy evidence, and Helm-owned `JobReadinessPacket` shape |
| Warden Compliance | `tests/warden_compliance_marquee.rs`; `tests/fixtures/warden_compliance_shadow_run_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/warden-compliance` | Fifth app probe for Axiom-Helm-App contract, compliance registry shadow-run readiness, and Helm ledger approval/publication receipt candidates |
| Triage Keeper | `tests/triage_keeper_marquee.rs`; `tests/fixtures/triage_keeper_maintenance_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/triage-keeper` | Sixth app probe for Axiom-Helm-App contract, weekly maintenance readiness, and Helm ledger decision/approval/plan receipt candidates |
| Inkling Notes | `tests/inkling_notes_marquee.rs`; `tests/fixtures/inkling_vault_navigation_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/inkling-notes` | Seventh app probe for Axiom-Helm-App contract, local-first vault enrichment, snapshot/permission/index receipt candidates, and non-destructive suggestion boundaries |
| Plumb Execution | `tests/plumb_execution_marquee.rs`; `tests/fixtures/plumb_strategy_revision_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/plumb-execution` | Eighth app probe for Axiom-Helm-App contract, closed-loop strategy execution readiness, and Helm ledger drift/revision/approval/commit receipt candidates |
| Catalyst Biz | `tests/catalyst_biz_marquee.rs`; `tests/fixtures/catalyst_inbound_account_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/catalyst-biz` | Ninth app probe for Axiom-Helm-App contract, inbound business-operation readiness, and Helm ledger approval/routing/action/outcome receipt candidates |
| Fathom Narrative | `tests/fathom_narrative_marquee.rs`; `tests/fixtures/fathom_temporal_evidence_transcript.json`; `/Users/kpernyer/dev/reflective/marquee-apps/fathom-narrative` | Tenth app probe for Axiom-Helm-App contract, temporal evidence windows, corpus snapshot receipts, disagreement preservation, and cited claim boundaries |

## Scenario Tags

| Tag | Meaning |
|---|---|
| `@invariant` | Scenario is an invariant check |
| `@structural` | Structural invariant class |
| `@semantic` | Semantic invariant class |
| `@acceptance` | Acceptance invariant class |
| `@id:name` | Named identifier |
| `@llm` | Requires LLM provider |
| `@test` | Test-only scenario |
