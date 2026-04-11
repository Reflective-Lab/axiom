---
tags: [stack, llm]
---
# vLLM

vLLM is a high-throughput LLM inference server. Converge consumes it as a remote endpoint — no custom adapter needed.

## Integration Pattern

vLLM exposes an OpenAI-compatible API. Point an `OpenAiProvider` at it:

```rust
OpenAiProvider::new(OpenAiConfig {
    api_key: "not-needed".into(),
    base_url: "http://gpu-server:8000/v1".into(),
    model: "meta-llama/Llama-3.2-70B".into(),
})
```

The provider doesn't know or care that it's talking to vLLM instead of OpenAI. Same [[Architecture/Ports|port]], different backend.

## When to Use

| Scenario | Choice |
|---|---|
| Single-user development | [[Stack/Ollama]] |
| In-process deterministic inference | [[Stack/Burn]] |
| Multi-user production GPU serving | **vLLM** |
| Batch inference at scale | **vLLM** |

## Deployment

Deploy vLLM separately on GPU hardware. Converge connects to it as a remote LLM provider. The hexagonal architecture means swapping between Ollama (dev) and vLLM (prod) is a config change, not a code change.

See also: [[Stack/Ollama]], [[Stack/Burn]], [[Architecture/Providers]]
