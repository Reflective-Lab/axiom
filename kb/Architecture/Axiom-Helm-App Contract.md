---
tags: [architecture, helm, apps, contract]
source: codex
---

# Axiom-Helm-App Contract

This page defines how Axiom, Helm, and applications should play together when
apps become thinner Helm-operated experiences rather than bespoke workflow
islands.

The goal is not to make Axiom run the app. The goal is to make the app's
important jobs expressible as Truth Packages that Helm can show, operate,
review, and replay while Organism, Mosaic, and Converge keep their runtime
responsibilities.

## Product Posture

The long-term product shape is:

```text
App domain event or operator intent
  -> Helm captures or selects a JTBD
  -> Axiom decodes a Truth Package
  -> Helm presents the package, generated .truths, obligations, and priors
  -> Organism selects and runs Formations
  -> Mosaic supplies concrete capabilities and suggestors
  -> Converge reaches a fixed point or an honest stop reason
  -> App adapter emits an AxiomRunObservation
  -> Axiom verifies the run and emits an AxiomRunReport
  -> Helm shows the verdict and reviews calibration reinforcements/concerns
```

Apps should keep domain ownership. Helm should absorb operator/control-plane
repetition. Axiom should absorb truth, lineage, verifier, and decoder-learning
repetition.

## Ownership

| Layer | Owns | Produces | Must not own |
|---|---|---|---|
| Axiom | JTBD decoding, Truth Package schema, generated `.truths`, verifier spec, proof obligations, lineage, reports, and decoder calibration priors. | `TruthPackage`, `IntentPacket`, WASM artifacts and manifests, `AxiomRunReport`, `LearningEpisode`, `CalibrationRecord`, `CalibrationTable`, `CalibrationSuggestion` and `CalibrationConcern` artifacts. | Operator UX, app domain state, formation selection, authority recompute, plugin hosting, specialist execution, raw run history. |
| Helm | Operator surface, package review, truth projection review, calibration review, plugin sandbox lifecycle, run-history display, and app-facing orchestration screens. | Reviewed package versions, accepted/rejected/reset calibration decisions, sandbox execution events, operator audit trails. | Truth semantics, source JTBD mutation, Converge promotion authority, Organism formation strategy, app business state. |
| App | Domain model, domain state transitions, user workflows, domain adapters, custody/payment/external integrations, and app-specific transcript shape. | Domain events, app transcripts, evidence handles, adapter output as `AxiomRunObservation`. | Generic truth review, generic calibration review, generic verifier logic, generic operator-control flows. |
| Organism | Admission, problem classification, Formation selection, planning, runtime dynamics, and learning from Formation outcomes. | Formation plans, stage records, runtime traces, selected work formations. | Truth parsing, authority recompute, app domain persistence. |
| Mosaic | Concrete providers, retrieval, analytics, algorithms, adapters, and fact-emitting suggestors. | Capability-backed suggestors, provider outputs, evidence candidates. | Truth semantics, Helm operator policy, Converge promotion decisions. |
| Converge | Fixed-point loop, fact promotion, authority recompute, stop reasons, evidence refs, trace links, and integrity proof. | Promoted facts, stop reason, integrity proof, promotion authority evidence. | App plugin hosting, Axiom decoding, Helm operator UX. |

## Three Loops

### Authoring Loop

```text
operator/app intent
  -> JTBD
  -> Axiom decode_jtbd(...)
  -> Truth Package
  -> Helm review of package, .truths projection, obligations, and calibration suggestions
  -> frozen package version for runtime
```

Axiom owns deterministic package construction. Helm owns the review surface.
The app should not hand-roll a second package review model.

### Runtime Loop

```text
frozen package version
  -> IntentPacket admitted by Organism
  -> Formation run with Mosaic-backed suggestors
  -> Converge promotes facts or stops honestly
  -> app/domain adapter maps run output to AxiomRunObservation
  -> AxiomRunReport verifies satisfaction, blockage, exhaustion, or invalidity
```

Axiom verifies after the run. It does not select the Formation or decide
promotion authority.

### Learning Loop

```text
AxiomRunReport + lineage audit
  -> LearningEpisode
  -> CalibrationRecord(s)
  -> Helm operator review
  -> accepted CalibrationTable
  -> future Truth Packages enriched by reviewed reinforcements and concerns
```

Axiom persists distilled decoder priors. Raw transcripts and operational
history remain downstream in Helm, the app, or ExperienceStore.

v0.15 splits accepted calibration into two operator-visible signals:

- `Reinforcement` records become `CalibrationSuggestion` artifacts: the
  decoder should keep reaching for evidence, failure, policy, or verifier
  templates that have worked before.
- `Concern` records become `CalibrationConcern` artifacts: the decoder should
  surface prompts, warnings, default evidence expectations, or alternate
  scaffolding for clause shapes that repeatedly go uncovered.

Both signals are decoder-only. Neither signal weakens the source JTBD,
changes verifier requirements, selects a Formation, or grants promotion
authority.

## Helm Operator Control And Ledger

Helm's post-EPIC direction makes the boundary sharper: Helm becomes the
obvious Operator Control surface for long-running jobs, human-in-the-loop
steps, and shared operational audit. It will have access to a shared
append-only ledger.

That ledger should be treated as a control-plane journal, not as a new truth
or authority engine.

Ledger-shaped records should include:

- package review decisions;
- calibration accept/reject/reset decisions;
- HITL prompts, responses, and operator decisions;
- long-running job lifecycle events;
- adapter execution receipts, including `ObservationAdapterReceipt`;
- links to raw app transcripts, Converge runs, and Axiom reports;
- replay hashes and deterministic ids needed to prove a view was rebuilt from
  the same inputs.

The ledger must not own:

- app domain state mutation;
- source JTBD changes;
- generated `.truths` mutation;
- Converge fact promotion;
- Organism Formation selection;
- Mosaic provider execution;
- Axiom verifier verdict semantics.

This reinforces the design rule for Axiom artifacts: anything Helm may place
on the ledger should be deterministic, serializable, and backlink-oriented.
Axiom should store ids, refs, hashes, lineage, reports, suggestions, concerns,
and adapter receipts. Raw app transcripts stay in the app, Helm trace storage,
or downstream stores.

Current fixture proof: `tests/escrow_release_marquee.rs` journals two local
Helm ledger entries for a release review, one for the
`ObservationAdapterReceipt` and one for the `ReleaseReadinessPacket`.
`tests/warden_compliance_marquee.rs` expands the pattern to four entries:
adapter receipt, job readiness packet, compliance approval receipt, and
registry publication receipt. `tests/triage_keeper_marquee.rs` expands it
again to five entries: adapter receipt, job readiness packet, operator
decision receipt, client approval receipt, and maintenance plan receipt.
`tests/inkling_notes_marquee.rs` adds a private-corpus variant: adapter
receipt, job readiness packet, vault snapshot receipt, permission receipt, and
vault index receipt. `tests/plumb_execution_marquee.rs` adds a closed-loop
execution variant: adapter receipt, job readiness packet, drift verdict
receipt, revision proposal receipt, sponsor approval receipt, and strategy
commit receipt. `tests/catalyst_biz_marquee.rs` adds an everyday business
operation variant: adapter receipt, job readiness packet, HITL approval
receipt, routing decision receipt, next action receipt, and outcome tracking
receipt. `tests/fathom_narrative_marquee.rs` adds a temporal-evidence
variant: adapter receipt, job readiness packet, corpus snapshot receipt,
evidence window receipt, disagreement receipt, analyst review receipt, and
narrative claim receipt. All entries are deterministic, carry payload hashes
and backlinks, and explicitly record no authority effect. The serialized
ledger entries do not contain raw signatures, custody refs, rule owner emails,
rule citations, document ids, package names, affected services, ticket refs,
note paths, import source paths, strategy sponsor ids, source systems,
rejected proposal text, business operator ids, owner ids, CRM consent refs,
filing ids, company names, source section refs, source commands, or local app
paths.

## Tally As Boundary-Finding Loop

Tally should stay the next proving ground because escrow release is crisp:
once funds leave escrow, the system cannot treat a bad outcome as harmless.

Use each Tally iteration to classify repeated code:

| Repeated shape | Destination |
|---|---|
| Evidence clauses, failure modes, verifier expectations, lineage, and verdict computation | Axiom |
| Package display, operator acceptance/rejection/reset, calibration reinforcement/concern review, run verdict display | Helm |
| Escrow agreement state, release transition, custody receipt, payment rail, app transcript | Tally |
| Signing, capability-backed evidence retrieval, analytics, policy suggestors | Organism/Mosaic/Converge through their public contracts |

Tally's current adapter already maps a release transcript into
`AxiomRunObservation`. The next useful iterations should ask:

1. Which Tally release requirements can be expressed as JTBD clauses and
   generated verifier obligations?
2. Which hand-built app controls are really Helm operator controls?
3. Which evidence-mapping code is app-specific adapter logic versus generic
   Axiom report plumbing?
4. Which runtime choices belong to Organism/Mosaic instead of Tally or Axiom?
5. Which accepted calibration priors make future escrow packages more complete
   without weakening the source JTBD?
6. Which uncovered evidence concerns should Helm show before an operator
   attempts another release?

### Boundary Probe: Release Conditions Met

First pressure-tested control:
`release-requires-conditions-met`.

This control currently appears in Tally as:

- a `TruthSpec` constant in `tally-truths`;
- a `TransitionReason::ConditionsMet { satisfied }` domain payload;
- a `tally-kernel::Kernel::apply_transition` guard that rejects release when
  agreement conditions are not fully satisfied;
- a release transcript truth key adapted into Axiom's escrow-release
  observation;
- an Axiom evidence requirement and verifier expectation in the escrow-release
  Truth Package.

The contract split is:

| Layer | Boundary decision |
|---|---|
| Tally app/kernel | Owns the agreement state machine, condition indices, `ConditionsMet` payload, and hard rejection of `Released` when conditions are not satisfied. This is domain law and must stay close to the agreement aggregate. |
| Axiom | Owns the truth expression: release requires satisfied conditions before the job can be judged `Satisfied`; missing condition evidence yields `Invalid` or `Blocked`, can produce uncovered-clause `Concern` records, and must never weaken the source JTBD. |
| Helm | Should show the operator which release-condition evidence is present, missing, disputed, or accepted as a calibration concern. Helm owns the review/decision surface, not the transition law. |
| Organism/Mosaic | May assemble readiness Formations and suggestors to gather condition evidence, counterexamples, policy proofs, anomaly scores, or custody facts before the release attempt. They do not authorize release by themselves. |
| Converge | Promotes readiness facts, evidence refs, stop reasons, and integrity. It remains the authority boundary for promoted facts and policy gates. |

Immediate implication: do not move the release transition guard out of Tally's
kernel. Move repeated *explanation* and *review* work outward:

- Axiom should continue turning this control into verifier obligations,
  lineage, `AxiomRunReport` verdicts, and calibration concerns.
- Helm should eventually render the release-readiness packet: required
  condition evidence, missing evidence, accepted concerns, and the last
  verifier verdict.
- Tally should emit a clean release observation/transcript and keep the
  transition guard strict.

This is the template for future probes: if the logic mutates domain state, it
stays in the app/kernel; if it explains whether a job contract was satisfied,
it moves toward Axiom; if it lets an operator inspect or review the contract,
it moves toward Helm.

### Boundary Probe: Transition Signatures

Second pressure-tested control:
`transition-requires-signature`.

This control currently appears in Tally as:

- a `TruthSpec` constant in `tally-truths`;
- `SigningPolicy` and `PartySignature` domain types in `tally-domain`;
- a `tally-kernel::Kernel::apply_transition` guard that rejects transitions
  when the triggering party has not signed, and rejects obligation-changing
  transitions when `SigningPolicy::is_satisfied_by(...)` fails;
- an `OrganismSigningFlow` bridge in `tally-platform` that builds signing
  requests from principal party identities, validates opaque
  `PartySignature` values returned by Organism, and records
  `OrganismSignatureWitness` audit data;
- `AxiomTransitionRecord` truth keys and signers, including
  `transition-requires-signature`;
- an Axiom escrow-release observation that maps principal signing witnesses to
  buyer-authorization evidence.

The contract split is:

| Layer | Boundary decision |
|---|---|
| Tally app/kernel | Owns party roles, agreement membership, `SigningPolicy`, transition legality, and hard rejection when required signatures are missing. This is domain admission law. |
| Tally platform bridge | Owns the app-specific request/response adapter between Tally parties and Organism signing handles. It can record `OrganismSignatureWitness` values, but it should not become a generic Helm review surface. |
| Organism | Owns identity/signature execution and returns opaque signatures or witnesses through its public surface. Tally records handles; it does not generate raw key material. |
| Axiom | Owns the truth/verifier expression: a release or countersign job requires principal authorization evidence. Missing signatures yield `Invalid` or `Blocked` reports and may produce uncovered authorization `Concern` records; Axiom never verifies private keys or changes `SigningPolicy`. |
| Helm | Should show signer coverage before and after the transition: required signers, present witnesses, missing signatures, delegate-on-behalf markers, accepted authorization concerns, and last verifier verdict. Helm owns review, not signature generation or transition admission. |
| Converge | Promotes signing evidence, trace links, authority evidence, and integrity. It decides whether a signing fact can enter governed context. |

Candidate contract surface:

```text
TransitionSigningEvidence
  agreement_id
  transition from/to/reason
  truth_keys
  required_signers
  present_witnesses
  missing_signers
  signing_policy_satisfied
  promotion_authority
  trace_links
```

Do not implement this as a shared type yet. Treat it as the shape Helm should
eventually render and Axiom should be able to verify from app adapters. If the
same shape appears in Atlas, promote it to an explicit contract.

Immediate implication: do not move `SigningPolicy` out of Tally's domain
model, and do not move signature verification into Axiom. Move repeated
*signer coverage explanation* outward:

- Axiom should verify that required authorization evidence exists in the
  normalized observation and should turn missing signer evidence into
  concerns.
- Helm should render the signer coverage packet and operator review history.
- Tally should keep strict transition admission and emit a clean signing
  transcript/witness set.

### Boundary Probe: Custody Release Receipt

Third pressure-tested control:
custody release receipt.

This control currently appears in Tally as:

- a `CustodyAdapter` trait in `tally-platform` with `deposit`, `release`, and
  `refund` operations for the transitions that cross custody boundaries;
- typed `CustodyReceipt`, `DepositReceipt`, `ReleaseReceipt`, and
  `RefundReceipt` records with adapter name, external reference, timestamp,
  and notes;
- an `AttestationCustodyAdapter` that is registrar-bound, idempotent by
  agreement/operation, and records an off-platform release receipt for domain
  transfers;
- `App::release_with_organism`, which applies the signed `Verified ->
  Released` transition and records the `ReleaseReceipt`;
- an Axiom escrow-release observation that maps the release receipt to the
  `disbursement_recorded` evidence clause.

The contract split is:

| Layer | Boundary decision |
|---|---|
| Tally app | Owns release sequencing around the agreement aggregate: read agreement, validate matching custody adapter, compute payouts, apply the transition, call the custody adapter, and emit the release transcript. |
| Tally platform | Owns asset-class custody adapter implementations and typed receipts. Attestation can be a no-op recorder; money, vault, data, and gated-access custody need stronger side-effect discipline. |
| Axiom | Owns the verifier expression: release satisfaction requires a custody/payment/disbursement receipt or an explicit blocked/invalid reason. Missing receipt evidence can produce `Concern` records; Axiom does not call custody adapters. |
| Helm | Should show receipt provenance: adapter, external ref, timestamp, notes, trace link, present/missing state, accepted concerns, and latest verifier verdict. Helm reviews the evidence; it does not execute the custody rail. |
| Organism/Mosaic | May gather readiness evidence from registrars, payment rails, vaults, or data rooms before release. They can propose evidence, not mutate custody state. |
| Converge | Promotes custody receipt facts, evidence refs, trace links, policy decisions, and integrity. Promotion is the governed fact boundary. |

Candidate contract surface:

```text
CustodyReleaseEvidence
  agreement_id
  custody_kind
  adapter
  external_ref
  receipt_at
  notes
  payouts
  present_or_missing
  promotion_authority
  trace_links
```

Do not implement this as a shared type yet. Treat it as the receipt evidence
shape Helm should render and Axiom should verify from app adapters. Promote it
only after the same shape appears outside Tally.

Immediate implication: do not move custody adapters into Axiom or Helm. Move
repeated *receipt explanation* outward:

- Axiom should verify receipt evidence and turn missing receipts into
  calibration concerns for future packages.
- Helm should render receipt state and operator review history.
- Tally should keep asset-specific custody adapters and strict transcript
  emission.

Operational caution: M1's attestation adapter records an off-platform receipt,
so applying the transition before recording the receipt is acceptable for the
fixture. Real money, vault, credential, and gated-access rails should use a
transactional outbox or equivalent saga boundary before irreversible external
side effects. That is Tally/platform responsibility; Axiom verifies the
reported result afterward.

### Boundary Probe: Release Readiness Packet

Fourth pressure-tested control:
release readiness packet.

This is the first converged Helm-facing read model from the prior probes. It
does not authorize release. It tells an operator whether the release job has
the evidence Axiom will require, which facts Converge has promoted, which
Tally domain controls still block transition, and which accepted concerns
should be addressed before retrying.

Tally's own release-readiness note says dynamic Formations may make the release
review smarter, but the final transition remains strict. That maps cleanly to
the Axiom-Helm-App split:

| Layer | Boundary decision |
|---|---|
| Helm | Owns the release-readiness packet as an operator read model: display required evidence, present evidence, missing evidence, concerns, forbidden-action checks, latest verdict, and links to raw app/runtime traces. |
| Axiom | Supplies the Truth Package, verifier spec, proof obligations, accepted `CalibrationSuggestion` / `CalibrationConcern` artifacts, and last `AxiomRunReport`. Axiom can say what would be required to judge `Satisfied`; it does not approve release. |
| Tally app/kernel | Supplies agreement state, parties, conditions, custody kind, transition history, candidate release request, and hard transition admission. It remains the only owner of agreement mutation. |
| Organism/Mosaic | Run readiness Formations and specialist suggestors that gather condition evidence, custody evidence, policy proofs, anomaly signals, duplicate-transition checks, and dispute signals. |
| Converge | Promotes readiness facts and records stop reason, evidence refs, trace links, authority evidence, and integrity proof. |

Candidate contract surface:

```text
ReleaseReadinessPacket
  package_id
  truth_version
  domain_hint
  agreement_id
  current_state
  target_transition
  generated_truths_ref
  verifier_required_evidence[]
  verifier_forbidden_actions[]
  evidence_status[]
    clause_id
    label
    status: present | missing | disputed | blocked | concern
    source: tally | converge | axiom | helm
    evidence_refs[]
    trace_links[]
    concern_record_ids[]
  signing_evidence: TransitionSigningEvidence
  condition_evidence
  custody_release_evidence: CustodyReleaseEvidence
  policy_gate_evidence
  idempotency_evidence
  forbidden_action_checks[]
  calibration_suggestions[]
  calibration_concerns[]
  last_axiom_report
    verdict
    observed_stop_reason
    integrity
    promoted_fact_count
  raw_links
    tally_transcript
    converge_run
    app_record
```

This should be a Helm read model before it is a shared Rust type. The repeated
shape is now strong enough to name after Atlas and Quorum pressured it:
`ReleaseReadinessPacket` generalizes into a Helm-owned `JobReadinessPacket`.
It should still not become an Axiom public type because it is an operator read
model over app state, adapter receipts, Axiom reports, and next actions.

Immediate implication: Helm should not ask each app to reinvent "what is
missing before this job can be trusted?" Tally should emit domain facts and
transcripts; Axiom should provide verifier obligations and concerns; Converge
should provide promoted evidence and integrity; Helm should compose those into
one operator surface.

Allowed Helm operator actions from the packet:

- run or rerun a readiness Formation;
- request missing signatures;
- request or refresh custody receipts;
- inspect raw Tally / Converge traces;
- accept, reject, or reset calibration concerns;
- retry verification after new facts arrive.

Not allowed from the packet:

- bypass Tally's transition guard;
- treat Formation confidence as release authority;
- mutate the source JTBD or weaken verifier evidence;
- promote facts without Converge.

Current fixture proof: `tests/escrow_release_marquee.rs` keeps
`ReleaseReadinessPacket` local. It composes the Tally transcript, the local
`ObservationAdapterReceipt`, the Axiom verifier report, required evidence
clause coverage, forbidden-action summaries, and allowed operator actions. The
important boundary is explicit: adapter success does not imply verifier
satisfaction, and the packet never authorizes the transition. A missing
release-condition truth key becomes a Helm-visible missing-evidence status and
an `Invalid` Axiom verdict, not a Tally transition bypass.

### Boundary Probe: AxiomRunObservation Adapter

Fifth pressure-tested control:
raw app transcript to `AxiomRunObservation`.

This is the normalization boundary. Tally emits a domain transcript: release
record ID, state transition, truth keys, signing witnesses, custody receipt,
promotion authority, source command, source app path, and captured-at time.
Axiom verifies a different shape: stop reason, promoted facts, clause IDs,
evidence refs, trace links, integrity proof, replay notes, and optional staged
records.

The adapter maps between those two worlds. In the current Axiom fixture,
`adapt_tally_release_transcript(...)`:

- rejects non-release transitions (`Verified -> Released` only);
- rejects non-`ConditionsMet` release reasons;
- maps principal signing witnesses to buyer-authorization evidence;
- maps the release truth key to delivery/condition evidence;
- maps observed promotion authority to current-policy evidence;
- maps transition identity to idempotency and double-release guard evidence;
- maps custody receipt to disbursement/custody evidence;
- preserves source run metadata as replay notes.

The contract split is:

| Layer | Boundary decision |
|---|---|
| Tally app | Owns the raw transcript schema and domain meaning. It should emit stable, typed release transcripts or event projections. |
| App adapter | Owns the mapping from Tally transcript fields to `AxiomRunObservation` facts and clause IDs. This adapter is app-specific because it knows that a Tally signer maps to buyer authorization and a custody receipt maps to disbursement evidence. |
| Axiom | Owns the normalized observation and report types, verifier computation, lineage audit, and calibration feedstock. Axiom should not import Tally crates or parse Tally internals directly. |
| Helm | Should display both sides: raw app transcript links for audit and normalized `AxiomRunReport` for operator judgment. Helm can host adapter execution, but it should not redefine verifier semantics. |
| Converge | Supplies promoted facts, authority, evidence refs, trace links, and integrity where the adapter is backed by live runtime data rather than recorded fixture data. |

Candidate contract surface:

```text
ObservationAdapterReceipt
  adapter_id
  adapter_version
  status: succeeded | rejected
  source_app
  source_run_id
  source_transcript_ref
  package_id
  truth_version
  domain_hint
  mapped_fact_ids[]
  mapped_clause_ids[]
  dropped_source_fields[]
  warnings[]
  errors[]
  replay_notes[]
```

This is now a public Axiom truth-package type after both Tally and Atlas
repeated the envelope without app-specific nouns. Treat it as the audit
envelope around app-specific adapters, not as the adapter implementation
itself.

Immediate implication: do not centralize app-domain adapters inside Axiom.
Centralize the normalized target and audit expectations:

- Axiom should keep `AxiomRunObservation` and `AxiomRunReport` app-neutral.
- Tally should own the raw release transcript and mapping knowledge.
- Helm should eventually run or display adapters as operator-visible
  transforms with receipt metadata.
- Every adapter should be deterministic for the same transcript, package, and
  truth version.

Adapter failure should be explicit. A transcript that cannot be mapped to the
package should produce a blocked/invalid operator state, not a partial silent
report. The adapter must never invent evidence for a clause it cannot justify
from transcript fields or promoted runtime facts.

## Emerging Common Module Candidates

The probes have produced several named shapes. Do not promote all of them into
shared code at once.

| Shape | Current status | Likely home if repeated |
|---|---|---|
| `ObservationAdapterReceipt` | Promoted to public Axiom truth-package API after Tally and Atlas both repeated the same app-neutral adapter audit envelope; Quorum, Scout, Warden, Triage, Inkling, Plumb, Catalyst, and Fathom repeated it again. | Axiom owns the schema; Helm mirrors/renders/journals it. |
| `JobReadinessPacket` | Helm-owned read-model candidate after Tally, Atlas, Quorum, Scout, Warden, Triage, Inkling, Plumb, Catalyst, and Fathom repeated the same package/report/evidence/operator-action shape under different domain names. | Helm common module; Axiom should not own this public type. |
| Operator/approval/publication/plan/execution/action/outcome receipts | Warden introduced compliance approval and registry publication receipts; Triage repeated the family as operator decision, client approval, and maintenance plan receipts; Plumb repeated it as drift verdict, revision proposal, sponsor approval, and strategy commit receipts; Catalyst repeated it as HITL approval, routing decision, next action, and outcome tracking receipts. These are not Axiom types; they look like Helm ledger payloads for decisions and long-running job milestones. | Helm Operator Control / ledger; stable enough to prototype in Helm while keeping domain payload details app-local. |
| Corpus/window/disagreement/claim receipts | Fathom introduced corpus snapshot, evidence window, disagreement, analyst review, and narrative claim receipts for temporal analytical artifacts. This is not a generic operator-action family; it is the evidence-window sibling Helm needs when claims compare data over time. | Helm temporal-evidence/read-model module; Axiom verifies required evidence and keeps app-neutral report surfaces. |
| Snapshot/permission/index receipts | Inkling introduced vault snapshot, permission, and index receipts for local-first private corpus enrichment. This looks like the creative/knowledge-work sibling of the operator receipt family. | Helm ledger and app-local corpus surfaces; probe Folio/Wykkid/Moosemen/Wolfgang before hardening names. |
| `TransitionSigningEvidence` | Useful sub-shape, but still close to Tally's agreement/signing domain. | Wait for Atlas or another app to need signer/witness coverage. |
| `CustodyReleaseEvidence` | Useful sub-shape, but tied to escrow/custody domains. | Keep app/platform-local unless Atlas has analogous external-action receipt evidence. |

`ObservationAdapterReceipt` is different from the others: it describes the
adapter act, not the domain being adapted. That makes it the most plausible
common module.

Extraction criteria:

1. A second app, starting with Atlas, maps a raw domain transcript into
   `AxiomRunObservation`.
2. The adapter needs deterministic audit metadata: adapter id/version, source
   transcript ref, package id, truth version, mapped facts, mapped clauses,
   warnings, and replay notes.
3. The shape remains free of app nouns such as agreement, custody, signer,
   invoice, customer, or Atlas-specific source names.
4. The receipt can be serialized, reviewed in Helm, and stored without
   embedding raw transcript bodies.
5. The receipt can include hashes/fingerprints of input transcript and output
   observation so replay can prove the adapter was deterministic.

Those criteria now hold for Tally and Atlas, so `ObservationAdapterReceipt` was
promoted before any readiness packet. The public Rust shape adds deterministic
identity:

```text
ObservationAdapterReceipt
  receipt_id
  adapter_id
  adapter_version
  status: succeeded | rejected
  source_app
  source_run_id
  source_transcript_ref
  source_transcript_hash
  package_id
  truth_version
  domain_hint
  observation_hash
  mapped_fact_ids[]
  mapped_clause_ids[]
  dropped_source_fields[]
  warnings[]
  errors[]
  replay_notes[]
```

Adapter rule:

- A successful adapter returns both `AxiomRunObservation` and an
  `ObservationAdapterReceipt` with `status: succeeded`.
- A rejected adapter returns no observation and an
  `ObservationAdapterReceipt` with `status: rejected` plus explicit `errors`.
- `receipt_id` is content-derived from the full receipt input, including
  adapter identity, source transcript hash, package id, truth version, status,
  observation hash when an observation exists, mapped facts/clauses, warnings,
  errors, and replay notes.
- `mapped_fact_ids` and `mapped_clause_ids` describe what the adapter claims
  to have justified. They are not new authority; Axiom still verifies the
  resulting observation against the package.

Current fixture proof: `tests/escrow_release_marquee.rs`,
`tests/atlas_integration_marquee.rs`, `tests/quorum_sense_marquee.rs`,
`tests/scout_sourcing_marquee.rs`, `tests/warden_compliance_marquee.rs`, and
`tests/triage_keeper_marquee.rs`, `tests/inkling_notes_marquee.rs`, and
`tests/plumb_execution_marquee.rs`, `tests/catalyst_biz_marquee.rs`, and
`tests/fathom_narrative_marquee.rs` all use the public type while keeping
app-specific transcript adapters local. Tally proves success and rejection
receipts around escrow release. Atlas proves the same envelope around
identity-consolidation candidates. Quorum proves it again around
organizational sensemaking synthesis readiness. Scout repeats it for governed
sourcing. Warden repeats it for compliance registry shadow-runs. Triage
repeats it for weekly maintenance cycles. Inkling repeats it for local-first
vault enrichment. Plumb repeats it for closed-loop strategy execution.
Catalyst repeats it for everyday business operations. Fathom repeats it for
temporal narrative analysis.

Ownership:

- Axiom owns the schema because it verifies the resulting observation and cares
  about deterministic replay.
- Helm owns display, storage policy, review workflow, and operator navigation
  between receipt, raw transcript, and report.
- Apps own the adapter implementation and raw transcript semantics.

Do not put raw app transcript bodies in the common module. Store references and
hashes. Raw history remains in the app, Helm ExperienceStore, or downstream
trace store.

## Expression Boundary

Axiom should be able to express:

- the actor and job;
- evidence required before satisfaction;
- failure modes and forbidden observations;
- time budget and expected stop reasons;
- policy requirements as obligations;
- package artifacts and lineage;
- post-run verifier expectations;
- reviewed decoder reinforcements and concerns.

Axiom should not express:

- the app's domain state machine implementation;
- payment, custody, or external API execution;
- who signs or hosts plugins at runtime;
- which Formation wins;
- which Mosaic suggestor implementation is best;
- whether a fact is promoted under current authority.

If an app repeats verifier logic, move it toward Axiom. If an app repeats
operator review or truth package display, move it toward Helm. If Axiom starts
encoding runtime strategy, move it back toward Organism, Mosaic, or Converge.

## Atlas Next

After the Tally boundary loop is clearer, the next app vertical should be
`/Users/kpernyer/dev/reflective/marquee-apps/atlas-integration`.

Atlas should not start by copying Tally's app-specific scaffolding. It should
start by consuming the clarified contract:

1. identify one customer-relevant Atlas JTBD;
2. decode it into a Truth Package;
3. decide what Helm should render and operate;
4. define the Atlas adapter from domain outcome to `AxiomRunObservation`;
5. verify the run with Axiom;
6. feed the report into calibration only after the verdict labels are sharp.

The point of doing Tally first is to make Atlas thinner on purpose, not by
wishful abstraction.

## Atlas As Second Probe

Atlas pressure-tests the same surfaces against a different job family:
post-acquisition identity/auth consolidation. The first probe uses the Atlas
truths `candidate-has-reviewable-evidence`,
`owner-approval-before-writeback`, and `bounded-proof-language`.

The Atlas split mirrors Tally but changes the domain nouns:

| Layer | Boundary decision |
|---|---|
| Atlas app | Owns repository portfolio language, integration candidates, capability overlap, evidence packs, migration proposal copy, and raw candidate transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, normalized `AxiomRunObservation`, verifier report, clause-level evidence coverage, and calibration feedstock. It does not import Atlas crates. |
| Helm | Owns the integration cockpit, HITL review, readiness display, and ledger entries for adapter receipts and readiness packets. It does not authorize provider-side writeback. |
| Organism/Mosaic | Own cartography, similarity scoring, repo memory, bounded checks, policy review, and migration sequencing through public capability contracts. |
| Converge | Promotes candidate facts, counterarguments, receipts, policy decisions, and integrity. |

Current fixture proof: `tests/atlas_integration_marquee.rs` builds an Atlas
identity-consolidation JTBD, adapts a recorded candidate transcript into an
`AxiomRunObservation`, emits a public `ObservationAdapterReceipt`, builds an
operator-facing integration readiness packet, and journals backlink-only Helm
ledger entries. One negative path removes the bounded-proof truth key; another
removes the owner-approval truth key. In both cases the adapter still succeeds,
but Axiom returns `Invalid` and Helm sees missing evidence
(`bounded_contract_check` or `owner_approval_state`) without authorizing
writeback.

What repeated cleanly from Tally:

- `ObservationAdapterReceipt` stayed app-neutral: adapter id/version, status,
  source refs/hashes, package id, truth version, mapped facts, mapped clauses,
  warnings, errors, and replay notes.
- The append-only Helm ledger shape repeated: deterministic entry id, payload
  hash, backlink ids, package id, truth version, domain hint, summary, and
  `authority_effect: none`.
- Readiness packets repeated as a Helm read model, but the domain-specific
  name changed. Tally has `ReleaseReadinessPacket`; Atlas has an integration
  readiness packet. The common candidate is now `JobReadinessPacket`, not an
  escrow-specific release packet.

Extraction signal after Atlas: `ObservationAdapterReceipt` was promoted first
because it survived two app families without carrying escrow or Atlas nouns.
After Quorum and Scout, the readiness packet also has a stable generic name:
`JobReadinessPacket`. Its owner is Helm, not Axiom, because it is an operator
read model over Axiom reports, adapter receipts, app subject refs, missing
evidence, and next operator actions.

## Quorum As Third Probe

Quorum pressure-tests the contract against organizational sensemaking, where
the work is ambiguous and contested rather than a deterministic release or
integration candidate. The first probe uses the Quorum truths
`signal-requires-content`, `signal-requires-consent`,
`hypothesis-requires-signal`, `probe-cites-hypothesis`,
`quorum-requires-explicit-threshold`,
`minority-hypotheses-remain-visible`, and
`operator-approval-before-synthesis-action`.

The Quorum split is sharper because the app's value is not one decision. It is
the preservation of an epistemic state under uncertainty.

| Layer | Boundary decision |
|---|---|
| Quorum app | Owns inquiry threads, participant roles, consent refs, signals, hypotheses, probes, synthesis copy, and raw participant/source transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, evidence requirements for epistemic integrity, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not import Quorum crates or inspect raw participant prose. |
| Helm | Owns the operator cockpit for inquiry readiness, HITL synthesis review, missing-evidence actions, and append-only ledger entries for receipts and readiness packets. It does not authorize organizational action by itself. |
| Organism/Mosaic | Own adaptive inquiry Formations, signal extraction, fuzzy confidence propagation, probe generation, memory, policy, provider routing, and optimization through public capability contracts. |
| Converge | Promotes signals, hypotheses, probes, synthesis-readiness facts, stop reasons, integrity proofs, and authority evidence. |

Current fixture proof: `tests/quorum_sense_marquee.rs` builds a
release-readiness JTBD, adapts a recorded Quorum inquiry transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds a
generic Helm-facing `JobReadinessPacket`, and journals backlink-only Helm
ledger entries. One negative path removes minority-hypothesis preservation;
another removes operator approval. In both cases the adapter still succeeds,
but Axiom returns `Invalid` and Helm sees the missing evidence without
authorizing organizational action.

What repeated cleanly from Tally and Atlas:

- `ObservationAdapterReceipt` stayed app-neutral for a third app.
- The append-only Helm ledger shape repeated unchanged: deterministic id,
  payload hash, backlink ids, package id, truth version, domain hint, summary,
  and `authority_effect: none`.
- The readiness packet stabilized as `JobReadinessPacket`: package id, truth
  version, domain hint, job key, subject ref, adapter receipt id/status,
  Axiom verdict, clause-level evidence readiness, verifier forbidden actions,
  operator actions, and an explicit "does not authorize domain action" flag.

What did not move:

- Signal content, participant-source refs, consent semantics, threshold
  topology, and synthesis text remain Quorum/app-owned.
- Fuzzy confidence propagation, probe allocation, anomaly detection, provider
  routing, and memory remain Organism/Mosaic-owned runtime capabilities.
- Authority to promote facts and approve organizational action remains outside
  Helm and Axiom.

Extraction signal after Quorum: implement `JobReadinessPacket` in Helm when
Helm's Operator Control module is ready. Axiom should keep exposing stable
Truth Package ids, clause ids, `AxiomRunReport`, and
`ObservationAdapterReceipt`; Helm composes those with app subject refs and
operator actions.

## Scout As Fourth Probe

Scout pressure-tests the same surfaces against governed sourcing and vendor
selection. Unlike Tally, it is not an irreversible release rail. Unlike Atlas,
it is not an integration candidate. Unlike Quorum, it is not primarily
epistemic sensemaking. Scout is procurement: source packs, candidate vendors,
objective ranking, policy gates, HITL thresholds, and commit authority.

The first probe uses the Scout product truth `vendor-selection` and the
truth keys `source-pack-cited`, `intent-admitted`, `formation-assembled`,
`vendors-screened`, `shortlist-produced`, `policy-authorized`,
`human-approval-recorded`, and `decision-provenance-preserved`.

| Layer | Boundary decision |
|---|---|
| Scout app | Owns source-pack ingestion, vendor records, screening payloads, ranking objective, shortlist explanations, policy inputs, procurement decision records, and raw sourcing transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, sourcing evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not import Scout crates or rank vendors. |
| Helm | Owns the sourcing operator cockpit, missing-evidence review, HITL commitment flow, and append-only ledger entries for receipts and readiness packets. It does not authorize procurement spend by itself. |
| Organism/Mosaic | Own planning, formation assembly, provider/model routing, compliance/risk/cost suggestors, optimization, and synthesis through public capability contracts. |
| Converge | Promotes screened facts, shortlist facts, policy decisions, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/scout_sourcing_marquee.rs` builds a vendor
selection JTBD, adapts a recorded Scout selection transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds
the same generic Helm-facing `JobReadinessPacket`, and journals backlink-only
Helm ledger entries. One negative path removes source-pack provenance; another
turns the selected commitment into an over-threshold pending-approval case. In
both cases the adapter still succeeds, but Axiom returns `Invalid` and Helm
sees the missing evidence without authorizing procurement commitment.

What Scout adds to the common shape:

- `JobReadinessPacket` does not need vendor-specific fields. The existing
  generic fields still hold: `job_key`, `subject_ref`, adapter receipt,
  verdict, evidence status, forbidden actions, operator actions, and
  `authorizes_domain_action: false`.
- The app-specific subject ref changes from escrow agreement, integration
  candidate, or inquiry to `scout://selection/...`.
- The missing-evidence path is not just "missing data"; it can be a policy
  gate or HITL threshold. Helm can still show it through the same clause-level
  evidence status list.

What did not move:

- Vendor ranking, objective weights, source-pack parsing, and procurement
  record writing remain Scout-owned.
- Policy evaluation remains Arbiter/Converge-owned; Axiom reads the observed
  policy decision as report evidence.
- Helm displays readiness and routes HITL; it does not become the procurement
  authority.

Extraction signal after Scout: `JobReadinessPacket` is stable enough for Helm
implementation. Scout did not force new generic fields; it only supplied a new
domain subject and stronger HITL/policy examples.

## Warden As Fifth Probe

Warden pressure-tests the same contract against compliance registry
publication. This is closer to Helm's future Operator Control surface than the
prior apps: Warden has shadow-runs, historical corpora, rule registries,
impact diffs, compliance approval, publication receipts, rollback refs, and
cross-app subscribers.

The first probe uses the Warden truth keys `registry-metadata-complete`,
`immutable-registry-version`, `corpus-scope-named`,
`shadow-diff-produced`, `verdicts-trace-to-rules`,
`impact-breakdown-by-app`, `compliance-approval-recorded`,
`publication-receipt-issued`, and `warden-not-production-gate`.

| Layer | Boundary decision |
|---|---|
| Warden app | Owns rule registry content, rule owners/citations, corpus selection, shadow-run verdicts, registry publication, rollback references, and raw compliance transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, registry publication evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not execute rules or approve registry publication. |
| Helm | Owns the operator review surface, compliance approval record, publication receipt display, rollback navigation, and append-only ledger entries. It does not become the production compliance gate. |
| Organism/Mosaic | Own rule analysis, historical corpus retrieval, impact summarization, diffing, policy suggestors, and long-running shadow-run workers through public capability contracts. |
| Converge | Promotes shadow-run facts, policy decisions, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/warden_compliance_marquee.rs` builds a registry
shadow-run JTBD, adapts a recorded Warden transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals four backlink-only Helm
ledger entries: adapter receipt, readiness packet, compliance approval
receipt, and registry publication receipt. One negative path removes corpus
scope. Another leaves publication present but removes approval. In both cases
the adapter still succeeds, but Axiom returns `Invalid` and Helm sees missing
evidence without authorizing registry publication.

What Warden adds to the common shape:

- `JobReadinessPacket` still does not need registry-specific fields. The
  subject ref changes to `warden://shadow-run/...`, and clause-level evidence
  carries the rest.
- Helm's ledger likely needs an operator-decision payload family, not just a
  generic readiness packet. Warden's local `ComplianceApprovalReceipt` and
  `RegistryPublicationReceipt` prove that approval and publication milestones
  can be deterministic, hash-backed, and backlink-only without storing raw
  rule owners, citations, document ids, source commands, or local app paths.
- The production boundary must stay explicit: Warden can publish a reviewed
  registry that apps reload, but Warden does not become the runtime production
  gate for those apps.

What did not move:

- Rule execution, registry authoring, corpus selection, and publication remain
  Warden-owned.
- Helm records review and publication receipts, but those ledger entries have
  `authority_effect: none`; they do not promote facts or authorize app domain
  actions.
- Axiom verifies the job contract after adaptation. It does not compute
  compliance verdicts, choose the corpus, or bless registry publication.

Extraction signal after Warden: `JobReadinessPacket` is stable enough for Helm
implementation, and Helm Operator Control should expect a small receipt family
around long-running jobs: approval receipts, publication receipts, rollback
refs, and deterministic ledger backlinks. Keep those in Helm unless a narrower
Axiom verifier schema is needed.

## Triage As Sixth Probe

Triage Keeper pressure-tests the contract against sustaining-care operations:
dependency alerts, recurring defects, SLA/risk ranking, patch plans, checks,
client approval, emergency escalation policy, deferred risk, and the boundary
that a maintenance desk recommends and schedules but does not become production
deploy authority.

The first probe uses the Triage truth keys `dependency-alert-cited`,
`recurring-defect-linked`, `service-impact-cited`, `sla-risk-ranked`,
`patch-plan-with-rollback`, `checks-recorded`,
`client-approval-recorded`, `emergency-policy-named`,
`deferred-risk-recorded`, and `triage-not-deploy-authority`.

| Layer | Boundary decision |
|---|---|
| Triage app | Owns sustaining-care language, client support posture, dependency and incident fixtures, patch-plan copy, deferral wording, and raw maintenance-cycle transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, maintenance evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not scan dependencies, run tests, approve clients, or deploy. |
| Helm | Owns the operator review surface, missing-evidence actions, client approval display, maintenance-plan ledger receipts, and long-running job navigation. It does not become production deploy authority. |
| Organism/Mosaic | Own registry and issue-tracker connectors, risk scoring, memory of prior incidents, patch-window scheduling, policy evaluation, and provider/tool routing through public capability contracts. |
| Converge | Promotes alert facts, defect facts, risk rankings, check results, approvals, deferrals, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/triage_keeper_marquee.rs` builds a weekly
maintenance JTBD, adapts a recorded Triage transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals five backlink-only Helm
ledger entries: adapter receipt, readiness packet, operator decision receipt,
client approval receipt, and maintenance plan receipt. One negative path
removes check evidence. Another keeps the patch plan present but removes
client approval. In both cases the adapter still succeeds, but Axiom returns
`Invalid` and Helm sees missing evidence without authorizing deployment.

What Triage adds to the common shape:

- Warden's receipt family generalizes. Compliance approval/publication becomes
  a broader Helm ledger family: operator decisions, client approvals, and
  executable plan receipts.
- Maintenance plans need evidence for both "what should happen now" and "what
  is deliberately deferred." The same clause-level evidence list handles both
  without adding app-specific fields to `JobReadinessPacket`.
- Checks are not generic runtime noise; they are verifier evidence. Missing
  checks produce `Invalid` readiness even when the patch plan and risk ranking
  are otherwise present.

What did not move:

- Dependency scanning, incident correlation, risk scoring, scheduling, test
  execution, and deploy mechanics remain outside Axiom.
- Helm can journal decisions and approvals, but the ledger entries still have
  `authority_effect: none`.
- Triage can recommend, draft, schedule, and monitor. It does not deploy
  production changes outside client authority.

Extraction signal after Triage: Helm Operator Control should likely model a
generic decision/approval/plan receipt family for long-running jobs. Do not
harden the exact names yet; `plumb-execution` should pressure execution
receipts, and `catalyst-biz` should pressure business-change approvals and
outcome tracking.

## Plumb As Eighth Probe

Plumb Execution pressure-tests the contract against closed-loop strategy
execution: anchor a strategy as a versioned artifact, watch operating signals,
detect material drift, compare correction paths, reject unsafe paths, and
commit a governed strategy revision only through sponsor approval and a
promotion gate.

The first probe uses the Plumb truth keys `strategy-anchor-versioned`,
`source-signals-cited`, `drift-verdict-traced`,
`materiality-threshold-applied`, `revision-proposals-named`,
`rejected-path-preserved`, `feasibility-reviewed`,
`adversarial-review-recorded`, `sponsor-approval-recorded`,
`promotion-gate-recorded`, `anchor-revision-committed`, and
`honest-stop-or-reconciliation`.

| Layer | Boundary decision |
|---|---|
| Plumb app | Owns strategy anchors, objectives, assumptions, forbidden moves, operating signal transcripts, drift verdict semantics, revision proposal copy, and strategy commit records. |
| Axiom | Owns the JTBD-to-Truth Package, strategy-execution evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not revise strategy or choose correction paths. |
| Helm | Owns the operator review surface, missing-evidence actions, drift/revision/approval/commit receipt display, and append-only ledger entries. It does not become strategy authority. |
| Organism/Mosaic | Own signal retrieval, anomaly and drift analysis, feasibility review, adversarial review, provider routing, and suggestor execution through public capability contracts. |
| Converge | Promotes signals, drift verdicts, proposals, approvals, commit facts, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/plumb_execution_marquee.rs` builds a strategy
revision JTBD, adapts a recorded Plumb transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals six backlink-only Helm
ledger entries: adapter receipt, readiness packet, drift verdict receipt,
revision proposal receipt, sponsor approval receipt, and strategy commit
receipt. One negative path removes source signals. Another keeps the commit
receipt present but removes sponsor approval. In both cases the adapter still
succeeds, but Axiom returns `Invalid` and Helm sees missing evidence without
authorizing a strategy commit.

What Plumb adds to the common shape:

- The receipt family now covers execution progress, not only review or
  publication milestones. Drift verdict, revision proposal, sponsor approval,
  and strategy commit receipts form a deterministic chain.
- Versioned anchors matter. Axiom can require evidence that a strategy moved
  from one version to another without owning strategy mutation or allowing
  in-place rewrites.
- Rejected paths are evidence. A safe execution loop preserves why an option
  was rejected instead of hiding it after a preferred path is accepted.

What did not move:

- Strategy semantics, objective definitions, assumption ownership, source
  signal selection, and revision authoring remain Plumb-owned.
- Helm journals the execution chain, but every ledger entry has
  `authority_effect: none`.
- Axiom verifies the reported job contract. It does not decide the new
  strategy, choose the Formation, promote facts, or grant sponsor authority.

Extraction signal after Plumb: the Helm ledger family is no longer just
approval/publication/plan. It also needs execution receipts for stateful,
long-running work. `catalyst-biz` should pressure whether the same family can
cover business-change approvals, experiment outcomes, and post-change
tracking.

## Catalyst As Ninth Probe

Catalyst Biz pressure-tests the contract against everyday business operations:
an inbound account arrives messy, the system gathers account context, checks
consent, scores fit, chooses an owner, pauses for HITL approval, queues a next
business action, and registers outcome tracking. This is not one vertical
state machine. It spans CRM, marketing, calendar, opportunity, approval, and
analytics surfaces.

The first probe uses the Catalyst truth keys `job-definition-bound`,
`account-context-loaded`, `inbound-intent-captured`,
`consent-policy-checked`, `fit-score-explained`,
`owner-capacity-checked`, `hitl-approval-recorded`,
`routing-decision-recorded`, `next-action-receipt-issued`,
`outcome-tracking-registered`, `provider-facts-cited`, and
`honest-stop-or-business-result`.

| Layer | Boundary decision |
|---|---|
| Catalyst app | Owns job presentation, input forms, account canvas, product language, raw business transcripts, and how the resulting action is shown to the operator. |
| Axiom | Owns the JTBD-to-Truth Package, business-operation evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not qualify the lead, choose the owner, or send the follow-up. |
| Helm | Owns the operator review surface, HITL/approval display, missing-evidence actions, action/outcome receipt display, and append-only ledger entries. It does not become business authority. |
| Organism/Mosaic | Own dynamic job topology, account memory, consent/policy checks, fit scoring, owner routing, scheduling, provider routing, and suggestor execution through public capability contracts. |
| Converge | Promotes account facts, consent facts, fit assessments, approval facts, routing decisions, next-action receipts, outcome facts, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/catalyst_biz_marquee.rs` builds an inbound
account JTBD, adapts a recorded Catalyst transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals six backlink-only Helm
ledger entries: adapter receipt, readiness packet, HITL approval receipt,
routing decision receipt, next action receipt, and outcome tracking receipt.
One negative path removes consent policy evidence. Another keeps routing
present but removes HITL approval. In both cases the adapter still succeeds,
but Axiom returns `Invalid` and Helm sees missing evidence without authorizing
the customer-visible business action.

What Catalyst adds to the common shape:

- Outcome tracking is a ledger milestone. A job is not only "approved" or
  "executed"; the result must be registered for later measurement.
- Flow timelines are not truth. Catalyst can render progress through
  `helm-flow`, but the verifier evidence comes from promoted facts, receipts,
  and stop reasons.
- Everyday business operations need the same boundary as high-stakes apps:
  consent, owner responsibility, HITL approval, next-action receipt, and
  provider fact provenance.

What did not move:

- Catalyst keeps product UX, job forms, account context, business language,
  and customer-visible action presentation.
- Helm journals approvals, routing, next actions, and outcome tracking, but
  every ledger entry has `authority_effect: none`.
- Axiom verifies the reported job contract. It does not route accounts,
  select owners, approve outreach, promote facts, or run the Formation.

Extraction signal after Catalyst: Helm Operator Control has enough repeated
evidence to prototype a generic receipt family around long-running jobs:
adapter, readiness, approval, decision, plan/execution, action, and outcome.
The next materially different pressure should be Fathom's temporal evidence
windows or the creative/knowledge-work cluster's human-owned content receipts.

## Fathom As Tenth Probe

Fathom Narrative pressure-tests the contract against temporal analytical
claims: a portfolio screen over time-indexed disclosure filings must bind to a
fixed corpus snapshot, name comparable periods, cite filing sections, preserve
disagreements, escalate low-confidence signals, and avoid presenting analyst
attention artifacts as investment recommendations.

The first probe uses the Fathom truth keys `corpus-snapshot-bound`,
`comparison-window-named`, `filing-sections-cited`,
`query-plan-recorded`, `risk-count-delta-computed`,
`language-drift-computed`, `peer-cohort-boundary-named`,
`disagreement-preserved`, `hitl-escalation-recorded`,
`portfolio-selection-recorded`, `narrative-claims-cited`, and
`recommendation-boundary-declared`.

| Layer | Boundary decision |
|---|---|
| Fathom app | Owns filing corpus language, risk-factor fixtures, narrative-analysis product UX, raw analytical transcripts, and how claims are presented to analysts. |
| Axiom | Owns the JTBD-to-Truth Package, temporal-evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not score securities, write claims, or make recommendations. |
| Helm | Owns the operator/analyst review surface, missing temporal-evidence actions, snapshot/window/disagreement/claim receipt display, and append-only ledger entries. It does not become investment authority. |
| Organism/Mosaic | Own formation selection, SEC/corpus connectors, memory of prior runs, drift/language/ranking analytics, policy checks, analyst-capacity optimization, and suggestor execution through public capability contracts. |
| Converge | Promotes corpus facts, drift facts, disagreement facts, HITL requests, portfolio selections, claim citations, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/fathom_narrative_marquee.rs` builds a temporal
evidence JTBD, adapts a recorded Fathom transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals seven backlink-only Helm
ledger entries: adapter receipt, readiness packet, corpus snapshot receipt,
evidence window receipt, disagreement receipt, analyst review receipt, and
narrative claim receipt. One negative path removes the comparison window.
Another keeps cited narrative claims present but removes the recommendation
boundary. In both cases the adapter still succeeds, but Axiom returns
`Invalid` and Helm sees missing evidence without authorizing an analytical
recommendation.

What Fathom adds to the common shape:

- Temporal evidence windows are first-class. "As of which corpus snapshot?"
  and "which comparable periods?" are verifier evidence, not UI labels.
- Disagreement is a durable artifact. Count drift and language drift can
  conflict; the system must preserve that contradiction for analyst review
  instead of collapsing it into one score.
- Narrative claims require citations and boundaries. A claim can be admissible
  for analyst attention while still explicitly not being investment authority.

What did not move:

- Corpus ingestion, filing parsing, risk-language analytics, portfolio
  screening, claim language, and analyst workflow remain outside Axiom.
- Helm journals snapshot, window, disagreement, review, and claim receipts,
  but every ledger entry has `authority_effect: none`.
- Axiom verifies the reported analytical contract. It does not run the
  suggestors, promote facts, select the portfolio, or make capital decisions.

Extraction signal after Fathom: before implementing Helm Operator Control,
Helm should account for two sibling receipt families: long-running job
receipts and temporal-evidence receipts. They share deterministic ledger
backlinks and no authority effect, but temporal receipts need explicit
snapshot/window/as-of semantics.

## Creative And Knowledge-Work Cluster

Folio Editor, Inkling Notes, Moosemen Writer, Wykkid Preso, and Wolfgang Chat
are meaningful because they pressure a different boundary than the operator
apps. The prior probes are mostly about governed external action:
release funds, write back to repositories, publish a registry, schedule a
maintenance plan. This cluster is about private or public meaning-making:
notes, claims, manuscripts, decks, and research memos.

The shared risk is not "did a job run?" It is "did the system preserve human
ownership while enriching, organizing, drafting, or publishing content?"

Useful probe order:

- Inkling first for private corpus safety: snapshots, permissions,
  separable generated metadata, and non-destructive cleanup proposals.
- Folio next when we want public-claim publishing pressure: source evidence,
  standards review, layout/ad-safety, unresolved questions, and editor
  approval.
- Wykkid when presentation reliability matters: claim evidence, timing,
  confidential material, demo readiness, and fallbacks.
- Moosemen when creative canon protection matters: version history, canon
  citations, voice preservation, contradiction evidence, and writer approval.
- Wolfgang when adversarial research matters: cited claims, disagreement,
  high-risk scope narrowing, evidence quality, and sourced memo receipts.

These apps should not force Axiom to become a writing, research, or editor
engine. They should pressure Axiom and Helm to make provenance, permission,
approval, and proposal state legible.

## Inkling As Seventh Probe

Inkling pressure-tests the contract against local-first private vault
enrichment. The product can index, infer, summarize, group duplicates, identify
stale notes, and propose cleanup. The boundary is strict: original user notes
are not silently rewritten, external fetching and OCR are permissioned, and
generated metadata stays separable from the vault.

The first probe uses the Inkling truth keys `vault-snapshot-captured`,
`import-provenance-preserved`, `permissions-declared`,
`derived-metadata-separable`, `duplicates-evidenced`,
`freshness-analysis-evidenced`, `project-hubs-traced`,
`cleanup-suggestions-proposed`, `destructive-change-approval-recorded`, and
`local-first-boundary-preserved`.

| Layer | Boundary decision |
|---|---|
| Inkling app | Owns the vault tree, note editing, imports, capture workflows, generated metadata storage paths, cleanup UX, and raw vault transcripts. |
| Axiom | Owns the JTBD-to-Truth Package, vault-safety evidence requirements, normalized `AxiomRunObservation`, verifier report, and clause-level coverage. It does not read the vault directly or decide edits. |
| Helm | Owns the operator/user review surface for snapshots, permissions, missing evidence, cleanup proposals, and ledger receipts. It does not mutate notes. |
| Organism/Mosaic | Own OCR/PDF/link extraction, graph analysis, duplicate detection, freshness scoring, memory, scheduling, and policy through public capability contracts. |
| Converge | Promotes snapshot facts, permission facts, duplicate candidates, hub candidates, cleanup proposals, stop reasons, integrity proofs, and promotion authority evidence. |

Current fixture proof: `tests/inkling_notes_marquee.rs` builds a vault
navigation JTBD, adapts a recorded Inkling transcript into
`AxiomRunObservation`, emits the public `ObservationAdapterReceipt`, builds the
same Helm-facing `JobReadinessPacket`, and journals five backlink-only Helm
ledger entries: adapter receipt, readiness packet, vault snapshot receipt,
permission receipt, and vault index receipt. One negative path removes
permission evidence while external fetches appear. Another removes the
snapshot and attempts destructive changes. In both cases the adapter still
succeeds, but Axiom returns `Invalid` and Helm sees missing evidence without
authorizing note mutation.

What Inkling adds to the common shape:

- `JobReadinessPacket` still holds for private corpus enrichment; the subject
  ref changes to `inkling://vault-run/...`.
- Helm's receipt family likely needs a sibling for content/corpus safety:
  snapshot receipts, permission receipts, and generated index receipts.
- Suggestions are first-class evidence. Duplicate groups, stale-note cleanup,
  and project hubs are not domain mutations until the user accepts them.

What did not move:

- Vault storage, note editing, import adapters, OCR/PDF/link extraction, graph
  indexing, and cleanup application remain outside Axiom.
- Helm journals snapshot and permission state, but the ledger entries still
  have `authority_effect: none`.
- Inkling can enrich and organize. The user owns the notes.

Extraction signal after Inkling: the creative/knowledge-work apps should be
used when the platform needs to prove human-owned content is protected while
AI proposes structure. Folio, Wykkid, Moosemen, and Wolfgang are meaningful
follow-ups if we want to harden claim/presentation/canon/research variants of
the same provenance and proposal pattern.

## Control Plane Versus Temporal Probes

Warden has now served as the control-plane probe. It confirmed:

- `JobReadinessPacket` holds across a fifth app family;
- Helm likely needs first-class approval/publication receipt payloads;
- the append-only ledger needs typed long-running job milestones and rollback
  backlinks;
- Axiom can express versioned registry and historical corpus obligations
  without owning rule execution.

Fathom Narrative is a better later probe for large data sets and comparing
evidence over time. It should pressure-test a different common shape:

- evidence windows and corpus snapshots;
- longitudinal comparisons and replay as-of dates;
- time-series drift facts with provenance;
- disagreements between promoted facts across periods or cohorts.

Triage has now confirmed that Warden's approval/publication receipt family
does repeat outside compliance publication. Plumb has now confirmed the same
ledger family can carry execution receipts for a closed-loop strategy run:
drift verdict, revision proposal, sponsor approval, and versioned commit.
Catalyst has now confirmed the family also covers everyday business operations:
HITL approval, routing decision, next action, and outcome tracking.
Fathom has now confirmed that temporal analytical artifacts need a sibling
receipt family for corpus snapshots, evidence windows, disagreements, analyst
review, and cited narrative claims.

The practical sequence is now: prototype Helm Operator Control with a split
between long-running job receipts and temporal-evidence receipts. Keep the
common mechanics deterministic, backlink-only, and non-authoritative; keep the
domain payloads app-local until Helm has a real ledger module to host them.

## Falsifiable Signals

The contract is working when:

1. A new app can reuse Helm package review instead of inventing a review UI.
2. A new app can emit `AxiomRunObservation` rather than implement its own
   satisfaction verdict.
3. Axiom can verify app outcomes without importing app crates.
4. Accepted calibration priors improve future packages without changing the
   source JTBD, generated `.truths`, or runtime `IntentPacket`.
5. Accepted concerns produce operator-visible warnings or scaffolding without
   removing required evidence from the verifier spec.
6. Raw app transcripts stay outside Axiom while Axiom keeps durable backlinks
   to distilled learning episodes.
7. Formation selection, authority recompute, and specialist hosting remain out
   of Axiom.
