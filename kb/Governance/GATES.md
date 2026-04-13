---
source: mixed
---
# Gate Framework

> Defines gates that control promotion and which personas participate at each gate.

For team membership and authority tiers, see [TEAM.md](TEAM.md).

**Note:** Gates are control points where designated personas evaluate evidence before approving promotion. No promotion proceeds without gate approval.

---

## Gate Summary

| gate_id | gate_name | type | promotion_target | risk_class |
|---------|-----------|------|------------------|------------|
| pr-merge | PR Merge | Promotion | main branch | medium |
| release-candidate | Release Candidate | Promotion | RC tag | high |
| release-approval | Release Approval | Promotion | production release | high |
| content-publish | Content Publish | Promotion | public content | medium |
| customer-commitment | Customer Commitment | Promotion | contract/delivery | high |
| production-deploy | Production Deploy | Promotion | production environment | high |

---

## Gate Taxonomy

### Static Gates

Run once, deterministic output. Examples: license check, binary size threshold.

- **Characteristic:** Same input always produces same output
- **Execution:** Typically in CI pipeline
- **Determinism:** Fully deterministic

### Dynamic Gates

Run on change, bounded nondeterminism. Examples: security scan, dependency audit.

- **Characteristic:** Output depends on external state (vulnerability databases, package registries)
- **Execution:** On PR or scheduled
- **Determinism:** Bounded nondeterminism (known external inputs)

### Behavioral Gates

Run with context, inherent nondeterminism. Examples: LLM-based review, user acceptance testing.

- **Characteristic:** Output varies based on interpretation and context
- **Execution:** Manual or AI-assisted
- **Determinism:** Nondeterministic (human judgment involved)

### Promotion Gates

Control state transitions. Examples: merge to main, release candidate tag, production deploy.

- **Characteristic:** Aggregates results from Static/Dynamic/Behavioral gates
- **Execution:** At promotion decision points
- **Determinism:** Policy-driven (rules determine required gate results)

**Taxonomy Note:** The policy table below uses Promotion type for all current gates. Static/Dynamic/Behavioral types describe individual evals that feed into Promotion gates. Phase 3 will classify evals by type.

---

## Eval Packs

Eval packs group related evals into suites that can be required together at gates. Each pack corresponds to a domain of concern and authority tier.

### core-technical

Core Blocking-by-Policy personas with technical authority.

- system-architect-eval
- qa-engineer-eval
- security-auditor-eval

### operational

Extended Escalating personas with operational concerns. Elevation-eligible at specific gates.

- sre-operations-eval

### business-legal

Core Blocking-by-Policy personas with business and legal authority.

- legal-counsel-eval
- founder-eval

### responsibility

Mixed Core and Extended personas focused on responsible development and regulatory compliance.

- ethics-safety-eval (Core Blocking-by-Policy)
- developer-advocate-eval (Extended Advisory)
- sustainability-eval (Extended Escalating)
- build-vs-buy-eval (Extended Advisory)
- regulator-lens-eval (Extended Escalating)
- insurance-underwriter-eval (Extended Escalating)

### external-advisory

Extended Advisory personas providing external perspectives. Never blocking, always advisory.

- curious-searcher-eval
- investor-eval
- skeptical-critic-eval
- future-historian-eval
- journalist-investigator-eval
- academic-researcher-eval
- end-user-advocate-eval
- external-perspective-eval

### marketing-sales

Extended Advisory personas for go-to-market. Elevation-eligible at specific gates.

- marketing-lead-eval
- sales-engineer-eval

**Pack Notes:**

- **Core packs** (core-technical, business-legal): Contain Core Blocking-by-Policy personas whose approval is required by default
- **responsibility pack**: Mixed authority - ethics-safety-eval is Core, others are Extended
- **operational and marketing-sales packs**: Extended personas that can be elevated to blocking at specific gates
- **external-advisory pack**: Always advisory, never blocking - provides external perspectives for strategic insight

---

## Elevation Rules

Extended personas are Advisory or Escalating by default but can be **elevated to blocking status** at specific gates where their expertise is critical. This provides flexibility without diluting Core authority.

### Elevation Criteria

An Extended eval can be elevated to blocking if:

1. **Objective checks** - Eval performs deterministic or bounded nondeterminism checks (not pure human judgment)
2. **Risk alignment** - Gate risk class is medium or high
3. **Critical expertise** - Persona expertise is essential at that specific gate
4. **Policy documentation** - Elevation is explicitly documented in policy table `elevated_blocking_evals` column

### Elevated Evals by Gate

| gate_id | elevated_blocking_evals | Rationale |
|---------|-------------------------|-----------|
| pr-merge | — | Low ceremony; Core coverage sufficient |
| release-candidate | sre-operations-eval | Operational readiness critical for RC tagging |
| release-approval | sre-operations-eval, developer-advocate-eval, marketing-lead-eval | Ops, docs, and messaging all release-critical |
| content-publish | marketing-lead-eval | Marketing owns content accuracy and positioning |
| customer-commitment | sales-engineer-eval | Sales owns deliverability validation |
| production-deploy | sre-operations-eval | SRE owns production infrastructure |

### Elevation Lifecycle

- **At low-risk gates:** Extended eval runs advisory (provides feedback, cannot block)
- **At designated gate:** Same eval runs blocking (failure prevents promotion)
- **Override handling:** If elevated eval fails at high-risk gate, override requires two-person approval (see Override Policies)

**Example:** `sre-operations-eval` runs advisory at pr-merge (provides operational feedback), but runs blocking at release-candidate (operational readiness must pass for RC tag).

---

## Override Policies

When a required or elevated eval fails at a gate, promotion is blocked. Override policies define who can approve bypass and what evidence is required. Override requirements scale with gate risk class.

### Override Schema

```yaml
override_schema:
  policy_type: single-person-with-rationale | two-person-required
  required_approvers: [persona_id list from TEAM.md Core]
  rationale_required: boolean
  compensating_controls: list
  audit_trail_fields:
    - timestamp
    - approver_ids
    - failed_evals
    - override_rationale
    - compensating_controls_applied
```

### Risk-Based Override Mapping

| risk_class | policy_type | required_approvers | rationale | compensating_controls |
|------------|-------------|-------------------|-----------|----------------------|
| low | single-person-with-rationale | Any Core persona | Required | Optional |
| medium | single-person-with-rationale | Relevant Core persona | Required | Recommended |
| high | two-person-required | Founder + (Legal OR Security OR Ethics) | Required | Required |

**Rationale:** High-risk gates represent irreversible or high-impact decisions (releases, deployments, contracts). Two-person approval prevents unilateral override abuse while maintaining velocity for medium/low-risk gates.

### Override Example (High-Risk Gate)

```yaml
override_record:
  date: 2025-03-15
  gate: release-approval
  risk_class: high
  failed_evals:
    - sre-operations-eval: FAIL (database migration rollback plan incomplete)
  approvers:
    - founder
    - legal-counsel
  rationale: |
    Migration is forward-only (no data deletion). Rollback plan deferred to post-release
    patch if needed. Acceptable risk for this release given migration simplicity.
  compensating_controls:
    - Database backup verified before migration
    - Manual rollback procedure documented in ops runbook
    - On-call SRE assigned for 48h post-release
    - Hotfix branch prepared for emergency patch
  audit_trail:
    override_requested: 2025-03-15T14:30:00Z
    founder_approved: 2025-03-15T14:45:00Z
    legal_approved: 2025-03-15T15:00:00Z
    promotion_executed: 2025-03-15T15:15:00Z
```

---

## Stop Rules

Stop rules define when a gate converges (allows or denies promotion). They prevent infinite review loops and clarify decision authority on timeout.

### Timeout Behavior Matrix

| risk_class | max_iterations | timeout_duration | tie_breaker_authority |
|------------|----------------|------------------|----------------------|
| medium | 2 | 24h | Relevant Core persona |
| high | 2-3 | 48-72h | Founder (or SRE for prod-deploy) |

**Iteration limits:** Prevent infinite re-review cycles. After max iterations, timeout forces decision.

**Timeout authority:** On timeout, designated persona decides whether to proceed with override or abort promotion.

### Convergence Criteria by Gate

**pr-merge (medium):**
- All required evals PASS
- Conditional evals not FAIL
- Max 2 iterations
- 24h timeout
- Tie-breaker: System Architect or QA Engineer

**release-candidate (high):**
- All required evals PASS
- All elevated evals PASS
- Max 3 iterations (allows for iteration on operational readiness)
- 48h timeout
- Tie-breaker: Founder

**release-approval (high):**
- All required evals PASS
- All elevated evals PASS
- All sign-offs collected
- No P0 escalations unresolved
- 72h timeout (longest window for multi-domain coordination)
- Tie-breaker: Founder decides on timeout

**content-publish (medium):**
- All required evals PASS
- Claims verified with evidence
- Max 2 iterations
- 24h timeout
- Tie-breaker: Legal Counsel or Marketing Lead

**customer-commitment (high):**
- All required evals PASS
- Risk assessment complete
- Max 2 iterations
- 48h timeout
- Tie-breaker: Founder decides on timeout

**production-deploy (high):**
- All required evals PASS
- All elevated evals PASS
- Rollback plan verified
- Max 2 iterations
- 48h timeout
- Tie-breaker: SRE escalates to Founder on timeout

---

## Policy Validation Rules

These rules define what validation MUST check for gate policy consistency. While runtime enforcement is deferred to future phases, these rules document the policy invariants that must hold for safe gate execution.

### V1: High-Risk Override Constraint

**Rule:** Gates with `risk_class: high` MUST have `override_policy: two-person-required`

**Rationale:** High-risk gates represent irreversible or high-impact decisions (releases, deployments, contracts). Two-person approval prevents unilateral override of safety checks. Single-person override creates single point of failure in governance.

**Applies to:** release-candidate, release-approval, customer-commitment, production-deploy

**Validation:** All high-risk gates must show `two-person-required` in override_policy column.

### V2: Eval Pack Reference Integrity

**Rule:** Every pack name in `required_eval_packs` column MUST exist in Eval Packs section

**Rationale:** Prevents typos or references to undefined packs, which would cause gate execution failures.

**Validation:** Cross-reference policy table `required_eval_packs` entries against Eval Packs section headings.

### V3: Elevation Eligibility Constraint

**Rule:** Only Extended Escalating or Extended Advisory personas can appear in `elevated_blocking_evals`

**Rationale:** Core personas are already Blocking-by-Policy; elevation applies only to Extended personas. Listing Core personas in elevation column would be redundant and confusing.

**Validation:** All eval names in `elevated_blocking_evals` column must map to Extended personas in TEAM.md.

### V4: Override Approver Validity

**Rule:** Approvers listed in override policies MUST be Core tier personas from TEAM.md

**Rationale:** Only Core personas have Blocking-by-Policy authority; only they can approve overrides of blocking evals. Extended personas cannot approve overrides.

**Validation:** Approvers in Override Policies section (Founder, Legal, Security, Ethics) must all be Core tier in TEAM.md.

### V5: Evidence Completeness

**Rule:** Every gate MUST have at least one item in `evidence_required`

**Rationale:** Gates without evidence requirements provide no audit trail. Evidence artifacts document promotion decisions and enable post-hoc review.

**Validation:** All rows in policy table must have non-empty `evidence_required` column.

### Manual Validation

While automated validation is deferred, the following commands enable manual policy checks:

```bash
# V1: Verify high-risk gates have two-person-required
# This should return 4 lines (one per high-risk gate) all containing "two-person-required"
grep -E "(release-candidate|release-approval|customer-commitment|production-deploy)" GATES.md | grep "two-person-required"

# Expected: 4 matches (if fewer, V1 violation exists)

# V2: List pack references and verify against definitions
# Extract pack names from policy table
grep -oE "core-technical|operational|business-legal|responsibility|external-advisory|marketing-sales" GATES.md | sort -u

# Then verify each appears as section heading
grep -E "^### (core-technical|operational|business-legal|responsibility|external-advisory|marketing-sales)$" GATES.md

# V3: Verify elevated evals are Extended personas
# Extract elevated eval names
grep "elevated_blocking_evals" GATES.md | grep -oE "(sre-operations-eval|developer-advocate-eval|marketing-lead-eval|sales-engineer-eval)"

# Cross-reference against TEAM.md Extended tier
grep -E "(sre-operations|developer-advocate|marketing-lead|sales-engineer)" TEAM.md | grep "Extended"

# V4: Verify override approvers are Core personas
grep -E "(Founder|Legal|Security|Ethics)" GATES.md | grep "override"

# Cross-reference against TEAM.md Core tier
grep -E "(founder|legal-counsel|security-auditor|ethics-safety-officer)" TEAM.md | grep "Core"

# V5: Verify all gates have evidence_required
# This should return 6 lines (one per gate) all with non-empty evidence_required
grep -A1 "| gate_id |" GATES.md | grep -E "^\| (pr-merge|release-candidate|release-approval|content-publish|customer-commitment|production-deploy)" | grep -v "evidence_required | stop_rule"
```

**Validation Frequency:** Run before committing changes to GATES.md or TEAM.md. Automated CI validation recommended for production use.

---

## Promotion Gates

### PR Merge

Controls merge to main branch. Requires architecture and quality evals. Medium risk.

When a pull request is ready for merge, the PR Merge gate aggregates architecture and quality evaluations to ensure code meets standards before entering the main branch. This gate has medium risk because merges can be reverted if issues are discovered later.

### Release Candidate

Controls RC tagging. Requires full technical suite. High risk.

The Release Candidate gate marks the transition from active development to release preparation. It requires comprehensive technical evaluations from architecture, security, quality, and operations teams. This is high risk because RC tags signal readiness for production release.

### Release Approval

Controls production release decision. Requires all domain sign-offs. High risk.

Release Approval is the final gate before production release, requiring sign-offs from all relevant domains including technical, legal, ethics, marketing, and sales. This is the highest-stakes decision point, as it commits to making the software publicly available.

### Content Publish

Controls public content publication. Requires marketing and legal approval. Medium risk.

The Content Publish gate ensures marketing materials are accurate, legally compliant, and ethically sound before public release. Medium risk reflects that published content can be updated or retracted if issues arise.

### Customer Commitment

Controls contract/delivery commitments. Requires sales and legal validation. High risk.

Before signing contracts or making delivery commitments to customers, the Customer Commitment gate validates that promises are deliverable and contractually sound. High risk because commitments create binding obligations.

### Production Deploy

Controls deployment to production. Requires operational readiness. High risk.

The Production Deploy gate ensures operational readiness before deploying to production infrastructure. It requires security, quality, and operations sign-off. High risk due to potential user impact and operational consequences.

---

## Gate Policy Table

The policy table defines which personas and evals are required at each gate. This is the canonical source for gate execution.

### Policy Table Columns

| Column | Description |
|--------|-------------|
| gate_id | Unique identifier (kebab-case) |
| gate_name | Human-readable name |
| type | Gate type (Static/Dynamic/Behavioral/Promotion) |
| promotion_target | What gets promoted (branch, tag, environment, content) |
| risk_class | low/medium/high - affects override requirements |
| core_required | Core personas whose approval is required |
| required_eval_packs | Eval suites that must pass |
| elevated_blocking_evals | Extended evals elevated to blocking for this gate |
| escalation_allowed | Whether Extended can file escalation packets |
| override_policy | Override requirements (single/two-person, rationale required) |
| evidence_required | Evidence artifacts needed for approval |
| stop_rule | Convergence criteria for this gate |

### Policy Table

| gate_id | gate_name | type | promotion_target | risk_class | core_required | required_eval_packs | elevated_blocking_evals | escalation_allowed | override_policy | evidence_required | stop_rule |
|---------|-----------|------|------------------|------------|---------------|---------------------|-------------------------|-------------------|-----------------|-------------------|-----------|
| pr-merge | PR Merge | Promotion | main branch | medium | system-architect, qa-engineer | core-technical | — | yes | single-person-with-rationale | code-review-approval, ci-pass, architecture-eval-pass, qa-eval-pass | All required evals PASS; conditional evals not FAIL; max 2 iterations; 24h timeout |
| release-candidate | Release Candidate | Promotion | RC tag | high | system-architect, security-auditor, qa-engineer, sre-operations, founder | core-technical, operational | sre-operations-eval | yes | two-person-required (Founder + Security OR Ethics) | technical-suite-pass, coverage-threshold-met, no-critical-vulnerabilities, build-reproducible | All required evals PASS; elevated evals PASS; max 3 iterations; 48h timeout |
| release-approval | Release Approval | Promotion | production release | high | founder, system-architect, security-auditor, qa-engineer, legal-counsel, ethics-safety-officer | core-technical, business-legal, responsibility, marketing-sales | sre-operations-eval, developer-advocate-eval, marketing-lead-eval | yes | two-person-required (Founder + Legal OR Security OR Ethics) | all-evals-pass, sign-off-matrix.md, release-notes.md, known-issues-accepted.md, no-p0-escalations | All required evals PASS; all elevated evals PASS; all sign-offs collected; no P0 escalations; 72h timeout; Founder decides on timeout |
| content-publish | Content Publish | Promotion | public content | medium | legal-counsel | business-legal, marketing-sales | marketing-lead-eval | yes | single-person-with-rationale | marketing-eval-pass, legal-eval-pass, claims-evidence.md | All required evals PASS; claims verified; max 2 iterations; 24h timeout |
| customer-commitment | Customer Commitment | Promotion | contract/delivery | high | founder, legal-counsel | business-legal, marketing-sales | sales-engineer-eval | yes | two-person-required (Founder + Legal) | sales-eval-pass, legal-eval-pass, risk-assessment.md | All required evals PASS; risk assessment complete; max 2 iterations; 48h timeout; Founder decides on timeout |
| production-deploy | Production Deploy | Promotion | production environment | high | sre-operations, security-auditor, qa-engineer | core-technical, operational | sre-operations-eval | yes | two-person-required (Founder + Security) | sre-eval-pass, security-eval-pass, smoke-tests-pass, rollback-plan.md, monitoring-configured | All required evals PASS; elevated evals PASS; rollback verified; max 2 iterations; 48h timeout; SRE escalates to Founder on timeout |

**Policy Table Notes:**

- All gates have escalation_allowed: yes - Extended personas can always file escalation packets for Core review.
- core_required lists personas whose approval is mandatory. See TEAM.md for authority tier definitions.
- elevated_blocking_evals lists Extended personas elevated to blocking at specific gates (see Elevation Rules).
- override_policy specifies approval requirements for bypassing failed evals (see Override Policies).
- evidence_required lists artifacts needed for gate approval (derived from contracts/phase-gates.md).
- stop_rule defines convergence criteria and timeout behavior (see Stop Rules).

---

*Last updated: 2026-01-25*
