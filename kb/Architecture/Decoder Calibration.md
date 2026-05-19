---
tags: [architecture, learning, jtbd, verifier]
source: codex
---

# Decoder Calibration

Decoder calibration is the v0.13 learning loop for Axiom. v0.12 first made
the verifier sharp with Tally escrow release; v0.13 turns those audited
outcomes into reviewable decoder priors.

## Purpose

Axiom may learn as a decoder. The goal is to make future JTBD-to-Truth Package
compilation more complete and easier to verify:

- likely evidence requirements for recurring clause shapes;
- likely forbidden actions for recurring failure modes;
- scenario scaffolds that previously caught meaningful gaps;
- verifier expectations that produced clear reports;
- safer defaults based on accepted human overrides.

The learning loop must not make Axiom a runtime reasoner. It must not select
Formations, recompute authority, or host specialist execution.

## Implemented Shape

The v0.13 code introduces four public records in `src/truth_package.rs`:

- `LearningEpisode`: audited verifier outcome distilled from an
  `AxiomRunReport` plus `FactLineageAudit`;
- `CalibrationKey`: deterministic lookup key for a clause shape;
- `CalibrationRecord`: proposed/accepted/rejected/reset decoder prior;
- `CalibrationTable`: the table queried while enriching a regenerated package.

The loop is:

```text
AxiomRunReport + FactLineageAudit
  -> LearningEpisode
  -> proposed CalibrationRecord(s)
  -> operator review
  -> accepted CalibrationTable
  -> apply_decoder_calibration(...)
  -> TruthPackage.artifacts.calibration_suggestions
```

Calibration enriches package artifacts. It does not mutate the source JTBD,
the generated `.truths` projection, or the runtime `IntentPacket`.

## Input Signal

The calibration source is an audited `AxiomRunReport`, not a raw prompt:

- package ID and truth version;
- source JTBD clause IDs and fingerprints;
- verifier spec;
- promoted facts and evidence refs;
- lineage audit result;
- observed stop reason;
- verdict;
- explicit operator overrides or rejections.

The report needs a hard enough label to learn from. That is why Tally escrow
release precedes calibration: irreversible jobs make `Satisfied`, `Blocked`,
and `Invalid` harder to blur.

## Calibration Table

A calibration record is keyed by structured shape, not raw prose alone:

```text
clause_kind
normalized_clause_shape
domain_hint
decoder_rule_id
fingerprint_class
```

Values may include:

```text
suggested_evidence_templates
suggested_failure_scenarios
suggested_policy_requirements
suggested_verifier_expectations
confidence
rationale
source_episode_ids
```

Records are proposed by code and accepted by an operator. Rejected and reset
records stay explicit so calibration can be audited rather than silently
forgotten.

## Persistence And Ownership

Axiom owns the decoder calibration table because it is a decode-time input. The
raw run outcome still belongs downstream: Organism owns Formation behavior,
Converge owns promotion records, and ExperienceStore owns durable operational
facts. Axiom stores only the distilled decoder prior and its custody back to the
source `LearningEpisode`.

The first persistence shape should be append-only and reviewable: exported
`CalibrationRecord` rows keyed by `CalibrationKey`, with status and review note
included. A database can index the same shape later, but the canonical contract
is the typed record, not a hidden cache.

## Query Point

Calibration is applied after deterministic JTBD decoding and before the
enriched package is handed to downstream layers. Any generated artifact
influenced by calibration must carry both:

1. the originating JTBD clause ID;
2. the calibration entry ID.

The JTBD clause remains the root of custody. Calibration can explain why Axiom
filled in a richer artifact, but it cannot replace the human job as the source.

The v0.13 implementation writes these as
`ArtifactKind::CalibrationSuggestion` entries. Each suggestion has normal
`ArtifactLineage`, and `LineageMap::validate_closure(...)` still has to pass.

## Review Rule

Learned priors are auditable inputs, not hidden behavior. Operators must be
able to review, accept, reject, or reset them. Rejected priors should become
negative calibration signals rather than disappearing silently.

## Boundary Rule

Calibration may propose richer evidence templates, failure scenarios, policy
requirement summaries, and verifier expectations. It must not:

- select or score Formations;
- recompute authority;
- name approvers or promotion gates as authority decisions;
- host specialists, plugins, or WASM execution.

The Tally fixture serializes the produced calibration records and asserts those
runtime ownership concepts do not appear in the decoder priors.

## v0.12 Feedstock

The first useful calibration feed comes from the Tally escrow release proof:

- satisfied release: which evidence clauses were sufficient;
- blocked release: which unresolved authority/evidence gate stopped promotion;
- invalid release: which forbidden action made the run unacceptable.

v0.12 records the feedstock shape from the Tally transcript-backed report:
source run ID, domain hint, verdict, observed stop reason, clause fingerprints,
verifier expectations, promoted facts, lineage audit coverage, and observed
promotion policy hash. v0.13 converts that into proposed calibration records
and proves accepted records can enrich a regenerated Truth Package while
preserving deterministic lineage closure.

## Raw History Versus Distilled Priors

Axiom's calibration log persists only *distilled* decoder priors. The typed
records in a `CalibrationTable` carry:

- the clause shape, fingerprint class, domain hint, and decoder rule id that
  identify when the prior applies (`CalibrationKey`);
- the suggested evidence templates, failure scenarios, policy requirements,
  and verifier expectations the decoder should reach for next time
  (`CalibrationValue`);
- the operator's review status and note (`CalibrationStatus`, `review_note`);
- a backlink to the originating learning episode ids (`source_episode_ids`).

Notably absent: full `AxiomRunReport` bodies, full Tally transcripts, per-fact
summaries, promotion authority records, signing witnesses. Those *raw* run
artifacts live in downstream stores — Helms' ExperienceStore, the marquee
app's transition log, the run's trace backend. A reviewer who needs to audit
*why* a prior was proposed follows the `source_episode_ids` backlink into
whichever store holds the episode.

This split is intentional. Axiom owns "what the decoder should reach for next
time"; downstream stores own "what actually happened on a specific run."
Calibration tables stay small, deterministic, and git-trackable; raw run
history stays in systems built for it.

## Operator Review Workflow

The v0.14 review loop is:

1. A v0.12 verifier run produces an `AxiomRunReport` and a `FactLineageAudit`.
   `LearningEpisode::from_report` distills them.
2. `calibration_records_from_learning_episode` emits `Proposed` records — one
   per covered clause.
3. The operator persists the proposed table via `CalibrationTable::to_jsonl`
   and commits the file to a review workspace (typically a git-tracked
   operator-priors repo).
4. The operator opens each `Proposed` record and decides:
   - `table.accept(&record_id, "<note>")` — adopt the prior; future decoding
     will enrich packages with it.
   - `table.reject(&record_id, "<note>")` — discard the prior; it will never
     enrich a package.
   - `table.reset(&record_id, "<note>")` — discard the prior because it has
     become stale; the next learning episode for the same clause shape may
     re-propose it.
5. The operator persists the reviewed table back via `to_jsonl` and commits.
   The note is mandatory — blank or whitespace-only notes raise
   `CalibrationReviewError::EmptyNote`. Unknown record ids raise
   `CalibrationReviewError::RecordNotFound`.
6. The next decoder run loads the table via `from_jsonl` and calls
   `apply_decoder_calibration(package, &table, domain_hint)`. Only `Accepted`
   records influence the regenerated package.

Reviewing an already-reviewed record is allowed — operators can change a
verdict as new information arrives. The review boundary is decoder-only:
accepting a prior changes what the decoder suggests, not what Organism
selects, Converge promotes, or any specialist executes.

## Known Limitations

v0.13 only learns from clauses the run *covered* — either as evidence (cited
by a promoted fact) or as a failure guard. The signal "this clause shape
often goes uncovered" is currently lost:

- An `Invalid` verdict caused by missing required evidence does not produce
  a calibration record for the uncovered evidence clauses. The decoder learns
  nothing about which clause shapes tend to be hard to satisfy.
- A `Blocked` verdict where a HITL gate pauses promotion likewise does not
  generate priors for clauses whose evidence has not yet arrived.

This is intentional for v0.13 — the milestone scope is "learn from verdicts,"
not "learn from missing evidence." A v0.15+ extension can add a third signal
class for uncovered clauses ("this clause shape often goes uncovered →
adjust decoder default") without changing the v0.13 record shape. Track
separately when v0.14 (persistence) lands; the persistence format should
leave room for a `coverage_status` axis on `LearningClauseSignal` or a
parallel signal type.
