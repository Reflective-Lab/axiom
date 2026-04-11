# Sustainability Audit

> **Usage**: Quarterly comprehensive sustainability assessment.

---

## Mission

You are conducting a sustainability audit of Converge. Assess environmental impact across compute, storage, development practices, and product design. Identify waste and recommend efficiency improvements.

---

## 1) Compute Efficiency

### Build & CI/CD

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Average CI run time | | | |
| CI runs per day | | | |
| Failed CI runs (waste) | | | |
| Duplicate/unnecessary runs | | | |

**Compute Waste Identification:**
- [ ] Tests that run unnecessarily
- [ ] Builds triggered without code changes
- [ ] Redundant CI steps
- [ ] Over-provisioned runners

### Development Compute

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Local build time | | | |
| IDE/tooling overhead | | | |
| Dev environment size | | | |

### Runtime Compute

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| CPU efficiency | | | |
| Memory efficiency | | | |
| Idle resource consumption | | | |

---

## 2) Storage Efficiency

### Code Repository

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Repository size | | | |
| Binary assets in repo | | | |
| Build artifact retention | | | |
| Git history bloat | | | |

### Dependencies

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Total dependency size | | | |
| Unused dependencies | | | |
| Duplicate dependencies | | | |
| Dependency tree depth | | | |

### Generated Artifacts

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Build output size | | | |
| Documentation size | | | |
| Test artifacts | | | |

---

## 3) Code Efficiency

### Dead Code

| Type | Amount | Last Used | Action |
|------|--------|-----------|--------|
| Unused functions | | | |
| Unused dependencies | | | |
| Commented-out code | | | |
| Deprecated features | | | |

### Code Bloat

| Area | Lines | Necessary? | Reduction Opportunity |
|------|-------|------------|----------------------|
| | | | |

### Algorithmic Efficiency

| Hot Path | Current Complexity | Optimal | Improvement Opportunity |
|----------|-------------------|---------|------------------------|
| | | | |

---

## 4) Network Efficiency

### API Design

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Average payload size | | | |
| Unnecessary data transfer | | | |
| Compression used | | | |
| Caching effectiveness | | | |

### External Calls

| Service | Calls/Day | Necessary? | Optimization |
|---------|-----------|------------|--------------|
| | | | |

---

## 5) LLM/AI Sustainability

### Token Efficiency

| Metric | Current | Benchmark | Status |
|--------|---------|-----------|--------|
| Avg tokens per request | | | |
| Prompt efficiency | | | |
| Unnecessary re-prompting | | | |
| Cache hit rate | | | |

### Model Selection

| Use Case | Current Model | Right-Sized? | Alternative |
|----------|---------------|--------------|-------------|
| | | | Smaller model? |

### AI Compute Impact

| Metric | Estimate | Reduction Opportunity |
|--------|----------|----------------------|
| Est. CO2 per 1000 requests | | |
| Wasted inference (errors) | | |

---

## 6) Circular Economy Assessment

### Reduce

| What | Can Reduce? | How |
|------|-------------|-----|
| Features | | Remove unused features |
| Code | | Delete dead code |
| Dependencies | | Remove unused deps |
| Infrastructure | | Right-size |

### Reuse

| What | Reusing? | Opportunity |
|------|----------|-------------|
| Existing libraries | | Don't rebuild |
| Internal components | | Share across crates |
| Open source | | Contribute instead of fork |

### Recycle

| What | Recycling? | Opportunity |
|------|------------|-------------|
| Old code | | Refactor vs rewrite |
| Test fixtures | | Reuse across tests |
| Documentation | | Update vs recreate |

---

## 7) Required Output

### A. Sustainability Scorecard

```markdown
## Sustainability Audit - Q[X] [Year]

### Overall Score: [A-F]

### Category Scores
| Category | Score | Trend | Priority Issues |
|----------|-------|-------|-----------------|
| Compute | | ↑/↓/→ | |
| Storage | | ↑/↓/→ | |
| Code | | ↑/↓/→ | |
| Network | | ↑/↓/→ | |
| AI/LLM | | ↑/↓/→ | |
```

### B. Waste Identified

| Waste Type | Amount | Impact | Remediation |
|------------|--------|--------|-------------|
| | | | |

### C. Efficiency Opportunities

| Opportunity | Category | Effort | Savings | Priority |
|-------------|----------|--------|---------|----------|
| | | | | |

### D. Carbon Impact Estimate

```markdown
### Estimated Monthly CO2 Equivalent
- CI/CD: ~X kg
- Development: ~X kg
- Runtime (per user): ~X kg
- LLM usage: ~X kg
- Total: ~X kg

### Comparison
- Equivalent to: [X car miles / X tree-days / etc.]
```

### E. Action Items

| Action | Impact | Effort | Owner | Timeline |
|--------|--------|--------|-------|----------|
| | | | | |

### F. Metrics to Track

| Metric | Current | Target | Track How |
|--------|---------|--------|-----------|
| | | | |

---

## Sustainability Principles

1. **Efficiency first** - Do more with less
2. **Right-size everything** - Don't over-provision
3. **Eliminate waste** - Dead code, unused features, redundant work
4. **Choose wisely** - Smaller models, lighter libraries
5. **Measure to improve** - Track metrics to drive improvement
6. **Consider full lifecycle** - From development to deprecation

---

## Constraints

- Be practical—sustainability shouldn't block progress
- Focus on high-impact, low-effort improvements first
- Use estimates when precise measurement isn't possible
- Consider both direct and indirect environmental impact
- Balance sustainability with other concerns (performance, features)
