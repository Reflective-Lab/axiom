# Pilot Cost Tracking Rider (Internal Only)

> **NOT customer-facing.** This is an internal checklist Leo uses during pilot setup and execution to track costs against the $250 cap per pilot (per Priya's pilot commercial terms, REF-38 §7.1).

**Version:** 1.0
**Created by:** Leo Marin, Solutions Engineer
**Requested by:** Priya Chandran, Finance & Operations

---

## Per-Pilot Cost Tracker

**Pilot:** [Company Name]
**Pilot Agreement Date:** [Date]
**Pilot Duration:** [Start] – [End]

### Cost Categories

| Category | Budget | Actual | Notes |
|----------|--------|--------|-------|
| LLM inference costs | $160 max (2,500 runs × $0.064 avg) | $ | Track via telemetry |
| Engineering setup time | $50 max (~1 hr at internal rate) | $ | Onboarding + integration |
| Support time (ongoing) | $40 max (~0.5 hr/week × 4 weeks) | $ | Per touchpoint log |
| **Total** | **$250** | **$** | |

### Run Usage Tracking

| Week | Runs Used | Cumulative | % of Allowance | Burn Rate (runs/day) |
|------|-----------|------------|----------------|---------------------|
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |
| 4 | | | | |

**Overage protocol:** If cumulative usage exceeds 80% (2,000 runs) before Week 3, alert Leo + Priya. Discuss with partner whether to throttle or absorb overage cost.

### Cost Control Checkpoints

- [ ] **Pre-kickoff:** Confirm integration complexity is standard API (per §1.2). If non-standard, scope additional cost with Ren before proceeding.
- [ ] **Week 1:** Review actual LLM cost per run vs. budget estimate. Adjust forecast if needed.
- [ ] **Week 2 midpoint:** Check cumulative cost vs. $250 cap. Escalate to Priya if trending >$200.
- [ ] **Pilot end:** Final cost tally. Record in pilot closeout notes for future pricing calibration.

### Integration Complexity Assessment

| Factor | Standard (included) | Non-standard (requires scoping) |
|--------|---------------------|--------------------------------|
| API connections | REST/GraphQL with existing auth | Custom protocols, legacy SOAP, file-based |
| Number of systems | ≤3 | >3 |
| Data mapping | 1:1 field mapping | Complex transformations, multi-step ETL |
| Auth model | OAuth 2.0, API key | SSO federation, certificate-based, on-prem |

If any factor is non-standard, add scoping discussion to pre-kickoff call and estimate additional engineering hours before committing.

---

## Aggregate Pilot Economics

Track across all active pilots to feed Priya's financial model.

| Pilot | Start | Cost Cap | Actual Cost | Converted? | Production Tier |
|-------|-------|----------|-------------|------------|-----------------|
| [Partner 1] | | $250 | $ | | |
| [Partner 2] | | $250 | $ | | |
| [Partner 3] | | $250 | $ | | |

**Threshold:** If average pilot cost exceeds $200, review with Priya before offering next pilot.

---

*This rider is referenced from the pilot-to-contract playbook §3 and the pilot agreement template (internal use only). Updated whenever pilot commercial terms change.*
