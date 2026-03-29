# Converge.zone

**Converge** is a correctness-first, context-driven multi-agent runtime built in Rust.

Agents collaborate through shared context, not by calling each other. The engine runs agents repeatedly until a fixed point is reached — convergence is explicit and observable.

## Design Principles

1. **Agents suggest, engines decide.** `ProposedFact` is not `Fact`.
2. **Context is the API.** Agents communicate through shared context.
3. **Append-only truth.** Facts are never mutated; corrections are new facts.
4. **Safety by construction.** `unsafe_code = "forbid"` everywhere.

## Quick Start

```bash
git clone https://github.com/Reflective-Labs/converge.zone.git
cd converge.zone

just build-quick    # build (fast iteration)
just test           # run tests
just lint           # format + clippy
```

## Examples

```bash
just example hello-convergence   # engine loop, agents, facts
just example custom-agent        # implement the Agent trait
just example meeting-scheduler   # domain pack with constraints
just example custom-provider     # implement an LLM provider
```

See [examples/README.md](examples/README.md) for the full list.

## Architecture

```
crates/
├── traits/        # Public contract — partners implement these
├── core/          # Convergence engine
├── provider/      # Remote LLM providers (Anthropic, OpenAI, Gemini, ...)
├── domain/        # Domain packs (scheduling, routing, drafting, ...)
├── experience/    # Event-sourced audit store
├── knowledge/     # Vector knowledge base
├── optimization/  # Constraint solving (OR-Tools)
├── policy/        # Cedar policy engine
├── llm/           # Local LLM inference (Burn)
├── analytics/     # ML/analytics agents
├── tool/          # Development toolchain (Gherkin, JTBD)
├── runtime/       # HTTP/gRPC execution service
├── remote/        # gRPC client to runtime
└── application/   # Reference distribution
examples/
├── hello-convergence/   # Minimal convergence loop
├── custom-agent/        # Implement the Agent trait
├── meeting-scheduler/   # Domain pack with constraints
├── custom-provider/     # LLM provider adapter
└── local-inference/     # Local inference on Apple Silicon
```

## Publishable Crates

Nine crates are published to [crates.io](https://crates.io):

| Crate | Role |
|-------|------|
| `converge-traits` | Public contract — traits and types |
| `converge-core` | Convergence engine |
| `converge-provider` | Remote LLM provider adapters |
| `converge-domain` | Domain packs and use cases |
| `converge-experience` | Event-sourced audit store |
| `converge-knowledge` | Vector knowledge base |
| `converge-optimization` | Constraint solving |
| `converge-tool` | Development toolchain |
| `ortools-sys` | OR-Tools FFI bindings |

## Configuration

```env
CONVERGE_LLM_BACKEND=ndarray
CONVERGE_LLM_MODEL=llama3
CONVERGE_STORAGE_BACKEND=lancedb
CONVERGE_STORAGE_PATH=./data
RUST_LOG=info
```

## Documentation

- [DEVELOPMENT.md](DEVELOPMENT.md) — setup, build, git workflow (worktrees, jj)
- [CONTRIBUTING.md](CONTRIBUTING.md) — contribution guidelines
- [SECURITY.md](SECURITY.md) — vulnerability reporting
- [docs/deployment/QUICKSTART.md](docs/deployment/QUICKSTART.md) — local, container, and hosted startup
- [docs/deployment/TERRAFORM_GCP.md](docs/deployment/TERRAFORM_GCP.md) — default Google Cloud + Secret Manager deployment path
- [docs/deployment/GPU_WORKERS.md](docs/deployment/GPU_WORKERS.md) — Modal / Runpod guidance for GPU inference and training
- [crates/provider/.env.example](crates/provider/.env.example) — provider API key and endpoint catalog
- [docs/security/README.md](docs/security/README.md) — enterprise security and compliance package
- [docs/security/DATA_HANDLING_DECLARATION.md](docs/security/DATA_HANDLING_DECLARATION.md) — what this project is designed to handle and what it is not
- [docs/security/COMPLIANCE_READINESS.md](docs/security/COMPLIANCE_READINESS.md) — current declarations, recommendations, and non-claims
- [examples/](examples/) — runnable examples

## License

MIT — see [LICENSE](LICENSE).

Copyright 2024-2026 Reflective Labs

Kenneth Pernyer — [kenneth@reflective.se](mailto:kenneth@reflective.se)

Project: [github.com/Reflective-Labs/converge.zone](https://github.com/Reflective-Labs/converge.zone)
