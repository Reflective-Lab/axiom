---
eval_id: academic-researcher-eval
owner: academic-researcher

intent:
  risk_prevented: academically unsound or unsupported claims entering scholarly or technical discourse
  outcome_ensured: scholarly rigor and evidence standards assessed within peer review lens

determinism:
  class: nondeterministic
  justification: |
    Human judgment proxy for academic peer review perspective.
    Output depends on research methodology evaluation, evidence strength assessment, and scholarly judgment.
    Cannot be deterministically replayed - rigor assessments vary based on disciplinary standards interpretation.

governance:
  may_block_alone: false
  may_contribute_to_block: false  # Advisory personas never contribute to blocking
  eligible_for:
    pr-merge:
      may_contribute: true
      may_block: false
    release-candidate:
      may_contribute: true
      may_block: false
    release-approval:
      may_contribute: true
      may_block: false
    content-publish:
      may_contribute: true
      may_block: false
    customer-commitment:
      may_contribute: true
      may_block: false
    production-deploy:
      may_contribute: false
      may_block: false

evidence:
  requires:
    - Technical claims and assertions
    - Methodology documentation
    - Prior work citations and comparisons
    - Limitations and constraints disclosure
  format: markdown report with claims audit and methodology assessment
  location: eval-reports/academic-researcher/
source: mixed
---

# Academic Researcher Eval

> Scholarly rigor validation. Target: 10 minutes.

## Mission

View Converge through the eyes of an academic researcher. Are our claims evidenced? Would our methodology pass peer review? Are we contributing to or detracting from the field?

---

## Criteria Checklist

### 1. Claims Evidence Check (Critical)

| Claim We Make | Evidence | Strength |
|---------------|----------|----------|
| [Technical claim] | [What supports it] | Strong/Weak/None |
| [Performance claim] | [What supports it] | Strong/Weak/None |
| [Business claim] | [What supports it] | Strong/Weak/None |
| [Impact claim] | [What supports it] | Strong/Weak/None |

### 2. Methodology Rigor (High)

| Aspect | Status | Academic Standard |
|--------|--------|-------------------|
| Reproducibility | ✓/✗ | Can others replicate? |
| Transparency | ✓/✗ | Methods disclosed? |
| Limitations stated | ✓/✗ | Honest about constraints? |
| Baselines compared | ✓/✗ | Fair comparisons? |
| Statistical validity | ✓/✗ | Proper analysis? |

### 3. Prior Work Citation (High)

| Area | Prior Work Acknowledged? | Building On or Ignoring? |
|------|-------------------------|--------------------------|
| [Technical area] | Yes/No | Building/Ignoring |
| [Related approaches] | Yes/No | Building/Ignoring |
| [Competing methods] | Yes/No | Building/Ignoring |

### 4. Intellectual Honesty (Critical)

| Question | Honest Answer |
|----------|---------------|
| Are we overclaiming? | Yes/No |
| Are we hiding limitations? | Yes/No |
| Would a peer reviewer approve? | Yes/No |
| Are we contributing to the field? | Yes/No |

---

## Output Format

```markdown
# Academic Researcher Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Scholarly Rigor Score**: [1-5]
- **Run Date**: [date]

## Claims Audit

| Claim | Evidence Quality | Issue |
|-------|-----------------|-------|
| [Claim] | Strong/Adequate/Weak/None | [Problem if any] |

### Unsupported Claims

1. [Claim without adequate evidence]

### Overclaims

1. [Claim that overstates what evidence supports]

## Methodology Assessment

| Criterion | Status | Notes |
|-----------|--------|-------|
| Reproducible | ✓/✗ | |
| Transparent | ✓/✗ | |
| Limitations disclosed | ✓/✗ | |
| Fair comparisons | ✓/✗ | |
| Valid analysis | ✓/✗ | |

### Would This Pass Peer Review?

**Assessment**: Yes / With Revisions / No

**Reviewer Comments (Simulated)**:
> [What a reviewer would say]

## Prior Work

| Area | Citation Status | Issue |
|------|-----------------|-------|
| [Area] | Cited/Missing | [If missing, what should be cited] |

### Missing Citations

1. [Work we should reference]

## Contribution Assessment

| Question | Answer |
|----------|--------|
| Novel contribution? | Yes/Incremental/No |
| Advances the field? | Yes/Neutral/Harmful |
| Honest representation? | Yes/Mostly/No |

## Verdict

[ ] PASS - Meets academic standards
[ ] PARTIAL - Some rigor issues
[ ] FAIL - Would not pass scholarly review
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Claims evidenced, methods rigorous, honest limitations | PASS |
| Some claims weak, methodology mostly sound | PARTIAL |
| Major unsupported claims, methodological flaws | FAIL |
