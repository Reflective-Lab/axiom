# Capacity Planning

> **Usage**: Quarterly capacity planning and projection review.

---

## Mission

You are planning capacity for Converge. Assess current utilization, project future needs, and ensure infrastructure meets demand with appropriate headroom.

---

## 1) Current Utilization

### Compute Resources

| Resource | Allocated | Used (Avg) | Used (Peak) | Utilization |
|----------|-----------|------------|-------------|-------------|
| CPU | | | | |
| Memory | | | | |
| GPU (if any) | | | | |

### Storage Resources

| Resource | Allocated | Used | Growth Rate | Time to Full |
|----------|-----------|------|-------------|--------------|
| Block storage | | | /month | |
| Object storage | | | /month | |
| Database | | | /month | |

### Network Resources

| Resource | Capacity | Used (Avg) | Used (Peak) | Headroom |
|----------|----------|------------|-------------|----------|
| Bandwidth | | | | |
| Connections | | | | |
| Requests/sec | | | | |

---

## 2) Demand Projections

### Growth Assumptions

| Metric | Current | Growth Rate | Source |
|--------|---------|-------------|--------|
| Users | | /month | |
| Requests | | /month | |
| Data volume | | /month | |
| Features | | /quarter | |

### Projected Demand

| Resource | Now | +3 months | +6 months | +12 months |
|----------|-----|-----------|-----------|------------|
| Compute | | | | |
| Storage | | | | |
| Network | | | | |
| LLM tokens | | | | |

---

## 3) Capacity Requirements

### With Headroom

| Resource | Projected Need | Target Headroom | Required Capacity |
|----------|----------------|-----------------|-------------------|
| CPU | | +50% | |
| Memory | | +50% | |
| Storage | | +30% | |
| Network | | +100% | |

### Scaling Triggers

| Resource | Current | Yellow Alert | Red Alert | Scale Action |
|----------|---------|--------------|-----------|--------------|
| CPU | | 70% | 85% | Add capacity |
| Memory | | 75% | 90% | Add capacity |
| Storage | | 80% | 90% | Expand |
| Connections | | 70% | 85% | Scale out |

---

## 4) Bottleneck Analysis

### Current Bottlenecks

| Bottleneck | Resource | Impact | Mitigation |
|------------|----------|--------|------------|
| | | | |

### Potential Bottlenecks (Future)

| Bottleneck | When | Resource | Mitigation Plan |
|------------|------|----------|-----------------|
| | | | |

### Scaling Limitations

| Component | Max Scale | Constraint | Workaround |
|-----------|-----------|------------|------------|
| | | | |

---

## 5) Cost Projections

### Current Costs

| Component | Monthly Cost | % of Total |
|-----------|--------------|------------|
| Compute | | |
| Storage | | |
| Network | | |
| LLM API | | |
| Other | | |
| **Total** | | 100% |

### Projected Costs

| Component | Now | +3 months | +6 months | +12 months |
|-----------|-----|-----------|-----------|------------|
| Compute | | | | |
| Storage | | | | |
| Network | | | | |
| LLM API | | | | |
| **Total** | | | | |

### Cost Optimization Opportunities

| Opportunity | Current | Optimized | Savings |
|-------------|---------|-----------|---------|
| Reserved instances | | | |
| Right-sizing | | | |
| Spot instances | | | |
| Storage tiering | | | |

---

## 6) Capacity Actions

### Infrastructure Changes

| Change | Reason | Timeline | Cost Impact |
|--------|--------|----------|-------------|
| | | | |

### Architecture Changes

| Change | Reason | Timeline | Capacity Gained |
|--------|--------|----------|-----------------|
| | | | |

### Process Changes

| Change | Reason | Timeline | Impact |
|--------|--------|----------|--------|
| | | | |

---

## 7) Required Output

### A. Capacity Summary

```markdown
## Capacity Planning Report - Q[X] [Year]

### Current State
- Overall utilization: [X]%
- Highest utilized resource: [X] at [Y]%
- Time to capacity constraint: [X months]

### Key Metrics
| Resource | Utilization | Headroom | Status |
|----------|-------------|----------|--------|
| Compute | | | 🟢/🟡/🔴 |
| Storage | | | 🟢/🟡/🔴 |
| Network | | | 🟢/🟡/🔴 |

### Projections
| Timeframe | Capacity Needed | Cost |
|-----------|-----------------|------|
| +3 months | | |
| +6 months | | |
| +12 months | | |
```

### B. Capacity Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| | | | |

### C. Action Plan

| Action | Priority | Timeline | Owner | Cost |
|--------|----------|----------|-------|------|
| | | | | |

### D. Budget Request

| Item | Amount | Justification | Timeline |
|------|--------|---------------|----------|
| | | | |

### E. Monitoring Updates

| Metric | Current Alert | Recommended Alert |
|--------|---------------|-------------------|
| | | |

---

## Capacity Planning Principles

1. **Plan ahead** - 6-12 month horizon minimum
2. **Build headroom** - 50% headroom for compute, 30% for storage
3. **Consider variance** - Plan for peak, not average
4. **Cost-aware** - Balance capacity with cost
5. **Automate** - Auto-scaling where possible
6. **Monitor** - Track utilization trends

---

## Constraints

- Use conservative growth estimates
- Account for seasonal patterns
- Consider lead time for provisioning
- Balance cost and reliability
- Plan for failure scenarios (N+1, N+2)
