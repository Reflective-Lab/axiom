---
schema_id: disposition-feedback
version: 1.0
description: Schema for Core team disposition response to escalation packets
created: 2026-01-26
source: mixed
---

# Disposition Feedback Schema

> Structured Core team response to escalation packets showing reasoning and outcome.

**Purpose:** Enable learning loop by showing Extended team why Core made decisions and how to strengthen future escalations.

**Authority context:** Core team personas (Blocking-by-Policy tier) use this schema to respond to escalation packets from Extended team. See [TEAM.md](../TEAM.md) for team roster and authority tiers.

---

## Schema Definition

### Metadata Block

Required identifying information linking disposition to original escalation.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `disposition_id` | string | DISP-YYYY-NNN | Yes | Unique identifier for this disposition decision |
| `escalation_id` | string | ESC-YYYY-NNN | Yes | Links to original escalation packet (from escalation-packet.md) |
| `reviewed_by` | list | persona_id list | Yes | Core persona IDs who reviewed and approved this disposition |
| `reviewed_at` | timestamp | ISO 8601 | Yes | When disposition decision was finalized (YYYY-MM-DDTHH:MM:SSZ) |

**Field notes:**

- `disposition_id`: Use format DISP-YYYY-NNN where YYYY is year and NNN is zero-padded sequence number (e.g., DISP-2026-001)
- `escalation_id`: Must reference valid escalation_id from escalation packet (enables traceability)
- `reviewed_by`: Must be Core tier persona_ids from TEAM.md (only Core has authority to approve dispositions)
- `reviewed_at`: Timestamp when disposition was finalized, not when review started

### Outcome Block

Core decision and reasoning for Extended team.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `disposition` | enum | approved \| denied \| need_more_info \| deferred | Yes | Core team decision on escalation (see Disposition Types section) |
| `rationale` | string | plain text | Yes | Why Core team made this decision (teaches Extended the reasoning) |
| `precedent_references` | list | links to prior dispositions or policies | No | Similar past decisions showing consistency (helps Extended learn patterns) |

**Field notes:**

- `disposition`: See Disposition Types section for when to use each value
- `rationale`: Must explain reasoning in 2-3+ sentences minimum. For denied/need_more_info, explain what was missing or why concern invalid.
- `precedent_references`: Link to prior dispositions (DISP-YYYY-NNN), policy sections (GATES.md §X), or related decisions

### Learning Block

Coaching feedback to help Extended improve escalation quality.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `what_was_strong` | string | plain text | Yes | What made this escalation effective (positive reinforcement) |
| `what_to_improve` | string | plain text | No | How to strengthen future escalations (constructive feedback) |
| `coaching_notes` | string | plain text | No | Guidance for Extended team on this topic (teaching opportunity) |

**Field notes:**

- `what_was_strong`: Always provide positive feedback. Even denied escalations usually have something strong (e.g., "good awareness of GDPR").
- `what_to_improve`: Constructive feedback for growth. For approved escalations, suggest how to make even stronger. For denied, explain what would change decision.
- `coaching_notes`: Teaching moment. Reference documentation, suggest research, explain domain context.

### Action Block

Next steps and ownership following disposition.

| Field | Type | Format | Required | Description |
|-------|------|--------|----------|-------------|
| `next_steps` | string | plain text | conditional | What happens now (required if disposition: approved) |
| `compensating_controls` | list | plain text list | conditional | Mitigations if denied (required if disposition: denied) |
| `follow_up_owner` | string | persona_id | conditional | Who owns next action (required if disposition: approved or deferred) |

**Field notes:**

- `next_steps`: Required for approved dispositions. Numbered list of actions, timeline, and responsible party.
- `compensating_controls`: Required for denied dispositions. Show denial isn't dismissive - alternative mitigations exist.
- `follow_up_owner`: persona_id from TEAM.md (usually Core) who owns executing next steps or follow-up review (for deferred).

---

## Disposition Types

Core team decision outcomes and when to use each.

| Disposition | When to Use | Required Fields | Follow-Up |
|-------------|-------------|-----------------|-----------|
| **approved** | Concern is valid, action will be taken | next_steps, follow_up_owner | Execute action, verify resolution |
| **denied** | Concern not valid or already addressed | compensating_controls | Document existing mitigation |
| **need_more_info** | Evidence insufficient, resubmit with strengthening | what_to_improve | Extended resubmits with additional evidence |
| **deferred** | Valid concern but not blocking current gate | follow_up_owner, rationale for deferral | Review in future gate or planning cycle |

### Approved

**Use when:** Core agrees concern is valid and merits action.

**Example scenarios:**

- Security vulnerability found, patch required
- Binary size bloat confirmed, optimization justified
- Legal compliance gap identified, policy update needed

**Required response:** Clear next steps with timeline and owner. Extended should see their escalation led to concrete action.

### Denied

**Use when:** Core determines concern is not valid or already mitigated.

**Example scenarios:**

- GDPR concern already addressed in privacy policy
- Performance concern within acceptable bounds
- Duplicate escalation of previously resolved issue

**Required response:** Explain why concern doesn't merit action. Provide compensating controls showing alternative mitigations exist. Link precedent if similar escalation was previously denied.

**Important:** Denial is not dismissive. Show respect for Extended's concern by explaining existing mitigation.

### Need More Info

**Use when:** Concern may be valid but evidence is insufficient to decide.

**Example scenarios:**

- Evidence links broken or not provided
- Investigation summary too brief to assess
- Stop rule unclear or unmeasurable

**Required response:** Specific guidance on what evidence would strengthen escalation. Extended can resubmit with additions.

**Important:** This is not rejection. Signal willingness to reconsider with better evidence.

### Deferred

**Use when:** Concern is valid but doesn't block current gate or requires longer-term resolution.

**Example scenarios:**

- Strategic concern not urgent for current release
- Issue requires cross-team coordination beyond gate timeline
- Valid concern but acceptable risk for this release

**Required response:** Explain why deferring (not denying). Assign follow-up owner and timeline for revisit. Extended should see concern is acknowledged, not ignored.

---

## Field Rationale

Why each field exists and what problem it solves.

### Why `disposition_id`?

**Problem:** Without unique identifiers, dispositions are lost in comment threads.

**Solution:** ID enables tracking disposition lifecycle, linking to escalation, and building precedent library. Supports metrics (disposition rate, time to disposition).

**Source:** NIST SP 800-61r3 incident tracking, ITIL service management.

### Why `escalation_id` linkage?

**Problem:** Disposition disconnected from original escalation. No traceability.

**Solution:** Linking via escalation_id creates audit trail. Can track escalation → disposition → action taken → outcome.

**Source:** Governance traceability requirements, compliance audit trails.

### Why `rationale` required?

**Problem:** Core makes decisions without explaining reasoning. Extended cannot learn patterns.

**Solution:** Rationale teaches Extended why decisions are made. Over time, Extended learns which concerns merit escalation and which are already addressed.

**Source:** NIST SP 800-61r3 feedback-driven lifecycle, learning organization practices.

### Why `precedent_references`?

**Problem:** Decisions appear inconsistent. Extended doesn't know if similar concern would be approved or denied.

**Solution:** Linking precedents shows consistency. Extended learns "we denied GDPR concern in DISP-2025-203 for same reason" or "approved similar binary size escalation in DISP-2025-187".

**Source:** Legal precedent systems, knowledge management best practices.

### Why `what_was_strong` required?

**Problem:** Negative feedback discourages escalation. Extended stops escalating even valid concerns.

**Solution:** Always highlight strengths. Even denied escalations have good elements (awareness, investigation effort). Positive reinforcement encourages continued engagement.

**Source:** Coaching and feedback best practices, learning culture principles.

### Why `what_to_improve` optional?

**Problem:** Too much criticism on every escalation feels punitive.

**Solution:** Make optional. For excellent escalations, "what was strong" is sufficient. For weak escalations, provide constructive improvement guidance.

**Source:** Balanced feedback frameworks, growth mindset coaching.

### Why `coaching_notes`?

**Problem:** Extended lacks domain expertise. Repeats same mistakes.

**Solution:** Coaching notes provide teaching moments. Reference documentation, explain domain context, suggest learning resources. Builds Extended capability over time.

**Source:** Mentorship models, capability building in learning organizations.

### Why `compensating_controls` for denials?

**Problem:** Denial feels dismissive. Extended thinks Core doesn't care about concern.

**Solution:** Show denial isn't rejection of concern, just that mitigation already exists. "We denied but here's existing privacy policy that addresses this" respects Extended's vigilance.

**Source:** Risk management frameworks (accept with controls), respectful communication patterns.

### Why `next_steps` for approved?

**Problem:** Extended escalates but never sees outcome. Feels like escalation goes into void.

**Solution:** Clear next steps show escalation led to action. Extended sees their work mattered. Builds trust in escalation system.

**Source:** Action tracking, accountability frameworks, feedback loop closure.

---

## Cross-References

This schema references and is referenced by other governance documents.

### References to Other Documents

- **escalation-packet.md**: `escalation_id` must reference valid escalation packet
- **TEAM.md**: `reviewed_by` and `follow_up_owner` must be persona_ids from TEAM.md
- **GATES.md**: Context for which gates allow escalation (all current gates have escalation_allowed: yes)

### Referenced by Other Documents

- **Future escalation history tracking**: Metrics will aggregate disposition outcomes by type, persona, time to disposition
- **Precedent library**: Future dispositions will reference this as precedent via precedent_references

### Validation Checks

Manual validation commands to verify schema compliance:

```bash
# Verify disposition_id matches escalation_id (same year)
grep "disposition_id:" disposition-instance.yaml | awk '{print $2}' | cut -d'-' -f2
grep "escalation_id:" disposition-instance.yaml | awk '{print $2}' | cut -d'-' -f2

# Verify reviewed_by are Core personas
grep "reviewed_by:" disposition-instance.yaml | grep -f <(grep "Core" ../TEAM.md | awk -F'|' '{print $2}' | tr -d ' ')

# Verify disposition is valid enum
grep "disposition:" disposition-instance.yaml | grep -E "approved|denied|need_more_info|deferred"

# Verify rationale exists (non-empty)
grep "rationale:" disposition-instance.yaml | grep -v "rationale: *$"
```

---

## Example Usage

See research file `.planning/phases/04-escalation-structure/04-RESEARCH.md` for complete examples:

- Example 2: Disposition feedback for approved escalation (binary size bloat)
- Example 3: Disposition feedback for denied escalation (GDPR concern with precedent)

**Template location:** `.planning/templates/disposition-feedback-template.yaml` (to be created in Phase 4 implementation)

---

*Schema version: 1.0*
*Last updated: 2026-01-26*
