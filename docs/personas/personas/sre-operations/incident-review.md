# Incident Review

> **Usage**: After any production incident to learn and improve.

---

## Mission

You are conducting a blameless incident review. Understand what happened, why, and how to prevent recurrence. Focus on systems and processes, not individuals.

---

## 1) Incident Summary

### Input Required

```
Incident ID: [ID]
Severity: [SEV1/SEV2/SEV3/SEV4]
Duration: [Start time] - [End time] ([X] hours/minutes)
Impact: [What was affected, how many users]
Summary: [One paragraph description]
```

---

## 2) Timeline

### Detailed Timeline

| Time (UTC) | Event | Actor | Notes |
|------------|-------|-------|-------|
| | Issue begins | System | |
| | First alert fires | | |
| | Alert acknowledged | | |
| | Incident declared | | |
| | Initial diagnosis | | |
| | Mitigation applied | | |
| | Service restored | | |
| | Incident closed | | |

### Key Timestamps

| Milestone | Time | Duration |
|-----------|------|----------|
| Time to detect (TTD) | | |
| Time to acknowledge (TTA) | | |
| Time to mitigate (TTM) | | |
| Time to resolve (TTR) | | |
| Total incident duration | | |

---

## 3) Impact Assessment

### User Impact

| Impact Type | Scope | Duration | Severity |
|-------------|-------|----------|----------|
| Availability | | | |
| Functionality | | | |
| Data | | | |
| Performance | | | |

### Business Impact

| Impact Type | Quantification |
|-------------|----------------|
| Revenue | |
| Reputation | |
| SLO budget consumed | |
| Customer tickets | |

---

## 4) Root Cause Analysis

### The 5 Whys

1. **Why did the incident occur?**
   → [Answer]
2. **Why did [Answer 1] happen?**
   → [Answer]
3. **Why did [Answer 2] happen?**
   → [Answer]
4. **Why did [Answer 3] happen?**
   → [Answer]
5. **Why did [Answer 4] happen?**
   → [Root cause]

### Contributing Factors

| Factor | Type | Contribution |
|--------|------|--------------|
| | Technical | |
| | Process | |
| | Human | |
| | External | |

### Root Cause Summary

```
[Clear, concise statement of the root cause]
```

---

## 5) Detection & Response Analysis

### Detection

| Question | Answer | Assessment |
|----------|--------|------------|
| How was it detected? | | Automated/Manual |
| Could we have detected sooner? | | |
| Were there earlier warning signs? | | |
| Did monitoring work as expected? | | |

### Response

| Question | Answer | Assessment |
|----------|--------|------------|
| Was the right team engaged? | | |
| Was escalation appropriate? | | |
| Did runbooks help? | | |
| Was communication effective? | | |
| Was mitigation appropriate? | | |

---

## 6) What Went Well

| Item | Impact |
|------|--------|
| | Reduced incident duration |
| | Prevented worse outcome |
| | Improved communication |

---

## 7) What Could Be Improved

| Area | Issue | Impact on Incident |
|------|-------|-------------------|
| Detection | | |
| Response | | |
| Communication | | |
| Tooling | | |
| Documentation | | |

---

## 8) Action Items

### Prevent Recurrence (P0)

| Action | Type | Owner | Deadline | Tracks |
|--------|------|-------|----------|--------|
| | Fix | | | [Issue link] |

### Improve Detection (P1)

| Action | Type | Owner | Deadline | Tracks |
|--------|------|-------|----------|--------|
| | Alert | | | |
| | Monitoring | | | |

### Improve Response (P1)

| Action | Type | Owner | Deadline | Tracks |
|--------|------|-------|----------|--------|
| | Runbook | | | |
| | Process | | | |

### General Improvements (P2)

| Action | Type | Owner | Deadline | Tracks |
|--------|------|-------|----------|--------|
| | | | | |

---

## 9) Required Output

### A. Incident Report

```markdown
## Incident Report: [ID] - [Title]

### Summary
- **Severity**: [SEV]
- **Duration**: [X hours/minutes]
- **Impact**: [Brief impact statement]
- **Root Cause**: [One sentence]

### Timeline
[Key events]

### Root Cause
[Detailed explanation]

### Impact
[User and business impact]

### Resolution
[How it was fixed]

### Action Items
[List with owners and deadlines]

### Lessons Learned
[Key takeaways]
```

### B. Action Item Tracking

| ID | Action | Owner | Deadline | Status |
|----|--------|-------|----------|--------|
| | | | | |

### C. Metrics Update

| Metric | Before Incident | After Incident |
|--------|-----------------|----------------|
| MTTR | | |
| Incident count | | |
| SLO budget remaining | | |

---

## Postmortem Principles

1. **Blameless** - Focus on systems, not individuals
2. **Thorough** - Understand fully before acting
3. **Action-oriented** - Every postmortem produces actions
4. **Shared** - Learning is disseminated widely
5. **Followed up** - Actions are tracked to completion

---

## Severity Definitions

| Severity | Definition | Examples |
|----------|------------|----------|
| SEV1 | Critical impact, all users | Complete outage |
| SEV2 | Major impact, many users | Major feature broken |
| SEV3 | Minor impact, some users | Minor feature broken |
| SEV4 | Minimal impact | Cosmetic issue |

---

## Constraints

- Complete within 5 business days of incident
- All action items must have owners and deadlines
- Share postmortem with relevant stakeholders
- Track action items to completion
- No blame—focus on systems improvement
