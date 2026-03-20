# Converge Week (cw) — The Company Speed Unit

> Calendar time is the bottleneck, not headcount. Dependencies and decision latency matter more than labor capacity.

**Owner:** Ren Akiyama, VP Engineering
**Status:** Draft v1.0
**Date:** 2026-03-12
**Reviewers:** Morgan Vale (CEO), Kenneth Pernyer (Board)

---

## 1. Definition

**1 Converge Week (1 cw)** = one calendar week of company output, with all active agents executing in parallel.

A Converge Week is not a person-week. It is a *throughput unit* — the total productive output of the entire agent team operating concurrently for 5 business days.

### Why Not Person-Weeks?

| Traditional | Converge |
|------------|----------|
| 1 engineer = 1 person-week of output per calendar week | 13 agents = 13 parallel work streams per calendar week |
| Adding engineers adds coordination overhead | Adding agents adds coordination overhead, but agents coordinate via Paperclip — overhead is lower |
| Calendar time ≈ person-weeks / headcount | Calendar time ≈ critical path length, regardless of parallelizable work |

The key insight: in a traditional team, you can trade headcount for calendar time (up to a point). In an agent team, **parallelizable work is nearly free** — the bottleneck is the **dependency graph** and **decision latency** (time waiting for human review/approval).

## 2. What 1 cw Contains

### 2.1 Current Roster (as of 2026-03-12)

| Team | Agents | Parallel Capacity |
|------|--------|-------------------|
| Engineering | Eli (core/traits), Kira (provider/infra), Jules (frontend), Sam (QA), Dex (DevOps), Ava (security) | 6 streams |
| GTM | Blake (marketing/sales), Rio (design), Caroline (editorial) | 3 streams |
| External Contributors | Alice (systems pragmatist), Bob (builder-experimentalist) | 2 streams |
| Management | Morgan (CEO), Nadia (product), Ren (VP Eng), Priya (finance) | Coordination, not IC output |
| **Total** | **16 agents** | **~11 IC streams + 5 coordination** |

### 2.2 Theoretical Maximum

In a perfect week with no blockers, no decision latency, and no dependencies:

```
1 cw = 11 parallel IC agent-weeks of output
     ≈ 11 person-weeks equivalent (assuming 1 agent ≈ 1 senior engineer)
```

### 2.3 Realistic Expectation

Dependencies, decision latency, and coordination reduce effective output:

```
1 cw (realistic) ≈ 6-8 person-weeks equivalent
```

The gap comes from:
- **Dependency stalls**: Agent B can't start until Agent A finishes (e.g., Kira waits for Eli's traits)
- **Decision latency**: Human review/approval gates (Kenneth reviews plan, Morgan approves pricing)
- **Rework cycles**: Agent output rejected, needs revision
- **Context switching**: Agents reassigned mid-task

## 3. Baseline Measurement

### 3.1 Week 1 Actuals (cw-1: 2026-03-11 to 2026-03-14)

This is our first operational week. Observed output:

| Agent | Completed | In Progress | Blocked Time |
|-------|-----------|-------------|--------------|
| Eli Marsh | converge-traits v0.3.0 audit + stabilize | converge-core proof examples | ~0.5 days (unclear requirements) |
| Kira Novak | Architecture prep for converge-provider | converge-provider implementation | ~2 days (waiting for traits freeze) |
| Jules Carrera | Full lead-to-cash interactive demo | Browser demo review pending | ~0 |
| Sam Okafor | Pilot metrics framework v1.0 | — | ~0 |
| Dex Tanaka | — | Git push fix (REF-33) | First task, ramping |
| Ava Petrov | Security one-pager content, SOC 2 outline + 4 policies | REF-19 SOC 2 continuation | ~0 |
| Blake Harmon | converge-business story audit | Landing page, pricing page, playbook | ~0 |
| Rio Castellan | Design system v1.0, pricing page spec, landing page spec | — | ~0 |
| Alice Mercer | Technical review of pilot metrics framework | Signals article, Tech Voices | Blocked on proof examples |
| Bob Calder | — | TBD | Ramping |
| Nadia Reeves | Pilot Program PRD v1.0 | PRD review | ~0 |
| Priya Chandran | — | Financial model | ~0 |

**cw-1 estimate: ~7-8 person-weeks equivalent** (high parallelism on independent GTM tasks, but Wave 1 dependency chain limited engineering throughput).

### 3.2 Tracking Template

Each week, record:

```
cw-N (YYYY-MM-DD to YYYY-MM-DD)
  Completed items: [count]
  Blocked agent-days: [count]
  Decision latency (avg): [hours]
  Effective throughput: [estimated person-weeks]
  Critical path bottleneck: [description]
```

## 4. Using cw in Planning

### 4.1 Estimation Rules

1. **Estimate in cw, not person-weeks.** "This wave takes 2 cw" means 2 calendar weeks.
2. **Separate parallel work from serial work.** If 4 agents can work in parallel, their combined output in 1 cw is 4 agent-weeks — but it still takes 1 calendar week.
3. **Add decision latency explicitly.** If a milestone requires Kenneth's approval, add 0.5-1 cw for review cycles.
4. **Critical path determines calendar time.** The longest chain of dependent tasks sets the minimum cw, regardless of how much parallel work exists.

### 4.2 Example

```
Wave 2 has 5 crates that can be built in parallel.
Each crate takes ~1 agent-week.
Critical path: converge-provider (blocks nothing else in Wave 2).
Estimate: 1 cw for all 5 crates (parallel).
Add 0.5 cw for QA + review.
Total: 1.5 cw.
```

## 5. Calibration Schedule

| Checkpoint | Date | Action |
|-----------|------|--------|
| cw-1 baseline | 2026-03-14 (end of week 1) | Record actuals, establish first ratio |
| cw-3 calibration | 2026-03-28 | Compare estimates vs actuals, adjust model |
| cw-6 recalibration | 2026-04-18 | Formal review with CEO — is the model predictive? |
| Ongoing | Weekly | Record in daily notes, update running average |

## 6. Comparison to Traditional Teams

| Metric | Traditional (6 engineers) | Converge (16 agents) |
|--------|--------------------------|---------------------|
| Calendar weeks for same output | 1 week | 1 week |
| Person-weeks produced | ~4-5 (meetings, context-switching) | ~7-8 (parallel execution) |
| Coordination cost | High (standups, planning, PRs) | Lower (Paperclip orchestration) |
| Decision latency impact | Moderate | **Dominant bottleneck** |
| Scaling cost | $150K+/engineer/year | Agent compute costs (TBD — Priya modeling) |

---

## Key Takeaway

The Converge Week reframes planning around what actually constrains us: **the dependency graph and decision latency**, not labor capacity. When we say "this takes 3 cw," we mean 3 calendar weeks — and we should immediately ask "what's on the critical path?" not "do we have enough people?"
