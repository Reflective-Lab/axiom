---
schema_id: escalation-packet
version: 1.0
description: Schema for Extended team escalation packets to Core team
created: 2026-01-26
source: mixed
---

# Escalation Packet Schema

> Structured data format for Extended team to escalate concerns to Core team.

**Purpose:** Replace unstructured "please review" comments with evidence-based escalation packets that enable learning and traceability.

**Authority context:** Extended team personas (Escalating or Advisory tier) use this schema to surface concerns requiring Core team review. See [TEAM.md](../TEAM.md) for team roster and authority tiers.

---

## Schema Definition

### Metadata Block

Required identifying information for tracking and routing escalations.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `escalation_id` | string | ESC-YYYY-NNN | Yes | Unique identifier for tracking this escalation across its lifecycle |
| `escalated_by` | string | persona_id from TEAM.md | Yes | Extended team persona submitting this escalation (must be Extended tier) |
| `escalated_at` | timestamp | ISO 8601 | Yes | When this escalation was filed (YYYY-MM-DDTHH:MM:SSZ) |
| `gate_id` | string | gate_id from GATES.md | Yes | Which gate this escalation applies to (pr-merge, release-candidate, etc.) |
| `eval_id` | string | eval filename | Yes | Which eval surfaced this concern (e.g., sustainability-eval, regulator-lens-eval) |
| `severity` | enum | P0 \| P1 \| P2 | Yes | Priority tier determining review SLA (see Severity SLA section) |

**Field notes:**

- `escalation_id`: Use format ESC-YYYY-NNN where YYYY is year and NNN is zero-padded sequence number (e.g., ESC-2026-001)
- `escalated_by`: Must reference Extended tier persona from TEAM.md (Core personas are already blocking, don't escalate)
- `gate_id`: Must reference valid gate from GATES.md where escalation_allowed: yes
- `eval_id`: Should match filename of eval that detected concern (links escalation to eval that surfaced it)

### Concern Block

Core content describing the risk and evidence supporting escalation.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `risk_prevented` | string | plain text | Yes | What bad outcome this escalation prevents (focus on impact, not mechanism) |
| `evidence` | list | URLs, file paths, log excerpts | Yes | Links, logs, test results, commit refs proving concern validity |
| `stop_rule` | string | plain text | Yes | What would satisfy this escalation (clear resolution criteria) |
| `confidence` | enum | high \| medium \| low | Yes | How certain is escalator that concern is valid (helps Core prioritize) |

**Field notes:**

- `risk_prevented`: State harm in user/business terms (e.g., "users waste bandwidth" not "binary too large")
- `evidence`: Each item should be verifiable (link to commit, log line with timestamp, test output, metrics screenshot)
- `stop_rule`: Must be specific and measurable (e.g., "binary size <20MB" not "optimize binary")
- `confidence`: High = escalator investigated thoroughly and concern is clear; Medium = investigation done but uncertainty remains; Low = preliminary concern needing Core expertise

### Context Block

Background information helping Core team understand investigation depth and history.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `investigation_summary` | string | plain text | Yes | What Extended team already checked (proves homework was done) |
| `related_escalations` | list | escalation_id list | No | Prior escalation IDs on same topic (shows pattern or precedent) |
| `recommended_disposition` | enum | approve \| deny \| conditional | No | Extended's suggestion (not binding, helps Core understand escalator's view) |

**Field notes:**

- `investigation_summary`: Demonstrate due diligence (e.g., "Analyzed dependency tree, tested with minimal features, all tests pass")
- `related_escalations`: Link to prior escalations if this is repeat concern (helps Core see if guidance was followed)
- `recommended_disposition`: Advisory only - Core makes final decision but escalator's view helps frame review

---

## Severity SLA

Escalation priority determines review target time. Core team should aim to provide disposition feedback within these windows.

| Severity | Definition | Review Target | Use When |
|----------|------------|---------------|----------|
| **P0** | Immediate blocking concern | 4 hours | Critical vulnerability, legal violation, production outage imminent |
| **P1** | Same-day review needed | 24 hours | High-risk concern that blocks current gate, significant user impact |
| **P2** | Weekly review acceptable | 1 week | Important concern but not immediately blocking, strategic issue |

**SLA notes:**

- **P0 escalations are rare.** Reserve for genuine emergencies (security breach, legal exposure, production down).
- **P1 is typical high-priority.** Most blocking concerns at release gates fall here.
- **P2 for strategic concerns.** Long-term sustainability, future risk, non-blocking observations.
- SLAs are targets, not guarantees. Core team prioritizes based on severity and available capacity.
- Escalator should select severity honestly - inflating severity for faster response degrades system trust.

---

## Field Rationale

Why each field exists and what problem it solves.

### Why `escalation_id`?

**Problem:** Without unique identifiers, escalations are lost in comment threads.

**Solution:** ID enables tracking escalation from submission through disposition through follow-up. Supports metrics (approval rate, time to disposition) and precedent linking.

**Source:** NIST SP 800-61r3 incident tracking requirements, ITIL escalation management.

### Why `severity` tiers?

**Problem:** Flat escalation priority means critical issues buried in routine concerns (escalation fatigue).

**Solution:** P0/P1/P2 severity enables risk-based routing. Core team can triage P0 immediately while P2 waits for weekly review.

**Source:** NIST escalation patterns, industry-standard severity classification.

### Why `evidence` required?

**Problem:** "Please review" escalations without investigation create noise. Core team must do Extended's homework.

**Solution:** Evidence requirement forces Extended to investigate before escalating. Links, logs, and test results prove concern validity.

**Source:** 2026 compliance shift to evidence-based governance (defensible oversight, not theater).

### Why `stop_rule`?

**Problem:** Open-ended escalations never converge. Core doesn't know when escalation is satisfied.

**Solution:** Stop rule clarifies resolution criteria. Core can approve with clear understanding of what Extended needs.

**Source:** GATES.md stop rules, convergence criteria for iterative review.

### Why `confidence` signal?

**Problem:** Core team treats all escalations equally. High-confidence concerns should prioritize.

**Solution:** Confidence level helps Core triage. High confidence = escalator did thorough investigation. Low confidence = preliminary concern needing Core expertise to validate.

**Source:** Risk assessment frameworks (likelihood × impact), escalation management best practices.

### Why `investigation_summary`?

**Problem:** Extended escalates without doing homework. Core must investigate from scratch.

**Solution:** Investigation summary proves Extended did due diligence. Shows what was already checked, preventing duplicate work.

**Source:** ITIL escalation best practices (escalator should exhaust L1 troubleshooting before L2 escalation).

### Why `related_escalations`?

**Problem:** Repeat escalations on same topic suggest Extended didn't learn from prior Core feedback.

**Solution:** Linking related escalations shows whether guidance was followed. Helps Core identify coaching opportunities.

**Source:** Feedback loop patterns, learning organization practices.

### Why `recommended_disposition`?

**Problem:** Extended feels powerless - just filing concerns without stating view.

**Solution:** Recommendation field lets Extended state opinion (not binding). Helps Core understand escalator's framing.

**Source:** Collaborative decision-making, respecting Extended expertise while preserving Core authority.

---

## Cross-References

This schema references and is referenced by other governance documents.

### References to Other Documents

- **TEAM.md**: `escalated_by` must be Extended tier persona_id from TEAM.md roster
- **GATES.md**: `gate_id` must be valid gate from GATES.md where escalation_allowed: yes
- **Eval files**: `eval_id` should match eval filename in evals/ directory

### Referenced by Other Documents

- **disposition-feedback.md**: Disposition feedback schema links to escalation via `escalation_id`
- **Future escalation history tracking**: Metrics and trends will aggregate by escalation_id

### Validation Checks

Manual validation commands to verify schema compliance:

```bash
# Verify escalated_by is Extended persona
grep "escalated_by:" escalation-packet-instance.yaml | grep -f <(grep "Extended" ../TEAM.md | awk -F'|' '{print $2}' | tr -d ' ')

# Verify gate_id is valid gate
grep "gate_id:" escalation-packet-instance.yaml | grep -f <(grep "| gate_id |" ../GATES.md -A 10 | awk -F'|' '{print $2}' | tr -d ' ')

# Verify severity is P0, P1, or P2
grep "severity:" escalation-packet-instance.yaml | grep -E "P[0-2]"
```

---

## Example Usage

See research file `.planning/phases/04-escalation-structure/04-RESEARCH.md` for complete examples:

- Example 1: P1 escalation for binary size bloat (approved with conditions)
- Example 2: P1 escalation for GDPR concern (denied with precedent reference)

**Template location:** `.planning/templates/escalation-packet-template.yaml` (to be created in Phase 4 implementation)

---

*Schema version: 1.0*
*Last updated: 2026-01-26*
