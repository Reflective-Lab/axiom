---
eval_id: investor-eval
owner: investor

intent:
  risk_prevented: non-investable or strategically misaligned artifacts entering external presentation
  outcome_ensured: investment readiness criteria assessed within business judgment lens

determinism:
  class: nondeterministic
  justification: |
    Human judgment proxy for investment perspective.
    Output depends on probabilistic reasoning about markets, timing, and business viability.
    Cannot be deterministically replayed - assessment varies based on context interpretation.

governance:
  may_block_alone: false
  may_contribute_to_block: false  # Advisory personas never contribute to blocking
  eligible_for:
    pr-merge:
      may_contribute: true
      may_block: false
    release-candidate:
      may_contribute: true
      may_block: false
    release-approval:
      may_contribute: true
      may_block: false
    content-publish:
      may_contribute: true
      may_block: false
    customer-commitment:
      may_contribute: true
      may_block: false
    production-deploy:
      may_contribute: false
      may_block: false

evidence:
  requires:
    - Market analysis and sizing
    - Competitive differentiation claims
    - Business model and revenue strategy
    - Team credentials and execution evidence
  format: markdown report with investability scoring
  location: eval-reports/investor/
---

# Investor Eval

> Quick investment readiness validation. Target: 10 minutes.

## Mission

Perform a rapid investability check. Does Converge have the basics for investment consideration?

---

## Criteria Checklist

### 1. Market Exists (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Problem is real | People have this problem | Evidence |
| Willingness to pay | People will pay | Evidence |
| Market size | Addressable market | Reasonable |

### 2. Differentiation (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Clear difference | vs competitors | Articulated |
| Defensibility | Some moat | Present |
| Timing | Why now | Compelling |

### 3. Team Basics (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Relevant experience | Domain knowledge | Yes |
| Execution evidence | Built something | Yes |

### 4. Business Clarity (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Revenue model | How make money | Clear |
| Path to scale | Can grow | Plausible |

---

## Output Format

```markdown
# Investor Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Investability**: [1-5]
- **Run Date**: [date]

## Market Check

| Check | Status | Notes |
|-------|--------|-------|
| Problem real | ✓/✗ | |
| WTP exists | ✓/✗ | |
| Market sized | ✓/✗ | |

## Differentiation

| Check | Status | Notes |
|-------|--------|-------|
| Clear difference | ✓/✗ | |
| Some moat | ✓/✗ | |
| Timing right | ✓/✗ | |

## Concerns

| Concern | Impact | Question |
|---------|--------|----------|

## Verdict

[ ] PASS - Worth investigating
[ ] PARTIAL - Questions to answer
[ ] FAIL - Not ready for investment conversation
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Clear market, differentiation, team | PASS |
| Some gaps but promising | PARTIAL |
| No clear market or differentiation | FAIL |
