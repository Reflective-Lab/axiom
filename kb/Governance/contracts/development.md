# Development Phase Contracts

Responsibilities during feature development and implementation.

---

## System Architect

**Role**: Accountable for architecture decisions

### Responsibilities
- Review architectural implications of new features
- Ensure layer boundaries are maintained
- Validate adherence to Converge axioms
- Approve significant design decisions

### Gates
- [ ] Architecture review before implementation begins
- [ ] Design document for significant features
- [ ] Layer boundary check before PR merge

### Deliverables
- Architecture decision records (ADRs)
- Design review feedback
- Technical guidance

### Collaboration
- **With QA Engineer**: Ensure testability of designs
- **With Security Auditor**: Security architecture review
- **With Build vs Buy**: Dependency decisions

---

## QA Engineer

**Role**: Consulted on testability, Responsible for test creation

### Responsibilities
- Review designs for testability
- Write test plans for new features
- Create tests alongside development
- Maintain test infrastructure

### Gates
- [ ] Test plan exists before development complete
- [ ] Tests written before PR merge
- [ ] CI pipeline passes

### Deliverables
- Test plans
- Unit and integration tests
- Test coverage reports

### Collaboration
- **With System Architect**: Testability feedback
- **With Developer**: Pair on test writing
- **With Security Auditor**: Security test coverage

---

## Security Auditor

**Role**: Consulted for security review

### Responsibilities
- Review security implications of new features
- Identify potential vulnerabilities in design
- Advise on secure implementation patterns
- Review authentication/authorization changes

### Gates
- [ ] Security review for sensitive features
- [ ] No new high/critical vulnerabilities introduced

### Deliverables
- Security review feedback
- Vulnerability identification
- Secure coding guidance

### Collaboration
- **With System Architect**: Security architecture
- **With QA Engineer**: Security test requirements

---

## Build vs Buy Analyst

**Role**: Consulted for dependency decisions

### Responsibilities
- Review any new dependencies
- Assess build vs buy decisions
- Check for existing solutions before building
- Evaluate dependency health

### Gates
- [ ] New dependencies reviewed and approved
- [ ] No reinventing the wheel without justification

### Deliverables
- Dependency decision records
- Build vs buy recommendations

### Collaboration
- **With System Architect**: Technical fit
- **With Security Auditor**: Dependency security
- **With Legal Counsel**: License compatibility

---

## Ethics & Safety Officer

**Role**: Consulted for feature ethics

### Responsibilities
- Review features for ethical implications
- Assess potential for misuse
- Ensure responsible AI practices in development
- Flag features needing ethics review

### Gates
- [ ] Ethics review for features affecting users
- [ ] Harm assessment for significant features

### Deliverables
- Ethics review feedback
- Harm assessment (if needed)

### Collaboration
- **With System Architect**: Design implications
- **With Founder**: Strategic ethics alignment

---

## Sustainability Lead

**Role**: Consulted for efficiency

### Responsibilities
- Review features for resource efficiency
- Flag wasteful patterns
- Advise on efficient implementation

### Gates
- [ ] No significant resource waste introduced

### Deliverables
- Efficiency feedback

### Collaboration
- **With System Architect**: Efficient design
- **With Build vs Buy**: Dependency efficiency

---

## Developer Advocate

**Role**: Consulted for developer experience

### Responsibilities
- Review APIs for developer ergonomics
- Ensure documentation is planned
- Provide user perspective on features

### Gates
- [ ] Documentation plan exists for new features

### Deliverables
- DX feedback
- Documentation requirements

### Collaboration
- **With System Architect**: API design
- **With QA Engineer**: Example quality

---

## Phase Completion Checklist

Before moving to Testing phase:

```markdown
## Development Phase Sign-off

- [ ] Architecture review complete (System Architect)
- [ ] Security review complete (Security Auditor)
- [ ] Test plan exists (QA Engineer)
- [ ] Dependencies approved (Build vs Buy)
- [ ] Ethics review complete (Ethics & Safety)
- [ ] Documentation planned (Developer Advocate)

Approved by: _______________
Date: _______________
```
