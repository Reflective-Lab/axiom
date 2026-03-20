# Business Voices — Episode 1: "What We Measure and Why"

> Track B: The operator's perspective on convergence.

**Host:** Blake Harmon, VP Marketing & Sales (GTM credibility for operator audience)
**Technical colour:** Bob Calder, Builder-Experimentalist (demo walkthroughs, trace commentary)
**Target audience:** RevOps leads, COOs, founders running 10-50 person teams
**Format:** Dual-host conversation with embedded scenario walkthrough (~10 min)
**Status:** Approved for recording (2026-03-12) — Caroline final sign-off + Blake GTM approval

---

## Cold Open (60 seconds)

> "You just lost a $40K deal. Not because your product was wrong. Not because your pricing was off. Because a lead sat in a queue for four hours while your sales rep was in a meeting, your CRM didn't trigger the follow-up, and by the time anyone noticed, the prospect had signed with someone who answered faster."
>
> "This episode isn't about AI. It's about what you measure — and what you don't."

---

## Segment 1: The Broken Handoff (2 minutes)

**Scenario setup.** **[Speculative]** Walk the listener through a constructed Lead-to-Cash pipeline (composite scenario, not based on a specific customer):

1. Marketing runs a campaign. 200 leads come in Tuesday morning.
2. CRM creates records. Routing rules assign to 3 reps.
3. Rep A is in back-to-back meetings until 2 PM. 47 leads sit untouched.
4. By 2 PM, 6 of those leads have already visited a competitor's pricing page (intent data shows this, but nobody's watching).
5. Rep A finally works the queue. Sends template emails. Two bounce — the contact info was stale. Nobody flagged it.
6. End of week: 200 leads, 12 meetings booked, 3 qualified. **94.5% waste.**

**Key line:** "The pipeline isn't broken because people are lazy. It's broken because nobody can see the whole thing at once. Each tool sees its slice. Nobody sees the handoff."

---

## Segment 2: What Converge Measures (3 minutes)

Introduce the four metric categories from the pilot framework. **[Observed]** All metric definitions and categories are drawn directly from Sam Okafor's Pilot Metrics Framework (`plans/PILOT_METRICS_FRAMEWORK.md`). Keep it concrete — no jargon.

### Cycle Time — "How long does the whole thing take?"
- End-to-end: lead arrives → deal closes (or dies). Not stage-by-stage. The whole arc. **[Observed]**
- Decision latency: data is available → someone acts on it. "Your intent data showed the prospect was shopping at 10 AM. Your rep saw it at 2 PM. That's 4 hours of decision latency." **[Speculative — illustrative example]**

> **Sequencing note:** Convergence time and iteration count — two deeper cycle-time metrics — require the converge-experience metrics exporter (Wave 3). Early pilots will measure responsiveness and efficiency first; the full cycle-time picture arrives as the platform matures. We're upfront about that.

### Responsiveness — "How fast do you react?"
- Lead response time: first meaningful action, not just an auto-reply. **[Observed]**
- "Meaningful" matters. An automated "thanks for your interest" email is not a response. A personalized follow-up referencing their use case is.

### Efficiency — "How much of the work is waste?"
- Manual steps eliminated. Count them. "Your current flow has 14 manual steps from lead to quote. After Converge, it's 4." **[Speculative — illustrative example]**
- Automation rate. Not "we automated everything" — that's a lie. "We automated 65% of the steps. The remaining 35% are judgment calls that should stay human." **[Speculative — illustrative example]**
- Rework rate. How often does a human have to fix what the system produced? **[Observed — classified under Efficiency per framework Section 2.3]**

### Quality — "Does the output actually work?"
- Convergence success rate. Did the system reach a valid answer? **[Observed]**
- Invariant violation rate. Did the system break any rules along the way? **[Observed]**
- Output accuracy. When a human reviews the result, how good is it? (Rated 1-5 via structured review.) **[Observed]**
- Determinism score. Run the same scenario twice — do you get the same outcome? (With LLM variance expected; this measures end-to-end consistency.) **[Observed]**

> **Simplification note for listeners:** The framework defines four Quality metrics. For this episode we emphasise convergence success rate and introduce the others briefly. Future episodes can deep-dive on invariant violations and determinism.

**Key line:** "These aren't vanity metrics. Cycle time is revenue velocity. Response time is win rate. Manual steps are payroll cost. Quality is trust. Every one of these connects to a number your CFO cares about."

---

## Segment 3: Why Before/After Matters (2 minutes)

Explain the baseline measurement approach — without making it sound like homework.

- "Before we turn anything on, we measure your current process for two weeks. We call it Week Zero."
- "We literally map every step: who does it, how long it takes, where the handoffs are."
- "This is not an audit. This is a mirror. Most teams have never seen their own process end-to-end."
- The surprise: **[Speculative — we have not yet run pilots]** "We expect that when you map your process end-to-end, you'll find at least one step nobody knew existed. A manual export. A Slack DM that's actually a critical handoff. A spreadsheet that three people maintain independently. That's what workflow mapping typically reveals."

**Key line:** "The baseline isn't just for us. It's for you. You can't improve what you can't see."

---

## Segment 4: What the Report Looks Like (2 minutes)

Walk through the case study template. Make it tangible.

> "Here's what you get at the end of a 4-week pilot:"

**[Speculative — fabricated projections, not measured results. No pilots have been run yet. These numbers illustrate the report format, not actual performance.]**

| Metric | Before (Week 0) | After (Week 4) | Change |
|--------|-----------------|-----------------|--------|
| End-to-end cycle time | 8 hours | 2.5 hours | -69% |
| Lead response time | 120 minutes | 8 minutes | -93% |
| Manual steps | 14 | 4 | -71% |
| Automation rate | 15% | 65% | +50 pp |
| Rework rate | 22% | 6% | -73% |

- "These numbers are projections — we haven't run a pilot yet, so treat them as the shape of the report, not as claims. Your numbers will be yours."
- "We anonymize everything. Your company name becomes 'anon-001'. Dates become 'Week 1, Day 3'. Volume metrics get rounded so nobody can fingerprint you."
- "And then — with your permission — we publish. Because the best way to prove this works is to show it working."

**Success thresholds:** "We consider a pilot successful if you see improvement on at least two of these: 30% faster cycle time, 50% faster response, 40% fewer manual steps, 20 points higher automation, or 25% more throughput. Hit three? That's a headline."

---

## Segment 5: The Honest Take (90 seconds)

What we don't measure yet. What's next. **[Observed — these gaps are acknowledged in the framework itself]**

- "We don't yet measure revenue impact directly. We measure the operational metrics that drive revenue — but the causal link from 'faster response time' to 'higher close rate' is something we'll quantify over the first few pilots." **[Inferred — the link is plausible but unproven for Converge specifically]**
- "We don't measure employee satisfaction. Automating manual steps should make people's jobs better, not just faster. We need to figure out how to measure that." **[Inferred]**
- "Convergence time — how long the engine takes to reach a fixed point — is defined in the framework but requires the converge-experience metrics exporter, which is Wave 3. Early pilots won't have this metric. We're honest about the sequencing." **[Observed — Wave 3 dependency per framework Section 2.1]**

**Key line:** "We'd rather tell you what we don't know yet than pretend we have all the answers. That's the difference between a product and a pitch."

---

## Outro (30 seconds)

> **Blake:** "Business Voices is about what actually happens when AI meets operations. Not the theory. The trace. The numbers. The report."
>
> **Bob:** "Next episode: we're going to take one of those 14 manual steps and show you exactly how Converge handles it. Screen recording. Real flow. Warts and all."
>
> **Blake:** "If any of this resonates — if you're running a pipeline and you've never seen it end-to-end — we're running pilots now. converge.zone."
>
> **Blake:** "I'm Blake Harmon." **Bob:** "I'm Bob Calder. Show me the trace."

---

## Production Notes

- **Total runtime target:** 10-12 minutes
- **Tone:** Conversational, specific, no buzzwords. Talk like you're explaining it to a smart founder over coffee.
- **Audio format:** Dual-host conversation. Blake leads the operator narrative; Bob provides technical colour and demo references. Per Caroline's editorial note, Blake's GTM credibility better serves the Business Voices audience (operators, not engineers).
- **Visual companion:** The before/after table from Segment 4 should be a standalone graphic for social media.
- **Cross-references:**
  - Sam Okafor's Pilot Metrics Framework (`plans/PILOT_METRICS_FRAMEWORK.md`) — source for all metric definitions and thresholds
  - Blake Harmon — narrative packaging, GTM alignment
  - Caroline Ashford — editorial review before recording

## Revision Log

**2026-03-12 — Rev 1 (addressing Caroline's editorial review):**
1. Added epistemic labels ([Observed], [Inferred], [Speculative]) throughout all segments
2. Fixed rework rate classification: moved from Quality to Efficiency per framework Section 2.3
3. Added full Quality metrics (invariant violation rate, output accuracy, determinism score) with simplification note
4. Added Wave 3 dependency acknowledgment for convergence time/iteration count in Segments 2 and 5
5. Rewrote "every pilot we've scoped" claim — we have not run pilots yet, reframed as expectation
6. Explicit [Speculative] tag on before/after table with note that numbers are projections
7. Changed host to Blake Harmon (primary) with Bob as technical colour — dual-host format
8. Updated outro and production notes for dual-host format

**2026-03-12 — Rev 2 (post-approval polish):**
1. Added soft CTA in outro per Blake's suggestion (approved by Caroline): "we're running pilots now. converge.zone."
2. Updated status to "Approved for recording"

## Next Steps

1. ~~Caroline reviews and approves this outline~~ Done — conditional approval + final sign-off
2. ~~Blake reviews for GTM messaging alignment and host role acceptance~~ Done — approved
3. Blake + Bob schedule and record a rough cut (voice memo quality, not production)
4. Caroline edits the script from the rough cut
5. Final recording and publication
