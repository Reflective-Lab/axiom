# Business Continuity & Disaster Recovery Plan

**Version:** 0.1 (Draft)
**Owner:** Ava Petrov, Security Engineer
**SOC 2 Mapping:** A1.1, A1.2, A1.3
**Last Updated:** 2026-03-12

---

## 1. Purpose

Define procedures to maintain availability of Converge platform services and pilot customer environments during disruptions, and to recover within defined objectives.

## 2. Scope

- Converge SaaS platform (all crates and services)
- Pilot customer environments (`pilot-data/` stores, integration webhooks)
- Development and CI/CD infrastructure
- LLM provider dependencies
- Internal collaboration tools

## 3. Roles and Responsibilities

| Role | Responsibility |
|------|---------------|
| CEO (Morgan Vale) | Declares disaster, approves external communications |
| VP Engineering (Ren Akiyama) | Owns recovery execution, makes triage decisions |
| DevOps (Dex Tanaka) | Executes infrastructure recovery, manages backups |
| Security Engineer (Ava Petrov) | Validates recovery integrity, manages incident comms |
| Founding Engineer (Eli Marsh) | Core platform recovery, data integrity verification |

## 4. Recovery Objectives

| System | RTO (Recovery Time) | RPO (Recovery Point) | Priority |
|--------|---------------------|----------------------|----------|
| Converge API (production) | 4 hours | 1 hour | Critical |
| Pilot customer data stores | 4 hours | 1 hour | Critical |
| LLM provider connectivity | 2 hours (failover) | N/A | Critical |
| CI/CD pipeline | 8 hours | 24 hours | High |
| Monitoring & alerting | 4 hours | N/A | High |
| Development environments | 24 hours | 24 hours | Medium |
| Documentation & wiki | 48 hours | 7 days | Low |

## 5. Disaster Scenarios

### 5.1 Cloud Infrastructure Failure

**Trigger:** Primary cloud region outage or account compromise.

**Response:**
1. Activate secondary region (if multi-region) or failover provider
2. Restore from latest backup to alternate infrastructure
3. Verify data integrity with checksums before bringing services online
4. Redirect DNS / load balancer to recovered infrastructure
5. Notify affected pilot customers within 1 hour of confirmed impact

**Prevention:**
- Infrastructure-as-code for reproducible deployments
- Automated backups with cross-region replication
- Regular restore testing (quarterly)

### 5.2 LLM Provider Outage

**Trigger:** Primary LLM provider (e.g., Anthropic, OpenAI) unavailable.

**Response:**
1. converge-provider automatically routes to fallback provider (if configured)
2. Alert engineering team if no fallback configured
3. Degrade gracefully: queue convergence requests, return cached results where safe
4. Communicate expected resolution to pilot customers

**Prevention:**
- converge-provider multi-provider architecture (Wave 2)
- Circuit breaker pattern with configurable timeout
- Cached responses for non-critical convergence queries

### 5.3 Data Breach / Compromise

**Trigger:** Unauthorized access to customer data, credentials, or source code.

**Response:**
1. Follow Incident Response Plan (see `incident-response-plan.md`)
2. Isolate affected systems immediately
3. Rotate all secrets and credentials
4. Assess blast radius — which customers, which data
5. Notify affected parties per legal requirements (72 hours max)
6. Preserve forensic evidence before remediation

**Prevention:**
- Secrets management (REF-7)
- Access control policy enforcement
- Audit logging on all data access
- Pre-commit PII scanning (pilot-data-precommit.sh)

### 5.4 Supply Chain Compromise

**Trigger:** Compromised dependency (Rust crate, npm package, container image).

**Response:**
1. Pin to last known good version immediately
2. Audit all builds since compromised version was introduced
3. Rebuild and redeploy from clean dependencies
4. Notify pilot customers if compromised code reached production

**Prevention:**
- `cargo audit` in CI pipeline
- Dependency review for all new crates (vendor-management-policy.md)
- Lockfile integrity verification
- WASM module signing (REF-6)

### 5.5 Pilot Data Loss

**Trigger:** Accidental or malicious deletion of pilot customer data.

**Response:**
1. Restore from backup within RPO window
2. Verify data integrity against last known checksums
3. Run pilot-pii-scan.sh to confirm no data leakage during recovery
4. Communicate timeline and impact to affected customer

**Prevention:**
- Automated daily backups of `pilot-data/` directories
- Write-once audit logs (cannot be retroactively modified)
- Deletion requires `PILOT_DISPOSE_CONFIRM=yes` gate
- Per-customer data isolation (REF-15)

## 6. Backup Strategy

### 6.1 What is Backed Up

| Data | Frequency | Retention | Encryption |
|------|-----------|-----------|------------|
| Pilot customer data | Daily | 90 days | AES-256 at rest |
| Converge database (SurrealDB) | Hourly | 30 days | AES-256 at rest |
| Experience store (LanceDB) | Daily | 30 days | AES-256 at rest |
| Source code (Git) | Continuous | Indefinite | TLS in transit |
| Secrets / credentials | On change | 30 days | Vault-managed |
| CI/CD configuration | Daily | 30 days | AES-256 at rest |

### 6.2 Backup Verification

- **Weekly:** Automated backup integrity check (checksum validation)
- **Monthly:** Test restore of one backup to staging environment
- **Quarterly:** Full disaster recovery drill (simulate scenario from Section 5)

## 7. Communication Plan

### Internal

| Severity | Channel | Notification Time |
|----------|---------|-------------------|
| Critical | Slack #incidents + phone tree | Immediate |
| High | Slack #incidents | Within 30 minutes |
| Medium | Slack #engineering | Within 2 hours |
| Low | Email | Within 24 hours |

### External (Pilot Customers)

| Severity | Channel | Notification Time |
|----------|---------|-------------------|
| Data breach | Email + phone call | Within 24 hours |
| Service outage > 1 hour | Email | Within 2 hours |
| Degraded performance | Status page | Within 4 hours |
| Planned maintenance | Email (advance) | 72 hours before |

## 8. Testing Schedule

| Test Type | Frequency | Owner |
|-----------|-----------|-------|
| Backup restore verification | Monthly | DevOps |
| Tabletop exercise | Quarterly | Security + VP Eng |
| Full DR drill | Biannually | All engineering |
| Communication test | Quarterly | Security |

## 9. Plan Maintenance

- Review and update this plan quarterly or after any incident
- All team members must acknowledge reading this plan annually
- Changes require VP Engineering approval
- Post-incident reviews must include BCP adequacy assessment

---

## Appendix: Recovery Runbooks

*(To be developed as infrastructure matures)*

- [ ] Cloud failover runbook
- [ ] LLM provider failover runbook
- [ ] Database restore runbook
- [ ] Secret rotation runbook
- [ ] Pilot data restore runbook
