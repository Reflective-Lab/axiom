# Control-Plane Handoff (cloud-agents)

This document is the deployment handoff contract for managing Converge pilot
infrastructure from an external control-plane repo (for example
`/Users/kpernyer/repo/cloud-agents`).

## Runtime service

- Service: `converge-runtime`
- Container port: `8080`
- Required endpoints:
  - `GET /health` (200, body `ok`)
  - `GET /ready` (200 when ready)
  - `POST /api/pilot/release-readiness/runs`
  - `GET /api/pilot/release-readiness/runs/:run_id/events` (SSE)
- Required runtime env:
  - `RUST_LOG=info`
  - `FIREBASE_PROJECT_ID=<firebase-project-id>` (or equivalent project env)
- Build/runtime feature expectation:
  - include Firebase auth support in production runtime build

## Web service

- Service: `converge-www`
- Must route `/api/**` to runtime service.
- Keep same-origin routing for browser compatibility and simpler CORS.

## Functions

- Deploy `releaseReadinessInviteValidate` from `converge-www/functions`.
- Keep allowlist origins aligned with production/staging hostnames.

## Smoke test sequence

1. Open `/login` and sign in.
2. Open `/demo/release-readiness`.
3. Unlock with a valid invite code.
4. Start run and confirm timeline events stream in.
5. Submit approval decision and export packet.
