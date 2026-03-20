# Vendor Management Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Annually, or when adding new vendors
> SOC 2 mapping: CC9.1, CC9.2

---

## 1. Purpose

This policy governs how Converge evaluates, onboards, monitors, and offboards third-party vendors. It ensures that vendors handling Converge data or providing critical services meet security standards proportional to their risk.

## 2. Scope

All third-party relationships that involve:
- Processing, storing, or transmitting Converge or customer data
- Providing infrastructure or services Converge depends on
- Supplying software dependencies integrated into the platform

## 3. Vendor Risk Tiers

### Tier 1 — Critical

Vendors whose compromise would directly impact platform security or customer data.

**Current Tier 1 vendors**:
- **LLM providers** (e.g., Anthropic, OpenAI) — process customer context data, produce ProposedFact outputs
- **Cloud infrastructure** (e.g., GCP, AWS) — host production systems, store data
- **Secrets management** (Google Secret Manager / HashiCorp Vault) — store all credentials

**Requirements**:
- SOC 2 Type II report (or equivalent) reviewed annually
- Data Processing Agreement (DPA) in place
- Incident notification clause (72-hour maximum)
- Annual security review by Ava (Security Engineer)
- Documented in vendor register with risk rating

### Tier 2 — Important

Vendors providing services that support operations but with limited data access.

**Current Tier 2 vendors**:
- **Database providers** (SurrealDB, LanceDB) — store convergence data
- **CI/CD platform** — processes source code
- **Monitoring/observability** — receives operational telemetry

**Requirements**:
- SOC 2 Type I or equivalent security documentation
- DPA if handling customer data
- Security review at onboarding and biennially
- Documented in vendor register

### Tier 3 — Standard

Vendors providing commodity services with no data access.

**Current Tier 3 vendors**:
- **Domain registrar** (converge.zone)
- **Communication tools** (Slack, email)
- **Development tools** (GitHub, IDE tooling)

**Requirements**:
- Basic security posture review at onboarding
- MFA enabled for all accounts
- Documented in vendor register

---

## 4. Vendor Evaluation Process

### 4.1 Pre-Onboarding Assessment

Before engaging any Tier 1 or Tier 2 vendor:

1. **Security questionnaire** — vendor completes a security assessment covering:
   - Data handling practices
   - Encryption (at rest and in transit)
   - Access control and authentication
   - Incident response procedures
   - Subprocessor management
   - Compliance certifications

2. **Documentation review** — review available:
   - SOC 2 report (Type I minimum for Tier 2, Type II for Tier 1)
   - Privacy policy and DPA
   - Security whitepapers or architecture documentation
   - Penetration test summaries (if available)

3. **Risk assessment** — document:
   - What data the vendor will access
   - Data classification level per Data Classification Policy
   - Impact if vendor is compromised
   - Compensating controls

4. **Approval** — Tier 1 requires VP Engineering + CEO approval. Tier 2 requires VP Engineering approval. Tier 3 requires Security Engineer approval.

### 4.2 Converge-Specific: LLM Provider Evaluation

LLM providers are unique vendors because they process customer business context as part of convergence runs. Additional evaluation criteria:

| Criterion | Requirement |
|-----------|-------------|
| Data retention | Provider must not retain prompt/completion data beyond processing |
| Training exclusion | Customer data must be excluded from model training |
| API security | API keys rotatable, OAuth or equivalent auth available |
| Rate limiting | Provider must support rate limits to prevent abuse |
| Output filtering | Provider must document content filtering behavior |
| Subprocessors | Full list of subprocessors available |
| Data residency | Must support data residency requirements if applicable |
| Incident notification | SLA for security incident notification |

---

## 5. Rust Crate Dependency Management

Open-source Rust crate dependencies are a form of vendor relationship with unique supply chain risks.

### 5.1 New Dependency Criteria

Before adding a new crate to any Converge workspace:

| Check | Requirement |
|-------|-------------|
| `cargo audit` | No known vulnerabilities |
| License | Permissive license (MIT, Apache 2.0, BSD). No GPL in core crates without legal review. |
| Maintenance | Active maintainer (commit in last 6 months), or widely used (>1000 downloads/day) |
| Transitive dependencies | Review transitive deps for same criteria. Flag deep dependency trees. |
| Justification | Documented reason — why can't this be implemented in-house? |
| Alternatives | At least one alternative considered and documented |

### 5.2 Ongoing Monitoring

- `cargo audit` runs in CI on every build
- Weekly automated check for new advisories
- Yanked crate detection in CI
- Major version updates reviewed for breaking changes and security implications

### 5.3 Emergency Response

If a critical vulnerability is discovered in a dependency:
1. Assess exploitability in Converge context (is the vulnerable code path reachable?)
2. If exploitable: patch or replace within 24 hours
3. If not exploitable: patch within 1 week
4. Document in risk register and incident log

---

## 6. Vendor Register

Maintain a vendor register (spreadsheet or database) with:

| Field | Description |
|-------|-------------|
| Vendor name | Legal entity name |
| Service provided | What they do for Converge |
| Tier | 1, 2, or 3 |
| Data access | What data classifications they access |
| DPA in place | Yes/No/N/A |
| SOC 2 status | Type I, Type II, None, N/A |
| Last review date | Date of most recent security review |
| Next review date | Scheduled next review |
| Owner | Converge team member responsible for relationship |
| Notes | Any risk acceptances or special conditions |

*Vendor register to be created as a separate operational document after policy approval.*

---

## 7. Ongoing Monitoring

- **Tier 1**: Annual SOC 2 report review, continuous monitoring of security advisories
- **Tier 2**: Biennial review, monitor for major incidents
- **Tier 3**: Review at renewal or when role changes
- **All tiers**: Immediate review if vendor discloses a security incident

---

## 8. Offboarding

When a vendor relationship ends:
1. Revoke all access credentials and API keys
2. Confirm data deletion or return per DPA
3. Remove vendor integrations from codebase
4. Update vendor register
5. For Tier 1: obtain written confirmation of data destruction

---

*Next step: Submit to VP Eng (Ren) for review. After approval, create the operational vendor register and begin Tier 1 vendor assessments.*
