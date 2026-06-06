// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.
//! The Axiom truth layer for the Reflective stack.
//!
//! This crate turns human jobs and `.truths` specifications into typed,
//! reviewable contracts that downstream runtime layers can execute and Axiom
//! can verify afterward.
//!
//! Release-surface modules:
//!
//! - [`truth_package`]: deterministic JTBD decoding, Truth Packages, verifier
//!   specs, run observations, run reports, observation adapter receipts, and
//!   decoder calibration.
//! - [`intent`]: compile a parsed Truth into the runtime contract Organism
//!   consumes.
//! - [`gherkin`], [`truths`], [`simulation`], [`policy_lens`], [`predicate`],
//!   [`codegen`], and [`compile`]: validate, simulate, analyze, generate, and
//!   compile `.truths` projections and WASM invariant artifacts.
//!
//! # Converge Truths Validation
//!
//! Converge uses `.truths` files as the canonical format, with `.truth` and
//! `.feature` accepted for compatibility.
//! The validator uses LLMs to check specs for:
//!
//! 1. **Business Sense**: Does the spec describe a meaningful invariant?
//! 2. **Compilability**: Can this be translated to a Rust invariant?
//! 3. **Conventions**: Does it follow Converge's patterns?
//!
//! ## Validation example
//!
//! ```ignore
//! use axiom_truth::gherkin::{GherkinValidator, ValidationConfig};
//! use axiom_truth::mock_llm::StaticChatBackend;
//! use std::sync::Arc;
//!
//! let backend = Arc::new(StaticChatBackend::constant("Valid spec"));
//! let validator = GherkinValidator::new(backend, ValidationConfig::default());
//!
//! let result = validator.validate_file("specs/money.truths").await?;
//! println!("Valid: {}", result.is_valid);
//! ```
//!
//! # Compiling to runtime intent
//!
//! Truth lives here. Organism's runtime consumes a typed `IntentPacket` and
//! knows nothing about Truth-shaped sources. Callers compile a parsed
//! [`TruthDocument`] into an `organism_pack::IntentPacket` via
//! [`intent::compile_intent`], then hand the packet to organism's runtime for
//! admission.
//!
//! ```ignore
//! let truth   = axiom_truth::parse_truth_document(source)?;
//! let intent  = axiom_truth::compile_intent(&truth)?;
//! let receipt = runtime.admit_intent(&intent, actor, src, &mut ctx)?;
//! ```
//!
//! See [`intent`] for the full field mapping (Authority, Constraint, Exception,
//! reversibility overrides, expiry parsing, etc.).
//!
//! # Truth Packages and run verification
//!
//! `JtbdInput` is the preferred human-intent entrypoint for the release
//! surface. [`truth_package::decode_jtbd`] produces a deterministic
//! [`truth_package::TruthPackage`] with stable clause IDs, generated `.truths`,
//! proof obligations, a verifier spec, an `IntentPacket`, replay metadata, and
//! lineage.
//!
//! After Organism, Mosaic, Converge, Helm, or an app produces run output, callers
//! normalize it into [`truth_package::AxiomRunObservation`] and call
//! [`truth_package::AxiomRunReport::verify`]. Axiom judges whether the package
//! was satisfied, blocked, exhausted, or invalid without selecting formations,
//! recomputing authority, hosting specialists, or mutating app state.

pub mod applet_manifest;
pub mod codegen;
pub mod compile;
pub mod editor;
pub mod gherkin;
pub mod guidance;
pub mod intent;
pub mod jtbd;
pub mod mock_llm;
pub mod policy_lens;
pub mod predicate;
pub mod provenance;
pub mod simulation;
pub mod truth_package;
pub mod truths;
pub mod validation_view;

pub use applet_manifest::{
    APPLET_MANIFEST_JSON_SCHEMA, APPLET_MANIFEST_TYPESCRIPT_DECLARATIONS, APPLET_MANIFEST_VERSION,
    AppletManifest, AppletManifestError, AppletProjection, AppletStatus, AuthorityEnvelope,
    ConflictPolicy, EmotionalNeed, EvidenceAuthority, EvidenceContract, EvidenceSource,
    FunctionalNeed, RelationalNeed, Reversibility, applet_manifest_json_schema,
    parse_applet_manifest_json, parse_applet_manifest_value,
};
pub use gherkin::{
    GherkinValidator, InvariantClassTag, IssueCategory, ScenarioKind, ScenarioMeta, Severity,
    SpecGenerator, SpecValidation, ValidationConfig, ValidationIssue, extract_all_metas,
    extract_scenario_meta,
};
pub use intent::{
    CompileError, CompileFromSourceError, compile_intent, compile_intent_from_source,
};
pub use mock_llm::StaticChatBackend;
pub use policy_lens::{
    PolicyCoverageReport, PolicyRequirements, PolicyRule, SpendingThreshold, check_coverage,
};
pub use provenance::{
    AXIOM_PROVENANCE, AxiomTruth, TruthPackageSeedPayload, truth_package_seed_fact,
    truth_package_seed_facts,
};
pub use simulation::{
    DeterministicTrace, DomainProfile, DomainProfileCoverage, DomainProfileReport,
    SimulationConfig, SimulationReport, TraceStep, VendorSelectionCoverage, Verdict, simulate,
    simulate_spec,
};
pub use truth_package::{
    ArtifactId, ArtifactKind, ArtifactLineage, AxiomRunObservation, AxiomRunReport,
    AxiomRunStageRecord, AxiomRunVerdict, CalibrationError, CalibrationKey,
    CalibrationPersistenceError, CalibrationRecord, CalibrationReviewError, CalibrationSignalKind,
    CalibrationStatus, CalibrationTable, CalibrationValue, ClauseCoverageStatus, ClauseDisposition,
    ClauseFingerprint, ClauseId, ClauseInput, DecodeJtbdError, EvidenceRefRecord,
    ExpectedStopReason, FactLineageAudit, FactLineageAuditError, GeneratedArtifact, JtbdClause,
    JtbdClauseKind, JtbdDocument, JtbdInput, LearningClauseSignal, LearningEpisode, LineageError,
    LineageMap, ObservationAdapterReceipt, ObservationAdapterReceiptInput,
    ObservationAdapterStatus, ObservedStopReason, PromotedFactRecord, PromotionAuthorityRecord,
    ProofObligation, ProofObligationKind, ReplayProfile, RunIntegrityProof, TimeBudget,
    TraceLinkRecord, TruthOverlayError, TruthPackage, TruthPackageArtifacts, TruthPackageError,
    TruthPackageId, TruthProjectionOverlay, TruthProjectionSource, TruthProjectionVersion,
    VerifierSpec, apply_decoder_calibration, apply_truth_projection_overlay,
    calibration_records_from_learning_episode, canonicalize_clause_text, decode_jtbd,
};
pub use truths::{
    AuthorityBlock, ConstraintBlock, EvidenceBlock, ExceptionBlock, IntentBlock, TruthDocument,
    TruthGovernance, parse_truth_document,
};
