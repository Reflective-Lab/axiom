---
eval_id: legal-counsel-eval
owner: legal-counsel

intent:
  risk_prevented: legal exposure from non-compliant releases, unreviewed contracts, or regulatory violations
  outcome_ensured: known legal requirements checked against current artifacts; no known violations detected

determinism:
  class: deterministic
  justification: |
    Legal compliance checks are rule-based against local artifacts (contracts, terms,
    policies, regulatory requirements). Same artifacts produce same compliance assessment.
    No external mutable state dependencies.

governance:
  may_block_alone: true
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: true
    content-publish:
      may_contribute: true
      may_block: true
    customer-commitment:
      may_contribute: true
      may_block: true
---

# Legal Counsel Eval

> Quick legal/compliance validation. Target: 10 minutes.

## Mission

Perform a rapid legal check of the Converge codebase and public materials. Verify license compliance, claim accuracy, and regulatory alignment. Flag critical legal risks only.

---

## Criteria Checklist

### 1. License Compliance (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| LICENSE file exists | Root LICENSE file | Present and valid |
| License headers | Source files have license headers (if required) | Consistent |
| Dependency licenses | No GPL/AGPL if incompatible with our license | None incompatible |

**Quick checks:**
```bash
# Check for LICENSE file
ls -la LICENSE*

# Check for problematic licenses in Cargo.lock
cargo tree --format "{l}" | sort | uniq -c | sort -rn

# Look for GPL dependencies
cargo license | grep -i gpl
```

### 2. Third-Party Compliance (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Attribution present | Third-party notices | Complete |
| License compatibility | All deps compatible | Verified |
| No proprietary deps | Proprietary code included | None |

### 3. Claim Accuracy (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| README claims | Claims match reality | Verified |
| "Compliant" claims | No false compliance guarantees | None |
| "Safe" claims | No absolute safety guarantees | Qualified |
| "Deterministic" claims | Appropriately qualified | Honest |

**Review these patterns:**
- "ensures compliance" → should be "supports compliance"
- "eliminates risk" → should be "helps mitigate risk"
- "fully deterministic" → should specify conditions
- "prevents hallucination" → should be "helps detect"

### 4. Regulatory Awareness (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| AI Act considerations | Any claims about EU AI Act compliance | Appropriately qualified |
| No legal advice | Product doesn't give legal advice | Verified |
| Disclaimers present | Appropriate disclaimers in docs | Present |

### 5. IP/Trade Secret (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No third-party code | Copied code without attribution | None found |
| No leaked secrets | API keys, internal URLs | None found |

---

## Output Format

```markdown
# Legal Counsel Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Issues**: [count]
- **High Issues**: [count]
- **Run Date**: [date]

## License Compliance

| Check | Status | Notes |
|-------|--------|-------|
| LICENSE file | ✓/✗ | |
| License headers | ✓/✗ | |
| Dependency licenses | ✓/✗ | |

## Problematic Dependencies

| Dependency | License | Issue | Resolution |
|------------|---------|-------|------------|
| [if any] | | | |

## Claim Review

| Claim | Location | Issue | Recommended Change |
|-------|----------|-------|-------------------|
| [if any] | | | |

## Regulatory Concerns

| Concern | Status | Notes |
|---------|--------|-------|
| AI Act positioning | ✓/✗ | |
| No legal advice given | ✓/✗ | |
| Disclaimers present | ✓/✗ | |

## Issues Found

### Critical
[List with location]

### High
[List with location]

## Verdict

[ ] PASS - No critical legal issues
[ ] PARTIAL - Minor issues to address
[ ] FAIL - Critical legal risk, block release
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Zero critical, zero high | PASS |
| Zero critical, 1-2 high | PARTIAL |
| Any critical issue | FAIL |

**Critical issues:**
- Incompatible license (GPL infecting codebase)
- False compliance claims
- Absolute safety/security guarantees
- Third-party code without attribution

**High issues:**
- Missing license headers
- Overclaiming without qualification
- Missing disclaimers
