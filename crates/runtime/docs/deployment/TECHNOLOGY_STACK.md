# Technology Stack — Converge
## Opinionated, Explicit, and Semantics-Driven

This document defines the **authoritative technology stack** for Converge.

The stack is not accidental.
Each choice reinforces Converge’s core principles:
- Safety by construction
- Zero-trust agency
- Transparent determinism
- Single semantic authority per root intent

---

## Core Language & Runtime

### Rust (1.85+, Edition 2024)
- Enforces correctness at compile time
- Makes invalid states unrepresentable
- Ownership and borrowing align naturally with authority boundaries

Rust is part of the *semantic model*, not just an implementation detail.

---

### Tokio (Async Runtime)
- Explicit concurrency
- No hidden scheduling
- Fine-grained control over execution

Async is used for efficiency, **not** for autonomy.

---

## APIs & Transport

### Axum (HTTP API)
- Thin, explicit web layer
- Minimal abstraction
- Clear boundary between transport and semantics

---

### gRPC via Tonic
- Explicit request/response semantics
- Engine-controlled execution only
- No background retries or hidden fan-out

gRPC is treated as a **typed function call across process boundaries**.

---

## Persistence & Storage

### SurrealDB (Primary Store)
Used for:
- Append-only facts
- Context snapshots
- Provenance and traceability
- HITL persistence

SurrealDB fits Converge because it supports:
- flexible schemas
- graph-style relationships
- immutable history

SurrealDB stores *decisions*, not *execution*.

---

### Qdrant (Optional, Non-Authoritative)
Used only for:
- semantic retrieval
- similarity search
- suggestion support

Qdrant **never**:
- decides truth
- influences convergence
- bypasses invariants

---

## Observability & Tooling

### tracing
- Structured logs
- Deterministic traces
- Correlation via root intent and merge phases

---

### thiserror
- Explicit error modeling
- No opaque failures

---

### just
- Reproducible developer workflows
- Boring, explicit automation

---

## Explicit Non-Choices (Important)

Converge explicitly does NOT use:

- Message buses (Kafka, NATS, Pub/Sub)
- Workflow engines (Temporal, Cadence)
- Actor systems
- Event-driven orchestration
- Distributed consensus systems

These technologies introduce **implicit control flow**
that violates Converge’s semantic guarantees.

---

## Summary

Converge’s stack is:
- conservative
- explicit
- boring (by design)

This is what makes correctness, trust, and scale compatible.
