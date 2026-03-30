// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.
#![allow(deprecated)] // converge_core::llm::LlmProvider is deprecated upstream

//! Development tools for Converge.
//!
//! This crate provides tooling for developing Converge applications:
//!
//! - [`gherkin`]: Converge Truths validation (business sense, compilability, conventions)
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
//! # Example
//!
//! ```ignore
//! use converge_tool::gherkin::{GherkinValidator, ValidationConfig};
//! use converge_core::llm::MockProvider;
//! use std::sync::Arc;
//!
//! let provider = Arc::new(MockProvider::constant("Valid spec", 0.9));
//! let validator = GherkinValidator::new(provider, ValidationConfig::default());
//!
//! let result = validator.validate_file("specs/money.truths")?;
//! println!("Valid: {}", result.is_valid);
//! ```

pub mod codegen;
pub mod compile;
pub mod gherkin;
pub mod jtbd;
pub mod mock_llm;
pub mod predicate;
pub mod provider_bridge;
pub mod truths;

pub use gherkin::{
    GherkinValidator, InvariantClassTag, IssueCategory, ScenarioKind, ScenarioMeta, Severity,
    SpecGenerator, SpecValidation, ValidationConfig, ValidationIssue, extract_all_metas,
    extract_scenario_meta,
};
pub use mock_llm::StaticLlmProvider;
pub use provider_bridge::ProviderBridge;
pub use truths::{
    AuthorityBlock, ConstraintBlock, EvidenceBlock, ExceptionBlock, IntentBlock, TruthDocument,
    TruthGovernance, parse_truth_document,
};
