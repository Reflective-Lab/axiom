# Converge Examples

Self-contained examples demonstrating Converge capabilities.
Each subdirectory is a standalone project with its own `Cargo.toml`.

## Getting Started

| Example | What it shows | Run |
|---------|--------------|-----|
| [hello-convergence](hello-convergence/) | Engine loop, agents, facts, invariants | `cargo run -p example-hello-convergence` |
| [custom-agent](custom-agent/) | Implement the `Suggestor` trait | `cargo run -p example-custom-agent` |
| [meeting-scheduler](meeting-scheduler/) | Domain pack with constraint agents | `cargo run -p example-meeting-scheduler` |
| [custom-provider](custom-provider/) | Implement an LLM provider adapter | `cargo run -p example-custom-provider` |
| [vendor-selection](vendor-selection/) | Multi-criteria vendor selection | `cargo run -p example-vendor-selection` |
| [expense-approval](expense-approval/) | Governed expense approval flow | `cargo run -p example-expense-approval` |
| [loan-application](loan-application/) | Loan application processing | `cargo run -p example-loan-application` |
| [local-inference](local-inference/) | Run LLM inference on Apple Silicon | See [local-inference/README.md](local-inference/README.md) |
| [gemma-inference](gemma-inference/) | Gemma GGUF inference via llama.cpp | `cargo run -p example-gemma-inference` |

## For Partners

Start with **hello-convergence** to understand the core model, then move to
**custom-agent** to build your own. The remaining examples show specific
capabilities you can compose.
