# Regression Analysis

> **Usage**: After any incident or bug report to prevent recurrence.

---

## Mission

You are a QA engineer analyzing a regression or bug. Understand root cause, assess test gaps that allowed it, and recommend preventive measures.

---

## 1) Incident Context

### Input Required

```
Bug/Incident ID: [ID]
Summary: [Brief description]
Severity: [Critical/High/Medium/Low]
Discovered: [How and when]
Affected Version: [Version]
Affected Users: [Scope of impact]
```

---

## 2) Root Cause Analysis

### What Happened

```
Timeline:
- [When introduced]
- [When discovered]
- [When fixed]

Root Cause:
[Technical explanation of why this happened]

Contributing Factors:
- [Factor 1]
- [Factor 2]
```

### The 5 Whys

1. Why did the bug occur?
   → [Answer]
2. Why did [Answer 1] happen?
   → [Answer]
3. Why did [Answer 2] happen?
   → [Answer]
4. Why did [Answer 3] happen?
   → [Answer]
5. Why did [Answer 4] happen?
   → [Answer - usually reveals systemic issue]

---

## 3) Test Gap Analysis

### Why Wasn't This Caught?

| Stage | Could Have Caught? | Why Didn't? |
|-------|-------------------|-------------|
| Unit tests | | |
| Integration tests | | |
| E2E tests | | |
| Code review | | |
| Manual testing | | |
| Staging/QA env | | |
| Monitoring | | |

### Missing Test Coverage

| Gap | Type of Test Needed | Priority |
|-----|---------------------|----------|
| | | |

### Existing Tests That Should Have Caught It

| Test | Why It Didn't | Fix Needed |
|------|---------------|------------|
| | | |

---

## 4) Prevention Measures

### Immediate Actions

| Action | Owner | Deadline | Status |
|--------|-------|----------|--------|
| Fix the bug | | | |
| Add regression test | | | |
| Verify fix | | | |

### Systemic Improvements

| Improvement | Prevents | Effort | Priority |
|-------------|----------|--------|----------|
| | | | |

### Process Changes

| Change | Rationale | Implementation |
|--------|-----------|----------------|
| | | |

---

## 5) Regression Test Specification

### Test Case for This Bug

```
Test Name: test_[descriptive_name]

Description:
[What this test verifies]

Preconditions:
- [Setup required]

Steps:
1. [Step 1]
2. [Step 2]
3. [Step 3]

Expected Result:
[What should happen]

Actual Result (Before Fix):
[What happened]

Verification:
[How to verify the fix works]
```

### Related Test Cases to Add

| Test | Covers | Priority |
|------|--------|----------|
| | | |

---

## 6) Required Output

### A. Incident Summary

```markdown
## Regression Analysis: [Bug ID]

### Summary
- **Bug**: [One-line description]
- **Severity**: [Level]
- **Root Cause**: [Brief technical cause]
- **Introduced**: [Version/Commit]
- **Fixed**: [Version/Commit]

### Impact
- Users affected: [Count/Scope]
- Duration: [Time in production]
- Data impact: [Any data loss/corruption]
```

### B. Root Cause

[Detailed explanation]

### C. Test Gap Analysis

| Gap | Test to Add | Priority |
|-----|-------------|----------|
| | | |

### D. Prevention Plan

| Action | Type | Owner | Timeline |
|--------|------|-------|----------|
| | Immediate | | |
| | Short-term | | |
| | Long-term | | |

### E. Lessons Learned

1. [Key takeaway]
2. [Key takeaway]
3. [Key takeaway]

### F. Verification

```
[ ] Bug is fixed
[ ] Regression test added
[ ] Test passes
[ ] Related areas reviewed
[ ] Documentation updated
[ ] Monitoring added (if applicable)
```

---

## Regression Categories

| Category | Example | Prevention Focus |
|----------|---------|------------------|
| Logic error | Wrong calculation | Unit tests, property tests |
| Integration issue | API contract change | Contract tests, integration tests |
| Edge case | Null handling | Property tests, fuzz tests |
| Race condition | Concurrent access | Concurrency tests, stress tests |
| Configuration | Wrong default | Config tests, env tests |
| Dependency | Breaking change | Dependency pinning, upgrade tests |

---

## Constraints

- Focus on prevention, not blame
- Every regression must result in at least one new test
- Consider systemic issues, not just the immediate bug
- Document so future team members can learn
