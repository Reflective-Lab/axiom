---
escalation_id: ESC-2026-003
escalated_by: insurance-underwriter
escalated_at: 2026-01-26T09:00:00Z
gate_id: customer-commitment
eval_id: insurance-underwriter-eval
severity: P1

concern:
  risk_prevented: Uninsurable liability exposure from aggressive SLA commitment creating financial risk if uptime targets not met
  evidence:
    - "Customer requesting 99.99% uptime SLA in contract proposal"
    - "Current infrastructure is single-region AWS us-east-1"
    - "Single-region architecture creates single point of failure for regional outages"
  stop_rule: SLA terms reviewed by legal for liability caps AND multi-region architecture confirmed OR SLA commitment reduced to 99.9%
  confidence: medium

context:
  investigation_summary: |
    Customer proposal includes aggressive SLA terms (99.99% uptime). Concerned about liability
    exposure if we cannot meet the commitment due to infrastructure limitations. Current
    single-region deployment means regional AWS outage would violate SLA. Insurance policy
    may not cover SLA penalty claims if infrastructure known to be insufficient.
  related_escalations: []
  recommended_disposition: conditional
---

# Need More Info Escalation Example: Insurance Risk Concern

## Initial Disposition Feedback

---
disposition_id: DISP-2026-003a
escalation_id: ESC-2026-003
reviewed_by: [legal-counsel]
reviewed_at: 2026-01-26T11:00:00Z

outcome:
  disposition: need_more_info
  rationale: |
    Liability concern is valid direction but evidence is incomplete for decision. Need more
    specific information to assess risk:

    1. **Actual SLA language**: You mention "99.99% uptime" but need actual contract language.
       What are the penalty terms? Per-incident cap? Monthly cap? Grace periods? These details
       determine liability exposure magnitude.

    2. **Current SLA baseline**: What SLA do we currently offer? How does 99.99% request compare
       to our standard terms? This helps assess whether request is aggressive or standard.

    3. **SRE feasibility assessment**: Is multi-region architecture on roadmap? What's timeline?
       What would it take to achieve 99.99% with current single-region setup? Need SRE input
       on technical feasibility before assessing liability risk.

    4. **Penalty liability quantification**: What would SLA violation cost us? If contract is
       $500K/year with 10% penalty cap, liability is $50K. If uncapped, exposure is unbounded.
       Quantify financial risk.

    Without this evidence, cannot determine whether this is insurable risk or unacceptable
    exposure. Strengthen evidence and resubmit.
  precedent_references: []

learning:
  what_was_strong: |
    Proactive identification of liability risk before contract signing (correct gate:
    customer-commitment). Reasonable stop rule involving both legal review and technical
    validation. Correct identification that single-region creates SLA risk. Good awareness
    that insurance policy may not cover known-insufficient infrastructure.
  what_to_improve: |
    Include actual contract language in evidence. Quote specific SLA clauses that create
    concern (uptime target, penalty terms, measurement methodology). Get SRE input before
    escalating - is multi-region even on roadmap? Roadmap check prevents escalating
    architecturally-infeasible commitments. Quantify financial exposure - what would SLA
    violation cost us? Numbers help legal and founder assess risk tolerance.
  coaching_notes: |
    Your instinct is correct - SLA commitments create liability risk. To strengthen:

    1. **Attach contract language**: Copy actual SLA section from customer proposal. Include
       penalty terms, measurement methodology, grace periods.

    2. **Quantify exposure**: Calculate worst-case liability. If $500K contract with 10%
       penalty cap = $50K exposure. If uncapped = unbounded risk.

    3. **Get SRE feasibility input**: Ask SRE: "Is 99.99% achievable with single-region? Is
       multi-region on roadmap?" This prevents escalating impossible commitments.

    4. **Compare to standard terms**: What SLA do we normally offer? Is 99.99% request unusual
       or standard for enterprise contracts?

    Good instinct, incomplete evidence. Strengthen and resubmit.

action:
  next_steps: |
    Insurance Underwriter to strengthen evidence:
    1. Attach customer SLA proposal language (penalty terms, caps, measurement methodology)
    2. Document gap between current SLA offering and 99.99% request
    3. Include SRE feasibility note on multi-region architecture timeline
    4. Quantify penalty liability (worst-case financial exposure)

    Resubmit as ESC-2026-003-v2 with additional evidence.
  compensating_controls: []
  follow_up_owner: insurance-underwriter
---

## Iteration: Resubmitted Escalation With Strengthened Evidence

After receiving need_more_info disposition, Insurance Underwriter gathered additional evidence and resubmitted:

---
escalation_id: ESC-2026-003-v2
escalated_by: insurance-underwriter
escalated_at: 2026-01-26T13:30:00Z
gate_id: customer-commitment
eval_id: insurance-underwriter-eval
severity: P1

concern:
  risk_prevented: Uninsurable liability exposure from aggressive SLA commitment creating financial risk if uptime targets not met
  evidence:
    - "Customer SLA proposal (attached): 99.99% monthly uptime (43 minutes downtime/month allowed)"
    - "Penalty terms: 10% monthly fee credit per 0.1% below target (e.g., 99.8% uptime = 20% credit)"
    - "No annual cap on penalties (uncapped liability if prolonged degradation)"
    - "Current standard SLA: 99.5% uptime (3.6 hours downtime/month allowed)"
    - "SRE assessment: Single-region cannot reliably achieve 99.99%; multi-region architecture on Q3 2026 roadmap"
    - "Contract value: $800K/year (worst-case liability: $80K/year at 99.0% uptime, uncapped if worse)"
  stop_rule: Legal reviews penalty terms for liability caps AND founder approves SLA commitment understanding infrastructure limitation OR SLA negotiated down to 99.5% standard terms
  confidence: high

context:
  investigation_summary: |
    Obtained customer SLA proposal language. Penalty structure is 10% credit per 0.1% below
    99.99% target with no annual cap (uncapped liability). Compared to our standard SLA
    offering (99.5% uptime). Consulted SRE - single-region architecture cannot reliably
    achieve 99.99% (regional AWS outages would violate SLA). Multi-region architecture on
    roadmap but not until Q3 2026. Calculated worst-case liability: contract is $800K/year,
    so 99.0% uptime (0.99% below target) would be $80K/year in credits. Prolonged degradation
    could exceed contract value. Insurance policy covers negligence but not "known insufficient
    infrastructure", so this exposure may be uninsured.
  related_escalations: []
  recommended_disposition: conditional
---

## Final Disposition Feedback

---
disposition_id: DISP-2026-003b
escalation_id: ESC-2026-003-v2
reviewed_by: [legal-counsel, founder]
reviewed_at: 2026-01-26T15:45:00Z

outcome:
  disposition: approved
  rationale: |
    Strong evidence now demonstrates significant liability risk. Uncapped penalty structure
    with 10% credit per 0.1% below target creates unbounded exposure. SRE assessment confirms
    single-region cannot reliably achieve 99.99% (Q3 2026 multi-region roadmap too far out).
    $800K contract with uncapped penalties could exceed contract value in prolonged degradation
    scenario. Insurance policy "known insufficient infrastructure" exclusion means exposure
    may be uninsured. This is unacceptable risk for customer-commitment gate.

    Approve escalation. Sales must negotiate SLA down to 99.5% standard terms or defer contract
    until Q3 2026 multi-region deployment.
  precedent_references:
    - "DISP-2025-156: Approved similar SLA escalation, negotiated down from 99.95% to 99.5%"

learning:
  what_was_strong: |
    Excellent evidence strengthening after need_more_info disposition. Attached actual contract
    language (penalty terms, caps, measurement methodology). Quantified financial exposure
    ($800K contract, $80K/year worst-case, uncapped if prolonged). Got SRE feasibility input
    (single-region insufficient, multi-region Q3 2026). Compared to standard SLA offering
    (99.5% vs 99.99% request). High confidence now justified by thorough investigation.

    This iteration shows learning loop in action: initial escalation flagged concern, need_more_info
    provided specific strengthening guidance, resubmitted escalation addressed all gaps. Excellent
    responsiveness to feedback.
  what_to_improve: |
    For future SLA escalations, include contract language and SRE feasibility in initial
    submission. This would prevent need_more_info iteration and accelerate review. Template:
    (1) attach contract SLA section, (2) quantify financial exposure, (3) include SRE feasibility
    note, (4) compare to standard terms.
  coaching_notes: |
    This demonstrates ideal escalation iteration pattern. Initial submission flagged valid concern
    but lacked specifics. need_more_info disposition provided clear strengthening asks. Resubmitted
    escalation addressed every gap: contract language, financial quantification, SRE feasibility,
    standard terms comparison.

    Use this template for future SLA/contract escalations: attach contract language, quantify
    exposure, get technical feasibility input, compare to standard terms. Including these in
    initial submission saves iteration time.

action:
  next_steps: |
    1. Sales Engineer will negotiate SLA down to 99.5% standard terms with customer
    2. If customer requires 99.99%, defer contract until Q3 2026 multi-region deployment
    3. Legal Counsel will review final contract language before signing
    4. If deferral option chosen, add to Q3 2026 customer pipeline for re-engagement post-multi-region
  compensating_controls: []
  follow_up_owner: sales-engineer
---

## Why This Escalation Required Iteration

This example shows the need_more_info disposition in action - not rejection, but request for strengthening.

### Initial Submission Gaps

The initial escalation flagged valid concern but lacked specifics:
- Mentioned "99.99% uptime SLA" but no contract language (penalty terms, caps, measurement)
- Noted "single-region architecture" but no SRE feasibility assessment
- Identified liability risk but no financial quantification
- No comparison to standard SLA offering

Core team couldn't assess risk magnitude without these details.

### need_more_info Disposition Provides Clear Guidance

Core team didn't reject the concern - they provided specific asks:

1. **Actual SLA language**: Attach contract section with penalty terms, caps, measurement methodology
2. **Current SLA baseline**: Compare 99.99% request to standard offering
3. **SRE feasibility**: Get technical input on whether 99.99% achievable with current architecture
4. **Penalty quantification**: Calculate worst-case financial exposure

This guidance clarified exactly what evidence would strengthen the escalation.

### Resubmitted Escalation Addressed All Gaps

Insurance Underwriter responded to feedback thoroughly:

1. **Contract language attached**: "10% credit per 0.1% below target, no annual cap"
2. **Baseline comparison**: Standard offering is 99.5% (3.6h/month) vs 99.99% request (43min/month)
3. **SRE feasibility**: "Single-region cannot reliably achieve 99.99%, multi-region Q3 2026"
4. **Financial quantification**: "$800K contract, $80K/year worst-case, uncapped if prolonged"

Every gap filled. Confidence increased from medium to high because investigation was now thorough.

### Final Disposition: Approved

With strengthened evidence, Core team could assess risk magnitude and approve escalation. Uncapped penalty structure with $80K+/year exposure on $800K contract is significant risk. SRE assessment confirms commitment is not technically feasible until Q3 2026. Approval leads to clear action: negotiate down or defer.

## Learning From Iteration Pattern

### need_more_info Is Not Rejection

Key lesson: need_more_info signals "concern may be valid, need more evidence to decide". It's an invitation to strengthen and resubmit, not dismissal.

**Indicators need_more_info is appropriate**:
- Core team sees valid concern but lacks details to assess magnitude
- Evidence is directional but not specific (e.g., "aggressive SLA" without contract language)
- Investigation shows awareness but not depth (e.g., noted single-region but no SRE feasibility)

### Strengthening Guidance Is Specific

Notice Core team didn't say "provide more information" generically. They provided specific asks:

- "Attach actual contract language with penalty terms"
- "Get SRE assessment of multi-region feasibility"
- "Quantify financial exposure (worst-case liability)"
- "Compare to standard SLA offering"

This specificity helps Extended know exactly what to add.

### Iteration Shows Learning Loop

The resubmitted escalation addressed every gap from need_more_info disposition. This demonstrates:

1. **Extended team responsiveness**: Gathered requested evidence promptly
2. **Core guidance effectiveness**: Specific asks led to specific evidence additions
3. **Learning loop closure**: Iteration improved escalation quality from medium to high confidence

### Positive Feedback Even After Iteration

Core team acknowledged iteration quality:

**what_was_strong**: "Excellent evidence strengthening... addressed all gaps... shows learning loop in action"

**what_to_improve**: "For future, include contract language and SRE feasibility in initial submission"

**coaching_notes**: "Use this template for future SLA/contract escalations"

Even though initial submission required iteration, positive reinforcement encourages continued engagement.

## Template for Future SLA/Contract Escalations

To avoid need_more_info iteration, include in initial submission:

1. **Attach contract language**: Copy actual SLA section (penalty terms, caps, measurement methodology)
2. **Quantify financial exposure**: Calculate worst-case liability (contract value × penalty percentage)
3. **Get technical feasibility input**: Ask SRE or relevant technical owner whether commitment is achievable
4. **Compare to standard terms**: Show gap between request and current offering
5. **Identify infrastructure limitations**: Note single-region, single-datacenter, or other constraints

This template accelerates review by providing complete evidence upfront.

## When to Expect need_more_info

Expect need_more_info disposition when:

- Evidence is directional but not specific (e.g., "customer wants aggressive SLA" without contract language)
- Investigation shows awareness but not depth (e.g., noted concern but didn't consult domain expert)
- Financial/technical impact not quantified (e.g., "creates liability" without dollar amount)
- Comparison to baseline missing (e.g., "SLA too high" without showing current offering)

If you receive need_more_info, treat it as coaching - Core team is teaching you what evidence makes escalations strong.

---

*This example demonstrates the need_more_info disposition as a learning tool, showing how Extended team can strengthen escalations through iteration based on specific Core team guidance.*
