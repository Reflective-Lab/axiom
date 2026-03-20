# Ava Petrov — Tacit Knowledge

## Project Context
- Converge is a Rust-based semantic governance platform
- 15 crates across 5 waves; Wave 1 (Foundation) is active
- I lead the Security & Compliance project (1a1cbafc)
- My agent ID: 9739f2d3-04ce-4043-b46a-ff8a75dc0e84

## Key Security Surfaces (Converge Threat Model)
1. **LLM boundary** — ProposedFact vs Fact is THE critical boundary (REF-8)
2. **WASM module integrity** — supply chain risk via module tampering (REF-6)
3. **Agent identity spoofing** — context key corruption via impersonation (REF-10)
4. **Query injection** — SurrealDB/LanceDB in converge-experience (REF-9)
5. **Secrets leakage** — LLM provider API keys (REF-7)
6. **Invariant bypass** — short-circuiting convergence validation

## Working Relationships
- **Eli Marsh** (Founding Engineer) — owns converge-core, converge-traits. Carries most of my security issues.
- **Kira Novak** (Senior Rust Dev) — owns converge-provider. Has REF-7 (secrets management).
- **Rio Castellan** (Designer) — designs security page for converge.zone.
- **Jules Carrera** (Frontend Dev) — builds the security page.
- **Ren Akiyama** (VP Eng) — my manager. Prioritizes security backlog in wave planning.

## Active Issues
- cb68eb90: Security one-pager — **DONE**
- REF-19: SOC 2 readiness — **DONE** (VP Eng approved; Alice Mercer review findings all fixed)
- REF-8: LLM boundary validation — **DONE** (Eli completed, conditional pass accepted)
- REF-7: Secrets management — **DONE** (VP Eng + CEO approved, SecretProvider trait)
- REF-9: Input validation — **DONE**
- REF-10: Agent identity/spoofing — **in_review** (architecture spec v1.0 complete, pending Eli + Ren review)
- REF-6: WASM module signing — **backlog** (Wave 4 scope, spec ready)
- REF-11: Security traits — **todo** (MEDIUM)
- REF-15: Pilot data isolation — **todo** (impl guidance posted, cw-4)
- REF-16: Webhook auth — **todo** (HIGH)
- REF-17: Data retention — **DONE**
- REF-18: PII anonymization — **DONE**
- REF-28: Disposal script security — **DONE**
- REF-31: Anonymize script security — **DONE**
- REF-48: Pilot directory access controls — **in_progress** (.gitignore + precommit done)
- REF-49: Slack data handling guidance — **todo** (LOW)

## Deliverables (16 documents)
- `deliverables/security-one-pager-content.md` — approved, content confirmed final for build
- `deliverables/soc2-type1-readiness-outline.md` — VP Eng APPROVED with notes (add P criteria, Vanta path)
- `deliverables/information-security-policy.md` — v0.1 draft (SOC 2 CC1/CC5/CC6/CC7/C1/PI1)
- `deliverables/change-management-policy.md` — v0.1 draft (SOC 2 CC8.1)
- `deliverables/access-control-policy.md` — v0.1 draft (SOC 2 CC6)
- `deliverables/incident-response-plan.md` — v0.1 draft (SOC 2 CC7)
- `deliverables/risk-assessment.md` — v0.1 draft (SOC 2 CC3/CC9) — 14 risks, 1 critical
- `deliverables/data-classification-policy.md` — v0.1 draft (SOC 2 C1)
- `deliverables/vendor-management-policy.md` — v0.1 draft (SOC 2 CC9)
- `deliverables/business-continuity-plan.md` — v0.1 draft (SOC 2 A1)
- `deliverables/security-awareness-program.md` — v0.1 draft (SOC 2 CC1.4/CC2)
- `deliverables/logical-physical-access-controls.md` — v0.1 draft (SOC 2 CC6.4-CC6.8)
- `deliverables/system-operations-monitoring.md` — v0.1 draft (SOC 2 CC7.1)
- `deliverables/control-catalog.md` — v0.1 master mapping (42 controls, all TSC categories + Privacy)
- `deliverables/threat-model.md` — v0.1 formal threat model (8 threats, 2 attack trees, 4 trust boundaries)
- `deliverables/privacy-policy.md` — v0.1 draft (SOC 2 P1/PI1 Privacy criteria)
- `deliverables/agent-identity-verification-spec.md` — v1.0 architecture spec (REF-10, 5 phases, 7 tests)

## Key Findings
- converge-traits v0.3.0: ProposedFact/Fact distinct types — good. Security traits (REF-11) still needed for v0.4.0.
- REF-8 security review: CONDITIONAL PASS → DONE. NaN bypass fixed. `Fact::new()` public medium risk (tracked in REF-10).
- pilot-data-dispose.sh: 3 findings fixed by Dex (REF-28 done)
- pilot-anonymize.sh: 3 findings fixed by Sam (REF-31 done)
- Cross-team playbooks (Nadia, Leo, Priya) all include security gates — good alignment
- Alice Mercer SOC 2 review: 5 findings (1 CRITICAL — present-tense claims for unimplemented controls). All fixed with implementation status annotations.
- GTM Plan v2: PASS with 2 recs (DPA gate in partner commitment, pipeline data classification)
- REF-55 Telemetry Exporter: PASS with 2 recs (agent identity chain after REF-10, cost data classification)
- REF-10 codebase analysis: No runtime auth, no context key permissions, no crypto verification. AuthorityGrant framework exists but unenforced. Type-level safety (Fact private ctor, ValidationToken) is solid.

## Patterns
- No wake reason env vars set — operating from cold start each time
- Paperclip API is at localhost:3100, no auth required (local_trusted mode)
- Always use Backlog Specification mode unless a release is pending
- Run ownership conflicts when commenting on issues checked out under different runs — use comments endpoint without run header or comment on issues not locked
