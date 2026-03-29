# GPU Workers and Training

## Short Answer

If you want to start `converge-training` or GPU-backed inference quickly:

- keep `converge-runtime` on Cloud Run
- use a separate GPU worker for model inference or training
- prepare one worker shape for each of:
  - Cloud Run GPU
  - Runpod
  - Modal

## What Fits Where

### Good fit for Cloud Run

- `converge-runtime`
- stateless HTTP APIs
- orchestration and job submission
- lightweight validation or policy services

### Good fit for Modal or Runpod

- `converge-llm-server`
- local-model inference with GPU backends
- fine-tuning, LoRA, or heavy batch jobs
- experiments that need CUDA and large model weights

## Current Repository Status

The repository already contains:

- local inference examples for Apple Silicon
- GPU feature flags in `converge-llm`
- analytics/training code paths
- deployment templates under `deploy/gpu/`

What is now prepared:

- `deploy/gpu/cloudrun/` for Cloud Run GPU
- `deploy/gpu/runpod/` for Runpod-style worker images
- `deploy/gpu/modal/` for a Modal worker starter

What is still intentionally incomplete:

- production artifact registry for model weights
- training orchestration and checkpoint lifecycle
- mTLS/auth wiring between runtime and worker

## Practical Recommendation

Start with this architecture:

1. Deploy `converge-runtime` on Cloud Run.
2. Package a separate GPU worker container for `converge-llm-server`.
3. Let the runtime call the GPU worker over HTTP or gRPC.
4. Keep model weights and training outputs outside the runtime container.

## Modal vs Runpod

### Modal

Better when you want:

- fast iteration
- Python-driven orchestration
- batch jobs and endpoints with less infra work

Less ideal if you want the whole stack to stay purely Rust-native.

### Runpod

Better when you want:

- more direct GPU VM control
- long-running model servers
- custom CUDA images

Less convenient than Modal for quick developer UX, but usually closer to a
standard container deployment model.

## Recommendation For This Repo

If your immediate goal is "get something useful running fast":

- use native or container local startup for development
- use Cloud Run for the main runtime
- use Cloud Run GPU first if you want one Google-native path
- use Modal for fast experimentation
- use Runpod when you need steadier container-centric GPU hosting
