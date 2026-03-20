---
eval_id: insurance-underwriter-eval
owner: insurance-underwriter

intent:
  risk_prevented: uninsurable liability exposures and uncontrolled loss scenarios
  outcome_ensured: risks are identified, adequately controlled, and insurable

determinism:
  class: bounded
  justification: |
    Risk assessment involves quantitative modeling (annual loss potential, severity
    ratings) but requires bounded judgment about control adequacy and insurability
    factors. Loss scenarios are evaluated against probability distributions and
    impact ranges, not simple pass/fail criteria. Insurance underwriting combines
    actuarial data with qualitative risk factors.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: false
---

# Insurance Underwriter Eval

> Risk and insurability validation. Target: 10 minutes.

## Mission

Evaluate Converge from an insurance/risk perspective. What exposures exist? Are risks insurable? Are adequate controls in place?

---

## Criteria Checklist

### 1. Liability Exposure Check (Critical)

| Exposure Type | Present? | Severity | Controls |
|---------------|----------|----------|----------|
| Professional liability (bad advice) | Yes/No | H/M/L | Strong/Weak/None |
| Product liability (harm from AI) | Yes/No | H/M/L | Strong/Weak/None |
| Cyber liability (data breach) | Yes/No | H/M/L | Strong/Weak/None |
| E&O (product doesn't perform) | Yes/No | H/M/L | Strong/Weak/None |
| Regulatory fines | Yes/No | H/M/L | Strong/Weak/None |

### 2. Loss Scenario Check (Critical)

| Scenario | Likelihood | Impact | Insured? |
|----------|------------|--------|----------|
| AI gives wrong recommendation | H/M/L | $? | Yes/No/Partial |
| Data breach exposes user data | H/M/L | $? | Yes/No/Partial |
| System failure causes losses | H/M/L | $? | Yes/No/Partial |
| Discrimination claim | H/M/L | $? | Yes/No/Partial |
| Regulatory enforcement | H/M/L | $? | Yes/No/Partial |

### 3. Controls Assessment (High)

| Control | Status | Adequate? |
|---------|--------|-----------|
| Audit trail/documentation | ✓/✗ | Yes/No |
| Human oversight | ✓/✗ | Yes/No |
| Testing/validation | ✓/✗ | Yes/No |
| Incident response plan | ✓/✗ | Yes/No |
| Contract protections | ✓/✗ | Yes/No |
| Error correction ability | ✓/✗ | Yes/No |

### 4. Insurability Factors (High)

| Factor | Status | Impact on Insurability |
|--------|--------|----------------------|
| Clear scope/use cases | ✓/✗ | Positive/Negative |
| Human-in-loop | ✓/✗ | Positive/Negative |
| Reversible decisions | ✓/✗ | Positive/Negative |
| Good documentation | ✓/✗ | Positive/Negative |
| Incident history | Clean/Issues | Positive/Negative |

---

## Output Format

```markdown
# Insurance Underwriter Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Overall Risk Level**: Low / Moderate / High / Very High
- **Insurability**: Readily Insurable / Insurable with Conditions / Difficult
- **Run Date**: [date]

## Exposure Assessment

| Exposure Type | Severity | Annual Loss Potential | Adequately Controlled? |
|---------------|----------|----------------------|----------------------|
| [Type] | H/M/L | $X-Y range | Yes/No |

### Top Loss Scenarios

1. **[Scenario Name]** - Likelihood: H/M/L, Impact: $X
   - Current controls: [What exists]
   - Gap: [What's missing]

2. [Repeat for top 3-5]

## Controls Evaluation

| Control Area | Status | Rating |
|--------------|--------|--------|
| Documentation | [Description] | Strong/Adequate/Weak |
| Human oversight | [Description] | Strong/Adequate/Weak |
| Testing | [Description] | Strong/Adequate/Weak |
| Incident response | [Description] | Strong/Adequate/Weak |

### Control Gaps

1. [Gap that increases risk]

## Insurability Assessment

| Factor | Assessment | Impact |
|--------|------------|--------|
| Scope definition | Clear/Unclear | +/- |
| Loss history | Clean/Issues | +/- |
| Controls quality | Strong/Weak | +/- |
| Documentation | Good/Poor | +/- |

### Insurance Recommendation

| Coverage Type | Needed? | Estimated Difficulty |
|---------------|---------|---------------------|
| Tech E&O | Yes/No | Easy/Moderate/Hard |
| Cyber | Yes/No | Easy/Moderate/Hard |
| Product Liability | Yes/No | Easy/Moderate/Hard |

## Risk Improvement Priorities

1. [Most impactful improvement]
2. [Second priority]
3. [Third priority]

## Verdict

[ ] PASS - Risks understood and controlled, insurable
[ ] PARTIAL - Some gaps, insurable with improvements
[ ] FAIL - Significant uncontrolled risks, insurability concerns
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Risks identified, controls adequate, readily insurable | PASS |
| Some control gaps, insurable with conditions | PARTIAL |
| Major uncontrolled exposures, difficult to insure | FAIL |
