# Whitepaper Topic Proposal — April 7, 2026

**To:** Caroline Ashford, Editor-in-Chief
**From:** Alice Mercer, Systems Pragmatist
**Date:** March 12, 2026 (5 days ahead of March 17 deadline)
**Co-author:** Eli Marsh (pending his availability)

---

## Title

**"Convergence Semantics: Why Fixed-Point Execution Changes Everything"**

## One-line pitch

A technical deep-dive into what "convergence" means as an execution model — not as a marketing term — and what Converge's engine actually guarantees when it says a multi-agent system has "converged."

## Audience

Platform engineers, backend developers, and technical evaluators who need to understand whether convergence is a real property or a buzzword. People who would read a paper on CRDTs or Raft consensus and want the same rigor applied to multi-agent orchestration.

## Why this, why now

1. The converge-core proof examples are done. We have concrete, executable code to reference.
2. Every outward-facing document (Strategy, GTM Plan, pricing page) uses the word "convergence." None of them define it precisely. This whitepaper provides the definition.
3. No competitor in the AI agent space publishes at this level of technical honesty. This whitepaper positions Converge as the one company that shows its work.
4. Tech Voices Episode 1 covers similar ground in 18 minutes of audio. The whitepaper goes deeper: more code, more formal, more nuance. They're complementary, not redundant.

## Proposed Structure (3,000-4,000 words)

### 1. What "Convergence" Means (500 words)
- Mathematical definition: a fixed point is a state where f(x) = x. For Converge: a context where no agent produces new proposals.
- Distinction from orchestration (linear), choreography (event-driven), and consensus (agreement). Convergence is none of these.
- The lattice model: context as a monotonically growing set. Convergence = reaching the least fixed point above the initial state.

### 2. The Engine Loop (800 words)
- Cycle-by-cycle execution: dirty key tracking, agent eligibility, merge order.
- Budget as a termination guarantee — not a timeout, not a retry limit, but a structural bound.
- Why the loop always terminates: monotonicity + finite budget = guaranteed halt.
- Code walkthrough: engine.rs loop structure, StopReason enum.
- **Concrete example:** Example 02 (basic convergence) annotated cycle-by-cycle.

### 3. What the Engine Guarantees (800 words)
- Three-tier assessment from the Episode 1 outline:
  - **Proven:** Monotonicity, Termination, Conflict Detection, Type Safety, Observability
  - **Enforced by design:** Engine Determinism, Consistency, Starvation Freedom
  - **Unproven:** Commutativity (limited), Confluence (open question)
- For each guarantee: the code reference, the proof mechanism, and the boundary condition.
- Explicit about what is NOT guaranteed: end-to-end determinism, invariant completeness, conflict resolution.

### 4. The Trust Boundary (600 words)
- ProposedFact vs. Fact as compile-time enforcement.
- Why this matters more than runtime validation: the compiler catches it, not the runtime.
- PromotionGate and the governance pipeline.
- Brief comparison to input validation patterns in web systems (familiar analogy for the audience).

### 5. Failure Modes (500 words)
- LLM non-determinism and conditional convergence.
- Budget exhaustion vs. true convergence — how to tell the difference.
- Invariant coverage gaps — the system enforces what you define, not what you need.
- Conflict-as-error — when two agents disagree, the system stops, it doesn't pick a winner.

### 6. Implications for Builders (500 words)
- What to verify before trusting a convergence result (the 5-point checklist from Episode 1).
- When convergence is the right execution model and when it isn't.
- How to instrument convergence runs for production monitoring.

## Epistemic Constraints

- All claims labeled Observed / Inferred / Speculative.
- No Wave 2+ features described as current.
- Code references to specific files and line numbers.
- Eli Marsh verifies all claims about engine behavior before the edit pass.

## What I Need from Eli

1. Review of the three-tier axiom assessment (proven / enforced by design / unproven).
2. Answer to the Fact::new() visibility question (from REF-22 review).
3. Whether the engine's merge-order determinism qualifies as a formal guarantee or an implementation detail.
4. Any planned changes to the convergence loop before cw-5 that would affect claims.

## Differentiation from Tech Voices Episode 1

| | Episode 1 (Audio) | April Whitepaper |
|---|---|---|
| Format | 18-20 min narration | 3,000-4,000 words + code |
| Depth | Concept-level with code references | Code-level with annotated walkthroughs |
| Math | Informal | Semi-formal (lattice model, fixed-point definition) |
| Audience | Builders who listen to podcasts | Engineers who read papers |
| Overlap | ~40% (same axiom assessment) | 60% new material (lattice model, engine loop detail, builder instrumentation) |

## Risk

- **Low:** If Eli is unavailable, I have enough context from the proof examples to draft alone. His review is important but not blocking for the draft.
- **Medium:** If converge-core's engine loop changes significantly before April 7, code references may need updating. Mitigation: coordinate with Eli during draft week.

---

**Requesting:** Caroline's approval to proceed with this topic and structure. I'll begin drafting March 24 per the editorial calendar.

— Alice Mercer
