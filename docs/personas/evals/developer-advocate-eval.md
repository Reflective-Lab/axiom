---
eval_id: developer-advocate-eval
owner: developer-advocate

intent:
  risk_prevented: poor documentation blocking developer adoption
  outcome_ensured: documentation exists and onboarding path is functional

determinism:
  class: bounded
  justification: |
    Documentation quality assessment depends on LLM-based review of docs and examples.
    While structured criteria are used (README exists, examples work), quality
    judgments involve semantic understanding of clarity and completeness, not just
    rule-based checks.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    release-approval:
      may_contribute: true
      may_block: false
---

# Developer Advocate Eval

> Quick documentation/DX validation. Target: 10 minutes.

## Mission

Perform a rapid developer experience check. Verify documentation exists, examples work, and onboarding path is clear. Flag critical gaps only.

---

## Criteria Checklist

### 1. Getting Started (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| README exists | Root README.md | Present |
| Quick start section | Clear first steps | Present and clear |
| Installation instructions | How to install | Present and work |
| First example | Runnable example | Present and works |

**Quick checks:**
```bash
# README exists
ls README.md

# Check for getting started section
grep -i "getting started\|quick start\|installation" README.md
```

### 2. Documentation Coverage (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Core concepts documented | Key abstractions explained | Present |
| API reference | Public API documented | Present |
| Examples directory | Working examples | Present |

**Key concepts that must be documented:**
- [ ] Proposal vs Fact
- [ ] Authority model
- [ ] Promotion gates
- [ ] Audit/tracing
- [ ] Determinism model

### 3. Example Quality (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Examples compile | `cargo build --examples` | Passes |
| Examples are runnable | Clear run instructions | Present |
| Examples are documented | Comments explain what's happening | Adequate |

**Quick check:**
```bash
# Try to build examples
cargo build --examples

# Check examples exist
ls examples/
```

### 4. Error Messages (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Custom error types | Good error messages | Present |
| Actionable errors | Errors suggest fixes | Some present |

### 5. Onboarding Path (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Clear progression | Beginner → Advanced path | Exists |
| No dead ends | Links work, next steps clear | Verified |

---

## Output Format

```markdown
# Developer Advocate Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Critical Gaps**: [count]
- **High Gaps**: [count]
- **Run Date**: [date]

## Getting Started

| Check | Status | Notes |
|-------|--------|-------|
| README present | ✓/✗ | |
| Quick start clear | ✓/✗ | |
| Install instructions | ✓/✗ | |
| First example works | ✓/✗ | |

## Documentation Coverage

| Concept | Documented? | Quality |
|---------|-------------|---------|
| Proposal/Fact | ✓/✗ | |
| Authority | ✓/✗ | |
| Promotion gates | ✓/✗ | |
| Audit/tracing | ✓/✗ | |
| Determinism | ✓/✗ | |

## Example Status

| Example | Compiles | Documented | Notes |
|---------|----------|------------|-------|
| | | | |

## Onboarding Assessment

| Stage | Status | Friction Points |
|-------|--------|-----------------|
| Discovery | ✓/✗ | |
| First run | ✓/✗ | |
| Understanding | ✓/✗ | |
| Production use | ✓/✗ | |

## Gaps Found

### Critical (Blocking Adoption)
[List]

### High (Significant Friction)
[List]

### Nice to Have
[List]

## Verdict

[ ] PASS - Good enough for developers to succeed
[ ] PARTIAL - Usable but has friction
[ ] FAIL - Significant gaps blocking adoption
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Getting started works, core concepts documented | PASS |
| Getting started works, some gaps | PARTIAL |
| Getting started broken or missing | FAIL |

**Critical gaps:**
- No README
- Getting started doesn't work
- Examples don't compile
- Core concepts completely undocumented

**High gaps:**
- Missing API reference
- Examples undocumented
- Broken links
