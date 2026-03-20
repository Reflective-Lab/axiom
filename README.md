# Converge.zone

**Converge** is a correctness-first, context-driven multi-agent runtime built in Rust. It provides a modular platform for building and deploying intelligent agents with LLM inference capabilities.

## Overview

Converge is designed as a workspace of 15 Rust crates that work together to provide:

- **Agent Runtime**: Context-driven execution engine
- **LLM Inference**: Local and remote LLM support using Burn framework
- **Knowledge Management**: Semantic embedding and recall systems
- **Policy Engine**: Rule-based decision making
- **Multi-Backend Support**: LanceDB, SurrealDB, and other storage options

## Quick Start

### Prerequisites

- Rust 1.90+
- Cargo (comes with Rust)
- Optional: CUDA/Vulkan/WGPU for GPU acceleration

### Building

```bash
# Clone the repository
git clone https://github.com/Reflective-Labs/converge.zone.git
cd converge.zone

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

### Running Examples

```bash
# LLM inference example
cargo run --example local_inference --features "llama3,ndarray"

# Start the gRPC server
cargo run --bin converge-llm-server --features "server,llama3"
```

## Architecture

```
converge.zone/
├── crates/
│   ├── traits/          # Core trait definitions
│   ├── core/            # Agent runtime engine
│   ├── llm/             # LLM inference (Burn-based)
│   ├── domain/          # Domain models
│   ├── experience/      # Experience management
│   ├── knowledge/       # Knowledge base
│   ├── policy/          # Policy engine
│   ├── runtime/         # Runtime environment
│   ├── provider/        # External providers
│   ├── analytics/       # Analytics
│   ├── optimization/    # Optimization algorithms
│   ├── remote/          # Remote services
│   ├── tool/            # Tooling
│   └── application/     # Application layer
└── Cargo.toml           # Workspace configuration
```

## Features

### Core Features

- **Multi-Agent Runtime**: Execute multiple agents with context sharing
- **LLM Backends**: Local (Llama3, TinyLLM) and remote (Anthropic) support
- **Knowledge Recall**: Semantic embedding with ONNX models
- **Policy Engine**: Cedar-based policy evaluation
- **Observability**: OpenTelemetry integration

### Backend Support

- **Storage**: LanceDB, SurrealDB, Firestore
- **Compute**: CUDA, Vulkan, WGPU, CPU
- **Network**: gRPC, HTTP, NATS

## Configuration

Create a `.env` file or set environment variables:

```env
# LLM Configuration
CONVERGE_LLM_BACKEND=ndarray
CONVERGE_LLM_MODEL=llama3

# Storage Configuration
CONVERGE_STORAGE_BACKEND=lancedb
CONVERGE_STORAGE_PATH=./data

# Logging
RUST_LOG=info
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright © 2024 Reflective Group AB

## Contact

Kenneth Pernyer - [kenneth@reflective.se](mailto:kenneth@reflective.se)

Project Link: [https://github.com/Reflective-Labs/converge.zone](https://github.com/Reflective-Labs/converge.zone)