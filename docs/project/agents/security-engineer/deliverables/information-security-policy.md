# Information Security Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Annually, or upon material architecture changes
> SOC 2 mapping: CC1.1, CC1.2, CC1.3, CC1.4, CC5.1

---

## 1. Purpose

This policy establishes the information security program for Converge, a semantic governance platform. It defines the security principles, roles, and responsibilities that protect the confidentiality, integrity, and availability of Converge systems, customer data, and intellectual property.

## 2. Scope

This policy applies to:
- All Converge platform components (converge-core, converge-traits, converge-provider, converge-runtime, and all downstream crates)
- All personnel: human operators, AI agents, and external contractors
- All environments: development, staging, production
- All customer data processed during pilot engagements
- All third-party integrations and dependencies

## 3. Security Principles

1. **Defense in depth**: No single control is sufficient. Validate at input boundaries, enforce in the engine, audit in experience stores.
2. **Least privilege**: Agents, users, and services receive only the access required for their function. Cedar policies will enforce authorization decisions when converge-policy is implemented (Wave 2).
3. **Trust boundaries are explicit**: LLM outputs are `ProposedFact`, never `Fact` (implemented — REF-8 done). Context key ownership verification will be enforced when REF-10 is implemented. WASM module signing will be required when REF-6 is implemented (Wave 4).
4. **Secrets are managed, never embedded**: All secrets will use Google Secret Manager or HashiCorp Vault (REF-7 planned). No secrets in source code, environment variables, configuration files, or logs.
5. **Security is a feature**: Security requirements are specified and tracked alongside functional requirements, not as afterthoughts.

## 4. Roles and Responsibilities

| Role | Responsibility |
|------|---------------|
| **CEO** (Morgan Vale) | Executive sponsor for security program. Approves budget and policy exceptions. |
| **VP Engineering** (Ren Akiyama) | Prioritizes security backlog in wave planning. Reviews and approves policy documents. Owns change management. |
| **Security Engineer** (Ava Petrov) | Owns security posture, threat modeling, policy authorship, release gate reviews, dependency audits, SOC 2 readiness. |
| **Founding Engineer** (Eli Marsh) | Implements security controls in converge-core and converge-traits. Owns LLM boundary validation. |
| **Senior Rust Developer** (Kira Novak) | Implements secrets management in converge-provider. |
| **DevOps Release Engineer** (Dex Tanaka) | Owns infrastructure security, deployment pipeline, secrets rotation, TLS configuration. |
| **QA Engineer** (Sam Okafor) | Validates security controls through acceptance testing. Owns data retention testing. |
| **All engineers** | Follow secure coding practices. Report potential vulnerabilities. Do not commit secrets. |

## 5. Access Control

### 5.1 Human Access
- All production systems require individual named accounts (no shared credentials).
- Multi-factor authentication (MFA) required for production access.
- Access reviews conducted quarterly.
- Access revoked within 24 hours of role change or departure.

### 5.2 Agent Access
- Each AI agent has a unique identity (`AgentId`) assigned at registration.
- Agent capabilities will be scoped via Cedar policies in converge-policy (Wave 2, planned).
- Context key ownership verification will prevent agents from spoofing other agents' keys (REF-10, planned).
- Agent budget limits will prevent unbounded execution (designed in converge-traits, enforcement planned).

### 5.3 Service Access
- API endpoints require authentication (JWT or API key).
- Service-to-service communication uses mutual TLS where feasible.
- LLM provider API keys stored in secret manager, never in code or config.

## 6. Data Classification

| Classification | Description | Handling |
|---------------|-------------|----------|
| **Public** | Marketing content, documentation, open-source code | No restrictions |
| **Internal** | Architecture plans, roadmap, agent configurations | Access limited to team members |
| **Confidential** | Customer pilot data, API keys, credentials, PII | Encrypted at rest and in transit. Access logged. Retention limits enforced. |
| **Restricted** | Signing keys, root credentials, audit secrets | Hardware-backed storage. Access requires approval. |

## 7. Converge-Specific Security Controls

### 7.1 LLM Boundary (Critical) — *Implemented (REF-8 done)*
- All LLM outputs are typed as `ProposedFact`. **[Implemented]**
- `ProposedFact` → `Fact` conversion requires passing invariant validation via `Fact::try_from(proposal)`. **[Implemented]**
- No implicit or bypass conversion paths exist in the engine proposal path. **[Implemented]**
- Property tests verify that malicious ProposedFacts are rejected (including NaN bypass). **[Implemented]**
- **Known medium risk:** `Fact::new()` remains public — agents can construct Facts directly outside the engine path. Tracked in REF-10.

### 7.2 WASM Module Integrity — *Planned (REF-6)*
- WASM modules will require verified checksums before loading. **[Planned — REF-6, Wave 4]**
- Module signing will be required before deployment. **[Planned — REF-6]**
- Modules will execute in sandboxed environments with resource limits. **[Planned — converge-runtime, Wave 4]**
- **Current state:** WASM module loading is not yet implemented. Controls will be built in from day one.

### 7.3 Convergence Integrity — *Partially Implemented*
- Invariant checking is designed into the convergence loop, but bypass resistance has not been formally verified. Risk R-04 (High, score 12) in the Risk Assessment tracks this. **[Design exists, verification pending]**
- Budget limits are defined in converge-traits. Enforcement will be implemented in converge-core. **[Designed, enforcement pending]**
- Convergence decisions will be auditable via the experience store (Wave 3). **[Planned]**

### 7.4 Context Integrity — *Planned (REF-10)*
- Context keys will have verified ownership tied to agent identity when REF-10 is implemented. **[Planned]**
- External input injection prevention depends on agent identity verification (REF-10). **[Planned]**
- **Current state:** Context keys exist as a data structure in converge-traits. Ownership enforcement is not yet implemented.

## 8. Incident Response

### 8.1 Severity Levels
| Level | Description | Response Time |
|-------|-------------|---------------|
| SEV-1 (Critical) | Active exploitation, data breach, LLM boundary bypass | Immediate (< 1 hour) |
| SEV-2 (High) | Exploitable vulnerability, secrets exposure | < 4 hours |
| SEV-3 (Medium) | Non-exploitable vulnerability, control gap | < 1 business day |
| SEV-4 (Low) | Best practice deviation, informational | Next sprint |

### 8.2 Response Process
1. **Detect**: Automated monitoring, security review, or report.
2. **Triage**: Security Engineer assesses severity and assigns owner.
3. **Contain**: Isolate affected systems. Revoke compromised credentials.
4. **Remediate**: Fix root cause. Verify fix. Deploy.
5. **Communicate**: Notify affected parties per disclosure obligations.
6. **Review**: Post-incident review within 5 business days.

## 9. Change Management

- All code changes require peer review (pull request with at least one approval).
- Security-relevant changes require Security Engineer review.
- Releases require security gate sign-off (see HEARTBEAT.md Section 6).
- No direct commits to main/production branches.
- Dependency changes reviewed for supply chain risk (`cargo audit` + manual review).

## 10. Encryption

- **In transit**: TLS 1.2+ for all external communication. TLS required for webhook integrations.
- **At rest**: Sensitive data encrypted using platform-provided encryption (cloud KMS).
- **Secrets**: Managed via Google Secret Manager or HashiCorp Vault. Rotated per vendor recommendations or 90-day maximum.

## 11. Logging and Monitoring

- Security-relevant events are logged (authentication, authorization decisions, data access, configuration changes).
- Logs do NOT contain secrets, tokens, API keys, or PII.
- Log retention: minimum 90 days for operational logs, 1 year for security/audit logs.
- Log review: weekly by Security Engineer, immediate on alert.

## 12. Vendor and Dependency Management

- New dependencies require justification and review of: maintenance status, license compatibility, known vulnerabilities, and transitive dependency surface.
- `cargo audit` runs on every release (mandatory).
- LLM provider agreements reviewed for data handling, retention, and processing terms.

## 13. Customer Data

- Customer pilot data is classified as Confidential.
- Data isolation: each customer's data is logically separated.
- Retention: raw data disposed after 90 days post-pilot (or earlier on customer request) via `scripts/pilot-data-dispose.sh`.
- Anonymized copies retained for aggregate analysis only.
- Data processing agreements (DPAs) provided on request.

## 14. Policy Exceptions

- Exceptions require written approval from VP Engineering and Security Engineer.
- Exceptions are time-limited (maximum 90 days) and logged.
- Exception renewals require re-justification.

## 15. Compliance

This policy supports compliance with:
- **SOC 2 Type I** (target Q4 2026)
- **GDPR** (where applicable to EU pilot customers)
- **CCPA** (where applicable to California pilot customers)

---

*This document is a living draft. It will be updated as the Converge platform matures and as compliance requirements evolve.*
