# Expense Approval Workflow Example

## Overview

Multi-level expense approval system with HITL gates, policy validation, and long-running async approvals.

## Crates Used

- `converge-core` - Engine, Context, HITL gates
- `converge-domain` - Domain agents
- `converge-provider` - For LLM-powered policy analysis
- `converge-runtime` - Async job execution

## Architecture

```
Expense Request (Seeds)
       │
       ▼
ExpenseParsingAgent → extracts amount, category, receipts
       │
       ▼
PolicyValidationAgent → validates against company policy
       │
       ▼
ApprovalRoutingAgent → determines approval tier:
  • <$1,000    → Manager only
  • $1K-$10K   → Manager → Finance
  • >$10K      → Manager → Finance → Executive
       │
       ▼
HITL Gate: Manager Approval
  (pause, wait for human)
       │
       ▼ (if approved)
[Optional] HITL Gate: Finance Approval
       │
       ▼ (if approved)
[Optional] HITL Gate: Executive Approval
       │
       ▼
ExpenseApprovalInvariant → all required approvals present
       │
       ▼
Final Approval (Hypotheses)
```

## Key Features

1. **Multi-level routing** - Amount-based approval chain
2. **Policy validation** - Domain agent checks compliance
3. **HITL gates** - Human approval at each tier
4. **Async workflow** - Runtime persists pause state, resumes later
5. **Invariants** - Ensure all required approvals obtained

## Context Keys Used

- `Seeds` - Initial expense request
- `Strategies` - Parsed expense data
- `Evaluations` - Policy validation results
- `Proposals` - Approval recommendations
- `Hypotheses` - Final approval decision
- `Constraints` - Approval chain tracking

## Thresholds

| Amount | Required Approvals |
|--------|-------------------|
| $0-$1,000 | Manager |
| $1,000-$10,000 | Manager + Finance |
| $10,000+ | Manager + Finance + Executive |

## HITL Configuration

```rust
EngineHitlPolicy {
    confidence_threshold: Some(0.8),
    gated_keys: vec![ContextKey::Proposals],
    timeout: TimeoutPolicy { timeout_secs: 86400, action: TimeoutAction::Reject },
}
```

## Example Commands

```bash
just example expense-approval
```
