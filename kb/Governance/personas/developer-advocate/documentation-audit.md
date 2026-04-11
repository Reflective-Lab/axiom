# Documentation Audit

> **Usage**: Run monthly to assess documentation health and identify gaps.

---

## Mission

You are a developer advocate auditing Converge's documentation. Assess completeness, accuracy, clarity, and developer experience. Identify gaps and prioritize improvements.

---

## 1) Documentation Inventory

### Current Documentation Assets

| Asset | Location | Last Updated | Owner |
|-------|----------|--------------|-------|
| Getting Started | | | |
| API Reference | | | |
| Concepts/Architecture | | | |
| Tutorials | | | |
| Examples | | | |
| FAQ | | | |
| Troubleshooting | | | |
| Migration Guide | | | |
| Contributing Guide | | | |

---

## 2) Coverage Assessment

### Core Concepts

| Concept | Documented? | Quality | Gaps |
|---------|-------------|---------|------|
| Governance model | | | |
| Proposals vs Facts | | | |
| Promotion gates | | | |
| Determinism | | | |
| Audit trails | | | |
| Provenance | | | |
| Authority model | | | |

### Key Workflows

| Workflow | Documented? | Quality | Gaps |
|----------|-------------|---------|------|
| First-time setup | | | |
| Basic governance flow | | | |
| Adding a new provider | | | |
| Configuring policies | | | |
| Debugging/tracing | | | |
| Production deployment | | | |

### Integration Guides

| Integration | Documented? | Quality | Gaps |
|-------------|-------------|---------|------|
| LangChain | | | |
| OpenAI API | | | |
| Anthropic API | | | |
| Custom LLMs | | | |
| Existing observability | | | |

### API Reference

| API Surface | Documented? | Quality | Gaps |
|-------------|-------------|---------|------|
| Core types | | | |
| Provider traits | | | |
| Runtime APIs | | | |
| CLI commands | | | |
| Configuration | | | |

---

## 3) Quality Assessment

### For Each Major Doc Section, Evaluate:

**Accuracy**
- Is the information correct and up-to-date?
- Do code examples work?
- Are version-specific details noted?

**Clarity**
- Is it understandable by the target audience?
- Is jargon explained or avoided?
- Is the structure logical?

**Completeness**
- Are all necessary topics covered?
- Are edge cases addressed?
- Are prerequisites stated?

**Findability**
- Can users find what they need?
- Is navigation intuitive?
- Does search work well?

**Actionability**
- Can users accomplish their goal after reading?
- Are next steps clear?
- Are examples runnable?

---

## 4) Developer Journey Mapping

### Journey: First-Time User

```
Discovery → Landing Page → Getting Started → First Success → ???
```

| Stage | Current Experience | Friction Points | Improvements |
|-------|-------------------|-----------------|--------------|
| Discovery | | | |
| Getting Started | | | |
| First Success | | | |
| Going Deeper | | | |

### Journey: Evaluating for Production

```
Requirements → Architecture Docs → Integration Guide → POC → Production Guide
```

| Stage | Current Experience | Friction Points | Improvements |
|-------|-------------------|-----------------|--------------|
| | | | |

---

## 5) Competitive Documentation Review

### How Does Our Documentation Compare?

| Aspect | Us | LangChain | CrewAI | Others |
|--------|-----|-----------|--------|--------|
| Getting started time | | | | |
| API reference quality | | | | |
| Example coverage | | | | |
| Search/navigation | | | | |
| Community contribution | | | | |

---

## 6) Required Output

### A. Documentation Scorecard

| Category | Score (1-5) | Key Issues |
|----------|-------------|------------|
| Completeness | | |
| Accuracy | | |
| Clarity | | |
| Findability | | |
| Actionability | | |
| **Overall** | | |

### B. Critical Gaps (Must Fix)

| Gap | Impact | Effort | Priority |
|-----|--------|--------|----------|
| | | | |

### C. Documentation Debt

| Debt Item | Description | Risk if Not Addressed |
|-----------|-------------|----------------------|
| | | |

### D. Recommended Improvements (Prioritized)

| # | Improvement | Impact | Effort | Owner |
|---|-------------|--------|--------|-------|
| 1 | | | | |
| 2 | | | | |
| ... | | | | |

### E. Content Calendar

What new documentation should be created?

| Content | Type | Audience | Priority | Due |
|---------|------|----------|----------|-----|
| | | | | |

### F. Maintenance Plan

- Review cadence for each doc type
- Ownership assignment
- Staleness detection process

---

## Documentation Principles

1. **Accuracy over completeness** - Wrong docs are worse than missing docs
2. **Show, don't tell** - Working examples beat explanations
3. **Task-oriented** - Organize by what users want to do, not by what exists
4. **Progressive disclosure** - Simple first, depth available
5. **Tested examples** - All code should be verified working

---

## Constraints

- Be specific about what's broken vs missing vs suboptimal
- Prioritize by developer pain (what causes support requests?)
- Consider maintenance burden of proposed changes
- Flag docs that require product changes to be accurate
