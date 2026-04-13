---
tags: [philosophy]
source: mixed
---
# Convergence Explained

Convergence is the central concept. If you understand this, everything else follows.

## The Mental Model

Imagine a room of experts. Each expert has a specialty. They all look at the same whiteboard (the context). Each cycle, every expert who has something to contribute writes a proposal on a sticky note. A moderator (the engine) reviews each note, validates it, and if it passes, pins it to the whiteboard.

The experts look at the updated whiteboard. Some have new things to say based on what was just added. They write more notes. The moderator reviews again.

Eventually, no expert has anything new to add. The whiteboard hasn't changed. That is convergence — a fixed point.

## Formal Definition

```
Convergence = Context[n+1] == Context[n]
              AND no new facts
              AND no new proposals
              AND all acceptance invariants pass
```

The engine detects this automatically. No agent declares "I'm done." The system observes that the collective state has stabilized.

## Step-by-Step Example

```
Cycle 1:
  AgentA has no dependencies → runs
  Proposes: fact-1, fact-2
  Engine validates and promotes both

Cycle 2:
  Context changed → AgentB wakes up (depends on AgentA's key)
  Proposes: fact-3
  Engine validates and promotes
  AgentA runs again — sees its own facts, has nothing new → no proposals

Cycle 3:
  Context changed → AgentC wakes up
  Proposes: fact-4
  Engine validates and promotes
  AgentB runs again — nothing new

Cycle 4:
  No context changes → no agents produce proposals
  Fixed point detected → convergence
```

## Why This Matters

**Correctness**: The system doesn't stop because someone told it to. It stops because there is genuinely nothing more to learn. Or it stops because the budget ran out and tells you honestly.

**Determinism**: Same inputs, same registration order, same result. Core execution is sequential today, and effects merge serially in registration order. No hidden interleaving in the core engine.

**Auditability**: Every fact traces back to the agent that proposed it, the cycle it was promoted in, and the confidence score it carried. You can replay exactly how the system reached its conclusion.

## Termination Guarantee

The engine guarantees termination through budgets:

- **Cycle budget**: maximum number of convergence cycles
- **Fact budget**: maximum facts per context key
- **Token budget**: maximum LLM tokens consumed

If the budget is exhausted before convergence, the run terminates with `StopReason::BudgetExhausted`. This is not a failure — it is an honest report that the system needed more resources than allocated.

## How Agents Wake Up

Each cycle, the engine:
1. Checks which context keys changed since last cycle
2. For each suggestor, calls `dependencies()` — which keys does it watch?
3. If any watched key changed, calls `accepts()` — pure predicate
4. If `accepts()` returns true, calls `execute()` — suggestor reads context, returns proposals
5. Engine promotes proposals through the governance gate
6. If no new facts were promoted → fixed point → convergence

Suggestors with no dependencies run on cycle 1. Suggestors that depend on Seeds run when Seeds changes. This is how the cascade works — without suggestors calling each other.

See also: [[Philosophy/Nine Axioms]], [[Concepts/Agents]], [[Concepts/Context and Facts]]
