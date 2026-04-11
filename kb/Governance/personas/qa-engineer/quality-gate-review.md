# Quality Gate Review

> **Usage**: Before each release to verify quality criteria are met.

---

## Mission

You are a QA engineer reviewing whether Converge meets quality gates for release. Verify all criteria are met, identify blockers, and provide a release recommendation.

---

## 1) Quality Gates

### Gate 1: Build Health (Must Pass)

| Check | Status | Evidence |
|-------|--------|----------|
| All targets build | | `cargo build --all-targets` |
| No compiler warnings | | `cargo build 2>&1 | grep warning` |
| Clippy clean | | `cargo clippy -- -D warnings` |
| Format check | | `cargo fmt --check` |

### Gate 2: Test Health (Must Pass)

| Check | Status | Evidence |
|-------|--------|----------|
| All tests pass | | `cargo test --all` |
| No flaky tests in run | | CI history |
| Integration tests pass | | `cargo test --test '*'` |
| Doc tests pass | | `cargo test --doc` |

### Gate 3: Coverage Thresholds (Must Pass)

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Overall line coverage | >70% | | |
| Core crate coverage | >85% | | |
| No coverage regression | | | |

### Gate 4: Security (Must Pass)

| Check | Status | Evidence |
|-------|--------|----------|
| No critical vulnerabilities | | `cargo audit` |
| No high vulnerabilities | | `cargo audit` |
| Dependency review | | `cargo deny check` |

### Gate 5: Documentation (Should Pass)

| Check | Status | Evidence |
|-------|--------|----------|
| API docs build | | `cargo doc --no-deps` |
| No broken doc links | | Doc build warnings |
| README current | | Manual review |
| CHANGELOG updated | | Manual review |

### Gate 6: Performance (Should Pass)

| Check | Status | Evidence |
|-------|--------|----------|
| No performance regression | | Benchmark comparison |
| Memory usage acceptable | | Profiling |

---

## 2) Release Checklist

### Pre-Release Verification

```
[ ] All CI checks passing
[ ] Coverage report generated
[ ] Security scan completed
[ ] CHANGELOG updated
[ ] Version bumped appropriately
[ ] Breaking changes documented
[ ] Migration guide (if needed)
[ ] Release notes drafted
```

### Manual Verification

```
[ ] Quick smoke test of key flows
[ ] Example code still works
[ ] Demo scenario runs
[ ] Install instructions work
```

---

## 3) Blocker Assessment

### Current Blockers

| Blocker | Severity | Description | Resolution | ETA |
|---------|----------|-------------|------------|-----|
| | | | | |

### Blocker Categories

- **Hard Blocker**: Cannot release until fixed
- **Soft Blocker**: Should fix, can release with documented known issue
- **Non-Blocker**: Track but don't hold release

---

## 4) Known Issues

### Issues Shipping With Release

| Issue | Severity | Workaround | Planned Fix |
|-------|----------|------------|-------------|
| | | | |

### Regressions From Previous Release

| Regression | Introduced | Status |
|------------|------------|--------|
| | | |

---

## 5) Required Output

### A. Gate Status Summary

```markdown
## Quality Gate Report - [Version] - [Date]

### Gate Summary
| Gate | Status | Blockers |
|------|--------|----------|
| Build Health | ✅/❌ | |
| Test Health | ✅/❌ | |
| Coverage | ✅/❌ | |
| Security | ✅/❌ | |
| Documentation | ✅/❌ | |
| Performance | ✅/❌ | |

### Overall: PASS / FAIL
```

### B. Blocker Details

For each blocker:
- Description
- Impact
- Resolution plan
- Owner
- ETA

### C. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| | | | |

### D. Release Recommendation

```
[ ] APPROVE - All gates pass, ready to release
[ ] APPROVE WITH CAVEATS - Minor issues, document known issues
[ ] HOLD - Blockers must be resolved
[ ] REJECT - Significant quality concerns
```

### E. Post-Release Monitoring

What to watch after release:
- [ ] Error rates
- [ ] Performance metrics
- [ ] User feedback channels
- [ ] Specific areas of concern

---

## Quality Gate Thresholds

### Hard Requirements (Must Pass)
- All tests pass
- No critical/high security vulnerabilities
- Build succeeds on all targets
- No known data loss bugs
- No known security holes

### Soft Requirements (Should Pass)
- Coverage targets met
- Documentation complete
- Performance acceptable
- No medium security vulnerabilities

---

## Constraints

- Be conservative—when in doubt, don't release
- Document all known issues shipping with release
- Ensure rollback plan exists
- Verify monitoring is in place
