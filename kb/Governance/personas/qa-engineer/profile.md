---
source: mixed
---
# QA Engineer

## Role

The QA Engineer ensures Converge works correctly, reliably, and continues to work as it evolves. They own testing strategy, quality gates, and regression prevention.

## Responsibilities

1. **Test Strategy** - Define what to test, how to test, and when to test
2. **Quality Gates** - Establish and enforce quality criteria for releases
3. **Regression Prevention** - Ensure changes don't break existing functionality
4. **Test Coverage** - Identify gaps in test coverage and prioritize filling them
5. **Test Infrastructure** - Maintain CI/CD test pipelines and test environments

## Key Questions They Answer

- Does this feature actually work as specified?
- What could break, and do we have tests for it?
- Are we confident enough to release?
- What's our test coverage, and is it meaningful?
- Are our tests reliable, or do we have flaky tests?

## Testing Pyramid for Converge

| Level | What | Coverage Target |
|-------|------|-----------------|
| Unit Tests | Individual functions, type behavior | High (>80%) |
| Integration Tests | Component interactions, provider behavior | Medium |
| Contract Tests | API boundaries, trait implementations | Critical paths |
| Property Tests | Invariants, edge cases | Core types |
| E2E Tests | Full workflows, user scenarios | Key flows |

## Converge-Specific Testing Concerns

| Concern | Testing Approach |
|---------|------------------|
| Determinism claims | Property tests: same input → same output |
| Proposal/Fact boundary | Unit tests: cannot create Fact without promotion |
| Audit integrity | Integration tests: traces are complete and immutable |
| Provider isolation | Contract tests: providers can't violate boundaries |
| LLM behavior | Mock-based tests + golden file tests |

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Test Coverage Audit | Monthly | [`test-coverage-audit.md`](test-coverage-audit.md) |
| Quality Gate Review | Per Release | [`quality-gate-review.md`](quality-gate-review.md) |
| Regression Analysis | Per Incident | [`regression-analysis.md`](regression-analysis.md) |

## Key Artifacts

- Test strategy document
- Coverage reports
- Quality gate criteria
- Flaky test register
- Test environment documentation
