# Dependency Review

> **Usage**: Quarterly review of our dependency health and strategy.

---

## Mission

You are reviewing Converge's dependencies to assess health, identify risks, and ensure we're making good build vs buy decisions. Focus on strategic fit, maintenance burden, and risk.

---

## 1) Dependency Inventory

### Direct Dependencies by Category

**Core Infrastructure**
| Dependency | Purpose | Version | Health | Critical? |
|------------|---------|---------|--------|-----------|
| | | | | |

**Serialization/Data**
| Dependency | Purpose | Version | Health | Critical? |
|------------|---------|---------|--------|-----------|
| | | | | |

**Async/Networking**
| Dependency | Purpose | Version | Health | Critical? |
|------------|---------|---------|--------|-----------|
| | | | | |

**ML/AI**
| Dependency | Purpose | Version | Health | Critical? |
|------------|---------|---------|--------|-----------|
| | | | | |

**Utilities**
| Dependency | Purpose | Version | Health | Critical? |
|------------|---------|---------|--------|-----------|
| | | | | |

---

## 2) Health Assessment

### Health Criteria

For each dependency, assess:

| Criterion | Good | Warning | Bad |
|-----------|------|---------|-----|
| Last release | <3 months | 3-12 months | >12 months |
| Open issues | <50 | 50-200 | >200 |
| Maintainers | >3 | 2-3 | 1 |
| Bus factor | >2 | 2 | 1 |
| Breaking changes | Rare, well-communicated | Occasional | Frequent |

### Health Summary

| Dependency | Last Release | Maintainers | Issues | Health Score |
|------------|--------------|-------------|--------|--------------|
| | | | | 🟢/🟡/🔴 |

### At-Risk Dependencies

| Dependency | Risk | Impact if Abandoned | Contingency |
|------------|------|---------------------|-------------|
| | | | |

---

## 3) Strategic Fit Review

### Should We Still Use These?

| Dependency | Original Reason | Still Valid? | Alternatives | Action |
|------------|-----------------|--------------|--------------|--------|
| | | | | Keep/Replace/Remove |

### Dependencies We've Outgrown

| Dependency | Issue | Better Option | Migration Effort |
|------------|-------|---------------|------------------|
| | | | |

### Missing Dependencies

| Need | Current Solution | Better Library? | Action |
|------|------------------|-----------------|--------|
| | Custom code | | Evaluate |
| | Workaround | | Evaluate |

---

## 4) Maintenance Burden

### Upgrade Backlog

| Dependency | Current | Latest | Breaking? | Effort | Priority |
|------------|---------|--------|-----------|--------|----------|
| | | | | | |

### Pinned/Frozen Dependencies

| Dependency | Pinned At | Why | Unpin Plan |
|------------|-----------|-----|------------|
| | | | |

### Dependency Churn

| Dependency | Upgrades (12mo) | Breaking Changes | Maintenance Cost |
|------------|-----------------|------------------|------------------|
| | | | |

---

## 5) Consolidation Opportunities

### Overlapping Dependencies

| Function | Dependencies | Consolidate To |
|----------|--------------|----------------|
| | A, B | Choose one |

### Heavy Dependencies We Barely Use

| Dependency | Features Used | % of Crate | Lighter Alternative |
|------------|---------------|------------|---------------------|
| | | | |

---

## 6) Required Output

### A. Dependency Health Dashboard

```markdown
## Dependency Review - [Quarter] [Year]

### Summary
- Total direct dependencies: [N]
- Healthy (🟢): [N]
- Warning (🟡): [N]
- At-risk (🔴): [N]

### Critical Dependencies Health
| Dependency | Health | Action Needed |
|------------|--------|---------------|
| | | |
```

### B. Risk Register

| Risk | Dependency | Probability | Impact | Mitigation |
|------|------------|-------------|--------|------------|
| | | | | |

### C. Action Items

**Immediate**
| Action | Dependency | Reason | Owner |
|--------|------------|--------|-------|
| | | | |

**This Quarter**
| Action | Dependency | Reason | Owner |
|--------|------------|--------|-------|
| | | | |

**Watch**
| Dependency | Concern | Check Back |
|------------|---------|------------|
| | | |

### D. Recommendations

1. **Add**: [Dependencies we should adopt]
2. **Remove**: [Dependencies we should eliminate]
3. **Replace**: [Dependencies to swap]
4. **Contribute**: [Upstream contributions to make]

### E. Decision Log Updates

| Decision | Date | Rationale |
|----------|------|-----------|
| Added X | | |
| Removed Y | | |
| Replaced Z with W | | |

---

## Dependency Guidelines

### Good Dependency Characteristics
- Active maintenance (commits in last 3 months)
- Multiple maintainers
- Clear versioning (semver)
- Good documentation
- Reasonable issue response time
- Compatible license
- No excessive transitive dependencies

### Red Flags
- Single maintainer
- No releases in 12+ months
- Frequent breaking changes
- Poor documentation
- Unclear license
- Excessive dependencies
- Known security issues

---

## Constraints

- Focus on strategic impact, not just technical health
- Consider total cost (integration + maintenance + risk)
- Don't keep dependencies out of inertia
- Don't remove dependencies out of NIH syndrome
- Document decisions for future reference
