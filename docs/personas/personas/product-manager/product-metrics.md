# Product Manager: Product Metrics Review

## Mission

Define, track, and analyze product metrics to understand product health, user behavior, and feature impact.

---

## Context Needed

- Current metrics dashboards
- Analytics data
- Business goals and OKRs
- Recent releases and features
- User feedback

---

## Metrics Framework

### 1. Metric Categories

#### North Star Metric
The one metric that best captures the value users get from the product.

```markdown
**North Star**: [Metric name]
**Definition**: [How it's calculated]
**Why**: [Why this represents user value]
**Current**: [Value]
**Target**: [Goal]
```

#### Input Metrics
Metrics that drive the North Star:

| Metric | Relationship to North Star | Current | Target |
|--------|---------------------------|---------|--------|
| [Metric] | [How it contributes] | [Value] | [Goal] |

### 2. AARRR Framework (Pirate Metrics)

| Stage | Metric | Definition | Current | Target |
|-------|--------|------------|---------|--------|
| **Acquisition** | [Metric] | | | |
| **Activation** | [Metric] | | | |
| **Retention** | [Metric] | | | |
| **Revenue** | [Metric] | | | |
| **Referral** | [Metric] | | | |

### 3. Feature-Specific Metrics

For each major feature:

```markdown
## Feature: [Name]

**Launch Date**: [Date]
**Success Criteria**: [What success looks like]

| Metric | Pre-Launch | Current | Target | Status |
|--------|------------|---------|--------|--------|
| Adoption rate | N/A | X% | Y% | ✓/⚠/✗ |
| Usage frequency | N/A | X/week | Y/week | ✓/⚠/✗ |
| Task completion | N/A | X% | Y% | ✓/⚠/✗ |
```

---

## Output Format

```markdown
# Product Metrics Review

## Summary
- **Period**: [Date range]
- **Overall Health**: Healthy / Warning / Critical
- **Key Insight**: [One sentence]

---

## North Star Metric

**[Metric Name]**: [Current Value]

| Period | Value | Change | Trend |
|--------|-------|--------|-------|
| This month | X | +Y% | ↑/↓/→ |
| Last month | X | | |
| 3 months ago | X | | |

**Analysis**: [What's driving the trend]

---

## Funnel Health

### Acquisition
| Metric | Current | Previous | Change | Status |
|--------|---------|----------|--------|--------|
| [Metric] | X | Y | +/-Z% | ✓/⚠/✗ |

**Insight**: [What we're seeing]

### Activation
| Metric | Current | Previous | Change | Status |
|--------|---------|----------|--------|--------|
| [Metric] | X | Y | +/-Z% | ✓/⚠/✗ |

**Insight**: [What we're seeing]

### Retention
| Metric | Current | Previous | Change | Status |
|--------|---------|----------|--------|--------|
| D1 retention | X% | Y% | | |
| D7 retention | X% | Y% | | |
| D30 retention | X% | Y% | | |

**Insight**: [What we're seeing]

### Revenue
| Metric | Current | Previous | Change | Status |
|--------|---------|----------|--------|--------|
| [Metric] | $X | $Y | +/-Z% | ✓/⚠/✗ |

**Insight**: [What we're seeing]

---

## Feature Performance

### [Feature Name]
- **Status**: Meeting goals / Below target / Exceeding
- **Adoption**: X% of eligible users
- **Engagement**: [Usage pattern]

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| [Metric] | X | Y | ✓/⚠/✗ |

**Analysis**: [What's working, what's not]
**Recommendation**: [Next steps]

### [Feature Name]
[Repeat structure]

---

## Cohort Analysis

### [Cohort Definition]

| Cohort | Week 1 | Week 2 | Week 4 | Week 8 |
|--------|--------|--------|--------|--------|
| [Date] | X% | Y% | Z% | W% |

**Insight**: [What cohort data reveals]

---

## Experiments & Tests

| Experiment | Hypothesis | Result | Significance | Decision |
|------------|------------|--------|--------------|----------|
| [Test name] | [Hypothesis] | [Outcome] | p=X | Ship/Kill/Iterate |

---

## Anomalies & Investigations

| Anomaly | When | Impact | Root Cause | Action |
|---------|------|--------|------------|--------|
| [Anomaly] | [Date] | [Effect] | [Cause] | [Fix] |

---

## Metric Health Check

| Metric | Data Quality | Tracking Status | Issues |
|--------|--------------|-----------------|--------|
| [Metric] | Good/Fair/Poor | Active/Broken | [Issue] |

---

## Insights & Recommendations

### Key Insights
1. [Insight with evidence]
2. [Insight with evidence]

### Recommended Actions
| Action | Expected Impact | Priority | Owner |
|--------|-----------------|----------|-------|
| [Action] | [Impact] | P1/P2/P3 | [Who] |

### Questions to Investigate
1. [Open question]

---

## Next Review
- **Date**: [When]
- **Focus**: [What to watch]
```

---

## Metrics Hygiene

### Good Metrics Are:
- **Actionable**: You can change behavior based on them
- **Accessible**: Team can understand and access them
- **Auditable**: You can verify accuracy

### Metrics Anti-Patterns:
- Vanity metrics (look good but don't drive decisions)
- Lagging-only metrics (no leading indicators)
- Too many metrics (no focus)
- Metric gaming (optimizing metric, not outcome)
