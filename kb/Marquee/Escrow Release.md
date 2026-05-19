---
tags: [marquee, jtbd, verifier, irreversible]
source: codex
---

# Escrow Release

Escrow release is the v0.12 strict-verdict pressure test for Axiom. It is an
irreversible commitment: once funds leave escrow, the system cannot treat a
bad release as a harmless planning miss.

This is currently a fixture proof plus an adapter recipe, not a live marquee
proof. The concrete app target is
`/Users/kpernyer/dev/reflective/marquee-apps/tally-escrow`; it already has
Tally domain types, release TruthSpecs, a real Converge suggestor, an
attestation custody adapter, Organism signing flow, and bilateral Axiom
transition records. The Axiom fixture should be replaced by a live Tally run
once the app emits a stable release outcome that can be adapted into
`AxiomRunObservation`.

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
Tally release outcome into `AxiomRunObservation`. The recipe deliberately uses a
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

## Residual Gap

The fixture does not yet prove the Tally runtime can produce the observations.
The remaining v0.12 gap is replacing the local wire-shaped adapter fixture with
a live `tally-escrow` release transition run and translating the policy
requirement artifacts into a concrete Cedar envelope. Without that, this is
still valuable as a strict verdict proof plus adapter recipe, but it should be
labeled as fixture-backed.

## Handoff To Calibration

Escrow release is also the first intended feedstock for v0.13 decoder
calibration. v0.12 should not apply learned priors, but its final report should
contain enough typed outcome data to become a future learning episode:

- the source JTBD clause IDs and fingerprints;
- the verifier spec;
- the promoted facts and evidence refs;
- the lineage audit result;
- the observed stop reason and final verdict.

Those hard labels are what make calibration worth trusting later.

## Marquee Sequence

The intended marquee sequence after the round-driven v0.11 proof is:

1. Tally escrow release — irreversible commitment, strict verifier semantics.
2. Quorum sensemaking — ambiguous synthesis and softer satisfaction
   conditions.
3. Scout sourcing — governed commercial decisioning with evidence and policy
   gates.
