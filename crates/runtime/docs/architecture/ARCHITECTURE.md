# Converge — Architecture

Converge is a **pure Rust Agent Operating System** for building correctness-first,
context-driven, multi-agent systems that **provably converge**.

Converge is not:
- a chatbot framework
- a workflow engine
- a prompt orchestration system

Converge is a runtime where:
- Context is the API
- Agents collaborate through data, not calls
- Execution proceeds until a fixed point
- Behavior is constrained by Gherkin invariants
- LLMs are tools, never authorities

---

## 1. Core Axioms

1. Context is the only shared state  
2. Agents never call each other  
3. Execution is graph-based, not linear  
4. Context evolution is monotonic  
5. Convergence is explicit and observable  
6. Correctness is verified semantically  
7. LLMs may suggest, never decide  

---

## 2. High-Level System View

```
┌──────────────────────────────────────────┐
│ Specification Layer (Gherkin)            │
│ Behavioral invariants & success criteria │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Orchestration Layer                      │
│ Execution graph & convergence engine     │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Agent Layer                              │
│ Deterministic • LLM • Solver • IO        │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Context Layer                            │
│ Typed, shared, monotonic job state       │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│ Tool Layer                               │
│ LLMs • Search • Solvers • APIs           │
└──────────────────────────────────────────┘
```

---

## 3. Root Intent

Every execution starts with a Root Intent.
It defines scope, constraints, and success conditions.

```rust
struct RootIntent {
    kind: IntentKind,
    goal: Goal,
    constraints: Constraints,
    success_criteria: SuccessCriteria,
}
```

The Root Intent defines the *universe of discourse*.
Nothing outside it may exist.

---

## 4. Context Model

Context is the working memory of a job.

```
Context
├─ Job (RootIntent)
├─ Facts
├─ State
├─ Constraints
├─ Evidence
└─ Trace
```

Rules:
- Context is append-only in meaning
- Facts are typed
- Provenance is mandatory

---

## 5. Agent Model

Agents are semantic capabilities.

```rust
trait Agent {
    fn accepts(&self, ctx: &Context) -> bool;
    fn execute(&self, ctx: &mut Context) -> AgentEffect;
}
```

Agents:
- never call other agents
- never control flow
- never decide termination

---

## 6. Agent Taxonomy

- Deterministic agents (policy, validation, aggregation)
- Retrieval agents (IO, search, DB)
- LLM agents (interpretation, explanation)
- Solver agents (optimization, planning)
- Governance agents (safety, compliance)

---

## 7. Execution & Convergence

Execution proceeds in cycles:

```
repeat
  select eligible agents
  execute agents
  merge effects into context
until context reaches fixed point
```

Convergence occurs when:

```
Contextₙ₊₁ == Contextₙ
```

---

## 8. Delegation & Aggregation

Delegation:
- agents emit sub-intents into context

Aggregation:
- explicit synthesis agents
- no implicit joins

---

## 9. Progressive Convergence

Converge supports anytime answers:

- Early convergence
- Primary convergence
- Extended convergence
- Exploratory background work

---

## 10. Knowledge vs Context

Context is ephemeral.
Knowledge is curated and persistent.

```
Knowledge DB ──query──▶ Context
```

Never the reverse.

---

## 11. Gherkin

Gherkin expresses semantic invariants.

```gherkin
Then the system converges
And no constraints are violated
```

It never drives execution.

---

## 12. Crate Structure

Converge is organized into layered crates with strict dependency discipline:

```
converge-core      Traits, engine, context (no external deps)
      ↓
converge-provider  LLM implementations (Anthropic, OpenAI, etc.)
      ↓
converge-domain    Domain agents (GrowthStrategy, Scheduling)
converge-tool      Dev tools (Gherkin validator)
      ↓
converge-runtime   HTTP/gRPC servers, TUI
```

**Key rule:** Dependencies flow downward only. Core never imports from provider.

See `docs/development/DECISIONS.md` §6 for full layering rules.

---

## 13. Summary

Converge is a Rust-based Agent OS where context is the API,
agents collaborate through a data-driven execution graph,
and correctness is enforced through convergence and invariants.
