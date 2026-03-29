# Converge.zone — Codex Handoff

This file captures the most useful project-specific context learned during
recent work, with emphasis on deployment, verification, and what is worth
picking up next time.

## What This Repo Is

Converge is a Rust workspace for a correctness-first multi-agent runtime.

Important crates for deployment work:

- `crates/runtime` — HTTP/gRPC runtime service
- `crates/application` — CLI/TUI distribution layer
- `crates/provider` — external provider adapters and model catalog
- `crates/llm` — local / GPU inference server and model runtime

## Current Deployment Direction

The repo is now explicitly oriented toward:

- local native startup for macOS/Linux
- local container startup with Docker/Podman-compatible compose
- default hosted path on Google Cloud Run
- default identity posture using Firebase Auth
- default infra posture using Terraform + Google Secret Manager
- optional GPU workers separated from the main runtime

## Reference Implementation

Use `wolfgang-app` as the arrowhead for deployment and auth decisions:

- `/Users/kpernyer/repo/wolfgang-app`

Why:

- it already has a real frontend/backend split
- it already uses Firebase client auth in the web app
- it already uses bearer-token auth middleware in the Rust backend
- it already has Terraform modules for Cloud Run, Artifact Registry, LB, and IAM
- it already uses Secret Manager-driven runtime configuration

Treat the Wolfgang repo as the implementation reference for:

- Firebase auth flow
- Cloud Run service layout
- Terraform module structure
- frontend hosting/deploy conventions
- Secret Manager env injection

Treat `converge.zone` as the platform/runtime repo that should align to those
patterns where they make sense.

Key files:

- `scripts/dev-up.sh`
- `scripts/dev-down.sh`
- `scripts/smoke-test.sh`
- `scripts/deploy-cloud-run.sh`
- `infra/environments/prod/converge-runtime/main.tf`
- `infra/modules/cloud-run-service/main.tf`
- `infra/modules/artifact-registry/main.tf`
- `Dockerfile`
- `compose.yaml`
- `docs/deployment/QUICKSTART.md`
- `docs/deployment/TERRAFORM_GCP.md`
- `docs/deployment/GPU_WORKERS.md`

## Verified Facts

These were verified during implementation, not just inferred:

- `converge-runtime` builds in baseline mode.
- `converge-runtime` also builds with `--features gcp,auth,firebase`.
- `converge-llm-server` builds with `--features server`.
- Native startup via `scripts/dev-up.sh native` can bring up the runtime.
- The runtime responds on `/health` once started.

## Important Fixes Already Made

### 1. Local startup no longer depends on Swagger UI downloads

`crates/runtime/src/http.rs` now exposes raw OpenAPI JSON instead of relying on
`utoipa-swagger-ui` downloading assets during build.

Reason:

- local/sandboxed builds were failing on external network access

### 2. Axum route syntax was updated

Old `:param` route syntax caused runtime panics on startup with current Axum.
Those routes were converted to `{param}`.

Files touched:

- `crates/runtime/src/handlers.rs`
- `crates/runtime/src/pilot.rs`

### 3. Native launcher now detaches correctly

`scripts/dev-up.sh` uses `nohup` so the runtime survives after the wrapper exits.

### 4. GCP/Firebase feature path was repaired

The runtime had undeclared feature references and a missing `rand` dependency in
the GCP-backed DB path.

Files touched:

- `crates/runtime/Cargo.toml`
- `crates/runtime/src/main.rs`

### 5. Rustls provider needed explicit install for GCP path

Firestore/rustls startup required:

- `rustls::crypto::aws_lc_rs::default_provider().install_default()`

Without that, the Google-first runtime path could compile but panic at startup.

## Current Defaults

### Runtime feature defaults used by launcher/build scripts

The deployment scripts assume:

- `gcp,auth,firebase`

This is set through:

- `.env.example`
- `scripts/dev-up.sh`
- `Dockerfile`
- `compose.yaml`

### Provider env catalog

There is now a provider-focused env template at:

- `crates/provider/.env.example`

This includes the providers and env keys currently represented in code and the
model catalog, including:

- OpenAI
- Anthropic
- Gemini
- Mistral
- Perplexity
- OpenRouter
- Grok
- DeepSeek
- Qwen
- Kimi
- MinMax
- Zhipu
- Baidu
- Cohere
- AI21
- Groq
- Together
- Fireworks
- Azure OpenAI
- Brave
- Hugging Face
- DashScope
- patent/research sources

## Wolfgang Alignment Notes

Important concrete patterns observed in `wolfgang-app`:

- frontend: Firebase SDK in `apps/web/src/lib/firebase.ts`
- frontend auth state: `apps/web/src/lib/stores/auth.svelte.ts`
- backend auth middleware: `backend/src/http/auth.rs`
- backend route protection through axum middleware layers
- infra root: `infra/environments/prod/wolfgang-bot/main.tf`
- reusable Cloud Run module: `infra/modules/cloud-run-service/main.tf`
- Firebase Hosting deploy config: `deploy/frontend/firebase.json`
- operational commands centralized in `Justfile`

This means the next deployment work in `converge.zone` should bias toward:

- keeping Terraform as the deployment source of truth rather than growing ad hoc shell scripts
- copying the frontend Firebase bootstrapping pattern rather than inventing a new one
- copying Secret Manager-backed env injection for Cloud Run
- adopting the same local dev toggle pattern (`DISABLE_AUTH=true`) only as a
  clearly dev-only escape hatch

## Current Hosted Infra Posture

Hosted deployment now has a first-pass Wolfgang-style layout:

- Terraform environment:
  - `infra/environments/prod/converge-runtime`
- Terraform modules:
  - `infra/modules/artifact-registry`
  - `infra/modules/cloud-run-service`
- `just` commands for:
  - remote state bucket bootstrap
  - `terraform init/plan/apply/output`
  - Cloud Build image publishing
  - Secret Manager secret creation and file-based version upload

The current intended hosted flow is:

1. create the GCS state bucket
2. run `just infra-init`
3. create required Secret Manager secrets
4. run `just infra-apply`
5. build and push an image with `just cloud-build <tag>`
6. roll the Cloud Run service with `just deploy-runtime <tag>`

## Current HTTP Auth Posture

`converge-runtime` now follows the Wolfgang-style backend pattern more closely:

- public routes stay public:
  - `/health`
  - `/ready`
  - `/metrics` when enabled
  - `/api-docs/openapi.json`
- protected HTTP routes are wrapped in shared Axum auth middleware
- the middleware validates Firebase bearer tokens when built with
  `gcp,auth,firebase`
- pilot SSE routes can also accept `?token=...` to support browser EventSource
  usage
- local development can bypass auth with `DISABLE_AUTH=true`

Key files:

- `crates/runtime/src/http_auth.rs`
- `crates/runtime/src/http.rs`
- `crates/runtime/src/handlers.rs`
- `crates/runtime/src/pilot.rs`

## GPU Worker Preparation

Scaffolding now exists for:

- `deploy/gpu/cloudrun/`
- `deploy/gpu/runpod/`
- `deploy/gpu/modal/`

Also important:

- `crates/llm/src/bin/server.rs` was updated so the LLM server is no longer
  hard-wired to CPU `NdArray`; it can resolve backend type from compiled
  features (`cuda`, `wgpu`, fallback CPU).

## Sharp Edges / Known Gaps

### 1. Firestore is optional in behavior but default in hosted posture

With `gcp` enabled, startup attempts DB initialization. In local mode it may
fall back gracefully, but this path should be tested with:

- no emulator
- Firestore emulator
- real GCP credentials

### 2. Terraform environment is intentionally minimal

The new Terraform layout covers Artifact Registry, Cloud Run, service account,
Secret Manager bindings, and Firestore IAM, but it does not yet include:

- Firebase Hosting
- load balancer / custom domains
- ingress policy
- private/internal auth between services
- production CORS policy
- separate environments beyond `prod`

### 3. GPU worker scaffolding is prepared, not production-complete

The templates exist, but they still need:

- actual model artifact strategy
- secure service-to-service auth
- runtime-to-worker call path
- deployment test on real GPU infrastructure

### 4. Runtime still has lingering feature-flag cleanup work

The runtime had many `cfg(feature = "...")` references without matching feature
declarations. The main missing ones were added, but warnings remain for:

- `sentry`

That can be cleaned up later by either declaring the feature or removing stale
gates.

## Best Next Steps

If continuing deployment work, the highest-value next items are:

1. Align frontend Firebase config and hosting conventions with Wolfgang.
2. Add load balancer / domain / multi-env Terraform modules as needed.
3. Decide the runtime-to-GPU-worker protocol and implement one real caller path.
4. Tighten production CORS and route-level authorization policy on top of the
   new auth middleware.
5. Decide whether to retire `scripts/deploy-cloud-run.sh` or keep it as a
   documented fallback only.

## Good Commands To Resume With

```bash
just dev-up
just smoke-test
just dev-down

cargo check -p converge-runtime --features gcp,auth,firebase
cargo check -p converge-llm --bin converge-llm-server --features server
```

## Practical Session Summary

If picking this up later, assume:

- deployment work is in progress, not finished
- Google Cloud Run + Firebase is the intended default path
- local startup exists and is usable
- GPU worker support is scaffolded but not yet integrated end to end
