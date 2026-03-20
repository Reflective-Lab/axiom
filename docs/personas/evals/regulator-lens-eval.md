---
eval_id: regulator-lens-eval
owner: regulator-lens

intent:
  risk_prevented: regulatory investigation triggers and compliance violations
  outcome_ensured: no obvious red flags that would trigger regulatory scrutiny

determinism:
  class: bounded
  justification: |
    Regulatory compliance assessment involves interpretation of legal frameworks
    (EU AI Act, GDPR, FTC Act) against product features. While specific regulations
    are documented, determining "spirit of law" compliance and investigation risk
    requires bounded judgment about how regulators would interpret implementation
    choices.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: false
---

# Regulator Lens Eval

> Regulatory perspective validation. Target: 10 minutes.

## Mission

View Converge through the eyes of a regulator. Would this trigger scrutiny? Are we operating within the spirit of regulations, not just the letter?

---

## Criteria Checklist

### 1. Investigation Trigger Check (Critical)

| Red Flag | Present? | Evidence |
|----------|----------|----------|
| Consumer harm potential | Yes/No | |
| Deceptive practices | Yes/No | |
| Unfair competitive behavior | Yes/No | |
| Data protection violations | Yes/No | |
| Discrimination risk | Yes/No | |

### 2. Regulatory Framework Alignment (High)

| Framework | Applicable? | Compliant? | Spirit Followed? |
|-----------|-------------|------------|------------------|
| EU AI Act | Yes/No | Yes/No/Partial | Yes/No |
| GDPR | Yes/No | Yes/No/Partial | Yes/No |
| FTC Act (unfair/deceptive) | Yes/No | Yes/No/Partial | Yes/No |
| CCPA/CPRA | Yes/No | Yes/No/Partial | Yes/No |
| Sector-specific | Yes/No | Yes/No/Partial | Yes/No |

### 3. Regulatory Mindset Check (High)

| Concern | Our Status | Regulator Would See |
|---------|------------|---------------------|
| Consumer protection | | |
| Transparency | | |
| Accountability | | |
| Fairness | | |
| Systemic risk | | |

### 4. Future Regulation Readiness (Medium)

| Emerging Regulation | Impact | Readiness |
|--------------------|--------|-----------|
| AI-specific laws | H/M/L | Ready/Preparing/Not Ready |
| Algorithm transparency | H/M/L | Ready/Preparing/Not Ready |
| Automated decision rights | H/M/L | Ready/Preparing/Not Ready |

---

## Output Format

```markdown
# Regulator Lens Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Investigation Risk**: Low / Medium / High
- **Run Date**: [date]

## Investigation Trigger Analysis

| Potential Trigger | Risk Level | Mitigation |
|-------------------|------------|------------|
| [Trigger] | H/M/L | [What we do] |

### Would We Trigger an Investigation?

**Assessment**: Yes / Possibly / Unlikely / No
**Rationale**: [Why]

## Regulatory Compliance

| Framework | Letter | Spirit | Gap |
|-----------|--------|--------|-----|
| [Framework] | ✓/✗ | ✓/✗ | [Issue] |

## The Regulator's View

If a regulator looked at us today, they would see:

**Positive:**
- [What looks good]

**Concerning:**
- [What raises flags]

**Press Release Risk:**
> "[Hypothetical negative headline]"

## Future Readiness

| Upcoming Regulation | Status | Action Needed |
|--------------------|--------|---------------|
| [Regulation] | Ready/Not | [Action] |

## Verdict

[ ] PASS - Would not trigger regulatory scrutiny
[ ] PARTIAL - Some areas need attention
[ ] FAIL - Significant regulatory risk
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No red flags, spirit of law followed | PASS |
| Minor gaps, letter followed but spirit questionable | PARTIAL |
| Would likely trigger investigation | FAIL |
