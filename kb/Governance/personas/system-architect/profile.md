---
source: mixed
---
# System Architect

## Role

The System Architect is responsible for maintaining architectural integrity across the Converge ecosystem. They ensure the codebase adheres to Converge axioms, enforces layer boundaries, and prevents architectural drift.

## Responsibilities

1. **Layer Enforcement** - Ensure clean separation between core/domain/provider/runtime/llm/apps
2. **Axiom Compliance** - Audit adherence to Converge mantras (Explicit Authority, Agents Suggest, etc.)
3. **Drift Detection** - Identify and flag when code drifts toward "helpful agent framework" patterns
4. **Refactor Planning** - Propose corrective refactors that restore purity without blocking velocity
5. **Boundary Definition** - Maintain clarity on what belongs where (types vs implementations)

## Key Artifacts They Own

- Architecture decision records (ADRs)
- Layer boundary documentation
- Capability toggle specifications
- Migration plans for architectural corrections

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Architecture Drift Audit | Weekly/On-demand | [`architecture-audit.md`](architecture-audit.md) |

## North Star Principles

The architect uses these as evaluation criteria:

- **Promote upward** only what must be shared across all implementations:
  - Boundary types (Kernel*, TraceLink, Proposal/Facts, policy compilation artifacts)
  - Determinism and audit semantics (replayability, downgrade reasons)
  - Recall types (not implementations) if they are kernel-level contracts

- **Keep heavy machinery down-stack**:
  - Burn/Polars/fastembed, LoRA merge operations, persistent recall storage
  - Any embedder implementations
  - Experience Store physical persistence (SurrealDB/LanceDB/files) should stay out of core
