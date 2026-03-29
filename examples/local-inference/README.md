# Local Inference Example

Run LLM inference locally on Apple Silicon (M1/M2/M3/M4) using Burn.

## Quick Start

```bash
# CPU (always works)
cargo run -p example-local-inference --features "ndarray,llama3,pretrained" --release

# Metal GPU (recommended on macOS)
cargo run -p example-local-inference --features "wgpu,llama3,pretrained" --release

# Quick test with tiny model
cargo run -p example-local-inference --features "wgpu,tiny,pretrained" --release
```

## Expected Performance (M4 Mac)

| Model | Tokens/sec |
|-------|-----------|
| Tiny (1.1B) | ~100+ |
| Llama 3.2 3B (quantized) | ~20-40 |
| Llama 3 8B (quantized) | ~10-20 |

First run downloads model weights (~2-6 GB depending on model).
