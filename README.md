# Axiom

**The truth layer** — validation, simulation, guidance, and policy lens for [Converge](https://github.com/Reflective-Lab/converge).

Axiom validates business specifications written in Gherkin-style `.truths` files using LLMs, generates Rust invariant code, and provides policy analysis tooling.

```
┌─────────────────────────────────────────────┐
│  Helm          Decision frameworks          │
├─────────────────────────────────────────────┤
│  Axiom         Truth validation & codegen   │  ← you are here
├─────────────────────────────────────────────┤
│  Organism      Reasoning, planning, debate  │
├─────────────────────────────────────────────┤
│  Converge      Engine, governance, commit   │
├─────────────────────────────────────────────┤
│  Providers     LLMs, tools, storage         │
└─────────────────────────────────────────────┘
```

## What it does

| Module | Purpose |
|---|---|
| `gherkin` | Validate `.truths` specs for business sense, compilability, and conventions |
| `codegen` | Generate Rust invariant code from validated specs |
| `compile` | Compile and verify generated invariants |
| `simulation` | Simulate outcomes against specs |
| `guidance` | Contextual guidance for spec authors |
| `policy_lens` | Policy analysis and compliance checking |
| `jtbd` | Jobs-to-be-done framework integration |
| `predicate` | Predicate logic for truth evaluation |

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

## Converge Boundary

Axiom uses the narrow Converge provider surface for live validation help:

- `converge-provider-api` for chat contracts and selection vocabulary
- `converge-provider` for concrete backend implementations and selection helpers

Axiom does not depend on the Converge engine crate as part of its public integration contract.

## Architecture

See [architecture/](architecture/) for ADRs and API surface documentation.

## License

[MIT](LICENSE) — Copyright 2024–2026 Reflective Group AB
