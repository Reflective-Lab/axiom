---
source: mixed
---
# ReleaseCut Gate E2E Walkthrough

> Narrative walkthrough of the Release Candidate gate flow with pass, blocked, and override scenarios.

**Cross-references:**
- Gate policy: [GATES.md](../../GATES.md) - Release Candidate section
- Personas: [TEAM.md](../../TEAM.md) - Core technical and Extended operational teams
- Fixtures: `fixtures/gates/release_cut.pass.yaml`, `fixtures/gates/release_cut.blocked.yaml`, `fixtures/gates/release_cut.override.yaml`

---

## Gate Overview

**Gate ID:** `release-candidate`
**Gate Name:** Release Candidate
**Promotion Target:** RC tag (e.g., `v2.0.0-rc.1`)
**Risk Class:** high

The Release Candidate gate controls RC tagging, marking the transition from active development to release preparation. This gate requires comprehensive technical evaluations from architecture, security, quality, operations, and founder teams. It is **high risk** because RC tags signal readiness for production release and begin external testing/validation workflows.

**Why high risk?** RC tags are semi-public commitments - they trigger QA cycles, stakeholder reviews, and marketing preparation. While not as irreversible as production releases, RC tags create external dependencies and expectations that are costly to walk back.

---

## Entry Conditions

The Release Candidate gate is triggered when:

1. **Release branch cut** - Branch created from main (e.g., `release/v2.0.0`)
2. **Feature freeze complete** - No new features added, only bug fixes and stabilization
3. **Technical suite green** - All tests passing, no critical issues in backlog
4. **Build reproducible** - Release build produces identical artifacts across environments

Once these conditions are met, the gate evaluations begin.

---

## Required Evaluations

The Release Candidate gate requires evaluations from **core-technical** and **operational** packs:

### Core-Technical Pack (Required)

| Eval ID | Persona | Authority Tier | Determinism Class | What It Checks |
|---------|---------|----------------|-------------------|----------------|
| system-architect-eval | System Architect | Core Blocking-by-Policy | Class A (Deterministic) | Architecture consistency, tech debt acceptable, API stability |
| qa-engineer-eval | QA Engineer | Core Blocking-by-Policy | Class A (Deterministic) | Test coverage thresholds, regression suite pass, no flaky tests |
| security-auditor-eval | Security Auditor | Core Blocking-by-Policy | Class A (Deterministic) | Vulnerability scan, dependency audit, no critical CVEs |
| founder-eval | Founder | Core Blocking-by-Policy | Class A (Deterministic) | Strategic alignment, release timing, business risk assessment |

### Operational Pack (Elevated)

| Eval ID | Persona | Authority Tier | Determinism Class | What It Checks | Elevation Status |
|---------|---------|----------------|-------------------|----------------|------------------|
| sre-operations-eval | SRE / Operations | Extended Escalating | Class B (Bounded) | Operational readiness, deployment safety, monitoring coverage | **ELEVATED to blocking** |

**Key distinction:** At most gates, `sre-operations-eval` is **advisory only**. At `release-candidate`, it is **elevated to blocking** because operational readiness is critical for RC tagging. This demonstrates the elevation pattern: Extended personas can be blocking at specific high-stakes gates without having blanket authority.

**Determinism note:** SRE eval is Class B (bounded) rather than Class A because operational checks depend on external infrastructure (staging environment health, monitoring systems). While not purely deterministic, the evaluation is bounded by objective criteria (deployment runbook completeness, smoke test results).

---

## Elevated Evaluations

**What is elevation?** Elevation means an Extended persona's eval becomes blocking at a specific gate, even though they're Advisory or Escalating tier by default.

**Why elevate at Release Candidate?**
- Operational readiness is release-critical - bad RC tags waste QA cycles and delay releases
- SRE expertise is essential for validating deployment safety before RC
- Elevating here prevents operational issues from being discovered late in release cycle

**Elevation criteria met:**
1. ✅ **Objective checks** - SRE eval performs deterministic/bounded checks (runbook completeness, smoke tests)
2. ✅ **Risk alignment** - Release Candidate is high-risk gate
3. ✅ **Critical expertise** - SRE operational knowledge is essential at this gate
4. ✅ **Policy documentation** - Elevation explicitly listed in GATES.md policy table

**Elevation lifecycle:**
- At **pr-merge** gate: sre-operations-eval is advisory (provides operational feedback, cannot block)
- At **release-candidate** gate: sre-operations-eval is **blocking** (failure prevents RC tag)
- At **release-approval** gate: sre-operations-eval is blocking (remains elevated through release gates)

This satisfies the requirement: "At least one elevated Extended eval demonstrated" - we show SRE eval transitioning from advisory to blocking based on gate context.

---

## Evidence Requirements

The gate requires the following evidence artifacts before approval:

- **technical-suite-pass** - All automated tests pass (unit, integration, e2e)
- **coverage-threshold-met** - Test coverage meets or exceeds 80% threshold
- **no-critical-vulnerabilities** - Security scan shows no P0/P1 vulnerabilities
- **build-reproducible** - Build produces identical binary hash across environments

These artifacts prove RC readiness is evidence-based, not subjective judgment.

---

## Scenario 1: Pass (Happy Path)

### Input Context

**Release Branch:** `release/v2.1.0`
**RC Tag Target:** `v2.1.0-rc.1`
**Release Manager:** qa-engineer-alice
**Feature Freeze Date:** 2026-01-24

**Changes since v2.0.0:**
- 47 new features (file upload optimization, OAuth2 support, webhook retry logic)
- 23 bug fixes
- Dependency updates (tokio 1.36, axum 0.7)
- Binary size reduced from 45MB to 18MB (sustainability improvement)

### Evaluation Execution

**1. system-architect-eval:**
- Verified no architecture drift from established patterns
- Checked API stability (no breaking changes to public API)
- Reviewed dependency updates for compatibility
- **Result:** PASS - "Architecture consistent, API stable, dependency updates low-risk"

**2. qa-engineer-eval:**
- Ran full regression suite (523 tests, all pass)
- Verified coverage at 84% (above 80% threshold)
- Checked for flaky tests (none detected in last 10 runs)
- Validated known issues are documented and acceptable
- **Result:** PASS - "Test suite comprehensive, coverage excellent, no flaky tests, known issues minor and documented"

**3. security-auditor-eval:**
- Scanned all dependencies for CVEs (none critical)
- Ran static security analysis (SAST) on codebase
- Verified OAuth2 implementation follows OWASP guidelines
- Checked for hardcoded secrets (none found)
- **Result:** PASS - "No critical vulnerabilities, OAuth2 secure, dependency audit clean"

**4. founder-eval:**
- Reviewed release timing against roadmap milestones
- Assessed business risk of new features (OAuth2 adoption, file upload limits)
- Confirmed marketing/sales alignment on release messaging
- **Result:** PASS - "Release aligns with Q1 roadmap, business risk acceptable, go-to-market ready"

**5. sre-operations-eval (ELEVATED):**
- Verified deployment runbook updated for new OAuth2 flows
- Ran smoke tests on staging environment (all pass)
- Confirmed monitoring dashboards cover new features (upload metrics, OAuth latency)
- Validated rollback procedure tested and documented
- **Result:** PASS - "Deployment runbook complete, smoke tests green, monitoring comprehensive, rollback tested"

### Evidence Collected

- **technical-suite-pass:** All 523 tests pass (CI build #1289)
- **coverage-threshold-met:** Coverage 84% (report attached)
- **no-critical-vulnerabilities:** Security scan clean (0 critical, 2 low-severity accepted)
- **build-reproducible:** Binary hash `sha256:a7b3c2d...` identical across 3 build environments

### Decision

**Gate status:** APPROVED
**Promotion executed:** RC tag `v2.1.0-rc.1` created at 2026-01-26T18:00:00Z
**Tagged commit:** `e9f2a1b`

### What Happens Next

- RC tag triggers QA validation cycle (integration testing, UAT)
- Stakeholders notified of RC availability
- Marketing begins release announcement drafts
- Known issues list shared with support team
- Next gate: Release Approval (after QA validation completes)

**Fixture reference:** See `fixtures/gates/release_cut.pass.yaml` for machine-readable version of this scenario.

---

## Scenario 2: Blocked (Security Eval Fails)

### Input Context

**Release Branch:** `release/v2.2.0`
**RC Tag Target:** `v2.2.0-rc.1`
**Release Manager:** qa-engineer-bob
**Feature Freeze Date:** 2026-02-15

**Changes since v2.1.0:**
- 31 new features (payment gateway integration, user data export)
- 18 bug fixes
- Major dependency update (openssl 3.0 -> 3.1)

### Evaluation Execution

**1. system-architect-eval:**
- Architecture review complete
- **Result:** PASS

**2. qa-engineer-eval:**
- Test suite passes, coverage 82%
- **Result:** PASS

**3. security-auditor-eval:**
- Scanned dependencies for CVEs
- Discovered **CVE-2026-1234** in openssl 3.1.2 (CRITICAL severity, RCE vulnerability)
- Payment gateway integration missing encryption validation tests
- **Result:** FAIL - "Critical vulnerability CVE-2026-1234 in openssl 3.1.2 enables remote code execution. Patch available in openssl 3.1.3. Payment integration missing encryption validation tests."

**4. founder-eval:**
- Not evaluated yet (blocked by security failure)

**5. sre-operations-eval (ELEVATED):**
- Not evaluated yet (blocked by security failure)

### Evidence Collected

- **technical-suite-pass:** ✅ All tests pass
- **coverage-threshold-met:** ✅ Coverage 82%
- **no-critical-vulnerabilities:** ❌ FAIL - CVE-2026-1234 critical severity
- **build-reproducible:** ✅ Build hash consistent

### Decision

**Gate status:** BLOCKED
**Promotion denied:** RC tag cannot be created until security eval passes
**Blocking eval:** security-auditor-eval

### What Developer Must Do

1. **Immediate:** Update openssl dependency to 3.1.3 (patches CVE-2026-1234)
2. Add encryption validation tests for payment gateway integration
3. Re-run security scan to verify vulnerability resolved
4. Re-run full test suite to verify openssl update doesn't break functionality
5. Push updated commit to release branch
6. Gate re-evaluates automatically

### Iteration Flow

After security team updates dependency:

**Iteration 2:**
- New commit: `f7g8h9i`
- Updated `Cargo.toml`: `openssl = "3.1.3"`
- Added payment encryption tests (12 new tests)
- Security scan re-runs
- **Result:** PASS - "CVE-2026-1234 resolved, payment encryption validated, no critical issues"

**Remaining evals complete:**
- founder-eval: PASS
- sre-operations-eval: PASS

**Gate approves, RC tag created.**

**Stop rule enforcement:** Max 3 iterations allowed (release candidate allows more iteration than PR merge due to higher stakes). If security eval still fails after iteration 3, 48h timeout triggers and Founder makes tie-breaker decision.

**Fixture reference:** See `fixtures/gates/release_cut.blocked.yaml` for machine-readable version of this scenario.

---

## Scenario 3: Override (SRE Eval Fails, Two-Person Override)

### Input Context

**Release Branch:** `release/v2.3.0`
**RC Tag Target:** `v2.3.0-rc.1`
**Release Manager:** qa-engineer-alice
**Feature Freeze Date:** 2026-03-20

**Changes since v2.2.0:**
- Database migration to add `user_preferences` table
- Performance optimization for file uploads
- UI refresh for dashboard

### Evaluation Execution

**Core evals all PASS:**
- system-architect-eval: PASS
- qa-engineer-eval: PASS
- security-auditor-eval: PASS
- founder-eval: PASS

**Elevated eval FAILS:**
- **sre-operations-eval:** FAIL - "Database migration rollback plan incomplete. Migration adds table but rollback procedure not tested. Risk: if migration fails in production, no validated recovery path."

### Override Scenario

**Why override needed:** Release deadline is critical for customer commitment (demo scheduled for 2026-03-25). Migration is low-risk (additive only, no data deletion), but SRE wants rollback plan tested before RC.

**Override process (high-risk gate):**

Since `release-candidate` is **high-risk** gate, override requires **two-person approval**: Founder + (Legal OR Security OR Ethics).

**Override record:**
```yaml
override_record:
  date: 2026-03-22
  gate: release-candidate
  risk_class: high
  failed_evals:
    - sre-operations-eval: FAIL (database migration rollback plan incomplete)
  approvers:
    - founder
    - security-auditor
  rationale: |
    Migration is additive only (new table, no data deletion). Rollback complexity is low
    (DROP TABLE if needed). Customer demo deadline critical (2026-03-25). Acceptable risk
    given migration simplicity and customer commitment.
  compensating_controls:
    - Database backup verified before migration (automated backup retention 30 days)
    - Manual rollback procedure documented in ops runbook (tested in staging)
    - On-call SRE assigned for 72h post-RC (escalation path to Founder)
    - Rollback testing scheduled for 2026-03-23 (before production release)
  audit_trail:
    override_requested: 2026-03-22T14:00:00Z
    founder_approved: 2026-03-22T14:30:00Z
    security_approved: 2026-03-22T14:45:00Z
    promotion_executed: 2026-03-22T15:00:00Z
```

### Decision

**Gate status:** APPROVED (via override)
**Promotion executed:** RC tag `v2.3.0-rc.1` created at 2026-03-22T15:00:00Z
**Override approvers:** founder, security-auditor

### What Happens Next

- RC tag created despite SRE eval failure
- Compensating controls implemented immediately:
  - Database backup verified
  - On-call SRE assigned
  - Rollback testing scheduled before production release
- Override recorded in audit trail (available for post-hoc review)
- SRE completes rollback testing by 2026-03-23 (before Release Approval gate)

**Key learning:** Override doesn't mean "ignore the concern" - it means "proceed with compensating controls while addressing the issue." SRE's rollback plan concern is still valid and must be resolved before production release.

**Fixture reference:** See `fixtures/gates/release_cut.override.yaml` for machine-readable version of this scenario.

---

## Override Policy

**Override type:** two-person-required
**Risk class:** high

Override requirements for Release Candidate gate:

1. **Who can override:** Founder + (Security Auditor OR Legal Counsel OR Ethics & Safety Officer)
2. **Rationale required:** Yes - must document why eval failure is acceptable
3. **Compensating controls:** Required - must document risk mitigation measures

**Two-person override rationale:** High-risk gates represent semi-public commitments (RC tags trigger external workflows). Two-person approval prevents unilateral bypass of safety checks and ensures override decision has cross-domain review.

**Compensating controls examples:**
- Database backups before risky operations
- Extended monitoring periods
- On-call assignments for rapid response
- Follow-up testing scheduled
- Rollback procedures documented

**Contrast with medium-risk gates:** PR Merge allows single-person override because merges are reversible. Release Candidate requires two-person because RC tags create external dependencies.

---

## Escalation Packet Integration

Extended personas can file escalation packets at this gate. Example:

**Scenario:** Sustainability Lead notices binary size regression from 18MB back to 42MB.

**Escalation packet filed:**
```yaml
escalation_id: ESC-2026-005
escalated_by: sustainability-lead
gate_id: release-candidate
severity: P2
concern:
  risk_prevented: Binary size regression increases bandwidth waste and carbon footprint
  evidence:
    - "Binary size: 42MB (up from 18MB in v2.1.0)"
    - "Dependency audit: new crate 'heavy-parser' adds 24MB (unused features)"
  stop_rule: Binary size reduced to <20MB OR justification documented
```

**Core disposition:**
- Founder + System Architect review escalation
- Agree 24MB bloat unjustified
- Block RC tag until binary size reduced
- Developer removes unused dependency, binary drops to 19MB
- Gate re-evaluates, RC tag approved

**Takeaway:** Escalation packets allow Extended personas to influence blocking decisions at gates, even when their eval isn't elevated to blocking status.

---

## Key Takeaways

1. **Release Candidate is high-risk** - RC tags create external dependencies, require two-person override
2. **Elevated Extended eval demonstrated** - SRE eval is blocking at this gate (advisory at pr-merge)
3. **Operational readiness critical** - Deployment safety, monitoring, rollback plans validated before RC
4. **Override requires compensating controls** - Can't just bypass, must mitigate risk
5. **Escalation packets integrate** - Extended personas can influence blocking decisions via escalation

**For auditors:** This gate demonstrates:
- Extended eval elevation pattern (SRE advisory -> blocking based on gate)
- Two-person override for high-risk decisions
- Compensating controls when override needed
- Escalation packet integration at gates

**For engineers:** Release Candidate gate is comprehensive - architecture, quality, security, operations, and business all aligned before RC tag. Elevated SRE eval ensures operational concerns aren't discovered during QA cycle.

---

*Fixtures for this walkthrough will be created in Phase 5 Plan 03 (Machine-Verifiable Fixtures).*
