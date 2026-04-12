# Converge — Capabilities

What Converge provides to layers built on top of it.

## Convergence Engine

The core loop: agents suggest, the engine decides. Proposals carry confidence and provenance. A promotion gate validates before anything becomes fact. Convergence is explicit — the engine runs until `Context(n+1) == Context(n)` or a termination condition is met.

- Four-way criterion results: Met, Blocked, Unmet, Indeterminate
- Honest stopping: `HumanInterventionRequired` with typed approval references
- Pack-scoped execution: agents belong to packs, intents activate specific packs
- Durable context: `ContextStore` for persistence across runs
- Full provenance on every fact and proposal

## Optimization

Pure Rust implementations — no external solver required.

| Algorithm | Problem | Notes |
|---|---|---|
| Hungarian (O(n³)) | Assignment | Rectangular matrices, dual potentials |
| Auction | Assignment | ε-complementary slackness |
| Push-Relabel (Goldberg-Tarjan) | Max flow | FIFO active-node queue |
| Successive Shortest Paths | Min-cost flow | Multi-supply/demand, infeasibility detection |
| Dijkstra | Shortest path | |
| Dynamic programming | 0-1 Knapsack | Backtracking, capacity guard |
| Greedy (ln(n) approx) | Set cover | Cost-effectiveness ratio |
| List scheduling | Scheduling | Disjunctive + cumulative resource constraints |

Five domain solver packs with typed problem specs, invariant suites, and determinism guarantees: meeting scheduling, inventory rebalancing, lead routing, budget allocation, anomaly triage.

Optional C++ OR-Tools FFI via `ortools-sys` (feature `ffi`). Native CP-SAT via Varisat (feature `sat`).

## Knowledge Base

Self-learning knowledgebase with vector search and agentic memory.

- **Vector search** with configurable HNSW parameters (m, ef_construction, ef_search)
- **GNN learning layer** — message-passing with attention, Xavier-initialized weights
- **AgenticDB** — five memory tables:
  - Reflexion memory (failure pattern matching)
  - Skill library (success rate tracking, usage counting)
  - Causal memory (hypergraph with evidence strength)
  - Learning sessions (turn-by-turn RL trajectories)
  - Temporal memory (time crystals, occurrence prediction)
- **Online learning** with distribution drift detection
- **Meta-learning** — MAML/Reptile-style task adaptation with few-shot strategy selection
- **Ingest** — PDF and Markdown parsers
- **Interfaces** — gRPC server, MCP server, CLI

## Policy Engine

Cedar-based Policy Decision Point.

- Real Amazon Cedar authorizer
- Agent authority levels: advisory, participatory, supervisory, sovereign
- Commitment actions: propose, commit, promote
- Amount thresholds and human approval requirements
- Phase gate enforcement
- Ed25519-signed, CBOR-encoded delegation tokens with time-scoping and replay protection

## Analytics & ML

Polars + Burn pipeline, wired as Converge agents.

- Temporal feature extraction and z-score analysis (Polars)
- Burn MLP training and inference
- Full training pipeline as Suggestor agents: dataset download, validation, feature engineering, hyperparameter search, model registry, monitoring, deployment decisions
- Object storage integration for model artifacts (feature `storage`)

## LLM Reasoning Kernel

Local inference with adapter lifecycle governance.

- **Inference engines**: LlamaEngine (Llama 3), TinyLlamaEngine, GemmaEngine (GGUF via llama.cpp) — all feature-gated
- **LoRA**: real Burn modules with low-rank A/B matrices, alpha/rank scaling, checkpoint persistence
- **Weight merging**: merge plans, delta canonicalization, layer mapping, verification
- **Adapter lifecycle**: state machine (Detached → Loading → Active → Merging → Error) with rollback
- **Recall**: blake3 deterministic embedder, optional fastembed semantic embedder, PII redaction, structurally enforced Recall ≠ Evidence boundary
- **Adversarial harness**: 6 scenario categories (Contradictory, Boundary, Underspecified, SemanticAdversarial, Extreme, Baseline)
- **Observability**: metrics recording for inference, adapter, recall, and backend operations
- **Remote inference**: gRPC server and client for GPU-isolated workers

## LLM Providers

14 remote providers plus local inference.

| Provider | Feature flag |
|---|---|
| Anthropic (Claude) | `anthropic` (default) |
| OpenAI (GPT-4/3.5) | `openai` (default) |
| Google Gemini | `gemini` (default) |
| Ollama (local) | `ollama` |
| Perplexity | `perplexity` |
| Mistral | `mistral` |
| DeepSeek | `deepseek` |
| Qwen | `qwen` |
| Grok (xAI) | `grok` |
| Kimi (Moonshot) | `kimi` |
| Baidu ERNIE | `baidu` |
| Zhipu GLM | `zhipu` |
| MinMax | `minmax` |
| Apertus | `apertus` |
| OpenRouter | (via OpenAI-compatible) |
| Kong AI Gateway | `kong` |

Plus: fitness-based model selection, fallback chaining, provider-specific prompt builders, and secret management with zeroizing.

## Tool Integration

- **OpenAPI**: parse specs, convert operations to tool definitions with tag filtering
- **GraphQL**: parse introspection, convert queries/mutations/subscriptions to tools
- **MCP**: full 2024-11-05 protocol types, stdio and HTTP server transports
- **Brave Search**: web search capability

## Experience Store

Event-sourced audit ledger with three backends:

| Backend | Feature | Notes |
|---|---|---|
| In-memory | (always on) | Property-tested, input validation |
| SurrealDB | `surrealdb` | WebSocket, namespaced |
| LanceDB | `lancedb` | Vector-indexed, Arrow schema, similarity search |

## Object Storage

Unified `Arc<dyn ObjectStore>` abstraction:

| Backend | Feature | URI scheme |
|---|---|---|
| Local filesystem | `local` (default) | `file://` |
| S3-compatible (AWS, MinIO, RustFS) | `s3` | `s3://` |
| Google Cloud Storage | `gcs` | `gs://` |

## Wire Protocol

- Protobuf package `converge.v1` — bidirectional streaming for mobile/CLI
- Generated Rust types in `converge-protocol`
- Idiomatic client SDK in `converge-client`

## Guarantees

- `unsafe_code = "forbid"` across all crates
- Determinism: seeded randomness, reproducible solver reports
- Termination: cycle/fact/token budgets
- Isolation: agents cannot affect each other
- Auditability: full provenance chain on every fact and proposal
- Supply chain: `cargo-deny` for RUSTSEC advisories and license compliance
