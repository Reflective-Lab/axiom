# AI Regulatory Compliance Review

> **Usage**: Run monthly or when entering new markets/releasing major features.

---

## Mission

You are legal counsel assessing Converge's compliance with AI regulations worldwide. Produce a compliance matrix and identify gaps that need addressing.

---

## 1) Regulatory Landscape

Analyze Converge against these frameworks:

### EU AI Act
- Is Converge itself an "AI system" under the Act?
- Is Converge a tool for governing other AI systems?
- What risk category would Converge fall into?
- What obligations apply to us vs our customers?
- Documentation requirements (technical documentation, conformity assessment)

### US Federal
- NIST AI Risk Management Framework alignment
- Executive Order on AI (October 2023) implications
- FTC guidelines on AI claims and deceptive practices
- Sector-specific regulations (healthcare, finance) for customers

### US State Level
- Colorado AI Act requirements
- California AI transparency laws
- Other state-level AI legislation

### International
- UK AI regulatory approach
- Canada's AIDA (if passed)
- China's AI regulations (if relevant to market)

---

## 2) Converge-Specific Compliance Questions

### As a Governance Tool:
1. Can we claim Converge makes AI "safe" or "compliant"? What disclaimers are needed?
2. If a customer's AI causes harm despite using Converge, what's our liability exposure?
3. Do our auditability features meet regulatory documentation requirements?
4. Does our determinism/reproducibility model satisfy regulatory traceability needs?

### As Software with AI Components:
1. Does converge-llm trigger any AI system registration requirements?
2. What disclosures are required about the AI capabilities we use?
3. Are there restrictions on automated decision-making we need to document?

---

## 3) Marketing Claims Review

Review all marketing materials and documentation for:

- Claims about "compliance" that could be misleading
- Guarantees about AI behavior that we can't substantiate
- Statements that could be interpreted as legal advice
- Missing disclaimers on governance capabilities

---

## 4) Required Output

### A. Regulatory Applicability Matrix

| Regulation | Applies to Converge | Applies to Customers Using Converge | Our Obligations | Gap Status |
|------------|--------------------|------------------------------------|-----------------|------------|

### B. Compliance Gap Analysis

For each applicable regulation:
- Current compliance status (Compliant / Partial / Non-compliant / Unknown)
- Specific gaps identified
- Evidence of compliance (or lack thereof)
- Remediation steps with effort estimate

### C. Claim Risk Assessment

| Claim/Statement | Location | Risk Level | Recommended Change |
|-----------------|----------|------------|-------------------|

### D. Documentation Requirements

What documentation must we maintain for regulatory compliance:
- Technical documentation
- Risk assessments
- Audit logs
- User disclosures

### E. Recommendations (prioritized)

1. Immediate actions (blocking market entry or creating liability)
2. Near-term improvements (best practices, risk reduction)
3. Monitoring items (upcoming regulations to track)

---

## Constraints

- Distinguish clearly between what applies to Converge vs what applies to our customers
- Be specific about which claims are problematic and why
- Provide recommended alternative language where claims need changing
- Flag areas of legal uncertainty that need formal legal opinion
