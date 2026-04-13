---
eval_id: sustainability-eval
owner: sustainability-lead

intent:
  risk_prevented: wasteful resource consumption and environmental impact
  outcome_ensured: basic efficiency checks pass; no obvious waste detected

determinism:
  class: bounded
  justification: |
    Sustainability metrics like build times, binary size, and dependency counts have
    measurable thresholds, but assessment of "waste" involves judgment about what's
    acceptable. Environmental impact models are quantitative but require bounded
    interpretation of what constitutes excessive consumption.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: false
source: mixed
---

# Sustainability Eval

> Quick efficiency/sustainability validation. Target: 10 minutes.

## Mission

Perform a rapid sustainability check. Verify we're not wasting resources and are reasonably efficient.

---

## Criteria Checklist

### 1. No Obvious Waste (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No dead code | Unused functions, files | Minimal |
| No unused dependencies | Deps not used | None |
| No redundant work | Duplicate CI steps, etc. | Minimal |

**Quick checks:**
```bash
# Dead code (basic check)
cargo +nightly udeps 2>/dev/null || echo "Check manually"

# Large files that shouldn't be
find . -size +1M -type f | grep -v target | grep -v ".git"
```

### 2. Reasonable Build Efficiency (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Build time acceptable | Full build duration | <5 min |
| Incremental builds work | Change → rebuild time | <30 sec |
| CI not wasteful | Unnecessary jobs | Minimal |

### 3. Dependency Efficiency (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Not over-dependent | Too many deps | Reasonable |
| Right-sized deps | Heavy dep for light use | Appropriate |
| Feature flags used | Minimize compiled features | Some usage |

**Quick check:**
```bash
# Count dependencies
cargo tree | wc -l

# Check binary size
cargo build --release
ls -lh target/release/[binary]
```

### 4. LLM Efficiency (If Applicable)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Caching used | Repeated calls cached | Yes |
| Right-sized models | GPT-4 for simple tasks? | Appropriate |
| Prompts efficient | Bloated prompts | Reasonable |

---

## Output Format

```markdown
# Sustainability Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Waste Found**: [count items]
- **Efficiency Score**: [1-5]
- **Run Date**: [date]

## Waste Check

| Check | Status | Notes |
|-------|--------|-------|
| No dead code | ✓/✗ | |
| No unused deps | ✓/✗ | |
| No redundant work | ✓/✗ | |

## Efficiency Check

| Metric | Value | Status |
|--------|-------|--------|
| Build time | | ✓/✗ |
| Binary size | | ✓/✗ |
| Dependency count | | ✓/✗ |

## Waste Identified

| Waste | Type | Impact | Action |
|-------|------|--------|--------|
| [if any] | | | |

## Efficiency Opportunities

| Opportunity | Effort | Savings |
|-------------|--------|---------|
| [if any] | | |

## Verdict

[ ] PASS - Reasonably efficient
[ ] PARTIAL - Some waste to address
[ ] FAIL - Significant waste
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No major waste, reasonable efficiency | PASS |
| Some waste identified | PARTIAL |
| Significant waste or inefficiency | FAIL |
