---
tags: [marquee, jtbd, verifier, irreversible]
source: codex
---

# Escrow Release

Escrow release is the v0.12 strict-verdict pressure test for Axiom. It is an
irreversible commitment: once funds leave escrow, the system cannot treat a
bad release as a harmless planning miss.

This is a strict verifier proof plus a recorded Tally release transcript. The
concrete app target is
`/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow`; it already has
Tally domain types, release TruthSpecs, a real Converge suggestor, an
attestation custody adapter, Organism signing flow, and bilateral Axiom
transition records. Axiom does not depend on the Tally crate. The test keeps a
local wire transcript for the release outcome and proves the adapter contract.

## JTBD

```text
Actor: escrow operator
Functional job: release escrowed funds to the vendor after buyer authorization
So that: the buyer's payment obligation is settled and the vendor is paid for verified delivery
Evidence required:
  - buyer authorization signed and on file with non-revoked status
  - vendor delivery confirmed by buyer attestation or trusted third-party signal
  - policy gate cleared (sanctions screening current and KYC valid)
  - release request carries a unique idempotency key not previously promoted
  - disbursement transaction recorded with the payment rail and reconciled
Failure modes:
  - release proceeds despite a prior promotion of the same idempotency key
  - release proceeds without an active buyer authorization on file
  - release proceeds despite sanctions screening flagging the recipient
  - release proceeds while the underlying transaction has an open dispute
  - release proceeds without verified delivery evidence
Time budget: 15 minutes
```

## Verdict Cases

| Case | Observation | Expected verdict | Why |
|---|---|---|---|
| Satisfied release | Converged run with all five evidence clauses cited and the double-release guard cited | `Satisfied` | The commitment has the required buyer, delivery, policy, idempotency, and disbursement evidence. |
| Blocked release | Human-in-the-loop gate pending for buyer authorization | `Blocked` | Funds did not leave escrow while a required human/authority gate was unresolved. |
| Invalid release | A promoted policy fact states that release proceeded despite a sanctioned recipient | `Invalid` | The run crossed an explicit failure-mode/forbidden-action boundary. |

## Tally Adapter Recipe

`tests/escrow_release_marquee.rs` includes the Axiom-side recipe for adapting a
Tally release outcome into `AxiomRunObservation`. The recipe reads
`tests/fixtures/tally_escrow_release_transcript.json` and deliberately uses a
local wire shape instead of depending on the Tally crate:

- transition record: `Verified -> Released`, reason `ConditionsMet`, and truth
  keys including `transition-requires-signature` and
  `release-requires-conditions-met`;
- signing witnesses: both principal roles covered by Organism signature refs;
- custody receipt: release receipt adapter and external reference;
- promotion authority: Converge gate ID, policy hash, and approver observed at
  promotion.

The adapter maps those fields onto the escrow JTBD clauses:

- principal signatures -> buyer authorization evidence;
- release conditions truth key -> delivery/condition evidence;
- observed promotion authority -> current policy evidence;
- transition record ID -> idempotency evidence plus double-release guard;
- custody release receipt -> disbursement/custody evidence.

The verifier still decides the verdict. If the Tally release truth key is
missing, the adapted observation lacks required evidence and the report becomes
`Invalid`.

## Calibration Feedstock

The transcript-backed report also proves the data needed for the v0.13
learning loop:

- source run ID and domain hint;
- verdict and observed stop reason;
- package ID, truth version, source clause IDs, and clause fingerprints;
- verifier required evidence and forbidden actions;
- promoted fact IDs and evidence/failure coverage from `audit_fact_lineage`;
- observed promotion policy hash from Converge authority evidence.

v0.13 now turns this data into a `LearningEpisode`, proposes
`CalibrationRecord` rows, and proves accepted records can enrich a regenerated
Truth Package without making Axiom select Formations, recompute authority, or
host Tally execution.

## Boundary Claims

- Axiom declares the release contract and verifies the observed report.
- Axiom does not host the payment rail, escrow runtime, or policy engine.
- Axiom does not recompute authority. Converge must still recompute authority
  against current policy at promotion.
- `PromotedFactRecord` preserves Converge's observed promotion gate and policy
  hash when facts are adapted from promoted `ContextFact`s.
- The fixture is allowed to synthesize promoted facts, but each fact must cite
  a real JTBD evidence or failure-mode clause and pass `audit_fact_lineage`.

## What This Tests

The v0.11 round-driven proof tests dynamic Organism behavior. Escrow release
tests the other side of the verifier: whether `Satisfied`, `Blocked`, and
`Invalid` remain concrete when the job is irreversible.

The first useful implementation shape is intentionally narrow:

- `decode_jtbd(...)` produces the Truth Package with required evidence,
  forbidden actions, time budget, and lineage closure.
- The deterministic decoder emits policy requirement artifacts for the
  commitment envelope: authority must flow through current Converge promotion
  policy, each evidence clause is required before promotion, and each
  failure-mode clause forbids promotion when observed.
- `AxiomRunReport::verify(...)` computes verdicts from the same verifier path
  v0.11 uses.
- `audit_fact_lineage(...)` proves every promoted commitment fact traces back
  to source JTBD clauses and the package truth version.

## Residual Gap To v1.0

v0.12 proves the irreversible commitment verifier and the Tally observation
contract, not the full three-proof vision. The remaining gaps are:

- package the Tally app's release flow as a stable live runner or transcript
  emitter instead of a recorded JSON fixture;
- translate Axiom's policy requirement artifacts into the concrete Cedar
  envelope used by the app/runtime;
- add Quorum sensemaking as the ambiguous-satisfaction proof;
- add Scout sourcing as the governed commercial-decision proof;
- persist and review accepted verifier outcomes as calibration table records.

## Handoff To Calibration

Escrow release is the first feedstock for v0.13 decoder calibration. v0.12 did
not apply learned priors, but its final report contains enough typed outcome
data to become a learning episode:

- the source JTBD clause IDs and fingerprints;
- the verifier spec;
- the promoted facts and evidence refs;
- the lineage audit result;
- the observed stop reason and final verdict.

Those hard labels are what make calibration worth trusting. The next operating
step is persistence and operator review of the calibration table, not expanding
Axiom into runtime strategy.

## Marquee Sequence

The intended marquee sequence after the round-driven v0.11 proof is:

1. Tally escrow release — irreversible commitment, strict verifier semantics.
2. Quorum sensemaking — ambiguous synthesis and softer satisfaction
   conditions.
3. Scout sourcing — governed commercial decisioning with evidence and policy
   gates.
