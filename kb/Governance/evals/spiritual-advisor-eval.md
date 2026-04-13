---
eval_id: spiritual-advisor-eval
owner: spiritual-advisor

intent:
  risk_prevented: dignity violations or purpose misalignment entering ethical decision-making
  outcome_ensured: human dignity and purpose alignment assessed within values-based lens

determinism:
  class: nondeterministic
  justification: |
    Human judgment proxy for values and dignity perspective.
    Output depends on ethical interpretation, dignity assessment, and purpose evaluation.
    Cannot be deterministically replayed - values judgments inherently subjective and context-dependent.

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
      may_contribute: true
      may_block: false

evidence:
  requires:
    - Purpose and mission statements
    - User treatment and dignity policies
    - Honesty and transparency practices
    - Ethical decision rationale
  format: markdown report with dignity and purpose assessment
  location: eval-reports/spiritual-advisor/
source: mixed
---

# Spiritual Advisor Eval

> Quick purpose and dignity validation. Target: 10 minutes.

## Mission

Perform a rapid check on purpose alignment and human dignity. Ensure we're building something worthy with integrity.

---

## Criteria Checklist

### 1. Purpose Clarity (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Purpose stated | Clear "why" beyond profit | Present |
| Purpose honest | Not just marketing | Genuine |
| Purpose worthy | Serves human flourishing | Yes |

### 2. Dignity Preserved (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Users respected | Treated as ends, not means | Yes |
| No manipulation | No dark patterns | None |
| Autonomy preserved | Users can choose | Yes |

### 3. Honesty (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Claims honest | No deception | Verified |
| Limitations stated | Not hidden | Present |
| Failures acknowledged | Not swept away | Yes |

### 4. The Golden Rule (High)

| Check | Question | Pass Criteria |
|-------|----------|---------------|
| Reciprocity | Would we want this done to us? | Yes |
| Universalizability | If everyone did this? | Acceptable |

---

## Output Format

```markdown
# Spiritual Advisor Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Dignity Score**: [1-5]
- **Run Date**: [date]

## Purpose Check

| Check | Status | Notes |
|-------|--------|-------|
| Purpose clear | ✓/✗ | |
| Purpose honest | ✓/✗ | |
| Purpose worthy | ✓/✗ | |

## Dignity Check

| Check | Status | Notes |
|-------|--------|-------|
| Users respected | ✓/✗ | |
| No manipulation | ✓/✗ | |
| Autonomy preserved | ✓/✗ | |

## Concerns

| Concern | Severity | Recommendation |
|---------|----------|----------------|

## Verdict

[ ] PASS - Worthy and dignified
[ ] PARTIAL - Concerns to address
[ ] FAIL - Dignity or honesty issues
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Dignity preserved, purpose clear, honest | PASS |
| Minor concerns | PARTIAL |
| Manipulation, deception, or dignity violation | FAIL |
