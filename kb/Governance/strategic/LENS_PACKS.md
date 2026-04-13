---
source: mixed
---
# Lens Packs

Lens packs specify which voices run at strategy gates, what they check, and how results are recorded.

---

## 0. Purpose

Lens packs operationalize strategic validation at high-consequence gates. A lens pack is a reusable bundle of voice checks with:
- explicit determinism class (A/B/C)
- explicit severities (PASS/WARN/STOP)
- explicit required artifacts
- explicit counter-voice obligations for WARN/STOP

**Lens packs are advisory-first. STOP is allowed only for deterministic (Class A) violations.**

---

## 1. Strategy Check Classes (A/B/C)

### Class A — Deterministic

Rule-based and replayable (lint, pattern matching, policy cross-check).
- May emit: PASS/WARN/STOP
- **STOP permitted**

### Class B — Bounded

Rubric-based critique with bounded output space (LLM rubric scoring).
- May emit: PASS/WARN
- **STOP forbidden**

### Class C — Nondeterministic

Human judgment or open-ended assessment.
- May emit: NOTE only (treated as advisory)
- **PASS/WARN/STOP forbidden**

**Principle:** We elevate certainty, not confidence.

---

## 2. Output Record Format

Every gate run emits `lens_results[]` records.

```yaml
lens_results:
  - check_id: <voice>.<check_name>.v1
    voice: <voice>
    class: A|B|C
    severity: PASS|WARN|STOP|NOTE
    summary: <one line>
    evidence_refs: [<ids-or-links>]
    remediation: <optional>
```

---

## 3. WARN Acknowledgment Format

Any WARN outcome requires acknowledgment.

```yaml
acknowledgment:
  gate_id: <gate>
  artifact_ref: <id/link>
  warnings_accepted:
    - check_id: <voice.check>
      rationale: <why acceptable now>
      mitigation: <how bounded>
  counter_voice_consulted:
    voice: <counter-voice>
    conclusion: <one line>
  owner:
    name: <person>
    role: <role>
  timestamp: <iso8601>
```

**Constraints:**
- A WARN cannot be acknowledged without a mitigation or scope bound.
- If counter-voice is missing, WARN is treated as STOP until supplied.

---

## 4. STOP Override Format

STOP may be overridden per STRATEGY_GATES.md semantics (only for Class A STOP).

```yaml
override:
  gate_id: <gate>
  stop_checks:
    - <check_id>
  justification: <why override is necessary>
  scope: <bounded scope>
  expiry: <date or review trigger>
  signatures:
    - name: <gate-owner>
      role: <role>
    - name: <counter-voice-owner>
      role: <role>
```

---

## 5. Lens Packs (v3.1)

### Pack: content-publish

**Goal:** Prevent overclaiming and phase mismatch in public messaging

| Role | Voices |
|------|--------|
| Primary | Christensen, Thaler |
| Secondary | Mollick, Nadella |
| Counter | Altman |

#### Required Artifacts
- Content diff or draft text
- Claim inventory (list of explicit claims)
- Evidence map for claims (links, internal proof, policy refs)

#### Checks

**Christensen**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `christensen.job_clarity.v1` | B | WARN | Flags abstract "platform" language without job-level wedge clarity |
| `christensen.wedge_first.v1` | B | WARN | Flags universal positioning with no adoption path |

**Thaler**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `thaler.banned_guarantees.v1` | A | STOP | Detects system-level guarantees and unbounded autonomy phrasing |
| `thaler.friction_language.v1` | B | WARN | Flags messaging that requires insider explanation to be believed |

**Mollick**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `mollick.adoption_realism.v1` | B | WARN | Flags "overnight transformation" implied without transition mechanics |

**Nadella**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `nadella.enterprise_legibility.v1` | B | WARN | Flags missing integration framing, unclear operational surface |

**Altman (counter)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `altman.ambition_fit.v1` | B | NOTE | Ensures vision is present but bounded; never blocks |

#### Pack-Level Conditions

**STOP conditions (deterministic only):**
- Any `thaler.banned_guarantees.v1` STOP
- Any claim in inventory missing evidence mapping

**WARN conditions:**
- Any B/WARN triggers acknowledgment with Altman counter-voice consultation noted

---

### Pack: customer-commitment

**Goal:** Prevent authority leakage and exception drift

| Role | Voices |
|------|--------|
| Primary | Ostrom, Hemingway, SELL |
| Secondary | Christensen, Thaler |
| Counter | Vogels |

#### Required Artifacts
- Commitment artifact (email excerpt, SOW/MSA excerpt, deal note)
- Promise inventory (explicit + implied promises)
- Authority inventory (what the system is allowed to do)

#### Checks

**Ostrom**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `ostrom.authority_concession_scan.v1` | A | STOP | Detects commitments that bypass gates, overrides, audit, or invariants |
| `ostrom.local_governance_fit.v1` | B | WARN | Flags commitments that ignore local institutional constraints |

**Hemingway**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `hemingway.exception_phrase_detect.v1` | A | WARN | Detects "just for this customer", "hard-code", "quick patch" |
| `hemingway.gradual_drift_risk.v1` | B | WARN | Flags repeated small concessions that accumulate |

**SELL (Benioff)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `sell.buyability_check.v1` | B | WARN | Flags commitments that increase buying friction |
| `sell.deal_scope_integrity.v1` | B | WARN | Flags scope creep hidden inside "standard" language |

**Thaler**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `thaler.system_level_guarantees_in_terms.v1` | A | STOP | Detects "guarantee compliance/security/zero downtime" in legal terms |

**Vogels (counter)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `vogels.operational_surface.v1` | B | NOTE | Highlights ops risk; required to consult on WARN acknowledgments |

#### Pack-Level Conditions

**STOP conditions:**
- Any authority concession STOP
- Any unbounded guarantee STOP

**WARN conditions:**
- Any exception language without exception artifact ref

---

### Pack: roadmap-promotion

**Goal:** Prevent non-compounding commitments and adoption-chain failures

| Role | Voices |
|------|--------|
| Primary | Aghion, Mokyr, Howitt |
| Secondary | Christensen, Adner |
| Counter | SELL |

#### Required Artifacts
- Job statement
- Reuse statement (compounding story)
- Dependency chain
- Success metric (non-ARR allowed)

#### Checks

**Aghion**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `aghion.compounding_path.v1` | B | WARN | Flags feature with no reuse flywheel or learning loop |

**Mokyr**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `mokyr.institutionalization.v1` | B | WARN | Flags knowledge not captured into reusable artifacts |

**Howitt**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `howitt.diffusion_loop.v1` | B | WARN | Flags no mechanism for feedback → validation → reinvestment |

**Christensen**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `christensen.job_statement_required.v1` | A | STOP | Missing job statement is deterministic failure |
| `christensen.non_consumption_fit.v1` | B | WARN | Flags solving problems incumbents already solve well |

**Adner**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `adner.adoption_chain.v1` | B | WARN | Flags missing sequencing of complements and stakeholders |

**SELL (counter)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `sell.time_to_revenue.v1` | B | NOTE | Highlights near-term revenue implications |

#### Pack-Level Conditions

**STOP conditions:**
- Missing job statement
- Missing reuse statement (if policy requires it)

---

### Pack: architecture-promotion

**Goal:** Prevent governance drift, hidden coupling, scale paralysis

| Role | Voices |
|------|--------|
| Primary | Vogels, Thaler, Ostrom |
| Secondary | Howitt |
| Counter | Christensen |

#### Required Artifacts
- Architecture change summary
- Affected invariants + gates
- Rollback plan
- Replayability/audit impact statement

#### Checks

**Vogels**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `vogels.reliability_surface.v1` | B | WARN | Flags new failure domains without isolation plan |
| `vogels.operational_simplicity.v1` | B | WARN | Flags added complexity without measurable risk reduction |

**Thaler**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `thaler.default_correctness.v1` | A | STOP | Any change that makes incorrect behavior easier than correct |
| `thaler.enforcement_not_documentation.v1` | B | WARN | Flags reliance on "read the doc" vs architectural enforcement |

**Ostrom**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `ostrom.authority_path_explicit.v1` | A | STOP | Missing explicit authority paths, unclear ownership |

**Howitt**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `howitt.feedback_reinvestment.v1` | B | WARN | Flags no instrumentation loop for learning |

**Christensen (counter)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `christensen.job_pressure_check.v1` | B | NOTE | Flags abstraction without active job pressure |

#### Pack-Level Conditions

**STOP conditions:**
- Any regression in audit trace or replayability
- Any new authority path not explicit

---

### Pack: funding-narrative

**Goal:** Prevent narrative vs reality gap, capital-driven distortion

| Role | Voices |
|------|--------|
| Primary | Marks, Altman |
| Secondary | Lessin, Nadella |
| Counter | Ostrom |

#### Required Artifacts
- Narrative artifact (deck/memo)
- Claim-to-proof map
- Use of proceeds
- Risk register

#### Checks

**Marks**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `marks.claim_proof_map_required.v1` | A | STOP | Missing proof mapping for key capability claims |
| `marks.cycle_risk_language.v1` | B | WARN | Flags "inevitable" narrative without downside articulation |

**Altman**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `altman.ambition_alignment.v1` | B | WARN | Flags vision too small or incoherent relative to thesis |

**Lessin**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `lessin.moat_economics.v1` | B | WARN | Flags no structural margin story, no lock-in mechanism |

**Nadella**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `nadella.platform_adoption.v1` | B | WARN | Flags enterprise adoption story missing |

**Ostrom (counter)**

| Check ID | Class | Severity | Description |
|----------|-------|----------|-------------|
| `ostrom.governance_autonomy.v1` | B | NOTE | Ensures capital story does not imply governance concessions |

#### Pack-Level Conditions

**STOP conditions:**
- Missing claim-to-proof map
- Any banned guarantee language

---

## 6. Minimal Fixture Shapes

### Strategy Gate Execution

**Path:** `fixtures/strategy_gates/<gate_id>.<scenario>.yaml`

```yaml
fixture_id: <gate_id>.<scenario>
gate_id: <gate_id>
lens_pack_version: v3.1
artifact:
  type: <website|contract|roadmap|architecture|deck>
  ref: <id/link>
  summary: <one line>
lens_results: []
decision:
  outcome: PASS|WARN|STOP
acknowledgment_ref: <optional>   # required if outcome == WARN
override_ref: <optional>         # required if outcome == STOP and overridden
```

### Acknowledgment

**Path:** `fixtures/strategy_gates/acks/<gate_id>.<scenario>.yaml`

Uses the acknowledgment format defined in Section 3.

### Override

**Path:** `fixtures/strategy_gates/overrides/<gate_id>.<scenario>.yaml`

Uses the override format defined in Section 4.

---

## 7. Validation Hooks

### Structural Validation

- `class` in {A, B, C}
- `severity` in {PASS, WARN, STOP, NOTE}
- STOP implies `class == A`
- WARN implies `acknowledgment_ref` exists

### Logical Validation

- All primary voice checks present for the gate
- Banned patterns produce deterministic checks
- Counter-voice consulted for WARN acknowledgments
- Override signatures satisfy gate risk level semantics

---

*Lens Packs v3.1*
*Last updated: 2026-01-26*
