# The Trust Boundary: Why ProposedFact ≠ Fact

*Signals — Converge Architecture Series*
*Author: Alice Mercer, Systems Pragmatist*
*Status: APPROVED — editorial, GTM, and technical reviews complete. Publish: Thursday March 20, 2026.*

---

Every system that integrates LLM outputs must answer one question: when does a suggestion become a truth? Most systems answer this question by not answering it. The LLM returns a string, the string goes into a database, and now it's real. Converge answers it with a type boundary.

## LLM Outputs Are Untrusted Inputs

**[Observed]** An LLM does not produce facts. It produces plausible text conditioned on a prompt. The distinction matters because plausible text can be wrong, hallucinated, or adversarially injected via prompt manipulation.

In most AI orchestration frameworks, the LLM's response is a string. It gets parsed, maybe validated against a schema, and then stored alongside human-authored data with no distinction in authority. Once stored, downstream consumers have no way to know whether a piece of context came from a human expert, a deterministic calculation, or an LLM that was 30% confident.

This is an input validation failure. The same kind web developers learned to fix twenty years ago.

**[Observed]** Converge treats LLM outputs the way a web application should treat user input: as untrusted data that must pass through a validation boundary before it can influence system state.

## The Type Boundary

The core mechanism is two separate Rust types. Not a flag on a struct. Not a `validated: bool` field. Two types with no implicit conversion between them.

```rust
// converge-traits/src/fact.rs

/// A validated, authoritative assertion in the context.
pub struct Fact {
    pub id: String,
    pub key: ContextKey,
    pub content: String,
}

/// An unvalidated suggestion from a non-authoritative source.
pub struct ProposedFact {
    pub id: String,
    pub target_key: ContextKey,
    pub content: String,
    pub source_agent: String,
}
```

**[Observed]** `ProposedFact` is not `Fact`. The compiler enforces this. You cannot pass a `ProposedFact` where a `Fact` is expected. You cannot accidentally assign one to the other. There is no `impl From<ProposedFact> for Fact`. The only promotion path is through explicit validation.

This is a deliberate design choice documented in the source:

> *"This is the most important design decision in Converge: LLMs suggest, the engine validates. `ProposedFact` is not `Fact`. There is no implicit conversion between them."*
> — `converge-traits/src/fact.rs`, module documentation

Why types instead of flags? Because a `validated: bool` field can be set to `true` by anyone. A type boundary cannot be crossed without calling the right function. The compiler is the auditor.

## What Happens at the Boundary

**[Observed]** The `ValidationAgent` is the gateway. It reads from `ContextKey::Proposals` (where `ProposedFact` entries live), applies validation checks, and either promotes the proposal to a `Fact` in the target context key or emits a rejection record.

```rust
// converge-core/src/validation.rs

pub enum ValidationResult {
    Accepted(Fact),
    Rejected { proposal_id: String, reason: String },
}
```

Validation checks include:

- **Confidence threshold**: Proposals below a configurable minimum are rejected. If the LLM reports 30% confidence and the threshold is 70%, the proposal does not become a fact.
- **Content validity**: Empty or whitespace-only content is rejected.
- **Provenance requirement**: Proposals must declare their source (which model, which prompt hash). No anonymous suggestions.
- **Forbidden terms**: Configurable blocklist prevents overstatement. If an LLM claims something is "guaranteed," the validator catches it. (Yes, this is ironic given my editorial principles.)
- **Content length**: Bounds checking prevents unbounded outputs from consuming context space.

**[Observed]** Rejections are not silent. They produce audit records in `ContextKey::Signals`:

```rust
Fact {
    key: ContextKey::Signals,
    id: format!("validation:rejected:{proposal_id}"),
    content: format!("Proposal '{proposal_id}' rejected: {reason}"),
}
```

Every rejection is traceable. You can reconstruct why a proposal was rejected, when, and by which validation policy.

## The Promotion Gate: Defense in Depth

**[Observed]** Beyond the `ValidationAgent`, Converge has a second enforcement layer: the `PromotionGate` in `converge-core`. This implements the principle through Rust's type-state pattern.

```rust
// converge-core/src/gates/promotion.rs

// A proposal starts as Draft
let draft: Proposal<Draft> = /* from agent */;

// Validation produces a ValidatedProposal (bundle of proposal + report)
let validated: ValidatedProposal = gate.validate_proposal(draft, &context)?;

// Only ValidatedProposal can be promoted to Fact
let fact: Fact = gate.promote_to_fact(validated, approver, evidence, trace)?;
```

Three properties make this hard to subvert:

1. **No bypass path.** `promote_to_fact()` requires a `ValidatedProposal`. The only way to get one is through `validate_proposal()`. There is no constructor.
2. **No forgery.** `ValidationReport` has a private token field. External code cannot construct one. The report can only be created by the gate itself.
3. **No separation.** `ValidatedProposal` bundles the proposal and its validation report together. You cannot present a report from proposal A to promote proposal B — the gate checks that the report's proposal ID matches.

**[Observed]** The `Proposal<Validated>` constructor is `pub(crate)` — visible only within `converge-core`. External crates cannot create validated proposals. The type system makes the security boundary a compile-time guarantee, not a runtime check.

## What Happens If the Boundary Is Bypassed

**[Inferred]** If someone modified the code to allow implicit `ProposedFact` → `Fact` conversion, the consequences cascade:

1. **Context corruption.** An LLM with a hallucination or an adversarial prompt injection could write arbitrary claims into the trusted context. Other agents would treat these as ground truth.
2. **Convergence on wrong answers.** Downstream agents (policy, optimization) operate on context they assume is validated. Garbage in, deterministic garbage out.
3. **Audit trail collapse.** Without the proposal → validation → promotion chain, you cannot reconstruct the provenance of a fact. Compliance and forensic replay become impossible.
4. **Invisible failure.** The system would still converge. It would still produce results. They would just be wrong. This is the worst kind of failure — the kind that looks like success.

This is why the `fact.rs` documentation states: *"Any weakening of this boundary — implicit promotion, auto-validation, or type coercion — is a correctness and security issue. Treat changes to these types as requiring security review."*

## The Web Developer Analogy

If you've built web applications, you already understand this pattern. Replace "LLM" with "user" and "ProposedFact" with "form input":

| Web Application | Converge |
|----------------|----------|
| User submits form data | LLM returns a response |
| Raw input is untrusted | `ProposedFact` is untrusted |
| Validation middleware checks input | `ValidationAgent` checks proposal |
| Valid input written to database | `Fact` written to context |
| SQL injection if validation skipped | Context corruption if validation skipped |

The difference is scope. A SQL injection corrupts one query. A bypassed trust boundary in a convergence engine corrupts the reasoning of every agent in the system for every subsequent cycle. The blast radius is the entire convergence run.

## What This Means for Production

**[Observed]** Three things practitioners should verify before trusting a Converge deployment:

1. **The ValidationAgent is registered.** Without it, proposals accumulate in `ContextKey::Proposals` but are never promoted or rejected. The system converges — on nothing useful.

2. **Validation policy matches your risk tolerance.** The default confidence threshold is 0.5. For high-stakes domains (medical, financial, legal), you probably want 0.8 or higher. "Probably" is how incidents start — measure your threshold against your domain's cost of a false positive.

3. **Rejection signals are monitored.** The validation agent emits rejection records. If rejections spike, either your LLM is degrading or your prompts have drifted. Either way, you want to know.

**[Speculative]** Future iterations may support multiple promotion paths with different validation policies — e.g., a stricter gate for financial claims vs. a lighter gate for exploratory hypotheses. The `PromotionGate` architecture already supports configurable policies per gate instance. Whether this becomes a production feature depends on real deployment feedback.

## The Contract

Converge makes one guarantee about the trust boundary, and it is unconditional within the system's design:

**A `ProposedFact` cannot become a `Fact` without passing through validation.**

This is not a runtime assertion. It is not a policy that can be disabled. It is a property of the type system. The Rust compiler enforces it at build time.

Every other guarantee in Converge depends on this one. Determinism assumes validated inputs. Monotonicity assumes facts are authoritative. Consistency assumes invariants are checked against real data. The trust boundary is the foundation. If it holds, the rest of the system's properties have a chance. If it doesn't, nothing else matters.

---

*Alice Mercer writes the Technology Voices podcast and systems content for Signals. She believes "probably" is how incidents start.*

---

## Epistemic Labels

- **[Observed]**: Verified against source code in `converge-traits/src/fact.rs`, `converge-core/src/validation.rs`, `converge-core/src/gates/promotion.rs`, and `converge-core/examples/07_proposals.rs`.
- **[Inferred]**: Logically derived from observed code structure but not tested in production.
- **[Speculative]**: Reasonable extrapolation, not supported by current implementation.

## Review Checklist

- [ ] Ava Petrov (Security): Verify security claims, especially bypass analysis
- [ ] Eli Marsh (Founding Engineer): Confirm type-state pattern description accuracy
- [x] Caroline Ashford (Editor-in-Chief): Editorial approval — approved 2026-03-12
- [ ] Code examples: Confirm they compile against current converge-core
