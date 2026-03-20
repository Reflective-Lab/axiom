# Lead-to-Cash in 47 Seconds: What the Demo Shows

> Signals — Content Week 3 | Publish: April 3, 2026
> Author: Bob Calder | Status: Final (Caroline approved)

---

You watched the demo — or someone sent you the link and said "look at this." Eight agents. Nine cycles. One lead goes from website form to paid invoice while you're still deciding what to have for lunch.

Now you want to know: how does it actually work?

This is the walkthrough. Not a product spec. Not a press release. A guided tour of what the demo does, what it proves, and what it doesn't.

---

## 1. What the Demo Runs

The Lead-to-Cash demo orchestrates eight agents through a complete revenue pipeline — from raw inbound lead to collected payment. Each agent owns one stage. Each stage produces facts. Each fact feeds the next agent.

Here's the lineup:

| Agent | Role | What it produces |
|-------|------|-----------------|
| **LeadIngestion** | Capture & normalize | Lead data (company, contact, source, UTM tags) |
| **Enrichment** | CRM + firmographic lookup | ARR, employee count, tech stack, hiring signals |
| **Qualification** | ICP scoring | ICP score (0.87), intent score (0.92), tier (A) |
| **PolicyGuard** | Discount/approval rules | Discount request flagged; HITL barrier triggered |
| **OpportunityBuilder** | Deal creation | Opportunity record with provenance links |
| **QuoteEngine** | Pricing & CPQ | Line-item quote with margin check |
| **ContractAgent** | Terms & signatures | Legal agreement, net-30, auto-renew |
| **BillingAgent** | Invoice & collection | Invoice generated, payment collected |

Nine cycles, not eight — the final cycle is a convergence check. The system confirms no new facts were produced, which means the pipeline has stabilized. That's convergence: the point where every agent has said everything it has to say. **[Observed — verified against `DemoLeadToCash.tsx` source]**

---

## 2. What the Viewer Sees

The UI has three elements:

**The agent grid.** Eight cards, one per agent. Each card shows the agent's name and current state: idle (gray), working (active), or done (green). As the demo runs, you watch the cards light up in sequence — LeadIngestion fires, produces its facts, goes green. Enrichment picks up, produces its facts, goes green. Left to right, top to bottom, like dominoes.

**The convergence log.** A scrolling panel on the right. Every fact, every invariant check, every agent transition gets logged in real time. By the end you have ~35 entries — a complete audit trail of what happened, in what order, and why. Cycle headers in white. Facts in green. Invariant results in blue. HITL decisions in orange. It's dense, but that's the point: nothing is hidden.

**The HITL popup.** Midway through the demo, everything stops. PolicyGuard has detected a 15% discount request on a $42,000 deal — company policy caps automatic approvals at 10%. A modal appears with a wine-coloured border (the brand's visual shorthand for "decision required"), showing the full deal context: company name, discount percentage, deal value, ICP score, intent score, and qualification tier.

Two buttons. Approve or Reject. No ambiguity. No Slack thread. No email chain. The VP Sales sees the context and decides — right there, right then.

Click Approve and the pipeline resumes. The discount is recorded as a fact (provenance tag: `hitl-001`), the deal value drops to $35,700, and four more agents finish the job: opportunity, quote, contract, invoice. Click Reject and standard pricing holds at $42,000. Either way, the decision is logged with full provenance. **[Observed]**

---

## 3. What the 47 Seconds Prove

Three things happened that matter:

**Convergence.** Nine cycles. Eight agents. Each one produced facts. After the eighth agent finished, the system ran one more cycle, confirmed no new facts appeared, and declared convergence. The pipeline stabilized on its own. No orchestrator forced it to stop. No timeout killed it. The agents ran until they had nothing left to say. **[Observed — demo logs show "CONVERGED in 9 cycles"]**

**Invariant enforcement.** Six invariants were checked during the flow:

1. *Minimum ICP for fast-track* ≥ 0.80 — **passed** (0.87)
2. *Discount policy threshold* — 15% exceeds 10% — **HITL barrier triggered**
3. *Provenance chain required* — deal links back to scoring data — **passed**
4. *Margin floor* ≥ 65% — **passed**
5. *Required legal terms present* — annual, net-30, auto-renew confirmed — **passed**
6. *Revenue recognition policy* — invoice and payment recorded — **passed**

Every invariant is visible in the log. None were skipped. None were soft-failed. The system enforced its own rules and showed you the receipts (these are scripted checks in the demo — Section 4 explains what that means and why it matters). **[Observed]**

**Provenance.** If someone asks "why did Acme get 15% off?" — the trace answers it. ICP score 0.87, sourced from Qualification. Intent score 0.92. VP Sales approved via HITL gate (provenance: `hitl-001`). OpportunityBuilder created the deal at $35,700 referencing `icp_score`, `intent_score`, and `enrichment_crm` as justifying evidence. Every fact points to its source. Every decision points to its justification. That's not a dashboard summary — that's an audit trail. **[Observed]**

---

## 4. What the Demo Hides

This is the section that matters most.

**The agents are scripted, not thinking.** Each agent in the demo is a state machine that waits a fixed number of milliseconds, then emits pre-written facts. LeadIngestion waits 1,200ms, then outputs "Acme Corp, VP RevOps." There's no LLM inference. No model is scoring the lead. No enrichment service is querying a real database. The *architecture* is real. The *computation* is simulated. **[Observed — source confirms `await wait(ms)` pattern with hard-coded fact payloads]**

**The speed is artificial.** Total agent execution time is ~8.8 seconds across eight agents. Add the HITL pause and you're at 12-19 seconds wall clock. That's browser-local JavaScript running scripted delays. In production, each agent would hit real APIs — CRM lookups, LLM inference, payment processing — with real latency. A production pipeline would be faster than 14 manual steps, but it would not be 47 seconds. Honest estimate: minutes, not seconds. **[Inferred — based on typical API latencies for CRM, LLM, and payment services]**

**There is exactly one scenario.** Acme Corp. Sarah Chen. 15% discount. $42,000 deal. ICP 0.87. The demo does not show what happens when the ICP score is borderline (0.52). It does not show what happens when enrichment data is missing. It does not show a credit card decline, a contract rejection, or an agent that produces contradictory facts. Those are production realities. The demo handles one golden path and one branch (approve vs. reject). **[Observed]**

**No real systems are connected.** No Salesforce. No HubSpot. No Stripe. No Docusign. The "CRM data" is hard-coded. The "payment" uses Stripe's test card number (4242). The "contract" is a fact in a log, not a PDF sent for signature. Integration is the hard part of any platform — and the demo doesn't touch it. **[Observed]**

We'd rather show you a simulation and tell you it's a simulation than show you a slide deck and pretend it's a product.

---

## 5. What It Would Take in Production

Moving from demo to deployment means closing four gaps:

- **Real agent backends.** Each of the eight agents needs actual implementation — LLM inference for qualification, API integrations for enrichment (ZoomInfo, Clearbit), real CPQ logic for quoting, Stripe or payment processor integration for billing. The orchestration model stays the same. The guts change entirely.

- **Failure handling.** The demo always succeeds. Production doesn't. Enrichment services timeout. Credit cards decline. Contracts get rejected. ICP scores land in the gray zone. Each failure mode needs a defined response — retry, escalate, degrade gracefully, or halt and ask a human. This is Wave 2 and 3 engineering work.

- **Latency under load.** Eight sequential agents with real API calls, real LLM inference, and real database queries won't complete in 9 seconds. Production architecture needs async execution, parallel where possible, and SLA-aware timeouts. The convergence model supports this — agents can fire concurrently and converge independently — but it hasn't been built yet. **[Speculative — architectural direction, not implemented]**

- **Integration connectors.** Salesforce, HubSpot, Stripe, Docusign, Slack — the specific systems a customer uses. Each connector is custom work. The platform needs an integration framework that makes adding connectors feasible without rebuilding the pipeline. This is the Wave 3 tooling roadmap.

---

## 6. Try It Yourself

The demo is live at [converge.zone/demo/lead-to-cash](/demo/lead-to-cash). No login. No signup. Click Start and watch it run.

Try both paths. Approve the discount and watch the deal close at $35,700. Reject it and watch standard pricing hold at $42,000. Read the convergence log — count the invariants, trace the provenance chain, see where every fact came from.

Then listen to [Business Voices Episode 2: "Show Me One Step"](/podcast/business-voices/ep02) for the narrated walkthrough — Blake and I walk through the discount approval step by step and explain why it matters for your pipeline.

If you want to talk about what a pilot looks like for your team — the metrics we'd measure, the baseline we'd capture, the governance gates that matter for your workflow — [explore our pilot programme at converge.zone/pilot](/pilot). We'll walk you through it with your numbers.

---

*Bob Calder is the Builder-Experimentalist on the Converge editorial team. He builds demos, runs benchmarks, and writes about what works (and what doesn't). Show me the trace.*

---

## Revision Log

| Rev | Date | Changes |
|-----|------|---------|
| 0 | 2026-03-13 | Initial draft per REF-53 commission brief |
| 1 | 2026-03-13 | Addressed Alice and Blake review feedback: (1) Section 3 invariant language — added parenthetical clarifying scripted checks with pointer to Section 4 (Alice MEDIUM finding), (2) Section 6 CTA updated from /contact to /pilot per Blake GTM alignment |
| 2 | 2026-03-13 | Caroline editorial review: (1) British spelling "wine-coloured", (2) CTA already fixed to /pilot in Rev 1, (3) cross-link to trust boundary article noted as pending. All edits applied. Status: Final. |

## Editorial Notes

- **Epistemic labels:** Applied throughout per editorial standards.
- **Cross-references:** Business Voices Ep. 2, DemoLeadToCash.tsx source (verified).
- **Section 4 emphasis:** Longest section as requested. Four honest gaps documented.
- **Word count:** ~1,450 words (target: 1,200-1,500).
- **Pending:** Cross-link to Alice's trust boundary article (REF-50 or successor) once published. Confirm demo URL with Jules and Dex before publish.
