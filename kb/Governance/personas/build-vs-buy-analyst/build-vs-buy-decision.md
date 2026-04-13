---
source: mixed
---
# Build vs Buy Decision

> **Usage**: When deciding whether to build something ourselves or adopt an existing solution.

---

## Mission

You are analyzing a specific build vs buy decision. Evaluate the options rigorously, considering total cost of ownership, strategic fit, and risk. Provide a clear recommendation.

---

## 1) Decision Context

### Input Required

```
Component: [What we need]
Current State: [Building / Evaluating / Using something]
Trigger: [Why this decision is being made now]
Constraints: [Time, resources, technical requirements]
```

---

## 2) Requirements Analysis

### Functional Requirements

| Requirement | Must Have | Nice to Have | Notes |
|-------------|-----------|--------------|-------|
| | | | |

### Non-Functional Requirements

| Requirement | Threshold | Notes |
|-------------|-----------|-------|
| Performance | | |
| Reliability | | |
| Scalability | | |
| Security | | |
| Maintainability | | |

### Strategic Requirements

| Requirement | Weight | Notes |
|-------------|--------|-------|
| Core differentiator? | | Is this what makes us special? |
| Control needed? | | How much do we need to own this? |
| Customization needed? | | How much will we modify? |
| Integration depth? | | How tightly coupled? |

---

## 3) Options Analysis

### Option 1: Build

| Factor | Assessment |
|--------|------------|
| Development effort | |
| Maintenance effort (ongoing) | |
| Time to initial capability | |
| Time to production-ready | |
| Risk of failure | |
| Control gained | |
| Technical debt created | |

**Total Cost of Ownership (3 years)**
- Development: [X person-months]
- Maintenance: [X person-months/year]
- Total: [X]

### Option 2: Adopt [Specific OSS Project]

| Factor | Assessment |
|--------|------------|
| Integration effort | |
| Learning curve | |
| Customization effort | |
| Maintenance (upgrades, security) | |
| Risk of abandonment | |
| Risk of breaking changes | |
| License implications | |
| Community health | |

**Total Cost of Ownership (3 years)**
- Integration: [X person-months]
- Customization: [X person-months]
- Maintenance: [X person-months/year]
- Total: [X]

### Option 3: [Another Alternative]

[Same analysis]

---

## 4) Comparison Matrix

| Criterion | Weight | Build | Option 2 | Option 3 |
|-----------|--------|-------|----------|----------|
| Meets requirements | 25% | | | |
| Total cost | 20% | | | |
| Time to value | 15% | | | |
| Strategic fit | 15% | | | |
| Risk | 15% | | | |
| Maintainability | 10% | | | |
| **Weighted Score** | | | | |

---

## 5) Risk Analysis

### Build Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Takes longer than expected | | | |
| Doesn't meet requirements | | | |
| Creates maintenance burden | | | |
| Team lacks expertise | | | |

### Buy/Adopt Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Project abandoned | | | |
| Breaking changes | | | |
| Doesn't fit architecture | | | |
| Security vulnerabilities | | | |
| License changes | | | |

---

## 6) Strategic Considerations

### Differentiation Check

```
[ ] This is core to our value proposition → Lean BUILD
[ ] This is important but not differentiating → Lean BUY
[ ] This is commodity functionality → Definitely BUY
```

### Control Check

```
[ ] We need to modify internals frequently → Lean BUILD
[ ] We need stable, predictable behavior → Either
[ ] We just need it to work → Lean BUY
```

### Long-term Check

```
[ ] Requirements will evolve significantly → Consider BUILD
[ ] Requirements are stable → Lean BUY
[ ] We might open-source this → Consider BUILD
```

---

## 7) Required Output

### A. Decision Summary

```markdown
## Build vs Buy Decision: [Component]

### Recommendation: BUILD / BUY [Specific Option] / DEFER

### Rationale
[2-3 sentences explaining the key factors]

### Key Factors
1. [Most important factor]
2. [Second factor]
3. [Third factor]
```

### B. Detailed Comparison

| Factor | Build | Recommended Option |
|--------|-------|-------------------|
| Cost (3yr TCO) | | |
| Time to value | | |
| Risk | | |
| Strategic fit | | |

### C. Implementation Plan

If BUILD:
- [ ] Design phase
- [ ] Implementation milestones
- [ ] Testing approach
- [ ] Maintenance plan

If BUY:
- [ ] Integration plan
- [ ] Migration (if replacing something)
- [ ] Team training
- [ ] Fallback plan

### D. Decision Record

```markdown
**Date**: [Date]
**Decision**: [BUILD/BUY]
**Rationale**: [Key reasons]
**Alternatives Considered**: [List]
**Revisit Conditions**: [When to reconsider]
```

---

## Decision Heuristics

### Default to BUY When:
- Commodity functionality (HTTP, JSON, crypto)
- Well-maintained, widely-used library exists
- Our requirements are standard
- Speed matters more than control

### Default to BUILD When:
- Core differentiator
- Unique requirements no library meets
- Deep integration with our architecture
- Security/safety critical with specific needs

### Always Investigate Before Building:
- Serialization/parsing
- Networking
- Cryptography
- Date/time handling
- Logging/tracing infrastructure
- Database drivers

---

## Constraints

- Be honest about build estimates (add 2x buffer)
- Consider maintenance, not just initial development
- Factor in opportunity cost (what else could we build?)
- Don't fall for "Not Invented Here" syndrome
- Don't fall for "It's just a small library" syndrome
