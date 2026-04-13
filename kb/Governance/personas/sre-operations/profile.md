---
source: mixed
---
# SRE / Operations

## Role

The SRE (Site Reliability Engineer) / Operations persona ensures Converge is reliable, observable, and operable in production. They own reliability, incident response, and operational excellence.

## Responsibilities

1. **Reliability** - Ensure Converge meets reliability targets (SLOs)
2. **Observability** - Implement monitoring, logging, and tracing
3. **Incident Response** - Prepare for and respond to production incidents
4. **Capacity Planning** - Ensure infrastructure meets demand
5. **Operational Excellence** - Runbooks, automation, and operational maturity

## Key Questions They Answer

- Will Converge stay up when users depend on it?
- Can we detect and diagnose problems quickly?
- How do we respond when things go wrong?
- Are we ready for the next 10x of scale?
- What keeps us up at night operationally?

## Reliability Principles

1. **Measure everything** - If you can't measure it, you can't improve it
2. **Automate toil** - Humans for judgment, machines for repetition
3. **Prepare for failure** - Everything fails; plan for it
4. **Gradual rollouts** - Reduce blast radius of changes
5. **Blameless postmortems** - Learn from incidents without blame

## SLO Framework

| Service | SLI | SLO Target | Error Budget |
|---------|-----|------------|--------------|
| API availability | Successful requests / Total requests | 99.9% | 43.2 min/month |
| API latency | p99 response time | <500ms | |
| Data durability | Successful writes / Total writes | 99.99% | |
| Governance accuracy | Correct decisions / Total decisions | 99.9% | |

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Operational Readiness Review | Per Release | [`operational-readiness.md`](operational-readiness.md) |
| Incident Review | Per Incident | [`incident-review.md`](incident-review.md) |
| Capacity Planning | Quarterly | [`capacity-planning.md`](capacity-planning.md) |

## Key Artifacts

- Runbooks
- Incident response playbooks
- SLO dashboard
- Capacity projections
- On-call documentation
- Architecture diagrams (operational view)
