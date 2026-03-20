# Business Voices — Episode 2: "Show Me One Step"

> Track B: The operator's perspective on convergence.

**Host:** Blake Harmon, VP Marketing & Sales
**Technical colour:** Bob Calder, Builder-Experimentalist
**Target audience:** RevOps leads, COOs, founders running 10-50 person teams
**Format:** Dual-host conversation with live demo walkthrough (~12 min)
**Status:** Approved for recording (Caroline sign-off 2026-03-12, minor notes addressed Rev 1)
**Depends on:** Jules Carrera's Lead-to-Cash browser demo (in_review, converge-www `/demo/lead-to-cash`)

---

## Cold Open (45 seconds)

> **Blake:** "Last episode, we talked about what Converge measures. Fourteen manual steps in a typical lead-to-cash pipeline. This episode, we're going to show you one of those steps — the one where deals die."
>
> **Bob:** "Specifically: a discount approval. A rep asks for 15% off to close the deal. In most orgs, that request goes to Slack, sits in a queue, and the prospect signs with someone else while you're waiting for a thumbs-up emoji."
>
> **Blake:** "Today we're going to watch Converge handle it. Live. In the browser. Warts and all."

---

## Segment 1: The Step That Kills Deals (2 minutes)

**The problem.** **[Inferred — composite from common sales operations patterns]** Walk through why discount approvals are a revenue bottleneck:

1. Rep qualifies a lead. ICP score is strong. Intent is high. The prospect is ready.
2. Prospect asks: "Can you do 15% off?" Rep says yes before checking policy.
3. Rep submits a discount request. It goes to... where? Slack? Email? A spreadsheet? The CRM has a field but no workflow.
4. VP Sales is in a board meeting. Won't see the request for 3 hours.
5. Prospect gets a competitor's quote in those 3 hours. Signs by end of day.
6. Deal lost. Not because the discount was wrong — because the approval took too long.

**Key line:** "The discount itself is a governance question. The delay is an operations failure. Most tools solve one or the other. Converge solves both at once."

---

## Segment 2: Watch It Happen (4 minutes)

**Live demo walkthrough.** **[Observed — describes the actual browser demo at `/demo/lead-to-cash`]** This segment is narrated over a screen recording of Jules' Lead-to-Cash demo.

### Pre-HITL flow (narrated, ~30 seconds of demo time)

> **Bob:** "Here's a lead coming in. Sarah Chen, VP RevOps at Acme Corp. $4.2M ARR SaaS company. She filled out the demo form."

Walk through the first three agents:
- **LeadIngestion** captures the raw lead + source facts. "Notice the log — it's recording every fact. Lead source, UTM campaign, contact details. This is the provenance chain starting."
- **Enrichment** queries CRM and firmographic data. "ICP score: 0.87. Intent: 0.92. Tier A prospect."
- **Qualification** confirms the lead is fast-track eligible. "The invariant check: minimum ICP for fast-track is 0.80. She's above it. Green light."

> **Bob:** "Three agents, maybe 4 seconds — that's browser-local simulation speed, not production latency; we'll come back to that caveat. A human doing this manually — pulling CRM data, checking firmographics, running the scoring model, updating the record — that's 20-30 minutes if everything's in the right place. Which it usually isn't."

### The HITL gate (narrated, ~60 seconds including pause)

> **Blake:** "And now the demo stops."
>
> **Bob:** "This is the moment. PolicyGuard detected a 15% discount request on a $42,000 deal. Company policy says anything over 10% needs VP approval. So Converge doesn't just flag it — it stops the entire pipeline and asks a human."

Walk through the popup:
- Wine-colored border = problem/decision required (matches brand: wine = pain)
- Shows the deal context: company, discount %, deal value, ICP/intent scores
- Two clear choices: Approve or Reject
- **No ambiguity. No Slack thread. No email chain. No "I'll get back to you."**

> **Blake:** "This is what governance actually looks like. Not a policy document in a drawer. A decision point that fires the moment it's needed, with all the context right there."
>
> **Bob:** "And notice what's in the popup — ICP 0.87, Intent 0.92, Tier A. The VP isn't making this decision blind. They have the scoring data, the deal value, the policy threshold. In most orgs, the VP gets a Slack message that says 'can we do 15% for Acme?' and has to go dig up the context themselves."

**Click approve.** Demo continues.

### Post-HITL flow (narrated, ~30 seconds of demo time)

> **Bob:** "Approval logged. Watch the provenance — `discount_approved, vp_sales, 15%`. That's an audit trail. Now OpportunityBuilder creates the deal at $35,700. QuoteEngine generates the proposal. ContractAgent assembles the agreement. BillingAgent processes the invoice."

> **Blake:** "Four more agents, another 4 seconds. Total time from lead to invoice: under 15 seconds. With one human decision point that took maybe 10 seconds because the context was already there."

---

## Segment 3: What You Just Saw (2 minutes)

Break down what happened in terms the audience cares about. **[Observed — metrics from the demo; Inferred — business impact claims]**

### The numbers

| What | Manual process | Converge demo |
|------|---------------|---------------|
| Lead to qualified | 20-30 min | 4.6 seconds |
| Discount approval | 3 hours (avg) | 10 seconds (HITL) |
| Quote to invoice | 1-2 days | 4.6 seconds |
| Total steps | 14 manual | 9 automated + 1 human |
| Audit trail | Scattered across 4 tools | Single provenance chain |

**[Speculative — manual process estimates are industry composites, not measured from a specific customer. Converge demo times are from the simulation, not production.]**

> **Blake:** "I want to be clear — the demo is a simulation. Those 4-second execution times are the browser running locally, not a production system under load. But the step count is real. The governance gate is real. The provenance chain is real. The architecture that makes this possible — that's what we're selling."

### The governance story

> **Bob:** "Six invariants were checked during this flow. ICP threshold. Discount policy. Provenance chain integrity. Margin floor at 65%. Legal terms present. Revenue recognition compliance. All six passed. All six are logged. If an auditor asks 'why did you give Acme 15% off?' — here's the trace. Every fact, every check, every decision, timestamped."

---

## Segment 4: What the Demo Doesn't Show (90 seconds)

Honest limitations. **[Observed — acknowledged gaps]**

- "This is a client-side simulation. No backend API calls. No real CRM integration. No actual LLM inference. The agents are scripted, not converging." **[Observed]**
- "In production, convergence time would depend on LLM response latency, CRM API speed, and the number of agents participating. We're building toward that — it's Wave 3 work." **[Observed — Wave 3 dependency]**
- "The demo has one scenario. Sarah Chen, Acme Corp, 15% discount. Real deployments need to handle edge cases: what if ICP is borderline? What if the discount is 25%? What if the lead data is incomplete?" **[Inferred — edge cases we'll need to address]**
- "We also haven't shown what happens when the VP rejects the discount. The demo supports it — try clicking Reject — but we didn't walk through it today. That's a different episode." **[Observed]**

**Key line:** "We'd rather show you a simulation and tell you it's a simulation than show you a slide deck and pretend it's a product."

---

## Segment 5: Try It Yourself (60 seconds)

> **Blake:** "The demo is live at converge.zone/demo/lead-to-cash. No login. No signup. Just click Start and watch it run." **[Production note: confirm with Jules and Dex that `/demo/lead-to-cash` is publicly accessible before recording. If pending build verification, hold this CTA.]**
>
> **Bob:** "Click Approve and see what happens. Click Reject and see what changes. Look at the convergence log. Count the invariants. This is the trace — the whole point of Converge is that you can see every step."
>
> **Blake:** "And if you want to talk about what a pilot looks like for your team — the metrics we'd measure, the baseline we'd capture — reach out. We'll walk you through Episode 1's framework with your numbers."

---

## Outro (30 seconds)

> **Blake:** "That was one step. One discount approval. One governance gate. Multiply that by 14 steps in your pipeline and you start to see why we built this."
>
> **Bob:** "Next episode: we break something. What happens when an invariant fails? When the margin floor gets violated? When an agent produces garbage? Show me the trace — including the ugly parts."
>
> **Blake:** "I'm Blake Harmon." **Bob:** "I'm Bob Calder. Show me the trace."

---

## Production Notes

- **Total runtime target:** 10-12 minutes
- **Tone:** Conversational, demo-forward. "Watch this" energy.
- **Audio format:** Dual-host. Blake frames the business context; Bob narrates the demo.
- **Visual companion:** Screen recording of `/demo/lead-to-cash` running. The recording IS the episode — audio is voiceover, not standalone.
- **Recording dependency:** Jules' demo must pass build verification and Blake's copy review before we can capture. Current status: in_review.
- **Slow mode request:** I've asked Jules to add a `?speed=slow` URL param (2-3 second waits) for narrated capture. Not a blocker — we can capture at normal speed and add pauses in post.
- **Cross-references:**
  - Jules Carrera's Lead-to-Cash demo (converge-www `/demo/lead-to-cash`)
  - Business Voices Episode 1 (`deliverables/business-voices-ep01-outline.md`) — metrics framework context
  - Sam Okafor's Pilot Metrics Framework (`plans/PILOT_METRICS_FRAMEWORK.md`)
  - Blake Harmon — narrative, GTM alignment, co-host
  - Caroline Ashford — editorial review before recording

## Episode 3 Tease

"What happens when things go wrong?" — Walk through an invariant violation, a failed convergence, and how Converge surfaces the problem. Coordinate with Sam Okafor for failure/recovery demo scenarios.

---

## Revision Log

| Rev | Date | Changes |
|-----|------|---------|
| 0 | 2026-03-12 | Initial draft |
| 1 | 2026-03-12 | Addressed Caroline's editorial notes: (1) Segment 1 epistemic label corrected to [Inferred — composite], (2) Segment 2 timing caveat added for browser-local speed, (3) Segment 5 production note added to confirm URL liveness before recording, (4) Ep. 3 coordination with Sam noted. Status: Approved for recording. |
