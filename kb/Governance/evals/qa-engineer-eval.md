---
eval_id: qa-engineer-eval
owner: qa-engineer

intent:
  risk_prevented: quality issues and untested critical paths reaching production
  outcome_ensured: tests pass and critical paths have coverage under current test suite

determinism:
  class: deterministic
  justification: |
    Test execution produces identical results for identical codebase and test suite.
    Coverage measurement and build health checks are deterministic operations.
    Same code + same tests = same pass/fail results.

governance:
  may_block_alone: true
  may_contribute_to_block: true
  eligible_for:
    pr-merge:
      may_contribute: true
      may_block: true
    release-candidate:
      may_contribute: true
      may_block: true
    release-approval:
      may_contribute: true
      may_block: true
    production-deploy:
      may_contribute: true
      may_block: true
source: mixed
---

# QA Engineer Eval

> Quick quality validation. Target: 10 minutes.

## Mission

Perform a rapid quality check of the Converge codebase. Verify tests exist, pass, and cover critical paths. Flag quality blockers only.

---

## Criteria Checklist

### 1. Tests Pass (Critical)

| Check | Command | Pass Criteria |
|-------|---------|---------------|
| All tests pass | `cargo test --all` | Zero failures |
| Doc tests pass | `cargo test --doc` | Zero failures |
| Clippy clean | `cargo clippy -- -D warnings` | Zero warnings |

### 2. Test Coverage (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Unit tests exist | Tests in each crate | Present |
| Integration tests exist | `tests/` directory | Present |
| Critical paths tested | Governance flows | Covered |

**Critical paths that must be tested:**
- [ ] Proposal creation
- [ ] Fact promotion
- [ ] Validation gates
- [ ] Authority checking
- [ ] Audit trail recording

### 3. Test Quality (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Tests are meaningful | Not just "it compiles" | Actual assertions |
| Edge cases covered | Error paths, boundaries | Some coverage |
| No flaky tests | Consistent results | Stable |

### 4. Build Health (Critical)

| Check | Command | Pass Criteria |
|-------|---------|---------------|
| Build succeeds | `cargo build --all-targets` | Zero errors |
| No warnings | Build output | Zero warnings |
| Examples build | `cargo build --examples` | Success |

---

## Output Format

```markdown
# QA Engineer Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Test Failures**: [count]
- **Coverage Gaps**: [count]
- **Run Date**: [date]

## Test Results

| Check | Status | Notes |
|-------|--------|-------|
| All tests pass | ✓/✗ | |
| Doc tests pass | ✓/✗ | |
| Clippy clean | ✓/✗ | |
| Build clean | ✓/✗ | |

## Critical Path Coverage

| Path | Tested | Quality |
|------|--------|---------|
| Proposal creation | ✓/✗ | |
| Fact promotion | ✓/✗ | |
| Validation gates | ✓/✗ | |
| Authority checking | ✓/✗ | |
| Audit trail | ✓/✗ | |

## Issues Found

### Critical (Blocking)
[List any test failures or critical gaps]

### High (Should Fix)
[List quality concerns]

## Verdict

[ ] PASS - Quality sufficient for release
[ ] PARTIAL - Minor gaps, can proceed with tracking
[ ] FAIL - Quality issues block release
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| All tests pass, critical paths covered | PASS |
| All tests pass, some coverage gaps | PARTIAL |
| Any test failures | FAIL |
