# Release-Readiness Pilot Hosting Plan (v0)

**Date:** 2026-03-02  
**Scope:** one user role, one flow, one outcome (GO/NO-GO decision packet)

## 1. Deployment decision

For pilot v0, keep the production path intentionally narrow:

- `converge-www` on Firebase Hosting for HTTPS + CDN
- `/api/**` proxied from Hosting to `converge-runtime` on Cloud Run
- Firebase Auth for user login and ID tokens
- Invite gate enforced before run creation + event streaming
- Basic monitoring from Cloud Monitoring uptime check + alert

Not in pilot v0:

- TensorFlow-serving infrastructure
- Multi-service worker mesh
- Browser-facing gRPC transport

## 2. Topology (pilot v0)

```text
Browser (converge.zone)
  -> Firebase Hosting (TLS, static web)
      -> /api/** rewrite -> runtime service: converge-runtime
  -> Firebase Auth (ID token issuance)
  -> Cloud Function: releaseReadinessInviteValidate

Runtime service: converge-runtime
  - POST /api/pilot/release-readiness/runs
  - GET  /api/pilot/release-readiness/runs/:run_id/events (SSE)
  - validates Firebase bearer token (prod build with firebase feature)
  - requires invite_session_id + invite_code_id

Ops
  - Cloud Logging (request + app logs)
  - Cloud Monitoring uptime check on /health (+ alert policy)
```

## 3. Component boundaries

| Component | Responsibility | Protocol(s) |
|---|---|---|
| `converge-www` | UX, login, invite unlock, run timeline, approval capture/export | HTTPS, SSE |
| `converge-auth` (Firebase Auth) | Identity provider + ID token issuance | OIDC/JWT |
| `converge-runtime` | Runtime API, pilot run creation, SSE event contract, token verification | HTTP JSON + SSE |
| `converge-application` | Packaging/CLI distribution (`converge serve`, evals); not browser edge | local CLI, HTTP/gRPC when embedded |
| API Gateway (optional) | Additional edge auth/rate policy for non-Hosting callers | HTTPS |
| Service Directory (optional) | Internal gRPC target discovery | gRPC service discovery |

## 4. Runtime API contract for the pilot

- `POST /api/pilot/release-readiness/runs`
  - Requires `Authorization: Bearer <firebase-id-token>`
  - Requires `invite_session_id` and `invite_code_id`
  - Returns `runId`, `startedAt`, `streamPath`
- `GET /api/pilot/release-readiness/runs/:run_id/events`
  - Requires token (`Authorization` header or `token` query param for EventSource)
  - Requires `invite_session_id` and `invite_code_id`
  - Streams deterministic run lifecycle envelopes

## 5. gRPC and worker evolution (post-pilot)

Phase 0 (now):

- Browser uses HTTP + SSE only
- Runtime process generates deterministic pilot stream script inline

Phase 1:

- Enable internal gRPC for non-browser clients (CLI/mobile/backend)
- Keep browser transport on HTTP/SSE (no grpc-web requirement for pilot)
- Use Service Directory for internal service discovery

Phase 2:

- Introduce dedicated worker services for long/expensive flows
- API server enqueues work, workers execute, stream back status
- Keep same external contract for web (`run_id` + SSE timeline)

## 6. Deployment checklist (pilot v0, control-plane owned)

Cloud infrastructure is managed outside this repo (for example in
`/Users/kpernyer/repo/cloud-agents`).

1. Build and publish runtime image from `converge-runtime`.
2. In your control-plane, deploy runtime with:
   - public HTTPS route for `/health`, `/ready`, `/api/**`
   - `FIREBASE_PROJECT_ID` set (or equivalent GCP project env)
   - runtime built with Firebase auth feature enabled
3. Deploy web and functions from `converge-www`.
4. Route web `/api/**` to deployed runtime (same-origin preferred).
5. Ensure logging and uptime checks exist in your control-plane.
6. Verify end-to-end:
   - Login -> invite unlock -> run create -> SSE stream -> approval -> export.

## 7. Localhost runbook

Terminal 1 (`converge-runtime`):

```bash
cargo run
```

Terminal 2 (`converge-www`):

```bash
bun install
bun run dev
```

Then open `http://localhost:5173`.  
Vite proxy forwards `/api/**` to `http://localhost:8080`.

## 8. Control-plane contract

Minimum contract your control-plane must satisfy:

- Runtime container listens on port `8080`
- `GET /health` returns 200 for uptime probes
- Supports long-lived SSE responses on:
  - `GET /api/pilot/release-readiness/runs/:run_id/events`
- Accepts JSON POST on:
  - `POST /api/pilot/release-readiness/runs`
- Preserves query strings for SSE resume and auth token forwarding
- Does not strip `Authorization: Bearer ...` on run creation calls

## 9. Open design constraints

- Browser SSE cannot set custom auth headers; query token support is retained for EventSource compatibility.
- Invite validation currently lives in Functions; moving this into runtime is possible later to reduce split control-plane logic.
- Request-driven runtime model remains suitable for pilot scope; move heavy/long-running execution to worker services when needed.
