---
tags: [building]
source: llm
---

# Getting Started

## Prerequisites

- Rust 1.94+ (edition 2024)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `just` task runner: `brew install just` or `cargo install just`

## Build

```bash
git clone https://github.com/Reflective-Lab/axiom.git
cd axiom

just build       # Release build
just build-quick # Fast iteration
just check       # Type check only
```

## Test

```bash
just test        # All tests
just test-one NAME  # Single test
```

## Lint

```bash
just lint        # Format check + clippy
just fix-lint    # Auto-fix
```

## Environment

Create a `.env` file (see `.env.example` if available) or set:

```env
RUST_LOG=info
```

LLM validation requires provider API keys configured through Converge's provider system.
