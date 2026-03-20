# Security Awareness & Training Program

**Version:** 0.1 (Draft)
**Owner:** Ava Petrov, Security Engineer
**SOC 2 Mapping:** CC1.4, CC2.2, CC2.3
**Last Updated:** 2026-03-12

---

## 1. Purpose

Establish a security awareness and training program that ensures all team members (human and agent) understand their security responsibilities, recognize threats, and follow secure practices when building and operating the Converge platform.

## 2. Scope

- All engineering team members
- All AI agents operating within the Converge development environment
- External contributors with access to source code or customer data
- Contractors and consultants

## 3. Training Requirements

### 3.1 Onboarding Training (Required — First Week)

All new team members must complete before accessing production systems:

| Topic | Duration | Format |
|-------|----------|--------|
| Information Security Policy overview | 30 min | Self-paced document review |
| Access control and credential management | 30 min | Self-paced + verification |
| Data classification and handling | 30 min | Self-paced document review |
| Secrets management (no secrets in code/env/logs) | 15 min | Checklist + verification |
| Incident reporting procedures | 15 min | Self-paced document review |
| Acceptable use policy | 15 min | Self-paced document review |

**Verification:** New team member acknowledges completion and understanding in writing.

### 3.2 Converge-Specific Security Training (Required — First Month)

Platform-specific security knowledge:

| Topic | Audience | Duration |
|-------|----------|----------|
| ProposedFact vs Fact boundary | All engineers | 45 min |
| Context key integrity and agent identity | All engineers | 30 min |
| Cedar policy evaluation | Engineers working on auth | 45 min |
| WASM module security model | Engineers working on runtime | 30 min |
| LLM prompt injection risks | Engineers working with LLM providers | 45 min |
| Pilot data handling and PII requirements | All engineers | 30 min |

### 3.3 Annual Refresher Training (Required)

All team members complete annually:

| Topic | Duration |
|-------|----------|
| Security policy updates and changes | 30 min |
| Emerging threat landscape (supply chain, AI-specific) | 30 min |
| Incident review — lessons from past year | 30 min |
| Secure coding practices refresher | 30 min |

### 3.4 Role-Specific Training

| Role | Additional Training |
|------|-------------------|
| DevOps | Infrastructure hardening, secret rotation procedures, backup/restore |
| Frontend | XSS prevention, CSP headers, client-side data handling |
| Backend/Rust | Memory safety patterns, unsafe code review, dependency auditing |
| QA | Security testing methodology, vulnerability verification |
| AI Agents | Secure prompt construction, output validation, boundary enforcement |

## 4. Agent-Specific Security Guidelines

AI agents operating in the Converge environment must be configured with:

1. **No secret exfiltration** — agents must never output secrets, API keys, or credentials
2. **No destructive commands** — unless explicitly authorized by VP Engineering or CEO
3. **Responsible disclosure** — vulnerability details stay internal, not in public comments
4. **Data minimization** — agents access only the data needed for their current task
5. **Audit trail** — all agent actions that modify code or infrastructure are logged
6. **Pre-commit compliance** — all agent commits pass pre-commit hooks (PII scan, etc.)

## 5. Security Champions Program

Designate one engineer per team as a Security Champion:

**Responsibilities:**
- Attend monthly security review meetings
- Serve as first point of contact for security questions within their team
- Review PRs for common security issues (OWASP Top 10)
- Escalate potential vulnerabilities to the Security Engineer

**Current Champions:**
- Core/Traits: Eli Marsh (Founding Engineer)
- Provider/Infrastructure: Kira Novak (Senior Rust Developer)
- Frontend: Jules Carrera (Frontend Developer)
- DevOps: Dex Tanaka (DevOps Release Engineer)

## 6. Phishing and Social Engineering Awareness

- Quarterly simulated phishing exercises
- Report suspicious communications to #security Slack channel
- Never share credentials, even with team members claiming urgency
- Verify identity before granting access or sharing sensitive information

## 7. Secure Development Practices

All engineers must follow:

1. **No secrets in code** — use Google Secret Manager or Vault
2. **Input validation** — validate at all external boundaries
3. **Parameterized queries** — never interpolate user input into queries
4. **Dependency review** — justify and review all new dependencies
5. **Pre-commit hooks** — never bypass with `--no-verify`
6. **Code review** — all changes reviewed before merge, security-relevant changes by Security Engineer
7. **Least privilege** — request minimum permissions needed

## 8. Metrics and Tracking

| Metric | Target | Frequency |
|--------|--------|-----------|
| Onboarding training completion | 100% within 7 days | Per hire |
| Annual refresher completion | 100% by anniversary | Annual |
| Phishing simulation click rate | < 5% | Quarterly |
| Security champion meeting attendance | > 80% | Monthly |
| Time to report security incidents | < 1 hour | Per incident |

## 9. Non-Compliance

- First occurrence: Reminder and re-training
- Repeated non-compliance: Escalation to VP Engineering
- Willful disregard of security policies: Disciplinary action per HR policy

## 10. Program Review

- Training content reviewed and updated biannually
- Program effectiveness assessed annually via metrics review
- Post-incident training gaps addressed within 30 days
- New Converge-specific threats added to training within 60 days of identification

---

## Appendix: Training Record Template

| Name | Role | Onboarding Date | Onboarding Complete | Converge Training Complete | Last Annual Refresher |
|------|------|-----------------|--------------------|-----------------------------|----------------------|
| | | | | | |
