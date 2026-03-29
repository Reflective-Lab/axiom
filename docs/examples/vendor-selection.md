# Vendor Selection & Procurement Example

## Overview

Swarm-based vendor evaluation with consensus aggregation and HITL for final sign-off.

## Crates Used

- `converge-core` - Engine, Context, HITL gates
- `converge-domain` - Domain agents
- `converge-provider` - For LLM-powered evaluation
- `converge-runtime` - Job execution

## Architecture

```
RFP Documents (Seeds)
       │
       ▼
┌──────────────────────────────────────────┐
│      Parallel Evaluator Swarm            │
├──────────────────────────────────────────┤
│ PriceEvaluatorAgent      → Evaluations   │
│ ComplianceEvaluatorAgent → Evaluations   │
│ RiskEvaluatorAgent       → Evaluations   │
│ TimelineEvaluatorAgent  → Evaluations   │
│ QualityEvaluatorAgent   → Evaluations   │
└──────────────────────────────────────────┘
       │
       ▼
VendorConsensusAgent → aggregates scores
       │
       ▼
HITL Gate: Final Approval
  (pause, wait for procurement sign-off)
       │
       ▼
Final Vendor Selection (Proposals)
```

## Key Features

1. **Swarm evaluation** - 5 parallel agents evaluate different dimensions
2. **Consensus aggregation** - Weighted score calculation
3. **Multi-criteria scoring** - Price, compliance, risk, timeline, quality
4. **HITL gate** - Human sign-off before final selection
5. **Confidence threshold** - Auto-HITL for low-confidence recommendations

## Context Keys Used

- `Seeds` - RFP/vendor data
- `Signals` - Parsed vendor information
- `Evaluations` - All agent scores (swarm output)
- `Proposals` - Final recommendation with scores
- `Hypotheses` - Approved vendor selection

## Scoring Weights

| Criterion | Weight |
|-----------|--------|
| Price | 30% |
| Compliance | 25% |
| Risk | 20% |
| Timeline | 15% |
| Quality | 10% |

## HITL Configuration

```rust
EngineHitlPolicy {
    confidence_threshold: Some(0.75),  // Auto-pause below 75%
    gated_keys: vec![ContextKey::Proposals],
    timeout: TimeoutPolicy { timeout_secs: 604800, action: TimeoutAction::Reject },
}
```

## Example Commands

```bash
just example vendor-selection
```
