# Monthly Tech Whitepaper — Editorial Calendar

**Cadence:** First Tuesday of each month
**Owner:** Alice Mercer (draft), Caroline Ashford (edit), Eli Marsh / Kira Novak (technical input)
**Distribution:** converge.zone, Kenneth Pernyer LinkedIn post same-day PM

---

## Process (per issue)

| Week | Milestone | Owner |
|------|-----------|-------|
| T-3 weeks | Topic proposal → Caroline for approval | Alice |
| T-2 weeks | Draft with technical input from Eli or Kira | Alice + Eli/Kira |
| T-1 week | Caroline edits; Bob creates supporting demo/benchmark if applicable | Caroline + Bob |
| T-0 (pub day) | Jules publishes to converge.zone; Caroline final review | Jules + Caroline |
| T-0 PM | Notify Kenneth for LinkedIn post | Alice |

---

## 2026 Whitepaper Schedule

### April 7, 2026 — "Convergence Semantics: Why Fixed-Point Execution Changes Everything"

**Co-author:** Eli Marsh (converge-core)
**Deadlines:**
- March 17: Topic proposal to Caroline ← **5 days from now**
- March 24: Draft complete with Eli's input
- March 31: Caroline edit pass; Bob benchmark (optional)
- April 7: Publish

**Scope:** What fixed-point convergence means vs. traditional orchestration. The engine loop. Dirty key tracking. Budget termination. Cycle-by-cycle observability. Concrete examples from converge-core 01-04.

**Epistemic constraint:** Only claims backed by converge-core examples. No Wave 2+ features.

### May 5, 2026 — "The Type Boundary: How ProposedFact vs Fact Makes AI Trustworthy"

**Co-author:** Eli Marsh (converge-core, converge-traits)
**Deadlines:**
- April 14: Topic proposal to Caroline
- April 21: Draft complete
- April 28: Edit pass
- May 5: Publish

**Scope:** The compile-time trust boundary. TryFrom validation. PromotionGate and type-state pattern. Governance pipeline. How the type system prevents LLM hallucinations from becoming trusted facts.

**Note:** This overlaps with REF-23 (Signals article on trust boundary). The whitepaper should be deeper and more code-heavy. The Signals article is the accessible version.

### June 2, 2026 — "From Spec to WASM: The JTBD Compilation Pipeline"

**Co-author:** Kira Novak (converge-provider, converge-runtime)
**Deadlines:**
- May 12: Topic proposal to Caroline
- May 19: Draft complete
- May 26: Edit pass
- June 2: Publish

**Scope:** How a Jobs-to-be-Done specification compiles to a WASM module. The provider trait boundary. Runtime isolation. What the JTBD compiler guarantees vs. what it delegates.

**Dependency:** Requires converge-tool (Wave 3) and converge-runtime (Wave 4) to have at least draft implementations. If these are not ready, substitute topic:
- **Fallback:** "Invariant Design Patterns for Multi-Agent Systems" — using converge-core's three invariant classes (structural, semantic, acceptance) with production examples.

---

## Standing Rules

1. No whitepaper publishes without Caroline's editorial approval.
2. Every system claim must be labeled: Observed / Inferred / Speculative.
3. No Wave 2+ features described as current capabilities. If referencing future work, use future tense and label as Speculative.
4. Technical claims verified by Eli (core/traits) or Kira (provider/runtime) before Caroline's edit pass.
5. Kenneth's LinkedIn post is same-day, but Alice tags him — do not assume he's monitoring the publish.
