# Discovery Brief — [Customer Company Name]

> One-page summary from the discovery call. Used to decide whether to offer a pilot.

**Date:** [YYYY-MM-DD]
**Prepared by:** Leo Marin, Solutions Engineer
**Discovery call attendees:** [Names and roles from both sides]

---

## 1. Company Overview

| Field | Detail |
|-------|--------|
| **Company** | [Name] |
| **Industry** | [e.g., B2B SaaS, Professional Services, Financial Services, Logistics] |
| **Size** | [Employees, ARR range if known] |
| **Decision maker** | [Name, Title] |
| **Technical lead** | [Name, Title] |
| **Pilot lead (day-to-day)** | [Name, Title] |

---

## 2. Pain & Workflow

### What process are they trying to improve?

[Describe the specific workflow in their words. Not "general AI" — a concrete process.]

### Current workflow (end-to-end)

| Step | Actor | Tool/System | Pain Point |
|------|-------|-------------|------------|
| 1 | [Who] | [What system] | [What's broken/slow] |
| 2 | | | |
| 3 | | | |

### What does "success" look like in their words?

> [Direct quote or close paraphrase from the decision maker]

---

## 3. Technical Fit

| Question | Answer |
|----------|--------|
| Systems involved | [CRM, ERP, email, Slack, internal tools — list all] |
| APIs available? | [Yes/No/Partial — which systems have APIs?] |
| Data formats | [JSON, CSV, proprietary — what are we dealing with?] |
| Auth model | [OAuth, API key, SSO — how do their systems authenticate?] |
| Compliance requirements | [SOC 2, HIPAA, GDPR, industry-specific — list all] |
| Data sensitivity | [PII involved? Financial data? Customer data?] |

### Converge capability match

| Their need | Converge capability | Status |
|------------|-------------------|--------|
| [Need 1] | [Which crate/feature] | [Available / Wave N / Not planned] |
| [Need 2] | | |

### Blockers or gaps

- [Any capability gaps, missing integrations, or Wave 2+ dependencies]

---

## 4. Commercial Context

| Field | Detail |
|-------|--------|
| **Timeline** | [Urgent / This quarter / Exploratory] |
| **Budget range** | [If shared — otherwise "Not discussed"] |
| **Procurement process** | [Who approves? How long does it take?] |
| **Competitive alternatives** | [What else are they evaluating? Build vs buy?] |
| **Expected contract value** | [Leo's estimate based on tier fit] |

---

## 5. Qualification Decision

### Fit score

| Criterion | Score (1-5) | Notes |
|-----------|-------------|-------|
| Use case fits current capabilities | | |
| Baseline data is measurable | | |
| Decision maker is engaged | | |
| Integration complexity is manageable | | |
| Timeline aligns with our roadmap | | |
| **Total** | **/25** | |

### Disqualifier check

- [ ] Use case requires crates shipping 8+ weeks out — **STOP**
- [ ] Customer cannot provide baseline data — **STOP**
- [ ] Requires on-prem deployment — **STOP**
- [ ] Data compliance exceeds our current posture — **STOP**
- [ ] No clear decision maker or decision maker not engaged — **STOP**

### Recommendation

**Offer pilot?** Yes / No

**Rationale:** [2-3 sentences on why this is or isn't a good fit]

**Estimated pilot cost to Converge:** [Engineering hours, infra, Leo's time]

---

## 6. Next Steps

- [ ] [Action item 1 — owner, deadline]
- [ ] [Action item 2 — owner, deadline]
- [ ] [Action item 3 — owner, deadline]

---

*Share this brief with Ren Akiyama (technical assessment) and Priya Chandran (pilot economics) before offering a pilot.*
