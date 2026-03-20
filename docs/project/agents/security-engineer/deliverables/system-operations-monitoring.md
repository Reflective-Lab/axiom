# System Operations and Monitoring Policy

**Version:** 0.1 (Draft)
**Owner:** Ava Petrov, Security Engineer
**SOC 2 Mapping:** CC7.1, CC7.2 (partial — see also incident-response-plan.md)
**Last Updated:** 2026-03-12

---

## 1. Purpose

Define operational monitoring, vulnerability management, and change detection controls to ensure Converge platform infrastructure operates securely and anomalies are detected promptly.

## 2. Scope

- All production and staging environments
- CI/CD pipelines
- Converge engine runtime (convergence loops, agent execution)
- Database systems (SurrealDB, LanceDB)
- LLM provider integrations
- Pilot customer environments

## 3. Infrastructure Monitoring

### 3.1 Health Metrics

| Metric | Threshold | Alert Channel | Severity |
|--------|-----------|---------------|----------|
| API response latency (p99) | > 2s for 5 min | PagerDuty | High |
| API error rate (5xx) | > 1% for 5 min | PagerDuty | High |
| CPU utilization | > 85% for 15 min | Slack #ops | Medium |
| Memory utilization | > 90% for 10 min | PagerDuty | High |
| Disk utilization | > 80% | Slack #ops | Medium |
| Database connection pool | > 90% capacity | PagerDuty | High |
| Certificate expiry | < 30 days | Slack #ops | Medium |
| Certificate expiry | < 7 days | PagerDuty | Critical |

### 3.2 Converge-Specific Metrics

| Metric | Threshold | Alert Channel | Severity |
|--------|-----------|---------------|----------|
| Convergence loop iterations | > budget limit | Slack #ops | High |
| ProposedFact rejection rate | > 50% in 1 hour | Slack #security | Medium |
| Cedar policy deny rate | > 20% in 1 hour | Slack #security | Medium |
| WASM module load failure | Any | Slack #ops | High |
| LLM provider error rate | > 5% for 10 min | PagerDuty | High |
| LLM provider latency (p99) | > 30s for 5 min | Slack #ops | Medium |
| Experience store write failures | Any | Slack #ops | High |
| Agent identity verification failures | Any | Slack #security | Critical |

### 3.3 Uptime Monitoring

- External synthetic checks every 60 seconds from 3+ geographic regions
- Internal health endpoints checked every 30 seconds
- Status page updated automatically on outage detection
- Monthly uptime report generated for pilot customers

## 4. Security Monitoring

### 4.1 Log Aggregation

All security-relevant logs centralized to a tamper-resistant log store:

| Log Source | Events Captured | Retention |
|------------|----------------|-----------|
| API gateway | All requests (method, path, source IP, response code) | 1 year |
| Authentication service | Login success/failure, MFA events, session creation/expiry | 1 year |
| Cedar policy engine | All allow/deny decisions with context | 1 year |
| Converge engine | Convergence starts/completions, fact promotions, invariant results | 1 year |
| Database | Query execution (slow queries, errors), connection events | 90 days |
| CI/CD | Build triggers, deployments, artifact creation | 1 year |
| Infrastructure | SSH access, IAM changes, security group modifications | 2 years |

### 4.2 Security Alerts

| Alert | Trigger | Response |
|-------|---------|----------|
| Brute force attempt | 5+ auth failures from same IP in 10 min | Auto-block IP, notify Security |
| Privilege escalation | Any T3/T4 grant outside change management | Investigate within 1 hour |
| Anomalous API usage | 10x normal request volume from single key | Investigate, throttle if needed |
| Secret access anomaly | Access to secrets outside deployment context | Investigate within 30 min |
| ProposedFact boundary bypass attempt | Direct Fact creation without invariant check | **Immediate investigation** — critical |
| Cedar policy modification | Any policy file change | Verify against approved change record |
| New dependency introduced | `cargo audit` flags or new crate in lockfile | Security review within 24 hours |

### 4.3 Log Integrity

- Logs written to append-only storage (no modification or deletion)
- Log shipping encrypted in transit (TLS)
- Log access restricted to Security and DevOps (T3+)
- Tampering detection: hash chain verification on log batches

## 5. Vulnerability Management

### 5.1 Scanning Schedule

| Scan Type | Frequency | Tool | Owner |
|-----------|-----------|------|-------|
| Dependency audit (Rust crates) | Every CI build + daily | `cargo audit` | DevOps |
| Container image scan | Every build | Trivy or equivalent | DevOps |
| Static analysis (Rust) | Every CI build | `clippy` + custom lints | DevOps |
| Infrastructure scan | Weekly | Cloud provider tools | DevOps |
| Secret detection in code | Every commit (pre-commit) | pilot-pii-scan.sh + gitleaks | DevOps |
| Manual penetration test | Annually (or pre-launch) | External firm | Security |

### 5.2 Vulnerability Response SLAs

| Severity | Response Time | Remediation Time |
|----------|--------------|-----------------|
| Critical (CVSS 9.0+) | 4 hours | 24 hours |
| High (CVSS 7.0-8.9) | 24 hours | 7 days |
| Medium (CVSS 4.0-6.9) | 72 hours | 30 days |
| Low (CVSS < 4.0) | 7 days | 90 days |

### 5.3 Patch Management

- Security patches for critical/high vulnerabilities applied within SLA
- Non-security patches bundled into regular release cycle
- Emergency patches follow expedited change management (see change-management-policy.md)
- All patches tested in staging before production deployment

## 6. Configuration Management

### 6.1 Baseline Configuration

- All infrastructure defined as code (Terraform, Pulumi, or equivalent)
- Configuration changes tracked in version control
- No manual configuration changes in production without change record
- Configuration drift detection: daily scan comparing running state to defined state

### 6.2 Hardening Standards

- Production servers follow CIS benchmarks for the relevant OS/platform
- Database instances configured per vendor security hardening guides
- Default credentials changed on all systems before production use
- Unnecessary services and ports disabled

## 7. Capacity Management

- Capacity review monthly: current utilization vs. projected growth
- Auto-scaling configured for stateless services
- Database capacity planned 3 months ahead based on pilot customer growth
- LLM provider rate limits monitored and increased proactively

## 8. Backup Monitoring

- Backup job success/failure alerts (see business-continuity-plan.md)
- Backup integrity verification: weekly automated checksum validation
- Monthly restore test to staging environment
- Backup storage utilization monitored (alert at 80% capacity)

## 9. Operational Runbooks

Documented procedures for common operational tasks:

| Runbook | Owner | Last Reviewed |
|---------|-------|---------------|
| Production deployment | DevOps | TBD |
| Database failover | DevOps | TBD |
| Secret rotation | Security + DevOps | TBD |
| LLM provider failover | Engineering | TBD |
| Incident triage | Security | TBD |
| Backup restore | DevOps | TBD |
| Certificate renewal | DevOps | TBD |

Runbooks reviewed quarterly and after every incident that reveals a gap.

## 10. Reporting

| Report | Frequency | Audience |
|--------|-----------|----------|
| Uptime and availability | Monthly | VP Engineering, pilot customers |
| Security event summary | Weekly | Security, VP Engineering |
| Vulnerability status | Weekly | Security, VP Engineering |
| Capacity forecast | Monthly | VP Engineering, CEO |
| Compliance evidence collection | Quarterly | Security (for SOC 2 auditor) |

---

## Appendix: Control-to-TSC Mapping

| Control | TSC Criteria |
|---------|-------------|
| Infrastructure monitoring | CC7.1 |
| Security monitoring and alerting | CC7.1, CC7.2 |
| Vulnerability management | CC7.1 |
| Configuration management | CC7.1 |
| Log aggregation and integrity | CC7.1, CC7.2 |
