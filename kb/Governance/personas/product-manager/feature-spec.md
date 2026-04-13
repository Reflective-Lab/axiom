---
source: mixed
---
# Product Manager: Feature Specification

## Mission

Create a clear, complete feature specification that engineering can build from and stakeholders can align on.

---

## Context Needed

- User research insights
- Business objectives
- Technical constraints
- Related features/dependencies
- Success metrics goals

---

## Specification Framework

### 1. Problem Definition

```markdown
## Problem Statement

**Who** has this problem: [User segment]
**What** is the problem: [Clear description]
**Why** it matters: [Impact on user/business]
**How** we know: [Evidence - research, data, feedback]

### Current State
[How users solve this today, including workarounds]

### Pain Points
1. [Pain point 1]
2. [Pain point 2]
```

### 2. Solution Overview

```markdown
## Proposed Solution

**One-line description**: [What we're building]

### User Story
As a [user type], I want to [action] so that [outcome].

### Jobs to be Done
- Primary: [Main job this enables]
- Secondary: [Supporting jobs]

### Key Capabilities
1. [Capability 1]
2. [Capability 2]
```

### 3. Detailed Requirements

```markdown
## Requirements

### Functional Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| FR-1 | [Requirement] | Must/Should/Could | [How to verify] |

### Non-Functional Requirements

| Category | Requirement | Target |
|----------|-------------|--------|
| Performance | [Requirement] | [Metric] |
| Security | [Requirement] | [Standard] |
| Accessibility | [Requirement] | [Standard] |
```

### 4. Scope Definition

```markdown
## Scope

### In Scope
- [What's included]

### Out of Scope
- [What's explicitly excluded]

### Future Considerations
- [What we might add later]
```

---

## Output Format

```markdown
# Feature Spec: [Feature Name]

## Metadata
- **Author**: [Name]
- **Date**: [Date]
- **Status**: Draft / Review / Approved
- **Version**: [X.Y]

## Executive Summary

**Problem**: [One sentence]
**Solution**: [One sentence]
**Impact**: [Expected outcome]
**Effort**: T-shirt size (S/M/L/XL)

---

## 1. Problem Definition

### User Problem
[Description of user problem]

### Evidence
| Source | Finding |
|--------|---------|
| [Research/Data] | [What it shows] |

### Current Experience
[How users handle this today]

---

## 2. Proposed Solution

### Overview
[High-level description]

### User Stories

**Primary Story**:
> As a [user], I want to [action] so that [outcome].

**Supporting Stories**:
> As a [user], I want to [action] so that [outcome].

### Key User Flows

1. **[Flow Name]**
   - Step 1: [Action]
   - Step 2: [Action]
   - Expected outcome: [Result]

---

## 3. Requirements

### Functional Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|-------------|----------|---------------------|
| FR-1 | | Must | Given [context], when [action], then [result] |
| FR-2 | | Should | |
| FR-3 | | Could | |

### Non-Functional Requirements

| Category | Requirement | Metric |
|----------|-------------|--------|
| Performance | | |
| Security | | |
| Accessibility | | |
| Scalability | | |

### Constraints
- [Technical constraint]
- [Business constraint]

---

## 4. Scope

### In Scope
- [ ] [Capability 1]
- [ ] [Capability 2]

### Out of Scope
- [Explicitly excluded item]

### Dependencies
| Dependency | Status | Impact if Delayed |
|------------|--------|-------------------|
| [Dependency] | [Status] | [Impact] |

---

## 5. Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| [Metric] | [Baseline] | [Goal] | [How measured] |

### Success Criteria
- [ ] [Criterion 1]
- [ ] [Criterion 2]

---

## 6. Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| [Risk] | H/M/L | H/M/L | [Mitigation] |

---

## 7. Open Questions

| Question | Owner | Due Date | Decision |
|----------|-------|----------|----------|
| [Question] | [Who] | [When] | [TBD] |

---

## 8. Appendix

### Mockups / Wireframes
[Links or embedded images]

### Technical Notes
[Engineering considerations]

### Related Documents
- [Link to research]
- [Link to designs]
```

---

## Spec Quality Checklist

- [ ] Problem is clearly defined with evidence
- [ ] Solution addresses the stated problem
- [ ] Requirements are testable
- [ ] Scope is clear (in/out)
- [ ] Success metrics are defined
- [ ] Risks are identified
- [ ] Dependencies are documented
- [ ] Open questions are tracked
