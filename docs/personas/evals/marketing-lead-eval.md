---
eval_id: marketing-lead-eval
owner: marketing-lead

intent:
  risk_prevented: incomplete or misleading marketing artifacts entering public channels
  outcome_ensured: required marketing artifacts present and internally consistent with product capabilities

determinism:
  class: bounded
  justification: |
    Marketing accuracy checks involve bounded interpretation of claims against capabilities.
    Output varies based on content complexity but failure modes are known: unsupported claims,
    inconsistent messaging, missing disclaimers. Escalates to legal for borderline cases.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    content-publish:
      may_contribute: true
      may_block: true
    release-approval:
      may_contribute: true
      may_block: true
---

# Marketing Lead Eval

> Quick marketing/messaging validation. Target: 10 minutes.

## Mission

Perform a rapid messaging check. Verify positioning is consistent, claims are accurate, and differentiation is clear. Flag critical messaging issues only.

---

## Criteria Checklist

### 1. Positioning Clarity (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Clear value prop | What Converge does in one sentence | Present |
| Target audience | Who it's for is clear | Stated |
| Category position | Where we fit in market | Defined |

**Check README for:**
- One-liner description
- "For [audience] who [need]" statement
- Category (AI governance, agent framework, etc.)

### 2. Differentiation (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| vs Competitors | How we're different | Clear |
| Unique value | What only we do | Articulated |
| Not "better" | Specific claims, not superlatives | Verified |

**Should NOT see:**
- "Best AI framework"
- "Most powerful"
- Vague differentiators

**Should see:**
- Specific capabilities (governance, determinism, audit)
- Concrete differences from alternatives

### 3. Claim Consistency (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| README matches docs | Claims are consistent | Consistent |
| Docs match code | Features actually exist | Verified |
| No overclaiming | Claims are honest | Verified |

### 4. Messaging Patterns (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Governance focus | Governance is central message | Consistent |
| Technical accuracy | Claims match architecture | Verified |
| Avoid buzzwords | Not just "AI" soup | Substantive |

**Key messages to verify:**
- [ ] Governance by construction
- [ ] Proposal/Fact distinction
- [ ] Auditability/traceability
- [ ] Determinism capabilities

### 5. Call to Action (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Next steps clear | What to do after reading | Present |
| CTA present | Action for reader | Clear |

---

## Output Format

```markdown
# Marketing Lead Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Issues**: [count]
- **High Issues**: [count]
- **Run Date**: [date]

## Positioning Check

| Check | Status | Notes |
|-------|--------|-------|
| Clear value prop | ✓/✗ | |
| Target audience | ✓/✗ | |
| Category position | ✓/✗ | |

## Current Positioning Statement
> [Capture current one-liner/positioning]

## Differentiation

| Check | Status | Notes |
|-------|--------|-------|
| vs Competitors clear | ✓/✗ | |
| Unique value stated | ✓/✗ | |
| Specific (not superlative) | ✓/✗ | |

## Claim Consistency

| Claim | README | Docs | Code | Status |
|-------|--------|------|------|--------|
| Governance | | | | ✓/✗ |
| Determinism | | | | ✓/✗ |
| Auditability | | | | ✓/✗ |

## Messaging Issues

| Issue | Location | Concern | Fix |
|-------|----------|---------|-----|
| [if any] | | | |

## Verdict

[ ] PASS - Messaging is clear and accurate
[ ] PARTIAL - Minor inconsistencies
[ ] FAIL - Significant messaging problems
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Clear positioning, consistent claims | PASS |
| Minor inconsistencies or gaps | PARTIAL |
| Unclear positioning or false claims | FAIL |

**Critical issues:**
- No clear value proposition
- Claims don't match code capabilities
- Inconsistent messaging across materials

**High issues:**
- Vague differentiation
- Missing target audience
- Buzzword-heavy, substance-light
