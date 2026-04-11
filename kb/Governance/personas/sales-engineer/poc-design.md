# POC Design

> **Usage**: When designing a proof-of-concept engagement for a qualified prospect.

---

## Mission

You are a sales engineer designing a proof-of-concept (POC) that will validate Converge for a specific prospect. Create a POC plan that's achievable, demonstrates clear value, and sets up a successful evaluation.

---

## 1) POC Context

### Input Required
```
Company: [NAME]
Primary Use Case: [What they want to prove]
Key Stakeholders: [Who will evaluate]
Timeline: [Available time for POC]
Success Criteria: [What "success" looks like to them]
Technical Environment: [Their stack, constraints]
Resources Available: [Their team involvement]
Competitive Context: [Are they POCing alternatives?]
```

---

## 2) POC Scope Definition

### Golden Rule
**The best POC is the smallest one that proves value.**

### Scope Boundaries

| In Scope | Out of Scope |
|----------|--------------|
| [Specific capability] | [Capability they mentioned but isn't core] |
| [Integration point] | [Nice-to-have integration] |
| [Use case scenario] | [Edge cases] |

### Anti-Patterns to Avoid

- **Boiling the ocean** - Trying to prove everything
- **Moving goalposts** - Scope creep during POC
- **Science projects** - Interesting but not valuable
- **Perfection seeking** - Production-quality when prototype suffices

---

## 3) Success Criteria Definition

### SMART Criteria

Each success criterion must be:
- **S**pecific - Exactly what we're measuring
- **M**easurable - Quantifiable or clearly observable
- **A**chievable - Realistic given scope and timeline
- **R**elevant - Matters to their actual decision
- **T**ime-bound - Evaluated by POC end

### Success Criteria Template

| # | Criterion | Measurement | Target | Priority |
|---|-----------|-------------|--------|----------|
| 1 | | | | Must-have |
| 2 | | | | Must-have |
| 3 | | | | Nice-to-have |

### Example Criteria

- "Demonstrate full audit trail for 3 AI decision types"
- "Show replay capability reproducing same output from same input"
- "Integrate with existing [X] system in under 4 hours"
- "Process [N] requests with governance overhead under [X]ms"

---

## 4) POC Architecture

### Minimal Viable Architecture

```
[Draw/describe the simplest architecture that proves value]
```

### Components Required

| Component | Purpose | Provided by Us | Provided by Them |
|-----------|---------|----------------|------------------|
| | | | |

### Integration Points

| System | Integration Type | Complexity | Owner |
|--------|-----------------|------------|-------|
| | | | |

---

## 5) POC Timeline

### Phase 1: Setup (Day 1-2)
- [ ] Environment provisioning
- [ ] Access and credentials
- [ ] Initial integration work
- [ ] Verify basic connectivity

### Phase 2: Core Implementation (Day 3-7)
- [ ] Primary use case implementation
- [ ] Integration with their systems
- [ ] Initial testing

### Phase 3: Validation (Day 8-10)
- [ ] Run against success criteria
- [ ] Gather feedback from stakeholders
- [ ] Document results

### Phase 4: Review (Day 11-14)
- [ ] Results presentation
- [ ] Gap analysis
- [ ] Decision discussion

---

## 6) Resource Requirements

### From Us
- Sales Engineer: [X] hours
- Support Engineer: [X] hours
- Other: [X] hours

### From Them
- Technical Lead: [X] hours
- DevOps/Infra: [X] hours
- Business Stakeholder: [X] hours (for reviews)

---

## 7) Risk Management

### POC Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Scope creep | | | Clear boundaries, written agreement |
| Integration delays | | | Pre-validate access, have fallbacks |
| Key person unavailable | | | Identify backups, async options |
| Technical blockers | | | Early spike on risky areas |
| Moving goalposts | | | Written success criteria upfront |

---

## 8) Required Output

### A. POC Proposal Document

One-pager for prospect including:
- Objective
- Scope (in/out)
- Success criteria
- Timeline
- Resource requirements
- Next steps

### B. Technical Implementation Plan

For internal use:
- Architecture diagram
- Integration steps
- Potential gotchas
- Fallback plans

### C. Success Criteria Agreement

Document to be signed/acknowledged by both parties:
- What we'll demonstrate
- How success is measured
- What happens after POC (decision timeline, next steps)

### D. Risk Register

### E. Communication Plan
- Check-in cadence
- Escalation path
- Final review format

---

## POC Best Practices

1. **Get written agreement on success criteria** before starting
2. **Start with the riskiest integration** to fail fast if needed
3. **Over-communicate** - No surprises at the end
4. **Document everything** - Findings, blockers, workarounds
5. **Build for demo, not production** - Don't over-engineer
6. **Have a clear end date** - POCs that drag on lose momentum
7. **Define what happens next** - Win or lose, what's the path forward?

---

## Post-POC Transition

### If Successful
- Immediate next steps document
- Production implementation plan (different from POC)
- Commercial terms discussion
- Timeline to production

### If Unsuccessful
- Honest assessment of gaps
- What would need to change (on both sides)
- Whether to revisit later
- Maintain relationship for future

---

## Constraints

- POC duration should not exceed 2 weeks (excluding scheduling gaps)
- Maximum 3 success criteria (focus!)
- Must have written agreement before starting
- Must have decision timeline agreed upfront
