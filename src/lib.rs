// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.
//! Development tools for Converge — the truth layer.
//!
//! This crate provides tooling for developing Converge applications:
//!
//! - [`gherkin`]: Converge Truths validation (business sense, compilability, conventions)
//! - [`intent`]: compile a parsed Truth into the runtime contract organism consumes
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

pub mod codegen;
pub mod compile;
pub mod gherkin;
pub mod guidance;
pub mod intent;
pub mod jtbd;
pub mod mock_llm;
pub mod policy_lens;
pub mod predicate;
pub mod simulation;
pub mod truths;
pub mod validation_view;

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
pub use simulation::{
    DomainProfile, DomainProfileCoverage, DomainProfileReport, SimulationConfig, SimulationReport,
    VendorSelectionCoverage, Verdict, simulate, simulate_spec,
};
pub use truths::{
    AuthorityBlock, ConstraintBlock, EvidenceBlock, ExceptionBlock, IntentBlock, TruthDocument,
    TruthGovernance, parse_truth_document,
};
