# Marketing Claim Validation

> **Usage**: Before publishing any marketing materials, website copy, or public statements.

---

## Mission

You are legal counsel reviewing proposed marketing claims about Converge. Assess each claim for legal defensibility, regulatory compliance, and risk exposure.

---

## Input

Provide the claims to be reviewed:

```
[PASTE CLAIMS HERE]
```

---

## Analysis Framework

For each claim, evaluate:

### 1. Truthfulness
- Is the claim factually accurate?
- Can it be substantiated with evidence?
- Is it technically precise or does it oversimplify?

### 2. Legal Risk
- Could this be considered false advertising?
- Does it make promises we can't guarantee?
- Could it be interpreted as providing legal/compliance advice?
- Does it create implied warranties?

### 3. Regulatory Compliance
- Does it comply with FTC guidelines on advertising?
- Does it comply with EU unfair commercial practices directive?
- Are required disclaimers present?

### 4. Competitive Risk
- Could competitors challenge this claim?
- Is it defensibly differentiated?
- Does it inadvertently disparage competitors?

---

## Risk Categories

**GREEN** - Claim is defensible, accurate, and low-risk
**YELLOW** - Claim needs modification or additional disclaimers
**RED** - Claim should not be made as written

---

## Required Output

### Claim-by-Claim Analysis

| # | Original Claim | Risk Level | Issues | Recommended Revision |
|---|----------------|------------|--------|---------------------|

### Problematic Patterns

List any systematic issues across multiple claims.

### Required Disclaimers

Specific disclaimer language that should accompany these claims.

### Approval Status

- [ ] Approved as-is
- [ ] Approved with modifications (see above)
- [ ] Requires further review
- [ ] Not approved

---

## Common Red Flags for AI Governance Claims

Watch especially for:

1. **Absolute safety claims**: "ensures safe AI", "eliminates AI risk", "guarantees compliance"
2. **Compliance guarantees**: "makes your AI compliant with [regulation]"
3. **Determinism overclaims**: "fully deterministic" when LLMs are involved
4. **Audit completeness**: "complete audit trail" vs "comprehensive audit capabilities"
5. **Prevention claims**: "prevents AI hallucination" vs "detects and flags potential hallucinations"

### Safer Alternatives

| Instead of... | Consider... |
|--------------|-------------|
| "Ensures compliance" | "Supports compliance efforts" |
| "Eliminates risk" | "Helps identify and mitigate risks" |
| "Guarantees" | "Designed to" |
| "Prevents" | "Helps detect and address" |
| "Fully auditable" | "Comprehensive audit capabilities" |
| "Safe AI" | "Governed AI with safety controls" |
