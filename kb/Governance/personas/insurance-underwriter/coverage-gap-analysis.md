---
source: mixed
---
# Insurance Underwriter: Coverage Gap Analysis

## Mission

Analyze current insurance coverage against identified risks to find gaps, overlaps, and optimization opportunities.

---

## Context Needed

- Current insurance policies (COIs, policy documents)
- Risk assessment output
- Business activities and revenue
- Contract requirements from customers
- Historical claims

---

## Analysis Framework

### 1. Current Coverage Inventory

Document existing coverage:

```markdown
## Coverage Inventory

| Policy Type | Carrier | Limit | Deductible | Premium | Expiration |
|-------------|---------|-------|------------|---------|------------|
| [Type] | [Carrier] | $X | $Y | $Z/yr | [Date] |
```

### 2. Risk-to-Coverage Mapping

Map identified risks to available coverage:

```markdown
## Coverage Mapping

| Risk | Potential Loss | Coverage Available | Gap |
|------|---------------|-------------------|-----|
| [Risk] | $X | [Policy, limit] | [Uncovered amount/scenario] |
```

### 3. Gap Identification

For each gap:

```markdown
## Gap: [Name]

**Risk**: What risk is uncovered?
**Exposure**: What's the potential loss?
**Why Gap Exists**: Exclusion? Sublimit? No coverage purchased?
**Options**: How to address?
**Recommendation**: What to do?
```

### 4. Coverage Optimization

Identify overlaps and optimization opportunities:

| Overlap/Inefficiency | Policies Involved | Recommendation |
|---------------------|-------------------|----------------|
| [Description] | [Policies] | [Action] |

---

## Output Format

```markdown
# Coverage Gap Analysis Report

## Summary
- **Total Identified Risks**: [count]
- **Fully Covered**: [count]
- **Partially Covered**: [count]
- **Uncovered**: [count]
- **Assessment Date**: [date]

## Coverage Inventory

| Policy | Limit | Key Coverages | Key Exclusions |
|--------|-------|---------------|----------------|
| Tech E&O | $X | [list] | [list] |
| Cyber | $X | [list] | [list] |
| [etc] | | | |

## Gap Analysis

### Critical Gaps (Uninsured Exposures)

| Gap | Exposure | Impact | Priority |
|-----|----------|--------|----------|
| [Gap] | $X | [Scenario] | High |

### Partial Coverage (Sublimits/Exclusions)

| Risk | Current Coverage | Gap | Impact |
|------|-----------------|-----|--------|
| [Risk] | [What's covered] | [What's not] | $X exposure |

### Contract Requirements

| Customer/Contract | Required Coverage | Current Status | Gap |
|------------------|-------------------|----------------|-----|
| [Name] | [Requirement] | Met/Not Met | [Issue] |

## Recommendations

### Immediate Actions (Critical Gaps)

1. **[Action]**
   - Gap addressed: [Gap]
   - Estimated cost: $X
   - Timeline: [When]

### Near-term Improvements

1. [Action with rationale]

### Coverage Optimization

| Current | Recommended | Savings/Benefit |
|---------|-------------|-----------------|
| [Current setup] | [Better setup] | $X or [benefit] |

## Renewal Recommendations

For upcoming renewals:
- [Policy]: [Recommendation]
```

---

## Common Gaps in AI/Tech Companies

| Common Gap | Why It Matters |
|------------|----------------|
| AI-specific exclusions | Many policies exclude AI-related claims |
| Bodily injury from tech | Tech E&O often excludes BI |
| Reputational harm | Often excluded or sublimited |
| Regulatory fines | May not be covered |
| Contractual liability | Coverage may not follow contracts |
| Retroactive date issues | Prior acts may be excluded |
