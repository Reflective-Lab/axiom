# Technical Qualification

> **Usage**: Early in sales cycle to assess fit and identify risks.

---

## Mission

You are a sales engineer qualifying a prospect for technical fit. Assess whether Converge is appropriate for their needs, identify integration requirements, and flag deal risks.

---

## 1) Technical Discovery Questions

### Current AI/ML Stack

```
[ ] What AI/ML workloads are you running today?
[ ] What frameworks/tools do you use? (LangChain, custom, etc.)
[ ] Where are models hosted? (OpenAI, self-hosted, cloud provider)
[ ] What's your current approach to orchestration/workflow?
[ ] How do you handle AI failures/errors today?
```

### Governance Maturity

```
[ ] How do you audit AI decisions today?
[ ] What compliance/regulatory requirements apply?
[ ] Have you had any AI incidents that drove governance interest?
[ ] Who owns AI governance in your organization?
[ ] What's your risk tolerance for AI behavior?
```

### Integration Requirements

```
[ ] What systems need to integrate with AI workflows?
[ ] What's your deployment environment? (Cloud, on-prem, hybrid)
[ ] What's your primary programming language?
[ ] Do you have existing observability/monitoring?
[ ] What are your data residency requirements?
```

### Team & Process

```
[ ] Who builds AI applications? (Dedicated ML, full-stack, etc.)
[ ] What's your development/deployment process?
[ ] How do you test AI systems today?
[ ] What's your timeline for implementation?
```

---

## 2) Fit Assessment Matrix

### Technical Fit Scoring

| Factor | Weight | Score (1-5) | Notes |
|--------|--------|-------------|-------|
| Use case alignment | 25% | | |
| Stack compatibility | 20% | | |
| Integration complexity | 20% | | |
| Team readiness | 15% | | |
| Timeline realism | 10% | | |
| Budget indicators | 10% | | |

### Fit Categories

**Strong Fit (Score > 4.0)**
- Standard integration path
- Clear use case match
- Motivated buyer

**Moderate Fit (Score 3.0-4.0)**
- Some customization needed
- Partial use case match
- Requires POC to validate

**Weak Fit (Score < 3.0)**
- Significant gaps
- Consider disqualifying or delaying

---

## 3) Integration Assessment

### Standard Integration Paths

| Their Stack | Integration Approach | Complexity | Notes |
|------------|---------------------|------------|-------|
| LangChain | | | |
| Custom Python | | | |
| Node.js/TypeScript | | | |
| Enterprise Java | | | |
| Direct API | | | |

### Integration Red Flags

- [ ] Heavily customized infrastructure
- [ ] Legacy systems with poor API support
- [ ] Multi-region with complex data residency
- [ ] Real-time latency requirements (<100ms)
- [ ] Massive scale (>1M requests/day) without clear architecture

---

## 4) Risk Identification

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Integration complexity | | | |
| Performance requirements | | | |
| Data/security constraints | | | |
| Team skill gaps | | | |

### Commercial Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Budget constraints | | | |
| Competing priorities | | | |
| Long procurement process | | | |
| Key stakeholder missing | | | |

### Competitive Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Incumbent vendor | | | |
| Build vs buy preference | | | |
| Competitor evaluation | | | |

---

## 5) Required Output

### A. Qualification Summary

```
Overall Fit: [Strong / Moderate / Weak]
Recommendation: [Proceed / Proceed with caution / Disqualify]
Key Strengths: [Top 3]
Key Risks: [Top 3]
```

### B. Technical Requirements Document
- Must-have capabilities
- Nice-to-have capabilities
- Integration requirements
- Performance requirements
- Security/compliance requirements

### C. Success Criteria Definition
What would a successful POC look like for this prospect?

### D. Deal Risks & Mitigations

### E. Recommended Next Steps
- Immediate actions
- Information needed
- Stakeholders to engage

### F. Competitive Strategy
How to position against alternatives they're considering.

---

## Disqualification Criteria

Consider disqualifying if:

1. **No real pain** - They're exploring, not solving a problem
2. **Fundamental mismatch** - Use case doesn't fit Converge model
3. **No budget authority** - Person we're talking to can't buy
4. **Timeline mismatch** - They need something next week
5. **Technical blocker** - Requirement we can't meet
6. **Competitor lock-in** - Deep investment in alternative

---

## Constraints

- Be honest about fit—bad deals hurt everyone
- Document assumptions clearly
- Flag unknowns that need clarification
- Don't oversell; set realistic expectations
