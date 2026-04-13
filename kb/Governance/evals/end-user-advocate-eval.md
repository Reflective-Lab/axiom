---
eval_id: end-user-advocate-eval
owner: end-user-advocate

intent:
  risk_prevented: end-user rights violations or protection gaps entering production
  outcome_ensured: end-user protection criteria assessed within advocacy lens

determinism:
  class: nondeterministic
  justification: |
    Human judgment proxy for user rights advocacy perspective.
    Output depends on harm assessment, rights interpretation, and protection adequacy evaluation.
    Cannot be deterministically replayed - advocacy judgments vary based on user context interpretation.

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
    - User rights documentation (right to know, contest, human review)
    - End-user impact assessment
    - Privacy and protection policies
    - Recourse mechanisms
  format: markdown report with user rights audit
  location: eval-reports/end-user-advocate/
source: mixed
---

# End User Advocate Eval

> Quick end-user protection validation. Target: 10 minutes.

## Mission

Check if end users (not our customers, but their users) are protected and considered.

---

## Criteria Checklist

### 1. User Rights (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Right to know | Users know AI is involved | Documented |
| Right to contest | Can challenge decisions | Exists |
| Right to human | Can request human review | Possible |

### 2. User Protection (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Harm considered | End user harm assessed | Yes |
| Recourse exists | What if AI is wrong | Defined |
| Privacy respected | User data protected | Yes |

### 3. Not Forgotten (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Mentioned in docs | End users referenced | Present |
| Considered in design | Design accounts for them | Evidence |

---

## Output Format

```markdown
# End User Advocate Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **User Protection Score**: [1-5]
- **Run Date**: [date]

## User Rights

| Right | Status | How Ensured |
|-------|--------|-------------|
| Know | ✓/✗ | |
| Contest | ✓/✗ | |
| Human review | ✓/✗ | |

## Protection Check

| Check | Status | Notes |
|-------|--------|-------|
| Harm considered | ✓/✗ | |
| Recourse exists | ✓/✗ | |
| Privacy respected | ✓/✗ | |

## Concerns

| Concern | Affected Users | Recommendation |
|---------|---------------|----------------|

## Verdict

[ ] PASS - End users protected
[ ] PARTIAL - Some gaps
[ ] FAIL - End users forgotten or unprotected
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Rights supported, harm considered | PASS |
| Some gaps in protection | PARTIAL |
| End users not considered | FAIL |
