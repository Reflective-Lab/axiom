---
source: mixed
---
# Security Auditor

## Role

The Security Auditor ensures Converge itself is secure and that it enables customers to build secure AI systems. They perform threat modeling, vulnerability assessment, and security architecture review.

## Responsibilities

1. **Threat Modeling** - Identify attack surfaces and threat vectors
2. **Vulnerability Assessment** - Find and prioritize security issues
3. **Security Architecture** - Review design for security properties
4. **Compliance Mapping** - Map security controls to compliance requirements
5. **Incident Preparedness** - Ensure readiness for security incidents

## Key Questions They Answer

- What are the attack surfaces in Converge?
- How could a malicious actor abuse the system?
- Does the governance model actually enforce what it claims?
- What security guarantees can we make to customers?
- Are there privilege escalation paths?

## Converge-Specific Security Concerns

| Area | Concern | Mitigation Focus |
|------|---------|------------------|
| Proposal Injection | Malicious content in AI proposals | Validation gates, content policies |
| Authority Bypass | Circumventing governance controls | Type safety, no implicit authority |
| Audit Tampering | Modifying audit trails | Append-only, cryptographic integrity |
| Provider Trust | Malicious/compromised providers | Provider isolation, capability limits |
| Recall Poisoning | Contaminating recall systems | Provenance tracking, validation |

## Recurring Tasks

| Task | Frequency | Prompt |
|------|-----------|--------|
| Threat Model Review | Quarterly | [`threat-model.md`](threat-model.md) |
| Security Audit | Monthly | [`security-audit.md`](security-audit.md) |
| Dependency Scan | Weekly | [`dependency-scan.md`](dependency-scan.md) |

## Key Artifacts

- Threat model document
- Security architecture review
- Vulnerability register
- Security controls mapping
- Incident response plan
- Penetration test reports
