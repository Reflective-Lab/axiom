---
tags: [architecture, security, audit]
---
# 2026-04-11 Security Review

## Scope

Security review of the exposed Converge control surfaces with a pentest mindset:

- `converge-policy` HTTP service
- `converge-runtime` HTTP and gRPC services
- auth, delegation, logging, and default network posture

This review was run after the semantic contract hardening and before additional product work.

## Findings Summary

### Closed in this pass

1. **Unauthenticated delegation minting in `converge-policy`**
   - Before: any caller could `POST /issue-delegation` and mint signed authority.
   - Now:
     - delegation signing is disabled unless `POLICY_SIGNING_KEY_B64` is configured
     - issuance is disabled unless `POLICY_ENABLE_DELEGATION_ISSUANCE=true`
     - issuance also requires `Authorization: Bearer <POLICY_ISSUE_ADMIN_TOKEN>`
     - default bind is `127.0.0.1:8080`

2. **Weak runtime auth fallback**
   - Before: non-Firebase `auth` builds accepted any bearer token longer than 20 chars.
   - Now:
     - protected runtime routes fail closed unless JWT or Firebase auth is configured
     - JWT auth requires `JWT_SECRET` and `JWT_ISSUER`
     - optional `JWT_AUDIENCE` defaults to `converge-runtime`

3. **Query-string bearer tokens**
   - Before: `?token=...` was accepted and leaked through request logging.
   - Now:
     - only `Authorization: Bearer ...` is accepted
     - request tracing logs the path, not the full URI

4. **Default-open runtime posture**
   - Before: default runtime bound to `0.0.0.0`, auth was not in the default feature set, and gRPC auth was unwired.
   - Now:
     - runtime default features include `auth`
     - default bind addresses are loopback:
       - HTTP: `127.0.0.1:8080`
       - gRPC: `127.0.0.1:50051`
     - gRPC methods authenticate through the same bearer-token contract as HTTP
     - `GetCapabilities` remains the only public gRPC method

5. **Overly permissive browser surface**
   - Before: runtime used `CorsLayer::permissive()`.
   - Now:
     - permissive CORS was removed
     - browser access is same-origin by default until an explicit policy is added

6. **Config / secret leakage in startup logs**
   - Before: runtime logged the full config struct.
   - Now:
     - startup logs emit non-secret operational fields only

7. **Request-size enforcement gap**
   - Before: `HttpConfig.max_body_size` existed but was not enforced.
   - Now:
     - HTTP server applies `RequestBodyLimitLayer`

## Regression Gate

Run this before merging changes that touch policy, runtime, auth, transport, or public control surfaces:

```bash
just security-gate
```

Current gate contents:

```bash
cargo check --workspace
cargo test -p converge-policy
cargo test -p converge-runtime --lib
cargo test -p converge-pack --test compile_fail
cargo test -p converge-core --test compile_fail --test truth_pipeline --test negative --test properties
cargo test -p converge-client --test messages
```

## Remaining Follow-Up Review Areas

These were not the blocking issues in this pass, but they still deserve focused review:

- WASM module trust and activation policy under real multi-tenant deployment
- secret-backend usage and production secret-loading paths
- transport hardening for explicit external deployments (TLS and mTLS operational guidance)
- dependency / supply-chain review (`cargo deny`, `cargo audit`, `cargo geiger`)

## Acceptance Rule

Converge must fail closed on authority and control surfaces. Development convenience does not justify a public bypass path.
