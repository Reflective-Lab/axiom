# Production Phase Contracts

Responsibilities during live operation.

---

## SRE / Operations

**Role**: Accountable for reliability

### Responsibilities
- Monitor production systems
- Respond to incidents
- Manage capacity
- Maintain operational health
- Coordinate deployments

### Gates
- [ ] SLOs are being met
- [ ] No critical incidents unresolved
- [ ] Capacity is sufficient

### Deliverables
- Monitoring dashboards
- Incident reports
- Capacity reports
- Operational metrics

### Collaboration
- **With Security Auditor**: Security monitoring
- **With QA Engineer**: Issue triage
- **With System Architect**: Performance issues

---

## Security Auditor

**Role**: Responsible for security monitoring

### Responsibilities
- Monitor for security incidents
- Respond to security events
- Maintain security posture
- Coordinate vulnerability remediation

### Gates
- [ ] No active security incidents
- [ ] Vulnerabilities addressed per SLA

### Deliverables
- Security monitoring
- Incident response
- Vulnerability tracking

### Collaboration
- **With SRE**: Incident coordination
- **With Legal Counsel**: Breach notification
- **With System Architect**: Security remediation

---

## QA Engineer

**Role**: Consulted for issue triage

### Responsibilities
- Triage production bugs
- Verify fixes
- Maintain regression suite
- Support incident investigation

### Gates
- [ ] Critical bugs fixed promptly
- [ ] Regression tests updated

### Deliverables
- Bug triage
- Fix verification
- Regression updates

### Collaboration
- **With SRE**: Incident support
- **With Developer**: Bug fixes

---

## Developer Advocate

**Role**: Responsible for customer communication

### Responsibilities
- Communicate with customers about issues
- Update documentation for production issues
- Gather feedback from production usage
- Support community

### Gates
- [ ] Customers informed of significant issues
- [ ] Documentation reflects production reality

### Deliverables
- Customer communications
- Status updates
- Documentation updates

### Collaboration
- **With SRE**: Incident communication
- **With Sales Engineer**: Customer coordination

---

## End User Advocate

**Role**: Consulted for impact assessment

### Responsibilities
- Assess user impact of incidents
- Ensure users are considered in decisions
- Advocate for user-centric response

### Gates
- [ ] User impact minimized and communicated

### Deliverables
- User impact assessment

### Collaboration
- **With SRE**: Impact evaluation
- **With Ethics & Safety**: Harm assessment

---

## Ethics & Safety Officer

**Role**: Consulted for harm response

### Responsibilities
- Assess harm from incidents
- Ensure responsible incident response
- Review for ethical implications

### Gates
- [ ] Harm minimized and addressed

### Deliverables
- Harm assessment
- Ethics guidance

### Collaboration
- **With SRE**: Incident response
- **With Legal Counsel**: Liability assessment

---

## Legal Counsel

**Role**: Consulted for legal response

### Responsibilities
- Advise on legal implications of incidents
- Manage breach notification
- Handle legal response

### Gates
- [ ] Legal obligations met

### Deliverables
- Legal guidance
- Breach notification (if needed)

### Collaboration
- **With SRE**: Incident coordination
- **With Security Auditor**: Breach assessment

---

## Founder

**Role**: Accountable for major incidents

### Responsibilities
- Escalation point for major incidents
- Customer communication for severe issues
- Strategic decisions during crises

### Gates
- [ ] Major incidents resolved appropriately

### Deliverables
- Executive decisions
- Strategic communication

### Collaboration
- **With all personas**: Escalation

---

## Incident Response Roles

```markdown
## Incident Response Team

| Role | Primary | Backup |
|------|---------|--------|
| Incident Commander | SRE | |
| Technical Lead | System Architect | |
| Communications | Developer Advocate | |
| Security | Security Auditor | |
| Legal | Legal Counsel | |
| Executive | Founder | |
```

## Production Health Checklist

Ongoing production responsibilities:

```markdown
## Production Health - [Date]

- [ ] SLOs met (SRE)
- [ ] No critical incidents (SRE)
- [ ] Security posture maintained (Security)
- [ ] Capacity sufficient (SRE)
- [ ] Customer communication current (DevRel)
- [ ] Legal obligations met (Legal)

Status: HEALTHY / DEGRADED / CRITICAL

Signed: _______________
```
