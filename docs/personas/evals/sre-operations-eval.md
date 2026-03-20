---
eval_id: sre-operations-eval
owner: sre-operations

intent:
  risk_prevented: [deployment to unready infrastructure causing outages or data loss]
  outcome_ensured: [operational readiness checks passed - monitoring, rollback, capacity verified]

determinism:
  class: bounded
  justification: |
    Operational checks depend on external infrastructure state (monitoring systems,
    deployment targets, capacity metrics). Output varies with infrastructure state but
    failure modes are known and bounded. Never sole blocker per D2 rules.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    pr-merge:
      may_contribute: true
      may_block: false
    release-candidate:
      may_contribute: true
      may_block: true
    release-approval:
      may_contribute: true
      may_block: true
    production-deploy:
      may_contribute: true
      may_block: true
---

# SRE / Operations Eval

> Quick operational readiness validation. Target: 10 minutes.

## Mission

Perform a rapid operational readiness check. Verify observability, reliability basics, and operational support exist.

---

## Criteria Checklist

### 1. Observability Basics (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Logging exists | Structured logging | Present |
| Metrics exist | Basic instrumentation | Present |
| Health check exists | /health or equivalent | Present |

**Quick checks:**
```bash
# Look for logging
grep -r "tracing::\|log::" src/ | head -10

# Look for metrics
grep -r "metrics::\|prometheus" src/ | head -5

# Look for health check
grep -ri "health" src/ | head -5
```

### 2. Error Handling (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Errors are typed | Custom error types | Present |
| Errors are informative | Good error messages | Present |
| No panics in happy path | Unwrap/expect usage | Minimal |

**Quick check:**
```bash
# Count unwrap/expect usage
grep -r "\.unwrap()\|\.expect(" src/ | wc -l
```

### 3. Configuration (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Config externalized | Not hardcoded | Verified |
| Env vars supported | Config from environment | Yes |
| Defaults are safe | Secure defaults | Yes |

### 4. Documentation (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| README has run instructions | How to run | Present |
| Config documented | Config options listed | Present |
| Troubleshooting exists | Common issues | Some |

---

## Output Format

```markdown
# SRE / Operations Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Gaps**: [count]
- **Run Date**: [date]

## Observability

| Check | Status | Notes |
|-------|--------|-------|
| Logging present | ✓/✗ | |
| Metrics present | ✓/✗ | |
| Health check | ✓/✗ | |

## Reliability

| Check | Status | Notes |
|-------|--------|-------|
| Error types | ✓/✗ | |
| Error messages | ✓/✗ | |
| Panic safety | ✓/✗ | unwrap count: X |

## Configuration

| Check | Status | Notes |
|-------|--------|-------|
| Externalized | ✓/✗ | |
| Env vars | ✓/✗ | |
| Safe defaults | ✓/✗ | |

## Documentation

| Check | Status | Notes |
|-------|--------|-------|
| Run instructions | ✓/✗ | |
| Config docs | ✓/✗ | |
| Troubleshooting | ✓/✗ | |

## Gaps

| Gap | Severity | Recommendation |
|-----|----------|----------------|
| [if any] | | |

## Verdict

[ ] PASS - Operationally ready
[ ] PARTIAL - Gaps but operable
[ ] FAIL - Not ready for production
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Logging + health check + good error handling | PASS |
| Some observability gaps | PARTIAL |
| No logging or unhandled errors | FAIL |
