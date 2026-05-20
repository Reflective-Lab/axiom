# Axiom

[![CI](https://github.com/Reflective-Lab/axiom/actions/workflows/ci.yml/badge.svg)](https://github.com/Reflective-Lab/axiom/actions/workflows/ci.yml)
[![Security](https://github.com/Reflective-Lab/axiom/actions/workflows/security.yml/badge.svg)](https://github.com/Reflective-Lab/axiom/actions/workflows/security.yml)
[![Stability](https://github.com/Reflective-Lab/axiom/actions/workflows/stability.yml/badge.svg)](https://github.com/Reflective-Lab/axiom/actions/workflows/stability.yml)
[![Dependency Analysis](https://github.com/Reflective-Lab/axiom/actions/workflows/dependency-analysis.yml/badge.svg)](https://github.com/Reflective-Lab/axiom/actions/workflows/dependency-analysis.yml)
![coverage](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/kpernyer/91948402ce6ceccca2cd19b2ff83248a/raw/axiom-coverage.json)
[![Crates.io](https://img.shields.io/crates/v/axiom-truth.svg)](https://crates.io/crates/axiom-truth)
[![docs.rs](https://docs.rs/axiom-truth/badge.svg)](https://docs.rs/axiom-truth)
[![dependency status](https://deps.rs/repo/github/Reflective-Lab/axiom/status.svg)](https://deps.rs/repo/github/Reflective-Lab/axiom)
![MSRV](https://img.shields.io/badge/MSRV-1.94.0-blue)
<img alt="gitleaks badge" src="https://img.shields.io/badge/protected%20by-gitleaks-blue">
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**The truth layer** — JTBD decoding, Truth Packages, validation, simulation, intent compilation, run verification, and decoder calibration for the Reflective stack.

Axiom turns human jobs and Gherkin-style `.truths` projections into auditable runtime contracts. It generates Rust invariant code, compiles governed intent for Organism, and verifies whether observed runs satisfied the truth they claimed to serve.

```
┌─────────────────────────────────────────────┐
│  Helm          Decision frameworks          │
├─────────────────────────────────────────────┤
│  Axiom         Truth, intent, run proof     │  ← you are here
├─────────────────────────────────────────────┤
│  Organism      Formations, planning, debate │
├─────────────────────────────────────────────┤
│  Converge      Fixed-point engine, commit   │
├─────────────────────────────────────────────┤
│  Mosaic        Providers, tools, analytics  │
└─────────────────────────────────────────────┘
```

## Release Slice: Axiom Layer v0.15

The releaseable Axiom layer is deliberately narrow:

1. Decode a structured `JtbdInput` into a deterministic `TruthPackage`.
2. Preserve stable clause IDs, clause fingerprints, generated `.truths`, proof obligations, verifier specs, replay metadata, and lineage.
3. Compile a package's truth projection into `organism_pack::IntentPacket` and WASM invariant artifacts.
4. Verify a normalized `AxiomRunObservation` into an `AxiomRunReport` with a verdict: `Satisfied`, `Blocked`, `Exhausted`, or `Invalid`.
5. Convert verified outcomes into reviewable decoder calibration records, suggestions, and concerns.

Everything else stays outside Axiom for this release. Helm owns operator review surfaces and plugin hosting. Apps own domain state and app-specific transcript adapters. Organism owns Formation selection. Mosaic owns concrete capabilities and suggestors. Converge owns promotion authority, stop reasons, and integrity.

## A New World

The old world demanded that all ambiguity be drained *before* execution; the new world keeps the gap between human intent and machine decision open and closes it safely at runtime. But "safely" only works if the upfront contract — the thing the human signs onto — is precise and enforceable. Axiom compiles that contract.

**Why it matters.** Truth Documents, validated and codegenned into Rust + WASM invariants, are how upfront human intent becomes runtime-enforceable structure. Without Axiom, the rest of the stack has no fixed point worth converging on; with it, the new world keeps its rigor where the old world used to keep its workflow.

## Runtime Boundary: Truth to Formation to Fixed Point

Axiom proves a human-authored job can remain legible across the stack:

```text
JtbdInput
  -> Axiom decodes a TruthPackage
  -> Axiom validates, simulates, and compiles IntentPacket + invariants
  -> Organism admits intent, selects a Formation, and instantiates Suggestors
  -> Mosaic supplies concrete providers, tools, analytics, and adapters
  -> Converge runs the Formation to StopReason::Converged / CriteriaMet
  -> an app or runtime adapter emits AxiomRunObservation
  -> Axiom returns AxiomRunReport
```

Axiom does not replace Helm, Organism, Mosaic, Converge, or the app. Its release job is to make the stack legible: show the source job, generated truth, compiled intent, observed stop reason, promoted facts, evidence refs, trace links, integrity proof, and verdict in one auditable report.

## What it does

| Module | Purpose |
|---|---|
| `gherkin` | Validate `.truths` specs for business sense, compilability, and conventions |
| `truths` | Parse governance blocks from `.truths` sources |
| `intent` | Compile `TruthDocument` into `organism_pack::IntentPacket` |
| `codegen` | Generate Rust invariant code from validated specs |
| `compile` | Compile and verify generated invariants |
| `simulation` | Simulate convergence readiness before runtime |
| `guidance` | Contextual guidance for spec authors |
| `policy_lens` | Policy analysis and compliance checking |
| `jtbd` | Jobs-to-be-done framework integration |
| `truth_package` | Deterministic JTBD decoding, Truth Packages, run observations, reports, adapter receipts, lineage closure, and calibration |
| `predicate` | Predicate logic for truth evaluation |
| `validation_view` | UI-friendly validation and governance summaries |

## CLI

Axiom ships `cz`, a workspace orchestrator:

```bash
cz doctor      # Check environment health
cz bootstrap   # Set up development environment
cz validate    # Validate .truths specs
cz test        # Run all tests
```

## Quick Start

```bash
git clone https://github.com/Reflective-Lab/axiom.git
cd axiom

just build      # Build (release)
just test       # Run tests
just lint       # Format + clippy
```

## Library Usage

```rust
use axiom_truth::gherkin::{GherkinValidator, ValidationConfig};

let validator = GherkinValidator::new(backend, ValidationConfig::default());
let result = validator.validate_file("specs/money.truths").await?;
```

Compile a `.truths` source into Organism's runtime intent contract:

```rust
use axiom_truth::compile_intent_from_source;

let intent = compile_intent_from_source(source)?;
```

Decode a structured job and verify an observed run:

```rust
use axiom_truth::{AxiomRunReport, AxiomRunObservation, JtbdInput, decode_jtbd};

fn verify_run(observation: AxiomRunObservation) -> Result<AxiomRunReport, Box<dyn std::error::Error>> {
    let package = decode_jtbd(JtbdInput::new(
        "vendor_selection",
        "procurement lead",
        "select a vendor with governed evidence",
        "the business can approve spend without hidden risk",
    ))?;

    Ok(AxiomRunReport::verify(&package, observation))
}
```

## Stack Boundary

Current Axiom uses the narrow public stack surfaces:

- `converge-provider` for chat contracts, provider capability vocabulary, and selection types
- `converge-manifold-adapters` for concrete backend selection helpers
- `organism-pack` for `IntentPacket` and related runtime intent types

Run verification is an app-neutral Axiom surface over normalized observations. Organism selects and instantiates Formations, Converge owns the engine and promotion gate, Mosaic supplies concrete provider and suggestor capabilities, Helm displays and journals operator-facing views, and apps own domain transcripts and adapters.

## Architecture

See [architecture/](architecture/) and [kb/Architecture/Truth-to-Formation Run Proof.md](kb/Architecture/Truth-to-Formation%20Run%20Proof.md) for ADRs, API surface documentation, and the v0.9 proof target.

## License

[MIT](LICENSE) — Copyright 2024–2026 Reflective Group AB
