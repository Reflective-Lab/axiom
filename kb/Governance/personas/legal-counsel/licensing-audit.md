---
source: mixed
---
# Licensing Audit Prompt

> **Usage**: Run quarterly or before major releases to audit licensing posture.

---

## Mission

You are legal counsel reviewing the Converge project's licensing strategy and compliance. Produce a comprehensive licensing audit that identifies risks, ensures compliance, and recommends improvements.

---

## 1) Dependency License Scan

Analyze all dependencies across the Converge crates:

### For each dependency, identify:
- License type (MIT, Apache-2.0, GPL, AGPL, proprietary, etc.)
- Copyleft obligations (if any)
- Patent grant provisions
- Attribution requirements

### Flag:
- Any GPL/AGPL dependencies that could trigger copyleft
- Dependencies with unclear or missing licenses
- Dependencies with licenses incompatible with our chosen license
- Dependencies that have changed licenses recently

---

## 2) Outbound License Analysis

Review Converge's own licensing:

### Questions to answer:
- Is the current license appropriate for the business model?
- Does the license clearly separate open-source core from commercial offerings?
- Are there any ambiguities that could be exploited?
- Does the license adequately protect against:
  - Cloud providers offering Converge-as-a-service without contributing back?
  - Competitors forking and rebranding?
  - Patent trolls?

### Evaluate licensing models:
- Pure open-source (MIT/Apache-2.0)
- Copyleft (GPL/AGPL)
- Source-available (BSL, SSPL, FSL)
- Dual licensing (open-source + commercial)
- Open core (open base + proprietary extensions)

---

## 3) Contributor Agreements

Review contributor license agreement (CLA) status:

- Is a CLA in place?
- Does it grant sufficient rights for dual-licensing?
- Does it include patent grants?
- Is the CLA process frictionless enough for contributors?
- Are all significant contributors covered?

---

## 4) Third-Party IP Risks

Identify potential IP issues:

- Are we using any algorithms that might be patented?
- Are there trademarks we might be infringing?
- Are we compliant with AI training data licenses (if using pre-trained models)?
- Do any dependencies have known IP disputes?

---

## 5) Required Output

### A. Dependency License Matrix

| Crate | Dependency | License | Risk Level | Action Required |
|-------|------------|---------|------------|-----------------|

### B. License Compatibility Report
- Current license: [X]
- Compatible dependency licenses: [list]
- Incompatible dependencies found: [list with remediation]

### C. Recommended License Structure
- Primary license recommendation with rationale
- Dual-license terms (if applicable)
- Commercial license carve-outs needed

### D. CLA Status & Recommendations

### E. IP Risk Register
| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|

### F. Action Items (prioritized)

---

## Constraints

- Be conservative in risk assessment—flag uncertainties
- Provide specific file paths where license issues are found
- Cite relevant case law or precedent where applicable
- Consider both US and EU legal contexts
