# Logical and Physical Access Controls

**Version:** 0.1 (Draft)
**Owner:** Ava Petrov, Security Engineer
**SOC 2 Mapping:** CC6.4, CC6.5, CC6.6, CC6.7, CC6.8
**Last Updated:** 2026-03-12

---

## 1. Purpose

Define controls governing logical access to Converge platform systems and physical access to infrastructure that processes or stores customer data.

## 2. Scope

- All production and staging environments
- CI/CD infrastructure (GitHub Actions, container registries)
- Cloud provider accounts (GCP/AWS)
- Developer workstations with access to customer data
- Secrets management systems (Google Secret Manager / Vault)
- SurrealDB and LanceDB instances
- LLM provider API integrations

## 3. Logical Access Controls

### 3.1 Authentication Requirements

| System | Auth Method | MFA Required |
|--------|-----------|-------------|
| Cloud console (GCP/AWS) | SSO + TOTP | Yes |
| GitHub (source code) | SSO + hardware key or TOTP | Yes |
| Production database (SurrealDB) | mTLS + service identity | N/A (machine) |
| Experience store (LanceDB) | Service identity + IAM | N/A (machine) |
| Secrets manager | IAM + approval workflow | Yes (console) |
| CI/CD pipeline | GitHub Actions OIDC | N/A (automated) |
| LLM provider APIs | API key via Secret Manager | N/A (machine) |
| Converge API (production) | API key + Cedar policy | Configurable |

### 3.2 Authorization Model

**Principle:** Least privilege. Every identity gets the minimum permissions needed for its function.

**Human access tiers:**

| Tier | Description | Approval | Review Frequency |
|------|-------------|----------|-----------------|
| T1: Read-only | View logs, metrics, non-sensitive config | Manager | Quarterly |
| T2: Developer | Deploy to staging, access dev databases | Manager | Quarterly |
| T3: Operator | Deploy to production, access prod databases | VP Engineering | Monthly |
| T4: Admin | IAM changes, secrets management, infra changes | VP Engineering + Security | Monthly |

**Agent/service access tiers:**

| Tier | Description | Approval |
|------|-------------|----------|
| S1: Compute-only | Execute convergence runs, no data persistence | Automated |
| S2: Read | Read from experience stores and databases | Service config review |
| S3: Write | Write to experience stores, create convergence records | Security review |
| S4: Admin | Manage agent identities, modify Cedar policies | VP Engineering + Security |

### 3.3 Converge-Specific Access Controls

**Context key ownership:**
- Each agent identity owns a namespace of context keys
- Only the owning agent (verified identity) can write to its keys (see REF-10)
- Cross-agent context reads are permitted; cross-agent writes are denied by Cedar policy

**Cedar policy evaluation:**
- All authorization decisions in the Converge engine go through converge-policy (Cedar)
- Policy changes require Security Engineer review before deployment
- Policy is versioned alongside code and subject to the same change management process

**ProposedFact → Fact boundary:**
- LLM outputs are always ProposedFact type (see REF-8)
- Promotion from ProposedFact to Fact requires invariant validation
- No identity (human or agent) can bypass this boundary without T4/S4 privileges and audit trail

**WASM module access:**
- WASM modules execute in sandboxed runtime with restricted capabilities
- Module loading requires signature verification (see REF-6)
- Modules cannot access host filesystem, network, or secrets directly

### 3.4 Session Management

- Session tokens expire after 8 hours of inactivity
- Maximum session duration: 24 hours (forced re-authentication)
- Concurrent session limit: 3 per human identity
- Session tokens stored server-side only (no client-side persistence of secrets)

### 3.5 API Key Management

- API keys rotated every 90 days (automated reminder)
- Compromised keys revoked immediately and re-issued
- API keys scoped to minimum required permissions
- All API key usage logged with source IP and action

## 4. Physical Access Controls

### 4.1 Current State (Pre-Production)

Converge currently operates on cloud infrastructure with no company-owned physical servers or data centers.

**Cloud provider reliance:**
- Physical security delegated to cloud provider (GCP/AWS) per their SOC 2 reports
- Converge validates cloud provider SOC 2 Type II reports annually (see vendor-management-policy.md)

### 4.2 Developer Workstation Controls

| Control | Requirement |
|---------|------------|
| Disk encryption | Full-disk encryption required (FileVault / BitLocker / LUKS) |
| Screen lock | Auto-lock after 5 minutes of inactivity |
| Firewall | Host firewall enabled |
| Antivirus/EDR | Endpoint protection installed and updated |
| OS patching | Security patches applied within 14 days |
| USB/removable media | Blocked for data transfer on machines with production access |

### 4.3 Future Considerations

If Converge deploys on-premises components or edge infrastructure:
- Implement physical access logging (badge readers, cameras)
- Visitor management procedures
- Environmental controls (fire suppression, HVAC, UPS)
- Hardware disposal procedures (NIST 800-88 media sanitization)

## 5. Network Access Controls

### 5.1 Network Segmentation

| Zone | Purpose | Ingress | Egress |
|------|---------|---------|--------|
| Public | API gateway, web frontend | HTTPS (443) | None |
| Application | Converge engine, agent runtime | From public zone only | To data zone, LLM providers |
| Data | SurrealDB, LanceDB, experience stores | From application zone only | None (except backups) |
| Management | CI/CD, monitoring, secrets | Authorized IPs only | Limited outbound |

### 5.2 TLS Requirements

- TLS 1.2+ for all external communication (no exceptions)
- TLS 1.3 preferred where supported
- Internal service-to-service: mTLS where available, TLS 1.2+ minimum
- Certificate rotation: automated via cert-manager or equivalent

### 5.3 Firewall Rules

- Default deny inbound on all zones
- Explicit allow rules documented and reviewed quarterly
- No broad CIDR ranges (e.g., 0.0.0.0/0) permitted in production
- VPN or bastion host required for management access

## 6. Access Reviews

| Review Type | Frequency | Reviewer | Scope |
|-------------|-----------|----------|-------|
| User access recertification | Quarterly | Manager + Security | All human accounts |
| Service account review | Quarterly | Security | All machine identities |
| Privileged access review | Monthly | VP Engineering + Security | T3/T4 and S4 accounts |
| Cedar policy review | Per change + quarterly | Security | All authorization policies |
| API key audit | Quarterly | Security | All active API keys |
| Cloud IAM review | Quarterly | Security + DevOps | All cloud permissions |

## 7. Access Revocation

### 7.1 Triggers

- Employee departure: immediate revocation of all access
- Role change: access adjusted within 24 hours
- Security incident: emergency revocation within 1 hour
- Extended leave (>30 days): access suspended, restored on return

### 7.2 Revocation Checklist

- [ ] Cloud console access removed
- [ ] GitHub organization membership removed
- [ ] VPN/bastion access revoked
- [ ] API keys revoked
- [ ] Secrets manager access removed
- [ ] Database credentials rotated (if shared)
- [ ] Session tokens invalidated
- [ ] Hardware returned and wiped (if applicable)

## 8. Logging and Monitoring

All access events are logged:

| Event | Logged Fields | Retention |
|-------|--------------|-----------|
| Authentication (success/failure) | Identity, timestamp, source IP, method | 1 year |
| Authorization decision | Identity, resource, action, Cedar policy result | 1 year |
| Privilege escalation | Identity, from-tier, to-tier, approver | 2 years |
| Access revocation | Identity, revoker, reason | 2 years |
| API key creation/rotation/revocation | Key ID, identity, action | 2 years |

Alerts trigger on:
- 5+ failed authentication attempts in 10 minutes
- T4/S4 privilege usage outside business hours
- Access from unfamiliar IP or geography
- Cedar policy deny events on production resources

---

## Appendix: Control-to-TSC Mapping

| Control | TSC Criteria |
|---------|-------------|
| Authentication requirements | CC6.4 |
| Authorization model (least privilege) | CC6.5 |
| Session management | CC6.6 |
| Network segmentation + TLS | CC6.7 |
| Access reviews + revocation | CC6.8 |
