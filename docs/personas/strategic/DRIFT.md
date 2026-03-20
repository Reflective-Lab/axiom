# Drift Taxonomy

Drift is governance entropy that compounds. It occurs when documented standards, validated patterns, and operational reality gradually diverge. Left unmonitored, drift accumulates until governance mechanisms lose authority.

This taxonomy defines drift as a classifiable phenomenon with stable tracking codes. It enables systematic monitoring of governance health across strategy gates.

**Relationship to Strategy Gates:**

Strategy gates (STRATEGY_GATES.md) govern high-consequence decisions through validation checks. Drift monitoring tracks when those governance patterns degrade over time. Gates prevent individual bad decisions; drift detection prevents systematic erosion.

---

## 1. Drift Codes

Stable identifiers for drift tracking. Format: D_[CLASS].

| Code | Class | Determinism | v4.1 Alerting | Primary Lens |
|------|-------|-------------|---------------|--------------|
| D_SPEC | Spec drift | Class A | Threshold-triggered | Thaler |
| D_FIX | Fixture drift | Class B | Threshold-triggered | Hemingway |
| D_NARR | Narrative drift | Class C | Advisory-only | Marks |
| D_AUTH | Authority drift | Class A | Threshold-triggered | Ostrom |
| D_OPS | Operational drift | Class B | Threshold-triggered | Vogels |

### Code Stability

**Drift codes are stable identifiers.** They are never versioned.

- **Detection rules are versioned, not codes.** Rules evolve (v1, v2, v3); codes remain constant.
- **Codes enable trend tracking.** Weekly digest shows "D_AUTH incidents: 3 this week, 1 last week, 0 prior 4 weeks."
- **Codes enable tooling integration.** External systems can reference D_SPEC without breaking on taxonomy updates.

**If a drift class definition fundamentally changes:**

- Deprecate the old code (mark as deprecated in this document)
- Introduce a new code with new semantics
- Document the change in the changelog section
- Never reuse deprecated codes for new meanings

**Example:** If "spec drift" fundamentally changes to mean something different, deprecate D_SPEC and introduce D_SCHEMA. Do not create D_SPEC_V2.

---

## Determinism Classes

Drift detection inherits determinism classification from strategy gates (STRATEGY_GATES.md). Detection methods fall into three classes:

| Class | Description | Can Trigger Alerts? | Replayable? |
|-------|-------------|---------------------|-------------|
| Class A | Rule-based, pattern matching | Yes (threshold-triggered) | Yes |
| Class B | Rubric-based with bounded output | Yes (threshold-triggered) | Partially |
| Class C | Human judgment required | No (advisory-only) | No |

**Class A checks** are deterministic and replayable. Examples: banned phrase detection, schema validation failure, cross-reference verification. These can produce STOP signals.

**Class B checks** use rubrics or thresholds with bounded output space. Examples: trend analysis, threshold comparison, coverage percentage. These can produce WARN signals.

**Class C checks** require human judgment and open-ended assessment. Example: narrative-reality gap assessment, strategic intuition. These produce advisory notes only, never automated alerts.

**v4.1 policy:** Only Class A and Class B drift types trigger automated alerting. Class C types are monitored manually.

---

## v4.1 Alerting Semantics

Drift classes have different alerting behaviors in v4.1:

| Alerting Mode | Description | Which Classes |
|---------------|-------------|---------------|
| Threshold-triggered | Automated alert when severity thresholds exceeded | D_SPEC, D_FIX, D_AUTH, D_OPS |
| Advisory-only | Monitored but no automated alerting | D_NARR |

### Threshold-Triggered Alerting

**Behavior:** When severity crosses defined threshold (High/Medium), automated alert generated and included in weekly digest.

**Applies to:** D_SPEC, D_FIX, D_AUTH, D_OPS

**Rationale:** These drift types have deterministic or rubric-based detection (Class A/B). Thresholds are based on operational data or proven patterns from governance research.

### Advisory-Only Monitoring

**Behavior:** Drift tracked manually in quarterly reviews. No automated alerts. Findings documented in digest as contextual information only.

**Applies to:** D_NARR

**Rationale:** Narrative drift requires comparing external claims to internal proofs. Without an established corpus of claim-proof pairs, thresholds would be arbitrary and generate false positives. Advisory-only monitoring allows building the corpus without alert fatigue.

**Future evolution:** When claim-proof corpus reaches sufficient size (≥50 validated examples), v5.0+ may introduce threshold-triggered alerting for D_NARR using PSI-style drift metrics.

---

## 2. Drift Class Definitions

Each drift class has a specific definition, observable manifestations, and severity levels that guide detection and response.

### D_SPEC: Spec Drift

**Definition:** Schemas, fixtures, and documentation express contradictory semantics, or semantics change without propagating updates through the system.

Spec drift occurs when the documented "contract" (schema, type definitions, API specs) falls out of sync with either the test fixtures that validate it or the documentation that explains it. This creates confusion about what the system actually guarantees.

**Manifestations:**

| Manifestation | Detection Method | Severity |
|---------------|------------------|----------|
| Schema-fixture disagreement | Schema validation failure | High |
| Semantic change without fixture update | Version diff analysis, manual review | Medium |
| Documentation lag | Cross-reference check between docs and schema | Low |

**Severity Levels:**

**High:** Schema validation fails, fixtures cannot execute correctly, or fixtures pass validation but encode semantics that contradict the schema's intent. This blocks integrity and requires immediate correction.

**Medium:** Fixtures execute and pass validation, but semantics have shifted without fixture updates. New schema fields added but not tested; enum values expanded but fixtures still use old subset. This compounds over time as gaps accumulate.

**Low:** Documentation references outdated behavior, field names, or validation rules, but code and fixtures are correct. Users may be confused, but system behavior is sound.

**Threshold Rationale:**

- High threshold: Any validation failure is deterministic and immediately actionable
- Medium threshold: ≥3 unaddressed semantic changes within 8-week window signals systematic lag
- Low threshold: Advisory tracking only; elevated to Medium if documentation is primary user interface (API docs, public schemas)

---

### D_FIX: Fixture Drift

**Definition:** Fixtures pass validation but stop reflecting operational reality. The "happy path" and edge cases encoded in fixtures no longer represent actual usage patterns, production scenarios, or real customer workflows.

Fixture drift is insidious because validation succeeds. Tests are green. But the tests validate the wrong thing.

**Manifestations:**

| Manifestation | Detection Method | Severity |
|---------------|------------------|----------|
| Fixture-reality gap | Production log analysis vs fixture coverage | High |
| Coverage decay | New code paths not covered by fixtures | Medium |
| Scenario obsolescence | Customer workflows changed but fixtures unchanged | Medium |

**Severity Levels:**

**High:** Production failures occur in scenarios that fixtures claim to cover. The fixture says "this works" but production says "this fails." Trust in validation is broken.

**Medium:** New operational patterns exist (new customer use cases, new integration points, new error conditions) but fixtures don't cover them. Coverage is decaying but not yet causing visible failures.

**Low:** Fixtures are old but still accurate for what they cover. No production gaps yet, but fixtures feel stale or use deprecated patterns (still valid, just not idiomatic).

**Threshold Rationale:**

- High threshold: Any production failure in "covered" scenario is critical (trust violation)
- Medium threshold: Coverage decay ≥15% (production code paths without fixture coverage) triggers alert
- Low threshold: Advisory tracking; no automated alerting

---

### D_NARR: Narrative Drift

**Definition:** External claims outpace internal proofs. Funding/content language moves faster than documented capabilities, tested scenarios, or operational evidence.

This is Marks territory (claim-proof discipline) intersecting with Thaler territory (defaults and enforcement). Narrative drift occurs when what we SAY we do runs ahead of what we can PROVE we do.

**v4.1 Semantics: Advisory-Only**

D_NARR does NOT trigger automated alerts in v4.1. It is monitored through manual quarterly reviews only.

**Rationale:** Narrative assessment requires comparing external claims (website, pitch decks, sales materials, press) to internal proof artifacts (fixtures, test results, operational metrics, audit logs). Without an established corpus of claim-proof pairs, thresholds would be arbitrary. Automated detection would generate false positives ("marketing language sounds like a claim but isn't a binding commitment") and false negatives ("technically accurate but misleading in context").

**Detection Method (Manual):**

Quarterly review process:

1. Extract claims from external artifacts (content-publish and funding-narrative gate executions)
2. Map claims to proof artifacts (fixtures, operational metrics, architectural constraints)
3. Identify gaps where claims lack corresponding proofs
4. Document gaps in drift report with severity assessment (human judgment)
5. Track gap closure over subsequent quarters

**Future Threshold Guidance (v5.0+):**

When claim-proof corpus ≥50 validated examples with consistent classification, consider threshold-based alerting:

- High: ≥3 unproven claims in public-facing artifacts
- Medium: 1-2 unproven claims OR proven claims with outdated proofs
- Low: Claims lag operational capability (underclaiming, not overclaiming)

**Why defer?** Class C drift types require human judgment. Automating too early creates alert fatigue and degrades trust in the monitoring system.

---

### D_AUTH: Authority Drift

**Definition:** Exception accumulation, "just this once" overrides, authority bypass patterns. Governance rules exist, but exceptions become the norm.

This is Ostrom territory (authority must be explicit and enforced) intersecting with Hemingway territory ("gradual, then sudden" failure). Authority drift is how governance dies: not in one dramatic failure, but through accumulated exceptions that erode the norm.

**Manifestations:**

| Manifestation | Detection Method | Severity |
|---------------|------------------|----------|
| Override accumulation | Override artifact frequency analysis | High |
| Acknowledgment repetition | Same check requiring ack multiple times | Medium |
| Bypass patterns | Gate executions skipped or deferred repeatedly | High |

**Severity Levels:**

**High:** ≥3 overrides in 4-week window, OR any instance of gate bypass (artifact published without gate execution), OR override without documented compensating controls. This signals governance erosion is active and compounding.

**Medium:** 1-2 overrides in 4-week window, OR same check requiring acknowledgment 2+ times for different artifacts within 8-week window. This indicates a systematic pressure point, not an isolated edge case.

**Low:** Single override with documented rationale and compensating controls, OR acknowledgment for check that rarely triggers. Contained exception, monitor for recurrence.

**Threshold Rationale:**

- **3-override window:** Based on Hemingway "gradual then sudden" pattern. Research on governance failure shows third exception is inflection point where "exception" becomes "norm."
- **4-week window:** Matches typical sprint/iteration cycle. Prevents drift from accumulating across iteration boundaries unnoticed.
- **Acknowledgment repetition threshold:** Same check needing ack twice signals systematic issue (timeline pressure, capability gap, misaligned check) rather than legitimate edge case.

**Response Patterns:**

- High severity: STOP new overrides, counter-voice review required, root cause analysis
- Medium severity: Counter-voice consultation before next acknowledgment, trend analysis
- Low severity: Monitor, document in digest, no intervention required

---

### D_OPS: Operational Drift

**Definition:** Manual workarounds, bypass channels, shadow processes. Documented procedures exist but operators route around them because the "official" path is too slow, too rigid, or too disconnected from reality.

This is Vogels territory (operational surface and reliability). Operational drift signals that governance model and operational reality have diverged. The system SAYS deploy through release-approval gate; operators ACTUALLY deploy through "quick patch to prod."

**Manifestations:**

| Manifestation | Detection Method | Severity |
|---------------|------------------|----------|
| Manual override frequency | Audit log analysis, operation frequency tracking | High |
| Undocumented workarounds | Process execution trace vs documented procedures | Medium |
| Process shortcuts | Steps skipped in multi-step procedures | Medium |

**Severity Levels:**

**High:** ≥5 manual overrides or bypass operations in 4-week window, OR critical path (deployment, security update, customer data access) bypassed even once. Operational governance is failing; reliability and auditability are at risk.

**Medium:** 2-4 manual overrides in 4-week window, OR documented workaround exists but isn't integrated into official process. Operators have found the "real" path but it's not captured institutionally.

**Low:** Single manual override with documented rationale, OR workaround exists for rare edge case. Contained operational flexibility, not systematic bypass.

**Threshold Rationale:**

- **5-override window for High:** Higher than D_AUTH because operational flexibility is expected; threshold targets systematic bypass, not occasional judgment
- **Critical path bypass threshold (1 instance = High):** Deployment, security, data access bypass even once is unacceptable; creates audit gap and reliability risk
- **Workaround detection:** Not just "happened once" but "undocumented and repeating" signals institutional capture failure

**Response Patterns:**

- High severity: Operational review, process redesign or enforcement strengthening required
- Medium severity: Document workaround, evaluate for process integration or elimination
- Low severity: Monitor, ensure workaround rationale is clear and reviewable

---

## 3. Classification

When a drift signal is detected, it must be classified into exactly one drift class. This section provides a decision tree for unambiguous classification and documents edge cases.

### Classification Decision Tree

When a drift signal is detected, follow this decision tree:

**1. Does it involve schema/spec/docs disagreement?**

- YES → **D_SPEC**
- NO → Continue to step 2

Examples: Schema validation fails, documentation describes field that doesn't exist, type definitions contradict fixture expectations, semantic change in schema not reflected in fixtures.

**2. Does it involve fixture-reality misalignment?**

- Do fixtures still pass validation?
  - YES → **D_FIX** (fixtures valid but don't reflect reality)
  - NO → **D_SPEC** (go back to step 1 - fixture failure is spec disagreement)
- NO → Continue to step 3

Examples: Production scenario fails but fixtures say it works, customer workflow changed but fixtures unchanged, new integration points not covered by fixtures.

**3. Does it involve external claims vs internal proofs?**

- YES → **D_NARR**
- NO → Continue to step 4

Examples: Website claims capability not yet in fixtures, pitch deck promises feature that's not built, press release overstates operational maturity, sales material contradicts documented limitations.

**4. Does it involve exceptions/overrides/authority bypass?**

- YES → **D_AUTH**
- NO → Continue to step 5

Examples: Gate override used, acknowledgment required for stopped check, gate execution skipped, same check acknowledged multiple times, compensating controls missing for override.

**5. Does it involve operational workarounds/manual bypass?**

- YES → **D_OPS**
- NO → **Unclassified** (review taxonomy)

Examples: Manual deployment outside release gate, undocumented "quick patch" process, operators skip documented procedure steps, shadow process exists alongside official process.

**Unclassified signals:** If a drift signal reaches step 5 and doesn't match D_OPS, it may indicate a gap in the taxonomy. Document the signal, bring to taxonomy review. Do not force-fit into existing class.

---

### Edge Cases

Specific scenarios that may be ambiguous and require explicit classification guidance:

| Scenario | Classification | Rationale |
|----------|----------------|-----------|
| Fixture fails validation AND fixture covers outdated operational reality | **D_SPEC** | Spec is the primary failure mode. Fixture failing validation means schema-fixture disagreement, regardless of whether fixture is also stale. |
| Override is used AND override follows documented process with compensating controls | **Not drift** | Governance working as designed. Overrides are part of the governance model; drift is when overrides accumulate or bypass controls. |
| Narrative claim has proof BUT proof is Class C (human judgment, not deterministic) | **D_NARR** | Proof quality matters. Class C proofs (manual review, subjective assessment) don't meet the bar for "proven capability." Claim is outpacing deterministic proof. |
| Documented override creates operational bypass (override process becomes too slow, operators route around it) | **D_OPS** | Classify by consequence, not origin. The override mechanism itself is D_AUTH territory, but the resulting workaround behavior is operational drift. |
| Same check needs acknowledgment 2+ times within 8-week window | **D_AUTH** | This is authority drift (exception repetition), not operational drift. Systematic pressure on governance check signals authority erosion, even if operational impact is minimal. |
| Fixture covers new scenario but fixture quality is poor (brittle, unclear, not idiomatic) | **Not drift** | Fixture quality issues are technical debt, not governance drift. Drift is about alignment (spec-fixture-reality-claims); quality is about implementation. |
| Documentation lag is extreme (docs are 6+ months outdated) but code/fixtures are correct | **D_SPEC (Low severity)** | Documentation is part of spec. Even if code is correct, extreme doc lag is Low severity D_SPEC. Elevated to Medium if docs are primary user interface. |

### Cross-Gate Drift Patterns

**Definition:** Drift accumulating across multiple gates (e.g., exception in customer-commitment gate + override in architecture-promotion gate for related decisions).

**v4.1 handling:** Cross-gate patterns are NOT a separate drift class. They are detected and reported by the drift detector (Phase 13) as aggregated findings. The weekly digest may highlight cross-gate patterns, but individual findings are still classified into D_SPEC, D_FIX, D_NARR, D_AUTH, or D_OPS.

**Example:** Customer commitment promises feature X with timeline Y. Architecture-promotion gate is overridden to rush X. Roadmap-promotion gate acknowledges that X doesn't compound. This is not "D_CROSS" drift; it's:

- 1x D_AUTH (architecture override)
- 1x D_AUTH (customer commitment acknowledgment of timeline pressure)
- Cross-gate pattern noted in digest as systemic pressure

**Rationale:** Cross-gate patterns are important for root cause analysis and organizational learning, but adding a sixth drift class increases cognitive load and classification ambiguity. Better to surface cross-gate patterns through aggregation and visualization.

---

*Drift Taxonomy v4.1*

*Last updated: 2026-01-27*
