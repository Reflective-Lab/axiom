# Technology Voices — Episode 1
## "What Does Convergence Actually Prove?"

**Track:** A (Technology Voices)
**Voice:** Alice Mercer
**Format:** Solo narration, 18-20 minutes
**Target audience:** Platform engineers, SREs, backend developers
**Status:** Draft — pending Caroline Ashford editorial approval

---

## Cold Open (1 min)

*Opening line:* "Every multi-agent framework says its agents 'collaborate.' Most of them mean the agents call each other in some order someone hardcoded. Converge means something different. It means the system reaches a fixed point — a state where no agent has anything new to say. That's a mathematical claim. This episode is about whether the code backs it up."

**Framing:** This is not a product pitch. It's a technical audit of what the converge-core crate actually proves, what it's designed to prove but hasn't yet, and where honest engineers should remain skeptical.

---

## Segment 1: Two Vocabularies (3 min)

**Key point:** Converge uses two overlapping but distinct vocabularies. Confusing them is how marketing claims get overstated.

**Vocabulary 1 — Design Tenets (the 9 constitutional axioms in `converge-core/src/lib.rs:52-145`):**
1. Explicit Authority
2. Convergence Over Control Flow
3. Append-Only Truth
4. Agents Suggest, Engine Decides
5. Safety by Construction
6. Transparent Determinism
7. Human Authority First-Class
8. No Hidden Work
9. Scale by Intent Replication

*[Observed — these are documented in the crate's module-level docs, lines 52-145]*

**Vocabulary 2 — Mathematical Properties (what the engine claims about its execution model):**
1. Monotonicity — context only grows
2. Determinism — same inputs → same outputs
3. Idempotency — re-running an agent on same context produces same proposals
4. Commutativity — agent order doesn't affect converged result
5. Termination — every run reaches a fixed point or exhausts budget
6. Consistency — no invariant violated in the final state
7. Starvation freedom — every agent proposes in every cycle
8. Confluence — different execution paths converge to the same fixed point
9. Observability — every state transition is logged

**Talking point:** The tenets describe *design intent*. The mathematical properties describe *system behavior*. A tenet is a contract with the developers. A property is a claim about the system that can be tested. Conflating the two is how "designed for determinism" becomes "guarantees determinism" in a slide deck.

*[Inferred — this distinction is not explicit in the codebase but follows from how the two lists are used]*

---

## Segment 2: What the Code Actually Proves (5 min)

Walk through the 8 proof examples (`converge-core/examples/01-08`) and assess what each one demonstrates.

### Proven in code — Observed:

**Monotonicity (Example 01, context.rs)**
- Context is append-only. `add_fact()` returns `Ok(false)` for duplicates — no mutation, no error.
- No `&mut` methods on `Fact`. Private fields.
- Corrections create new `CorrectionEvent` facts, not mutations.
- **Verdict: Enforced by construction.** The type system prevents fact deletion.

**Termination (Example 04, engine.rs)**
- `Budget { max_cycles, max_facts }` enforced in the engine loop.
- Budget exhaustion returns a specific `StopReason` variant — it's information, not failure.
- An `ExpandingAnalyst` agent that always has more to say is provably halted by the budget.
- **Verdict: Proven.** Every convergence run terminates. The mechanism is explicit.

**Conflict Detection (Example 07, context.rs:229-234)**
- Two agents proposing the same fact ID with different content → `ConvergeError::Conflict`.
- The system refuses ambiguity rather than silently picking a winner.
- **Verdict: Proven.** The system errors rather than accepting inconsistency.

**Compile-Time Type Safety (Examples 07, 08)**
- `ProposedFact` and `Fact` are distinct types. No implicit conversion.
- Only promotion path: `TryFrom<ProposedFact> for Fact` (validates confidence in [0.0, 1.0], non-empty content).
- Type-state pattern: `Proposal<Draft>` → `Proposal<Validated>` → `Fact`. Only `PromotionGate` creates `Validated`.
- **Verdict: Enforced by construction.** The compiler rejects invalid state transitions.

**Observability (Examples 02, 04, 08)**
- `StreamingCallback` trait emits `on_cycle_start`, `on_fact`, `on_cycle_end` events.
- `PromotionRecord` traces gate, approver, evidence, and replayability.
- `StopReason` is an exhaustive enum — no unlabeled exits.
- **Verdict: Structurally enforced.** The engine cannot converge without emitting observable events.

### Enforced by design, not formally proven — Inferred:

**Determinism (Examples 02, 05)**
- Engine merges effects in sorted order by `AgentId` (engine.rs:443). This makes the merge order deterministic.
- Example 05 shows: same seed → same convergence. Different seed → different convergence. Deterministic relative to input.
- **Critical caveat:** This is *engine-level* determinism. The engine is a pure function from (agents, context) → result. But if any agent wraps an LLM, that agent is non-deterministic (temperature, model version, prompt caching). The engine's guarantee is: "given the same agent outputs, I produce the same result." It does NOT guarantee that agents produce the same outputs.
- **Verdict: Engine determinism is proven. End-to-end determinism depends on agent purity, which is outside converge-core's scope.**

**Consistency (Example 03)**
- Three invariant classes: structural (every merge), semantic (every cycle), acceptance (at convergence).
- Invariants are checked, and violations are rejected.
- **Caveat:** Consistency is only as strong as the invariants the user defines. The engine enforces invariants — it doesn't generate them.
- **Verdict: The enforcement mechanism is proven. Coverage depends on user-defined invariants.**

**Starvation Freedom**
- All eligible agents run in each cycle. Eligibility is determined by dirty key intersection.
- No priority queue, no scheduling — if your dependencies are dirty, you run.
- **Caveat:** An agent whose dependencies are never dirtied will never become eligible. This is by design, not a bug — but calling it "starvation freedom" without qualification is misleading if the listener expects every agent to run at least once.
- **Verdict: All eligible agents run. Eligibility itself is the constraint.**

### Claimed but unproven — Speculative:

**Commutativity**
- The engine merges in deterministic order, which makes the *result* deterministic. But commutativity claims that *any* merge order would produce the same result.
- With LLM agents whose proposals depend on what's already in context (i.e., they read other agents' facts), execution order *can* affect proposals. If Agent A reads Agent B's output before proposing, the proposal differs from when Agent A proposes without B's output.
- Dirty key tracking mitigates this — agents only run when their dependencies change. But within a single cycle, multiple agents may be eligible, and their merge order is fixed (by AgentId), not arbitrary.
- **Verdict: Commutativity holds for a specific definition: the engine produces the same result regardless of AgentId ordering during merge. It does NOT hold for arbitrary interleaving of execution and observation.**

**Confluence**
- The system detects *a* fixed point. Whether multiple fixed points exist for the same input is an open question.
- Example 05 demonstrates that different seeds produce different fixed points — but that's intentional (different inputs, different outputs).
- Whether the same input can reach *different* fixed points depending on non-deterministic agent behavior is not tested.
- **Verdict: Convergence to *a* fixed point is proven. Uniqueness of that fixed point is not.**

---

## Segment 3: The Trust Boundary (4 min)

Deep dive on the ProposedFact → Fact boundary — the single most important design decision for builders.

**Code references:**
- `context.rs:82-152` — ProposedFact and Fact definitions
- `gates/promotion.rs` — PromotionGate, type-state pattern
- Example 07 — validation and rejection scenarios
- Example 08 — full governance pipeline with Actor types

**Talking points:**

1. "There are exactly two types in converge-core that matter: `ProposedFact` and `Fact`. An LLM produces the first. Only the engine — after validation — produces the second. The compiler enforces this. Not a convention. Not a config flag. The type system."

2. "A `ProposedFact` carries `confidence: f64` and `provenance: String`. A `Fact` does not. When promotion happens via `TryFrom`, the confidence and provenance are validated and then *discarded*. The Fact doesn't remember it was uncertain. That's the point — once promoted, it's trusted. The audit trail lives in the `PromotionRecord`, not in the Fact itself."

3. "Example 07 shows four scenarios: valid high-confidence proposal (promoted), valid low-confidence proposal (also promoted — the current validation doesn't reject on low confidence, only on out-of-range), invalid confidence > 1.0 (rejected), empty content (rejected). Two out of four make it through. The engine doesn't choose winners. It filters failures."

4. "The governance pipeline in Example 08 adds another layer: `Proposal<Draft>` → `Proposal<Validated>` → `Fact`. The `PromotionGate` runs named checks (`schema_valid`, `confidence_threshold`). Every `Fact` gets a `PromotionRecord` with: gate ID, approver (human/agent/system), evidence references, and a `TraceLink` — `LocalTrace` for replay-eligible, `RemoteRef` for audit-only."

**Honest assessment:** *[Inferred]* This is a strong trust boundary for the *engine*. But it doesn't prevent a bad actor from constructing a `Fact::new()` directly in application code outside the engine. The boundary is structural within converge-core's execution model, not a sandbox around untrusted code.

---

## Segment 4: Where It Breaks (3 min)

Every system has failure modes. Here are the ones builders should know about.

**1. LLM Non-Determinism (Observed)**
- The engine is deterministic. The agents are not. If an agent wraps `claude-3` and the model returns different text on retry, the converged state changes.
- The `Replayability` type and `TraceLink::LocalTrace` vs `TraceLink::RemoteRef` distinction acknowledges this. The system doesn't lie about it.
- **What to do:** Treat convergence results as *conditional* on agent outputs. Log the full agent trace. Compare across runs.

**2. Infinite Proposal Loops (Observed — Example 04)**
- An agent that always has new things to say will exhaust the budget. This is the correct behavior.
- But if your budget is too generous, you'll burn tokens before the budget stops you.
- **What to do:** Set budgets conservatively. Treat budget exhaustion as a signal, not a bug.

**3. Invariant Coverage Gaps (Inferred)**
- The engine enforces invariants you define. It cannot enforce invariants you forget.
- There's no static analysis to tell you "you should have an invariant for X."
- **What to do:** Treat invariant design as a first-class engineering task. Review invariants like you review API contracts.

**4. Conflict as Error, Not Resolution (Observed — Example 07)**
- Two agents disagreeing on the same fact ID causes a hard error. There's no conflict resolution strategy.
- This is a design choice — ambiguity is rejected, not resolved. But it means agents must coordinate implicitly through context keys, not fact IDs.
- **What to do:** Design your context key schema carefully. Use unique fact IDs per agent or per proposal path.

---

## Segment 5: What Builders Should Verify (2 min)

Before trusting a convergence result, check five things:

1. **Did it converge or exhaust?** Check `result.converged` and `StopReason`. Budget exhaustion is not convergence.
2. **What invariants were checked?** Review the `InvariantClass` breakdown: structural, semantic, acceptance. Gaps in invariant coverage are gaps in trust.
3. **How many cycles?** A 2-cycle convergence on a complex problem probably means agents didn't interact. A 50-cycle convergence probably means agents are oscillating. Both warrant investigation.
4. **Were any proposals rejected?** Silent rejection of `ProposedFact` is by design. But if your most important agent's proposals keep failing validation, you have a data quality problem.
5. **Can you replay it?** Check `PromotionRecord.trace_link`. `LocalTrace` = you can reproduce this. `RemoteRef` = you can audit it but not replay it. Know the difference.

*[Inferred — these are derived from the code structure but not documented as a verification checklist]*

---

## Close (1 min)

"Converge doesn't guarantee that your agents are smart. It guarantees that their collaboration is structured, observable, and terminates. That's a lower bar than the marketing might suggest — and a higher bar than anything else in the multi-agent space actually delivers. The axioms are real. The proofs are in the code. The gaps are documented. That's what honest infrastructure looks like."

*Sign-off:* "This is Technology Voices. I'm Alice Mercer. Probably is how incidents start."

---

## Epistemic Label Summary

| Claim | Label | Source |
|-------|-------|--------|
| 9 design tenets documented in lib.rs | Observed | converge-core/src/lib.rs:52-145 |
| Monotonicity enforced by construction | Observed | context.rs (append-only, no &mut on Fact) |
| Termination via budget | Observed | engine.rs, Example 04 |
| Conflict detection on duplicate IDs | Observed | context.rs:229-234, Example 07 |
| Compile-time ProposedFact/Fact separation | Observed | types, TryFrom impl, Example 07 |
| StreamingCallback observability | Observed | engine.rs, Example 02 |
| PromotionGate governance pipeline | Observed | gates/promotion.rs, Example 08 |
| Engine determinism (merge order by AgentId) | Observed | engine.rs:443 |
| End-to-end determinism depends on agent purity | Inferred | Follows from LLM non-determinism being outside scope |
| Consistency depends on user-defined invariants | Inferred | Engine enforces but doesn't generate invariants |
| Starvation freedom conditional on eligibility | Inferred | Dirty key tracking determines eligibility |
| Commutativity limited to merge order | Speculative | Not tested for arbitrary execution interleaving |
| Confluence (fixed-point uniqueness) | Speculative | Only one fixed point per run observed, uniqueness unproven |
| Direct Fact construction possible outside engine | Inferred | Fact::new() is public, trust boundary is structural not sandboxed |
| Design tenets vs. mathematical properties are distinct | Inferred | Implied by code structure, not explicitly stated |

---

## Production Notes

- **Code references:** 8 examples from converge-core (01-08), engine.rs, context.rs, gates/promotion.rs, types/provenance.rs, lib.rs
- **Dependencies met:** converge-core proof examples landed (status: done)
- **Review required:** Caroline Ashford (editorial), Eli Marsh (technical verification of axiom claims)
- **Open question for Eli:** Is `Fact::new()` intentionally public? If so, the trust boundary claim in Segment 3 needs a qualifier about application-level usage vs. engine-internal usage.
