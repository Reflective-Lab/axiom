---
source: mixed
---
# Operational Readiness Review

> **Usage**: Before each release to verify operational readiness.

---

## Mission

You are an SRE reviewing whether Converge is operationally ready for release. Verify observability, reliability, and operational support are adequate for production.

---

## 1) Observability Checklist

### Logging

| Check | Status | Evidence |
|-------|--------|----------|
| Structured logging implemented | | |
| Log levels appropriate | | |
| Sensitive data not logged | | |
| Log aggregation configured | | |
| Log retention defined | | |

### Metrics

| Check | Status | Evidence |
|-------|--------|----------|
| Key metrics instrumented | | |
| SLI metrics available | | |
| Resource metrics (CPU, mem) | | |
| Business metrics | | |
| Dashboards created | | |

### Tracing

| Check | Status | Evidence |
|-------|--------|----------|
| Distributed tracing enabled | | |
| Trace context propagated | | |
| Key spans instrumented | | |
| Trace sampling configured | | |

### Alerting

| Check | Status | Evidence |
|-------|--------|----------|
| SLO-based alerts defined | | |
| Alert thresholds tuned | | |
| Alert routing configured | | |
| Alert runbooks linked | | |
| On-call schedule exists | | |

---

## 2) Reliability Checklist

### Failure Handling

| Check | Status | Evidence |
|-------|--------|----------|
| Graceful degradation | | |
| Circuit breakers | | |
| Retry logic (with backoff) | | |
| Timeout configuration | | |
| Error handling comprehensive | | |

### Resilience

| Check | Status | Evidence |
|-------|--------|----------|
| No single points of failure | | |
| Health checks implemented | | |
| Liveness probes | | |
| Readiness probes | | |
| Graceful shutdown | | |

### Data Protection

| Check | Status | Evidence |
|-------|--------|----------|
| Backup strategy defined | | |
| Backup tested | | |
| Recovery procedure documented | | |
| Recovery tested | | |
| Data durability verified | | |

---

## 3) Deployment Readiness

### Deployment Process

| Check | Status | Evidence |
|-------|--------|----------|
| Deployment automated | | |
| Rollback procedure documented | | |
| Rollback tested | | |
| Canary/staged rollout | | |
| Feature flags for risky changes | | |

### Configuration

| Check | Status | Evidence |
|-------|--------|----------|
| Config externalized | | |
| Secrets management | | |
| Environment parity | | |
| Config validation | | |

---

## 4) Runbook Checklist

### Essential Runbooks

| Runbook | Exists | Current | Tested |
|---------|--------|---------|--------|
| Service restart | | | |
| Scale up/down | | | |
| Rollback procedure | | | |
| Database recovery | | | |
| Incident triage | | | |
| Debug guide | | | |

### Runbook Quality

| Check | Status |
|-------|--------|
| Step-by-step instructions | |
| Copy-pasteable commands | |
| Expected outcomes documented | |
| Escalation paths clear | |
| Recently reviewed | |

---

## 5) Incident Preparedness

### Incident Response

| Check | Status | Evidence |
|-------|--------|----------|
| Incident response process documented | | |
| Roles defined (IC, scribe, etc.) | | |
| Communication channels established | | |
| Stakeholder notification process | | |
| Postmortem process defined | | |

### On-Call

| Check | Status | Evidence |
|-------|--------|----------|
| On-call rotation defined | | |
| Escalation paths clear | | |
| On-call documentation | | |
| Alert fatigue managed | | |

---

## 6) Capacity & Performance

### Current State

| Metric | Current | Headroom | Concern? |
|--------|---------|----------|----------|
| CPU utilization | | | |
| Memory utilization | | | |
| Storage utilization | | | |
| Network utilization | | | |

### Load Testing

| Check | Status | Evidence |
|-------|--------|----------|
| Load testing performed | | |
| Performance baselines | | |
| Breaking point known | | |
| Scaling triggers defined | | |

---

## 7) Required Output

### A. Readiness Summary

```markdown
## Operational Readiness Report - [Version]

### Overall Status: READY / READY WITH CAVEATS / NOT READY

### Category Status
| Category | Status | Blockers |
|----------|--------|----------|
| Observability | ✅/⚠️/❌ | |
| Reliability | ✅/⚠️/❌ | |
| Deployment | ✅/⚠️/❌ | |
| Runbooks | ✅/⚠️/❌ | |
| Incident Prep | ✅/⚠️/❌ | |
| Capacity | ✅/⚠️/❌ | |
```

### B. Blockers

| Blocker | Severity | Resolution | Owner |
|---------|----------|------------|-------|
| | | | |

### C. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| | | | |

### D. Post-Release Monitoring Plan

| What to Watch | Threshold | Action if Breached |
|---------------|-----------|-------------------|
| | | |

### E. Action Items

**Before Release**
| Action | Owner | Deadline |
|--------|-------|----------|
| | | |

**After Release**
| Action | Owner | Timeline |
|--------|-------|----------|
| | | |

---

## Readiness Criteria

### Must Have (Blocking)
- [ ] All critical metrics instrumented
- [ ] SLO alerts configured
- [ ] Rollback procedure tested
- [ ] On-call coverage confirmed
- [ ] Health checks working

### Should Have (Non-blocking, tracked)
- [ ] All runbooks current
- [ ] Load testing completed
- [ ] Dashboards reviewed
- [ ] Alert runbooks linked

---

## Constraints

- Be conservative—operational gaps can cause outages
- Verify, don't trust—test rollbacks, check alerts fire
- Document gaps even if not blocking
- Consider blast radius of this release
