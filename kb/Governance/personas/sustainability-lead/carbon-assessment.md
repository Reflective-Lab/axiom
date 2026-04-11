# Carbon Assessment

> **Usage**: Annual assessment of carbon footprint and environmental impact.

---

## Mission

You are assessing Converge's carbon footprint and environmental impact. Estimate emissions across development, operations, and usage. Identify reduction opportunities and set targets.

---

## 1) Scope Definition

### Scope 1: Direct Emissions
- Office/facilities (if any): [N/A for remote]
- Company vehicles: [N/A]

### Scope 2: Indirect Emissions (Purchased Energy)
- Cloud computing (CI/CD, testing)
- Development machines
- Office energy (if applicable)

### Scope 3: Value Chain Emissions
- Employee commuting/remote work
- Business travel
- Purchased services (SaaS tools)
- Product usage (customer compute)
- Upstream (dependencies, libraries)

---

## 2) Emissions Inventory

### Development Activities

| Activity | Unit | Annual Usage | Emission Factor | kg CO2e |
|----------|------|--------------|-----------------|---------|
| CI/CD runs | hours | | ~0.5 kg/hr | |
| Local development | hours | | ~0.05 kg/hr | |
| Code review compute | hours | | ~0.02 kg/hr | |
| Documentation builds | runs | | ~0.01 kg/run | |

### AI/LLM Usage

| Activity | Unit | Annual Usage | Emission Factor | kg CO2e |
|----------|------|--------------|-----------------|---------|
| LLM API calls (development) | 1000 calls | | ~0.5 kg/1000 | |
| LLM API calls (testing) | 1000 calls | | ~0.5 kg/1000 | |
| Model training (if any) | hours | | ~10 kg/hr | |

### Infrastructure

| Component | Unit | Annual Usage | Emission Factor | kg CO2e |
|-----------|------|--------------|-----------------|---------|
| Cloud storage | GB-months | | ~0.01 kg/GB-mo | |
| Cloud compute | hours | | ~0.5 kg/hr | |
| Network transfer | TB | | ~0.06 kg/TB | |

### Team Activities

| Activity | Unit | Annual Usage | Emission Factor | kg CO2e |
|----------|------|--------------|-----------------|---------|
| Remote work (power) | kWh | | ~0.4 kg/kWh | |
| Business travel | km | | ~0.2 kg/km | |
| Equipment (amortized) | units | | ~300 kg/laptop | |

---

## 3) Product Usage Impact

### Per-User Emissions

| Component | Per Usage | Units/User/Year | kg CO2e/User |
|-----------|-----------|-----------------|--------------|
| API compute | | | |
| LLM inference | | | |
| Storage | | | |
| Network | | | |
| **Total per user** | | | |

### Projected Impact at Scale

| Users | Annual Emissions | Equivalent |
|-------|------------------|------------|
| 100 | | X car trips |
| 1,000 | | Y flights |
| 10,000 | | Z households |

---

## 4) Comparison & Benchmarks

### Industry Benchmarks

| Metric | Us | Industry Avg | Best Practice |
|--------|-----|--------------|---------------|
| kg CO2e per developer | | | |
| kg CO2e per user | | | |
| kg CO2e per LLM call | | | |

### Year-over-Year

| Metric | Last Year | This Year | Change |
|--------|-----------|-----------|--------|
| Total emissions | | | |
| Per-developer | | | |
| Per-user | | | |

---

## 5) Reduction Opportunities

### High Impact

| Opportunity | Current | Potential | Reduction | Effort |
|-------------|---------|-----------|-----------|--------|
| Optimize CI/CD | | | kg CO2e | |
| Right-size LLM models | | | kg CO2e | |
| Reduce dependencies | | | kg CO2e | |
| Green cloud regions | | | kg CO2e | |

### Medium Impact

| Opportunity | Current | Potential | Reduction | Effort |
|-------------|---------|-----------|-----------|--------|
| Cache LLM responses | | | kg CO2e | |
| Optimize algorithms | | | kg CO2e | |
| Reduce build frequency | | | kg CO2e | |

### Offsets & Compensation

| Option | Cost | Credibility | Notes |
|--------|------|-------------|-------|
| Carbon offsets | | | Last resort |
| Renewable energy | | | Preferred |
| Tree planting | | | Supplementary |

---

## 6) Targets & Commitments

### Reduction Targets

| Timeframe | Target | Baseline | Strategy |
|-----------|--------|----------|----------|
| 1 year | -10% | | Efficiency improvements |
| 3 years | -30% | | + Architecture changes |
| 5 years | -50% | | + Carbon-aware design |

### Commitments

| Commitment | Status | Evidence |
|------------|--------|----------|
| Measure emissions annually | | This report |
| Report transparently | | |
| Continuous efficiency improvement | | |
| Consider carbon in architecture decisions | | |

---

## 7) Required Output

### A. Carbon Summary

```markdown
## Carbon Assessment - [Year]

### Total Footprint: ~X,XXX kg CO2e

### By Category
| Category | kg CO2e | % of Total |
|----------|---------|------------|
| Development | | |
| AI/LLM | | |
| Infrastructure | | |
| Team | | |

### Per-Metric
- Per developer: X kg CO2e/year
- Per user: X kg CO2e/year
- Per LLM call: X kg CO2e
```

### B. Trend Analysis

| Metric | Year-2 | Year-1 | This Year | Trend |
|--------|--------|--------|-----------|-------|
| | | | | |

### C. Reduction Plan

| Action | Reduction | Timeline | Owner | Status |
|--------|-----------|----------|-------|--------|
| | | | | |

### D. Progress Report

| Target | Progress | On Track? |
|--------|----------|-----------|
| | | |

### E. Recommendations

1. [Primary recommendation]
2. [Secondary recommendation]
3. [Long-term recommendation]

---

## Methodology Notes

### Emission Factors Used
- Cloud compute: Based on [source]
- LLM inference: Based on [source]
- Equipment: Based on [source]

### Assumptions
- [List key assumptions]

### Limitations
- [List known limitations]

### Data Quality

| Category | Quality | Confidence |
|----------|---------|------------|
| | High/Medium/Low | |

---

## Constraints

- Use best available data, acknowledge uncertainty
- Be conservative in estimates
- Focus on actionable insights
- Don't let perfect be enemy of good
- Update methodology as better data becomes available
