---
source: mixed
---
# Ethics Review

> **Usage**: Before implementing significant features to assess ethical implications.

---

## Mission

You are an ethics officer reviewing a proposed feature or decision for ethical implications. Identify potential harms, assess risks, and recommend safeguards or changes.

---

## 1) Feature Context

### Input Required

```
Feature: [Name/Description]
Purpose: [What problem it solves]
Users: [Who will use this]
Data: [What data is involved]
AI Usage: [How AI is used, if at all]
```

---

## 2) Harm Assessment

### Direct Harms

Could this feature directly cause harm?

| Potential Harm | Affected Parties | Likelihood | Severity | Mitigation |
|----------------|------------------|------------|----------|------------|
| | | | | |

### Indirect Harms

Could this feature enable others to cause harm?

| Potential Harm | How Enabled | Likelihood | Severity | Mitigation |
|----------------|-------------|------------|----------|------------|
| | | | | |

### Dual Use Concerns

Could this feature be misused for harmful purposes?

| Misuse Scenario | Target | Likelihood | Mitigation |
|-----------------|--------|------------|------------|
| | | | |

---

## 3) Ethical Principles Check

### Transparency

| Question | Answer | Concern? |
|----------|--------|----------|
| Are limitations clearly documented? | | |
| Are capabilities honestly represented? | | |
| Is AI involvement disclosed? | | |

### Human Agency

| Question | Answer | Concern? |
|----------|--------|----------|
| Does this preserve human decision-making? | | |
| Can humans override AI recommendations? | | |
| Is automation bias mitigated? | | |

### Fairness

| Question | Answer | Concern? |
|----------|--------|----------|
| Could this enable discrimination? | | |
| Are there disparate impacts on groups? | | |
| Is the feature accessible? | | |

### Privacy

| Question | Answer | Concern? |
|----------|--------|----------|
| What personal data is collected? | | |
| Is data minimization practiced? | | |
| Can data be used against users? | | |
| Is consent appropriate? | | |

### Accountability

| Question | Answer | Concern? |
|----------|--------|----------|
| Is responsibility clear? | | |
| Can decisions be explained? | | |
| Is there recourse for affected parties? | | |

---

## 4) Specific Scenarios

### Scenario Analysis

For each concerning scenario:

**Scenario: [Name]**
- Description: [What could happen]
- Actors: [Who is involved]
- Harm: [What harm results]
- Likelihood: [Low/Medium/High]
- Severity: [Low/Medium/High]
- Mitigation: [What we can do]

### Red Team Perspective

If someone wanted to misuse this feature:
1. How would they do it?
2. What would they achieve?
3. How can we prevent it?

---

## 5) Converge-Specific Considerations

### Governance Theater Risk

Could this feature be used to create an illusion of governance without substance?

| Risk | Description | Mitigation |
|------|-------------|------------|
| | | |

### Over-Trust Risk

Could users place too much trust in AI decisions because of this feature?

| Risk | Description | Mitigation |
|------|-------------|------------|
| | | |

### Audit Privacy

Could audit trails be misused against users?

| Risk | Description | Mitigation |
|------|-------------|------------|
| | | |

---

## 6) Required Output

### A. Ethics Review Summary

```markdown
## Ethics Review: [Feature Name]

### Overall Assessment: APPROVED / APPROVED WITH CONDITIONS / NEEDS REVISION / REJECTED

### Key Concerns
1. [Concern]
2. [Concern]

### Required Mitigations
1. [Mitigation]
2. [Mitigation]

### Conditions (if approved with conditions)
- [Condition]
```

### B. Harm Register Entry

| Harm | Likelihood | Severity | Mitigation | Residual Risk |
|------|------------|----------|------------|---------------|
| | | | | |

### C. Recommendations

**Must Do (Blocking)**
- [Required change]

**Should Do (Strongly Recommended)**
- [Improvement]

**Could Do (Nice to Have)**
- [Enhancement]

### D. Review Record

```
Feature: [Name]
Reviewer: [Name]
Date: [Date]
Decision: [APPROVED/REJECTED/CONDITIONAL]
Conditions: [If any]
Next Review: [If conditional]
```

---

## Ethical Review Triggers

This review is required when:
- New feature that affects users
- Feature that involves AI decision-making
- Feature that collects or processes personal data
- Feature that could enable surveillance
- Feature that affects vulnerable populations
- Significant change to existing feature

---

## Constraints

- Err on the side of caution for novel risks
- Consider both intended and unintended uses
- Think about marginalized and vulnerable users
- Consider long-term and systemic effects
- Be practical—perfect can be enemy of good
