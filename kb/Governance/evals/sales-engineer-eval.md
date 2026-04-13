---
eval_id: sales-engineer-eval
owner: sales-engineer

intent:
  risk_prevented: customer commitments that exceed delivery capacity or capability
  outcome_ensured: customer commitments verified against documented capabilities and capacity

determinism:
  class: deterministic
  justification: |
    Deliverability checks are rule-based: compare commitment scope against capability
    inventory and capacity constraints. Same commitments and constraints produce same
    assessment. No probabilistic judgment involved.

governance:
  may_block_alone: false
  may_contribute_to_block: true
  eligible_for:
    customer-commitment:
      may_contribute: true
      may_block: true
source: mixed
---

# Sales Engineer Eval

> Quick demo/sales readiness validation. Target: 10 minutes.

## Mission

Perform a rapid sales readiness check. Verify the product is demonstrable, differentiators are provable, and integration paths are clear. Flag demo blockers only.

---

## Criteria Checklist

### 1. Demo Readiness (Critical)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Demo environment | Can run locally or hosted | Works |
| Demo script exists | Walkthrough documented | Present |
| Key flows work | Core scenarios demonstrable | Verified |
| Recovery possible | Can handle demo failures | Practiced |

**Key demo scenarios:**
- [ ] Create a governance policy
- [ ] Show proposal → fact promotion
- [ ] Demonstrate audit trail
- [ ] Show determinism/replay (if claimed)

### 2. Differentiator Proof (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Governance visible | Can show governance in action | Demonstrable |
| vs Competition | Can show what others can't do | Clear |
| Audit trail demo | Can show complete trace | Works |

### 3. Integration Clarity (High)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Integration docs | How to integrate documented | Present |
| Example integrations | Working integration examples | Present |
| Time to integrate | Estimated effort known | Documented |

**Key integrations to check:**
- [ ] OpenAI / Anthropic APIs
- [ ] LangChain (if claimed)
- [ ] Custom workflows

### 4. Objection Handling (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| Common objections | Responses documented | Present |
| Technical limitations | Known and documentable | Clear |
| Competitive comparison | Battle cards exist | Present |

### 5. POC Readiness (Medium)

| Check | What to Look For | Pass Criteria |
|-------|------------------|---------------|
| POC template | Standard POC structure | Exists |
| Success criteria | Standard criteria defined | Documented |
| Setup instructions | Customer can set up | Clear |

---

## Output Format

```markdown
# Sales Engineer Eval Report

## Summary
- **Status**: PASS / PARTIAL / FAIL
- **Demo Ready**: Yes / Partial / No
- **Run Date**: [date]

## Demo Readiness

| Check | Status | Notes |
|-------|--------|-------|
| Demo environment works | ✓/✗ | |
| Demo script exists | ✓/✗ | |
| Core flows work | ✓/✗ | |
| Failure recovery | ✓/✗ | |

## Demo Scenarios

| Scenario | Works | Demo Time | Notes |
|----------|-------|-----------|-------|
| Governance policy | ✓/✗ | | |
| Proposal → Fact | ✓/✗ | | |
| Audit trail | ✓/✗ | | |
| Determinism | ✓/✗ | | |

## Integration Status

| Integration | Documented | Example | Works |
|-------------|------------|---------|-------|
| OpenAI | ✓/✗ | ✓/✗ | ✓/✗ |
| Anthropic | ✓/✗ | ✓/✗ | ✓/✗ |
| LangChain | ✓/✗ | ✓/✗ | ✓/✗ |
| Custom | ✓/✗ | ✓/✗ | ✓/✗ |

## Sales Support Materials

| Material | Exists | Current |
|----------|--------|---------|
| Battle cards | ✓/✗ | |
| Objection responses | ✓/✗ | |
| POC template | ✓/✗ | |
| Pricing info | ✓/✗ | |

## Blockers

### Demo Blockers
[List anything that would cause demo failure]

### Sales Blockers
[List anything blocking sales conversations]

## Verdict

[ ] PASS - Ready for customer demos
[ ] PARTIAL - Can demo with caveats
[ ] FAIL - Not ready for customer-facing
```

---

## Pass/Fail Thresholds

| Threshold | Result |
|-----------|--------|
| Core demos work, integrations documented | PASS |
| Demos work but gaps in materials | PARTIAL |
| Demos broken or can't show differentiation | FAIL |

**Critical issues:**
- Demo environment doesn't work
- Can't demonstrate core governance features
- Can't show differentiation from competitors

**High issues:**
- Missing integration documentation
- No objection handling materials
- POC process undefined
