---
eval_id: curious-searcher-eval
owner: curious-searcher

intent:
  risk_prevented: unclear or misleading first impressions entering external communication
  outcome_ensured: clarity and comprehension validated within bounded perspective of newcomer UX

determinism:
  class: bounded
  justification: |
    LLM-based first-impression evaluation with bounded variability.
    Output varies within explainable limits based on interpretation of clarity and accessibility.
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
    - README.md or equivalent landing page content
    - Quick start documentation
    - Example code or demos
  format: markdown report with clarity scoring
  location: eval-reports/curious-searcher/
---

# Curious Searcher Eval

> Quick clarity and first-impression validation. Target: 10 minutes.

## Mission

Perform a rapid first-impression check. Can a newcomer understand what Converge is and why they should care?

---

## Criteria Checklist

### 1. 30-Second Clarity (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| What it is | Clear in 30 seconds | Yes |
| Who it's for | Obvious audience | Clear |
| Why it matters | Value proposition | Compelling |

### 2. Getting Started (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Next step obvious | What to do next | Clear |
| Easy to try | Low friction start | Yes |
| Examples work | Can run something | Yes |

### 3. Jargon Check (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Plain language | Understandable | Mostly |
| Jargon explained | Technical terms defined | Yes |

### 4. Trust Signals (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Credibility shown | Evidence of quality | Present |
| Limitations honest | Not overselling | Balanced |

---

## Output Format

```markdown
# Curious Searcher Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Clarity Score**: [1-5]
- **Run Date**: [date]

## First Impression

### In 30 seconds I understood:
- What it is: [Yes/No/Partial]
- Who it's for: [Yes/No/Partial]
- Why I'd care: [Yes/No/Partial]

## Getting Started

| Check | Status | Notes |
|-------|--------|-------|
| Next step clear | ✓/✗ | |
| Low friction | ✓/✗ | |
| Examples work | ✓/✗ | |

## Confusion Points

| What | Why Confusing | Suggestion |
|------|---------------|------------|

## Verdict

[ ] PASS - Newcomer can succeed
[ ] PARTIAL - Some confusion
[ ] FAIL - Newcomers will struggle
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Clear value prop, easy to start | PASS |
| Some confusion but workable | PARTIAL |
| Unclear what it is or how to start | FAIL |
