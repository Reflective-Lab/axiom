# Converge Security Control Catalog

**Version:** 0.1 (Draft)
**Owner:** Ava Petrov, Security Engineer
**Purpose:** Master mapping of SOC 2 Trust Services Criteria → Converge controls → policies → evidence
**Last Updated:** 2026-03-12

---

## How to Read This Catalog

Each row maps a SOC 2 criterion to:
- **Control**: What we do to satisfy it
- **Policy**: Where it's documented
- **Evidence**: What an auditor would examine
- **Status**: Implemented / In Progress / Planned
- **Owner**: Who is responsible

---

## CC1: Control Environment

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 1 | CC1.1 | Board/management commitment to integrity and ethics | Information Security Policy §2 | Signed policy, org chart | Planned | CEO |
| 2 | CC1.2 | Board oversight of security function | Information Security Policy §4 | Quarterly security review minutes | Planned | CEO + Security |
| 3 | CC1.3 | Organizational structure with defined security roles | Information Security Policy §4 | AGENTS.md definitions, Paperclip agent roster | In Progress | VP Eng |
| 4 | CC1.4 | Security awareness and training | Security Awareness Program | Training completion records | Planned | Security |

## CC2: Communication and Information

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 5 | CC2.1 | Internal security communication | Information Security Policy §1 | Policy distribution records | Planned | Security |
| 6 | CC2.2 | Security training program | Security Awareness Program §3 | Training materials, completion records | Planned | Security |
| 7 | CC2.3 | Ongoing security education | Security Awareness Program §3.3 | Annual refresher records | Planned | Security |

## CC3: Risk Assessment

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 8 | CC3.1 | Risk identification process | Risk Assessment §3 | Risk register (14 risks documented) | In Progress | Security |
| 9 | CC3.2 | Risk analysis (likelihood × impact) | Risk Assessment §3 | Scored risk matrix | In Progress | Security |
| 10 | CC3.3 | Risk from external threats | Risk Assessment §3 (Infrastructure/Supply Chain) | Threat model, `cargo audit` results | In Progress | Security |
| 11 | CC3.4 | Risk from internal changes | Risk Assessment §3 (Operational) | Change management records | Planned | Security |

## CC5: Control Activities

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 12 | CC5.1 | Control activities to mitigate risks | All policies | Control catalog (this document) | In Progress | Security |
| 13 | CC5.2 | Technology controls selected and developed | Information Security Policy §5-9 | Architecture docs, code reviews | In Progress | VP Eng |
| 14 | CC5.3 | Policy-based control deployment | All policies | Deployment records, Cedar policies | Planned | Security + DevOps |

## CC6: Logical and Physical Access

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 15 | CC6.1 | Logical access security software/infrastructure | Access Control Policy, Logical/Physical Access §3 | IAM configs, MFA records | Planned | DevOps |
| 16 | CC6.2 | User registration and authorization | Access Control Policy §3-4 | Onboarding checklists, access request logs | Planned | VP Eng |
| 17 | CC6.3 | User deprovisioning | Access Control Policy §5 | Offboarding checklists, access revocation logs | Planned | VP Eng |
| 18 | CC6.4 | Authentication mechanisms | Logical/Physical Access §3.1 | SSO config, MFA enforcement logs | Planned | DevOps |
| 19 | CC6.5 | Least privilege enforcement | Logical/Physical Access §3.2 | IAM role definitions, access tier matrix | Planned | Security |
| 20 | CC6.6 | Session management | Logical/Physical Access §3.4 | Session config, timeout settings | Planned | DevOps |
| 21 | CC6.7 | Network access restrictions | Logical/Physical Access §5 | Firewall rules, network diagrams | Planned | DevOps |
| 22 | CC6.8 | Access review process | Logical/Physical Access §6 | Quarterly access review records | Planned | Security |

## CC7: System Operations

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 23 | CC7.1 | Operational monitoring and vulnerability management | System Operations & Monitoring §3-5 | Monitoring dashboards, `cargo audit` logs, scan reports | Planned | DevOps + Security |
| 24 | CC7.2 | Incident detection | System Operations & Monitoring §4, Incident Response Plan §4 | Alert configurations, incident logs | Planned | Security |
| 25 | CC7.3 | Incident response procedures | Incident Response Plan §4-5 | Incident reports, postmortems | Planned | Security |
| 26 | CC7.4 | Incident communication | Incident Response Plan §6 | Customer notifications, status page updates | Planned | Security |

## CC8: Change Management

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 27 | CC8.1 | Change management process | Change Management Policy §3-6 | PR reviews, deployment records, approval logs | In Progress | DevOps |

## CC9: Risk Mitigation

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 28 | CC9.1 | Vendor risk management | Vendor Management Policy §3-5 | Vendor assessments, SOC 2 report reviews | Planned | Security |
| 29 | CC9.2 | Vendor monitoring | Vendor Management Policy §6 | Quarterly vendor review records | Planned | Security |

## C1: Confidentiality

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 30 | C1.1 | Data classification | Data Classification Policy §3-4 | Classification labels, data inventory | In Progress | Security |
| 31 | C1.2 | Data handling procedures | Data Classification Policy §5 | Handling matrix, pre-commit hook configs | In Progress | Security + DevOps |

## PI1: Processing Integrity

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 32 | PI1.1 | Processing integrity (convergence validation) | Information Security Policy §7 | Invariant check logs, ProposedFact→Fact audit trail | Planned | Engineering |

## P1: Privacy

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 36 | P1.1 | Privacy notice provided before collection | Privacy Policy §5.1 | Published privacy notice, consent records | Planned | Security |
| 37 | P1.2 | Choice and consent mechanisms | Privacy Policy §5.2 | Consent logs, opt-in/opt-out records | Planned | Security |
| 38 | PI1.1 (Privacy) | Collection limited to stated purposes | Privacy Policy §6 | DPAs, data minimization evidence | Planned | Security + Solutions |
| 39 | PI1.2 (Privacy) | Retention and disposal per policy | Privacy Policy §7 | Disposal audit logs, PII scan results | In Progress | Security + DevOps |
| 40 | PI1.3 (Privacy) | Data subject access requests fulfilled | Privacy Policy §9 | DSR response logs | Planned | Security |
| 41 | PI1.4 (Privacy) | Disclosure to third parties controlled | Privacy Policy §8 | Sub-processor register, DPAs | Planned | Security |
| 42 | PI1.5 (Privacy) | Data quality maintained | Privacy Policy §9 (Rectification) | Rectification request logs | Planned | Security |

## A1: Availability

| # | TSC | Control | Policy | Evidence | Status | Owner |
|---|-----|---------|--------|----------|--------|-------|
| 33 | A1.1 | Recovery objectives defined | Business Continuity Plan §4 | RTO/RPO documentation | In Progress | Security |
| 34 | A1.2 | Disaster recovery procedures | Business Continuity Plan §5 | DR runbooks, test results | Planned | DevOps |
| 35 | A1.3 | Recovery testing | Business Continuity Plan §8 | Quarterly DR test reports | Planned | DevOps + Security |

---

## Summary

| Category | Controls | Implemented | In Progress | Planned |
|----------|----------|-------------|-------------|---------|
| CC1: Control Environment | 4 | 0 | 1 | 3 |
| CC2: Communication | 3 | 0 | 0 | 3 |
| CC3: Risk Assessment | 4 | 0 | 3 | 1 |
| CC5: Control Activities | 3 | 0 | 2 | 1 |
| CC6: Logical/Physical Access | 8 | 0 | 0 | 8 |
| CC7: System Operations | 4 | 0 | 0 | 4 |
| CC8: Change Management | 1 | 0 | 1 | 0 |
| CC9: Risk Mitigation | 2 | 0 | 0 | 2 |
| C1: Confidentiality | 2 | 0 | 2 | 0 |
| PI1: Processing Integrity | 1 | 0 | 0 | 1 |
| P1: Privacy | 7 | 0 | 1 | 6 |
| A1: Availability | 3 | 0 | 1 | 2 |
| **Total** | **42** | **0** | **11** | **31** |

**Current state:** All 13 policy documents drafted (including Privacy Policy for SOC 2 P criteria). 0 controls fully implemented (pre-production). 11 controls partially in progress through engineering backlog (REF-6 through REF-19) and pilot data scripts. 31 controls planned for implementation as infrastructure matures.

**Priority controls for pilot readiness:**
1. REF-8: ProposedFact→Fact boundary enforcement (PI1.1) — DONE ✓
2. REF-7: Secrets management (CC6.1, CC6.4) — HIGH
3. REF-10: Agent identity verification (CC6.5, Converge-specific) — HIGH
4. REF-15: Pilot data isolation (C1.1, C1.2) — HIGH
5. REF-6: WASM module signing (CC9.1, Converge-specific) — HIGH
