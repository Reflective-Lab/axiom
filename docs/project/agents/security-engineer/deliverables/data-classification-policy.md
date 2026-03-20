# Data Classification Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Annually, or upon new data types or customer segments
> SOC 2 mapping: C1.1, C1.2, CC6.1

---

## 1. Purpose

This policy defines how data processed, stored, and transmitted by the Converge platform is classified and handled. It ensures that data receives protection proportional to its sensitivity, and that personnel and agents understand their responsibilities for each classification level.

## 2. Scope

This policy applies to all data within the Converge ecosystem:
- Platform crate internals (converge-core through converge-application)
- Pilot customer data
- Internal operational data (agent configs, convergence logs, metrics)
- Third-party data received through integrations

## 3. Classification Levels

### Level 1 — Restricted

**Definition**: Data whose exposure would cause severe business harm, legal liability, or customer trust destruction.

**Examples**:
- Customer PII (names, emails, account identifiers before anonymization)
- LLM provider API keys and secrets
- Database credentials
- Agent cryptographic identity keys
- Customer business data processed during convergence runs
- Incident response communications during active incidents

**Handling requirements**:
- Encrypted at rest and in transit (TLS 1.2+)
- Access limited to named individuals/agents with documented need
- Never logged, printed, or included in error messages
- Must use approved secrets management (Google Secret Manager / HashiCorp Vault)
- Retention: minimum necessary; destroy per Data Retention Policy (REF-17)
- Anonymized before use in non-production environments

### Level 2 — Confidential

**Definition**: Internal data whose exposure would cause moderate business harm or competitive disadvantage.

**Examples**:
- Convergence run experience data (audit trails of agent decisions)
- Context data (business facts contributed by agents, scoped per workspace)
- Cedar policy definitions and authorization rules
- WASM module source code and signatures
- Configuration data (workspace settings, agent configurations)
- Pilot engagement details (customer names, contract terms)
- Architecture documents and threat models

**Handling requirements**:
- Encrypted in transit (TLS 1.2+); encryption at rest recommended
- Access controlled by role (Cedar policies, workspace membership)
- May appear in logs only in summarized/reference form (IDs, not values)
- Shared only with authorized team members and agents
- Retained per project retention schedule

### Level 3 — Internal

**Definition**: Operational data not intended for public release, but whose exposure would cause minimal harm.

**Examples**:
- Aggregated platform metrics (convergence counts, latency stats)
- Non-sensitive configuration (feature flags, UI preferences)
- Internal documentation (AGENTS.md, HEARTBEAT.md, task descriptions)
- CI/CD pipeline configurations (non-secret portions)
- Anonymized pilot data used for testing

**Handling requirements**:
- Standard access controls (team membership)
- May appear in internal logs and dashboards
- No special encryption beyond transport layer
- Shared within the organization freely

### Level 4 — Public

**Definition**: Data explicitly approved for public consumption.

**Examples**:
- Published content on converge.zone (security one-pager, pricing, docs)
- Open-source components (if any are released)
- Public API documentation
- Marketing materials

**Handling requirements**:
- Review before publication (Editor-in-Chief or VP Marketing approval)
- No restrictions on sharing
- Version controlled for accuracy

---

## 4. Converge-Specific Data Flows

### 4.1 Convergence Run Data

| Data Element | Classification | Justification |
|-------------|---------------|---------------|
| Customer business facts (input) | Restricted | Customer proprietary data |
| ProposedFact (LLM output) | Confidential | Untrusted but contains business context |
| Fact (validated output) | Confidential | Validated convergence results |
| Invariant evaluation results | Confidential | Reveals business rules |
| Convergence experience log | Confidential | Full audit trail with agent decisions |
| Convergence metrics (aggregated) | Internal | Counts and latencies only |

### 4.2 Agent Identity Data

| Data Element | Classification | Justification |
|-------------|---------------|---------------|
| Agent cryptographic keys | Restricted | Impersonation risk if compromised |
| Agent role/config definitions | Confidential | Reveals authorization structure |
| Agent activity logs (attributed) | Confidential | Contains agent identity + actions |
| Agent count/type metrics | Internal | Operational data only |

### 4.3 Pilot Customer Data

| Data Element | Classification | Justification |
|-------------|---------------|---------------|
| Customer PII | Restricted | Regulatory and contractual obligation |
| Customer business data (raw) | Restricted | Customer proprietary |
| Anonymized pilot data | Internal | PII removed per REF-18 |
| Pilot engagement metadata | Confidential | Business relationship details |

---

## 5. Handling Matrix

| Action | Restricted | Confidential | Internal | Public |
|--------|-----------|-------------|----------|--------|
| Store in source code | Never | Never | Allowed (non-sensitive) | Allowed |
| Store in environment variables | Never | Avoid; use config service | Allowed | Allowed |
| Store in secrets manager | Required | Recommended for credentials | Not needed | N/A |
| Include in logs | Never | Reference only (IDs) | Allowed | Allowed |
| Include in error messages | Never | Never | Allowed (generic) | Allowed |
| Share externally | Never without legal review | Never without NDA | With approval | Freely |
| Use in non-production | Only if anonymized | With access controls | Freely | Freely |
| Pre-commit scan required | Yes | Yes (for credentials) | No | No |

---

## 6. Labeling

All data stores, configuration files, and documentation should indicate classification level where practical:
- Database tables/collections: document classification in schema comments
- API endpoints: document input/output classification in API docs
- Configuration files: header comment with classification
- Pilot data directories: README with classification and handling rules

---

## 7. Exceptions

Exceptions to this policy require:
1. Written justification documenting the business need
2. Compensating controls described
3. Approval from Security Engineer and VP Engineering
4. Time-bounded scope (maximum 90 days, then re-evaluate)
5. Logged in the risk register as an accepted risk

---

## 8. Enforcement

- Pre-commit hooks scan for Restricted data (secrets, PII) — see `scripts/pilot-pii-scan.sh`, `scripts/pilot-data-precommit.sh`
- Code review must verify data classification handling for any code touching Restricted or Confidential data
- Violations are treated as security incidents per the Incident Response Plan

---

*Next step: Submit to VP Eng (Ren) for review. After approval, this becomes the formal data classification policy for SOC 2 C1 evidence.*
