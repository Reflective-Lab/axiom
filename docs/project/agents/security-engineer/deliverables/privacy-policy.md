# Privacy Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Morgan Vale, CEO / Ren Akiyama, VP Engineering]
> Effective date: [Pending approval]
> Review cadence: Annually, or upon new data types, customer segments, or regulatory changes
> SOC 2 mapping: P1.1, P1.2, PI1.1, PI1.2, PI1.3, PI1.4, PI1.5

---

## 1. Purpose

This policy defines how Converge collects, uses, retains, discloses, and disposes of personal information. It ensures compliance with SOC 2 Privacy criteria and applicable privacy regulations (GDPR, CCPA) for all data subjects whose personal information is processed by the Converge platform.

## 2. Scope

This policy applies to:
- **Pilot customer data**: Personal information processed during pilot engagements (CRM records, workflow participants, contact details)
- **Customer employee data**: Names, emails, roles of personnel using the Converge platform
- **Design partner contacts**: Business contacts acquired during sales and partnership activities
- **Internal personnel data**: Employee and contractor information
- **Agent-processed data**: Personal information that flows through convergence runs (e.g., lead names in a lead-to-close workflow)

This policy does NOT apply to:
- Anonymized or aggregated data that cannot be linked to an individual
- Publicly available business information (company names, public job titles)
- Synthetic data used in demonstrations

## 3. Definitions

| Term | Definition |
|------|-----------|
| **Personal Information (PI)** | Any information relating to an identified or identifiable natural person |
| **Data Subject** | The individual whose personal information is processed |
| **Data Controller** | The entity that determines the purposes and means of processing (Converge or customer, depending on context) |
| **Data Processor** | The entity that processes PI on behalf of the controller |
| **Processing** | Any operation performed on PI (collection, storage, use, disclosure, deletion) |
| **Sensitive PI** | Financial data, health data, government IDs, precise geolocation, biometric data |

## 4. Data Controller vs. Processor Roles

### 4.1 When Converge is Data Controller
- Internal employee/contractor data
- Business contact information for sales and marketing
- Website visitor data (analytics, cookies)
- Design partner contact details

### 4.2 When Converge is Data Processor
- **Pilot customer data**: Customer determines what PI flows through their workflows. Converge processes it on their behalf per the pilot agreement.
- **Convergence run data**: When a customer's workflow includes PI (e.g., lead names, email addresses in a CRM integration), Converge processes that data as instructed by the customer.

### 4.3 Data Processing Agreement (DPA)
Every pilot engagement that involves PI requires a DPA specifying:
- Categories of PI processed
- Purpose and duration of processing
- Customer's instructions for processing
- Security measures (reference: Information Security Policy)
- Sub-processor obligations
- Data subject rights assistance
- Breach notification obligations (see §10)

## 5. Notice and Consent

### 5.1 Privacy Notice Requirements
Before collecting PI, Converge provides notice of:
- What PI is collected and why
- How PI is used
- Who PI is shared with (including sub-processors)
- Retention period
- Data subject rights and how to exercise them

### 5.2 Consent
- **Explicit consent** required for: marketing communications, case study publication, data sharing beyond original purpose
- **Contractual basis** sufficient for: processing PI necessary to deliver the Converge service as defined in the pilot agreement
- **Legitimate interest** basis for: security monitoring, fraud detection, service improvement using aggregated metrics

### 5.3 Converge-Specific Notice
Customers are informed that:
- Their data is processed by AI agents during convergence runs
- Agent decisions are auditable via the convergence audit trail
- Human-in-the-loop gates exist for sensitive decisions (MVP-4)
- LLM providers may process data per their sub-processor agreements (see §8)

## 6. Collection and Use Limitation

### 6.1 Minimization Principle
Converge collects only the PI necessary to deliver the contracted service. Specifically:
- Convergence runs process only the data fields required by the customer's workflow configuration
- Telemetry captures run metadata (timing, success/failure, agent activity) — NOT the content of customer data
- Pilot metrics use anonymized identifiers, not raw PI (see `scripts/pilot-anonymize.sh`)

### 6.2 Purpose Limitation
PI collected for one purpose is not used for another without:
- New consent or contractual basis
- Written approval from the data subject or data controller (customer)

### 6.3 Prohibited Uses
- PI is NEVER used to train LLM models (contractual requirement with all LLM providers)
- PI is NEVER shared across pilot customers
- PI is NEVER used for profiling or automated decision-making without customer authorization
- PI from convergence runs is NEVER retained after disposal (see §7)

## 7. Retention and Disposal

### 7.1 Retention Periods

| Data Category | Retention Period | Basis |
|--------------|-----------------|-------|
| Active pilot data | Duration of pilot + 90 days | Contractual |
| Post-pilot customer data | 90 days after pilot end | Analysis window (Pilot Metrics Framework §11.5) |
| Anonymized pilot metrics | Indefinite | Aggregated, non-PI |
| Business contacts | Until relationship ends + 1 year | Legitimate interest |
| Employee data | Employment + 3 years | Legal requirement |
| Convergence audit logs | 1 year | Compliance |
| Website analytics | 13 months | GDPR Cookie Directive |

### 7.2 Disposal Procedures
- **Pilot data**: Disposed via `scripts/pilot-data-dispose.sh` with full audit trail
- **Anonymization**: Pre-disposal anonymization via `scripts/pilot-anonymize.sh`
- **PII scanning**: Pre-commit hook (`scripts/pilot-data-precommit.sh`) prevents PI from entering source control
- **Verification**: Post-disposal PII scan (`scripts/pilot-pii-scan.sh`) confirms no residual PI

### 7.3 Customer Data Portability
Upon request, customers receive their data in a structured, machine-readable format (JSON) within 30 days.

## 8. Disclosure and Sub-Processors

### 8.1 Sub-Processor Management
Converge uses third-party sub-processors that may process PI:

| Sub-Processor Category | Purpose | PI Exposure | Controls |
|----------------------|---------|-------------|----------|
| LLM Providers (OpenAI, Anthropic, etc.) | Agent reasoning during convergence | Workflow data in prompts | DPA, no-training clause, data retention limits |
| Cloud Infrastructure (GCP/AWS) | Platform hosting | All platform data | SOC 2 certified, encryption at rest |
| Database (SurrealDB, LanceDB) | Data storage | Convergence state, experience store | Encrypted, access-controlled |
| Secrets Manager (Google Secret Manager) | Credential storage | API keys only (no PI) | IAM-controlled |

### 8.2 Sub-Processor Obligations
All sub-processors must:
- Have a signed DPA
- Meet security standards per Vendor Management Policy
- Commit to no-training clauses for customer data
- Notify Converge of sub-processor changes
- Support data deletion requests

### 8.3 Customer Notification
Customers are notified of sub-processor changes 30 days in advance. Customers may object if the change materially affects their data processing.

## 9. Data Subject Rights

Converge supports the following rights for data subjects:

| Right | Description | Response Time |
|-------|-----------|---------------|
| **Access** | Request a copy of their PI | 30 days |
| **Rectification** | Correct inaccurate PI | 30 days |
| **Erasure** | Request deletion of PI | 30 days |
| **Restriction** | Limit processing of PI | 30 days |
| **Portability** | Receive PI in machine-readable format | 30 days |
| **Objection** | Object to processing based on legitimate interest | 30 days |
| **Withdraw Consent** | Revoke previously given consent | Immediate |

### 9.1 Process for Exercising Rights
1. Data subject submits request via email to privacy@converge.zone
2. Identity verified within 3 business days
3. Request assessed and fulfilled within 30 calendar days
4. If the request involves customer-controlled data (processor role), the customer is notified and handles the response with our assistance

### 9.2 Converge-Specific Considerations
- **Convergence audit logs** containing PI are subject to erasure requests, but deletion may be deferred if required for active compliance obligations
- **LLM provider data**: Erasure requests are forwarded to relevant sub-processors per their DPA terms
- **Anonymized data**: Cannot be erased as it is no longer PI

## 10. Breach Notification

### 10.1 Internal Notification
- Security Engineer (Ava Petrov) notified within 1 hour of discovery
- VP Engineering notified within 4 hours
- CEO notified within 8 hours

### 10.2 External Notification
| Obligation | Timeline | Basis |
|-----------|----------|-------|
| Customer (data controller) | 24 hours | DPA obligation |
| Supervisory authority (GDPR) | 72 hours | GDPR Art. 33 |
| Affected individuals (GDPR) | Without undue delay | GDPR Art. 34 (high risk only) |
| California AG (CCPA) | As required | CCPA §1798.82 |

### 10.3 Breach Response
Per Incident Response Plan (`deliverables/incident-response-plan.md`), specifically Scenario 5 (Privacy Breach).

## 11. Cross-Border Data Transfers

### 11.1 Current State
Converge processes data within the United States. LLM providers may process data in other jurisdictions per their infrastructure.

### 11.2 Safeguards
- Standard Contractual Clauses (SCCs) required for transfers outside the US/EU
- LLM provider DPAs must specify data processing locations
- Customer consent required for cross-border transfers not covered by SCCs

### 11.3 Future State
As Converge scales internationally, this section will be updated with:
- Regional data processing options
- Data residency controls per customer requirement
- Country-specific compliance addenda

## 12. Privacy by Design

### 12.1 Principles Applied
- **Data minimization**: Convergence runs process minimum necessary data
- **Purpose limitation**: Strict separation of customer data across pilot engagements
- **Pseudonymization**: Pilot metrics use anonymized identifiers
- **Transparency**: Audit trails make all agent decisions reviewable
- **Security**: Encryption at rest and in transit for all PI

### 12.2 Privacy Impact Assessment (PIA)
Required before:
- Adding a new data collection mechanism
- Integrating a new sub-processor
- Processing a new category of sensitive PI
- Changing the purpose of existing PI processing

PIA template available from Security Engineer.

## 13. Training and Awareness

Per Security Awareness & Training Program (`deliverables/security-awareness-program.md`):
- All personnel complete privacy fundamentals during onboarding
- Annual privacy refresher training
- Role-specific training for personnel handling PI (Solutions Engineer, QA Engineer)
- Agent-specific guidelines for handling PI in convergence runs

## 14. Policy Governance

| Aspect | Detail |
|--------|--------|
| Owner | Ava Petrov, Security Engineer |
| Approver | Morgan Vale, CEO |
| Review frequency | Annually or upon material change |
| Exception process | Written request to CEO with risk assessment |
| Version history | Maintained in this document header |

---

## Appendix A: SOC 2 Privacy Criteria Mapping

| Criterion | Description | Section |
|-----------|-----------|---------|
| P1.1 | Privacy notice | §5.1 |
| P1.2 | Choice and consent | §5.2 |
| PI1.1 | Collection | §6 |
| PI1.2 | Use, retention, and disposal | §6, §7 |
| PI1.3 | Access | §9 |
| PI1.4 | Disclosure to third parties | §8 |
| PI1.5 | Quality | §9 (Rectification) |

## Appendix B: Converge Data Flow — Privacy View

```
Customer CRM/Systems
        │
        ▼
  [Integration Layer]     ← TLS, webhook auth (REF-16)
        │
        ▼
  [Convergence Engine]    ← Minimum necessary data, audit trail
        │
        ├──► [LLM Provider]     ← Sub-processor DPA, no-training clause
        │         │
        │         ▼
        │    [ProposedFact]      ← LLM output, NOT trusted
        │         │
        │         ▼
        │    [Validation]        ← Invariant checking (REF-8)
        │         │
        │         ▼
        ├──► [Fact / State]      ← Validated, stored encrypted
        │
        ▼
  [Experience Store]      ← Encrypted at rest, access-controlled
        │
        ▼
  [Pilot Metrics]         ← Anonymized (pilot-anonymize.sh)
        │
        ▼
  [Disposal]              ← pilot-data-dispose.sh, verified by PII scan
```

## Appendix C: Regulatory Quick Reference

| Regulation | Applicability | Key Requirements |
|-----------|--------------|-----------------|
| GDPR | EU data subjects | Consent, DPA, 72h breach notification, data subject rights, SCCs |
| CCPA/CPRA | California residents | Right to know, delete, opt-out of sale, no discrimination |
| HIPAA | If healthcare customer | BAA required, PHI safeguards, breach notification |
| SOX | If financial services customer | Data integrity, audit trails, access controls |

**Note:** Converge does not currently target healthcare customers (HIPAA) or publicly traded financial institutions (SOX). These entries are included for future reference.
