# Converge Security Overview

> Content for converge.zone/security — ready for Rio to design and Jules to build.

---

## Security at Converge

Converge is a semantic governance platform that orchestrates AI agents, business rules, and optimization to converge on correct outcomes. Security is built into the architecture, not bolted on after the fact.

---

## Architecture Security

### Trust Boundaries

Converge enforces strict trust boundaries at every layer:

- **LLM outputs are untrusted.** All LLM responses produce `ProposedFact` — never `Fact`. Every proposed fact passes through invariant validation before it can influence convergence outcomes.
- **Agent identity is verified.** Each agent has a verified identity. Context contributions are tagged with the agent's identity and cannot be forged.
- **Policy enforcement is a first-class agent.** Cedar policies evaluate authorization decisions as part of the convergence loop, not as an afterthought.

### WASM Sandboxing

Domain logic runs as signed WASM modules in a sandboxed runtime:

- Modules are isolated from each other and from the host system
- All modules are signature-verified before loading
- Resource limits (memory, CPU) are enforced per module
- No direct filesystem or network access from within WASM modules

---

## Data Handling

### Data in Transit

- All external communication uses TLS 1.2+
- Internal service-to-service communication is encrypted
- gRPC connections use TLS with certificate verification

### Data at Rest

- Experience store data is encrypted at rest
- Database credentials are managed through secret management systems (Google Secret Manager / HashiCorp Vault)
- No secrets in source code, configuration files, or logs

### Data Classification

- **Context data** — business facts contributed by agents during convergence. Scoped per workspace.
- **Experience data** — event-sourced audit trail of all convergence runs. Append-only, tamper-evident.
- **Configuration data** — workspace settings, agent configurations, policy definitions. Access-controlled.

---

## Access Controls

### Authentication

- API endpoints require authentication
- Agent identity is cryptographically verified
- No anonymous access to production APIs

### Authorization

- Cedar policy engine enforces fine-grained authorization
- Context keys have ownership — agents can only write to their assigned keys
- Role-based access for workspace management

### Audit Trail

- Every convergence run is fully traced in the experience store
- All agent contributions are attributed to verified identities
- Event-sourced architecture enables full replay and forensic analysis
- Append-only storage prevents retroactive modification

---

## Incident Response

### Monitoring

- Observability built into converge-runtime (metrics, tracing, logging)
- Anomaly detection on convergence patterns (unbounded loops, budget overruns)
- Logging excludes secrets, tokens, and PII

### Response Process

1. **Detection** — automated monitoring + manual review
2. **Triage** — severity assessment within 1 hour
3. **Containment** — isolate affected workspaces
4. **Remediation** — patch and deploy fix
5. **Disclosure** — notify affected customers within 72 hours

### Responsible Disclosure

We welcome security researchers to report vulnerabilities. Contact: security@converge.zone

---

## Supply Chain Security

- All Rust dependencies audited via `cargo audit`
- New dependencies require justification, license review, and maintenance status check
- WASM modules are signed and integrity-verified before loading
- CI/CD pipeline uses locked dependencies (Cargo.lock committed)

---

## Compliance Roadmap

| Milestone | Target | Status |
|-----------|--------|--------|
| Security one-pager (this page) | Q1 2026 | In progress |
| GDPR data processing statement | Q2 2026 | Planned |
| Data residency policy | Q2 2026 | Planned |
| SOC 2 Type I readiness | Q3 2026 | Planned |
| SOC 2 Type I audit | Q4 2026 | Planned |

---

## Questions?

For security inquiries, contact **security@converge.zone**.
For responsible disclosure, see our vulnerability reporting policy.

---

*Last updated: 2026-03-11*
