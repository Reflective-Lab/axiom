# converge.zone

Converge Agent OS — a correctness-first, context-driven multi-agent runtime.

## Crate Map

| Crate | Description | License |
|-------|-------------|---------|
| `converge-traits` | Public trait contracts and backend abstraction | MIT |
| `converge-core` | Core engine — contexts, invariants, convergence | MIT |
| `converge-provider` | LLM provider implementations (Anthropic, Gemini, OpenAI, etc.) | MIT |
| `converge-domain` | Domain-specific agent packs | Proprietary |
| `converge-experience` | Experience store (SurrealDB, LanceDB) | MIT |
| `converge-knowledge` | Vector search knowledgebase with gRPC and MCP | MIT |
| `converge-optimization` | Optimization algorithms (OR-Tools subset in Rust) | Apache-2.0 |
| `converge-analytics` | Analytics and ML pipeline (Polars, Burn) | Proprietary |
| `converge-llm` | Local LLM inference and training (Burn) | Proprietary |
| `converge-policy` | Cedar-based policy decision point | Proprietary |
| `converge-runtime` | HTTP/gRPC server, consensus, crypto, identity | Proprietary |
| `converge-remote` | gRPC client driver for the runtime | Proprietary |
| `converge-tool` | Dev tools, Gherkin validation, `cz` CLI | Proprietary |
| `converge-application` | Full distribution binary | Proprietary |

## Getting Started

```bash
# Build (default members — excludes heavy crates)
make build-quick

# Run tests
make test

# Full workspace build including analytics, llm, runtime
cargo build --workspace --release

# Lint
make lint
```

## License

See individual crate licenses. Core platform crates are MIT-licensed.
Public optimization crates are Apache-2.0. Domain-specific and infrastructure
crates are proprietary to Reflective Labs.

Copyright (c) 2024-2026 Reflective Labs
