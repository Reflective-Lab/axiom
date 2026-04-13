---
source: mixed
---
# MergeToMain Gate E2E Walkthrough

> Narrative walkthrough of the PR Merge gate flow with pass and blocked scenarios.

**Cross-references:**
- Gate policy: [GATES.md](../../GATES.md) - PR Merge section
- Personas: [TEAM.md](../../TEAM.md) - Core technical team
- Fixtures: `fixtures/gates/merge_to_main.pass.yaml`, `fixtures/gates/merge_to_main.blocked.yaml`

---

## Gate Overview

**Gate ID:** `pr-merge`
**Gate Name:** PR Merge
**Promotion Target:** main branch
**Risk Class:** medium

The PR Merge gate controls what code enters the main branch. It aggregates architecture, security, and quality evaluations to ensure code meets standards before merge. This gate has medium risk because merges can be reverted if issues are discovered later.

**Why medium risk?** Merging to main is reversible - problematic code can be reverted or fixed in follow-up PRs. This differs from high-risk gates like releases or deployments where reversal is costly or impossible.

---

## Entry Conditions

The PR Merge gate is triggered when:

1. **Pull request created** - Developer opens PR against main branch
2. **CI pipeline passes** - Automated tests, linting, and build succeed
3. **PR marked ready** - Draft status removed, ready for review
4. **Required reviews obtained** - Code review approval from relevant team member

Once these conditions are met, the gate evaluations begin.

---

## Required Evaluations

The PR Merge gate requires evaluations from the **core-technical** pack:

| Eval ID | Persona | Authority Tier | Determinism Class | What It Checks |
|---------|---------|----------------|-------------------|----------------|
| system-architect-eval | System Architect | Core Blocking-by-Policy | Class A (Deterministic) | Architecture consistency, pattern compliance, dependency hygiene |
| qa-engineer-eval | QA Engineer | Core Blocking-by-Policy | Class A (Deterministic) | Test coverage thresholds, test quality, regression suite pass |
| security-auditor-eval | Security Auditor | Core Blocking-by-Policy | Class A (Deterministic) | Static security scans, dependency vulnerabilities, secret detection |

All three evals must PASS for the PR to merge. If any eval returns FAIL, the PR is blocked until issues are resolved.

**Note on determinism:** These are all Class A (Deterministic) evals - they run static checks on the codebase that produce identical results for identical code. There's no human judgment or external variability at this gate.

---

## Evidence Requirements

The gate requires the following evidence artifacts before approval:

- **code-review-approval** - At least one code review approval from a team member
- **ci-pass** - CI pipeline green status (all tests pass, build succeeds)
- **architecture-eval-pass** - System Architect eval returns PASS
- **qa-eval-pass** - QA Engineer eval returns PASS

These artifacts create an audit trail showing the promotion decision was evidence-based, not arbitrary.

---

## Scenario 1: Pass (Happy Path)

### Input Context

**Pull Request:** #847
**Commit SHA:** `a7b3c2d`
**Author:** developer-jane
**Description:** "Add minimal tokio features to reduce binary size"

**Changes:**
- Modified `Cargo.toml` to use `tokio = { version = "1.36", features = ["rt", "fs"] }` instead of `tokio[full]`
- Removed unused async-io and tracing dependencies
- Updated integration tests to verify functionality preserved

### Evaluation Execution

**1. system-architect-eval:**
- Checked dependency tree for architecture violations
- Verified minimal feature set aligns with project dependency policy
- Confirmed no new external dependencies introduced
- **Result:** PASS - "Dependency reduction improves maintainability, minimal features sufficient"

**2. qa-engineer-eval:**
- Verified test coverage remains above 80% threshold
- Ran full regression suite (487 tests)
- Checked for new test coverage on modified code paths
- **Result:** PASS - "All tests pass, coverage maintained at 84%, no regressions"

**3. security-auditor-eval:**
- Scanned for new vulnerabilities in updated dependencies
- Verified no secrets or credentials in changeset
- Checked dependency license compatibility
- **Result:** PASS - "No new vulnerabilities, license compatibility confirmed"

### Evidence Collected

- **code-review-approval:** Code review approval from senior-dev-alice
- **ci-pass:** CI pipeline green (build #1247 passed)
- **architecture-eval-pass:** system-architect-eval returned PASS
- **qa-eval-pass:** qa-engineer-eval returned PASS

### Decision

**Gate status:** APPROVED
**Promotion executed:** PR #847 merged to main at 2026-01-26T15:30:00Z
**Merge commit:** `e9f2a1b`

### What Happens Next

- PR #847 status updated to "merged"
- Main branch CI runs on merge commit
- Binary size reduction tracked in release notes
- Developer notified of successful merge

**Fixture reference:** See `fixtures/gates/merge_to_main.pass.yaml` for machine-readable version of this scenario.

---

## Scenario 2: Blocked (QA Eval Fails)

### Input Context

**Pull Request:** #852
**Commit SHA:** `c4d5e6f`
**Author:** developer-bob
**Description:** "Refactor authentication flow to support OAuth2"

**Changes:**
- Rewrote authentication module to support OAuth2 providers
- Added new `oauth` crate dependency
- Modified session management logic
- Updated API endpoints for OAuth callback handling

### Evaluation Execution

**1. system-architect-eval:**
- Checked OAuth integration follows existing auth patterns
- Verified new dependency is maintained and secure
- Confirmed API design aligns with REST conventions
- **Result:** PASS - "OAuth integration follows architecture guidelines, dependency acceptable"

**2. qa-engineer-eval:**
- Verified test coverage on new authentication code
- Checked for regression tests on session management changes
- Analyzed test quality for OAuth edge cases
- **Result:** FAIL - "Test coverage dropped to 62% (below 80% threshold). Missing tests for OAuth token refresh, session expiry edge cases, and failure scenarios. Regression tests needed for modified session logic."

**3. security-auditor-eval:**
- Scanned OAuth dependency for known vulnerabilities
- Reviewed OAuth implementation for security best practices
- Checked secret management for OAuth client credentials
- **Result:** PASS - "No vulnerabilities in oauth crate v0.16.1, implementation follows OWASP guidelines"

### Evidence Collected

- **code-review-approval:** Code review approval from senior-dev-alice
- **ci-pass:** CI pipeline green (build #1251 passed)
- **architecture-eval-pass:** system-architect-eval returned PASS
- **qa-eval-pass:** ❌ qa-engineer-eval returned FAIL (below coverage threshold, missing edge case tests)

### Decision

**Gate status:** BLOCKED
**Promotion denied:** PR #852 cannot merge until QA eval passes
**Blocking eval:** qa-engineer-eval

### What Developer Must Do

1. Add missing test coverage for OAuth token refresh logic
2. Write tests for session expiry edge cases (expired tokens, invalid refresh tokens)
3. Add regression tests for modified session management code
4. Increase overall coverage back to 80%+ threshold
5. Push updated commit to PR #852
6. Gate re-evaluates automatically on new commit

### Iteration Flow

After developer-bob adds missing tests:

**Iteration 2:**
- New commit: `f7g8h9i`
- qa-engineer-eval re-runs
- **Result:** PASS - "Coverage now 83%, all edge cases tested, regression suite comprehensive"
- Gate approves, PR merges

**Stop rule enforcement:** Max 2 iterations allowed. If QA eval still fails after iteration 2, 24h timeout triggers and System Architect or QA Engineer makes tie-breaker decision (proceed with override + compensating controls, or close PR).

**Fixture reference:** See `fixtures/gates/merge_to_main.blocked.yaml` for machine-readable version of this scenario.

---

## Override Policy

**Override type:** single-person-with-rationale
**Risk class:** medium

If a required eval fails and override is needed:

1. **Who can override:** Any relevant Core persona (System Architect, QA Engineer, Security Auditor)
2. **Rationale required:** Yes - must document why eval failure is acceptable
3. **Compensating controls:** Recommended but not required

**Example override scenario:** QA eval fails due to integration test flakiness (not real coverage gap). QA Engineer overrides with rationale: "Coverage threshold met, test failure is infrastructure flakiness not code issue. Compensating control: Manual smoke test performed, integration test fix tracked in ticket #860."

**Override audit trail:**
```yaml
override_record:
  date: 2026-01-26
  gate: pr-merge
  failed_eval: qa-engineer-eval
  approver: qa-engineer
  rationale: "Integration test flakiness, not real coverage gap"
  compensating_controls:
    - "Manual smoke test performed on staging"
    - "Integration test fix tracked in ticket #860"
```

---

## Key Takeaways

1. **PR Merge is medium-risk** - Merges are reversible, so override is single-person with rationale (not two-person)
2. **Core-technical pack is required** - All three Core personas (Architect, QA, Security) must approve
3. **Deterministic evals only** - No human judgment at this gate, all Class A deterministic checks
4. **Evidence creates audit trail** - Code review, CI pass, and eval results document decision
5. **Stop rules prevent infinite loops** - Max 2 iterations, 24h timeout, tie-breaker authority defined

**For auditors:** This gate demonstrates that code entering main branch has been reviewed for architecture, quality, and security. The evidence trail (code review + CI + evals) provides post-hoc verification of promotion decisions.

---

*Fixtures for this walkthrough will be created in Phase 5 Plan 03 (Machine-Verifiable Fixtures).*
