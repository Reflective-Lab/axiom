# Support Touchpoint Instrumentation Spec

> Define and instrument every customer interaction during pilots. Data feeds the support cost model and SLA definitions.

**Owner:** Leo Marin, Solutions Engineer
**Reviewer:** Priya Chandran (cost model input)
**Status:** Draft v1.0
**Created:** 2026-03-12

---

## 1. Purpose

We need to know exactly how much effort customer support costs during pilots so we can:

1. Price support tiers accurately (REF-45)
2. Identify which touchpoint types consume the most engineering time
3. Set realistic SLA commitments for production customers
4. Decide when to hire dedicated support staff
5. Feed Blake's pricing model with real cost data

**Principle:** If it's not logged, it didn't happen. Every customer interaction gets captured.

---

## 2. Support Touchpoint Taxonomy

### 2.1 Touchpoint Types

| Type | Channel | Example | Expected Frequency |
|------|---------|---------|-------------------|
| `async_message` | Slack / Teams / Email | "Dashboard isn't loading" | 3-5 per week per pilot |
| `sync_call` | Video / Phone | Scheduled weekly check-in | 1 per week per pilot |
| `ad_hoc_call` | Video / Phone | Unscheduled troubleshooting call | 0-1 per week per pilot |
| `escalation_internal` | Internal Slack | Leo → Engineer: "Integration dropping events" | 0-2 per week per pilot |
| `escalation_customer` | Customer → Exec sponsor | Customer escalates to their leadership | 0-1 per pilot total |
| `onboarding_session` | Video (recorded) | Training session (Phase 4 of onboarding playbook) | 1 per pilot |
| `hitl_support` | Async | Customer needs help with HITL gate decision | 1-3 per week per pilot |
| `integration_fix` | Engineering work | Fix broken webhook, API auth, schema change | 0-2 per week per pilot |
| `config_change` | Engineering work | Adjust agent prompts, thresholds, policies | 1-2 per week per pilot |
| `incident` | All channels | System down, data issue, security event | 0 per pilot (target) |

### 2.2 Severity Levels

| Severity | Definition | Response Target | Resolution Target |
|----------|-----------|----------------|------------------|
| `low` | Question, how-to, cosmetic issue | 8 hours (business) | 24 hours |
| `medium` | Feature not working as expected, workaround exists | 4 hours (business) | 8 hours |
| `high` | Pilot blocked, no workaround | 1 hour | 4 hours |
| `critical` | Data issue, security event, complete system failure | 15 minutes | 1 hour |

### 2.3 Resolution Categories

| Category | Description |
|----------|-------------|
| `answered` | Question answered, no code change needed |
| `config_fix` | Resolved by changing configuration (agent prompts, thresholds, etc.) |
| `bug_fix` | Required code change in Converge |
| `integration_fix` | Required change to customer integration |
| `customer_action` | Required action on customer's side |
| `wont_fix` | Out of scope or not a bug |
| `duplicate` | Same issue as a previous touchpoint |

---

## 3. Logging Schema

Every support touchpoint is logged as a structured record.

### 3.1 Touchpoint Record

```json
{
  "touchpoint_id": "tp-{uuid}",
  "pilot_id": "pilot-{customer}-{date}",
  "customer_name": "string",
  "timestamp_created": "ISO-8601",
  "timestamp_first_response": "ISO-8601 | null",
  "timestamp_resolved": "ISO-8601 | null",
  "type": "async_message | sync_call | ad_hoc_call | escalation_internal | escalation_customer | onboarding_session | hitl_support | integration_fix | config_change | incident",
  "severity": "low | medium | high | critical",
  "channel": "slack | teams | email | video | phone | internal",
  "initiated_by": "customer | converge",
  "subject": "string (one-line summary)",
  "description": "string (details, NO customer PII or data)",
  "resolution_category": "answered | config_fix | bug_fix | integration_fix | customer_action | wont_fix | duplicate",
  "resolution_notes": "string",
  "effort_minutes": {
    "leo": 0,
    "engineer": 0,
    "other": 0
  },
  "satisfaction_rating": "1-5 | null",
  "follow_up_required": "boolean",
  "linked_issue": "string (engineering issue ID if bug_fix or integration_fix) | null",
  "tags": ["onboarding", "integration", "hitl", "metrics", "billing"]
}
```

### 3.2 Data Rules

- **No PII in touchpoint records.** Use customer company name, not individual names. Never log customer data content — only metadata about the interaction.
- **Effort tracking is mandatory.** Every touchpoint must have `effort_minutes` filled in when resolved. This is the primary input to the cost model.
- **Satisfaction is optional but encouraged.** After resolving high/critical touchpoints, ask the customer: "On a scale of 1-5, how satisfied are you with how we handled this?"

### 3.3 Where to Log

**Phase 1 (now, pre-tooling):** Structured markdown file per pilot.

```
agents/solutions-engineer/pilots/{customer-name}/support-log.md
```

Each touchpoint is an entry in a markdown table. Leo updates manually after each interaction.

**Phase 2 (when CRM is chosen):** Migrate to CRM or dedicated support tool. The schema above maps directly to custom fields in most CRM/ticketing systems.

---

## 4. Instrumentation Points

### 4.1 Manual Capture (Leo's Responsibility)

| When | What to Log |
|------|------------|
| Customer sends a message (any channel) | Create touchpoint, set `timestamp_created` |
| Leo responds | Set `timestamp_first_response`, calculate response time |
| Issue is resolved | Set `timestamp_resolved`, fill `effort_minutes`, `resolution_category` |
| Engineering help is needed | Create linked `escalation_internal` touchpoint |
| Customer expresses satisfaction/frustration | Note in `resolution_notes`, update `satisfaction_rating` |

### 4.2 Automated Capture (Future)

When telemetry infrastructure exists (Pilot Metrics Framework, Wave 3):

| Signal | Auto-Logged As |
|--------|---------------|
| HITL gate timeout (customer doesn't respond in > 4 hours) | `hitl_support` touchpoint |
| Convergence failure visible to customer | `incident` touchpoint (severity: high) |
| Customer dashboard login (no activity for > 7 days) | Stall detection alert (not a touchpoint, but triggers engagement check) |
| Integration error (webhook delivery failure) | `integration_fix` touchpoint |

---

## 5. Weekly Support Metrics Report Template

Generated every Friday for internal review. Shared with Priya for cost model input.

```markdown
# Pilot Support Report — Week of {date}

## Pilot: {customer-name}
**Pilot week:** {N} of {total}
**Leo effort this week:** {X} hours
**Engineering effort this week:** {Y} hours

## Touchpoint Summary

| Metric | This Week | Cumulative |
|--------|-----------|-----------|
| Total touchpoints | X | X |
| Async messages | X | X |
| Scheduled calls | X | X |
| Ad-hoc calls | X | X |
| Escalations (internal) | X | X |
| HITL support requests | X | X |
| Integration fixes | X | X |
| Config changes | X | X |
| Incidents | X | X |

## Response Time

| Severity | Target | Actual (median) | Met SLA? |
|----------|--------|-----------------|----------|
| Low | 8 hrs | X hrs | Y/N |
| Medium | 4 hrs | X hrs | Y/N |
| High | 1 hr | X min | Y/N |
| Critical | 15 min | N/A | N/A |

## Resolution Time

| Severity | Target | Actual (median) | Met SLA? |
|----------|--------|-----------------|----------|
| Low | 24 hrs | X hrs | Y/N |
| Medium | 8 hrs | X hrs | Y/N |
| High | 4 hrs | X hrs | Y/N |
| Critical | 1 hr | N/A | N/A |

## Effort Breakdown

| Category | Leo (hrs) | Engineering (hrs) | Total |
|----------|-----------|-------------------|-------|
| Async support | X | X | X |
| Calls | X | X | X |
| Integration work | X | X | X |
| Config changes | X | X | X |
| Incidents | X | X | X |
| **Total** | **X** | **X** | **X** |

## Cost Estimate

- Blended cost per hour: ${rate} (Leo) / ${rate} (engineer)
- Total support cost this week: ${amount}
- Projected monthly support cost: ${amount}
- Per-customer-per-month estimate: ${amount}

## Customer Satisfaction

- Ratings received: {N}
- Average: {X}/5.0
- Trend: {improving / stable / declining}

## Observations

- {What types of support are consuming the most effort?}
- {Are there patterns that could be automated or self-served?}
- {Any process improvements for next week?}

## Action Items

- [ ] {Specific follow-ups}
```

---

## 6. Aggregated Metrics (Across All Pilots)

Once we have 2+ pilots running, generate a monthly aggregate:

| Metric | Target | How It's Used |
|--------|--------|--------------|
| Average touchpoints per pilot per week | Baseline | SLA planning |
| Median response time by severity | Meet targets in §2.2 | SLA tier definitions |
| Median resolution time by severity | Meet targets in §2.2 | SLA tier definitions |
| Leo hours per pilot per week | < 8 hrs | Capacity planning |
| Engineering hours per pilot per week | < 4 hrs | Capacity planning |
| Cost per pilot per month | Track | Pricing input for Priya/Blake |
| Customer satisfaction (avg) | ≥ 4.0/5.0 | Quality signal |
| Top 3 touchpoint types by volume | Identify | Automation/self-serve candidates |
| % of touchpoints requiring engineering | < 30% | Scale readiness |

**Capacity threshold:** When Leo is spending > 30 hours/week on support across all pilots, it's time to hire or restructure. Feed this signal to Ren and Morgan.

---

## 7. Integration with Other Deliverables

| Deliverable | How This Feeds It |
|-------------|------------------|
| **Support cost model (REF-45)** | `effort_minutes` and cost estimates are the primary input |
| **SLA tier definitions (REF-45)** | Response/resolution time actuals define what SLAs we can commit to |
| **Pricing tiers (Blake)** | Cost-per-customer data informs support cost component of each tier |
| **Pilot-to-contract playbook** | Touchpoint data proves the support model works (or doesn't) |
| **Field intelligence** | Touchpoint patterns reveal product gaps and feature requests |
| **Pilot Metrics Framework** | Support metrics are a dimension of overall pilot health |

---

## 8. Implementation Plan

| Phase | When | What |
|-------|------|------|
| **Now** | Pre-first-pilot | Create `pilots/` directory structure, support log template, weekly report template |
| **Pilot 1 kickoff** | Day 1 | Start manual logging; generate first weekly report at Day 7 |
| **Pilot 1 Week 2** | Day 14 | Review logging discipline — are we capturing everything? Adjust taxonomy if needed |
| **Pilot 1 end** | Day 30 | Generate aggregate report; first input to cost model (REF-45) |
| **Post-Pilot 1** | Day 31+ | Retrospective: refine schema, automate what's possible, update this spec |
| **CRM decision** | TBD | Migrate from markdown to CRM/ticketing tool |

---

## 9. Open Questions

1. **CRM/ticketing tool:** What system will we use for production support? Manual markdown works for 1-2 pilots but won't scale.
2. **Customer satisfaction method:** In-line rating (after each touchpoint) vs weekly survey vs post-pilot survey? Recommend in-line for high/critical, post-pilot survey for overall.
3. **Automated HITL timeout logging:** Requires converge-experience events to be queryable. Depends on Sam + Eli's telemetry work.
4. **Billing integration:** When we have usage-based pricing, support touchpoints should correlate with billing events. Requires Priya's finance tooling.
