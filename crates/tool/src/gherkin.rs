// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Converge Truths validation for Converge.
//!
//! This module provides LLM-powered validation of Converge Truths / Gherkin specifications
//! to ensure they:
//!
//! 1. Make business sense (semantic validity)
//! 2. Can be compiled to Rust invariants (technical feasibility)
//! 3. Follow Converge conventions (style compliance)
//!
//! # Converge Truths
//!
//! Converge uses "Truth" as a branded alias for "Feature" in Gherkin specs.
//! Both keywords are valid:
//!
//! ```gherkin
//! Truth: Get paid for delivered work    # Converge branded syntax
//! Feature: Get paid for delivered work  # Standard Gherkin syntax
//! ```
//!
//! The preprocessor automatically converts `Truth:` to `Feature:` before parsing.
//!
//! # File Extensions
//!
//! Converge supports `.truths` as the canonical extension, with `.truth`
//! and `.feature` accepted for backward compatibility.
//!
//! # Architecture
//!
//! ```text
//! .truths file → Preprocessor → Parser → Scenarios → LLM Validator → Report
//!               (Truth→Feature)              │
//!                                            ├── Business sense check
//!                                            ├── Compilability check
//!                                            └── Convention check
//! ```

use converge_core::traits::{ChatMessage, ChatRequest, ChatResponse, ChatRole, DynChatBackend};
use regex::Regex;
use std::path::Path;
use std::sync::Arc;

use crate::truths::{TruthGovernance, parse_truth_document};

/// Preprocesses Converge Truth syntax to standard Gherkin.
///
/// Converts `Truth:` keyword to `Feature:` for parser compatibility.
/// This allows Converge specs to use the branded "Truth" terminology
/// while maintaining compatibility with standard Gherkin parsers.
///
/// # Examples
///
/// ```
/// use converge_tool::gherkin::preprocess_truths;
///
/// let input = "Truth: Get paid for delivered work\n  Scenario: Invoice";
/// let output = preprocess_truths(input);
/// assert!(output.starts_with("Feature:"));
/// ```
pub fn preprocess_truths(content: &str) -> String {
    // Match "Truth:" at the start of a line (with optional leading whitespace)
    let re = Regex::new(r"(?m)^(\s*)Truth:").unwrap();
    re.replace_all(content, "${1}Feature:").to_string()
}

/// Configuration for Gherkin validation.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to check business sense.
    pub check_business_sense: bool,
    /// Whether to check compilability to Rust.
    pub check_compilability: bool,
    /// Whether to check convention compliance.
    pub check_conventions: bool,
    /// Minimum confidence threshold for LLM assessments.
    pub min_confidence: f64,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_business_sense: true,
            check_compilability: true,
            check_conventions: true,
            min_confidence: 0.7,
        }
    }
}

/// Issue found during validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// The scenario or step that has the issue.
    pub location: String,
    /// Category of the issue.
    pub category: IssueCategory,
    /// Severity level.
    pub severity: Severity,
    /// Human-readable description.
    pub message: String,
    /// Suggested fix (if available).
    pub suggestion: Option<String>,
}

/// Category of validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    /// The spec doesn't make business sense.
    BusinessSense,
    /// The spec cannot be compiled to a Rust invariant.
    Compilability,
    /// The spec doesn't follow conventions.
    Convention,
    /// Syntax error in Gherkin.
    Syntax,
    /// Error not related to Gherkin validation (e.g., LLM API errors, network issues).
    NotRelatedError,
}

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational suggestion.
    Info,
    /// Warning - might cause problems.
    Warning,
    /// Error - must be fixed.
    Error,
}

// ============================================================================
// Scenario Tag Extraction
// ============================================================================

/// Metadata extracted from Gherkin scenario tags.
///
/// Converge uses structured tags on scenarios to declare intent:
///
/// ```gherkin
/// @invariant @structural @id:brand_safety
/// Scenario: Strategies must not contain brand-unsafe terms
/// ```
///
/// This struct captures the parsed tag structure for downstream
/// compilation (codegen, WASM manifest generation, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioMeta {
    /// The scenario name from Gherkin.
    pub name: String,
    /// Parsed scenario kind from tags.
    pub kind: Option<ScenarioKind>,
    /// For invariant scenarios: the invariant class.
    pub invariant_class: Option<InvariantClassTag>,
    /// Unique identifier from `@id:<value>` tag.
    pub id: Option<String>,
    /// Provider type (e.g., "llm") from `@llm` tag.
    pub provider: Option<String>,
    /// Whether this is a test-only scenario (`@test` tag).
    pub is_test: bool,
    /// Raw tags as parsed (for extensibility).
    pub raw_tags: Vec<String>,
}

/// What kind of scenario this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScenarioKind {
    /// A runtime invariant (compiles to `Invariant` impl).
    Invariant,
    /// A proposal validation rule.
    Validation,
    /// An agent contract (what the agent may propose).
    Suggestor,
    /// An end-to-end integration test.
    EndToEnd,
}

/// Invariant class parsed from tags.
///
/// Maps to `converge_core::InvariantClass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvariantClassTag {
    /// Checked after every merge. Violation = immediate failure.
    Structural,
    /// Checked per cycle. Violation = blocks convergence.
    Semantic,
    /// Checked at convergence. Violation = rejects results.
    Acceptance,
}

/// Extract structured metadata from a parsed Gherkin scenario's tags.
///
/// Recognizes Converge tag conventions:
/// - Kind: `@invariant`, `@validation`, `@agent`, `@e2e`
/// - Class: `@structural`, `@semantic`, `@acceptance`
/// - Provider: `@llm`
/// - Test flag: `@test`
/// - Identity: `@id:<identifier>`
///
/// # Examples
///
/// ```
/// # fn main() {
/// use converge_tool::gherkin::{extract_scenario_meta, ScenarioKind, InvariantClassTag};
///
/// // Simulate what gherkin crate produces for:
/// //   @invariant @structural @id:brand_safety
/// //   Scenario: Strategies must not contain brand-unsafe terms
/// let tags = vec!["invariant".to_string(), "structural".to_string(), "id:brand_safety".to_string()];
/// let meta = extract_scenario_meta("Strategies must not contain brand-unsafe terms", &tags);
///
/// assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
/// assert_eq!(meta.invariant_class, Some(InvariantClassTag::Structural));
/// assert_eq!(meta.id.as_deref(), Some("brand_safety"));
/// # }
/// ```
pub fn extract_scenario_meta(name: &str, tags: &[String]) -> ScenarioMeta {
    let mut kind = None;
    let mut invariant_class = None;
    let mut id = None;
    let mut provider = None;
    let mut is_test = false;

    for raw_tag in tags {
        // Strip @ prefix if present (gherkin crate may or may not include it)
        let tag = raw_tag.strip_prefix('@').unwrap_or(raw_tag);

        match tag {
            "invariant" => kind = Some(ScenarioKind::Invariant),
            "validation" => kind = Some(ScenarioKind::Validation),
            "agent" => kind = Some(ScenarioKind::Suggestor),
            "e2e" => kind = Some(ScenarioKind::EndToEnd),
            "structural" => invariant_class = Some(InvariantClassTag::Structural),
            "semantic" => invariant_class = Some(InvariantClassTag::Semantic),
            "acceptance" => invariant_class = Some(InvariantClassTag::Acceptance),
            "llm" => provider = Some("llm".to_string()),
            "test" => is_test = true,
            t if t.starts_with("id:") => {
                id = Some(t.strip_prefix("id:").unwrap_or("").to_string());
            }
            _ => {} // Unknown tags are preserved in raw_tags
        }
    }

    ScenarioMeta {
        name: name.to_string(),
        kind,
        invariant_class,
        id,
        provider,
        is_test,
        raw_tags: tags.to_vec(),
    }
}

/// Extract metadata from all scenarios in a Gherkin/Truth string.
///
/// Parses the content (handling `Truth:` → `Feature:` conversion) and
/// returns a `ScenarioMeta` for each scenario found.
///
/// # Errors
///
/// Returns `ValidationError::ParseError` if the Gherkin cannot be parsed.
///
/// # Examples
///
/// ```
/// use converge_tool::gherkin::{extract_all_metas, ScenarioKind};
///
/// let content = r#"
/// Truth: Growth Strategy Pack
///
///   @invariant @structural @id:brand_safety
///   Scenario: Strategies must not contain brand-unsafe terms
///     Given any fact under key "Strategies"
///     Then it must not contain forbidden terms
///
///   @agent @llm @id:market_signal
///   Scenario: Market Signal agent proposes Signals
///     Given the Context contains facts under key "Seeds"
///     When agent "market_signal" executes
///     Then it proposes facts under key "Signals"
/// "#;
///
/// let metas = extract_all_metas(content).unwrap();
/// assert_eq!(metas.len(), 2);
/// assert_eq!(metas[0].kind, Some(ScenarioKind::Invariant));
/// assert_eq!(metas[1].kind, Some(ScenarioKind::Suggestor));
/// ```
pub fn extract_all_metas(content: &str) -> Result<Vec<ScenarioMeta>, ValidationError> {
    let document = parse_truth_document(content)?;
    let feature = gherkin::Feature::parse(&document.gherkin, gherkin::GherkinEnv::default())
        .map_err(|e| ValidationError::ParseError(format!("{e}")))?;

    Ok(feature
        .scenarios
        .iter()
        .map(|s| extract_scenario_meta(&s.name, &s.tags))
        .collect())
}

/// Result of validating a Gherkin specification.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecValidation {
    /// Whether the spec is valid overall.
    pub is_valid: bool,
    /// Path to the validated file.
    pub file_path: String,
    /// Number of scenarios validated.
    pub scenario_count: usize,
    /// Issues found during validation.
    pub issues: Vec<ValidationIssue>,
    /// Overall confidence score (0.0 - 1.0).
    pub confidence: f64,
    /// Parsed metadata for each scenario.
    pub scenario_metas: Vec<ScenarioMeta>,
    /// Parsed Converge governance declarations for the Truth.
    pub governance: TruthGovernance,
}

impl SpecValidation {
    /// Returns true if there are any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    /// Returns true if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Warning)
    }

    /// Returns a summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        let errors = self
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        let warnings = self
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();

        if self.is_valid {
            format!(
                "✓ {} validated ({} scenarios, {} warnings)",
                self.file_path, self.scenario_count, warnings
            )
        } else {
            format!(
                "✗ {} invalid ({} errors, {} warnings)",
                self.file_path, errors, warnings
            )
        }
    }
}

/// LLM-powered Gherkin specification validator.
pub struct GherkinValidator {
    backend: Arc<dyn DynChatBackend>,
    config: ValidationConfig,
}

impl std::fmt::Debug for GherkinValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GherkinValidator")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl GherkinValidator {
    /// Creates a new validator with the given chat backend.
    #[must_use]
    pub fn new(backend: Arc<dyn DynChatBackend>, config: ValidationConfig) -> Self {
        Self { backend, config }
    }

    /// Validates a Gherkin specification from a string.
    ///
    /// Supports both standard Gherkin (`Feature:`) and Converge Truth (`Truth:`) syntax.
    ///
    /// # Errors
    ///
    /// Returns error if the specification cannot be parsed or validated.
    /// LLM API errors are wrapped as `ValidationError::LlmError` with "`NOT_RELATED_ERROR`:" prefix
    /// to distinguish them from Gherkin validation issues.
    pub fn validate(
        &self,
        content: &str,
        file_name: &str,
    ) -> Result<SpecValidation, ValidationError> {
        let document = parse_truth_document(content)?;

        // Parse the Gherkin content
        // Syntax errors are Gherkin validation issues
        let feature = gherkin::Feature::parse(&document.gherkin, gherkin::GherkinEnv::default())
            .map_err(|e| ValidationError::ParseError(format!("{e}")))?;

        let mut issues = Vec::new();
        let scenario_count = feature.scenarios.len();

        // Validate each scenario
        for scenario in &feature.scenarios {
            let scenario_issues = self.validate_scenario(&feature, scenario)?;
            issues.extend(scenario_issues);
        }

        // Check overall feature structure
        let feature_issues = self.validate_feature(&feature, &document.governance)?;
        issues.extend(feature_issues);

        // Extract structured metadata from scenario tags
        let scenario_metas: Vec<ScenarioMeta> = feature
            .scenarios
            .iter()
            .map(|s| extract_scenario_meta(&s.name, &s.tags))
            .collect();

        let has_errors = issues.iter().any(|i| i.severity == Severity::Error);
        let confidence = if issues.is_empty() { 1.0 } else { 0.7 };

        Ok(SpecValidation {
            is_valid: !has_errors,
            file_path: file_name.to_string(),
            scenario_count,
            issues,
            confidence,
            scenario_metas,
            governance: document.governance,
        })
    }

    /// Validates a Gherkin specification from a file.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be read or validated.
    pub fn validate_file(&self, path: impl AsRef<Path>) -> Result<SpecValidation, ValidationError> {
        let path = path.as_ref();
        let content =
            std::fs::read_to_string(path).map_err(|e| ValidationError::IoError(format!("{e}")))?;

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        self.validate(&content, file_name)
    }

    /// Validates a single scenario.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if LLM API calls fail (wrapped as `NOT_RELATED_ERROR`).
    /// Gherkin validation issues are returned as `ValidationIssue` items, not errors.
    fn validate_scenario(
        &self,
        feature: &gherkin::Feature,
        scenario: &gherkin::Scenario,
    ) -> Result<Vec<ValidationIssue>, ValidationError> {
        let mut issues = Vec::new();

        // Check business sense if enabled
        if self.config.check_business_sense {
            match self.check_business_sense(feature, scenario) {
                Ok(Some(issue)) => issues.push(issue),
                Ok(None) => {} // No issue found
                Err(e) => {
                    // LLM errors are not Gherkin validation issues - propagate as error
                    return Err(e);
                }
            }
        }

        // Check compilability if enabled
        if self.config.check_compilability {
            match self.check_compilability(feature, scenario) {
                Ok(Some(issue)) => issues.push(issue),
                Ok(None) => {} // No issue found
                Err(e) => {
                    // LLM errors are not Gherkin validation issues - propagate as error
                    return Err(e);
                }
            }
        }

        // Check conventions if enabled (no LLM, so no errors possible)
        if self.config.check_conventions {
            issues.extend(self.check_conventions(scenario));
        }

        Ok(issues)
    }

    /// Validates the overall feature structure.
    fn validate_feature(
        &self,
        feature: &gherkin::Feature,
        governance: &TruthGovernance,
    ) -> Result<Vec<ValidationIssue>, ValidationError> {
        let mut issues = Vec::new();

        // Check that the feature has a description
        if feature.description.is_none() {
            issues.push(ValidationIssue {
                location: "Feature".to_string(),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: "Feature lacks a description".to_string(),
                suggestion: Some("Add a description explaining the business purpose".to_string()),
            });
        }

        // Check for empty feature
        if feature.scenarios.is_empty() {
            issues.push(ValidationIssue {
                location: "Feature".to_string(),
                category: IssueCategory::Convention,
                severity: Severity::Error,
                message: "Feature has no scenarios".to_string(),
                suggestion: Some("Add at least one scenario".to_string()),
            });
        }

        if governance.intent.is_none() {
            issues.push(ValidationIssue {
                location: "Truth".to_string(),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: "Truth lacks an Intent block".to_string(),
                suggestion: Some(
                    "Add an Intent block with at least Outcome or Goal for governance context"
                        .to_string(),
                ),
            });
        }

        let mentions_approval = feature.scenarios.iter().any(|scenario| {
            scenario.name.to_ascii_lowercase().contains("approval")
                || scenario.steps.iter().any(|step| {
                    let text = step.value.to_ascii_lowercase();
                    text.contains("approval") || text.contains("approved")
                })
        });

        if mentions_approval && governance.authority.is_none() {
            issues.push(ValidationIssue {
                location: "Truth".to_string(),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: "Approval semantics appear in scenarios without an Authority block"
                    .to_string(),
                suggestion: Some(
                    "Add an Authority block to make approval and authorization explicit"
                        .to_string(),
                ),
            });
        }

        Ok(issues)
    }

    /// Uses LLM to check if a scenario makes business sense.
    fn check_business_sense(
        &self,
        feature: &gherkin::Feature,
        scenario: &gherkin::Scenario,
    ) -> Result<Option<ValidationIssue>, ValidationError> {
        let prompt = format!(
            r"You are a business analyst validating Gherkin specifications for a multi-agent AI system called Converge.

Feature: {}
Scenario: {}

Steps:
{}

Evaluate if this scenario makes business sense:
1. Is the precondition (Given) realistic and well-defined?
2. Is the action (When) meaningful and testable?
3. Is the expected outcome (Then) measurable and valuable?

Respond with ONLY one of:
- VALID: if the scenario makes business sense
- INVALID: <reason> if it doesn't make sense
- UNCLEAR: <question> if more context is needed",
            feature.name,
            scenario.name,
            format_steps(&scenario.steps)
        );

        let system_prompt = "You are a strict business requirements validator. Be concise.";

        let response = chat_sync(&self.backend, system_prompt, &prompt, Some(200), Some(0.3))?;

        let content = response.content.trim();

        if content.starts_with("INVALID:") {
            let reason = content.strip_prefix("INVALID:").unwrap_or("").trim();
            Ok(Some(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::BusinessSense,
                severity: Severity::Error,
                message: reason.to_string(),
                suggestion: None,
            }))
        } else if content.starts_with("UNCLEAR:") {
            let question = content.strip_prefix("UNCLEAR:").unwrap_or("").trim();
            Ok(Some(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::BusinessSense,
                severity: Severity::Warning,
                message: format!("Ambiguous: {question}"),
                suggestion: Some("Clarify the scenario requirements".to_string()),
            }))
        } else {
            Ok(None) // VALID
        }
    }

    /// Uses LLM to check if a scenario can be compiled to a Rust invariant.
    fn check_compilability(
        &self,
        feature: &gherkin::Feature,
        scenario: &gherkin::Scenario,
    ) -> Result<Option<ValidationIssue>, ValidationError> {
        let prompt = format!(
            r"You are a Rust developer checking if a Gherkin scenario can be compiled to a runtime invariant.

In Converge, invariants are Rust structs implementing:
```rust
trait Invariant {{
    fn name(&self) -> &str;
    fn class(&self) -> InvariantClass; // Structural, Semantic, or Acceptance
    fn check(&self, ctx: &Context) -> InvariantResult;
}}
```

The Context has typed facts in categories: Seeds, Hypotheses, Strategies, Constraints, Signals, Competitors, Evaluations.

Feature: {}
Scenario: {}
Steps:
{}

Can this scenario be implemented as a Converge Invariant?

Respond with ONLY one of:
- COMPILABLE: <invariant_class> - brief description of implementation
- NOT_COMPILABLE: <reason why it cannot be a runtime check>
- NEEDS_REFACTOR: <suggestion to make it compilable>",
            feature.name,
            scenario.name,
            format_steps(&scenario.steps)
        );

        let system_prompt =
            "You are a Rust expert. Be precise about what can be checked at runtime.";

        let response = chat_sync(&self.backend, system_prompt, &prompt, Some(200), Some(0.3))?;

        let content = response.content.trim();

        if content.starts_with("NOT_COMPILABLE:") {
            let reason = content.strip_prefix("NOT_COMPILABLE:").unwrap_or("").trim();
            Ok(Some(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::Compilability,
                severity: Severity::Error,
                message: format!("Cannot compile to invariant: {reason}"),
                suggestion: None,
            }))
        } else if content.starts_with("NEEDS_REFACTOR:") {
            let suggestion = content.strip_prefix("NEEDS_REFACTOR:").unwrap_or("").trim();
            Ok(Some(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::Compilability,
                severity: Severity::Warning,
                message: "Scenario needs refactoring to be compilable".to_string(),
                suggestion: Some(suggestion.to_string()),
            }))
        } else {
            Ok(None) // COMPILABLE or unrecognized format
        }
    }

    /// Checks scenario against Converge Gherkin conventions (no LLM needed).
    fn check_conventions(&self, scenario: &gherkin::Scenario) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Check scenario naming convention
        if scenario.name.is_empty() {
            issues.push(ValidationIssue {
                location: "Scenario".to_string(),
                category: IssueCategory::Convention,
                severity: Severity::Error,
                message: "Scenario has no name".to_string(),
                suggestion: Some("Add a descriptive name".to_string()),
            });
        }

        // Check for Given/When/Then structure
        let has_given = scenario
            .steps
            .iter()
            .any(|s| matches!(s.ty, gherkin::StepType::Given));
        let has_when = scenario
            .steps
            .iter()
            .any(|s| matches!(s.ty, gherkin::StepType::When));
        let has_then = scenario
            .steps
            .iter()
            .any(|s| matches!(s.ty, gherkin::StepType::Then));

        if !has_given && !has_when {
            issues.push(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::Convention,
                severity: Severity::Warning,
                message: "Scenario lacks Given or When steps".to_string(),
                suggestion: Some("Add preconditions (Given) or actions (When)".to_string()),
            });
        }

        if !has_then {
            issues.push(ValidationIssue {
                location: format!("Scenario: {}", scenario.name),
                category: IssueCategory::Convention,
                severity: Severity::Error,
                message: "Scenario lacks Then steps (expected outcomes)".to_string(),
                suggestion: Some(
                    "Add at least one Then step defining the expected outcome".to_string(),
                ),
            });
        }

        // Check for Converge-specific patterns
        for step in &scenario.steps {
            if step.value.contains("should") && matches!(step.ty, gherkin::StepType::Then) {
                // Good pattern: "Then X should Y"
            } else if step.value.contains("must") || step.value.contains("always") {
                // Good pattern for invariants
            } else if step.value.contains("might") || step.value.contains("maybe") {
                issues.push(ValidationIssue {
                    location: format!("Step: {}", step.value),
                    category: IssueCategory::Convention,
                    severity: Severity::Warning,
                    message: "Uncertain language in step ('might', 'maybe')".to_string(),
                    suggestion: Some("Use definite language for testable assertions".to_string()),
                });
            }
        }

        issues
    }
}

/// LLM-powered Gherkin specification generator.
pub struct SpecGenerator {
    backend: Arc<dyn DynChatBackend>,
}

impl std::fmt::Debug for SpecGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpecGenerator").finish_non_exhaustive()
    }
}

impl SpecGenerator {
    /// Creates a new generator with the given chat backend.
    #[must_use]
    pub fn new(backend: Arc<dyn DynChatBackend>) -> Self {
        Self { backend }
    }

    /// Generates a Gherkin/Truth specification from free text.
    ///
    /// # Errors
    ///
    /// Returns error if the LLM API call fails.
    pub fn generate_from_text(&self, text: &str) -> Result<String, ValidationError> {
        let prompt = format!(
            r"You are a requirements engineer for a multi-agent AI system called Converge.
Convert the following free text into a valid Gherkin/Truth specification.

Free Text:
{text}

Rules for generation:
1. Use Converge Truth syntax (`Truth:` instead of `Feature:`).
2. Include a concise business description immediately after the Truth header.
3. Ensure at least one scenario is generated.
4. Each scenario must have Given/When/Then steps.
5. Use definite language (avoid 'might', 'maybe').
6. Focus on testable business outcomes.

Return ONLY the Gherkin content, no explanation or preamble.

Example Format:
Truth: <name>
  <description line 1>
  <description line 2>

  Scenario: <name>
    Given <state>
    When <action>
    Then <outcome>"
        );

        let system_prompt =
            "You are an expert Gherkin spec writer. Respond with ONLY the specification.";

        let response = chat_sync(&self.backend, system_prompt, &prompt, Some(1000), Some(0.3))?;

        Ok(response.content.trim().to_string())
    }
}

/// Synchronously execute a chat request against a `DynChatBackend`.
fn chat_sync(
    backend: &Arc<dyn DynChatBackend>,
    system: &str,
    user_prompt: &str,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
) -> Result<ChatResponse, ValidationError> {
    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: ChatRole::User,
            content: user_prompt.to_string(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }],
        system: Some(system.to_string()),
        tools: Vec::new(),
        response_format: Default::default(),
        max_tokens,
        temperature,
        stop_sequences: Vec::new(),
        model: None,
    };

    futures::executor::block_on(backend.chat(request)).map_err(|e| {
        ValidationError::LlmError(format!("NOT_RELATED_ERROR: LLM API call failed: {e}"))
    })
}

/// Formats Gherkin steps for display.
fn format_steps(steps: &[gherkin::Step]) -> String {
    steps
        .iter()
        .map(|s| format!("{:?} {}", s.keyword, s.value))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Error during Gherkin validation.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Failed to parse the Gherkin file.
    ParseError(String),
    /// IO error reading file.
    IoError(String),
    /// LLM call failed.
    LlmError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
            Self::IoError(msg) => write!(f, "IO error: {msg}"),
            Self::LlmError(msg) => write!(f, "LLM error: {msg}"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_llm::StaticChatBackend;

    fn mock_valid_backend() -> Arc<dyn DynChatBackend> {
        Arc::new(StaticChatBackend::queued([
            "VALID",
            "COMPILABLE: Acceptance - check strategy count",
        ]))
    }

    #[test]
    fn preprocess_converts_truth_to_feature() {
        let input = "Truth: Get paid for delivered work\n  Scenario: Invoice";
        let output = preprocess_truths(input);
        assert!(output.starts_with("Feature:"));
        assert!(output.contains("Scenario: Invoice"));
    }

    #[test]
    fn preprocess_preserves_feature_keyword() {
        let input = "Feature: Standard Gherkin\n  Scenario: Test";
        let output = preprocess_truths(input);
        assert_eq!(input, output);
    }

    #[test]
    fn validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.check_conventions);
        assert!(config.check_business_sense);
        assert!(config.check_compilability);
        assert_eq!(config.min_confidence, 0.7);
    }

    #[test]
    fn validation_config_custom() {
        let config = ValidationConfig {
            check_business_sense: false,
            min_confidence: 0.9,
            ..ValidationConfig::default()
        };
        assert!(!config.check_business_sense);
        assert_eq!(config.min_confidence, 0.9);
        assert!(config.check_conventions);
    }

    #[test]
    fn validates_truth_syntax() {
        let content = r"
Truth: Get paid for delivered work
  Scenario: Invoice and collect
    Given work is marked as delivered
    When the system converges
    Then invoice is issued
";

        let validator = GherkinValidator::new(mock_valid_backend(), ValidationConfig::default());

        let result = validator.validate(content, "money.truth").unwrap();

        assert_eq!(result.scenario_count, 1);
        // Should parse successfully with Truth: syntax and .truth extension
    }

    #[test]
    fn validates_truth_with_governance_blocks() {
        let content = r#"
Truth: Governed release
  Intent:
    Outcome: "Ship safely"

  Authority:
    Actor: release_manager
    Requires Approval: security_owner

  Evidence:
    Requires: test_report

  Scenario: Approval is required before release
    Given a release candidate exists
    When the system converges
    Then deployment MUST NOT occur without approval
"#;

        let validator = GherkinValidator::new(mock_valid_backend(), ValidationConfig::default());
        let result = validator.validate(content, "release.truths").unwrap();

        assert_eq!(result.scenario_count, 1);
        assert_eq!(
            result.governance.authority.unwrap().requires_approval,
            vec!["security_owner".to_string()]
        );
        assert!(result.governance.intent.is_some());
    }

    #[test]
    fn validates_simple_feature() {
        let content = r"
Feature: Growth Strategy Validation
  Scenario: Multiple strategies required
    When the system converges
    Then at least two distinct growth strategies exist
";

        let validator = GherkinValidator::new(mock_valid_backend(), ValidationConfig::default());

        let result = validator.validate(content, "test.feature").unwrap();

        assert_eq!(result.scenario_count, 1);
        // May have convention warnings but should be parseable
    }

    #[test]
    fn detects_missing_then() {
        let content = r"
Feature: Bad Spec
  Scenario: No assertions
    Given some precondition
    When something happens
";

        let validator = GherkinValidator::new(
            mock_valid_backend(),
            ValidationConfig {
                check_business_sense: false,
                check_compilability: false,
                check_conventions: true,
                min_confidence: 0.7,
            },
        );

        let result = validator.validate(content, "bad.feature").unwrap();

        assert!(result.has_errors());
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.category == IssueCategory::Convention && i.message.contains("Then"))
        );
    }

    #[test]
    fn detects_uncertain_language() {
        let content = r"
Feature: Uncertain Spec
  Scenario: Maybe works
    When something happens
    Then it might succeed
";

        let validator = GherkinValidator::new(
            mock_valid_backend(),
            ValidationConfig {
                check_business_sense: false,
                check_compilability: false,
                check_conventions: true,
                min_confidence: 0.7,
            },
        );

        let result = validator.validate(content, "uncertain.feature").unwrap();

        assert!(result.has_warnings());
        assert!(result.issues.iter().any(|i| i.message.contains("might")));
    }

    #[test]
    fn handles_llm_invalid_response() {
        let backend: Arc<dyn DynChatBackend> = Arc::new(StaticChatBackend::queued([
            "INVALID: The scenario describes an untestable state",
            "COMPILABLE: Acceptance",
        ]));

        let content = r"
Feature: Test
  Scenario: Bad business logic
    When magic happens
    Then everything is perfect forever
";

        let validator = GherkinValidator::new(backend, ValidationConfig::default());

        let result = validator.validate(content, "test.feature").unwrap();

        assert!(
            result.issues.iter().any(
                |i| i.category == IssueCategory::BusinessSense && i.severity == Severity::Error
            )
        );
    }

    #[test]
    fn generates_spec_from_text() {
        let mock_spec = "Truth: Test\n  Scenario: Test\n    Given X\n    Then Y";
        let backend: Arc<dyn DynChatBackend> = Arc::new(StaticChatBackend::queued([mock_spec]));

        let generator = SpecGenerator::new(backend);
        let result = generator.generate_from_text("Make a test spec").unwrap();

        assert_eq!(result, mock_spec);
    }

    // =========================================================================
    // Tag Extraction Tests
    // =========================================================================

    #[test]
    fn extract_invariant_structural_tags() {
        let tags = vec![
            "invariant".to_string(),
            "structural".to_string(),
            "id:brand_safety".to_string(),
        ];
        let meta = extract_scenario_meta("Strategies must not contain brand-unsafe terms", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
        assert_eq!(meta.invariant_class, Some(InvariantClassTag::Structural));
        assert_eq!(meta.id.as_deref(), Some("brand_safety"));
        assert!(!meta.is_test);
        assert!(meta.provider.is_none());
    }

    #[test]
    fn extract_invariant_acceptance_tags() {
        let tags = vec![
            "invariant".to_string(),
            "acceptance".to_string(),
            "id:require_multiple_strategies".to_string(),
        ];
        let meta = extract_scenario_meta("At least 2 strategies must exist", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
        assert_eq!(meta.invariant_class, Some(InvariantClassTag::Acceptance));
        assert_eq!(meta.id.as_deref(), Some("require_multiple_strategies"));
    }

    #[test]
    fn extract_invariant_semantic_tags() {
        let tags = vec![
            "invariant".to_string(),
            "semantic".to_string(),
            "id:require_evaluation_rationale".to_string(),
        ];
        let meta = extract_scenario_meta("Evaluations must include score", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
        assert_eq!(meta.invariant_class, Some(InvariantClassTag::Semantic));
        assert_eq!(meta.id.as_deref(), Some("require_evaluation_rationale"));
    }

    #[test]
    fn extract_validation_tags() {
        let tags = vec![
            "validation".to_string(),
            "id:confidence_threshold".to_string(),
        ];
        let meta = extract_scenario_meta("Proposals must meet confidence threshold", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Validation));
        assert!(meta.invariant_class.is_none());
        assert_eq!(meta.id.as_deref(), Some("confidence_threshold"));
    }

    #[test]
    fn extract_agent_llm_tags() {
        let tags = vec![
            "agent".to_string(),
            "llm".to_string(),
            "id:market_signal".to_string(),
        ];
        let meta = extract_scenario_meta("Market Signal agent proposes Signals", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Suggestor));
        assert_eq!(meta.provider.as_deref(), Some("llm"));
        assert_eq!(meta.id.as_deref(), Some("market_signal"));
    }

    #[test]
    fn extract_e2e_test_tags() {
        let tags = vec!["e2e".to_string(), "test".to_string()];
        let meta = extract_scenario_meta("Pack converges from Seeds", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::EndToEnd));
        assert!(meta.is_test);
        assert!(meta.id.is_none());
    }

    #[test]
    fn extract_with_at_prefix() {
        // gherkin crate may include @ prefix — we handle both
        let tags = vec![
            "@invariant".to_string(),
            "@structural".to_string(),
            "@id:brand_safety".to_string(),
        ];
        let meta = extract_scenario_meta("Test with @ prefix", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
        assert_eq!(meta.invariant_class, Some(InvariantClassTag::Structural));
        assert_eq!(meta.id.as_deref(), Some("brand_safety"));
    }

    #[test]
    fn extract_no_tags() {
        let meta = extract_scenario_meta("Untagged scenario", &[]);

        assert!(meta.kind.is_none());
        assert!(meta.invariant_class.is_none());
        assert!(meta.id.is_none());
        assert!(!meta.is_test);
    }

    #[test]
    fn extract_unknown_tags_preserved() {
        let tags = vec![
            "custom".to_string(),
            "invariant".to_string(),
            "id:test".to_string(),
        ];
        let meta = extract_scenario_meta("With custom tag", &tags);

        assert_eq!(meta.raw_tags.len(), 3);
        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
    }

    #[test]
    fn extract_all_metas_from_truth_file() {
        let content = r#"
Truth: Growth Strategy Pack
  Multi-agent growth strategy analysis.

  @invariant @structural @id:brand_safety
  Scenario: Strategies must not contain brand-unsafe terms
    Given any fact under key "Strategies"
    Then it must not contain forbidden terms

  @invariant @acceptance @id:require_multiple_strategies
  Scenario: At least 2 strategies must exist at convergence
    Given the engine halts with reason "Converged"
    Then the Context key "Strategies" contains at least 2 facts

  @agent @llm @id:market_signal
  Scenario: Market Signal agent proposes Signals from Seeds
    Given the Context contains facts under key "Seeds"
    When agent "market_signal" executes
    Then it proposes facts under key "Signals"

  @e2e @test
  Scenario: Pack converges from Seeds to evaluated Strategies
    Given seed facts are present
    When the pack runs to convergence
    Then all invariants pass
"#;

        let metas = extract_all_metas(content).unwrap();
        assert_eq!(metas.len(), 4);

        // First: structural invariant
        assert_eq!(metas[0].kind, Some(ScenarioKind::Invariant));
        assert_eq!(
            metas[0].invariant_class,
            Some(InvariantClassTag::Structural)
        );
        assert_eq!(metas[0].id.as_deref(), Some("brand_safety"));

        // Second: acceptance invariant
        assert_eq!(metas[1].kind, Some(ScenarioKind::Invariant));
        assert_eq!(
            metas[1].invariant_class,
            Some(InvariantClassTag::Acceptance)
        );
        assert_eq!(metas[1].id.as_deref(), Some("require_multiple_strategies"));

        // Third: agent
        assert_eq!(metas[2].kind, Some(ScenarioKind::Suggestor));
        assert_eq!(metas[2].provider.as_deref(), Some("llm"));

        // Fourth: e2e test
        assert_eq!(metas[3].kind, Some(ScenarioKind::EndToEnd));
        assert!(metas[3].is_test);
    }

    #[test]
    fn validator_populates_scenario_metas() {
        let content = r#"
Truth: Test
  @invariant @structural @id:test_inv
  Scenario: Test invariant
    Given precondition
    When action occurs
    Then outcome is verified
"#;

        let validator = GherkinValidator::new(mock_valid_backend(), ValidationConfig::default());
        let result = validator.validate(content, "test.truth").unwrap();

        assert_eq!(result.scenario_metas.len(), 1);
        assert_eq!(result.scenario_metas[0].kind, Some(ScenarioKind::Invariant));
        assert_eq!(result.scenario_metas[0].id.as_deref(), Some("test_inv"));
    }

    // =========================================================================
    // Negative tests for tag extraction
    // =========================================================================

    #[test]
    fn extract_meta_invariant_without_class() {
        // Invariant without a class tag should still be recognized as invariant
        let tags = vec!["invariant".to_string(), "id:no_class".to_string()];
        let meta = extract_scenario_meta("Invariant without class", &tags);

        assert_eq!(meta.kind, Some(ScenarioKind::Invariant));
        assert!(meta.invariant_class.is_none()); // no class is valid — pipeline may warn later
    }

    #[test]
    fn extract_meta_class_without_kind() {
        // Class without invariant kind — should preserve class but no kind
        let tags = vec!["structural".to_string()];
        let meta = extract_scenario_meta("Orphan class", &tags);

        assert!(meta.kind.is_none());
        assert_eq!(meta.invariant_class, Some(InvariantClassTag::Structural));
    }

    #[test]
    fn extract_meta_empty_id() {
        // @id: with no value after colon
        let tags = vec!["invariant".to_string(), "id:".to_string()];
        let meta = extract_scenario_meta("Empty id", &tags);

        assert_eq!(meta.id.as_deref(), Some(""));
    }

    #[test]
    fn extract_all_metas_parse_error() {
        let bad = "This is not valid Gherkin at all";
        let result = extract_all_metas(bad);
        assert!(result.is_err());
    }

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn preprocess_never_crashes(s in "\\PC*") {
                let _ = preprocess_truths(&s);
            }

            #[test]
            fn truth_to_feature_conversion(s in ".*Truth:.*") {
                let _output = preprocess_truths(&s);
                // If the line started with Truth:, it should now start with Feature:
                // Note: preprocess_truths uses Regex with (?m)^(\s*)Truth:
                // We should check if the conversion happened for lines meeting the pattern
            }

            #[test]
            fn idempotency_of_feature(s in ".*Feature:.*") {
                // If it already has Feature:, it shouldn't change to something else incorrect
                // specifically, it shouldn't contain "Truth:" where "Feature:" was
                if !s.contains("Truth:") {
                    let output = preprocess_truths(&s);
                    assert_eq!(s, output);
                }
            }

            #[test]
            fn extract_meta_never_crashes(
                name in "\\PC{0,100}",
                tags in proptest::collection::vec("[a-z:_@]{1,30}", 0..10)
            ) {
                let _ = extract_scenario_meta(&name, &tags);
            }

            #[test]
            fn extract_meta_preserves_all_raw_tags(
                tags in proptest::collection::vec("[a-z]{1,10}", 0..5)
            ) {
                let meta = extract_scenario_meta("test", &tags);
                assert_eq!(meta.raw_tags.len(), tags.len());
            }

            #[test]
            fn extract_meta_id_always_from_id_prefix(
                suffix in "[a-z_]{1,20}"
            ) {
                let tags = vec![format!("id:{suffix}")];
                let meta = extract_scenario_meta("test", &tags);
                assert_eq!(meta.id.as_deref(), Some(suffix.as_str()));
            }
        }
    }
}
