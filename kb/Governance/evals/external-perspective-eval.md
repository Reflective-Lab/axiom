---
eval_id: external-perspective-eval
owner: external-perspective  # Meta-persona: composite of multiple external lenses

intent:
  risk_prevented: external scrutiny failures across multiple stakeholder perspectives entering public exposure
  outcome_ensured: multi-stakeholder external perception assessed across regulatory, press, academic, and historical lenses

scope:
  includes:
    - Regulator lens (compliance and consumer protection)
    - Journalist lens (press scrutiny and narrative risk)
    - Academic lens (scholarly rigor and evidence standards)
    - Historian lens (long-term implications and pattern awareness)

determinism:
  class: nondeterministic
  justification: |
    Composite human judgment proxy combining multiple external perspectives.
    Output depends on cross-perspective synthesis and multi-stakeholder interpretation.
    Cannot be deterministically replayed - each lens involves subjective judgment.

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
    - Compliance and regulatory posture artifacts
    - Public-facing communications and claims
    - Technical and scholarly documentation
    - Strategic decisions and long-term rationale
  format: markdown report with multi-lens perspective summary
  location: eval-reports/external-perspective/
source: mixed
---

# External Perspective Eval

> Combined eval for regulator, journalist, researcher, and historian lenses. Target: 15 minutes.

## Mission

View Converge through external eyes. How would outsiders perceive and evaluate us?

---

## 1. Regulator Lens

| Check | What They'd See | Concern? |
|-------|-----------------|----------|
| Compliance posture | | |
| Consumer protection | | |
| Transparency | | |

**Regulator's Question**: Would we trigger an investigation?

### Verdict: 🟢 / 🟡 / 🔴

---

## 2. Journalist Lens

| Story Angle | What They'd Write | Problem? |
|-------------|-------------------|----------|
| Positive story | | |
| Neutral story | | |
| Critical story | | |
| Hit piece | | |

**Journalist's Question**: What's the headline?

### Verdict: 🟢 / 🟡 / 🔴

---

## 3. Academic Lens

| Check | Status | Concern? |
|-------|--------|----------|
| Claims evidenced | | |
| Methodology sound | | |
| Prior work cited | | |
| Limitations stated | | |

**Researcher's Question**: Would this pass peer review?

### Verdict: 🟢 / 🟡 / 🔴

---

## 4. Future Historian Lens

| Check | Answer |
|-------|--------|
| What will seem obviously wrong? | |
| What will we be proud of? | |
| What will we regret? | |
| What are we not seeing? | |

**Historian's Question**: How will this era be judged?

### Verdict: 🟢 / 🟡 / 🔴

---

## Output Format

```markdown
# External Perspective Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Run Date**: [date]

## Perspective Summary

| Lens | Verdict | Key Concern |
|------|---------|-------------|
| Regulator | 🟢/🟡/🔴 | |
| Journalist | 🟢/🟡/🔴 | |
| Academic | 🟢/🟡/🔴 | |
| Historian | 🟢/🟡/🔴 | |

## Red Flags Found

| Flag | Source | Action |
|------|--------|--------|

## Overall Verdict

[ ] PASS - Would withstand external scrutiny
[ ] PARTIAL - Some concerning angles
[ ] FAIL - Significant external perception risk
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No red flags across lenses | PASS |
| Minor concerns | PARTIAL |
| Would fail scrutiny in any lens | FAIL |
