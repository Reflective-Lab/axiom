---
eval_id: ethics-safety-eval
owner: ethics-safety-officer

intent:
  risk_prevented: release of software causing user harm, discrimination, or ethical violations
  outcome_ensured: known ethical requirements checked; no known violations of documented ethics criteria

determinism:
  class: deterministic
  justification: |
    Ethics checks are rule-based against documented ethics criteria and harm taxonomies.
    Same artifacts and criteria produce same assessment. Judgment is pre-codified in
    ethics guidelines, not applied ad-hoc.

governance:
  may_block_alone: true
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: true
---

# Ethics & Safety Eval

> Quick ethics/safety validation. Target: 10 minutes.

## Mission

Perform a rapid ethics and safety check. Verify responsible AI practices, harm mitigations, and ethical alignment.

---

## Criteria Checklist

### 1. No Harmful Capabilities (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No surveillance features | User tracking beyond audit | None |
| No discrimination enablers | Bias amplification | None |
| No deception features | Fake confidence, misleading | None |

### 2. Honest Claims (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No false safety claims | "Ensures safe AI" | Qualified claims |
| No false compliance claims | "Guarantees compliance" | Qualified claims |
| Limitations documented | What we don't do | Present |

**Review README and docs for:**
- "guarantees" → should be "helps with"
- "ensures" → should be "supports"
- "prevents" → should be "helps detect"

### 3. Human Oversight (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Override mechanisms | Human can override AI | Present |
| Not fully automated | Critical decisions involve humans | Verified |
| Escalation paths | Uncertainty escalates | Designed |

### 4. Transparency (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| AI involvement clear | Users know when AI is used | Documented |
| Limitations stated | What could go wrong | Documented |
| No black boxes | Decisions are explainable | Traceable |

### 5. Practicing What We Preach (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Using own governance | Converge governs our AI | Some evidence |
| Responsible AI usage | Our AI use is responsible | Verified |

---

## Output Format

```markdown
# Ethics & Safety Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Issues**: [count]
- **Concerns**: [count]
- **Run Date**: [date]

## Harm Check

| Check | Status | Notes |
|-------|--------|-------|
| No surveillance features | ✓/✗ | |
| No discrimination enablers | ✓/✗ | |
| No deception | ✓/✗ | |

## Honesty Check

| Check | Status | Notes |
|-------|--------|-------|
| Claims are qualified | ✓/✗ | |
| Limitations documented | ✓/✗ | |
| No overclaiming | ✓/✗ | |

## Responsible Design

| Check | Status | Notes |
|-------|--------|-------|
| Human oversight possible | ✓/✗ | |
| Transparency adequate | ✓/✗ | |
| Explainability supported | ✓/✗ | |

## Concerns

| Concern | Severity | Recommendation |
|---------|----------|----------------|
| [if any] | | |

## Verdict

[ ] PASS - Ethically sound
[ ] PARTIAL - Minor concerns to address
[ ] FAIL - Ethical issues require attention
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No harmful capabilities, honest claims | PASS |
| Minor transparency gaps | PARTIAL |
| Harmful capabilities or deceptive claims | FAIL |
