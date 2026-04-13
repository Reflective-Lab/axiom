---
eval_id: build-vs-buy-eval
owner: build-vs-buy-analyst

intent:
  risk_prevented: reinventing existing solutions and dependency health issues
  outcome_ensured: critical dependencies are healthy; no obvious reinvention of standard libraries

determinism:
  class: deterministic
  justification: |
    Build vs buy decisions are rule-based: check if standard libraries (serde, reqwest,
    ring, etc.) are used instead of custom implementations. Dependency health checks
    against maintainability criteria (last update, maintainer count) are objective.
    Cost analysis is quantitative calculation on dependency counts and sizes.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: false
source: mixed
---

# Build vs Buy Eval

> Quick check for reinventing the wheel. Target: 10 minutes.

## Mission

Perform a rapid check for unnecessary duplication. Verify we're not building things that already exist and our dependencies are healthy.

---

## Criteria Checklist

### 1. Not Reinventing (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No custom parsing | Reinventing serde, nom, etc. | Using libraries |
| No custom HTTP | Reinventing reqwest, hyper | Using libraries |
| No custom crypto | Reinventing ring, rustls | Using libraries |
| No custom date/time | Reinventing chrono | Using libraries |
| No custom logging | Reinventing tracing, log | Using libraries |

**Quick checks:**
```bash
# Look for potential reinvention patterns
grep -r "fn parse" src/ --include="*.rs" | head -20
grep -r "struct HttpClient" src/
grep -r "fn encrypt\|fn decrypt" src/
```

### 2. Dependency Health (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No unmaintained deps | Last update > 12 months | None critical |
| No single-maintainer critical deps | Bus factor = 1 | Awareness |
| No abandoned deps | Archived repos | None |

**Quick check:**
```bash
# Check for outdated dependencies
cargo outdated
```

### 3. Unused Dependencies (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No unused deps | Deps not referenced | Clean |
| No duplicate functionality | Multiple libs doing same thing | Consolidated |

**Quick check:**
```bash
# Check for unused dependencies
cargo +nightly udeps 2>/dev/null || echo "Install with: cargo install cargo-udeps"
```

### 4. Right-Sized Dependencies (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Not over-engineered | Heavy dep for simple task | Appropriate |
| Feature flags used | Only needed features enabled | Optimized |

---

## Output Format

```markdown
# Build vs Buy Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Reinvention Concerns**: [count]
- **Dependency Issues**: [count]
- **Run Date**: [date]

## Reinvention Check

| Area | Status | Notes |
|------|--------|-------|
| Parsing | ✓/✗ | Using serde/nom/etc. |
| HTTP | ✓/✗ | Using reqwest/hyper |
| Crypto | ✓/✗ | Using ring/rustls |
| Date/Time | ✓/✗ | Using chrono |
| Logging | ✓/✗ | Using tracing/log |

## Dependency Health

| Check | Status | Concerns |
|-------|--------|----------|
| All maintained | ✓/✗ | |
| No critical bus-factor-1 | ✓/✗ | |
| No unused | ✓/✗ | |

## Reinvention Concerns

| Custom Code | Should Use | Priority |
|-------------|------------|----------|
| [if any] | | |

## Dependency Concerns

| Dependency | Issue | Action |
|------------|-------|--------|
| [if any] | | |

## Verdict

[ ] PASS - Not reinventing, deps healthy
[ ] PARTIAL - Minor concerns
[ ] FAIL - Significant reinvention or dep issues
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No reinvention, deps healthy | PASS |
| Minor reinvention or dep concerns | PARTIAL |
| Significant custom code that should be libraries | FAIL |
