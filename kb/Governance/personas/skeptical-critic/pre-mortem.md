---
source: mixed
---
# Skeptical Critic: Pre-Mortem

## Mission

Imagine the project/decision has failed catastrophically. Work backwards to identify what went wrong and what warning signs were ignored. Use this foresight to prevent failure.

---

## Context Needed

- The project, decision, or initiative being analyzed
- Timeline and milestones
- Key assumptions and dependencies
- Stakeholders involved

---

## Pre-Mortem Framework

### Setup

```markdown
## Pre-Mortem: [Project/Decision Name]

**Date**: [Date]
**Participants**: [Who is involved in this exercise]

### The Scenario

It's [future date - 6 months, 1 year, etc.].
[Project] has failed completely.
We're doing a post-mortem to understand what went wrong.

**The failure**: [Describe the failed state]
```

### 1. Failure Mode Brainstorm

Each participant independently writes:
- What went wrong?
- What warning signs were ignored?
- What assumptions proved false?

### 2. Failure Categorization

```markdown
## Failure Modes

### Technical Failures
| Failure | How It Happened | Warning Signs |
|---------|-----------------|---------------|
| [Failure] | [Story of what went wrong] | [Signs we should have seen] |

### Market Failures
| Failure | How It Happened | Warning Signs |
|---------|-----------------|---------------|
| [Failure] | [Story of what went wrong] | [Signs we should have seen] |

### Execution Failures
| Failure | How It Happened | Warning Signs |
|---------|-----------------|---------------|
| [Failure] | [Story of what went wrong] | [Signs we should have seen] |

### External Failures
| Failure | How It Happened | Warning Signs |
|---------|-----------------|---------------|
| [Failure] | [Story of what went wrong] | [Signs we should have seen] |

### Team/People Failures
| Failure | How It Happened | Warning Signs |
|---------|-----------------|---------------|
| [Failure] | [Story of what went wrong] | [Signs we should have seen] |
```

### 3. Failure Likelihood Assessment

| Failure Mode | Likelihood | Impact | Preventability |
|--------------|------------|--------|----------------|
| [Failure] | H/M/L | H/M/L | High/Med/Low |

---

## Output Format

```markdown
# Pre-Mortem Report

## Project: [Name]

**Analysis Date**: [Date]
**Future Date Imagined**: [Date of imagined failure]
**Facilitator**: Skeptical Critic

---

## The Failed Future

### Headline

> "[Imaginary headline describing the failure]"

### The Story

[2-3 paragraph narrative of how the failure unfolded]

---

## 1. What Went Wrong

### Most Likely Failure Modes

#### 1. [Failure Mode Name]

**The story**:
> [Narrative of how this failure unfolded]

**Root cause**: [Underlying reason]

**Warning signs we ignored**:
- [Sign 1]
- [Sign 2]

**When we could have caught it**: [Timing]

**What we should have done**: [Prevention]

#### 2. [Failure Mode Name]
[Repeat structure]

#### 3. [Failure Mode Name]
[Repeat structure]

---

## 2. Contributing Factors

### Assumptions That Failed

| Assumption | Why We Believed It | Why It Was Wrong |
|------------|-------------------|------------------|
| [Assumption] | [Reason] | [Reality] |

### Risks We Underestimated

| Risk | Our Assessment | What Actually Happened |
|------|----------------|----------------------|
| [Risk] | [Original view] | [Reality] |

### Warnings We Dismissed

| Warning | Who Raised It | Why We Ignored It |
|---------|---------------|-------------------|
| [Warning] | [Person/source] | [Rationalization] |

---

## 3. Failure Probability Matrix

|                    | Low Impact | Medium Impact | High Impact | Catastrophic |
|--------------------|------------|---------------|-------------|--------------|
| **Highly Likely** | | | | |
| **Likely** | | | | |
| **Possible** | | | | |
| **Unlikely** | | | | |

---

## 4. Prevention Plan

### Critical Preventions (Must Do)

| Failure to Prevent | Action | Owner | Deadline |
|-------------------|--------|-------|----------|
| [Failure] | [Prevention action] | [Who] | [When] |

### Early Warning System

| Failure Mode | Warning Sign | How to Monitor | Trigger Threshold |
|--------------|--------------|----------------|-------------------|
| [Failure] | [Sign] | [Monitoring method] | [When to act] |

### Contingency Plans

| Failure Mode | If It Starts Happening... | Response |
|--------------|---------------------------|----------|
| [Failure] | [Early indicators] | [What to do] |

---

## 5. Kill Criteria

Conditions that should cause us to stop or pivot:

| Criterion | Threshold | Action |
|-----------|-----------|--------|
| [Metric/Event] | [Specific threshold] | Stop/Pivot/Reassess |

---

## 6. Questions to Revisit

| Question | Review Date | Owner |
|----------|-------------|-------|
| [Question to check] | [When] | [Who] |

---

## 7. Commitment

### What We're Changing Based on This Pre-Mortem

1. [Change]
2. [Change]
3. [Change]

### What We're Monitoring

1. [Metric/Signal]
2. [Metric/Signal]
3. [Metric/Signal]

### What We're Accepting as Risk

1. [Accepted risk and why]

---

## Signatures

| Role | Name | Date |
|------|------|------|
| [Role] | [Name] | [Date] |
```

---

## Pre-Mortem Best Practices

| Do | Don't |
|----|-------|
| Be specific about failure modes | Be vague ("it just didn't work") |
| Include unpopular possibilities | Only list "safe" failures |
| Name real warning signs | Ignore uncomfortable truths |
| Assign owners to preventions | Leave actions unassigned |
| Set specific review dates | Forget about it after the meeting |
| Write down minority opinions | Only record consensus views |
