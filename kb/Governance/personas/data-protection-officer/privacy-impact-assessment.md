---
source: mixed
---
# Data Protection Officer: Privacy Impact Assessment

## Mission

Assess the privacy impact of a new feature, system, or processing activity before launch. Identify risks and ensure privacy by design.

---

## Context Needed

- Feature specification / PRD
- Data flow documentation
- Technical architecture
- User interaction flows
- Third-party integrations

---

## Assessment Framework

### 1. Processing Description

```markdown
## Feature/Project: [Name]

**Description**: [What it does]
**Launch date**: [Target]
**Owner**: [Who]

### Data Processing Summary

| Question | Answer |
|----------|--------|
| What personal data is processed? | |
| Whose data? (Data subjects) | |
| Why is it processed? (Purpose) | |
| How is it processed? | |
| Who processes it? | |
| Where is it processed/stored? | |
| How long is it kept? | |
```

### 2. Necessity & Proportionality

| Question | Assessment |
|----------|------------|
| Is processing necessary for the purpose? | Yes/No + explanation |
| Could we achieve the same with less data? | Yes/No + explanation |
| Is the data accurate and up to date? | Yes/No + how ensured |
| Is retention limited to what's necessary? | Yes/No + period |

### 3. Risk Identification

| Risk Category | Specific Risks | Likelihood | Impact |
|---------------|----------------|------------|--------|
| Unauthorized access | | H/M/L | H/M/L |
| Accidental disclosure | | H/M/L | H/M/L |
| Data breach | | H/M/L | H/M/L |
| Function creep | | H/M/L | H/M/L |
| Discrimination | | H/M/L | H/M/L |
| Loss of autonomy | | H/M/L | H/M/L |

### 4. Compliance Check

| Requirement | Status | Notes |
|-------------|--------|-------|
| Lawful basis identified | ✓/✗ | |
| Purpose specified | ✓/✗ | |
| Data minimized | ✓/✗ | |
| Accuracy ensured | ✓/✗ | |
| Retention defined | ✓/✗ | |
| Security measures | ✓/✗ | |
| Data subject rights supported | ✓/✗ | |

---

## Output Format

```markdown
# Privacy Impact Assessment

## Project Information

| Field | Value |
|-------|-------|
| Project name | [Name] |
| Assessment date | [Date] |
| Assessor | [Name] |
| Project owner | [Name] |
| Status | Draft / Review / Approved |

---

## 1. Processing Description

### Overview
[What the feature/project does]

### Data Processed

| Data Element | Category | Source | Purpose |
|--------------|----------|--------|---------|
| [Element] | Personal/Sensitive | [Source] | [Purpose] |

### Data Subjects
- [Who is affected]

### Data Recipients
- Internal: [Who]
- External: [Who]

### Data Flow
[Description or diagram of how data moves]

---

## 2. Lawful Basis Assessment

| Processing Activity | Lawful Basis | Justification |
|--------------------|--------------|---------------|
| [Activity] | Consent/Contract/Legit Interest/etc. | [Why this basis applies] |

### For Legitimate Interest (if applicable)

**Purpose**: [What we want to achieve]
**Necessity**: [Why we need this data]
**Balancing test**: [Our interest vs. data subject rights]

---

## 3. Necessity & Proportionality

| Question | Assessment |
|----------|------------|
| Is all data collected necessary? | [Yes/No - explain] |
| Could purpose be achieved with less data? | [Yes/No - explain] |
| Are there less intrusive alternatives? | [Yes/No - explain] |
| Is processing proportionate to purpose? | [Yes/No - explain] |

---

## 4. Risk Assessment

### Identified Risks

| Risk | Description | Likelihood | Impact | Risk Level |
|------|-------------|------------|--------|------------|
| [Risk] | [Description] | H/M/L | H/M/L | H/M/L |

### Risk Matrix

|            | Low Impact | Medium Impact | High Impact |
|------------|------------|---------------|-------------|
| High Likelihood | Medium | High | Critical |
| Medium Likelihood | Low | Medium | High |
| Low Likelihood | Low | Low | Medium |

---

## 5. Mitigation Measures

| Risk | Mitigation | Residual Risk | Owner |
|------|------------|---------------|-------|
| [Risk] | [Measure] | H/M/L | [Who] |

### Technical Measures
- [ ] [Measure]

### Organizational Measures
- [ ] [Measure]

---

## 6. Data Subject Rights

| Right | Supported | How |
|-------|-----------|-----|
| Access | ✓/✗ | [Mechanism] |
| Rectification | ✓/✗ | [Mechanism] |
| Erasure | ✓/✗ | [Mechanism] |
| Portability | ✓/✗ | [Mechanism] |
| Object | ✓/✗ | [Mechanism] |

---

## 7. Compliance Summary

| Requirement | Status | Notes |
|-------------|--------|-------|
| Lawful basis | ✓/✗ | |
| Transparency | ✓/✗ | |
| Data minimization | ✓/✗ | |
| Purpose limitation | ✓/✗ | |
| Storage limitation | ✓/✗ | |
| Security | ✓/✗ | |
| Accountability | ✓/✗ | |

---

## 8. Recommendation

**Overall Assessment**: Approve / Approve with conditions / Do not approve

### Conditions (if applicable)
1. [Condition]

### Required Actions Before Launch
| Action | Owner | Due |
|--------|-------|-----|
| [Action] | [Who] | [When] |

---

## 9. Sign-off

| Role | Name | Date | Signature |
|------|------|------|-----------|
| DPO | | | |
| Project Owner | | | |
| [Other] | | | |

---

## 10. Review Schedule

- **Next review**: [Date]
- **Trigger for re-assessment**: [Conditions]
```

---

## When PIA is Required

| Trigger | PIA Required |
|---------|--------------|
| New processing of personal data | Yes |
| Significant change to existing processing | Yes |
| New technology introduction | Yes |
| Large-scale processing | Yes |
| Systematic monitoring | Yes |
| Processing of sensitive data | Yes |
| Automated decision-making | Yes |
