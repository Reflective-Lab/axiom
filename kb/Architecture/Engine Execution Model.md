---
tags: [architecture]
source: mixed
---
# Engine Execution Model

The engine runs an 8-phase cycle until convergence or termination.

## The 8 Phases

### 1. Initialize
Context from [[Concepts/Root Intent|RootIntent]]. Seeds populated, budgets set, invariants registered.

### 2. Eligibility
For each suggestor: check `dependencies()` against changed keys, then call `accepts()`. Pure, deterministic.

### 3. Execution
In `converge-core`, eligible suggestors are currently awaited sequentially. The contract is async, but executor ownership remains outside the kernel. Suggestors read context immutably. No mutation, no coordination between suggestors.

### 4. Effect Buffering
Each suggestor returns an `AgentEffect`. The engine collects all effects without applying them.

### 5. Serialized Merge
Effects merge **one suggestor at a time**, in **registration order** (`SuggestorId`). This is the current determinism guarantee ([[Philosophy/Nine Axioms#6. Transparent Determinism|Axiom 6]]). Same registration order, same context, same merge order, same result.

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

1. Suggestor eligibility is pure — same context, same decision
2. Core execution is deterministic and merge happens serially in registration order
3. Conflict detection is deterministic
4. No randomness, no implicit retries, no shadow work ([[Philosophy/Nine Axioms#8. No Hidden Work|Axiom 8]])

## Execution Model

- Core currently awaits suggestors sequentially
- The authoring contract is async, but runtimes own scheduling and executor choice
- Runtimes may execute suggestors in parallel through an external executor surface
- Merge happens serially (one suggestor's effect at a time)

Sequential core execution, serial commitment. This keeps `converge-core` deterministic while still allowing async capability calls and leaving execution strategy to runtime crates.

See also: [[Philosophy/Convergence Explained]], [[Concepts/Agents]], [[Concepts/Invariants]]
