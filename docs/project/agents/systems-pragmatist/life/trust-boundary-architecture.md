# Trust Boundary Architecture

The ProposedFact → Fact trust boundary has two enforcement layers:

## Layer 1: ValidationAgent (Runtime)
- Location: `converge-core/src/validation.rs`
- Reads from `ContextKey::Proposals`, validates, promotes or rejects
- Configurable: confidence threshold, content length, forbidden terms, provenance requirement
- Rejections emit audit records to `ContextKey::Signals`
- Default confidence threshold: 0.5

## Layer 2: PromotionGate (Compile-time)
- Location: `converge-core/src/gates/promotion.rs`
- Type-state pattern: `Proposal<Draft>` → `Proposal<Validated>` → `Fact`
- `Proposal<Validated>` constructor is `pub(crate)` — external crates cannot create
- `ValidationReport` has private token field — cannot be forged
- `ValidatedProposal` bundles proposal + report — cannot be separated

## Key Source Files
- `converge-traits/src/fact.rs` — Fact and ProposedFact types
- `converge-core/src/validation.rs` — ValidationAgent
- `converge-core/src/gates/promotion.rs` — PromotionGate
- `converge-core/src/gates/boundary.rs` — AuthorityGrant, constitutional types
- `converge-core/src/kernel_boundary.rs` — KernelProposal, TraceLink
- `converge-core/examples/07_proposals.rs` — Usage examples

## Open Questions
- ValidationAgent required check auto-passes (line 182 promotion.rs: "for now, mark as passed") — real validation policy enforcement pending REF-8
- How does confidence threshold interact with domain-specific validation needs?
