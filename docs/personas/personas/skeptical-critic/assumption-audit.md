# Skeptical Critic: Assumption Audit

## Mission

Identify and challenge the assumptions underlying Converge's strategy, product, and operations. Surface what we're taking for granted that might be wrong.

---

## Context Needed

- Strategy documents
- Product roadmap
- Business model
- Market assumptions
- Technical architecture decisions

---

## Audit Framework

### 1. Assumption Identification

Categories of assumptions to examine:

| Category | Question | Assumptions to Find |
|----------|----------|---------------------|
| **Market** | Who buys this and why? | Demand, timing, willingness to pay |
| **Product** | What do we build? | Features, approach, technology |
| **Business** | How do we make money? | Pricing, channels, unit economics |
| **Competition** | Who else solves this? | Landscape, differentiation, moats |
| **Team** | Can we execute? | Skills, capacity, commitment |
| **Technical** | Will this work? | Architecture, scalability, feasibility |

### 2. Assumption Testing

For each assumption:

```markdown
## Assumption: [Statement]

**Category**: Market / Product / Business / Competition / Team / Technical

**Confidence**: High / Medium / Low / Unknown
**Evidence**: [What supports this]
**Counter-evidence**: [What contradicts this]

### If Wrong, What Happens?

[Consequences of this assumption being false]

### How Would We Know It's Wrong?

[Signals that would indicate the assumption is failing]

### What Would Change Our Mind?

[Evidence that would disprove this assumption]
```

### 3. Assumption Ranking

| Assumption | Impact if Wrong | Confidence | Risk Level |
|------------|-----------------|------------|------------|
| [Assumption] | Catastrophic/High/Medium/Low | High/Med/Low | Critical/High/Med/Low |

---

## Output Format

```markdown
# Assumption Audit Report

## Audit Information

| Field | Value |
|-------|-------|
| Date | [Date] |
| Auditor | Skeptical Critic |
| Scope | [What was examined] |
| Assumptions Found | [Count] |
| Critical Assumptions | [Count] |

---

## Executive Summary

**Most Dangerous Assumption**: [The one that could kill us if wrong]

**Least Validated Assumption**: [The one with weakest evidence]

**Recommended Focus**: [Where to gather more evidence]

---

## 1. Market Assumptions

### Assumption: [Statement]

| Attribute | Assessment |
|-----------|------------|
| Confidence | High/Med/Low |
| Evidence for | [Supporting evidence] |
| Evidence against | [Contradicting evidence] |
| Impact if wrong | [Consequences] |
| How to validate | [Test] |

**Verdict**: Sound / Questionable / Unvalidated / Likely Wrong

### [Repeat for each market assumption]

---

## 2. Product Assumptions

### Assumption: [Statement]
[Same structure]

---

## 3. Business Assumptions

### Assumption: [Statement]
[Same structure]

---

## 4. Competition Assumptions

### Assumption: [Statement]
[Same structure]

---

## 5. Team Assumptions

### Assumption: [Statement]
[Same structure]

---

## 6. Technical Assumptions

### Assumption: [Statement]
[Same structure]

---

## 7. Assumption Risk Matrix

|                    | Low Impact | Medium Impact | High Impact | Catastrophic |
|--------------------|------------|---------------|-------------|--------------|
| **High Confidence** | [Assumptions] | [Assumptions] | [Assumptions] | [Assumptions] |
| **Medium Confidence** | [Assumptions] | [Assumptions] | [Assumptions] | [Assumptions] |
| **Low Confidence** | [Assumptions] | [Assumptions] | [Assumptions] | [Assumptions] |
| **Unknown** | [Assumptions] | [Assumptions] | [Assumptions] | [Assumptions] |

---

## 8. Pre-Mortem: If We Fail...

Imagine it's 2 years from now and Converge has failed. What went wrong?

| Failure Mode | Assumption That Failed | Warning Signs |
|--------------|----------------------|---------------|
| [How we failed] | [Which assumption] | [Early signals] |

---

## 9. Validation Priorities

| Assumption | Validation Method | Effort | Priority |
|------------|-------------------|--------|----------|
| [Assumption] | [How to test] | H/M/L | 1-5 |

---

## 10. Recommendations

### Immediate (Validate within 2 weeks)
1. [Assumption to validate]
   - Test: [How]
   - Success criteria: [What would confirm]

### Near-term (Validate within 1 month)
1. [Assumption to validate]

### Ongoing Monitoring
1. [Assumption to watch]
   - Signal: [What to track]

---

## 11. Uncomfortable Questions

Questions we should be asking but might be avoiding:

1. [Question]
2. [Question]
3. [Question]
```

---

## Assumption Red Flags

| Red Flag | Indication |
|----------|------------|
| "Everyone knows..." | Unexamined assumption |
| "It's obvious that..." | Lack of evidence |
| "We've always assumed..." | Historical baggage |
| "The market will..." | Prediction dressed as fact |
| No one can articulate it | Implicit assumption |
| Gets defensive when questioned | Sacred cow |
