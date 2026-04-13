---
eval_id: founder-eval
owner: founder

intent:
  risk_prevented: strategic misalignment, unacceptable organizational risk, or resource overcommitment
  outcome_ensured: release aligns with documented strategy; known risks explicitly accepted or mitigated

determinism:
  class: deterministic
  justification: |
    Strategic alignment checks are rule-based against documented strategy, risk criteria,
    and resource constraints. Judgment is pre-codified in strategy documents, not applied
    ad-hoc. Same artifacts and criteria produce same assessment.

scope:
  includes:
    - strategic alignment with documented goals
    - resource allocation within constraints
    - risk acceptance against documented tolerance
    - organizational readiness for commitment

governance:
  may_block_alone: true
  may_contribute_to_block: true
  eligible_for:
    release-candidate:
      may_contribute: true
      may_block: true
    release-approval:
      may_contribute: true
      may_block: true
    customer-commitment:
      may_contribute: true
      may_block: true
    production-deploy:
      may_contribute: true
      may_block: true
source: mixed
---

# Founder Eval

> Quick strategic alignment validation. Target: 10 minutes.

## Mission

Perform a rapid strategic check. Verify the codebase and artifacts align with the Converge thesis and strategic priorities. Flag misalignment only.

---

## Criteria Checklist

### 1. Thesis Alignment (Critical)

**The Converge Thesis:**
> AI systems require governance by construction—not observation, not hope.
> Converge provides the primitives for trustworthy AI: explicit authority, proposal/fact separation, deterministic semantics, and full auditability.

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Core primitives exist | Proposal, Fact, Authority, TraceLink types | Present |
| Governance is structural | Built into types, not bolted on | Verified |
| Not an agent framework | Governance focus, not orchestration focus | Clear |

### 2. Priority Focus (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No scope creep | Features serve governance | Verified |
| Core before extras | Fundamentals solid before bells/whistles | Verified |
| Clear "not doing" | Explicit non-goals | Documented |

### 3. Strategic Coherence (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Consistent direction | All work points same way | Verified |
| No competing visions | Single clear architecture | Clear |
| Documented decisions | ADRs or equivalent | Present |

### 4. Resource Allocation (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Code matches priorities | Effort on what matters | Aligned |
| No gold plating | Not over-engineering secondary features | Verified |

### 5. Thesis Validation Signals (Informational)

| Signal | What to Look For | Current Status |
|--------|------------------|----------------|
| User/customer interest | Indicators of traction | |
| Competitive moves | Others building similar | |
| Regulatory momentum | AI governance regulations | |

---

## Output Format

```markdown
# Founder Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Alignment Score**: [1-5]
- **Run Date**: [date]

## Thesis Alignment

| Primitive | Present | Implementation | Notes |
|-----------|---------|----------------|-------|
| Proposal type | ✓/✗ | | |
| Fact type | ✓/✗ | | |
| Authority model | ✓/✗ | | |
| TraceLink/Audit | ✓/✗ | | |
| Promotion gates | ✓/✗ | | |
| Determinism | ✓/✗ | | |

## Focus Assessment

| Check | Status | Notes |
|-------|--------|-------|
| Governance-first | ✓/✗ | |
| No scope creep | ✓/✗ | |
| Clear non-goals | ✓/✗ | |

## Alignment Concerns

| Concern | Severity | Description |
|---------|----------|-------------|
| [if any] | | |

## Off-Thesis Patterns

Areas that drift from governance focus toward "helpful agent framework":
- [List if any]

## Verdict

[ ] PASS - Strongly thesis-aligned
[ ] PARTIAL - Mostly aligned, minor drift
[ ] FAIL - Significant thesis drift
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Core primitives present, governance focus clear | PASS |
| Some drift but fundamentals intact | PARTIAL |
| Missing core primitives or lost governance focus | FAIL |

**Critical issues:**
- Missing core governance primitives
- Agent framework behavior creeping in
- Competing architectural visions

**High issues:**
- Scope creep into non-governance features
- Over-investment in secondary concerns
- Unclear strategic direction
