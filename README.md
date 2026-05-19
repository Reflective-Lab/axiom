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

**The truth layer** — validation, simulation, guidance, intent compilation, and run-proof surface for [Converge](https://github.com/Reflective-Lab/converge).

Axiom validates business specifications written in Gherkin-style `.truths` files, generates Rust invariant code, compiles governed intent for Organism, and explains whether a truth can drive a real Converge run to a fixed point.

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

## A New World

The old world demanded that all ambiguity be drained *before* execution; the new world keeps the gap between human intent and machine decision open and closes it safely at runtime. But "safely" only works if the upfront contract — the thing the human signs onto — is precise and enforceable. Axiom compiles that contract.

**Why it matters.** Truth Documents, validated and codegenned into Rust + WASM invariants, are how upfront human intent becomes runtime-enforceable structure. Without Axiom, the rest of the stack has no fixed point worth converging on; with it, the new world keeps its rigor where the old world used to keep its workflow.

## Direction: Truth to Formation to Fixed Point

The next Axiom layer is the proof that a human-authored truth can drive the full stack:

```text
.truths
  -> Axiom validates, simulates, and compiles IntentPacket
  -> Organism admits intent, selects a Formation, and instantiates Suggestors
  -> Mosaic supplies concrete providers, tools, analytics, and adapters
  -> Converge runs the Formation to StopReason::Converged / CriteriaMet
  -> Axiom returns the run proof
```

Axiom should not replace Organism or Converge. Its job is to make the stack legible: show the validated truth, the compiled intent, the selected formation, the provider assignments, the Converge stop reason, the promoted facts, and the integrity proof in one auditable report.

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
| `truth_package` | Deterministic JTBD clause identity, fingerprints, and lineage closure |
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

## Stack Boundary

Current Axiom uses the narrow public stack surfaces:

- `converge-provider` for chat contracts, provider capability vocabulary, and selection types
- `converge-manifold-adapters` for concrete backend selection helpers
- `organism-pack` for `IntentPacket` and related runtime intent types

The v0.9 run-proof work should add an end-to-end integration path without moving ownership: Organism selects and instantiates Formations, Converge owns the engine and promotion gate, and Mosaic supplies concrete provider and suggestor capabilities.

## Architecture

See [architecture/](architecture/) and [kb/Architecture/Truth-to-Formation Run Proof.md](kb/Architecture/Truth-to-Formation%20Run%20Proof.md) for ADRs, API surface documentation, and the v0.9 proof target.

## License

[MIT](LICENSE) — Copyright 2024–2026 Reflective Group AB
