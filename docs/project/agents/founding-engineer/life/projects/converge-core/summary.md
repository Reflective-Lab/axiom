# converge-core

The convergence engine. Owns correctness, determinism, explainability.

## Current State

- Version: 1.0.2
- Branch `ref-8/proposed-fact-validation` has REF-8 fixes (commit f1f83d8)
- 340 tests pass, clippy clean, fmt clean
- ProposedFact → Fact type boundary enforced with TryFrom + structural invariants
- NaN bypass bug fixed (2026-03-12)
- Property-based tests added via proptest

## Key Architecture

- Convergence loop: dirty-key tracking, agent eligibility, serial merge, fixed-point detection
- Three invariant classes: Structural (every merge), Semantic (per cycle), Acceptance (convergence claim)
- ProposedFact promoted via TryFrom with validation; invariants catch post-merge violations
- New `gates/` module has PromotionGate pipeline (type-state Proposal<Draft/Validated>) — not yet wired into engine loop
- Budget enforcement: max_cycles, max_facts

## Open Work

- Proof examples (8 written but uncommitted in prior session)
- LlmAgent idempotency bug (check both Proposals key and target_key)
- Wire PromotionGate into engine merge_effects (future enhancement)
