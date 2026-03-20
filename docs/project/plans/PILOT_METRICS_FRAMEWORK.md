# Pilot Metrics Framework

> Instrument every pilot from day 1. The first 3-4 customers ARE the case studies.

**Owner (measurement):** Sam Okafor, QA Engineer
**Owner (narrative):** Blake Harmon, VP Marketing & Sales
**Status:** Active
**Created:** 2026-03-11

---

## 1. Purpose

Every Converge pilot must produce quantifiable before/after data. This framework defines what we measure, when we measure it, and how we turn raw numbers into publishable case studies. Anonymized results ship the week the pilot ends.

## 2. Core Metrics

### 2.1 Cycle Time Metrics

| Metric | Definition | Unit | Capture Method |
|--------|-----------|------|----------------|
| **End-to-end cycle time** | Time from trigger event to completed outcome | Minutes/Hours | Timestamp diff (trigger_ts - completion_ts) |
| **Decision latency** | Time from data availability to decision made | Minutes | Timestamp diff (data_ready_ts - decision_ts) |
| **Convergence time** | Time from engine start to fixed-point reached | Seconds | Engine telemetry (converge-experience store) ¹ |
| **Iteration count** | Number of convergence cycles to reach fixed point | Count | Engine telemetry ¹ |

> ¹ **Sequencing note:** Convergence time and iteration count require the converge-experience metrics exporter (Wave 3). Pilots before Wave 3 can measure Sections 2.2–2.3 (responsiveness and efficiency) only. Section 4.1 engine telemetry is also Wave 3-dependent.

### 2.2 Responsiveness Metrics

| Metric | Definition | Unit | Capture Method |
|--------|-----------|------|----------------|
| **Lead response time** | Time from lead creation to first meaningful action | Minutes | CRM event diff |
| **Escalation response time** | Time from escalation trigger to resolution start | Minutes | Workflow event diff |
| **Insight-to-action gap** | Time from insight surfaced to action taken | Minutes | Platform telemetry + CRM |

### 2.3 Efficiency Metrics

| Metric | Definition | Unit | Capture Method |
|--------|-----------|------|----------------|
| **Manual steps eliminated** | Count of human-manual steps replaced by converge agents | Count | Before/after workflow mapping |
| **Automation rate** | % of workflow steps handled by converge vs manual | Percentage | (automated_steps / total_steps) * 100 |
| **Rework rate** | % of outputs requiring human correction | Percentage | (corrections / total_outputs) * 100 |
| **Throughput** | Volume of items processed per time unit | Items/hour | Count over time window |

### 2.4 Quality Metrics

| Metric | Definition | Unit | Capture Method |
|--------|-----------|------|----------------|
| **Convergence success rate** | % of runs reaching valid fixed point | Percentage | Engine telemetry |
| **Invariant violation rate** | Policy/constraint violations per run | Count/run | Cedar policy logs |
| **Output accuracy** | Human-rated quality of converged outputs | 1-5 scale | Structured review form |
| **Determinism score** | Same root intent + same initial context produce same outputs across N runs (LLM responses may vary) | Percentage | Replay-and-compare test |

> **Determinism note:** "Same inputs" means same root intent and same initial context — LLM response variance is *not* controlled. This measures end-to-end determinism including LLM nondeterminism, which is the useful signal. Scores <100% are expected and informative; the goal is to measure how much LLM variance affects final convergence outcomes.

## 3. Measurement Protocol

### 3.1 Baseline Period (Week 0 -- before Converge)

1. **Workflow mapping:** Document the customer's current process end-to-end. Capture every step, who does it, and how long it takes.
2. **Instrument existing tools:** Add timestamps to CRM, email, spreadsheet workflows using lightweight hooks (Zapier, webhooks, or manual logs).
3. **Collect 2 weeks of baseline data** for each core metric.
4. **Deliverable:** `baseline-report-{customer-id}.json` with structured metric snapshots.

```json
{
  "customer_id": "anon-001",
  "baseline_period": { "start": "2026-03-01", "end": "2026-03-14" },
  "metrics": {
    "cycle_time_avg_minutes": 480,
    "lead_response_time_avg_minutes": 120,
    "manual_steps_count": 14,
    "automation_rate_pct": 15,
    "throughput_items_per_day": 8
  }
}
```

### 3.2 Pilot Period (Weeks 1-4 -- with Converge)

1. **Instrument converge-runtime:** Enable telemetry export on all pilot runs. Every convergence cycle emits:
   - `run_id`, `start_ts`, `end_ts`, `cycle_count`, `agent_count`, `success`, `invariant_violations`
2. **Instrument integration layer:** Log CRM/external system events with timestamps for response time metrics.
3. **Weekly metric snapshots:** Automated collection every Friday at 17:00 UTC.
4. **Deliverable:** `pilot-week-{n}-{customer-id}.json` per week.

### 3.3 Post-Pilot (Week 5)

1. Compute before/after deltas for every metric.
2. Run statistical significance checks (paired t-test or Wilcoxon for small samples). **When N < 8 data points**, report descriptive statistics only (median, IQR, before/after delta) without claiming statistical significance.
3. Generate anonymized case study draft.
4. Handoff to Blake for narrative packaging.

## 4. Instrumentation Spec

### 4.1 Converge Engine Telemetry

The converge-experience store already captures convergence events. We need a metrics exporter that produces:

```
converge_run_duration_seconds{customer, job_type} -- histogram
converge_run_cycles_total{customer, job_type} -- counter
converge_invariant_violations_total{customer, job_type, severity} -- counter
converge_agent_execution_seconds{customer, agent_type} -- histogram
converge_convergence_success{customer, job_type} -- gauge (0 or 1)
```

Format: JSON lines or Prometheus exposition format. No external dependencies required for MVP -- file-based export is fine.

### 4.2 Integration Telemetry

For each customer integration (CRM, email, etc.):

```
integration_event_received_at -- ISO 8601 timestamp
integration_action_taken_at -- ISO 8601 timestamp
integration_event_type -- string enum
integration_source -- string (crm, email, webhook)
```

### 4.3 Data Storage

- **Raw:** JSON lines files per customer, per week. Stored in `pilot-data/{customer-id}/`.
- **Aggregated:** Weekly summaries in structured JSON.
- **Reports:** Markdown + embedded charts (Mermaid or static images).

## 5. Anonymization Rules

Before any data leaves the pilot:

1. Replace customer name with `anon-{NNN}`.
2. Replace all PII (names, emails, phone numbers) with hashed tokens.
3. Round all volume metrics to nearest 5 (prevents fingerprinting small businesses). **k-anonymity floor:** suppress any metric where the rounded value could identify a business with fewer than k=5 candidates in the same industry/size bucket. For very low-throughput businesses (e.g., <5 items/day), report as "< 5" rather than the exact rounded value.
4. Industry and company size are publishable (with customer consent).
5. Exact dates replaced with relative offsets ("Week 1", "Day 3").

## 6. Case Study Template

Each pilot produces a case study following this structure:

```
# [Industry] Company Reduces [Primary Metric] by X% with Converge

## The Challenge
- Industry context (1-2 sentences)
- Specific pain point with quantified cost

## The Approach
- Which Converge capabilities were deployed
- Integration points
- Pilot duration and scope

## The Results
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Cycle time | X hrs | Y hrs | -Z% |
| Lead response | X min | Y min | -Z% |
| Manual steps | N | M | -K |
| Automation rate | X% | Y% | +Z pp |

## Key Insight
- One surprising finding from the pilot (1-2 sentences)
```

**Timeline:** Draft within 3 business days of pilot end. Published within 5.

## 7. Pilot Checklist

### Pre-Pilot
- [ ] Customer workflow mapped (all steps documented)
- [ ] Baseline metrics collection started (2-week window)
- [ ] Converge telemetry enabled and verified
- [ ] Integration hooks deployed
- [ ] Baseline report generated and reviewed

### During Pilot
- [ ] Weekly metric snapshots automated
- [ ] Weekly check-in with customer (15 min)
- [ ] Anomaly alerts configured (>2 std dev from baseline)
- [ ] Rework/correction events logged

### Post-Pilot
- [ ] Before/after delta report generated
- [ ] Statistical significance checked
- [ ] Anonymized case study draft written (Sam)
- [ ] Narrative review and polish (Blake)
- [ ] Customer approval for publication
- [ ] Published to converge.zone

## 8. Success Thresholds

> **Note:** These are initial targets, to be calibrated after Pilot 1. Thresholds will be revised based on actual pilot data. Smaller improvements that don't meet these thresholds may still represent genuine value — these gates determine publishable headline claims, not pilot utility.

A pilot is a **success** if it demonstrates improvement on at least 2 of the following:

| Metric | Minimum Improvement |
|--------|-------------------|
| End-to-end cycle time | -30% |
| Lead response time | -50% |
| Manual steps eliminated | -40% |
| Automation rate | +20 percentage points |
| Throughput | +25% |

A pilot is a **strong success** (publishable headline) if it hits 3+ thresholds.

## 9. Tooling Requirements

| Need | Solution | Status |
|------|----------|--------|
| Engine telemetry export | converge-experience metrics exporter | To build (Wave 3) |
| Integration event logging | Lightweight webhook logger | To build |
| Weekly aggregation | Script: `scripts/pilot-metrics-aggregate.sh` | To build |
| Anonymization | Script: `scripts/pilot-anonymize.sh` | **Done** |
| PII validation | Script: `scripts/pilot-pii-scan.sh` | **Done** |
| CI gate (PII) | Hook: `scripts/pilot-data-precommit.sh` | **Done** |
| Case study generation | Template + manual authoring | Template ready |
| Dashboard | Markdown reports (MVP), web dashboard (later) | Template ready |
| Data disposal | Script: `scripts/pilot-data-dispose.sh` | **Done** |

## 10. Ownership Matrix

| Activity | Owner | Reviewer |
|----------|-------|----------|
| Baseline data collection | Sam Okafor | Customer + Ren |
| Telemetry instrumentation | Sam Okafor + Eli Marsh | Ren Akiyama |
| Weekly metric snapshots | Sam Okafor (automated) | -- |
| Before/after analysis | Sam Okafor | Ren Akiyama |
| Anonymization | Sam Okafor | Ava Petrov |
| Case study draft | Sam Okafor | Blake Harmon |
| Case study narrative | Blake Harmon | Morgan Vale |
| Publication | Blake Harmon | Morgan Vale |
| Data retention enforcement | Sam Okafor | Ava Petrov |
| Deletion confirmation | Sam Okafor | Ren Akiyama |

## 11. Data Retention and Disposal Policy

### 11.1 Retention Periods

| Data Category | Examples | Retention Period | Justification |
|---------------|----------|-----------------|---------------|
| **Raw pilot telemetry** | `pilot-data/{customer-id}/*.jsonl`, baseline reports, integration event logs | **90 days** post-pilot completion | Contains workflow details, timestamps, and CRM event data that is commercially sensitive. Minimizes breach surface. |
| **Anonymized/aggregated data** | Anonymized case study data, weekly summaries with PII removed | **Indefinite** | No PII remains after anonymization. Needed for long-term benchmarking and marketing. |
| **Case study artifacts** | Published case studies, charts, narrative docs | **Indefinite** | Public-facing content, customer-approved. |
| **Integration credentials** | OAuth tokens, webhook secrets, API keys | **Deleted immediately** at pilot end | Credentials must not persist beyond the engagement. |
| **Baseline reports** (raw) | `baseline-report-{customer-id}.json` | **90 days** post-pilot completion | Contains pre-Converge workflow data with potential PII. |
| **Converge engine logs** | Convergence run logs with customer context | **90 days** post-pilot completion | May contain customer-specific context data. |

### 11.2 Retention Timeline

```
Pilot End (Day 0)
  │
  ├─ Day 0: Integration credentials revoked and deleted
  ├─ Day 0: Anonymized case study data extracted
  ├─ Day 1-5: Case study published (anonymized data persists indefinitely)
  ├─ Day 7: Retention clock starts for raw data
  │
  ├─ Day 30: First retention check — notify owner if raw data still accessed
  ├─ Day 60: Second retention check — flag for upcoming deletion
  │
  └─ Day 90: Automated deletion of raw pilot data
       ├─ Delete pilot-data/{customer-id}/ (raw telemetry)
       ├─ Delete baseline-report-{customer-id}.json
       ├─ Delete engine logs with customer context
       ├─ Verify anonymized copies exist
       └─ Log deletion confirmation to audit trail
```

### 11.3 Automated Deletion

Deletion is handled by `scripts/pilot-data-dispose.sh`. The script:

1. **Scans** `pilot-data/` for customer directories past their retention date
2. **Verifies** anonymized copies exist before deleting originals
3. **Deletes** raw data files (JSON lines, baseline reports, engine logs)
4. **Logs** each deletion to `pilot-data/disposal-audit.log` with:
   - Customer ID (anonymized)
   - Files deleted (count and total size)
   - Deletion timestamp (ISO 8601)
   - Operator (automated or manual)
   - Anonymized copy verification status (pass/fail)
5. **Fails safely**: if anonymized copies are missing, the script aborts deletion for that customer and alerts the owner

### 11.4 Audit Trail

Every deletion event produces an append-only audit record:

```json
{
  "event": "pilot_data_disposal",
  "customer_anon_id": "anon-001",
  "pilot_end_date": "2026-04-15",
  "disposal_date": "2026-07-14",
  "retention_days": 90,
  "files_deleted": 47,
  "total_size_bytes": 2340000,
  "anonymized_copy_verified": true,
  "operator": "automated",
  "disposal_script_version": "1.0.0"
}
```

The audit log (`pilot-data/disposal-audit.log`) is:
- **Append-only** — no update or delete operations
- **Retained indefinitely** — proves compliance even after data is gone
- **Contains no PII** — uses anonymized customer IDs only

### 11.5 Customer Notification

The pilot agreement must include:

> **Data Retention Clause:** Raw pilot data (telemetry, workflow events, baseline measurements) is retained for 90 days following pilot completion, then permanently deleted. Anonymized, aggregated performance data (with all personally identifiable information removed) is retained indefinitely for benchmarking purposes. Integration credentials are revoked and deleted at pilot end. You may request early deletion of raw data at any time by contacting your Converge pilot lead.

This clause must be present in every pilot agreement before data collection begins.

### 11.6 Early Deletion

Customers may request early deletion of their raw pilot data at any time. Process:

1. Customer sends written request to pilot lead
2. Sam Okafor runs `scripts/pilot-data-dispose.sh --customer {id} --force`
3. Deletion confirmation sent to customer within 2 business days
4. Audit log records early deletion with `"operator": "customer_request"`

### 11.7 Exceptions

- **Active dispute or legal hold**: Raw data retention extended until resolved. Requires VP Engineering approval.
- **Ongoing support engagement**: If customer converts to paying customer, retention policy resets per the production data policy (to be defined separately).
- **Regulatory request**: Data preserved per applicable law. Legal team notified.
