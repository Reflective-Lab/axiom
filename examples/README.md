# Converge Examples

Self-contained examples demonstrating Converge capabilities.
Each subdirectory is a standalone project with its own `Cargo.toml`.

## Quick Start

```bash
just examples
just example hello-convergence
```

Most examples map directly from the directory name to the package name:

- `just example hello-convergence`
- `cargo run -p example-hello-convergence`

Inference examples are the exception: they may need model files, feature flags,
or extra setup. See the notes below before running them.

## Example Catalog

| Example | What it shows | Run |
|---------|--------------|-----|
| [hello-convergence](hello-convergence/) | Core convergence loop: agents, facts, invariants | `just example hello-convergence` |
| [custom-agent](custom-agent/) | Implement a custom `Suggestor` | `just example custom-agent` |
| [meeting-scheduler](meeting-scheduler/) | Domain pack with constraint agents | `just example meeting-scheduler` |
| [custom-provider](custom-provider/) | Implement an LLM provider adapter | `just example custom-provider` |
| [vendor-selection](vendor-selection/) | Multi-criteria vendor selection with default Cedar-backed commit gating | `just example vendor-selection` |
| [expense-approval](expense-approval/) | Governed expense approval through the default `FlowGateAuthorizer` contract | `just example expense-approval` |
| [loan-application](loan-application/) | Loan application processing with domain packs and traits | `just example loan-application` |
| [local-inference](local-inference/) | Local inference example; defaults to the `gemma` feature | See [local-inference/README.md](local-inference/README.md) |
| [gemma-inference](gemma-inference/) | Minimal Gemma GGUF inference via `llama.cpp` | `cargo run -p example-gemma-inference` |

## Notes

- Gemma examples require a local GGUF model file in `~/models/` or at the path pointed to by `CONVERGE_GEMMA_MODEL_PATH`.
- `local-inference` has its own setup guide because it may need `CONVERGE_GEMMA_MODEL_PATH`, explicit features, and Apple Silicon / GPU-specific configuration.
- `gemma-inference` is a focused `converge-llm` example, not a general business-flow example.

## For Partners

Start with **hello-convergence** to understand the core model, then move to
**custom-agent** to build your own. The remaining examples show specific
capabilities you can compose.

If you need the canonical governance pattern, start with **expense-approval**
and **vendor-selection**. They show the intended default path for consequential
business actions:

- flow state projects into `FlowGateInput`
- `converge-policy` evaluates Cedar rules through `FlowGateAuthorizer`
- outcomes are `promote`, `reject`, or `escalate`
- HITL approval resumes the same convergence loop
