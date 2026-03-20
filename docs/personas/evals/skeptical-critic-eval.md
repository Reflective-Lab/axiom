---
eval_id: skeptical-critic-eval
owner: skeptical-critic

intent:
  risk_prevented: unchallenged assumptions and overstated claims entering external communication
  outcome_ensured: key assumptions identified and claim support validated within critical lens

determinism:
  class: bounded
  justification: |
    LLM-based critical evaluation with bounded variability.
    Output varies within explainable limits based on assumption identification and claim assessment.
    Not deterministic (varies by run), not nondeterministic (failure modes known and explainable).

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
    - Core claims and value propositions
    - Technical assertions and benchmarks
    - Business model assumptions
  format: markdown report with assumption and claim audit tables
  location: eval-reports/skeptical-critic/
---

# Skeptical Critic Eval

> Quick assumption and claim challenge. Target: 10 minutes.

## Mission

Challenge key assumptions and claims. Find weaknesses before others do.

---

## Criteria Checklist

### 1. Assumption Check (Critical)

| Assumption | Evidence? | Could Be Wrong? |
|------------|-----------|-----------------|
| Market exists | | |
| Timing is right | | |
| Approach is correct | | |
| Team can execute | | |

### 2. Claim Check (Critical)

| Claim | Supported? | Overstated? |
|-------|------------|-------------|
| [Major claim 1] | | |
| [Major claim 2] | | |
| [Major claim 3] | | |

### 3. Weakness Identification (High)

| Question | Honest Answer |
|----------|---------------|
| What's the weakest part? | |
| What could kill this? | |
| What are we avoiding? | |

### 4. Competitor View (Medium)

| Question | Answer |
|----------|--------|
| How would a competitor attack? | |
| What would a skeptic say? | |

---

## Output Format

```markdown
# Skeptical Critic Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Rigor Score**: [1-5]
- **Run Date**: [date]

## Assumption Audit

| Assumption | Evidence | Confidence |
|------------|----------|------------|
| | | High/Med/Low |

## Claim Audit

| Claim | Supported | Issue |
|-------|-----------|-------|
| | ✓/✗ | |

## Weaknesses Found

| Weakness | Severity | Addressed? |
|----------|----------|------------|

## Uncomfortable Questions

1. [Question we're avoiding]
2. [Question that challenges us]

## Verdict

[ ] PASS - Claims supported, weaknesses known
[ ] PARTIAL - Some unsupported claims
[ ] FAIL - Major unsupported assumptions
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Claims supported, weaknesses acknowledged | PASS |
| Some claims weak | PARTIAL |
| Core assumptions unsupported | FAIL |
