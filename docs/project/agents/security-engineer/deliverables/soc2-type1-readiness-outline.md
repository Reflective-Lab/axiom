# SOC 2 Type I Readiness Assessment — Outline

> Target: Q3 2026 readiness, Q4 2026 audit
> Owner: Ava Petrov, Security Engineer
> Status: Draft outline (2026-03-12)

---

## 1. Trust Service Criteria Mapping

SOC 2 Type I evaluates the **design** of controls at a point in time. We need controls across all applicable Trust Service Criteria (TSC).

### CC — Common Criteria (Security) — REQUIRED

| TSC | Area | Converge Status | Gap |
|-----|------|----------------|-----|
| CC1 | Control Environment | Partial — agent roles defined, no formal security policy doc | Need: Security policy, acceptable use policy |
| CC2 | Communication & Information | Partial — CLAUDE.md, AGENTS.md exist | Need: Formal security awareness for human operators |
| CC3 | Risk Assessment | Partial — threat model in HEARTBEAT.md | Need: Formal risk assessment document |
| CC4 | Monitoring Activities | Planned — converge-runtime observability | Need: Security monitoring, alerting, log review cadence |
| CC5 | Control Activities | Partial — type-system boundaries (ProposedFact/Fact). Cedar policies planned (converge-policy, Wave 2). | Need: Documented control catalog |
| CC6 | Logical & Physical Access | Partial — agent identity, context key ownership | Need: Access control policy, user access reviews |
| CC7 | System Operations | Planned — converge-runtime | Need: Incident response plan, change management |
| CC8 | Change Management | Minimal — git-based, no formal process | Need: Change management policy, approval gates |
| CC9 | Risk Mitigation | Partial — security backlog filed | Need: Formal risk treatment plan |

### A — Availability

| Area | Status | Gap |
|------|--------|-----|
| Uptime commitments | None defined | Need: SLA definition for pilot customers |
| Disaster recovery | None | Need: DR plan, backup strategy |
| Capacity planning | None | Need: Resource limits, scaling plan |

### C — Confidentiality

| Area | Status | Gap |
|------|--------|-----|
| Data classification | Defined in security one-pager | Need: Formal data classification policy |
| Encryption at rest | Planned (REF-7 secrets management) | Need: Implementation |
| Encryption in transit | TLS defined in architecture | Need: Verification and documentation |
| Data retention | Not defined (REF-17 filed) | Need: Retention policy |

### PI — Processing Integrity

| Area | Status | Gap |
|------|--------|-----|
| Input validation | REF-9 filed | Need: Implementation |
| Output accuracy | Pilot metrics framework defines quality metrics | Need: Documented validation procedures |
| Convergence integrity | LLM boundary (REF-8), invariant checking | Need: Implementation and testing |

### P — Privacy (if applicable)

| Area | Status | Gap |
|------|--------|-----|
| Privacy notice | None | Need: Privacy policy |
| PII handling | Anonymization rules defined (REF-18) | Need: Implementation, DPA template |
| Data subject rights | None | Need: Process for access/deletion requests |

---

## 2. Priority Controls to Implement (Q2 2026)

### Must-have before readiness assessment

1. **Security Policy Document** — Formal security policy covering scope, roles, responsibilities
2. **Access Control Policy** — How access is granted, reviewed, revoked for both humans and agents
3. **Incident Response Plan** — Detection, triage, containment, remediation, disclosure process
4. **Change Management Policy** — How code changes are reviewed, approved, deployed
5. **Risk Assessment Document** — Formalized from existing threat model
6. **Data Classification Policy** — Formalized from security one-pager content
7. **Vendor Management Policy** — How third-party dependencies and LLM providers are evaluated

### Must-have before audit (Q3-Q4 2026)

8. **Control Catalog** — Comprehensive list of all controls with evidence mapping
9. **Business Continuity Plan** — DR, backup, recovery procedures
10. **Employee/Agent Security Training** — Documented awareness program
11. **Encryption Implementation** — At rest and in transit, verified and documented
12. **Log Review Procedure** — Who reviews logs, how often, what triggers investigation

---

## 3. Evidence Collection Plan

For each control, we need evidence that it was designed and in place at the audit date:

| Evidence Type | Examples |
|---------------|----------|
| Policies | Written and approved policy documents |
| Configurations | Screenshots/exports of security configurations |
| Access lists | Current user/agent access lists with roles |
| Logs | Sample security logs showing monitoring is active |
| Architecture diagrams | System architecture with trust boundaries |
| Code artifacts | Type definitions (ProposedFact/Fact), Cedar policies |

---

## 4. Timeline

| Milestone | Target Date | Owner |
|-----------|------------|-------|
| This outline reviewed by VP Eng | 2026-03-18 | Ava |
| Security policy drafts complete | 2026-04-15 | Ava |
| Risk assessment document | 2026-04-30 | Ava |
| Control catalog v1 | 2026-05-15 | Ava |
| Readiness self-assessment | 2026-07-01 | Ava |
| Auditor selection | 2026-07-15 | Ava + Morgan |
| Remediation sprint | 2026-08-01 | Ava + Eng team |
| Audit window | 2026-10-01 | Auditor |

---

## 5. Open Questions

- [ ] Do we need Privacy (P) criteria? Depends on whether we process PII for customers.
- [ ] Auditor preference? (Vanta-assisted, Drata-assisted, or direct with Big 4/regional firm?)
- [ ] Budget for SOC 2 tooling (Vanta, Drata, Secureframe)?
- [ ] Who is the executive sponsor for the audit? (Likely Morgan)

---

## 6. Dependencies on Engineering

These filed issues must be resolved before SOC 2 readiness:

| Issue | Title | Severity | Why SOC 2 needs it |
|-------|-------|----------|-------------------|
| REF-6 | WASM module signing | HIGH | CC5 — control integrity |
| REF-7 | Secrets management | HIGH | C1 — confidentiality |
| REF-8 | LLM boundary validation | CRITICAL | PI1 — processing integrity |
| REF-9 | Input validation | HIGH | PI1 — processing integrity |
| REF-10 | Agent identity/spoofing | HIGH | CC6 — logical access |
| REF-11 | Security traits | MEDIUM | CC5 — control design |
| REF-15 | Pilot data isolation | HIGH | C1 — confidentiality |
| REF-16 | Webhook auth | HIGH | CC6 — logical access |
| REF-17 | Data retention policy | MEDIUM | C1 — confidentiality |
| REF-18 | PII anonymization | MEDIUM | P — privacy |

---

*Next step: Submit to VP Eng (Ren) for review and prioritization.*
