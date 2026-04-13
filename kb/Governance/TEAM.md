---
source: mixed
---
# Team Registry

> Defines Core vs Extended team membership and authority tiers for the Converge Personas governance system.

**Cross-reference:** For gate policies and participation, see [GATES.md](GATES.md).

**Escalation Guide:** For escalation packet format and examples, see [ESCALATION.md](docs/ESCALATION.md).

## Authority Tiers

**Advisory:**
- **Can:** Provide comments, suggestions, and feedback on promotion decisions
- **Cannot:** Block promotion; findings are informational only
- **Example:** Curious Searcher notes confusing onboarding UX; development team considers but proceeds

**Escalating:**
- **Can:** File escalation packets with evidence to elevate concerns; packet triggers Core review. See [Escalation Guide](docs/ESCALATION.md) for packet format and examples.
- **Cannot:** Unilaterally block promotion; requires Core disposition
- **Example:** Sustainability Lead files escalation with carbon footprint data; Core reviews and decides

**Blocking-by-Policy:**
- **Can:** Block promotion at designated gates; veto requires override process
- **Cannot:** Be overridden without two-person approval (high-risk gates) or documented rationale (low-risk)
- **Example:** Security Auditor blocks release for critical vulnerability; override requires Founder + Legal sign-off

## Team Roster

| persona_id | persona_name | team | authority_tier | tier_rationale |
|------------|--------------|------|----------------|----------------|
| system-architect | System Architect | Core | Blocking-by-Policy | Architectural violations compound; blocking prevents drift accumulation |
| qa-engineer | QA Engineer | Core | Blocking-by-Policy | Quality is binary for release; partial quality is not shippable |
| security-auditor | Security Auditor | Core | Blocking-by-Policy | Security vulnerabilities cannot be advisory; blocking prevents risk shipment |
| founder | Founder | Core | Blocking-by-Policy | Final authority on release decisions; organizational accountability |
| legal-counsel | Legal Counsel | Core | Blocking-by-Policy | Legal exposure requires blocking authority; advisory legal is unenforceable |
| ethics-safety-officer | Ethics & Safety Officer | Core | Blocking-by-Policy | Ethics violations harm users; blocking ensures responsible release |
| data-protection-officer | Data Protection Officer | Extended | Escalating | Privacy concerns need structured escalation path to legal/security |
| sustainability-lead | Sustainability Lead | Extended | Escalating | Environmental impact important but not release-blocking; escalation for tracking |
| sre-operations | SRE / Operations | Extended | Escalating | Operational concerns may require urgent escalation; not default blocking |
| insurance-underwriter | Insurance Underwriter | Extended | Escalating | Risk assessment may surface blocking concerns; escalation to legal/founder |
| regulator-lens | Regulator Lens | Extended | Escalating | Compliance gaps need escalation path; not default blocking |
| academic-researcher | Academic Researcher | Extended | Advisory | Research perspective valuable but not release-gating |
| build-vs-buy-analyst | Build vs Buy Analyst | Extended | Advisory | Strategic input for planning; not blocking individual releases |
| curious-searcher | Curious Searcher | Extended | Advisory | First impressions valuable for UX; comments improve without gating |
| developer-advocate | Developer Advocate | Extended | Advisory | Developer experience input; not release-blocking |
| end-user-advocate | End User Advocate | Extended | Advisory | User perspective for product decisions; advisory not blocking |
| future-historian | Future Historian | Extended | Advisory | Long-term perspective for strategy; not release-blocking |
| investor | Investor | Extended | Advisory | Business perspective for strategy; not release-blocking |
| journalist-investigator | Journalist Investigator | Extended | Advisory | External scrutiny perspective; advisory for PR risk awareness |
| marketing-lead | Marketing Lead | Extended | Advisory | Messaging alignment; advisory for go-to-market timing |
| product-manager | Product Manager | Extended | Advisory | Feature prioritization input; not release-blocking |
| sales-engineer | Sales Engineer | Extended | Advisory | Customer commitment validation; advisory for delivery feasibility |
| skeptical-critic | Skeptical Critic | Extended | Advisory | Critical perspective for blind spots; advisory not blocking |
| spiritual-advisor | Spiritual Advisor | Extended | Advisory | Values alignment perspective; advisory for ethical framing |

**Note:** Total: 24 personas (6 Core, 18 Extended). Core personas have blocking authority at designated gates. Extended personas contribute through advisory feedback or structured escalation.

---

*Last updated: 2026-01-26*
