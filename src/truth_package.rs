// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Truth Package spine types.
//!
//! This module starts the v0.10 JTBD-as-source path. It does not generate the
//! full Truth Package yet; it defines deterministic clause identity,
//! fingerprints, and lineage closure checks that later package artifacts can
//! rely on.

use crate::jtbd::JTBDMetadata;
use crate::{CompileError, compile_intent, parse_truth_document};
use chrono::{DateTime, Duration, Utc};
use converge_pack::{ContextFact, FactEvidenceRef, FactTraceLink};
use organism_pack::{ForbiddenAction, IntentPacket};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

const DECODER_VERSION: &str = "0.10.0";
const TRUTH_VERSION: &str = "v1";
/// Deterministic epoch used as the anchor for generated `.truths` `Expires:`
/// values. The IntentPacket's `expires` timestamp is `EPOCH + time_budget`
/// when the JTBD declares a budget, and `EPOCH` otherwise. The runtime carries
/// the budget separately via `IntentPacket.context["time_budget_seconds"]`.
const DETERMINISTIC_EXPIRES_EPOCH: &str = "2099-01-01T00:00:00Z";

/// JTBD-declared time budget the runtime must enforce for a job.
///
/// v0.11 carries only a duration in seconds. Richer expiry semantics
/// (deadline-relative-to-event, business-day windows, etc.) are deferred until
/// a marquee job demonstrates they are needed. Presence makes the
/// `TimeBudgetExhausted` stop reason reachable on a real run, which in turn
/// makes the `Exhausted` verdict reachable in `AxiomRunReport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeBudget(u64);

impl TimeBudget {
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn from_minutes(minutes: u64) -> Self {
        Self(minutes * 60)
    }

    pub const fn from_hours(hours: u64) -> Self {
        Self(hours * 3600)
    }

    pub const fn as_seconds(self) -> u64 {
        self.0
    }
}

fn deterministic_expires_line(budget: Option<TimeBudget>) -> String {
    let epoch: DateTime<Utc> = DateTime::parse_from_rfc3339(DETERMINISTIC_EXPIRES_EPOCH)
        .expect("DETERMINISTIC_EXPIRES_EPOCH is a valid RFC-3339 timestamp")
        .with_timezone(&Utc);
    let expires = match budget {
        Some(b) => {
            let seconds = i64::try_from(b.as_seconds()).unwrap_or(i64::MAX);
            epoch + Duration::seconds(seconds)
        }
        None => epoch,
    };
    expires.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Structured JTBD input supplied by a human or a higher-level authoring UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JtbdInput {
    /// Stable job key used as the root of package-local clause IDs.
    pub key: String,
    /// Actor trying to make progress.
    pub actor: String,
    /// Functional job the actor wants done.
    pub functional_job: String,
    /// Downstream value or risk reduction the job serves.
    pub so_that: String,
    /// Evidence that must exist before the job can be treated as satisfied.
    #[serde(default)]
    pub evidence_required: Vec<ClauseInput>,
    /// Failure modes the package must guard against.
    #[serde(default)]
    pub failure_modes: Vec<ClauseInput>,
    /// Optional time budget the runtime must enforce. When present, the
    /// generated `.truths` `Expires:` is anchored at `EPOCH + time_budget` and
    /// the IntentPacket context carries `time_budget_seconds` so a runtime can
    /// produce `TimeBudgetExhausted` honestly. Absent budgets fall back to the
    /// unbounded epoch sentinel.
    #[serde(default)]
    pub time_budget: Option<TimeBudget>,
}

impl JtbdInput {
    pub fn new(
        key: impl Into<String>,
        actor: impl Into<String>,
        functional_job: impl Into<String>,
        so_that: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            actor: actor.into(),
            functional_job: functional_job.into(),
            so_that: so_that.into(),
            evidence_required: Vec::new(),
            failure_modes: Vec::new(),
            time_budget: None,
        }
    }

    /// Builder helper: attach a time budget to a JTBD input.
    #[must_use]
    pub fn with_time_budget(mut self, budget: TimeBudget) -> Self {
        self.time_budget = Some(budget);
        self
    }

    /// Convert legacy `.truths` JTBD metadata into the new structured source
    /// shape. The caller supplies the package-local job key.
    pub fn from_metadata(key: impl Into<String>, metadata: &JTBDMetadata) -> Self {
        Self {
            key: key.into(),
            actor: metadata.actor.clone(),
            functional_job: metadata.job_functional.clone(),
            so_that: metadata.so_that.clone(),
            evidence_required: metadata
                .evidence_required
                .iter()
                .cloned()
                .map(ClauseInput::new)
                .collect(),
            failure_modes: metadata
                .failure_modes
                .iter()
                .cloned()
                .map(ClauseInput::new)
                .collect(),
            time_budget: None,
        }
    }
}

/// A clause in a collection field. Explicit keys preserve identity across
/// substantial wording changes; absent keys are derived deterministically from
/// canonical text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClauseInput {
    pub key: Option<String>,
    pub text: String,
}

impl ClauseInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            key: None,
            text: text.into(),
        }
    }

    pub fn with_key(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            text: text.into(),
        }
    }
}

/// Canonical structured JTBD document with deterministic clause IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JtbdDocument {
    pub key: String,
    pub clauses: Vec<JtbdClause>,
    /// JTBD-declared time budget the runtime must enforce. Not a clause —
    /// budgets are policy, not job content — but participates in deterministic
    /// package regeneration through the `.truths` `Expires:` value and the
    /// IntentPacket context.
    #[serde(default)]
    pub time_budget: Option<TimeBudget>,
}

impl JtbdDocument {
    pub fn from_input(input: JtbdInput) -> Result<Self, TruthPackageError> {
        let key = normalized_key(&input.key, "job");
        let mut clauses = vec![
            scalar_clause(&key, JtbdClauseKind::Actor, "actor", input.actor)?,
            scalar_clause(
                &key,
                JtbdClauseKind::FunctionalJob,
                "functional_job",
                input.functional_job,
            )?,
            scalar_clause(&key, JtbdClauseKind::SoThat, "so_that", input.so_that)?,
        ];

        clauses.extend(collection_clauses(
            &key,
            JtbdClauseKind::EvidenceRequired,
            "evidence",
            input.evidence_required,
        )?);
        clauses.extend(collection_clauses(
            &key,
            JtbdClauseKind::FailureMode,
            "failure",
            input.failure_modes,
        )?);

        ensure_unique_clause_ids(&clauses)?;

        Ok(Self {
            key,
            clauses,
            time_budget: input.time_budget,
        })
    }

    pub fn clause_ids(&self) -> impl Iterator<Item = &ClauseId> {
        self.clauses.iter().map(|clause| &clause.id)
    }

    pub fn clause(&self, id: &ClauseId) -> Option<&JtbdClause> {
        self.clauses.iter().find(|clause| &clause.id == id)
    }

    pub fn clauses_by_kind(&self, kind: JtbdClauseKind) -> impl Iterator<Item = &JtbdClause> {
        self.clauses
            .iter()
            .filter(move |clause| clause.kind == kind)
    }

    fn known_clause_ids(&self) -> BTreeSet<ClauseId> {
        self.clause_ids().cloned().collect()
    }
}

/// One canonical JTBD clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JtbdClause {
    pub id: ClauseId,
    pub kind: JtbdClauseKind,
    pub key: String,
    pub text: String,
    pub canonical_text: String,
    pub fingerprint: ClauseFingerprint,
}

/// Clause category used by deterministic decoder rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JtbdClauseKind {
    Actor,
    FunctionalJob,
    SoThat,
    EvidenceRequired,
    FailureMode,
}

/// Deterministic, package-local clause address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClauseId(String);

impl ClauseId {
    pub fn new(root_key: &str, path: &str) -> Self {
        Self(format!("jtbd.{root_key}.{path}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ClauseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// SHA-256 hash of canonical clause text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClauseFingerprint(String);

impl ClauseFingerprint {
    pub fn from_text(text: &str) -> Self {
        let canonical = canonicalize_clause_text(text);
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        Self(hex_lower(&hasher.finalize()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn short(&self) -> &str {
        &self.0[..12]
    }
}

/// Artifact identity in a Truth Package lineage map.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArtifactId(String);

impl ArtifactId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Deterministic Truth Package identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TruthPackageId(String);

impl TruthPackageId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TruthPackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Artifact category for lineage review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    TruthPackageManifest,
    TruthProjection,
    Scenario,
    Predicate,
    PolicyRequirement,
    InvariantArtifact,
    SimulationCase,
    ReplayProfile,
    IntentField,
    ProofObligation,
    VerifierExpectation,
}

/// Minimal Truth Package manifest produced by the v0.10 deterministic decoder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruthPackage {
    pub package_id: TruthPackageId,
    pub truth_version: String,
    pub source_jtbd: JtbdDocument,
    pub generated_truths: String,
    pub artifacts: TruthPackageArtifacts,
    pub intent_packet: IntentPacket,
    pub proof_obligations: Vec<ProofObligation>,
    pub verifier_spec: VerifierSpec,
    pub replay_profile: ReplayProfile,
    pub lineage: LineageMap,
}

impl TruthPackage {
    /// Return the immutable generated `.truths` projection as a versioned view.
    pub fn base_projection(&self) -> TruthProjectionVersion {
        let source_clause_ids: Vec<ClauseId> = self.source_jtbd.clause_ids().cloned().collect();
        TruthProjectionVersion {
            package_id: self.package_id.clone(),
            base_truth_version: self.truth_version.clone(),
            projection_version: self.truth_version.clone(),
            truths: self.generated_truths.clone(),
            source: TruthProjectionSource::BaseGenerated,
            lineage: ArtifactLineage::new(
                ArtifactId::new(format!("truth_projection.{}", self.source_jtbd.key)),
                ArtifactKind::TruthProjection,
                source_clause_ids,
                "truth_projection.v0",
                DECODER_VERSION,
                &self.source_jtbd,
            ),
        }
    }

    /// Apply a human-authored `.truths` projection overlay without mutating the
    /// immutable generated package. The returned projection is a versioned view
    /// over the package, not a replacement for `generated_truths`.
    pub fn apply_projection_overlay(
        &self,
        overlay: TruthProjectionOverlay,
    ) -> Result<TruthProjectionVersion, TruthOverlayError> {
        apply_truth_projection_overlay(self, overlay)
    }

    /// Compute the post-run verdict for an `AxiomRunObservation` against this
    /// package's verifier spec.
    ///
    /// The verdict reflects only what can be judged from the observation. Deep
    /// authority recompute, invariant enforcement, and promotion gating remain
    /// Converge's responsibility — this function reads what the run reported
    /// and decides whether the contract was kept.
    ///
    /// Precedence:
    /// 1. Promoted facts citing unknown clauses are a lineage violation →
    ///    `Invalid`.
    /// 2. Forbidden action text observed in promoted fact summaries or replay
    ///    notes → `Invalid`. Sharper enforcement runs through typed invariant
    ///    violations carried by the observed stop reason.
    /// 3. Unexpected stop reason → categorize: invalid-class variants
    ///    (`InvariantViolated`, `PromotionRejected`, `RuntimeError`,
    ///    `AgentRefused`) → `Invalid`; budget exhaustion → `Exhausted`;
    ///    HITL or human intervention → `Blocked`; everything else → `Invalid`.
    /// 4. Expected stop reason: every `EvidenceRequired` clause must be cited
    ///    by at least one promoted fact's `source_clause_ids`; otherwise the
    ///    verdict is `Invalid` (the contract specified evidence the run did
    ///    not produce). All conditions met → `Satisfied`.
    pub fn verify(&self, observation: &AxiomRunObservation) -> AxiomRunVerdict {
        compute_verdict(self, observation)
    }
}

/// Artifact groups reserved by the Truth Package contract. v0.10 fills only the
/// deterministic spine; later decoders can populate richer generated artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthPackageArtifacts {
    pub scenarios: Vec<GeneratedArtifact>,
    pub predicates: Vec<GeneratedArtifact>,
    pub policy_requirements: Vec<GeneratedArtifact>,
    pub evidence_expectations: Vec<GeneratedArtifact>,
    pub simulation_cases: Vec<GeneratedArtifact>,
    pub invariant_expectations: Vec<GeneratedArtifact>,
}

/// Reviewable generated artifact summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub summary: String,
    pub source_clause_ids: Vec<ClauseId>,
}

/// Human-authored override for the generated `.truths` projection.
///
/// The overlay is separate from the package so regeneration remains idempotent:
/// the same JTBD still produces the same base package, and reviewable human
/// edits live in their own versioned artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthProjectionOverlay {
    pub overlay_id: ArtifactId,
    pub target_package_id: TruthPackageId,
    pub target_truth_version: String,
    pub projection_version: String,
    pub edited_truths: String,
    pub reason: String,
    pub source_clause_ids: Vec<ClauseId>,
}

impl TruthProjectionOverlay {
    pub fn new(
        target_package_id: TruthPackageId,
        target_truth_version: impl Into<String>,
        projection_version: impl Into<String>,
        edited_truths: impl Into<String>,
        reason: impl Into<String>,
        source_clause_ids: Vec<ClauseId>,
    ) -> Self {
        let target_truth_version = target_truth_version.into();
        let projection_version = projection_version.into();
        let edited_truths = edited_truths.into();
        let overlay_id = overlay_id_for(
            &target_package_id,
            &target_truth_version,
            &projection_version,
            &edited_truths,
        );
        Self {
            overlay_id,
            target_package_id,
            target_truth_version,
            projection_version,
            edited_truths,
            reason: reason.into(),
            source_clause_ids,
        }
    }
}

/// Versioned `.truths` projection view. `BaseGenerated` is the deterministic
/// package output; `OverlayApplied` is a human-authored edit layered above it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthProjectionVersion {
    pub package_id: TruthPackageId,
    pub base_truth_version: String,
    pub projection_version: String,
    pub truths: String,
    pub source: TruthProjectionSource,
    pub lineage: ArtifactLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthProjectionSource {
    BaseGenerated,
    OverlayApplied {
        overlay_id: ArtifactId,
        reason: String,
    },
}

/// Obligation that must be checked by the verifier or by downstream runtime
/// evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofObligation {
    pub artifact_id: ArtifactId,
    pub kind: ProofObligationKind,
    pub description: String,
    pub source_clause_ids: Vec<ClauseId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofObligationKind {
    EvidenceRequired,
    FailureGuard,
}

/// Post-run expectations. This is data only; live run wiring is deferred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierSpec {
    pub expected_stop_reasons: BTreeSet<ExpectedStopReason>,
    pub required_evidence: Vec<String>,
    pub forbidden_actions: Vec<ForbiddenAction>,
    pub satisfaction_conditions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedStopReason {
    Converged,
    CriteriaMet,
    UserCancelled,
    HumanInterventionRequired,
    CycleBudgetExhausted,
    FactBudgetExhausted,
    TokenBudgetExhausted,
    TimeBudgetExhausted,
    InvariantViolated,
    PromotionRejected,
    RuntimeError,
    AgentRefused,
    HitlGatePending,
}

/// Deterministic replay metadata for the package spine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayProfile {
    pub profile_id: ArtifactId,
    pub deterministic: bool,
    pub input_clause_ids: Vec<ClauseId>,
}

/// Post-run verifier verdict emitted by Axiom after comparing an observed run
/// against a Truth Package's verifier spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxiomRunVerdict {
    Satisfied,
    Blocked,
    Exhausted,
    Invalid,
}

/// Converge stop reason shape captured without depending on Converge internals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ObservedStopReason {
    Converged,
    CriteriaMet {
        criteria: Vec<String>,
    },
    UserCancelled,
    HumanInterventionRequired {
        criteria: Vec<String>,
        approval_refs: Vec<String>,
    },
    CycleBudgetExhausted {
        cycles_executed: u32,
        limit: u32,
    },
    FactBudgetExhausted {
        facts_count: u32,
        limit: u32,
    },
    TokenBudgetExhausted {
        tokens_consumed: u64,
        limit: u64,
    },
    TimeBudgetExhausted {
        duration_ms: u64,
        limit_ms: u64,
    },
    InvariantViolated {
        class: String,
        name: String,
        reason: String,
    },
    PromotionRejected {
        proposal_id: String,
        reason: String,
    },
    RuntimeError {
        message: String,
        category: String,
    },
    AgentRefused {
        agent_id: String,
        reason: String,
    },
    HitlGatePending {
        gate_id: String,
        proposal_id: String,
        summary: String,
        agent_id: String,
        cycle: u32,
    },
}

impl ObservedStopReason {
    pub fn expectation_kind(&self) -> ExpectedStopReason {
        match self {
            Self::Converged => ExpectedStopReason::Converged,
            Self::CriteriaMet { .. } => ExpectedStopReason::CriteriaMet,
            Self::UserCancelled => ExpectedStopReason::UserCancelled,
            Self::HumanInterventionRequired { .. } => ExpectedStopReason::HumanInterventionRequired,
            Self::CycleBudgetExhausted { .. } => ExpectedStopReason::CycleBudgetExhausted,
            Self::FactBudgetExhausted { .. } => ExpectedStopReason::FactBudgetExhausted,
            Self::TokenBudgetExhausted { .. } => ExpectedStopReason::TokenBudgetExhausted,
            Self::TimeBudgetExhausted { .. } => ExpectedStopReason::TimeBudgetExhausted,
            Self::InvariantViolated { .. } => ExpectedStopReason::InvariantViolated,
            Self::PromotionRejected { .. } => ExpectedStopReason::PromotionRejected,
            Self::RuntimeError { .. } => ExpectedStopReason::RuntimeError,
            Self::AgentRefused { .. } => ExpectedStopReason::AgentRefused,
            Self::HitlGatePending { .. } => ExpectedStopReason::HitlGatePending,
        }
    }

    pub fn matches_expected(&self, expected: &BTreeSet<ExpectedStopReason>) -> bool {
        expected.contains(&self.expectation_kind())
    }

    pub fn is_budget_exhausted(&self) -> bool {
        matches!(
            self,
            Self::CycleBudgetExhausted { .. }
                | Self::FactBudgetExhausted { .. }
                | Self::TokenBudgetExhausted { .. }
                | Self::TimeBudgetExhausted { .. }
        )
    }

    pub fn is_blocked(&self) -> bool {
        matches!(
            self,
            Self::HumanInterventionRequired { .. } | Self::HitlGatePending { .. }
        )
    }

    pub fn is_invalid(&self) -> bool {
        matches!(
            self,
            Self::InvariantViolated { .. }
                | Self::PromotionRejected { .. }
                | Self::RuntimeError { .. }
                | Self::AgentRefused { .. }
        )
    }
}

/// One promoted fact as rendered by an Axiom run report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotedFactRecord {
    pub context_key: String,
    pub fact_id: String,
    pub summary: String,
    pub source_clause_ids: Vec<ClauseId>,
    pub evidence_refs: Vec<EvidenceRefRecord>,
    pub trace_link: Option<TraceLinkRecord>,
}

impl PromotedFactRecord {
    pub fn from_context_fact(fact: &ContextFact, source_clause_ids: Vec<ClauseId>) -> Self {
        let promotion = fact.promotion_record();
        Self {
            context_key: format!("{:?}", fact.key()),
            fact_id: fact.id().as_str().to_string(),
            summary: fact.text().map_or_else(
                || format!("{} v{}", fact.payload_family(), fact.payload_version()),
                ToString::to_string,
            ),
            source_clause_ids,
            evidence_refs: promotion
                .evidence_refs()
                .iter()
                .map(evidence_ref_record)
                .collect(),
            trace_link: Some(trace_link_record(promotion.trace_link())),
        }
    }
}

/// Evidence reference attached to a promoted fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRefRecord {
    pub evidence_id: String,
    pub source: String,
}

/// Replay trace link attached to a promoted fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLinkRecord {
    pub trace_id: String,
    pub location: Option<String>,
    pub replayable: bool,
}

/// Integrity proof summary captured from the Converge run boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunIntegrityProof {
    pub merkle_root: String,
    pub clock_time: u64,
    pub fact_count: usize,
    pub algorithm: String,
}

impl RunIntegrityProof {
    pub fn sha256_merkle(
        merkle_root: impl Into<String>,
        clock_time: u64,
        fact_count: usize,
    ) -> Self {
        Self {
            merkle_root: merkle_root.into(),
            clock_time,
            fact_count,
            algorithm: "sha256-merkle".to_string(),
        }
    }
}

/// Wiring-free observation of a run that Axiom can package into a report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomRunObservation {
    pub stop_reason: ObservedStopReason,
    pub promoted_facts: Vec<PromotedFactRecord>,
    pub integrity: RunIntegrityProof,
    pub replay_notes: Vec<String>,
    #[serde(default)]
    pub run_stages: Vec<AxiomRunStageRecord>,
}

impl AxiomRunObservation {
    pub fn from_stages(
        stop_reason: ObservedStopReason,
        integrity: RunIntegrityProof,
        replay_notes: Vec<String>,
        run_stages: Vec<AxiomRunStageRecord>,
    ) -> Self {
        let promoted_facts = run_stages
            .iter()
            .flat_map(|stage| stage.promoted_facts.iter().cloned())
            .collect();

        Self {
            stop_reason,
            promoted_facts,
            integrity,
            replay_notes,
            run_stages,
        }
    }
}

/// One reportable execution stage inside a larger job run.
///
/// A job may run through more than one Converge boundary. For example, a
/// dynamic design Formation can converge before a selected work Formation runs.
/// The top-level report still carries the overall observed stop reason; stages
/// preserve the intermediate stop reasons, promoted facts, trace links, and
/// integrity proofs that explain how the job got there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomRunStageRecord {
    pub stage_id: String,
    pub formation_id: Option<String>,
    pub observed_stop_reason: ObservedStopReason,
    pub promoted_facts: Vec<PromotedFactRecord>,
    pub integrity: RunIntegrityProof,
    pub replay_notes: Vec<String>,
}

fn evidence_ref_record(evidence_ref: &FactEvidenceRef) -> EvidenceRefRecord {
    match evidence_ref {
        FactEvidenceRef::Observation(id) => EvidenceRefRecord {
            evidence_id: id.as_str().to_string(),
            source: "observation".to_string(),
        },
        FactEvidenceRef::HumanApproval(id) => EvidenceRefRecord {
            evidence_id: id.as_str().to_string(),
            source: "human_approval".to_string(),
        },
        FactEvidenceRef::Derived(id) => EvidenceRefRecord {
            evidence_id: id.as_str().to_string(),
            source: "derived".to_string(),
        },
    }
}

fn trace_link_record(trace_link: &FactTraceLink) -> TraceLinkRecord {
    match trace_link {
        FactTraceLink::Local(local) => TraceLinkRecord {
            trace_id: local.trace_id().as_str().to_string(),
            location: Some(format!("span:{}", local.span_id().as_str())),
            replayable: true,
        },
        FactTraceLink::Remote(remote) => TraceLinkRecord {
            trace_id: remote.reference().as_str().to_string(),
            location: Some(remote.system().as_str().to_string()),
            replayable: false,
        },
    }
}

/// Auditable post-run report. v0.10 defines the shape; live Organism/Converge
/// adapters are deferred to v0.11.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxiomRunReport {
    pub package_id: TruthPackageId,
    pub truth_version: String,
    pub intent_packet_id: Uuid,
    pub source_clause_ids: Vec<ClauseId>,
    pub verifier_spec: VerifierSpec,
    pub verdict: AxiomRunVerdict,
    pub observed_stop_reason: ObservedStopReason,
    pub promoted_facts: Vec<PromotedFactRecord>,
    pub integrity: RunIntegrityProof,
    pub replay_profile_id: ArtifactId,
    pub replay_notes: Vec<String>,
    #[serde(default)]
    pub run_stages: Vec<AxiomRunStageRecord>,
}

impl AxiomRunReport {
    pub fn from_observation(
        package: &TruthPackage,
        verdict: AxiomRunVerdict,
        observation: AxiomRunObservation,
    ) -> Self {
        Self {
            package_id: package.package_id.clone(),
            truth_version: package.truth_version.clone(),
            intent_packet_id: package.intent_packet.id,
            source_clause_ids: package.source_jtbd.clause_ids().cloned().collect(),
            verifier_spec: package.verifier_spec.clone(),
            verdict,
            observed_stop_reason: observation.stop_reason,
            promoted_facts: observation.promoted_facts,
            integrity: observation.integrity,
            replay_profile_id: package.replay_profile.profile_id.clone(),
            replay_notes: observation.replay_notes,
            run_stages: observation.run_stages,
        }
    }

    /// Compute the verdict for `observation` against `package`'s verifier spec
    /// and build the corresponding `AxiomRunReport`.
    ///
    /// This is the canonical v0.11 entry point: callers with a raw
    /// `AxiomRunObservation` (hand-built, replayed, or adapted from a Converge
    /// run record) should prefer `verify` to `from_observation`. The latter
    /// remains available for adapters that have already computed the verdict
    /// elsewhere.
    pub fn verify(package: &TruthPackage, observation: AxiomRunObservation) -> Self {
        let verdict = package.verify(&observation);
        Self::from_observation(package, verdict, observation)
    }

    pub fn expected_stop_reason_matched(&self) -> bool {
        self.observed_stop_reason
            .matches_expected(&self.verifier_spec.expected_stop_reasons)
    }

    pub fn stage(&self, stage_id: &str) -> Option<&AxiomRunStageRecord> {
        self.run_stages
            .iter()
            .find(|stage| stage.stage_id == stage_id)
    }

    /// Prove that every promoted fact in this report traces back to the source
    /// JTBD clauses it served and that the truth version is consistent.
    ///
    /// Checks performed:
    /// - report `package_id` matches `package.package_id`
    /// - report `truth_version` matches `package.truth_version`
    /// - every `source_clause_ids` entry on every promoted fact is a known
    ///   clause in the package
    /// - every promoted fact cites at least one `EvidenceRequired` or
    ///   `FailureMode` clause it serves (scope-only facts that cite only
    ///   `Actor` / `FunctionalJob` / `SoThat` are rejected — facts must
    ///   discharge an evidence requirement or guard a failure mode)
    ///
    /// On success returns a `FactLineageAudit` summarizing which evidence and
    /// failure-mode clauses were covered by the run's facts.
    pub fn audit_fact_lineage(
        &self,
        package: &TruthPackage,
    ) -> Result<FactLineageAudit, FactLineageAuditError> {
        if self.package_id != package.package_id {
            return Err(FactLineageAuditError::PackageMismatch {
                report: self.package_id.clone(),
                package: package.package_id.clone(),
            });
        }
        if self.truth_version != package.truth_version {
            return Err(FactLineageAuditError::TruthVersionMismatch {
                report: self.truth_version.clone(),
                package: package.truth_version.clone(),
            });
        }

        let known_ids: BTreeSet<&ClauseId> = package
            .source_jtbd
            .clauses
            .iter()
            .map(|clause| &clause.id)
            .collect();
        let evidence_ids: BTreeSet<&ClauseId> = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .map(|clause| &clause.id)
            .collect();
        let failure_ids: BTreeSet<&ClauseId> = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .map(|clause| &clause.id)
            .collect();

        let mut evidence_coverage: BTreeSet<ClauseId> = BTreeSet::new();
        let mut failure_coverage: BTreeSet<ClauseId> = BTreeSet::new();

        for fact in &self.promoted_facts {
            let mut serves_contract = false;
            for clause_id in &fact.source_clause_ids {
                if !known_ids.contains(clause_id) {
                    return Err(FactLineageAuditError::UnknownClause {
                        fact_id: fact.fact_id.clone(),
                        clause_id: clause_id.clone(),
                    });
                }
                if evidence_ids.contains(clause_id) {
                    evidence_coverage.insert(clause_id.clone());
                    serves_contract = true;
                }
                if failure_ids.contains(clause_id) {
                    failure_coverage.insert(clause_id.clone());
                    serves_contract = true;
                }
            }
            if !serves_contract {
                return Err(FactLineageAuditError::ScopeOnlyFact {
                    fact_id: fact.fact_id.clone(),
                });
            }
        }

        Ok(FactLineageAudit {
            package_id: package.package_id.clone(),
            truth_version: package.truth_version.clone(),
            facts_audited: self.promoted_facts.len(),
            evidence_coverage,
            failure_coverage,
        })
    }
}

/// Summary of a successful fact-lineage audit. Captures which evidence and
/// failure-mode clauses the run's promoted facts covered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactLineageAudit {
    pub package_id: TruthPackageId,
    pub truth_version: String,
    pub facts_audited: usize,
    pub evidence_coverage: BTreeSet<ClauseId>,
    pub failure_coverage: BTreeSet<ClauseId>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FactLineageAuditError {
    #[error("report package_id {report:?} does not match audit package {package:?}")]
    PackageMismatch {
        report: TruthPackageId,
        package: TruthPackageId,
    },
    #[error("report truth_version {report} does not match audit package {package}")]
    TruthVersionMismatch { report: String, package: String },
    #[error("promoted fact {fact_id} cites clause {clause_id} not present in the package")]
    UnknownClause {
        fact_id: String,
        clause_id: ClauseId,
    },
    #[error(
        "promoted fact {fact_id} does not cite any evidence_required or failure_mode clause it serves"
    )]
    ScopeOnlyFact { fact_id: String },
}

fn compute_verdict(package: &TruthPackage, observation: &AxiomRunObservation) -> AxiomRunVerdict {
    let spec = &package.verifier_spec;

    // 1. Lineage closure: every cited clause must belong to this package.
    let known_clause_ids: BTreeSet<&ClauseId> = package
        .source_jtbd
        .clauses
        .iter()
        .map(|clause| &clause.id)
        .collect();
    let lineage_violation = observation
        .promoted_facts
        .iter()
        .flat_map(|fact| fact.source_clause_ids.iter())
        .any(|id| !known_clause_ids.contains(id));
    if lineage_violation {
        return AxiomRunVerdict::Invalid;
    }

    // 2. Forbidden action observed — best-effort substring match against
    //    promoted fact summaries and replay notes. Typed invariant violations
    //    on the stop reason are the stronger signal; this catches textual
    //    reports that name a forbidden outcome without raising a typed one.
    let forbidden_observed = spec.forbidden_actions.iter().any(|forbidden| {
        let needle = forbidden.action.as_str();
        observation
            .promoted_facts
            .iter()
            .any(|fact| fact.summary.contains(needle))
            || observation
                .replay_notes
                .iter()
                .any(|note| note.contains(needle))
            || observation
                .run_stages
                .iter()
                .any(|stage| stage.replay_notes.iter().any(|note| note.contains(needle)))
    });
    if forbidden_observed {
        return AxiomRunVerdict::Invalid;
    }

    // 3. Unexpected stop reason — categorize the deviation.
    if !observation
        .stop_reason
        .matches_expected(&spec.expected_stop_reasons)
    {
        if observation.stop_reason.is_invalid() {
            return AxiomRunVerdict::Invalid;
        }
        if observation.stop_reason.is_budget_exhausted() {
            return AxiomRunVerdict::Exhausted;
        }
        if observation.stop_reason.is_blocked() {
            return AxiomRunVerdict::Blocked;
        }
        return AxiomRunVerdict::Invalid;
    }

    // 4. Expected stop reason — every required-evidence clause must be cited
    //    by at least one promoted fact.
    let required_evidence_ids: BTreeSet<&ClauseId> = package
        .source_jtbd
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .map(|clause| &clause.id)
        .collect();
    let cited_clause_ids: BTreeSet<&ClauseId> = observation
        .promoted_facts
        .iter()
        .flat_map(|fact| fact.source_clause_ids.iter())
        .collect();
    if !required_evidence_ids.is_subset(&cited_clause_ids) {
        return AxiomRunVerdict::Invalid;
    }

    AxiomRunVerdict::Satisfied
}

/// One artifact-to-clause lineage statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLineage {
    pub artifact_id: ArtifactId,
    pub artifact_kind: ArtifactKind,
    pub source_clause_ids: Vec<ClauseId>,
    pub decoder_rule_id: String,
    pub decoder_version: String,
    pub input_fingerprints: Vec<ClauseFingerprint>,
}

impl ArtifactLineage {
    pub fn new(
        artifact_id: ArtifactId,
        artifact_kind: ArtifactKind,
        source_clause_ids: Vec<ClauseId>,
        decoder_rule_id: impl Into<String>,
        decoder_version: impl Into<String>,
        document: &JtbdDocument,
    ) -> Self {
        let input_fingerprints = source_clause_ids
            .iter()
            .filter_map(|id| document.clause(id))
            .map(|clause| clause.fingerprint.clone())
            .collect();

        Self {
            artifact_id,
            artifact_kind,
            source_clause_ids,
            decoder_rule_id: decoder_rule_id.into(),
            decoder_version: decoder_version.into(),
            input_fingerprints,
        }
    }
}

/// Explicit disposition for a clause that is not used by an artifact yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseDisposition {
    Deferred { reason: String },
    Rejected { reason: String },
}

/// Bidirectional closure check for clause-to-artifact custody.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LineageMap {
    pub artifacts: Vec<ArtifactLineage>,
    pub clause_dispositions: BTreeMap<ClauseId, ClauseDisposition>,
}

impl LineageMap {
    pub fn single_artifact_from_document(
        document: &JtbdDocument,
        artifact_id: ArtifactId,
        artifact_kind: ArtifactKind,
        decoder_rule_id: impl Into<String>,
        decoder_version: impl Into<String>,
    ) -> Self {
        let source_clause_ids: Vec<ClauseId> = document.clause_ids().cloned().collect();
        Self {
            artifacts: vec![ArtifactLineage::new(
                artifact_id,
                artifact_kind,
                source_clause_ids,
                decoder_rule_id,
                decoder_version,
                document,
            )],
            clause_dispositions: BTreeMap::new(),
        }
    }

    pub fn validate_closure(&self, document: &JtbdDocument) -> Result<(), LineageError> {
        let known = document.known_clause_ids();
        let mut accounted = BTreeSet::new();

        for artifact in &self.artifacts {
            if artifact.source_clause_ids.is_empty() {
                return Err(LineageError::ArtifactWithoutSource {
                    artifact_id: artifact.artifact_id.clone(),
                });
            }

            for clause_id in &artifact.source_clause_ids {
                if !known.contains(clause_id) {
                    return Err(LineageError::UnknownArtifactClause {
                        artifact_id: artifact.artifact_id.clone(),
                        clause_id: clause_id.clone(),
                    });
                }
                accounted.insert(clause_id.clone());
            }
        }

        for clause_id in self.clause_dispositions.keys() {
            if !known.contains(clause_id) {
                return Err(LineageError::UnknownDispositionClause {
                    clause_id: clause_id.clone(),
                });
            }
            accounted.insert(clause_id.clone());
        }

        for clause_id in &known {
            if !accounted.contains(clause_id) {
                return Err(LineageError::UnaccountedClause {
                    clause_id: clause_id.clone(),
                });
            }
        }

        Ok(())
    }
}

/// Errors while normalizing the Truth Package spine.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TruthPackageError {
    #[error("empty JTBD clause: {field}")]
    EmptyClause { field: String },
    #[error("duplicate clause id: {id}")]
    DuplicateClauseId { id: ClauseId },
}

/// Errors while validating a lineage map.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineageError {
    #[error("artifact {artifact_id} has no source clauses")]
    ArtifactWithoutSource { artifact_id: ArtifactId },
    #[error("artifact {artifact_id} references unknown clause {clause_id}")]
    UnknownArtifactClause {
        artifact_id: ArtifactId,
        clause_id: ClauseId,
    },
    #[error("clause disposition references unknown clause {clause_id}")]
    UnknownDispositionClause { clause_id: ClauseId },
    #[error("clause {clause_id} is not used, deferred, or rejected")]
    UnaccountedClause { clause_id: ClauseId },
}

#[derive(Debug, thiserror::Error)]
pub enum TruthOverlayError {
    #[error("overlay targets package {target}, but package is {actual}")]
    PackageMismatch {
        target: TruthPackageId,
        actual: TruthPackageId,
    },
    #[error("overlay targets truth version {target}, but package version is {actual}")]
    TruthVersionMismatch { target: String, actual: String },
    #[error("overlay projection version must not be empty")]
    EmptyProjectionVersion,
    #[error("overlay reason must not be empty")]
    EmptyReason,
    #[error("overlay must name at least one source clause")]
    MissingSourceClauses,
    #[error("overlay references unknown clause {clause_id}")]
    UnknownSourceClause { clause_id: ClauseId },
    #[error("overlay .truths projection did not parse: {message}")]
    TruthProjectionParse { message: String },
}

/// Decode a structured JTBD into the first deterministic Truth Package spine.
///
/// This intentionally uses rule-based generation only. LLM-backed enrichment
/// can be added later, but must feed this deterministic manifest builder.
pub fn decode_jtbd(input: JtbdInput) -> Result<TruthPackage, DecodeJtbdError> {
    let document = JtbdDocument::from_input(input)?;
    let package_id = package_id_for_document(&document);
    let generated_truths = generate_truth_projection(&document);
    let parsed_truth = parse_truth_document(&generated_truths).map_err(|err| {
        DecodeJtbdError::TruthProjectionParse {
            message: err.to_string(),
        }
    })?;
    let mut intent_packet = compile_intent(&parsed_truth)?;
    intent_packet.id = deterministic_uuid(package_id.as_str());
    intent_packet.context = json!({
        "truth_package_id": package_id.as_str(),
        "truth_version": TRUTH_VERSION,
        "source_clause_ids": document
            .clause_ids()
            .map(ClauseId::as_str)
            .collect::<Vec<_>>(),
    });
    if let Some(budget) = document.time_budget
        && let Some(ctx) = intent_packet.context.as_object_mut()
    {
        ctx.insert(
            "time_budget_seconds".to_string(),
            json!(budget.as_seconds()),
        );
    }

    let artifacts = build_artifacts(&document);
    let proof_obligations = build_proof_obligations(&document);
    let verifier_spec = build_verifier_spec(&document);
    let replay_profile = ReplayProfile {
        profile_id: ArtifactId::new(format!("replay_profile.{}", document.key)),
        deterministic: true,
        input_clause_ids: document.clause_ids().cloned().collect(),
    };
    let lineage = build_lineage_map(
        &document,
        &artifacts,
        &proof_obligations,
        &verifier_spec,
        &replay_profile,
    );
    lineage.validate_closure(&document)?;

    Ok(TruthPackage {
        package_id,
        truth_version: TRUTH_VERSION.to_string(),
        source_jtbd: document,
        generated_truths,
        artifacts,
        intent_packet,
        proof_obligations,
        verifier_spec,
        replay_profile,
        lineage,
    })
}

pub fn apply_truth_projection_overlay(
    package: &TruthPackage,
    overlay: TruthProjectionOverlay,
) -> Result<TruthProjectionVersion, TruthOverlayError> {
    if overlay.target_package_id != package.package_id {
        return Err(TruthOverlayError::PackageMismatch {
            target: overlay.target_package_id,
            actual: package.package_id.clone(),
        });
    }
    if overlay.target_truth_version != package.truth_version {
        return Err(TruthOverlayError::TruthVersionMismatch {
            target: overlay.target_truth_version,
            actual: package.truth_version.clone(),
        });
    }
    if canonicalize_clause_text(&overlay.projection_version).is_empty() {
        return Err(TruthOverlayError::EmptyProjectionVersion);
    }
    if canonicalize_clause_text(&overlay.reason).is_empty() {
        return Err(TruthOverlayError::EmptyReason);
    }
    if overlay.source_clause_ids.is_empty() {
        return Err(TruthOverlayError::MissingSourceClauses);
    }

    let known = package.source_jtbd.known_clause_ids();
    for clause_id in &overlay.source_clause_ids {
        if !known.contains(clause_id) {
            return Err(TruthOverlayError::UnknownSourceClause {
                clause_id: clause_id.clone(),
            });
        }
    }

    parse_truth_document(&overlay.edited_truths).map_err(|err| {
        TruthOverlayError::TruthProjectionParse {
            message: err.to_string(),
        }
    })?;

    let lineage = ArtifactLineage::new(
        overlay.overlay_id.clone(),
        ArtifactKind::TruthProjection,
        overlay.source_clause_ids.clone(),
        "truth_projection_overlay.v0",
        DECODER_VERSION,
        &package.source_jtbd,
    );

    Ok(TruthProjectionVersion {
        package_id: package.package_id.clone(),
        base_truth_version: package.truth_version.clone(),
        projection_version: overlay.projection_version,
        truths: overlay.edited_truths,
        source: TruthProjectionSource::OverlayApplied {
            overlay_id: overlay.overlay_id,
            reason: overlay.reason,
        },
        lineage,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeJtbdError {
    #[error(transparent)]
    TruthPackage(#[from] TruthPackageError),
    #[error("generated truth projection did not parse: {message}")]
    TruthProjectionParse { message: String },
    #[error(transparent)]
    Intent(#[from] CompileError),
    #[error(transparent)]
    Lineage(#[from] LineageError),
}

pub fn canonicalize_clause_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn package_id_for_document(document: &JtbdDocument) -> TruthPackageId {
    let mut hasher = Sha256::new();
    hasher.update(document.key.as_bytes());
    for clause in &document.clauses {
        hasher.update(clause.id.as_str().as_bytes());
        hasher.update(clause.fingerprint.as_str().as_bytes());
    }
    let digest = hex_lower(&hasher.finalize());
    TruthPackageId::new(format!("truth_package.{}.{}", document.key, &digest[..12]))
}

fn overlay_id_for(
    package_id: &TruthPackageId,
    target_truth_version: &str,
    projection_version: &str,
    edited_truths: &str,
) -> ArtifactId {
    let mut hasher = Sha256::new();
    hasher.update(package_id.as_str().as_bytes());
    hasher.update(target_truth_version.as_bytes());
    hasher.update(projection_version.as_bytes());
    hasher.update(canonicalize_clause_text(edited_truths).as_bytes());
    let digest = hex_lower(&hasher.finalize());
    ArtifactId::new(format!("truth_projection_overlay.{}", &digest[..12]))
}

fn generate_truth_projection(document: &JtbdDocument) -> String {
    let actor = required_clause(document, JtbdClauseKind::Actor);
    let functional_job = required_clause(document, JtbdClauseKind::FunctionalJob);
    let so_that = required_clause(document, JtbdClauseKind::SoThat);
    let evidence: Vec<&JtbdClause> = document
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .collect();
    let failures: Vec<&JtbdClause> = document
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .collect();

    let mut out = String::new();
    push_line(
        &mut out,
        &format!("Truth: {}", sentence_title(&functional_job.canonical_text)),
    );
    push_line(&mut out, "");
    push_line(&mut out, "  Intent:");
    push_line(
        &mut out,
        &format!("    Outcome: {}", so_that.canonical_text),
    );
    push_line(&mut out, "");
    push_line(&mut out, "  Authority:");
    push_line(&mut out, &format!("    Actor: {}", actor.canonical_text));
    push_line(
        &mut out,
        &format!("    May: attempt {}", functional_job.canonical_text),
    );
    push_line(
        &mut out,
        &format!(
            "    Expires: {}",
            deterministic_expires_line(document.time_budget)
        ),
    );
    if !failures.is_empty() {
        push_line(&mut out, "");
        push_line(&mut out, "  Constraint:");
        for failure in &failures {
            push_line(
                &mut out,
                &format!("    Must Not: {}", failure.canonical_text),
            );
        }
    }
    push_line(&mut out, "");
    push_line(&mut out, "  Evidence:");
    for item in &evidence {
        push_line(&mut out, &format!("    Requires: {}", item.canonical_text));
    }
    push_line(&mut out, "    Provenance: axiom_truth_package");
    push_line(&mut out, "    Audit: truth_package_lineage");
    push_line(&mut out, "");
    push_line(&mut out, "  @acceptance @invariant");
    push_line(
        &mut out,
        "  Scenario: Job is satisfied by required evidence",
    );
    push_line(&mut out, "    Given the actor has the declared job");
    push_line(&mut out, "    When the system evaluates the truth package");
    push_line(
        &mut out,
        "    Then the declared outcome is supported by required evidence",
    );
    out
}

fn build_artifacts(document: &JtbdDocument) -> TruthPackageArtifacts {
    let all_clause_ids: Vec<ClauseId> = document.clause_ids().cloned().collect();
    let scenarios = vec![GeneratedArtifact {
        artifact_id: ArtifactId::new(format!("scenario.{}.satisfaction", document.key)),
        artifact_kind: ArtifactKind::Scenario,
        summary: "Job is satisfied by required evidence".to_string(),
        source_clause_ids: all_clause_ids.clone(),
    }];
    let evidence_expectations = document
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .map(|clause| GeneratedArtifact {
            artifact_id: ArtifactId::new(format!("evidence_expectation.{}", clause.key)),
            artifact_kind: ArtifactKind::VerifierExpectation,
            summary: clause.canonical_text.clone(),
            source_clause_ids: vec![clause.id.clone()],
        })
        .collect();
    let invariant_expectations = document
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .map(|clause| GeneratedArtifact {
            artifact_id: ArtifactId::new(format!("invariant_expectation.{}", clause.key)),
            artifact_kind: ArtifactKind::InvariantArtifact,
            summary: format!("forbid {}", clause.canonical_text),
            source_clause_ids: vec![clause.id.clone()],
        })
        .collect();
    let simulation_cases = vec![GeneratedArtifact {
        artifact_id: ArtifactId::new(format!("simulation_case.{}.baseline", document.key)),
        artifact_kind: ArtifactKind::SimulationCase,
        summary: "baseline deterministic package readiness case".to_string(),
        source_clause_ids: all_clause_ids,
    }];

    TruthPackageArtifacts {
        scenarios,
        predicates: Vec::new(),
        policy_requirements: Vec::new(),
        evidence_expectations,
        simulation_cases,
        invariant_expectations,
    }
}

fn build_proof_obligations(document: &JtbdDocument) -> Vec<ProofObligation> {
    let mut obligations = Vec::new();
    obligations.extend(
        document
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .map(|clause| ProofObligation {
                artifact_id: ArtifactId::new(format!("proof_obligation.evidence.{}", clause.key)),
                kind: ProofObligationKind::EvidenceRequired,
                description: format!("evidence required: {}", clause.canonical_text),
                source_clause_ids: vec![clause.id.clone()],
            }),
    );
    obligations.extend(
        document
            .clauses_by_kind(JtbdClauseKind::FailureMode)
            .map(|clause| ProofObligation {
                artifact_id: ArtifactId::new(format!("proof_obligation.failure.{}", clause.key)),
                kind: ProofObligationKind::FailureGuard,
                description: format!("failure mode must be guarded: {}", clause.canonical_text),
                source_clause_ids: vec![clause.id.clone()],
            }),
    );
    obligations
}

fn build_verifier_spec(document: &JtbdDocument) -> VerifierSpec {
    let required_evidence = document
        .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
        .map(|clause| clause.canonical_text.clone())
        .collect();
    let forbidden_actions = document
        .clauses_by_kind(JtbdClauseKind::FailureMode)
        .map(|clause| ForbiddenAction {
            action: clause.canonical_text.clone(),
            reason: "failure_mode".to_string(),
        })
        .collect();
    let satisfaction_conditions = vec![
        required_clause(document, JtbdClauseKind::SoThat)
            .canonical_text
            .clone(),
    ];

    VerifierSpec {
        expected_stop_reasons: BTreeSet::from([
            ExpectedStopReason::Converged,
            ExpectedStopReason::CriteriaMet,
        ]),
        required_evidence,
        forbidden_actions,
        satisfaction_conditions,
    }
}

fn build_lineage_map(
    document: &JtbdDocument,
    artifacts: &TruthPackageArtifacts,
    proof_obligations: &[ProofObligation],
    verifier_spec: &VerifierSpec,
    replay_profile: &ReplayProfile,
) -> LineageMap {
    let all_clause_ids: Vec<ClauseId> = document.clause_ids().cloned().collect();
    let mut lineages = vec![
        ArtifactLineage::new(
            ArtifactId::new(format!("manifest.{}", document.key)),
            ArtifactKind::TruthPackageManifest,
            all_clause_ids.clone(),
            "truth_package_manifest.v0",
            DECODER_VERSION,
            document,
        ),
        ArtifactLineage::new(
            ArtifactId::new(format!("truth_projection.{}", document.key)),
            ArtifactKind::TruthProjection,
            all_clause_ids.clone(),
            "truth_projection.v0",
            DECODER_VERSION,
            document,
        ),
        ArtifactLineage::new(
            ArtifactId::new(format!("intent_packet.{}", document.key)),
            ArtifactKind::IntentField,
            all_clause_ids.clone(),
            "intent_packet.v0",
            DECODER_VERSION,
            document,
        ),
        ArtifactLineage::new(
            ArtifactId::new(format!("verifier_spec.{}", document.key)),
            ArtifactKind::VerifierExpectation,
            verifier_source_clause_ids(document, verifier_spec),
            "verifier_spec.v0",
            DECODER_VERSION,
            document,
        ),
        ArtifactLineage::new(
            replay_profile.profile_id.clone(),
            ArtifactKind::ReplayProfile,
            replay_profile.input_clause_ids.clone(),
            "replay_profile.v0",
            DECODER_VERSION,
            document,
        ),
    ];

    for artifact in generated_artifact_iter(artifacts) {
        lineages.push(ArtifactLineage::new(
            artifact.artifact_id.clone(),
            artifact.artifact_kind,
            artifact.source_clause_ids.clone(),
            "generated_artifact.v0",
            DECODER_VERSION,
            document,
        ));
    }
    for obligation in proof_obligations {
        lineages.push(ArtifactLineage::new(
            obligation.artifact_id.clone(),
            ArtifactKind::ProofObligation,
            obligation.source_clause_ids.clone(),
            "proof_obligation.v0",
            DECODER_VERSION,
            document,
        ));
    }

    LineageMap {
        artifacts: lineages,
        clause_dispositions: BTreeMap::new(),
    }
}

fn generated_artifact_iter(
    artifacts: &TruthPackageArtifacts,
) -> impl Iterator<Item = &GeneratedArtifact> {
    artifacts
        .scenarios
        .iter()
        .chain(artifacts.predicates.iter())
        .chain(artifacts.policy_requirements.iter())
        .chain(artifacts.evidence_expectations.iter())
        .chain(artifacts.simulation_cases.iter())
        .chain(artifacts.invariant_expectations.iter())
}

fn verifier_source_clause_ids(
    document: &JtbdDocument,
    verifier_spec: &VerifierSpec,
) -> Vec<ClauseId> {
    let mut ids = BTreeSet::new();
    ids.insert(required_clause(document, JtbdClauseKind::SoThat).id.clone());
    if !verifier_spec.required_evidence.is_empty() {
        ids.extend(
            document
                .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
                .map(|clause| clause.id.clone()),
        );
    }
    if !verifier_spec.forbidden_actions.is_empty() {
        ids.extend(
            document
                .clauses_by_kind(JtbdClauseKind::FailureMode)
                .map(|clause| clause.id.clone()),
        );
    }
    ids.into_iter().collect()
}

fn required_clause(document: &JtbdDocument, kind: JtbdClauseKind) -> &JtbdClause {
    document
        .clauses_by_kind(kind)
        .next()
        .expect("JtbdDocument always contains scalar clauses")
}

fn deterministic_uuid(seed: &str) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn sentence_title(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return "Untitled job".to_string();
    };
    format!("{}{}", first.to_uppercase(), chars.as_str())
}

fn push_line(buffer: &mut String, line: &str) {
    buffer.push_str(line);
    buffer.push('\n');
}

fn scalar_clause(
    root_key: &str,
    kind: JtbdClauseKind,
    path: &str,
    text: String,
) -> Result<JtbdClause, TruthPackageError> {
    build_clause(root_key, kind, path, path, text)
}

fn collection_clauses(
    root_key: &str,
    kind: JtbdClauseKind,
    path_prefix: &str,
    inputs: Vec<ClauseInput>,
) -> Result<Vec<JtbdClause>, TruthPackageError> {
    let mut implicit_counts = BTreeMap::<String, usize>::new();
    for input in &inputs {
        if input.key.is_none() {
            let canonical = canonicalize_clause_text(&input.text);
            let fingerprint = ClauseFingerprint::from_text(&canonical);
            let key = implicit_clause_key(&canonical, &fingerprint);
            *implicit_counts.entry(key).or_default() += 1;
        }
    }

    let mut clauses = Vec::with_capacity(inputs.len());
    for input in inputs {
        let canonical = canonicalize_clause_text(&input.text);
        let fingerprint = ClauseFingerprint::from_text(&canonical);
        let explicit = input.key.is_some();
        let base_key = input.key.map_or_else(
            || implicit_clause_key(&canonical, &fingerprint),
            |key| normalized_key(&key, "clause"),
        );
        let key = if explicit || implicit_counts.get(&base_key).copied().unwrap_or(0) <= 1 {
            base_key
        } else {
            format!("{base_key}_{}", fingerprint.short())
        };
        let path = format!("{path_prefix}.{key}");
        clauses.push(build_clause(root_key, kind, &path, &key, input.text)?);
    }

    clauses.sort_by(|left, right| left.id.cmp(&right.id));
    ensure_unique_clause_ids(&clauses)?;
    Ok(clauses)
}

fn build_clause(
    root_key: &str,
    kind: JtbdClauseKind,
    path: &str,
    key: &str,
    text: String,
) -> Result<JtbdClause, TruthPackageError> {
    let canonical_text = canonicalize_clause_text(&text);
    if canonical_text.is_empty() {
        return Err(TruthPackageError::EmptyClause {
            field: path.to_string(),
        });
    }

    Ok(JtbdClause {
        id: ClauseId::new(root_key, path),
        kind,
        key: key.to_string(),
        text,
        fingerprint: ClauseFingerprint::from_text(&canonical_text),
        canonical_text,
    })
}

fn ensure_unique_clause_ids(clauses: &[JtbdClause]) -> Result<(), TruthPackageError> {
    let mut seen = BTreeSet::new();
    for clause in clauses {
        if !seen.insert(clause.id.clone()) {
            return Err(TruthPackageError::DuplicateClauseId {
                id: clause.id.clone(),
            });
        }
    }
    Ok(())
}

fn implicit_clause_key(canonical_text: &str, fingerprint: &ClauseFingerprint) -> String {
    let slug = slugify(canonical_text);
    if slug.is_empty() {
        format!("clause_{}", fingerprint.short())
    } else {
        slug
    }
}

fn normalized_key(value: &str, fallback_prefix: &str) -> String {
    let slug = slugify(value);
    if slug.is_empty() {
        let fingerprint = ClauseFingerprint::from_text(value);
        format!("{fallback_prefix}_{}", fingerprint.short())
    } else {
        slug
    }
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            out.push('_');
            last_was_separator = true;
        }
    }

    out.trim_matches('_').to_string()
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vendor_input() -> JtbdInput {
        JtbdInput {
            key: "Vendor Commitment".to_string(),
            actor: "finance controller".to_string(),
            functional_job: "approve a vendor commitment".to_string(),
            so_that: "spend is traceable and policy-compliant".to_string(),
            evidence_required: vec![
                ClauseInput::new("vendor assessment"),
                ClauseInput::with_key("po", "purchase order"),
            ],
            failure_modes: vec![
                ClauseInput::new("bypassed approval"),
                ClauseInput::new("missing audit trail"),
            ],
            time_budget: None,
        }
    }

    #[test]
    fn deterministic_clause_ids_do_not_depend_on_collection_order() {
        let mut reordered = vendor_input();
        reordered.evidence_required.reverse();
        reordered.failure_modes.reverse();

        let original = JtbdDocument::from_input(vendor_input()).unwrap();
        let reordered = JtbdDocument::from_input(reordered).unwrap();

        assert_eq!(original, reordered);

        let ids: Vec<&str> = original.clause_ids().map(ClauseId::as_str).collect();
        assert!(ids.contains(&"jtbd.vendor_commitment.actor"));
        assert!(ids.contains(&"jtbd.vendor_commitment.functional_job"));
        assert!(ids.contains(&"jtbd.vendor_commitment.so_that"));
        assert!(ids.contains(&"jtbd.vendor_commitment.evidence.vendor_assessment"));
        assert!(ids.contains(&"jtbd.vendor_commitment.evidence.po"));
        assert!(ids.contains(&"jtbd.vendor_commitment.failure.bypassed_approval"));
        assert!(!ids.iter().any(|id| id.contains("[0]") || id.contains(".0")));
    }

    #[test]
    fn explicit_clause_key_preserves_id_while_fingerprint_changes() {
        let mut first = vendor_input();
        first.evidence_required = vec![ClauseInput::with_key("risk_review", "risk review")];

        let mut second = first.clone();
        second.evidence_required = vec![ClauseInput::with_key(
            "risk_review",
            "fresh risk review with policy citations",
        )];

        let first = JtbdDocument::from_input(first).unwrap();
        let second = JtbdDocument::from_input(second).unwrap();

        let first_clause = first
            .clauses
            .iter()
            .find(|clause| clause.key == "risk_review")
            .unwrap();
        let second_clause = second
            .clauses
            .iter()
            .find(|clause| clause.key == "risk_review")
            .unwrap();

        assert_eq!(first_clause.id, second_clause.id);
        assert_ne!(first_clause.fingerprint, second_clause.fingerprint);
    }

    #[test]
    fn lineage_map_closure_requires_every_clause_to_be_accounted_for() {
        let document = JtbdDocument::from_input(vendor_input()).unwrap();
        let map = LineageMap::single_artifact_from_document(
            &document,
            ArtifactId::new("truth_projection.vendor_commitment.v1"),
            ArtifactKind::TruthProjection,
            "truth_projection.v0",
            "0.10.0",
        );

        assert!(map.validate_closure(&document).is_ok());

        let mut missing_one = map.clone();
        missing_one.artifacts[0].source_clause_ids.pop();
        assert!(matches!(
            missing_one.validate_closure(&document),
            Err(LineageError::UnaccountedClause { .. })
        ));

        let mut unknown = map;
        unknown.artifacts[0]
            .source_clause_ids
            .push(ClauseId::new("vendor_commitment", "evidence.unknown"));
        assert!(matches!(
            unknown.validate_closure(&document),
            Err(LineageError::UnknownArtifactClause { .. })
        ));
    }

    #[test]
    fn decode_jtbd_builds_parseable_package_spine() {
        let package = decode_jtbd(vendor_input()).unwrap();

        assert!(
            package
                .generated_truths
                .contains("Truth: Approve a vendor commitment")
        );
        assert!(
            package
                .generated_truths
                .contains("Requires: purchase order")
        );
        assert!(
            package
                .generated_truths
                .contains("Must Not: bypassed approval")
        );
        assert_eq!(package.truth_version, TRUTH_VERSION);
        assert_eq!(
            package.intent_packet.outcome,
            "spend is traceable and policy-compliant"
        );
        assert_eq!(
            package
                .intent_packet
                .context
                .get("truth_package_id")
                .and_then(serde_json::Value::as_str),
            Some(package.package_id.as_str())
        );
        assert_eq!(package.verifier_spec.required_evidence.len(), 2);
        assert_eq!(package.verifier_spec.forbidden_actions.len(), 2);
        assert_eq!(package.proof_obligations.len(), 4);
        assert!(
            package
                .lineage
                .validate_closure(&package.source_jtbd)
                .is_ok()
        );
    }

    #[test]
    fn decode_jtbd_is_deterministic_for_same_semantic_input() {
        let mut reordered = vendor_input();
        reordered.evidence_required.reverse();
        reordered.failure_modes.reverse();

        let first = decode_jtbd(vendor_input()).unwrap();
        let second = decode_jtbd(reordered).unwrap();

        assert_eq!(first.package_id, second.package_id);
        assert_eq!(first.intent_packet.id, second.intent_packet.id);
        assert_eq!(first.generated_truths, second.generated_truths);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
    }

    #[test]
    fn truth_projection_overlay_versions_without_mutating_package() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let original_truths = package.generated_truths.clone();
        let mut edited_truths = package.generated_truths.clone();
        edited_truths.push_str("    And a finance owner can review the evidence\n");
        let actor_clause = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::Actor)
            .next()
            .unwrap()
            .id
            .clone();
        let overlay = TruthProjectionOverlay::new(
            package.package_id.clone(),
            package.truth_version.clone(),
            "v1.operator-review",
            edited_truths.clone(),
            "operator clarified review evidence",
            vec![actor_clause.clone()],
        );

        let applied = package.apply_projection_overlay(overlay.clone()).unwrap();

        assert_eq!(package.generated_truths, original_truths);
        assert_eq!(applied.truths, edited_truths);
        assert_eq!(applied.projection_version, "v1.operator-review");
        assert_eq!(
            applied.source,
            TruthProjectionSource::OverlayApplied {
                overlay_id: overlay.overlay_id,
                reason: "operator clarified review evidence".to_string(),
            }
        );
        assert_eq!(applied.lineage.source_clause_ids, vec![actor_clause]);
        assert_eq!(package.base_projection().truths, original_truths);
    }

    #[test]
    fn truth_projection_overlay_rejects_mismatches_and_unknown_clauses() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let known_clause = package.source_jtbd.clause_ids().next().unwrap().clone();
        let valid_overlay = TruthProjectionOverlay::new(
            package.package_id.clone(),
            package.truth_version.clone(),
            "v1.operator-review",
            package.generated_truths.clone(),
            "operator clarified review evidence",
            vec![known_clause],
        );

        let mut wrong_package = valid_overlay.clone();
        wrong_package.target_package_id = TruthPackageId::new("truth_package.other");
        assert!(matches!(
            package.apply_projection_overlay(wrong_package),
            Err(TruthOverlayError::PackageMismatch { .. })
        ));

        let mut wrong_version = valid_overlay.clone();
        wrong_version.target_truth_version = "v0".to_string();
        assert!(matches!(
            package.apply_projection_overlay(wrong_version),
            Err(TruthOverlayError::TruthVersionMismatch { .. })
        ));

        let mut unknown_clause = valid_overlay.clone();
        unknown_clause.source_clause_ids =
            vec![ClauseId::new("vendor_commitment", "actor.missing")];
        assert!(matches!(
            package.apply_projection_overlay(unknown_clause),
            Err(TruthOverlayError::UnknownSourceClause { .. })
        ));

        let mut invalid_truth = valid_overlay;
        invalid_truth.edited_truths = "Truth: Broken\n\n  Unknown:\n    Value: bad\n".to_string();
        assert!(matches!(
            package.apply_projection_overlay(invalid_truth),
            Err(TruthOverlayError::TruthProjectionParse { .. })
        ));
    }

    #[test]
    fn observed_stop_reason_matches_expected_set() {
        let expected = BTreeSet::from([
            ExpectedStopReason::Converged,
            ExpectedStopReason::CriteriaMet,
        ]);

        let criteria_met = ObservedStopReason::CriteriaMet {
            criteria: vec!["evidence_ready".to_string()],
        };
        let exhausted = ObservedStopReason::TokenBudgetExhausted {
            tokens_consumed: 10_001,
            limit: 10_000,
        };

        assert!(criteria_met.matches_expected(&expected));
        assert!(!exhausted.matches_expected(&expected));
        assert!(exhausted.is_budget_exhausted());
        assert_eq!(
            ObservedStopReason::HitlGatePending {
                gate_id: "gate-1".to_string(),
                proposal_id: "proposal-1".to_string(),
                summary: "approval required".to_string(),
                agent_id: "truth-policy-gate".to_string(),
                cycle: 2,
            }
            .expectation_kind(),
            ExpectedStopReason::HitlGatePending
        );
    }

    #[test]
    fn axiom_run_report_carries_stop_reason_facts_and_integrity() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let evidence_clause = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .next()
            .unwrap()
            .id
            .clone();
        let observation = AxiomRunObservation {
            stop_reason: ObservedStopReason::Converged,
            promoted_facts: vec![PromotedFactRecord {
                context_key: "Evidence".to_string(),
                fact_id: "fact.vendor_assessment".to_string(),
                summary: "vendor assessment present".to_string(),
                source_clause_ids: vec![evidence_clause.clone()],
                evidence_refs: vec![EvidenceRefRecord {
                    evidence_id: "evidence.vendor_assessment".to_string(),
                    source: "axiom_truth_package".to_string(),
                }],
                trace_link: Some(TraceLinkRecord {
                    trace_id: "trace.vendor_commitment.1".to_string(),
                    location: Some("fixture://vendor_commitment".to_string()),
                    replayable: true,
                }),
            }],
            integrity: RunIntegrityProof::sha256_merkle("sha256:abc123", 7, 5),
            replay_notes: vec!["deterministic replay profile matched".to_string()],
            run_stages: Vec::new(),
        };

        let report =
            AxiomRunReport::from_observation(&package, AxiomRunVerdict::Satisfied, observation);

        assert_eq!(report.package_id, package.package_id);
        assert_eq!(report.truth_version, "v1");
        assert_eq!(report.intent_packet_id, package.intent_packet.id);
        assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
        assert!(report.expected_stop_reason_matched());
        assert_eq!(report.promoted_facts.len(), 1);
        assert_eq!(
            report.promoted_facts[0].source_clause_ids,
            vec![evidence_clause]
        );
        assert_eq!(report.integrity.merkle_root, "sha256:abc123");
        assert_eq!(report.integrity.clock_time, 7);
        assert_eq!(report.integrity.fact_count, 5);
        assert_eq!(
            report.source_clause_ids.len(),
            package.source_jtbd.clauses.len()
        );
        assert_eq!(
            serde_json::to_value(&report).unwrap()["observed_stop_reason"]["kind"],
            "converged"
        );
    }

    fn evidence_clause_id(package: &TruthPackage, key: &str) -> ClauseId {
        package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .find(|clause| clause.key == key)
            .map_or_else(
                || panic!("missing evidence clause {key}"),
                |clause| clause.id.clone(),
            )
    }

    fn promoted_fact(
        context_key: &str,
        fact_id: &str,
        summary: &str,
        source_clause_ids: Vec<ClauseId>,
    ) -> PromotedFactRecord {
        PromotedFactRecord {
            context_key: context_key.to_string(),
            fact_id: fact_id.to_string(),
            summary: summary.to_string(),
            source_clause_ids,
            evidence_refs: vec![EvidenceRefRecord {
                evidence_id: format!("evidence.{fact_id}"),
                source: "test_fixture".to_string(),
            }],
            trace_link: Some(TraceLinkRecord {
                trace_id: format!("trace.{fact_id}"),
                location: Some("test://verifier".to_string()),
                replayable: true,
            }),
        }
    }

    fn satisfying_observation(package: &TruthPackage) -> AxiomRunObservation {
        let promoted_facts: Vec<PromotedFactRecord> = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::EvidenceRequired)
            .map(|clause| {
                promoted_fact(
                    "Evidence",
                    &format!("fact.{}", clause.key),
                    &format!("{} observed", clause.canonical_text),
                    vec![clause.id.clone()],
                )
            })
            .collect();
        AxiomRunObservation {
            stop_reason: ObservedStopReason::Converged,
            promoted_facts,
            integrity: RunIntegrityProof::sha256_merkle("sha256:test", 1, 1),
            replay_notes: vec!["deterministic".to_string()],
            run_stages: Vec::new(),
        }
    }

    #[test]
    fn verify_satisfied_when_evidence_complete_and_stop_expected() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let observation = satisfying_observation(&package);

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Satisfied);

        let report = AxiomRunReport::verify(&package, observation);
        assert_eq!(report.verdict, AxiomRunVerdict::Satisfied);
        assert!(report.expected_stop_reason_matched());
    }

    #[test]
    fn verify_invalid_when_promoted_fact_cites_unknown_clause() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.promoted_facts.push(promoted_fact(
            "Evidence",
            "fact.unknown",
            "stray fact with no real clause",
            vec![ClauseId::new("vendor_commitment", "evidence.missing")],
        ));

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Invalid);
    }

    #[test]
    fn verify_invalid_when_forbidden_action_text_appears_in_summary() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.promoted_facts.push(promoted_fact(
            "Diagnostic",
            "fact.violation",
            "bypassed approval detected on commitment ABC",
            vec![evidence_clause_id(&package, "vendor_assessment")],
        ));

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Invalid);
    }

    #[test]
    fn verify_invalid_when_forbidden_action_text_appears_in_replay_note() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation
            .replay_notes
            .push("missing audit trail surfaced post-run".to_string());

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Invalid);
    }

    #[test]
    fn verify_exhausted_when_unexpected_budget_exhaustion() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.stop_reason = ObservedStopReason::TokenBudgetExhausted {
            tokens_consumed: 1_000_000,
            limit: 100_000,
        };

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Exhausted);
    }

    #[test]
    fn verify_blocked_when_unexpected_hitl_gate_pending() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.stop_reason = ObservedStopReason::HitlGatePending {
            gate_id: "gate-1".to_string(),
            proposal_id: "proposal-1".to_string(),
            summary: "approval required".to_string(),
            agent_id: "policy-gate".to_string(),
            cycle: 3,
        };

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Blocked);
    }

    #[test]
    fn verify_invalid_when_invariant_violated() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.stop_reason = ObservedStopReason::InvariantViolated {
            class: "structural".to_string(),
            name: "spend_authority".to_string(),
            reason: "ceiling exceeded".to_string(),
        };

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Invalid);
    }

    #[test]
    fn verify_invalid_when_expected_stop_but_evidence_missing() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let evidence = evidence_clause_id(&package, "vendor_assessment");
        // Cite only vendor_assessment but not the second `po` evidence clause.
        let observation = AxiomRunObservation {
            stop_reason: ObservedStopReason::Converged,
            promoted_facts: vec![promoted_fact(
                "Evidence",
                "fact.vendor_assessment",
                "vendor assessment captured",
                vec![evidence],
            )],
            integrity: RunIntegrityProof::sha256_merkle("sha256:test", 1, 1),
            replay_notes: vec![],
            run_stages: Vec::new(),
        };

        assert_eq!(package.verify(&observation), AxiomRunVerdict::Invalid);
    }

    #[test]
    fn time_budget_absent_uses_epoch_sentinel_and_omits_context_field() {
        let package = decode_jtbd(vendor_input()).unwrap();

        assert!(
            package
                .generated_truths
                .contains("Expires: 2099-01-01T00:00:00Z")
        );
        assert_eq!(package.source_jtbd.time_budget, None);
        assert!(
            package
                .intent_packet
                .context
                .get("time_budget_seconds")
                .is_none()
        );
    }

    #[test]
    fn time_budget_shifts_expires_and_populates_intent_context() {
        let input = vendor_input().with_time_budget(TimeBudget::from_minutes(45));
        let package = decode_jtbd(input).unwrap();

        assert!(
            package
                .generated_truths
                .contains("Expires: 2099-01-01T00:45:00Z")
        );
        assert_eq!(
            package.source_jtbd.time_budget,
            Some(TimeBudget::from_minutes(45))
        );
        assert_eq!(
            package.intent_packet.context["time_budget_seconds"],
            serde_json::json!(45 * 60)
        );
    }

    #[test]
    fn time_budget_preserves_decode_determinism() {
        let first =
            decode_jtbd(vendor_input().with_time_budget(TimeBudget::from_hours(2))).unwrap();
        let second =
            decode_jtbd(vendor_input().with_time_budget(TimeBudget::from_hours(2))).unwrap();

        assert_eq!(first.package_id, second.package_id);
        assert_eq!(first.intent_packet.id, second.intent_packet.id);
        assert_eq!(first.generated_truths, second.generated_truths);
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
    }

    #[test]
    fn audit_fact_lineage_succeeds_when_facts_cite_evidence_clauses() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let report = AxiomRunReport::verify(&package, satisfying_observation(&package));

        let audit = report.audit_fact_lineage(&package).unwrap();
        assert_eq!(audit.package_id, package.package_id);
        assert_eq!(audit.truth_version, package.truth_version);
        assert_eq!(
            audit.facts_audited,
            package.verifier_spec.required_evidence.len()
        );
        assert_eq!(audit.evidence_coverage.len(), 2);
        assert!(audit.failure_coverage.is_empty());
    }

    #[test]
    fn audit_fact_lineage_rejects_unknown_clause() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut observation = satisfying_observation(&package);
        observation.promoted_facts.push(promoted_fact(
            "Evidence",
            "fact.unknown",
            "stray fact",
            vec![ClauseId::new("vendor_commitment", "evidence.missing")],
        ));
        // Use from_observation directly so the verdict path doesn't short-circuit
        // before audit_fact_lineage sees the unknown clause.
        let report =
            AxiomRunReport::from_observation(&package, AxiomRunVerdict::Satisfied, observation);

        match report.audit_fact_lineage(&package) {
            Err(FactLineageAuditError::UnknownClause { fact_id, .. }) => {
                assert_eq!(fact_id, "fact.unknown");
            }
            other => panic!("expected UnknownClause, got {other:?}"),
        }
    }

    #[test]
    fn audit_fact_lineage_rejects_fact_that_cites_only_scope_clauses() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let actor_clause = package
            .source_jtbd
            .clauses_by_kind(JtbdClauseKind::Actor)
            .next()
            .unwrap()
            .id
            .clone();
        let observation = AxiomRunObservation {
            stop_reason: ObservedStopReason::Converged,
            promoted_facts: vec![promoted_fact(
                "Diagnostic",
                "fact.scope-only",
                "actor identity confirmed",
                vec![actor_clause],
            )],
            integrity: RunIntegrityProof::sha256_merkle("sha256:test", 1, 1),
            replay_notes: vec![],
            run_stages: Vec::new(),
        };
        let report =
            AxiomRunReport::from_observation(&package, AxiomRunVerdict::Satisfied, observation);

        match report.audit_fact_lineage(&package) {
            Err(FactLineageAuditError::ScopeOnlyFact { fact_id }) => {
                assert_eq!(fact_id, "fact.scope-only");
            }
            other => panic!("expected ScopeOnlyFact, got {other:?}"),
        }
    }

    #[test]
    fn audit_fact_lineage_rejects_package_id_mismatch() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut report = AxiomRunReport::verify(&package, satisfying_observation(&package));
        report.package_id = TruthPackageId::new("truth_package.other");

        assert!(matches!(
            report.audit_fact_lineage(&package),
            Err(FactLineageAuditError::PackageMismatch { .. })
        ));
    }

    #[test]
    fn audit_fact_lineage_rejects_truth_version_mismatch() {
        let package = decode_jtbd(vendor_input()).unwrap();
        let mut report = AxiomRunReport::verify(&package, satisfying_observation(&package));
        report.truth_version = "v0".to_string();

        assert!(matches!(
            report.audit_fact_lineage(&package),
            Err(FactLineageAuditError::TruthVersionMismatch { .. })
        ));
    }
}
