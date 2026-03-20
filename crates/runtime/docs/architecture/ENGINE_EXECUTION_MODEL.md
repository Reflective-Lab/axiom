# Converge — Engine Execution Model

This document describes **how the Converge engine executes agents**,
with a focus on determinism, parallelism, and convergence.

---

## 1. High-Level Loop

At a high level, execution looks like:

```
initialize context from RootIntent

repeat
  determine eligible agents
  execute eligible agents
  collect effects
  merge effects into context
  apply pruning rules
until convergence or termination
```

---

## 2. Eligibility Phase

Each agent declares:

```rust
fn accepts(&self, ctx: &Context) -> bool;
```

Eligibility is:
- pure
- side-effect free
- deterministic

Only eligible agents may run.

---

## 3. Execution Phase

Eligible agents execute.

Important properties:
- Agents may execute in parallel
- Agents may read context concurrently
- Agents may call tools concurrently

Agents may NOT:
- mutate context
- call other agents
- affect scheduling

---

## 4. Effect Buffering

Each agent produces an `AgentEffect`:

```
AgentEffect
├─ Facts
├─ Intents
├─ Evaluations
├─ Traces
```

Effects are buffered.
No context mutation happens here.

---

## 5. Merge Phase (Serialized Commit)

The engine merges effects **one agent at a time**:

- deterministic order
- conflict detection
- invariant checking

This is the only point where `&mut Context` exists.

This gives:
- strong consistency
- reproducibility
- clear causality

---

## 6. Pruning Phase

After merge:
- dominated facts are dropped
- irrelevant agents are skipped
- exhausted branches are removed

Pruning strictly reduces future work.

---

## 7. Convergence Detection

Convergence is detected when:

```
Contextₙ₊₁ == Contextₙ
```

Meaning:
- no new facts
- no new intents
- no state change

At this point, execution halts.

---

## 8. Termination Conditions

Execution terminates when:
- convergence is reached
- budgets are exhausted
- invariants fail

Termination is explicit and explainable.

---

## 9. Mental Model

Think of the engine as:

- a database transaction coordinator
- a constraint solver
- a fixed-point computation

Not as:
- an event loop
- a message broker
- an actor scheduler

---

## One-sentence takeaway

> Converge runs agents in parallel, but commits knowledge serially to guarantee convergence.
