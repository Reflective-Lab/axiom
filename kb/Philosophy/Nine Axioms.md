---
tags: [philosophy]
---
# Nine Axioms

These are non-negotiable. Every design decision, every PR, every new crate must be evaluated against these axioms. If a change violates an axiom, the change is wrong — not the axiom.

## 1. Explicit Authority

No implicit permissions. Authority is always typed and traceable. An agent cannot promote its own proposals — the engine's promotion gate is the only path from proposal to fact.

## 2. Convergence Over Control Flow

The engine runs agents in cycles until a fixed point — not until a workflow completes. Outcomes are observable through the shared context, not hidden in agent-to-agent messages or imperative pipelines.

## 3. Append-Only Truth

Facts are never mutated. Corrections are new facts. History is preserved for auditability. You can always trace what was known at any point in time.

## 4. Agents Suggest, Engine Decides

`ProposedFact` is not `Fact`. Agents emit proposals. The promotion gate validates them — checking authority, schema, confidence. Only then does a proposal become a fact in the shared context.

## 5. Safety by Construction

Invalid states are unrepresentable. Type-state patterns enforce the lifecycle: Draft -> Validated -> Promoted. The type system catches governance violations at compile time, not runtime.

## 6. Transparent Determinism

All execution is deterministic and reproducible. Agents execute in parallel, but effects merge serially in name-sorted order. The distinction between replay-eligible (LocalTrace) and audit-only (RemoteRef) provenance is explicit.

## 7. Human Authority First-Class

Human review is not a workaround — it's a first-class concept. The engine has explicit pause/approve gates for consequential decisions. `CriterionResult::Blocked` with an `approval_ref` is a valid, honest outcome.

## 8. No Hidden Work

All effects are visible in the shared context. No shadow state, no side channels, no implicit retries. Resource consumption is explicit via budgets (cycles, facts, tokens).

## 9. Scale by Intent Replication

Scaling does not compromise governance. You scale by replicating the root intent and its invariants, not by relaxing constraints. Every replica runs under the same rules as the original.

---

When reviewing a design decision, ask: *which axiom does this serve?* If the answer is none, the decision needs justification. If it violates one, it needs to be rejected.

See also: [[Philosophy/Why Converge]], [[Philosophy/What Converge Is Not]]
