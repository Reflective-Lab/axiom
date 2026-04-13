---
source: mixed
---
# Testing Phase Contracts

Responsibilities during testing and quality validation.

---

## QA Engineer

**Role**: Accountable for quality gates

### Responsibilities
- Execute test plans
- Verify test coverage meets targets
- Identify and document bugs
- Manage regression testing
- Sign off on quality readiness

### Gates
- [ ] All tests pass
- [ ] Coverage targets met
- [ ] No P0/P1 bugs open
- [ ] Regression suite passes

### Deliverables
- Test execution report
- Bug reports
- Coverage report
- Quality sign-off

### Collaboration
- **With Developer**: Bug triage and fixes
- **With Security Auditor**: Security test coordination
- **With SRE**: Integration/performance testing

---

## Security Auditor

**Role**: Responsible for security testing

### Responsibilities
- Execute security tests
- Run vulnerability scans
- Verify security requirements
- Test authentication/authorization
- Check for OWASP Top 10

### Gates
- [ ] Security tests pass
- [ ] No critical/high vulnerabilities
- [ ] Dependency audit clean

### Deliverables
- Security test results
- Vulnerability scan report
- Security sign-off

### Collaboration
- **With QA Engineer**: Security test integration
- **With System Architect**: Security design validation

---

## System Architect

**Role**: Consulted for architecture validation

### Responsibilities
- Validate architecture in practice
- Review integration test results
- Confirm design assumptions hold

### Gates
- [ ] Architecture holds under test

### Deliverables
- Architecture validation feedback

### Collaboration
- **With QA Engineer**: Test result analysis
- **With SRE**: Performance assessment

---

## SRE / Operations

**Role**: Responsible for operational testing

### Responsibilities
- Performance testing
- Load testing
- Observability verification
- Deployment testing

### Gates
- [ ] Performance meets requirements
- [ ] Monitoring works
- [ ] Deployment pipeline tested

### Deliverables
- Performance test results
- Operational readiness assessment

### Collaboration
- **With QA Engineer**: Test coordination
- **With System Architect**: Performance analysis

---

## Ethics & Safety Officer

**Role**: Consulted for safety testing

### Responsibilities
- Review test scenarios for harm potential
- Verify safety controls work
- Check for unintended behaviors

### Gates
- [ ] Safety controls verified

### Deliverables
- Safety verification feedback

### Collaboration
- **With QA Engineer**: Safety test scenarios
- **With Security Auditor**: Security vs safety overlap

---

## Skeptical Critic

**Role**: Consulted for edge case identification

### Responsibilities
- Challenge test assumptions
- Identify untested scenarios
- Ask "what if" questions

### Gates
- [ ] Edge cases considered

### Deliverables
- Edge case suggestions
- Challenge feedback

### Collaboration
- **With QA Engineer**: Edge case testing

---

## Phase Completion Checklist

Before moving to Review phase:

```markdown
## Testing Phase Sign-off

- [ ] All tests pass (QA Engineer)
- [ ] Coverage targets met (QA Engineer)
- [ ] Security tests pass (Security Auditor)
- [ ] No critical vulnerabilities (Security Auditor)
- [ ] Performance acceptable (SRE)
- [ ] Safety controls verified (Ethics & Safety)

Quality Gate: PASS / FAIL

Approved by: _______________
Date: _______________
```
