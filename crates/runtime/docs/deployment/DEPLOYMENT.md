# Converge — Deployment & Operations Guide

This document provides comprehensive guidance for deploying and operating Converge in production, based on the system's architecture, testing practices, and operational characteristics.

---

## Table of Contents

1. [System Characteristics](#system-characteristics)
2. [Testing, Code Quality & Robustness](#testing-code-quality--robustness)
3. [First Deployment Setup](#first-deployment-setup)
4. [Session Isolation & Concurrency](#session-isolation--concurrency)
5. [Scaling Strategy](#scaling-strategy)
6. [Single-User vs Multi-User](#single-user-vs-multi-user)
7. [Deployment Models](#deployment-models)
8. [Infrastructure Requirements](#infrastructure-requirements)
9. [Operational Considerations](#operational-considerations)

---

## System Characteristics

### What You Can Expect

Converge provides the following guarantees and characteristics:

1. **Deterministic Execution**
   - Same input → same output (reproducible results)
   - Fixed-point convergence detection
   - Deterministic merge ordering (AgentId-based)

2. **Bounded Execution**
   - Budget limits (max cycles, max facts) guarantee termination
   - No infinite loops possible
   - Timeout protection at deployment layer

3. **Strong Consistency**
   - Serialized merge phase ensures total ordering
   - No concurrent writes to context
   - Conflict detection for conflicting facts

4. **Fault Isolation**
   - Agent failures don't corrupt context
   - Tool failures (LLMs, APIs) handled gracefully
   - Budget exhaustion is explicit, not silent

5. **Explicit Failures**
   - No silent retries or hidden recovery
   - All failures are structured and auditable
   - Diagnostic facts emitted before errors

6. **Monotonic Context**
   - Facts only added, never retracted
   - Invalidations are explicit facts
   - Context evolution is auditable

---

## Testing, Code Quality & Robustness

### Current Testing Infrastructure

**Property-Based Testing:**
- Uses `proptest` for generative property tests
- 12+ property tests covering:
  - Context fact preservation
  - Conflict detection
  - Determinism verification
  - Budget enforcement
  - Validation logic

**Integration Tests:**
- Convergence detection
- Parallel execution verification
- Conflict detection
- Validation & proposal promotion
- Observability/tracing
- Growth strategies

**Test Coverage:**
- Determinism: Same input produces same output
- Budget enforcement: Always terminates within limits
- Fact preservation: All emitted facts appear in final context
- Conflict detection: Conflicting facts are rejected
- Dirty-key tracking: Efficient change detection

### Code Quality Standards

**Enforced via Cargo.toml:**
```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
pedantic = "warn"
```

**Error Handling:**
- **NO** `unwrap()`, `expect()`, or `panic!` in production code
- Structured errors via `thiserror`
- Explicit error types (`ConvergeError` enum)
- All failures are recoverable or explicit

**Code Quality Tools:**
- `cargo fmt --all --check` (enforced)
- `cargo clippy --all-targets --all-features -- -D warnings` (enforced)
- `cargo test --all-targets` (required before merge)

### Robustness Features

**Termination Guarantees:**
- Budget limits (max cycles, max facts) prevent infinite execution
- Default budget: 100 cycles, 10,000 facts
- Configurable per job via `RootIntent`

**Error Recovery:**
- Agent failures: Isolated, don't crash runtime
- Tool failures: Graceful degradation with timeouts/retries
- Invariant violations: Hard failures with diagnostic facts
- Budget exhaustion: Explicit error, partial results preserved

**Conflict Handling:**
- Explicit conflict detection (same ID, different content)
- Conflicts return structured errors
- Diagnostic facts emitted before error
- No silent merging or overwriting

---

## First Deployment Setup

### Recommended Architecture: Cloud Run (Stateless)

For the first production deployment, we recommend **Google Cloud Run** with a stateless architecture.

```
┌─────────────────────────────────────────┐
│  Cloud Run Service (Stateless)         │
│  ┌───────────────────────────────────┐ │
│  │  Axum HTTP Server                 │ │
│  │  ┌───────────────────────────────┐ │ │
│  │  │  Session Manager             │ │ │
│  │  │  ┌──────────┐  ┌──────────┐ │ │ │
│  │  │  │ Job 1    │  │ Job 2    │ │ │ │
│  │  │  │ Context  │  │ Context  │ │ │ │
│  │  │  │ Engine   │  │ Engine   │ │ │ │
│  │  │  └──────────┘  └──────────┘ │ │ │
│  │  └───────────────────────────────┘ │ │
│  └───────────────────────────────────┘ │
└─────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────┐
│  SurrealDB (External, Optional)         │
│  - HITL state persistence               │
│  - Context snapshots                    │
│  - Audit trail                          │
└─────────────────────────────────────────┘
```

### Cloud Run Configuration

**Container:**
- Base: Distroless Rust binary
- User: Non-root
- Size: Minimal (Rust binary + dependencies)

**Resource Limits:**
```yaml
cpu: 2-4 vCPU          # For parallel agent execution
memory: 2-4 Gi          # Per job context (adjust based on fact volume)
timeout: 60-300s        # Based on job complexity
```

**Scaling Configuration:**
```yaml
min_instances: 0         # Scale to zero when idle
max_instances: 10-50      # Based on expected load
concurrency: 1            # One job per container (recommended)
                          # Or higher if jobs are very short
```

**Why Cloud Run:**
- ✅ Fits "job starts, converges, stops" model perfectly
- ✅ Automatic scaling based on request volume
- ✅ Pay-per-use (scale to zero)
- ✅ Built-in health checks
- ✅ Easy integration with Secret Manager
- ✅ Request-level isolation

### Alternative: GKE (For Long-Running HITL)

If you need persistent HITL sessions or more control:

**Use GKE when:**
- Jobs regularly exceed 60 minutes (Cloud Run limit)
- You need persistent connections
- You want more control over resource allocation
- You need custom networking

**Trade-offs:**
- More operational overhead
- Higher base cost (always-on nodes)
- More complex deployment

---

## Session Isolation & Concurrency

### Within a Single Process

**Isolation Model:**
- Each job gets its own `Context` instance
- Each job has its own `Engine` instance
- No shared mutable state between jobs
- Isolation enforced via Rust ownership

**Concurrency:**
- Multiple jobs can run concurrently via Tokio tasks
- Each job's context is independently owned
- Agents within a job execute in parallel (via `rayon`)
- Effect merge is serialized per job

**Implementation Pattern:**
```rust
// Each HTTP request spawns a new job
async fn handle_job(root_intent: RootIntent) -> Result<ConvergeResult> {
    let mut engine = Engine::new();
    // Register agents...
    let context = Context::from_root_intent(root_intent);
    engine.run(context) // Isolated execution
}
```

### Isolation Guarantees

**What is Isolated:**
- ✅ Context state (facts, dirty keys, version)
- ✅ Agent execution (parallel within job, isolated between jobs)
- ✅ Budget tracking (per job)
- ✅ Error state (failures don't propagate)

**What is Shared (Safe):**
- ✅ Agent registration (read-only after startup)
- ✅ Invariant registry (read-only)
- ✅ Tool clients (stateless, thread-safe)

**No Cross-Job Communication:**
- Jobs cannot access each other's contexts
- No message passing between jobs
- No shared mutable state

---

## Scaling Strategy

### When to Scale Up

**Scale horizontally when:**
1. **Request rate exceeds capacity**
   - Jobs are queuing or timing out
   - Response times degrading
   - CPU/memory utilization high

2. **Geographic distribution needed**
   - Latency requirements
   - Regulatory compliance (data residency)
   - Regional failover

3. **Tenant isolation required**
   - Separate deployments per tenant
   - Different security boundaries
   - Compliance requirements

**Do NOT scale when:**
- ❌ A single job is slow (optimize the job, not scale)
- ❌ You want to split one job across nodes (not supported)
- ❌ You want shared state between jobs (not supported)

### Scaling Models

#### 1. Single Process (Development/Testing)

**Characteristics:**
- Multiple jobs in one process (via Tokio tasks)
- Each job isolated by ownership
- Good for: low-to-medium load, development, testing

**Limits:**
- Single machine resources
- No geographic distribution
- Limited fault tolerance

#### 2. Multiple Processes (Production)

**Characteristics:**
- Each Cloud Run instance handles multiple jobs
- Load balancer routes requests
- Automatic scaling based on load

**Configuration:**
- Cloud Run handles scaling automatically
- Set `max_instances` based on expected peak load
- Monitor and adjust based on metrics

#### 3. Multiple Servers (Regional)

**Characteristics:**
- Deployments per region/zone
- Route jobs to nearest region
- Independent scaling per region

**Use Cases:**
- Latency optimization
- Regulatory compliance (data residency)
- Regional failover

### Scaling Like Databases/CI Systems

Converge scales like:
- **Databases**: Each query is independent, scale by running more queries
- **CI Systems**: Each build is independent, scale by running more builds

**Not like:**
- ❌ Distributed actor systems (no shared state)
- ❌ Workflow engines (no stateful orchestration)
- ❌ Message queues (no message passing)

---

## Single-User vs Multi-User

### Current Design: Single-User Per Job

**Model:**
- Each root intent = one user's request
- No shared state between users
- Each job is independent

**Isolation:**
- Jobs are isolated by design
- No cross-user context sharing
- No user authentication built-in (add at HTTP layer)

### Multi-User Support

**Adding Multi-User:**
- Multiple users can use the same server
- Isolation via job IDs (not user IDs in core)
- No cross-user context sharing
- Add authentication/authorization at HTTP layer

**Implementation:**
```rust
// Add at Axum handler layer
async fn handle_job(
    user: AuthenticatedUser,  // From auth middleware
    root_intent: RootIntent,
) -> Result<ConvergeResult> {
    // User ID can be added to root intent metadata
    let mut intent = root_intent;
    intent.metadata.user_id = user.id;
    
    // Job execution is unchanged
    engine.run(context)
}
```

### Tenant Isolation

**Options:**
1. **Runtime Instance** (strongest isolation)
   - Separate Cloud Run service per tenant
   - Complete isolation
   - Higher operational cost

2. **Namespace** (moderate isolation)
   - Add namespace to root intent
   - Filter jobs by namespace
   - Shared infrastructure

3. **Deployment** (regulatory/compliance)
   - Separate deployments per tenant
   - Geographic or regulatory boundaries
   - Independent scaling

**Recommendation:**
- Start with single-user model (one job = one user request)
- Add multi-user support via authentication middleware
- Add tenant isolation only when required

---

## Deployment Models

### Model 1: Short-Lived Jobs (Recommended)

**Pattern:**
```
Request → Job Starts → Agents Execute → Convergence → Response → Container Idle → Scale to Zero
```

**Characteristics:**
- Jobs typically complete in seconds to minutes
- HITL can extend duration (waiting for human input)
- Container can stay warm during HITL
- After convergence, container can scale to zero

**Configuration:**
```yaml
min_instances: 0        # Scale to zero
max_instances: 50        # Based on load
timeout: 300s           # 5 minutes (for HITL)
cpu: 2                  # For parallel agents
memory: 2Gi             # Per job context
concurrency: 1          # One job per container
```

**HITL Handling:**
- Use Cloud Run's request timeout (up to 60 minutes)
- Persist HITL state to SurrealDB (external)
- Resume from persisted state when human responds
- Or: Use separate long-running service for HITL jobs

**Best For:**
- Most use cases
- Cost optimization (scale to zero)
- Simple operations

### Model 2: Long-Running Service (For Persistent HITL)

**Pattern:**
```
Service Starts → Jobs Queue → Process Jobs → HITL Wait → Resume → Complete
```

**Characteristics:**
- Persistent service (min_instances: 1)
- Jobs can wait for HITL indefinitely
- State persisted in SurrealDB
- More operational overhead

**Configuration:**
```yaml
min_instances: 1        # Always running
max_instances: 10       # Based on load
timeout: 3600s          # 1 hour (Cloud Run max)
cpu: 2
memory: 4Gi
```

**Best For:**
- Frequent HITL workflows
- Long-running jobs (> 5 minutes)
- Stateful job management

### Model 3: Hybrid (Recommended for Production)

**Architecture:**
- **Short-lived service**: Normal jobs (scale to zero)
- **Long-running service**: HITL jobs (always on)
- Route based on job type or HITL flag

**Implementation:**
```rust
// Route based on root intent
if root_intent.requires_hitl {
    route_to_long_running_service(root_intent)
} else {
    route_to_short_lived_service(root_intent)
}
```

**Best For:**
- Production deployments
- Mix of job types
- Cost optimization + HITL support

---

## Infrastructure Requirements

### Required Services

**1. Compute:**
- Cloud Run (recommended) or GKE
- Container registry (Artifact Registry)

**2. Storage (Optional):**
- SurrealDB for HITL persistence
- Qdrant for vector search (if using LLM agents)

**3. Secrets:**
- Google Secret Manager (for LLM API keys)
- Or Vault (if self-hosting)

**4. Observability:**
- Cloud Logging (structured logs)
- Cloud Monitoring (metrics)
- OpenTelemetry (traces)

### Infrastructure as Code

**Terraform Structure:**
```
infra/
├── main.tf              # Cloud Run service
├── variables.tf        # Environment variables
├── outputs.tf          # Service URLs, etc.
├── secrets.tf           # Secret Manager setup
└── surreal.tf           # SurrealDB (if needed)
```

**Required Resources:**
- Cloud Run service
- Artifact Registry repository
- Service account with appropriate permissions
- Secret Manager secrets (LLM keys)
- IAM bindings

### CI/CD Pipeline

**GitHub Actions / Cloud Build:**
1. **Build:**
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets
   ```

2. **Container:**
   ```dockerfile
   # Multi-stage build
   FROM rust:1.85 as builder
   # ... build steps ...
   
   FROM gcr.io/distroless/cc-debian12
   COPY --from=builder /app/target/release/converge-runtime /
   ENTRYPOINT ["/converge-runtime"]
   ```

3. **Deploy:**
   - Push to Artifact Registry
   - Update Cloud Run service
   - Run smoke tests

---

## Operational Considerations

### Health Checks

**Required Endpoints:**
- `GET /health` - Liveness probe (service is running)
- `GET /ready` - Readiness probe (service can accept jobs)

**Implementation:**
```rust
async fn health() -> &'static str {
    "ok"
}

async fn ready() -> Result<&'static str> {
    // Check dependencies (SurrealDB, etc.)
    Ok("ready")
}
```

### Monitoring & Observability

**Required Metrics:**
- Job execution time (p50, p95, p99)
- Convergence rate (successful vs budget exhaustion)
- Agent execution counts
- Error rates by type
- Resource utilization (CPU, memory)

**Structured Logging:**
```rust
// Every job should have a correlation ID
let span = tracing::span!(
    tracing::Level::INFO,
    "job",
    job_id = %root_intent.id,
    user_id = %user.id
);
```

**Traces:**
- Root span per job
- Spans for each cycle
- Spans for agent execution
- Spans for merge phase

### Error Handling

**Error Types:**
- `BudgetExhausted` - Job hit limits (expected for some jobs)
- `InvariantViolation` - Gherkin constraint failed (critical)
- `AgentFailed` - Agent execution error (may be recoverable)
- `Conflict` - Conflicting facts detected (explicit failure)

**Error Response:**
- Return structured errors (JSON)
- Include context snapshot (if available)
- Include diagnostic facts
- Log with correlation ID

### Resource Limits

**Per-Job Limits:**
- Budget: Max cycles, max facts (from RootIntent)
- Timeout: Cloud Run request timeout
- Memory: Per-container limit

**Monitoring:**
- Alert on high error rates
- Alert on budget exhaustion patterns
- Alert on resource exhaustion
- Track convergence success rate

### Backup & Recovery

**State Persistence:**
- HITL state in SurrealDB (if enabled)
- Context snapshots (optional, for audit)
- No in-memory state to recover

**Recovery Model:**
- Jobs are stateless (can restart from RootIntent)
- HITL state can be resumed from SurrealDB
- No need for stateful recovery

### Security

**Container Security:**
- Non-root user
- Distroless base image
- Minimal attack surface
- No shell, no package manager

**Network Security:**
- Private VPC (if using GKE)
- IAM-based authentication
- Secret Manager for credentials
- No public endpoints (use Cloud Load Balancer)

**Data Security:**
- Encrypt secrets at rest
- TLS for all external communication
- No secrets in environment variables
- Audit logging for all job executions

---

## Summary

### Key Takeaways

1. **Start Simple:**
   - Cloud Run with scale-to-zero
   - Stateless jobs
   - Short-lived execution model

2. **Isolation is Built-In:**
   - Each job is independent
   - No shared mutable state
   - Rust ownership enforces isolation

3. **Scale Like Databases:**
   - Scale by running more jobs
   - Not by splitting one job
   - Horizontal scaling is straightforward

4. **Operational Simplicity:**
   - Stateless = easy to scale
   - No stateful recovery needed
   - Standard observability patterns

5. **Production Ready:**
   - Strong testing foundation
   - Robust error handling
   - Explicit failure modes
   - Deterministic execution

### Next Steps

1. **Create HTTP Server:**
   - Axum handlers for job submission
   - Health/ready endpoints
   - Authentication middleware (if multi-user)

2. **Containerization:**
   - Multi-stage Dockerfile
   - Distroless base image
   - Non-root user

3. **Infrastructure:**
   - Terraform for Cloud Run
   - Secret Manager setup
   - CI/CD pipeline

4. **Observability:**
   - Structured logging
   - Metrics export
   - Distributed tracing

5. **Documentation:**
   - API documentation (OpenAPI)
   - Runbook for common operations
   - Incident response procedures

---

## References

- [Technology Stack](../deployment/TECHNOLOGY_STACK.md)
- [Scaling Model](../04-reference-comparisons/SCALING_MODEL.md)
- [Distributed Systems Model](../04-reference-comparisons/DISTRIBUTED_SYSTEMS.md)
- [Failure Modes](../04-reference-comparisons/FAILURE_MODES.md)
- [CloudOps Guide](../06-assistant-guides/gemini-cloudops.md)

