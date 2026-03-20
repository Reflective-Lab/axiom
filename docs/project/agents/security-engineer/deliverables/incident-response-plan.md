# Incident Response Plan — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering / Morgan Vale, CEO]
> Effective date: [Pending approval]
> Review cadence: Annually, or after every Severity 1-2 incident
> SOC 2 mapping: CC7.2, CC7.3, CC7.4

---

## 1. Purpose

This plan defines how Converge detects, responds to, and recovers from security incidents. It establishes roles, communication protocols, and procedures to minimize impact and restore operations quickly.

## 2. Scope

This plan covers incidents affecting:
- Converge platform components (all crates, infrastructure, data stores)
- Customer data during pilot engagements
- Third-party integrations and dependencies
- Agent integrity and convergence correctness

## 3. Incident Severity Levels

| Level | Name | Definition | Response Time | Examples |
|-------|------|-----------|---------------|----------|
| SEV-1 | Critical | Active exploitation, data breach, or complete service outage | Triage within 15 min | Unauthorized data access, production compromise, customer data exfiltration |
| SEV-2 | High | Vulnerability with clear exploit path, partial outage, or data integrity risk | Triage within 1 hour | Unauthenticated API endpoint, LLM boundary bypass, agent impersonation |
| SEV-3 | Medium | Vulnerability requiring specific conditions, degraded functionality | Triage within 4 hours | WASM signature bypass (requires local access), policy misconfiguration |
| SEV-4 | Low | Informational finding, minor misconfiguration, no immediate risk | Triage within 24 hours | Verbose error messages, non-sensitive log exposure, outdated dependency |

## 4. Incident Response Team

| Role | Primary | Backup | Responsibilities |
|------|---------|--------|-----------------|
| Incident Commander | Ren Akiyama (VP Eng) | Morgan Vale (CEO) | Decision authority, communication, resource allocation |
| Security Lead | Ava Petrov | Eli Marsh | Technical investigation, containment, forensics |
| Engineering Lead | Eli Marsh | Kira Novak | Remediation implementation, deployment |
| DevOps Lead | Dex Tanaka | Eli Marsh | Infrastructure containment, log collection, deployment |
| Communications | Morgan Vale | Blake Harmon | Customer notification, public disclosure |

## 5. Response Phases

### Phase 1: Detection

**Automated sources:**
- Monitoring alerts: convergence anomalies, error rate spikes, resource limit violations
- `cargo audit`: dependency vulnerability alerts
- Pre-commit hooks: PII in pilot data (`pilot-data-precommit.sh`)
- Cedar policy violation logs: unauthorized access attempts

**Manual sources:**
- Security review findings (release gate, backlog review)
- External reports: responsible disclosure (security@converge.zone)
- Customer reports: pilot irregularities
- Team observations: unusual system behavior

**Detection → Triage trigger:**
Any of the above creates an incident ticket in Paperclip with `priority: critical|high` and tags `incident`, `security`.

### Phase 2: Triage

1. **Assign Incident Commander** (VP Eng by default)
2. **Assess severity** using the table in Section 3
3. **Determine scope**: which systems, data, and customers are affected
4. **Activate response team** per severity:
   - SEV-1/2: Full team, synchronous coordination
   - SEV-3/4: Security Lead + Engineering Lead, async

### Phase 3: Containment

**Immediate containment (minutes to hours):**
- Isolate affected workspaces (disable convergence runs)
- Revoke compromised credentials
- Block attacker access (IP, API key, agent ID)
- Preserve evidence: snapshot logs, database state, affected files before any changes

**Short-term containment (hours to days):**
- Deploy targeted fix or workaround
- Rotate all credentials the attacker may have accessed
- Enable enhanced monitoring on affected systems

### Phase 4: Eradication

1. Identify root cause (not just symptoms)
2. Develop and test fix
3. Code review: fix must pass security review before merge
4. Deploy fix following Change Management Policy (emergency path for SEV-1/2)
5. Verify eradication: confirm the vulnerability is no longer exploitable

### Phase 5: Recovery

1. Restore affected systems to normal operation
2. Re-enable disabled workspaces
3. Verify data integrity (convergence results, experience store, customer data)
4. Confirm monitoring is back to baseline
5. Communicate recovery status to affected customers

### Phase 6: Post-Incident Review

**Required for all SEV-1 and SEV-2 incidents. Optional for SEV-3/4.**

Within 5 business days of incident resolution:

1. **Timeline reconstruction**: minute-by-minute account of detection through recovery
2. **Root cause analysis**: what failed and why
3. **Impact assessment**: which systems, data, and customers were affected
4. **Response assessment**: what worked well, what could improve
5. **Action items**: specific, assigned, time-bounded improvements
6. **Policy updates**: update this plan, Information Security Policy, or other controls as needed

Post-incident review document stored in `agents/security-engineer/deliverables/incidents/` (with sensitive details redacted for general access).

## 6. Communication Protocols

### Internal Communication

| Severity | Channel | Frequency |
|----------|---------|-----------|
| SEV-1 | Synchronous (call/chat) + Paperclip issue | Continuous during active response |
| SEV-2 | Paperclip issue + direct messages | Every 2 hours during active response |
| SEV-3 | Paperclip issue | Daily updates until resolved |
| SEV-4 | Paperclip issue | At resolution |

### Customer Communication

| Timing | Content | Audience | Owner |
|--------|---------|----------|-------|
| Within 4 hours (SEV-1/2) | Acknowledgment: we are aware and investigating | Affected customers | CEO |
| Within 24 hours | Status update: scope, containment actions taken | Affected customers | CEO |
| Within 72 hours | Full disclosure: what happened, impact, remediation | Affected customers | CEO + Security Lead |
| At resolution | Final report: root cause, actions taken, preventive measures | Affected customers | CEO + Security Lead |

### Regulatory Notification

- GDPR: data breach involving EU personal data requires DPA notification within 72 hours
- Customer contractual obligations: per pilot agreement terms
- Legal counsel: engaged for any SEV-1 incident involving customer data

## 7. Converge-Specific Incident Scenarios

### 7a. LLM Boundary Bypass

**Scenario:** ProposedFact promoted to Fact without validation.
**Severity:** SEV-1 (data integrity + trust violation)
**Containment:**
- **Available now:** Halt all convergence runs. The ProposedFact→Fact boundary is enforced via `Fact::try_from(proposal)` (REF-8, implemented). Inspect engine logs for bypass evidence.
- **Available after Wave 3:** Audit experience store for unvalidated facts. Roll back affected convergence results via experience store API.

### 7b. Agent Impersonation

**Scenario:** Context contributions attributed to a spoofed agent identity.
**Severity:** SEV-2 (integrity violation, limited by agent scope)
**Containment:**
- **Available now:** Deregister affected agent via Paperclip API. Manually audit context contributions in the affected workspace.
- **Available after REF-10:** Disable affected agent ID at the engine level (cryptographic identity revocation). Automated audit of context contributions. Re-run affected convergence cycles with verified identities.

### 7c. Context Poisoning

**Scenario:** Malicious or incorrect facts injected into a convergence context.
**Severity:** SEV-2 (depends on downstream impact)
**Containment:**
- **Available now:** Quarantine affected workspace (halt convergence runs). Manual inspection of context data.
- **Available after Wave 3:** Trace poison source via experience store audit trail. Invalidate poisoned convergence results programmatically.

### 7d. WASM Module Tampering

**Scenario:** Modified WASM module loaded without valid signature.
**Severity:** SEV-1 (arbitrary code execution in convergence pipeline)
**Containment:**
- **Available now:** Halt all module loading. Quarantine affected runtime environment. Compare module checksums against known-good copies manually.
- **Available after REF-6 (Wave 4):** Verify all loaded module checksums against signed registry automatically. Reject unsigned modules at load time.

### 7e. Pilot Data Breach

**Scenario:** Raw customer pilot data accessed by unauthorized party.
**Severity:** SEV-1 (customer data exposure)
**Containment:** Revoke all pilot integration credentials. Preserve access logs. Initiate customer notification per Section 6.

### 7f. Supply Chain Compromise

**Scenario:** Compromised Rust crate in dependency tree.
**Severity:** SEV-1 if in production, SEV-2 if in development only
**Containment:** Pin to last-known-good version. Audit Cargo.lock for affected versions. Check if compromised code executed in any environment.

## 8. Emergency Access

During SEV-1/2 incidents:
- Incident Commander may authorize emergency ("break glass") access to any system
- Emergency access is logged automatically
- Post-incident: all emergency access is reviewed and revoked within 24 hours of resolution
- Emergency access events are included in the post-incident review

## 9. Testing

| Test Type | Frequency | Scope |
|-----------|-----------|-------|
| Tabletop exercise | Quarterly | Walk through a scenario with the response team |
| Communication test | Semi-annually | Verify notification channels and escalation paths work |
| Full simulation | Annually | End-to-end incident simulation (detection through recovery) |

## 10. Plan Maintenance

- This plan is reviewed annually or after every SEV-1/2 incident
- Action items from post-incident reviews may trigger plan updates
- Changes follow the Change Management Policy
- Current version is always available in `agents/security-engineer/deliverables/`

---

*This plan is a living document. Contact security@converge.zone for questions.*
