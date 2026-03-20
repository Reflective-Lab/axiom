# Change Management Policy — Converge

> Version: 0.1 (Draft)
> Owner: Ava Petrov, Security Engineer
> Approved by: [Pending — Ren Akiyama, VP Engineering]
> Effective date: [Pending approval]
> Review cadence: Annually
> SOC 2 mapping: CC8.1

---

## 1. Purpose

This policy ensures that changes to Converge platform code, infrastructure, and configuration are authorized, tested, and traceable. It prevents unauthorized or untested changes from reaching production.

## 2. Scope

Applies to all changes to:
- Converge crate source code (all 15 crates across 5 waves)
- Infrastructure configuration (deployment, networking, secrets)
- CI/CD pipeline definitions
- Dependencies (Cargo.toml changes)
- Agent configurations and policies (Cedar rules, when converge-policy is implemented in Wave 2)

## 3. Change Categories

| Category | Description | Approval Required | Examples |
|----------|-------------|-------------------|----------|
| **Standard** | Routine development, bug fixes, feature work | 1 peer review | New trait implementation, test additions |
| **Security** | Changes affecting trust boundaries, auth, encryption, secrets | Peer review + Security Engineer | ProposedFact validation, Cedar policy changes, secrets handling |
| **Emergency** | Urgent fix for active incident (SEV-1 or SEV-2) | Post-hoc review within 24 hours | Hotfix for data exposure, credential rotation |
| **Infrastructure** | Deployment, network, secrets rotation | DevOps + VP Engineering | TLS certificate update, secret rotation |

## 4. Change Process

### 4.1 Standard Changes
1. Developer creates a feature branch from `main`.
2. Changes are committed with descriptive messages.
3. Pull request opened with description of what changed and why.
4. At least one peer review approval required.
5. CI checks pass (build, test, lint, `cargo audit`).
6. Merge to `main` via squash or merge commit.

### 4.2 Security Changes
All steps from 4.1, plus:
- Security Engineer (Ava Petrov) must review and approve.
- Changes to trust boundaries (ProposedFact/Fact, Cedar policies, agent identity) require explicit security sign-off in PR comments.

### 4.3 Release Changes
1. Release candidate prepared by DevOps Release Engineer.
2. Security gate review per HEARTBEAT.md Section 6.
3. Security Engineer posts verdict (PASS / CONDITIONAL PASS / BLOCK).
4. BLOCK verdict halts release until critical/high findings are resolved.
5. Release tagged and deployed only after security sign-off.

### 4.4 Emergency Changes
1. Engineer applies fix directly (may bypass standard PR flow).
2. Fix deployed immediately.
3. Post-hoc PR created within 24 hours for review.
4. Incident review documents the emergency change.
5. Security Engineer verifies no security regression introduced.

## 5. Dependency Changes

Any modification to `Cargo.toml` or `Cargo.lock` requires:
- Justification for new dependencies.
- `cargo audit` run with zero critical/high findings.
- Review of dependency license, maintenance status, and transitive surface.
- Security Engineer review if the dependency handles: user input, network I/O, cryptography, or serialization.

## 6. Rollback

- All changes are reversible via `git revert`.
- Deployment rollback procedures documented by DevOps.
- Rollback does not require full review cycle but requires post-hoc documentation.

## 7. Audit Trail

- All changes tracked in git history with author attribution.
- PR reviews and approvals recorded in GitHub.
- Release gate verdicts recorded as Paperclip issue comments.
- Emergency changes logged in incident review documents.

---

*This document is a living draft subject to updates as the engineering process matures.*
