---
tags: [architecture]
source: mixed
---
# Providers

Providers are the adapter implementations that plug into [[Architecture/Ports|ports]]. They live outside the hexagon. The core never imports them — they import the core.

## LLM Providers

### Cloud (converge-provider)

All implement `Backend` + `LlmProvider`:

| Provider | Models | Data Sovereignty | Key Capabilities |
|---|---|---|---|
| `AnthropicProvider` | Claude | US | TextGeneration, Reasoning, CodeGeneration, StructuredOutput |
| `OpenAiProvider` | GPT-4, GPT-3.5 | US | TextGeneration, Reasoning, CodeGeneration |
| `GeminiProvider` | Gemini Pro | US | TextGeneration, Reasoning, ImageUnderstanding |
| `OllamaProvider` | Qwen, Llama, Mistral | **Local** | TextGeneration, Embedding |
| `QwenProvider` | Qwen | CN | TextGeneration, Reasoning |
| `MistralProvider` | Mistral, Mixtral | EU | TextGeneration, Reasoning |
| `DeepSeekProvider` | DeepSeek | CN | TextGeneration, Reasoning |
| `PerplexityProvider` | pplx | US | TextGeneration, WebSearch |
| `ApertusProvider` | Apertus | **EU (Switzerland)** | TextGeneration (digital sovereignty) |

### Local Inference (converge-llm)

All implement `LlmBackend`:

| Engine | Framework | GPU Support | Use Case |
|---|---|---|---|
| `LlamaEngine` | llama-burn | CUDA, Metal (Wgpu), CPU (NdArray) | Llama 3.2, LoRA adapters |
| `GemmaEngine` | Burn | CUDA, Metal, CPU | Google Gemma GGUF |
| `TinyLlamaEngine` | Burn | CPU | Resource-constrained environments |
| `GrpcBackend` | Tonic client | Remote GPU | Offload to GPU server |

### vLLM

vLLM is consumed as a remote inference endpoint. Deploy vLLM separately, point an `OpenAiProvider` at its OpenAI-compatible API. No custom adapter needed — vLLM speaks the same protocol.

## Storage Providers

All implement `ExperienceStore`:

| Provider | Protocol | Best For |
|---|---|---|
| `SurrealDbExperienceStore` | WebSocket | Multi-tenant, relational + document queries |
| `LanceDbExperienceStore` | Local/remote | Vector-indexed retrieval, similarity search |
| `InMemoryExperienceStore` | None | Development, testing |

Object stores (S3, GCS, local filesystem) for artifact persistence.

## Search & Embedding Providers

| Provider | Port | Purpose |
|---|---|---|
| LanceDB vector | `VectorRecall` | Similarity search over embeddings |
| Qdrant | `VectorRecall` | Managed vector search |
| FastEmbed | `Embedding` | Local vector embedding generation |
| Ollama embeddings | `Embedding` | Local embedding via nomic-embed, mxbai-embed |
| Cloud embeddings | `Embedding` | OpenAI, Hugging Face embedding APIs |
| Cross-encoder | `Reranking` | Re-rank search results by relevance |

## Optimization Providers

| Provider | Port | Purpose |
|---|---|---|
| OR-Tools (CP-SAT) | Constraint solver | Scheduling, resource allocation, multi-criteria optimization |

## Capability Presets

```rust
BackendRequirements::fast_llm()           // Low cost, fast response
BackendRequirements::reasoning_llm()      // High cost, reasoning-capable
BackendRequirements::access_policy()      // Policy engine
BackendRequirements::constraint_solver()  // Optimization
BackendRequirements::embedding_pipeline() // Vector embeddings
BackendRequirements::vector_search()      // Similarity recall
```

## The Adapter Rule

Providers produce **observations, never decisions** ([[Philosophy/Nine Axioms#4. Agents Suggest, Engine Decides|Axiom 4]]). An LLM response becomes an `Observation`. An agent turns that into a `ProposedFact`. The engine's promotion gate decides whether it becomes a `Fact`. The provider has no say in governance.

See also: [[Architecture/Ports]], [[Architecture/Hexagonal Architecture]], [[Concepts/Backends and Capabilities]]
