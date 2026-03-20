# Converge — Roadmap

**Author:** Morgan Vale, CEO
**Date:** March 12, 2026
**Status:** Draft for Kenneth's approval
**Time unit:** Converge Week (cw) = 1 calendar week. cw1 starts March 16, 2026.

---

## How to Read This Roadmap

Each milestone has:
- **What "done" looks like** — concrete, verifiable criteria
- **Who owns it** — the person accountable for delivery
- **Target date** — in Converge Weeks (cw) and calendar dates
- **How we celebrate** — because milestones matter

---

## Milestone 1: Wave 1 Complete (Foundation Proven)

**Target:** cw3 (March 30, 2026)
**Owner:** Eli Marsh (engineering), Ren Akiyama (review)

### Done when:
- [ ] converge-core has 5-6 proof examples that pass with property-based tests
- [ ] converge-traits v0.3.0 is frozen and published (DONE)
- [ ] LlmAgent idempotency bug is fixed or has deterministic workaround
- [ ] A new developer reads the proof examples and understands convergence in 15 minutes
- [ ] converge-business story audit is complete (DONE)
- [ ] CI/CD pipeline runs green for converge-core + converge-traits

### How we celebrate:
Eli writes a short "What convergence actually is" post for the team. We publish it as the first Backstage blog post. The foundation is real.

---

## Milestone 2: converge.zone Business-Ready

**Target:** cw2 (March 29, 2026)
**Owner:** Blake Harmon (content), Jules Carrera (implementation), Rio Castellan (design)

### Done when:
- [ ] Pricing page live on converge.zone with CEO-approved tiers
- [ ] Business-buyer landing page (/for/operations) live with outcome-led messaging
- [ ] Security one-pager page live
- [ ] About Us page live (DONE)
- [ ] Interactive demos working (DONE)

### How we celebrate:
Blake sends the first LinkedIn outreach message linking to the live pricing page. The website sells, not just explains.

---

## Milestone 3: First Design Partner Signed (LOI)

**Target:** cw4 (April 6, 2026)
**Owner:** Blake Harmon (pipeline), Kenneth Pernyer (warm intros)

### Done when:
- [ ] At least 1 company has signed a pilot charter + LOI
- [ ] Pilot owner named at the partner company
- [ ] Kickoff date scheduled
- [ ] Success criteria agreed (2+ metrics from the pilot framework)

### How we celebrate:
Morgan announces to the full team. Name the partner internally. This is the moment Converge stops being a project and starts being a product.

---

## Milestone 4: Pilot Runtime Ready

**Target:** cw5 (April 10, 2026)
**Owner:** Ren Akiyama (engineering lead), Eli Marsh (critical path)

### Done when:
- [ ] MVP-1: Convergence engine runs end-to-end (3+ agents, <60s)
- [ ] MVP-2: Observation UI shows live agent activity on converge-www
- [ ] MVP-3: At least one webhook integration works (inbound trigger + outbound API)
- [ ] MVP-4: HITL gate works (pause/approve/reject via Slack + API)
- [ ] MVP-5: Audit trail captures full run provenance
- [ ] MVP-6: Telemetry captures cycle time, convergence time, agent execution time
- [ ] MVP-7: No customer data leaves their environment
- [ ] End-to-end integration test passes
- [ ] Pilot runbook exists and has been dry-run

### How we celebrate:
Ren presents a live demo to the full team — real webhook trigger, real convergence, real HITL gate, real audit trail. Kenneth gives the go/no-go for pilot launch.

---

## Milestone 5: First Pilot Running

**Target:** cw6 (April 13, 2026)
**Owner:** Blake Harmon (partner relationship), Leo Marin (technical support)

### Done when:
- [ ] Design partner is running 2-3 real workflow executions (not hypotheticals)
- [ ] Telemetry is collecting baseline and pilot metrics
- [ ] Weekly working sessions are scheduled and happening
- [ ] Partner has a shared Slack channel with Converge team

### How we celebrate:
Real data flowing through real infrastructure for a real customer. Blake shares the first telemetry snapshot with the team. We're live.

---

## Milestone 6: First Case Study Published

**Target:** cw6-cw7 (April 20-26, 2026)
**Owner:** Blake Harmon (narrative), Sam Okafor (data), Caroline Ashford (editorial)

### Done when:
- [ ] Pilot 1 hits success criteria (improvement on 2+ metrics)
- [ ] Before/after analysis completed within 5 days of pilot end
- [ ] Anonymized case study written, reviewed by Caroline, approved by Kenneth
- [ ] Published on converge.zone
- [ ] Partner has given permission to reference (even if anonymized)

### How we celebrate:
The case study goes on the homepage. We have proof, not promises. Blake uses it in every outreach message from here on.

---

## Milestone 7: First Paid Contract

**Target:** cw8 (May 4, 2026)
**Owner:** Blake Harmon (sales), Priya Chandran (commercial terms)

### Done when:
- [ ] At least 1 design partner converts from free pilot to paid Professional tier
- [ ] Contract signed (MSA + subscription agreement)
- [ ] First invoice sent
- [ ] Production workspace provisioned
- [ ] Support SLA activated

### How we celebrate:
First revenue. Morgan sends a note to the team and Kenneth. We frame the first invoice (metaphorically). This validates the entire thesis.

---

## Milestone 8: Wave 2 Complete (Provider Integrations)

**Target:** cw5 (April 10, 2026) for converge-provider; cw8+ for remaining Wave 2 crates
**Owner:** Kira Novak (provider, llm), Ren Akiyama (wave coordination)

### Done when:
- [ ] converge-provider: Anthropic + OpenAI working with wiremock tests, no API keys needed
- [ ] converge-llm: Burn inference working
- [ ] Each Wave 2 crate has at least one example agent participating in convergence
- [ ] `cargo test` passes across all Wave 2 crates

### How we celebrate:
Kira demos a convergence loop with real LLM providers (mocked for safety). The platform can talk to the world.

---

## Milestone 9: Three Paying Customers

**Target:** cw12 (June 5, 2026)
**Owner:** Blake Harmon (pipeline), Morgan Vale (strategy)

### Done when:
- [ ] 3+ companies on signed production contracts (Professional tier or higher)
- [ ] MRR is $1,047+ (3 x $349/mo)
- [ ] At least 2 published case studies (anonymized is fine)
- [ ] At least 1 customer referral received
- [ ] Churn: zero (no pilot-to-contract losses in first 90 days)

### How we celebrate:
Retrospective with the full team. Kenneth reviews the 90-day scorecard. We publish a "Converge at 90 days" internal memo. We plan the next 90 days.

---

## Timeline Summary

```
cw1  Mar 16  ─── Outreach begins (Kenneth warm intros + Blake LinkedIn)
cw2  Mar 23  ─── converge.zone business-ready ★ Milestone 2
cw3  Mar 30  ─── Wave 1 complete ★ Milestone 1
cw4  Apr  6  ─── First design partner signed ★ Milestone 3
cw5  Apr 13  ─── Pilot runtime ready ★ Milestone 4
cw6  Apr 20  ─── First pilot running ★ Milestone 5
cw7  Apr 27  ─── First case study published ★ Milestone 6
cw8  May  4  ─── First paid contract ★ Milestone 7
     May  4  ─── Wave 2 complete (core crates) ★ Milestone 8
cw9  May 11  ─── Second pilot converting
cw10 May 18  ─── Community talk (RevGenius/Pavilion)
cw11 May 25  ─── Case study #2 published
cw12 Jun  5  ─── 3 paying customers ★ Milestone 9
```

---

## Post-cw12 Horizon (Waves 3-5)

These are deliberately coarse. Sequencing depends on pilot feedback.

| Wave | Crates | Earliest Start | What It Unlocks |
|------|--------|---------------|-----------------|
| Wave 3: Tooling | converge-tool, converge-domain, converge-experience | cw6 | JTBD compiler, domain modules, experience store |
| Wave 4: Infrastructure | converge-runtime | cw8 | WASM dynamic module loading |
| Wave 5: Experience | converge-remote, converge-application, converge-personas | cw10 | gRPC client, reference app, persona evals |

We will refine these after the first pilot tells us which capabilities customers actually pull for.

---

## Key Dependencies

| Dependency | Blocks | Owner | Risk |
|------------|--------|-------|------|
| Kenneth approves pricing | Milestone 2, 3 | Kenneth | HIGH — pricing page can't go live |
| Kenneth identifies warm intro targets | Milestone 3 | Kenneth | HIGH — primary pipeline source |
| Eli finishes converge-core proof examples | Milestone 1, 4 | Eli | MEDIUM — on track but single point of failure |
| HITL gate implementation | Milestone 4 | Eli | MEDIUM — strict MVP scope controls risk |
| Design partner says yes | Milestone 5, 6, 7 | Blake + Kenneth | HIGH — external dependency |

---

## Open Questions for Kenneth

1. **Are these milestone dates aggressive enough?** We could compress cw1-cw5 if Kenneth's warm intros land fast. We could also slip if Eli's critical path hits surprises.
2. **Should we run pilots sequentially or overlapping?** Overlapping is riskier (more support load on Blake) but gets us to 3 contracts faster.
3. **What's the hard deadline for first revenue?** Is cw12 (June 5) a target or a constraint?

---

*Morgan Vale, CEO — March 12, 2026*
