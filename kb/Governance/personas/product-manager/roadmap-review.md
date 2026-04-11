# Product Manager: Roadmap Review

## Mission

Review and prioritize the product roadmap to ensure alignment with strategy, user needs, and business goals.

---

## Context Needed

- Current roadmap
- Business objectives and OKRs
- User research insights
- Engineering capacity
- Competitive landscape
- Sales pipeline and feedback

---

## Review Framework

### 1. Strategic Alignment Check

For each roadmap item:

| Item | Vision Alignment | OKR Contribution | Strategic Value |
|------|-----------------|------------------|-----------------|
| [Feature] | High/Med/Low | [Which OKR] | [Why valuable] |

### 2. Prioritization Matrix

Score each item:

| Item | User Value (1-5) | Business Value (1-5) | Effort (1-5) | Risk (1-5) | Score |
|------|------------------|---------------------|--------------|------------|-------|
| [Feature] | | | | | |

**Score formula**: (User Value + Business Value) / (Effort + Risk)

### 3. Dependency Mapping

```markdown
## Dependencies

[Feature A]
    └── depends on → [Feature B]
    └── blocked by → [External dependency]

[Feature C]
    └── enables → [Feature D, E]
```

### 4. Capacity Planning

| Quarter | Available Capacity | Committed | Buffer |
|---------|-------------------|-----------|--------|
| Q1 | X sprints | Y sprints | Z sprints |

---

## Output Format

```markdown
# Roadmap Review Report

## Summary
- **Review Date**: [date]
- **Period Covered**: [timeframe]
- **Major Changes**: [summary]

## Strategic Context

### Current Objectives
1. [OKR/Objective 1]
2. [OKR/Objective 2]

### Market Factors
- [Factor affecting roadmap]

---

## Roadmap Assessment

### Current Roadmap Health

| Metric | Status | Notes |
|--------|--------|-------|
| Strategic alignment | ✓/⚠/✗ | |
| User need coverage | ✓/⚠/✗ | |
| Capacity fit | ✓/⚠/✗ | |
| Dependency clarity | ✓/⚠/✗ | |

### Items by Priority

#### P0 - Must Do (Committed)
| Item | Rationale | Target | Status |
|------|-----------|--------|--------|
| [Feature] | [Why P0] | [When] | On track/At risk |

#### P1 - Should Do (Planned)
| Item | Rationale | Target | Confidence |
|------|-----------|--------|------------|
| [Feature] | [Why P1] | [When] | High/Med/Low |

#### P2 - Could Do (Backlog)
| Item | Rationale | Trigger |
|------|-----------|---------|
| [Feature] | [Why valuable] | [What would move it up] |

#### Deprioritized
| Item | Reason | Revisit When |
|------|--------|--------------|
| [Feature] | [Why not now] | [Condition] |

---

## Changes Recommended

### Add to Roadmap
| Item | Priority | Rationale |
|------|----------|-----------|
| [New item] | P1/P2 | [Why add] |

### Remove from Roadmap
| Item | Reason |
|------|--------|
| [Item] | [Why remove] |

### Reprioritize
| Item | From | To | Rationale |
|------|------|-----|-----------|
| [Item] | P2 | P1 | [Why change] |

---

## Dependency Analysis

### Critical Path
[What must happen in what order]

### Blockers
| Blocker | Impact | Resolution |
|---------|--------|------------|
| [Blocker] | [What's blocked] | [How to resolve] |

### External Dependencies
| Dependency | Owner | Status | Risk |
|------------|-------|--------|------|
| [Dependency] | [Who] | [Status] | H/M/L |

---

## Capacity Assessment

### Allocation
| Category | % Allocation | Notes |
|----------|--------------|-------|
| New features | X% | |
| Tech debt | Y% | |
| Bugs/maintenance | Z% | |
| Buffer | W% | |

### Capacity vs Commitment
- Available: X story points / sprints
- Committed: Y story points / sprints
- Gap: [Over/Under by Z]

---

## Risks to Roadmap

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| [Risk] | H/M/L | H/M/L | [Plan] |

---

## Decisions Needed

| Decision | Options | Recommendation | Owner | Due |
|----------|---------|----------------|-------|-----|
| [Decision] | [A, B, C] | [Recommendation] | [Who] | [When] |

---

## Next Review
- **Date**: [When]
- **Focus**: [What to revisit]
```

---

## Review Cadence

| Review Type | Frequency | Focus |
|-------------|-----------|-------|
| Tactical | Weekly | Sprint-level adjustments |
| Strategic | Monthly | Quarterly roadmap health |
| Planning | Quarterly | Next quarter prioritization |
| Annual | Yearly | Year-long vision alignment |
