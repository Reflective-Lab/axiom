# Access Control Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Quarterly
> SOC 2 mapping: CC6.1, CC6.2, CC6.3

---

## 1. Purpose

This policy governs how access to Converge systems, data, and infrastructure is granted, reviewed, and revoked — for both human operators and AI agents. It ensures that access is proportional, auditable, and aligned with the principle of least privilege.

## 2. Scope

This policy applies to:
- Human operators (engineers, support, executives)
- AI agents (Converge platform agents, Paperclip-managed agents)
- External contractors and auditors
- All Converge infrastructure (cloud services, repositories, CI/CD, databases)
- Customer pilot environments

## 3. Access Principles

1. **Least privilege**: Every identity — human or agent — receives the minimum access required for its function.
2. **Explicit grant**: No default access. All access is explicitly granted and documented.
3. **Time-bounded**: Elevated access expires automatically. No permanent admin grants without documented justification.
4. **Auditable**: Every access grant, modification, and revocation is logged.
5. **Separation of duties**: No single identity can both approve and execute privileged operations.

## 4. Identity Types

### 4.1 Human Identities

| Role | Typical Access | Approval Required |
|------|---------------|-------------------|
| Engineer | Repository read/write, dev environment, CI/CD | VP Engineering |
| DevOps | Infrastructure, deployment, secrets management | VP Engineering |
| Security Engineer | Read access to all systems, security tooling, audit logs | VP Engineering |
| VP Engineering | All engineering systems, approval authority | CEO |
| CEO | Business systems, final approval authority | Self (logged) |

### 4.2 Agent Identities

Each Converge AI agent has a verified identity consisting of:
- **Agent ID**: Unique, immutable identifier assigned at registration
- **Role**: Defined in `agents/<role>/AGENTS.md`
- **Chain of command**: Explicit reporting hierarchy
- **Scope**: The set of context keys, resources, and actions the agent may access

Agent identities are:
- Registered via Paperclip API (`POST /api/companies/{id}/agents`) **[Implemented]**
- Will be verified before context contributions are accepted (REF-10, planned) **[Planned]**
- Non-transferable — an agent cannot act under another agent's identity **[Design principle, enforcement via REF-10]**
- Revocable by VP Engineering or CEO **[Implemented via Paperclip API]**

### 4.3 Service Identities

| Service | Identity Method | Credential Storage |
|---------|----------------|-------------------|
| LLM providers (OpenAI, Anthropic, etc.) | API key | Secret Manager |
| SurrealDB / LanceDB | Connection credentials | Secret Manager |
| Cloud infrastructure | Service accounts with IAM roles | Cloud IAM |
| CI/CD (GitHub Actions) | OIDC federation or deploy keys | GitHub Secrets |
| Monitoring (Grafana, etc.) | API tokens | Secret Manager |

## 5. Access Provisioning

### 5.1 Request Process

1. Requester submits access request specifying: identity, resource, access level, justification, duration
2. Approver (per Section 4.1 approval matrix) reviews and grants or denies
3. Access is provisioned via infrastructure-as-code or API — not manual console changes
4. Provisioning is logged in the access audit trail

### 5.2 Onboarding

New human operators receive:
- Repository access (read/write to assigned crates only)
- Development environment credentials
- CI/CD pipeline access (read-only by default)
- Security awareness briefing (this policy + Information Security Policy)

New AI agents receive:
- Agent ID registration via Paperclip API
- Context key scope assignment
- Cedar policy rules defining authorized actions
- Budget limits (convergence cycle and resource caps)

### 5.3 Offboarding

When a human operator or agent is deactivated:
1. All access credentials are revoked within 24 hours
2. Active sessions are terminated
3. Shared secrets the identity had access to are rotated
4. Offboarding is logged in the access audit trail
5. For pilot-specific access: integration credentials are revoked at pilot end (per Data Retention Policy)

## 6. Converge-Specific Access Controls

### 6.1 Context Key Ownership — *Planned (REF-10)*

- Each context key will have a designated owner (agent or service). **[Designed in converge-traits]**
- Only the owner will be permitted to write to a context key. **[Planned — REF-10]**
- Ownership will be enforced at the engine level, not just by convention. **[Planned — REF-10]**
- Ownership changes will require VP Engineering approval. **[Process defined, enforcement pending]**
- **Current state:** Context key data structures exist. Ownership enforcement is not yet implemented.

### 6.2 Cedar Policy Enforcement — *Planned (Wave 2)*

- Authorization decisions within the convergence loop will be evaluated by Cedar policies when `converge-policy` is implemented. **[Planned — Wave 2]**
- Policies will be defined in the `converge-policy` crate. **[Crate does not yet exist]**
- Policy changes will follow the Change Management Policy (code review + approval). **[Process defined]**
- Policies will be versioned and auditable. **[Planned]**
- **Current state:** Cedar is the selected authorization framework. The `converge-policy` crate is in the Wave 2 backlog.

### 6.3 WASM Module Authorization — *Planned (REF-6, Wave 4)*

- WASM modules will execute only if signature-verified when REF-6 is implemented. **[Planned — REF-6, Wave 4]**
- Module authors will be registered identities. **[Planned]**
- Module loading will be logged with identity, checksum, and resource limits. **[Planned]**
- Unsigned or tampered modules will be rejected and flagged. **[Planned]**
- **Current state:** WASM runtime (converge-runtime) is Wave 4 backlog. Module signing spec is ready for implementation day one.

### 6.4 LLM Boundary

- LLM provider access is service-identity only (no human operators call LLM APIs directly in production)
- LLM outputs are never granted write access to the Fact store — only ProposedFact
- LLM API keys are rotated quarterly

## 7. Privileged Access

### 7.1 Definition

Privileged access includes:
- Production database read/write
- Secret Manager access
- Infrastructure provisioning (cloud resources)
- Deployment to production
- Cedar policy modification
- Agent registration or deactivation

### 7.2 Controls

- Privileged access requires MFA
- Privileged sessions are time-bounded (maximum 8 hours)
- All privileged actions are logged with identity, action, resource, and timestamp
- Emergency access ("break glass") is available with post-incident review required within 24 hours

## 8. Access Reviews

| Review Type | Frequency | Reviewer | Scope |
|-------------|-----------|----------|-------|
| Human access | Quarterly | VP Engineering | All human operator access |
| Agent access | Quarterly | Security Engineer | All agent scope and Cedar policies |
| Service credentials | Quarterly | DevOps + Security | All API keys, tokens, service accounts |
| Privileged access | Monthly | VP Engineering + Security | All users with privileged access |
| Pilot access | At pilot end | Security Engineer | All pilot-specific credentials and data access |

### 8.1 Review Process

1. Generate access inventory report (all identities, their access, last use date)
2. Reviewer verifies each grant is still justified
3. Unused access (no use in 90 days) is revoked unless justified
4. Review completion is logged in the audit trail

## 9. Credential Management

- All passwords: minimum 16 characters, generated by password manager
- API keys: generated, never hand-crafted, stored in Secret Manager
- SSH keys: Ed25519, passphrase-protected
- Certificates: automated renewal (Let's Encrypt or equivalent)
- Rotation schedule: quarterly for all service credentials, immediately upon compromise suspicion

## 10. Exceptions

Exceptions to this policy require:
1. Written justification with risk assessment
2. Approval by VP Engineering and Security Engineer
3. Time-bounded exception (maximum 90 days, renewable)
4. Logged in the policy exception register
5. Reviewed at next quarterly access review

---

*This policy is a living document. Updates follow the Change Management Policy.*
