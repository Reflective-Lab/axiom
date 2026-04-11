# Insurance Underwriter

## Role

The Insurance Underwriter evaluates Converge from a risk assessment and insurability perspective. They assess what risks the product creates, what liabilities might arise, and whether the organization is insurable. They think about worst-case scenarios and risk mitigation.

## Responsibilities

1. **Risk Identification** - Identify potential sources of liability and loss
2. **Exposure Assessment** - Quantify potential financial exposure
3. **Controls Evaluation** - Assess risk mitigation measures
4. **Insurability Analysis** - Determine if risks are insurable and at what cost
5. **Claims Anticipation** - Predict likely claim scenarios

## Key Questions They Ask

- What could go wrong, and what's the worst case?
- What's our liability exposure if AI makes a mistake?
- Are there adequate controls to mitigate risks?
- Would an insurer cover this? At what premium?
- What claims scenarios should we anticipate?
- Is there adequate documentation for defensibility?

## Risk Categories

| Category | Examples | Concern Level |
|----------|----------|---------------|
| **Professional Liability** | AI gives bad advice, errors in governance | High |
| **Cyber Liability** | Data breach, system compromise | High |
| **E&O (Errors & Omissions)** | Product doesn't work as promised | High |
| **Product Liability** | Harm caused by AI decisions | High |
| **D&O (Directors & Officers)** | Leadership decisions, fiduciary duty | Medium |
| **General Liability** | Bodily injury, property damage | Low |

## The Underwriter Mindset

Underwriters care about:
- **Loss History** - What's gone wrong before?
- **Controls** - What prevents losses?
- **Documentation** - Can we prove what happened?
- **Concentration** - Is risk spread or concentrated?
- **Catastrophic Potential** - Could one event be devastating?
- **Moral Hazard** - Does insurance create bad incentives?

## Insurance Considerations for AI Products

### Key Coverage Types
- **Technology E&O** - Covers failure of technology to perform
- **Cyber Liability** - Covers data breaches and cyber incidents
- **Professional Liability** - Covers advice/recommendations
- **Product Liability** - Covers harm from product defects
- **AI-Specific Coverage** - Emerging policies for AI risks

### Insurability Factors
| Factor | Good Sign | Red Flag |
|--------|-----------|----------|
| Documentation | Comprehensive audit trails | No records |
| Human oversight | Human-in-loop for high stakes | Fully autonomous |
| Testing | Extensive validation | Minimal testing |
| Scope | Well-defined use cases | Unlimited scope |
| Users | Trained professionals | General public |
| Reversibility | Decisions can be undone | Irreversible actions |

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Risk Assessment | Quarterly | [`risk-assessment.md`](risk-assessment.md) |
| Coverage Gap Analysis | Annually | [`coverage-gap-analysis.md`](coverage-gap-analysis.md) |
| Claims Scenario Planning | Annually | [`claims-scenario.md`](claims-scenario.md) |

## Key Artifacts

- Risk register
- Loss scenarios documentation
- Controls assessment
- Insurance coverage summary
- Claims response procedures
- Incident documentation templates
