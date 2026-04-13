---
eval_id: journalist-investigator-eval
owner: journalist-investigator

intent:
  risk_prevented: press-vulnerable or scrutiny-failing content entering public communication
  outcome_ensured: external scrutiny resilience assessed within investigative journalism lens

determinism:
  class: nondeterministic
  justification: |
    Human judgment proxy for investigative journalism perspective.
    Output depends on narrative construction, source interpretation, and editorial judgment.
    Cannot be deterministically replayed - story angles vary unpredictably.

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
    - Public-facing claims and communications
    - Potential vulnerability areas (technical, business, ethical)
    - Response readiness for negative coverage scenarios
  format: markdown report with story angle analysis and headline tests
  location: eval-reports/journalist-investigator/
source: mixed
---

# Journalist / Investigator Eval

> Press scrutiny validation. Target: 10 minutes.

## Mission

View Converge through the eyes of an investigative journalist. What story would they write? What would they dig into? Could we withstand press scrutiny?

---

## Criteria Checklist

### 1. Story Angle Analysis (Critical)

| Story Type | What They'd Write | Our Exposure |
|------------|-------------------|--------------|
| **Positive/Puff piece** | | |
| **Neutral/Explainer** | | |
| **Critical/Skeptical** | | |
| **Investigative/Hit piece** | | |

### 2. "What Are They Hiding?" Check (Critical)

| Question | Honest Answer | Problem? |
|----------|---------------|----------|
| What wouldn't we want a journalist to find? | | Yes/No |
| What would look bad in a screenshot? | | Yes/No |
| What would look bad out of context? | | Yes/No |
| What internal discussions would be embarrassing? | | Yes/No |

### 3. Source Vulnerability (High)

| Potential Source | What They'd Say | Risk |
|------------------|-----------------|------|
| Disgruntled employee | | H/M/L |
| Unhappy customer | | H/M/L |
| Competitor | | H/M/L |
| Regulator | | H/M/L |
| Affected end user | | H/M/L |

### 4. Headline Test (High)

Write the headlines a journalist might use:

| Outlet Type | Likely Headline |
|-------------|-----------------|
| Tech press (positive) | |
| Tech press (skeptical) | |
| Mainstream (if interested) | |
| Investigative (worst case) | |

---

## Output Format

```markdown
# Journalist/Investigator Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Press Risk Level**: Low / Medium / High
- **Run Date**: [date]

## Story Analysis

### The Stories They'd Write

**Best Case (Positive Coverage)**
> [Headline]
> [Angle/Summary]

**Neutral Coverage**
> [Headline]
> [Angle/Summary]

**Critical Coverage**
> [Headline]
> [Angle/Summary]

**Worst Case (Investigation)**
> [Headline]
> [What they'd dig into]

## Vulnerability Assessment

| Area | Exposure | What They'd Find |
|------|----------|------------------|
| [Area] | H/M/L | [Description] |

### "Smoking Guns"

Things that would look bad if published:
1. [Item]
2. [Item]

### Source Risk

| Source Type | Likely Narrative | Credibility |
|-------------|------------------|-------------|
| [Source] | [What they'd say] | H/M/L |

## Our Response Readiness

| If This Story Breaks... | Our Response | Adequate? |
|------------------------|--------------|-----------|
| [Scenario] | [Response] | Yes/No |

## The Front Page Test

> Would we be comfortable if this appeared on the front page?

**Assessment**: Yes / Mostly / Concerning / No

## Verdict

[ ] PASS - Could withstand press scrutiny
[ ] PARTIAL - Some vulnerable areas
[ ] FAIL - Significant press exposure risk
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| No smoking guns, defensible practices | PASS |
| Some vulnerable areas, manageable | PARTIAL |
| Would not survive investigative scrutiny | FAIL |
