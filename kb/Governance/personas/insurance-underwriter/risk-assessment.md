---
source: mixed
---
# Insurance Underwriter: Risk Assessment

## Mission

Conduct a comprehensive risk assessment of Converge from an insurability perspective. Identify exposures, evaluate controls, and determine risk profile.

---

## Context Needed

Before running this assessment, gather:
- Product documentation and capabilities
- Use case descriptions
- Current insurance coverage (if any)
- Incident history
- Control documentation

---

## Assessment Framework

### 1. Exposure Identification

For each product capability, identify potential loss scenarios:

```markdown
## Exposure Analysis

### [Capability/Feature]

| Scenario | Likelihood | Severity | Exposure |
|----------|------------|----------|----------|
| [What could go wrong] | H/M/L | $K/$M/$B | [Description] |
```

### 2. Loss Scenarios

Develop detailed scenarios for top risks:

```markdown
## Loss Scenario: [Name]

**Trigger**: What causes this scenario?
**Sequence**: How does loss develop?
**Impact**: What damages result?
**Claimants**: Who would make claims?
**Estimated Loss**: Range of potential damages
**Defensibility**: How strong is our defense?
```

### 3. Controls Assessment

Evaluate existing risk mitigation:

| Control | Risk Addressed | Effectiveness | Gap |
|---------|---------------|---------------|-----|
| [Control name] | [Risk] | Strong/Adequate/Weak | [What's missing] |

### 4. Concentration Analysis

Assess risk concentration:

| Dimension | Concentration Level | Concern |
|-----------|--------------------|---------|
| Customer concentration | % in top 10 customers | |
| Use case concentration | % in top use case | |
| Geographic concentration | % in top region | |
| Technology concentration | Single points of failure | |

---

## Output Format

```markdown
# Risk Assessment Report

## Executive Summary
- **Overall Risk Level**: Low / Moderate / High / Very High
- **Insurability**: Readily insurable / Insurable with conditions / Difficult to insure
- **Key Exposures**: [Top 3]
- **Assessment Date**: [date]

## Risk Profile

### Exposure Summary

| Risk Category | Exposure Level | Annual Loss Potential | Controls |
|---------------|----------------|----------------------|----------|
| Professional Liability | H/M/L | $X-Y | Strong/Adequate/Weak |
| Cyber Liability | H/M/L | $X-Y | Strong/Adequate/Weak |
| Product Liability | H/M/L | $X-Y | Strong/Adequate/Weak |
| E&O | H/M/L | $X-Y | Strong/Adequate/Weak |

### Top Loss Scenarios

1. **[Scenario Name]**
   - Trigger: [What causes it]
   - Estimated Impact: $[range]
   - Likelihood: [H/M/L]
   - Current Controls: [What mitigates it]

2. [Repeat for top 5]

### Controls Assessment

| Control Area | Status | Recommendation |
|--------------|--------|----------------|
| Documentation/Audit | ✓/✗ | |
| Human Oversight | ✓/✗ | |
| Testing/Validation | ✓/✗ | |
| Incident Response | ✓/✗ | |
| Contract Protections | ✓/✗ | |

### Coverage Recommendations

| Coverage Type | Recommended Limit | Estimated Premium | Priority |
|---------------|-------------------|-------------------|----------|
| Tech E&O | $X | $Y/year | High/Medium/Low |
| Cyber | $X | $Y/year | High/Medium/Low |
| [etc] | | | |

## Risk Improvement Recommendations

| Recommendation | Risk Reduced | Effort | Priority |
|----------------|--------------|--------|----------|
| [Action] | [Which risk] | H/M/L | 1-5 |

## Conclusion

[Overall assessment and key takeaways]
```

---

## Assessment Criteria

### Insurability Thresholds

| Threshold | Insurability |
|-----------|--------------|
| Strong controls, good documentation, limited scope | Readily insurable |
| Adequate controls, some gaps, defined scope | Insurable with conditions |
| Weak controls, poor documentation, broad scope | Difficult to insure |
| No controls, no documentation, unlimited scope | May be uninsurable |
