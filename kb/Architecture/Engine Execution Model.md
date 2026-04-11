---
tags: [architecture]
---
# Engine Execution Model

The engine runs an 8-phase cycle until convergence or termination.

## The 8 Phases

### 1. Initialize
Context from [[Concepts/Root Intent|RootIntent]]. Seeds populated, budgets set, invariants registered.

### 2. Eligibility
For each agent: check `dependencies()` against changed keys, then call `accepts()`. Pure, deterministic.

### 3. Parallel Execution
All eligible agents execute concurrently via Rayon. Agents read context immutably. No mutation, no coordination between agents.

### 4. Effect Buffering
Each agent returns an `AgentEffect`. The engine collects all effects without applying them.

### 5. Serialized Merge
Effects merge **one agent at a time**, in **name-sorted order**. This is the determinism guarantee ([[Philosophy/Nine Axioms#6. Transparent Determinism|Axiom 6]]). Same agents, same context, same merge order, same result.

### 6. Pruning
Dominated facts and irrelevant agents removed. Structural [[Concepts/Invariants|invariants]] checked on every merge.

### 7. Convergence Detection
```
Context[n+1] == Context[n]
AND no new facts
AND no new proposals
AND no state change
```
If true, check acceptance invariants. If those pass, the run converged.

### 8. Termination
Stop if:
- Fixed point reached (convergence)
- Budget exhausted (cycles, facts, tokens)
- Invariant violation
- Human intervention required ([[Concepts/HITL Gates]])

## Determinism Guarantees

1. Agent eligibility is pure — same context, same decision
2. Agents execute in parallel but merge serially in name-sorted order
3. Conflict detection is deterministic
4. No randomness, no implicit retries, no shadow work ([[Philosophy/Nine Axioms#8. No Hidden Work|Axiom 8]])

## Parallelism Model

- Agents execute in parallel (Rayon)
- Agents read context concurrently (immutable borrows)
- Merge happens serially (one agent's effect at a time)

Parallel execution, serial commitment. This is how Converge runs agents in parallel while guaranteeing convergence.

See also: [[Philosophy/Convergence Explained]], [[Concepts/Agents]], [[Concepts/Invariants]]
