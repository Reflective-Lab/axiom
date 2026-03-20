# Converge — Governance Structure

**Author:** Morgan Vale, CEO
**Date:** March 12, 2026
**Status:** Draft for Kenneth's approval

---

## 1. Roles and Authority

### Kenneth Pernyer — Board / Founder

Kenneth is the sole board member and founder. Morgan Vale (CEO) reports directly to Kenneth.

**Kenneth's decision rights:**

- **Strategy approval** — Signs off on STRATEGY.md and any material changes to company direction
- **Capital allocation** — Approves any spending commitment >$5,000 or new recurring costs >$500/month
- **Hiring** — Approves new agent hires (headcount changes)
- **Pricing changes** — Approves tier pricing, discount structures, and contract terms
- **External-facing content** — Final approval on pricing page, landing pages, and public commitments
- **Milestone sign-off** — Confirms completion of major milestones (first pilot, first contract, wave completions)
- **CEO accountability** — Evaluates Morgan's performance against agreed metrics

### Morgan Vale — CEO

Morgan runs the company day-to-day. Reports to Kenneth.

**Morgan's operating authority:**

- Task assignment, prioritization, and workload management across all agents
- Content production within approved messaging and brand guidelines
- Pilot execution decisions (timing, scope adjustments, extensions)
- Technical decisions (delegated to Ren Akiyama, VP Engineering)
- Agent performance management (reassignment, workload redistribution)
- Expenditures under $5,000 and recurring costs under $500/month
- Issue creation, status changes, and project management in Paperclip

**Morgan does NOT have authority to:**

- Change pricing without Kenneth's approval
- Hire or terminate agents without Kenneth's approval
- Commit to external partnerships or contracts without Kenneth's review
- Change company strategy or target market
- Approve spending above the thresholds above

### Direct Reports to Morgan

| Role | Agent | Scope |
|------|-------|-------|
| VP Engineering | Ren Akiyama | Engineering execution, technical architecture, crate development |
| VP Marketing & Sales | Blake Harmon | GTM execution, pipeline, content, pilot management |
| Product Manager | Nadia Reeves | Product requirements, pilot program design, user stories |
| Finance & Operations | Priya Chandran | Financial model, unit economics, commercial terms |

### Ren Akiyama's Engineering Reports

Eli Marsh (Founding Engineer), Kira Novak (Senior Rust), Jules Carrera (Frontend), Sam Okafor (QA), Dex Tanaka (DevOps), Ava Petrov (Security), Leo Marin (Solutions)

### Blake Harmon's GTM Reports

Rio Castellan (Designer), Caroline Ashford (Editor-in-Chief → Alice Mercer, Bob Calder)

---

## 2. Board Cadence

| Cadence | Format | Content | Owner |
|---------|--------|---------|-------|
| **Weekly async update** | Paperclip comment or markdown summary | Metrics snapshot, blockers, decisions needed, spend | Morgan posts by Friday EOD |
| **Bi-weekly deep dive** | Sync conversation (every other week) | Strategy review, pipeline status, key decisions, milestone progress | Kenneth schedules |
| **Milestone review** | Async or sync (as milestones hit) | Milestone criteria met? Evidence? Next milestone? | Morgan presents, Kenneth approves |
| **Quarterly strategy review** | Sync | Full strategy refresh, financial model update, ICP validation | Morgan prepares, Kenneth decides |

---

## 3. Escalation Paths

### What goes to Kenneth (escalate immediately)

- Any risk that threatens the company's ability to land design partners
- Security incidents or data breaches
- Budget overruns >20% of planned spend
- Loss of a design partner or pilot failure
- Legal or compliance issues
- Disagreements between VP-level agents that Morgan can't resolve

### What Morgan decides (inform Kenneth in weekly update)

- Task reprioritization within existing strategy
- Agent workload redistribution
- Content publication (within approved messaging)
- Bug prioritization and technical debt decisions
- Pilot scope adjustments (extensions, reduced scope)

### What team leads decide (inform Morgan)

- Implementation approach within assigned tasks
- Code architecture decisions within their crate scope
- Design choices within the approved design system
- Content drafts (before editorial approval)

---

## 4. Communication Protocol

### How agents communicate with Kenneth

Agents do not communicate directly with Kenneth. All board communication flows through Morgan.

**Path:** Agent → their manager → Morgan → Kenneth

**Exception:** Kenneth may directly assign issues to any agent via Paperclip. The agent should execute and report back through their chain of command.

### How Kenneth communicates decisions

- **Via Paperclip issues:** Kenneth creates issues assigned to Morgan or specific agents
- **Via comments:** Kenneth comments on in_review items to approve, reject, or request changes
- **Via STRATEGY.md updates:** Kenneth may annotate or revise the strategy document directly

### Status visibility

Kenneth has full read access to:
- All Paperclip issues, comments, and status changes
- All plan documents in `plans/`
- All agent memory and deliverables in `agents/`
- Git history and all code changes

---

## 5. Decision Framework

### Two-way doors (reversible — Morgan decides, informs Kenneth)

- Feature prioritization changes
- Agent task reassignment
- Content scheduling
- Pilot timeline adjustments (within 2 weeks)
- Technical architecture within a single crate

### One-way doors (hard to reverse — Kenneth approves)

- Pricing changes
- New agent hires
- External commitments (contracts, partnerships, public statements)
- Strategy pivots
- Capital allocation >$5,000
- Removing or replacing a team lead

---

## 6. Performance and Accountability

### Morgan's 90-day scorecard

| Metric | Target | Measurement |
|--------|--------|-------------|
| Design partners signed | 3-4 LOIs | Signed documents |
| Production contracts | 3-4 | Executed agreements |
| First case study published | By cw6 (Apr 26) | Published URL |
| Wave 1 engineering complete | 100% | All Wave 1 issues done |
| Monthly burn rate | <$5,000/mo | Financial model actuals |
| Team velocity | Increasing week-over-week | Issues completed per week |

Kenneth reviews these metrics in the weekly async update and bi-weekly deep dive.

### Agent performance

Morgan evaluates team leads (Ren, Blake, Nadia, Priya) on delivery against their assigned goals. Team leads evaluate their reports. Performance issues are documented in Paperclip comments and escalated to Morgan if unresolved after one conversation.

---

*Morgan Vale, CEO — March 12, 2026*
