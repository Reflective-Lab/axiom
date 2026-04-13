---
eval_id: security-auditor-eval
owner: security-auditor

intent:
  risk_prevented: deployment with known critical vulnerabilities or authority model violations
  outcome_ensured: no known critical vulnerabilities or authority bypasses detected under current checks

determinism:
  class: deterministic
  justification: |
    Static code pattern matching for security vulnerabilities and authority model checks
    produce identical results for identical codebase state. Pattern-based vulnerability
    detection and type-based authority validation are deterministic operations.

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

# Security Auditor Eval

> Quick security validation. Target: 10-15 minutes.

## Mission

Perform a rapid security check of the Converge codebase. Verify authority model, audit integrity, and absence of common vulnerability patterns. Flag critical security issues only.

---

## Criteria Checklist

### 1. Authority Model (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No implicit authority | Default authority values | None found |
| Authority is parameter | Authority passed explicitly to functions | Consistent pattern |
| No authority bypass | Direct fact creation | Impossible by types |

**Quick checks:**
```bash
# Should NOT find implicit authority patterns:
grep -r "Authority::default()" src/
grep -r "impl Default for Authority" src/
```

### 2. Proposal/Fact Boundary (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Type separation | Proposal<T> and Fact<T> are distinct | Verified |
| Promotion required | Fact::new() doesn't exist or is private | Verified |
| Validation enforced | Promotion requires ValidationReport | Verified |

### 3. Audit Integrity (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Append-only | No mutation methods on trace types | None found |
| Immutable storage | Trace storage has no update/delete | Verified |
| Hash integrity | Traces include content hashes | Present |

### 4. Input Validation (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| External input validated | User/API input sanitized | Verified |
| LLM output not trusted | Proposals validated before use | Verified |
| No direct execution | LLM output not eval'd/exec'd | None found |

**Quick checks:**
```bash
# Should NOT find direct execution of strings:
grep -r "eval(" src/
grep -r "exec(" src/
grep -r "Command::new" src/ # Review each usage
```

### 5. Secrets Handling (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No hardcoded secrets | API keys, passwords in code | None found |
| Env vars for secrets | Secrets from environment | Verified |
| No secret logging | Secrets in log output | None found |

**Quick checks:**
```bash
# Should NOT find hardcoded secrets:
grep -ri "api_key\s*=" src/
grep -ri "password\s*=" src/
grep -ri "secret\s*=" src/
```

### 6. Error Handling (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| No sensitive data in errors | Stack traces, internal paths | Sanitized |
| Fail secure | Errors don't open access | Verified |

---

## Output Format

```markdown
# Security Auditor Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Issues**: [count]
- **High Issues**: [count]
- **Medium Issues**: [count]
- **Run Date**: [date]

## Authority Model

| Check | Status | Finding |
|-------|--------|---------|
| No implicit authority | ✓/✗ | |
| Authority as parameter | ✓/✗ | |
| No authority bypass | ✓/✗ | |

## Proposal/Fact Boundary

| Check | Status | Finding |
|-------|--------|---------|
| Type separation | ✓/✗ | |
| Promotion required | ✓/✗ | |
| Validation enforced | ✓/✗ | |

## Audit Integrity

| Check | Status | Finding |
|-------|--------|---------|
| Append-only | ✓/✗ | |
| Immutable storage | ✓/✗ | |
| Hash integrity | ✓/✗ | |

## Input Validation

| Check | Status | Finding |
|-------|--------|---------|
| External input validated | ✓/✗ | |
| LLM output validated | ✓/✗ | |
| No direct execution | ✓/✗ | |

## Secrets Handling

| Check | Status | Finding |
|-------|--------|---------|
| No hardcoded secrets | ✓/✗ | |
| Env vars for secrets | ✓/✗ | |
| No secret logging | ✓/✗ | |

## Issues Found

### Critical
[List with file:line]

### High
[List with file:line]

### Medium
[List with file:line]

## Verdict

[ ] PASS - No critical security issues
[ ] PARTIAL - Non-critical issues found
[ ] FAIL - Critical security issues, block release
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Zero critical, zero high | PASS |
| Zero critical, 1-2 high | PARTIAL |
| Any critical issue | FAIL |

**Critical issues:**
- Authority bypass possible
- Fact creation without validation
- Audit tampering possible
- Hardcoded production secrets

**High issues:**
- Missing input validation on external boundary
- LLM output used without validation
- Secrets in logs
