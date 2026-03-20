# Risk Assessment — Converge Platform

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Quarterly, or upon material architecture changes
> SOC 2 mapping: CC3.1, CC3.2, CC3.3, CC3.4, CC9.1

---

## 1. Purpose

This document identifies, assesses, and prioritizes risks to the Converge semantic governance platform. It formalizes the threat model maintained by the Security Engineer into a structured risk register suitable for SOC 2 Type I evidence and ongoing risk management.

## 2. Scope

- All Converge platform crates (converge-core through converge-application)
- Development, staging, and production environments
- Pilot customer data and integrations
- Third-party dependencies (Rust crates, LLM providers, vector databases)
- AI agent operations (both internal team agents and platform-managed agents)

## 3. Risk Assessment Methodology

### 3.1 Likelihood Rating

| Rating | Definition | Frequency |
|--------|-----------|-----------|
| 5 — Almost Certain | Expected to occur in most circumstances | Multiple times per year |
| 4 — Likely | Will probably occur | Once per year |
| 3 — Possible | Could occur | Once every 2-3 years |
| 2 — Unlikely | Not expected but possible | Once every 5 years |
| 1 — Rare | Exceptional circumstances only | Once every 10+ years |

### 3.2 Impact Rating

| Rating | Definition | Business Effect |
|--------|-----------|----------------|
| 5 — Critical | Platform compromise or data breach | Loss of all pilot customers, legal action, existential |
| 4 — Major | Significant data integrity loss | Loss of customer trust, contract termination |
| 3 — Moderate | Partial service degradation or limited data exposure | Customer escalation, remediation required |
| 2 — Minor | Localized issue, no data exposure | Internal effort to resolve, no customer impact |
| 1 — Negligible | Cosmetic or theoretical | Logged but no action required |

### 3.3 Risk Score

**Risk Score = Likelihood × Impact**

| Score Range | Risk Level | Response |
|-------------|-----------|----------|
| 20-25 | Critical | Immediate remediation required. Blocks releases. |
| 12-19 | High | Remediation within current wave. Tracked in backlog. |
| 6-11 | Medium | Remediation planned. Scheduled in future wave. |
| 1-5 | Low | Accept or monitor. Review at next assessment. |

---

## 4. Risk Register

### 4.1 Converge-Specific Risks

| ID | Risk | Threat | Likelihood | Impact | Score | Level | Mitigation | Issue |
|----|------|--------|-----------|--------|-------|-------|------------|-------|
| R-01 | **LLM output treated as trusted fact** | Attacker crafts prompt injection that produces a ProposedFact accepted without validation, corrupting convergence results | 4 | 5 | 20 | Critical | Type-system boundary: ProposedFact must never bypass validation to become Fact. Runtime check implemented in converge-core via `Fact::try_from(proposal)`. **[Mitigated — REF-8 done]** Residual risk: `Fact::new()` still public (tracked in REF-10). | REF-8 ✓ |
| R-02 | **Context key spoofing** | Malicious agent injects context keys belonging to another agent, poisoning convergence input | 3 | 5 | 15 | High | Agent identity verification before context contributions accepted. Cryptographic agent IDs bound to context keys. | REF-10 |
| R-03 | **WASM module tampering** | Compromised or tampered WASM module loaded into converge-runtime executes arbitrary logic | 2 | 5 | 10 | Medium | Module signing with checksum verification before loading. Allowlist of approved module hashes. | REF-6 |
| R-04 | **Invariant bypass** | Attacker or bug short-circuits invariant checking, allowing invalid convergence to complete | 3 | 4 | 12 | High | Invariants evaluated in a separate, non-bypassable phase. No early returns. Fuzzing of invariant engine. | — |
| R-05 | **Unbounded convergence loop** | Misconfigured or malicious convergence cycles without budget limits cause resource exhaustion | 3 | 3 | 9 | Medium | Budget enforcement (max iterations, max time, max cost) in converge-core loop controller. | — |

### 4.2 Data and Confidentiality Risks

| ID | Risk | Threat | Likelihood | Impact | Score | Level | Mitigation | Issue |
|----|------|--------|-----------|--------|-------|-------|------------|-------|
| R-06 | **Secrets in source or logs** | LLM provider API keys, database credentials, or tokens committed to source code or written to logs | 4 | 4 | 16 | High | Secrets management via Secret Manager/Vault. Pre-commit hooks. Log sanitization. | REF-7 |
| R-07 | **Pilot data cross-contamination** | One pilot customer's data accessible to another due to insufficient isolation | 3 | 5 | 15 | High | Tenant isolation at data layer. Access control policies per customer namespace. | REF-15 |
| R-08 | **PII exposure in pilot data** | Customer PII not properly anonymized in shared environments or logs | 3 | 4 | 12 | High | PII anonymization pipeline, pre-commit scan, data classification tagging. | REF-18 |

### 4.3 Infrastructure and Supply Chain Risks

| ID | Risk | Threat | Likelihood | Impact | Score | Level | Mitigation | Issue |
|----|------|--------|-----------|--------|-------|-------|------------|-------|
| R-09 | **Dependency vulnerability** | Known CVE in a Rust crate dependency exploited before patching | 3 | 4 | 12 | High | `cargo audit` in CI, automated alerts, dependency review for new crates. | — |
| R-10 | **Query injection (SurrealDB/LanceDB)** | User-controlled input reaches database queries unparameterized | 3 | 4 | 12 | High | Parameterized queries only. Input validation at all external boundaries. | REF-9 |
| R-11 | **Webhook endpoint abuse** | Unauthenticated pilot integration webhook allows data injection or exfiltration | 3 | 4 | 12 | High | HMAC signature verification, TLS, IP allowlisting for pilot webhooks. | REF-16 |

### 4.4 Operational Risks

| ID | Risk | Threat | Likelihood | Impact | Score | Level | Mitigation | Issue |
|----|------|--------|-----------|--------|-------|-------|------------|-------|
| R-12 | **Agent impersonation** | Unauthorized entity acts as a registered Converge agent | 2 | 4 | 8 | Medium | Agent identity bound to cryptographic credentials. Cedar policy evaluation on agent actions. | REF-10 |
| R-13 | **Audit log tampering** | Attacker modifies or deletes audit trail to cover tracks | 2 | 4 | 8 | Medium | Append-only audit log with hash chain. Separate log store with restricted access. | REF-28 |
| R-14 | **Script injection in ops tooling** | Unsanitized input in shell scripts enables command injection | 3 | 3 | 9 | Medium | Input validation in all scripts. No string interpolation into commands. | REF-28, REF-31 |

---

## 5. Risk Treatment Summary

| Risk Level | Count | Treatment |
|-----------|-------|-----------|
| Critical | 1 | R-01 — Immediate. Must be resolved before any production deployment. REF-8 blocks release. |
| High | 8 | R-02, R-04, R-06, R-07, R-08, R-09, R-10, R-11 — Remediate in Wave 1-2. Backlog issues filed. |
| Medium | 6 | R-03, R-05, R-12, R-13, R-14 — Schedule in Wave 2-3. Monitor. |
| Low | 0 | — |

## 6. Risk Acceptance Criteria

Risks may be accepted (rather than mitigated) when ALL of the following are true:
1. Risk score is 5 or below (Low)
2. No customer data is at risk
3. Documented approval from VP Engineering
4. Re-evaluated at next quarterly review

No Critical or High risks may be accepted without CEO approval.

## 7. Review and Update

- **Quarterly review**: Full risk register review with VP Engineering
- **Trigger-based review**: After any security incident, architecture change, new crate addition, or new pilot customer onboarding
- **Annual**: Comprehensive reassessment aligned with SOC 2 audit cycle

---

*Next step: Submit to VP Eng (Ren) for review. After approval, this becomes the formal risk register for SOC 2 CC3 evidence.*
