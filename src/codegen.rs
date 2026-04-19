// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Code generation for WASM invariant modules from Gherkin predicates.
//!
//! Converts parsed Gherkin predicates and scenario metadata into Rust source
//! code for WASM modules conforming to the Converge WASM ABI v1.
//!
//! # Pipeline
//!
//! ```text
//! ScenarioMeta + JTBDMetadata + Vec<Predicate>
//!     │
//!     ▼
//! ManifestBuilder.build() → manifest JSON string
//!     │
//!     ▼
//! generate_invariant_module() → Rust source code string
//!     │
//!     ▼
//! [Task #5: compilation] → .wasm bytes
//! ```
//!
//! # Manifest Builder
//!
//! [`ManifestBuilder`] assembles a WasmManifest-compatible JSON string from
//! Gherkin scenario tags ([`ScenarioMeta`]), JTBD metadata, and parsed
//! predicates. The JSON conforms to the `converge-runtime` `WasmManifest`
//! schema without requiring a direct crate dependency.
//!
//! # Code Generator
//!
//! [`generate_invariant_module`] produces a complete Rust source file that
//! exports the WASM ABI v1 functions (`converge_abi_version`, `converge_manifest`,
//! `alloc`, `dealloc`, `check_invariant`). Each [`Predicate`] is compiled to
//! a Rust check expression inside the generated `check()` function.

use std::collections::HashMap;

use serde::Serialize;

use crate::gherkin::{InvariantClassTag, ScenarioKind, ScenarioMeta};
use crate::jtbd::JTBDMetadata;
use crate::predicate::{Predicate, extract_dependencies};
use crate::truths::TruthGovernance;

// ============================================================================
// Manifest Builder (Task #4)
// ============================================================================

/// Error during manifest construction.
#[derive(Debug, Clone)]
pub enum ManifestError {
    /// Invariant module must declare an invariant class.
    MissingInvariantClass,
    /// Suggestor module must have at least one dependency.
    MissingDependencies,
    /// Module name could not be determined from tags or scenario name.
    MissingName,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInvariantClass => write!(
                f,
                "invariant module must declare an invariant class tag \
                 (@structural, @semantic, @acceptance)"
            ),
            Self::MissingDependencies => {
                write!(f, "agent module must reference at least one context key")
            }
            Self::MissingName => {
                write!(f, "module name could not be determined (use @id:name tag)")
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// JSON structure matching the `converge-runtime` `WasmManifest` schema.
///
/// Serialized with serde to produce JSON that the runtime's contract
/// types can deserialize without modification.
#[derive(Debug, Clone, Serialize)]
struct ManifestJson {
    name: String,
    version: String,
    kind: String,
    invariant_class: Option<String>,
    dependencies: Vec<String>,
    capabilities: Vec<String>,
    requires_human_approval: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    jtbd: Option<JtbdRefJson>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    metadata: HashMap<String, String>,
}

/// JTBD reference matching the `converge-runtime` `JtbdRef` schema.
#[derive(Debug, Clone, Serialize)]
struct JtbdRefJson {
    truth_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    job_functional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_hash: Option<String>,
}

/// Builder for constructing a WasmManifest-compatible JSON string.
///
/// Collects metadata from scenario tags, JTBD blocks, and parsed predicates
/// to produce a complete manifest for embedding in generated WASM modules.
///
/// # Examples
///
/// ```
/// use axiom_truth::codegen::ManifestBuilder;
/// use axiom_truth::gherkin::{ScenarioMeta, ScenarioKind, InvariantClassTag};
///
/// let meta = ScenarioMeta {
///     name: "Brand Safety".to_string(),
///     kind: Some(ScenarioKind::Invariant),
///     invariant_class: Some(InvariantClassTag::Structural),
///     id: Some("brand_safety".to_string()),
///     provider: None,
///     is_test: false,
///     raw_tags: vec![],
/// };
///
/// let json = ManifestBuilder::new()
///     .from_scenario_meta(&meta)
///     .build()
///     .unwrap();
///
/// assert!(json.contains("brand_safety"));
/// assert!(json.contains("Structural"));
/// ```
#[derive(Debug)]
pub struct ManifestBuilder {
    name: Option<String>,
    version: String,
    kind: Option<String>,
    invariant_class: Option<String>,
    dependencies: Vec<String>,
    capabilities: Vec<String>,
    requires_human_approval: bool,
    jtbd_truth_id: Option<String>,
    jtbd_actor: Option<String>,
    jtbd_job_functional: Option<String>,
    jtbd_source_hash: Option<String>,
    metadata: HashMap<String, String>,
}

impl ManifestBuilder {
    /// Create a new empty manifest builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            version: "0.1.0".to_string(),
            kind: None,
            invariant_class: None,
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            requires_human_approval: false,
            jtbd_truth_id: None,
            jtbd_actor: None,
            jtbd_job_functional: None,
            jtbd_source_hash: None,
            metadata: HashMap::new(),
        }
    }

    /// Populate from extracted scenario metadata (tags).
    ///
    /// Sets kind, invariant class, and name from the scenario's parsed tags.
    /// The `@id:<value>` tag becomes the module name; if absent, the scenario
    /// name is sanitized to a valid identifier.
    #[must_use]
    pub fn from_scenario_meta(mut self, meta: &ScenarioMeta) -> Self {
        if let Some(kind) = meta.kind {
            self.kind = Some(
                match kind {
                    ScenarioKind::Invariant | ScenarioKind::Validation | ScenarioKind::EndToEnd => {
                        "Invariant"
                    }
                    ScenarioKind::Suggestor => "Suggestor",
                }
                .to_string(),
            );
        }

        if let Some(class) = meta.invariant_class {
            self.invariant_class = Some(
                match class {
                    InvariantClassTag::Structural => "Structural",
                    InvariantClassTag::Semantic => "Semantic",
                    InvariantClassTag::Acceptance => "Acceptance",
                }
                .to_string(),
            );
        }

        if let Some(ref id) = meta.id {
            self.name = Some(id.clone());
        } else {
            self.name = Some(sanitize_module_name(&meta.name));
        }

        if meta.provider.is_some() && !self.capabilities.contains(&"Log".to_string()) {
            self.capabilities.push("Log".to_string());
        }

        self
    }

    /// Populate JTBD metadata from a parsed JTBD block.
    #[must_use]
    pub fn from_jtbd(mut self, jtbd: &JTBDMetadata) -> Self {
        self.jtbd_actor = Some(jtbd.actor.clone());
        self.jtbd_job_functional = Some(jtbd.job_functional.clone());
        self
    }

    /// Infer dependencies and capabilities from parsed predicates.
    ///
    /// Extracts context key references from predicates as dependencies.
    /// Automatically adds `ReadContext` capability when dependencies exist.
    #[must_use]
    pub fn from_predicates(mut self, predicates: &[Predicate]) -> Self {
        self.dependencies = extract_dependencies(predicates);
        if !self.dependencies.is_empty() && !self.capabilities.contains(&"ReadContext".to_string())
        {
            self.capabilities.insert(0, "ReadContext".to_string());
        }
        self
    }

    /// Populate manifest metadata from Converge Truths governance blocks.
    #[must_use]
    pub fn from_truth_governance(mut self, governance: &TruthGovernance) -> Self {
        if let Some(intent) = &governance.intent {
            if let Some(outcome) = &intent.outcome {
                self.metadata
                    .insert("intent.outcome".to_string(), outcome.clone());
            }
            if let Some(goal) = &intent.goal {
                self.metadata
                    .insert("intent.goal".to_string(), goal.clone());
            }
        }

        if let Some(authority) = &governance.authority {
            if let Some(actor) = &authority.actor {
                self.metadata
                    .insert("authority.actor".to_string(), actor.clone());
            }
            if let Some(expires) = &authority.expires {
                self.metadata
                    .insert("authority.expires".to_string(), expires.clone());
            }
            if !authority.may.is_empty() {
                self.metadata
                    .insert("authority.may".to_string(), authority.may.join(" | "));
            }
            if !authority.must_not.is_empty() {
                self.metadata.insert(
                    "authority.must_not".to_string(),
                    authority.must_not.join(" | "),
                );
            }
            if !authority.requires_approval.is_empty() {
                self.requires_human_approval = true;
                self.metadata.insert(
                    "authority.requires_approval".to_string(),
                    authority.requires_approval.join(" | "),
                );
            }
        }

        if let Some(constraint) = &governance.constraint {
            if !constraint.budget.is_empty() {
                self.metadata.insert(
                    "constraint.budget".to_string(),
                    constraint.budget.join(" | "),
                );
            }
            if !constraint.cost_limit.is_empty() {
                self.metadata.insert(
                    "constraint.cost_limit".to_string(),
                    constraint.cost_limit.join(" | "),
                );
            }
            if !constraint.must_not.is_empty() {
                self.metadata.insert(
                    "constraint.must_not".to_string(),
                    constraint.must_not.join(" | "),
                );
            }
        }

        if let Some(evidence) = &governance.evidence {
            if !evidence.requires.is_empty() {
                self.metadata.insert(
                    "evidence.requires".to_string(),
                    evidence.requires.join(" | "),
                );
            }
            if !evidence.provenance.is_empty() {
                self.metadata.insert(
                    "evidence.provenance".to_string(),
                    evidence.provenance.join(" | "),
                );
            }
            if !evidence.audit.is_empty() {
                self.metadata
                    .insert("evidence.audit".to_string(), evidence.audit.join(" | "));
            }
        }

        if let Some(exception) = &governance.exception {
            if !exception.escalates_to.is_empty() {
                self.requires_human_approval = true;
                self.metadata.insert(
                    "exception.escalates_to".to_string(),
                    exception.escalates_to.join(" | "),
                );
            }
            if !exception.requires.is_empty() {
                self.metadata.insert(
                    "exception.requires".to_string(),
                    exception.requires.join(" | "),
                );
            }
        }

        self
    }

    /// Set the module version.
    #[must_use]
    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    /// Set the source hash (SHA-256 of the `.truth` file content).
    #[must_use]
    pub fn with_source_hash(mut self, hash: &str) -> Self {
        self.jtbd_source_hash = Some(hash.to_string());
        self
    }

    /// Set the truth file ID.
    #[must_use]
    pub fn with_truth_id(mut self, id: &str) -> Self {
        self.jtbd_truth_id = Some(id.to_string());
        self
    }

    /// Build the manifest JSON string.
    ///
    /// # Errors
    ///
    /// Returns `ManifestError::MissingInvariantClass` if kind is Invariant
    /// but no class tag was provided.
    ///
    /// Returns `ManifestError::MissingDependencies` if kind is Suggestor but
    /// no context key dependencies were found.
    ///
    /// Returns `ManifestError::MissingName` if no name could be determined.
    pub fn build(self) -> Result<String, ManifestError> {
        let name = self.name.ok_or(ManifestError::MissingName)?;
        let kind = self.kind.unwrap_or_else(|| "Invariant".to_string());

        if kind == "Invariant" && self.invariant_class.is_none() {
            return Err(ManifestError::MissingInvariantClass);
        }
        if kind == "Suggestor" && self.dependencies.is_empty() {
            return Err(ManifestError::MissingDependencies);
        }

        let jtbd = if self.jtbd_actor.is_some() || self.jtbd_truth_id.is_some() {
            Some(JtbdRefJson {
                truth_id: self.jtbd_truth_id.unwrap_or_default(),
                actor: self.jtbd_actor,
                job_functional: self.jtbd_job_functional,
                source_hash: self.jtbd_source_hash,
            })
        } else {
            None
        };

        let manifest = ManifestJson {
            name,
            version: self.version,
            kind,
            invariant_class: self.invariant_class,
            dependencies: self.dependencies,
            capabilities: self.capabilities,
            requires_human_approval: self.requires_human_approval,
            jtbd,
            metadata: self.metadata,
        };

        Ok(serde_json::to_string(&manifest).expect("ManifestJson serialization cannot fail"))
    }
}

impl Default for ManifestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a scenario name into a valid Rust/module identifier.
///
/// Converts to lowercase, replaces non-alphanumeric characters with
/// underscores, and trims leading/trailing underscores.
pub(crate) fn sanitize_module_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim_matches('_').to_string()
}

// ============================================================================
// Code Generation (Task #3)
// ============================================================================

/// Configuration for code generation.
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Manifest JSON string to embed in the generated module.
    pub manifest_json: String,
    /// Module name (used in doc comments).
    pub module_name: String,
}

/// Generate a complete Rust source file for a WASM invariant module.
///
/// The generated source includes:
/// - Inline guest types (`GuestContext`, `GuestFact`, `GuestInvariantResult`)
/// - Bump allocator (`alloc`/`dealloc` exports)
/// - ABI version export (`converge_abi_version`)
/// - Manifest export (`converge_manifest` with embedded JSON)
/// - `check_invariant` export with generated predicate checks
///
/// # Examples
///
/// ```
/// use axiom_truth::codegen::{generate_invariant_module, CodegenConfig};
/// use axiom_truth::predicate::Predicate;
///
/// let config = CodegenConfig {
///     manifest_json: r#"{"name":"test","version":"1.0.0","kind":"Invariant","invariant_class":"Structural","dependencies":[],"capabilities":[],"requires_human_approval":false}"#.to_string(),
///     module_name: "test_invariant".to_string(),
/// };
///
/// let source = generate_invariant_module(&config, &[
///     Predicate::CountAtLeast { key: "Strategies".to_string(), min: 2 },
/// ]);
///
/// assert!(source.contains("check_invariant"));
/// assert!(source.contains("count < 2"));
/// ```
pub fn generate_invariant_module(config: &CodegenConfig, predicates: &[Predicate]) -> String {
    let checks = generate_check_body(predicates);
    let manifest_literal = format_raw_string(&config.manifest_json);

    let mut s = String::with_capacity(4096);

    // Module header
    s.push_str("//! Auto-generated Converge WASM invariant module.\n");
    s.push_str(&format!("//! Module: {}\n", config.module_name));
    s.push_str("//!\n");
    s.push_str("//! Generated by axiom-truth. Do not edit manually.\n\n");

    // Inline guest types
    s.push_str(GUEST_TYPES);

    // Manifest constant
    s.push_str("const MANIFEST_JSON: &str = ");
    s.push_str(&manifest_literal);
    s.push_str(";\n\n");

    // Allocator
    s.push_str(ALLOCATOR_CODE);

    // ABI exports
    s.push_str(ABI_EXPORTS);

    // Check function with generated predicates
    s.push_str("fn check(ctx: &GuestContext) -> GuestInvariantResult {\n");
    s.push_str(&checks);
    s.push_str("    GuestInvariantResult {\n");
    s.push_str("        ok: true,\n");
    s.push_str("        reason: None,\n");
    s.push_str("        fact_ids: Vec::new(),\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");

    // check_invariant export wrapper
    s.push_str(CHECK_INVARIANT_WRAPPER);

    s
}

// ---------------------------------------------------------------------------
// Static template fragments
// ---------------------------------------------------------------------------

const GUEST_TYPES: &str = r#"use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct GuestContext {
    facts: HashMap<String, Vec<GuestFact>>,
    #[allow(dead_code)]
    version: u64,
    #[allow(dead_code)]
    cycle: u32,
}

#[derive(serde::Deserialize)]
struct GuestFact {
    id: String,
    content: String,
}

#[derive(serde::Serialize)]
struct GuestInvariantResult {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    fact_ids: Vec<String>,
}

"#;

const ALLOCATOR_CODE: &str = r#"static BUMP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let prev = BUMP.fetch_add(size as usize, std::sync::atomic::Ordering::SeqCst);
    prev as i32
}

#[no_mangle]
pub extern "C" fn dealloc(_ptr: i32, _len: i32) {
    // Bump allocator: dealloc is a no-op
}

"#;

const ABI_EXPORTS: &str = r#"#[no_mangle]
pub extern "C" fn converge_abi_version() -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn converge_manifest() -> (i32, i32) {
    (MANIFEST_JSON.as_ptr() as i32, MANIFEST_JSON.len() as i32)
}

"#;

const CHECK_INVARIANT_WRAPPER: &str = r#"#[no_mangle]
pub extern "C" fn check_invariant(ctx_ptr: i32, ctx_len: i32) -> (i32, i32) {
    let ctx_bytes = unsafe {
        core::slice::from_raw_parts(ctx_ptr as *const u8, ctx_len as usize)
    };
    let ctx: GuestContext = match serde_json::from_slice(ctx_bytes) {
        Ok(c) => c,
        Err(e) => {
            return write_result(&GuestInvariantResult {
                ok: false,
                reason: Some(format!("failed to parse context: {}", e)),
                fact_ids: Vec::new(),
            });
        }
    };

    let result = check(&ctx);
    write_result(&result)
}

fn write_result(result: &GuestInvariantResult) -> (i32, i32) {
    let json = serde_json::to_vec(result).expect("serialize result");
    let ptr = alloc(json.len() as i32);
    unsafe {
        core::ptr::copy_nonoverlapping(json.as_ptr(), ptr as *mut u8, json.len());
    }
    (ptr, json.len() as i32)
}
"#;

// ---------------------------------------------------------------------------
// Check body generation
// ---------------------------------------------------------------------------

/// Generate the body of the `check()` function from predicates.
fn generate_check_body(predicates: &[Predicate]) -> String {
    if predicates.is_empty() {
        return "    // No predicates — invariant always holds\n".to_string();
    }

    let mut body = String::new();
    for (i, pred) in predicates.iter().enumerate() {
        body.push_str(&format!(
            "    // Check {}: {}\n",
            i + 1,
            predicate_summary(pred)
        ));
        body.push_str(&predicate_to_rust(pred));
        body.push('\n');
    }
    body
}

/// One-line summary of a predicate for code comments.
fn predicate_summary(pred: &Predicate) -> String {
    match pred {
        Predicate::CountAtLeast { key, min } => {
            format!("{key} must have at least {min} facts")
        }
        Predicate::CountAtMost { key, max } => {
            format!("{key} must have at most {max} facts")
        }
        Predicate::ContentMustNotContain { key, forbidden } => {
            format!("{key} must not contain {} forbidden terms", forbidden.len())
        }
        Predicate::ContentMustContain {
            key,
            required_field,
        } => {
            format!("{key} facts must contain field '{required_field}'")
        }
        Predicate::CrossReference {
            source_key,
            target_key,
        } => {
            format!("each {source_key} must be referenced by a {target_key}")
        }
        Predicate::HasFacts { key } => format!("{key} must have facts"),
        Predicate::RequiredFields { key, fields } => {
            format!("{key} facts must have {} required fields", fields.len())
        }
        Predicate::Custom { description } => {
            let short = if description.len() > 60 {
                format!("{}...", &description[..60])
            } else {
                description.clone()
            };
            format!("custom: {short}")
        }
    }
}

/// Generate Rust code for a single predicate check.
///
/// Returns a block of Rust code (with leading indentation) that evaluates
/// the predicate against `ctx: &GuestContext` and returns early with a
/// `GuestInvariantResult` violation if the check fails.
fn predicate_to_rust(pred: &Predicate) -> String {
    match pred {
        Predicate::CountAtLeast { key, min } => {
            let key_e = esc(key);
            let mut c = String::new();
            c.push_str("    {\n");
            c.push_str(&format!(
                "        let count = ctx.facts.get(\"{key_e}\").map(|v| v.len()).unwrap_or(0);\n"
            ));
            c.push_str(&format!("        if count < {min} {{\n"));
            c.push_str("            return GuestInvariantResult {\n");
            c.push_str("                ok: false,\n");
            c.push_str(&format!(
                "                reason: Some(format!(\"{key_e} contains {{}} facts, need at least {min}\", count)),\n"
            ));
            c.push_str("                fact_ids: Vec::new(),\n");
            c.push_str("            };\n");
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::CountAtMost { key, max } => {
            let key_e = esc(key);
            let mut c = String::new();
            c.push_str("    {\n");
            c.push_str(&format!(
                "        let count = ctx.facts.get(\"{key_e}\").map(|v| v.len()).unwrap_or(0);\n"
            ));
            c.push_str(&format!("        if count > {max} {{\n"));
            c.push_str("            return GuestInvariantResult {\n");
            c.push_str("                ok: false,\n");
            c.push_str(&format!(
                "                reason: Some(format!(\"{key_e} contains {{}} facts, max is {max}\", count)),\n"
            ));
            c.push_str("                fact_ids: Vec::new(),\n");
            c.push_str("            };\n");
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::ContentMustNotContain { key, forbidden } => {
            let key_e = esc(key);
            let mut c = String::new();
            c.push_str(&format!(
                "    if let Some(facts) = ctx.facts.get(\"{key_e}\") {{\n"
            ));
            c.push_str("        for fact in facts {\n");
            c.push_str("            let content_lower = fact.content.to_lowercase();\n");
            for term in forbidden {
                let term_lower = esc(&term.term.to_lowercase());
                let term_display = esc(&term.term);
                let reason_display = esc(&term.reason);
                c.push_str(&format!(
                    "            if content_lower.contains(\"{term_lower}\") {{\n"
                ));
                c.push_str("                return GuestInvariantResult {\n");
                c.push_str("                    ok: false,\n");
                c.push_str(&format!(
                    "                    reason: Some(format!(\"{key_e} fact '{{}}' contains forbidden term: {term_display} ({reason_display})\", fact.id)),\n"
                ));
                c.push_str("                    fact_ids: vec![fact.id.clone()],\n");
                c.push_str("                };\n");
                c.push_str("            }\n");
            }
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::ContentMustContain {
            key,
            required_field,
        } => {
            let key_e = esc(key);
            let field_e = esc(required_field);
            let mut c = String::new();
            c.push_str(&format!(
                "    if let Some(facts) = ctx.facts.get(\"{key_e}\") {{\n"
            ));
            c.push_str("        for fact in facts {\n");
            c.push_str(&format!(
                "            if !fact.content.contains(\"{field_e}\") {{\n"
            ));
            c.push_str("                return GuestInvariantResult {\n");
            c.push_str("                    ok: false,\n");
            c.push_str(&format!(
                "                    reason: Some(format!(\"{key_e} fact '{{}}' missing required field: {field_e}\", fact.id)),\n"
            ));
            c.push_str("                    fact_ids: vec![fact.id.clone()],\n");
            c.push_str("                };\n");
            c.push_str("            }\n");
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::CrossReference {
            source_key,
            target_key,
        } => {
            let src_e = esc(source_key);
            let tgt_e = esc(target_key);
            let mut c = String::new();
            c.push_str(&format!(
                "    if let Some(source_facts) = ctx.facts.get(\"{src_e}\") {{\n"
            ));
            c.push_str(&format!(
                "        let target_facts = ctx.facts.get(\"{tgt_e}\");\n"
            ));
            c.push_str("        let empty = Vec::new();\n");
            c.push_str("        let targets = target_facts.unwrap_or(&empty);\n");
            c.push_str("        for source in source_facts {\n");
            c.push_str(
                "            let referenced = targets.iter().any(|t| t.content.contains(&source.id));\n",
            );
            c.push_str("            if !referenced {\n");
            c.push_str("                return GuestInvariantResult {\n");
            c.push_str("                    ok: false,\n");
            c.push_str(&format!(
                "                    reason: Some(format!(\"{src_e} fact '{{}}' has no corresponding {tgt_e}\", source.id)),\n"
            ));
            c.push_str("                    fact_ids: vec![source.id.clone()],\n");
            c.push_str("                };\n");
            c.push_str("            }\n");
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::HasFacts { key } => {
            let key_e = esc(key);
            let mut c = String::new();
            c.push_str(&format!(
                "    if ctx.facts.get(\"{key_e}\").map(|v| v.is_empty()).unwrap_or(true) {{\n"
            ));
            c.push_str("        return GuestInvariantResult {\n");
            c.push_str("            ok: false,\n");
            c.push_str(&format!(
                "            reason: Some(\"{key_e} must contain at least one fact\".to_string()),\n"
            ));
            c.push_str("            fact_ids: Vec::new(),\n");
            c.push_str("        };\n");
            c.push_str("    }\n");
            c
        }

        Predicate::RequiredFields { key, fields } => {
            let key_e = esc(key);
            let mut c = String::new();
            c.push_str(&format!(
                "    if let Some(facts) = ctx.facts.get(\"{key_e}\") {{\n"
            ));
            c.push_str("        for fact in facts {\n");
            for field in fields {
                let field_e = esc(&field.field);
                let rule_e = esc(&field.rule);
                c.push_str(&format!(
                    "            if !fact.content.contains(\"{field_e}\") {{\n"
                ));
                c.push_str("                return GuestInvariantResult {\n");
                c.push_str("                    ok: false,\n");
                c.push_str(&format!(
                    "                    reason: Some(format!(\"{key_e} fact '{{}}' missing required field: {field_e} ({rule_e})\", fact.id)),\n"
                ));
                c.push_str("                    fact_ids: vec![fact.id.clone()],\n");
                c.push_str("                };\n");
                c.push_str("            }\n");
            }
            c.push_str("        }\n");
            c.push_str("    }\n");
            c
        }

        Predicate::Custom { description } => {
            let safe = description.replace("*/", "* /").replace('\\', "\\\\");
            let mut c = String::new();
            c.push_str("    // TODO: Custom predicate — manual implementation needed\n");
            c.push_str(&format!("    // Original step: \"{safe}\"\n"));
            c
        }
    }
}

/// Escape a string for use inside a Rust string literal.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Format a string as a Rust raw string literal `r#"..."#`.
///
/// Automatically determines the minimum number of `#` delimiters needed
/// to safely wrap the content.
fn format_raw_string(s: &str) -> String {
    // Find the longest run of consecutive '#' in the string
    let mut max_hashes = 0;
    let mut current = 0;
    for c in s.chars() {
        if c == '#' {
            current += 1;
            if current > max_hashes {
                max_hashes = current;
            }
        } else {
            current = 0;
        }
    }

    // Also check for `"` followed by '#' runs that could close the raw string
    let hashes_needed = if s.contains('"') { max_hashes + 1 } else { 1 };

    let delim = "#".repeat(hashes_needed);
    format!("r{delim}\"{s}\"{delim}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::{FieldRequirement, ForbiddenTerm};
    use crate::truths::{AuthorityBlock, IntentBlock, TruthGovernance};

    fn test_config() -> CodegenConfig {
        CodegenConfig {
            manifest_json: r#"{"name":"test","version":"1.0.0","kind":"Invariant","invariant_class":"Structural","dependencies":["Strategies"],"capabilities":["ReadContext"],"requires_human_approval":false}"#.to_string(),
            module_name: "test_invariant".to_string(),
        }
    }

    // =========================================================================
    // Codegen tests (Task #3)
    // =========================================================================

    #[test]
    fn codegen_count_at_least() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::CountAtLeast {
                key: "Strategies".to_string(),
                min: 2,
            }],
        );
        assert!(source.contains("count < 2"));
        assert!(source.contains(r#"ctx.facts.get("Strategies")"#));
    }

    #[test]
    fn codegen_count_at_most() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::CountAtMost {
                key: "Seeds".to_string(),
                max: 5,
            }],
        );
        assert!(source.contains("count > 5"));
        assert!(source.contains(r#"ctx.facts.get("Seeds")"#));
    }

    #[test]
    fn codegen_content_must_not_contain() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::ContentMustNotContain {
                key: "Strategies".to_string(),
                forbidden: vec![ForbiddenTerm {
                    term: "spam".to_string(),
                    reason: "illegal marketing".to_string(),
                }],
            }],
        );
        assert!(source.contains("content_lower.contains("));
        assert!(source.contains("spam"));
        assert!(source.contains("illegal marketing"));
    }

    #[test]
    fn codegen_content_must_contain() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::ContentMustContain {
                key: "Strategies".to_string(),
                required_field: "compliance_ref".to_string(),
            }],
        );
        assert!(source.contains(r#"fact.content.contains("compliance_ref")"#));
    }

    #[test]
    fn codegen_cross_reference() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::CrossReference {
                source_key: "Strategy".to_string(),
                target_key: "Evaluation".to_string(),
            }],
        );
        assert!(source.contains(r#"ctx.facts.get("Strategy")"#));
        assert!(source.contains(r#"ctx.facts.get("Evaluation")"#));
        assert!(source.contains("t.content.contains(&source.id)"));
    }

    #[test]
    fn codegen_has_facts() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::HasFacts {
                key: "Signals".to_string(),
            }],
        );
        assert!(source.contains(r#"ctx.facts.get("Signals")"#));
        assert!(source.contains("v.is_empty()"));
    }

    #[test]
    fn codegen_required_fields() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::RequiredFields {
                key: "Evaluations".to_string(),
                fields: vec![
                    FieldRequirement {
                        field: "score".to_string(),
                        rule: "integer 0..100".to_string(),
                    },
                    FieldRequirement {
                        field: "rationale".to_string(),
                        rule: "non-empty".to_string(),
                    },
                ],
            }],
        );
        assert!(source.contains(r#"fact.content.contains("score")"#));
        assert!(source.contains(r#"fact.content.contains("rationale")"#));
    }

    #[test]
    fn codegen_custom_predicate_is_todo() {
        let source = generate_invariant_module(
            &test_config(),
            &[Predicate::Custom {
                description: "something special happens".to_string(),
            }],
        );
        assert!(source.contains("TODO"));
        assert!(source.contains("something special happens"));
    }

    #[test]
    fn codegen_includes_manifest_json() {
        let config = test_config();
        let source = generate_invariant_module(&config, &[]);
        assert!(source.contains(&config.manifest_json));
    }

    #[test]
    fn codegen_includes_alloc_dealloc() {
        let source = generate_invariant_module(&test_config(), &[]);
        assert!(source.contains("fn alloc("));
        assert!(source.contains("fn dealloc("));
    }

    #[test]
    fn codegen_includes_abi_exports() {
        let source = generate_invariant_module(&test_config(), &[]);
        assert!(source.contains("fn converge_abi_version()"));
        assert!(source.contains("fn converge_manifest()"));
        assert!(source.contains("fn check_invariant("));
    }

    #[test]
    fn codegen_empty_predicates_returns_ok() {
        let source = generate_invariant_module(&test_config(), &[]);
        assert!(source.contains("ok: true"));
        assert!(source.contains("No predicates"));
    }

    #[test]
    fn codegen_multiple_predicates() {
        let source = generate_invariant_module(
            &test_config(),
            &[
                Predicate::HasFacts {
                    key: "Strategies".to_string(),
                },
                Predicate::CountAtLeast {
                    key: "Strategies".to_string(),
                    min: 2,
                },
            ],
        );
        assert!(source.contains("Check 1:"));
        assert!(source.contains("Check 2:"));
    }

    // =========================================================================
    // ManifestBuilder tests (Task #4)
    // =========================================================================

    #[test]
    fn manifest_from_invariant_tags_and_jtbd() {
        let meta = ScenarioMeta {
            name: "Brand Safety Check".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: Some(InvariantClassTag::Acceptance),
            id: Some("brand_safety".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let jtbd = JTBDMetadata {
            actor: "Ops Manager".to_string(),
            job_functional: "Ensure brand safety".to_string(),
            job_emotional: None,
            job_relational: None,
            so_that: "Brand is protected".to_string(),
            scope: None,
            success_metrics: vec![],
            failure_modes: vec![],
            exceptions: vec![],
            evidence_required: vec![],
            audit_requirements: vec![],
            links: vec![],
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .from_jtbd(&jtbd)
            .from_predicates(&[Predicate::CountAtLeast {
                key: "Strategies".to_string(),
                min: 2,
            }])
            .with_truth_id("growth-strategy.truth")
            .build()
            .unwrap();

        assert!(json.contains("\"brand_safety\""));
        assert!(json.contains("\"Invariant\""));
        assert!(json.contains("\"Acceptance\""));
        assert!(json.contains("\"Strategies\""));
        assert!(json.contains("Ops Manager"));
        assert!(json.contains("Ensure brand safety"));
        assert!(json.contains("growth-strategy.truth"));
    }

    #[test]
    fn manifest_from_agent_tags() {
        let meta = ScenarioMeta {
            name: "Market Signal Suggestor".to_string(),
            kind: Some(ScenarioKind::Suggestor),
            invariant_class: None,
            id: Some("market_signal".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .from_predicates(&[Predicate::HasFacts {
                key: "Signals".to_string(),
            }])
            .build()
            .unwrap();

        assert!(json.contains("\"Suggestor\""));
        assert!(json.contains("\"Signals\""));
        assert!(json.contains("\"market_signal\""));
    }

    #[test]
    fn manifest_deps_inferred_from_predicates() {
        let meta = ScenarioMeta {
            name: "test".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: Some(InvariantClassTag::Semantic),
            id: Some("test".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .from_predicates(&[
                Predicate::CountAtLeast {
                    key: "Strategies".to_string(),
                    min: 1,
                },
                Predicate::HasFacts {
                    key: "Evaluations".to_string(),
                },
            ])
            .build()
            .unwrap();

        assert!(json.contains("\"Evaluations\""));
        assert!(json.contains("\"Strategies\""));
        assert!(json.contains("\"ReadContext\""));
    }

    #[test]
    fn manifest_invariant_without_class_errors() {
        let meta = ScenarioMeta {
            name: "test".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: None,
            id: Some("test".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let result = ManifestBuilder::new().from_scenario_meta(&meta).build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestError::MissingInvariantClass
        ));
    }

    #[test]
    fn manifest_agent_without_deps_errors() {
        let meta = ScenarioMeta {
            name: "test".to_string(),
            kind: Some(ScenarioKind::Suggestor),
            invariant_class: None,
            id: Some("test_agent".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let result = ManifestBuilder::new().from_scenario_meta(&meta).build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestError::MissingDependencies
        ));
    }

    #[test]
    fn manifest_name_from_sanitized_scenario() {
        let meta = ScenarioMeta {
            name: "Brand Safety Check".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: Some(InvariantClassTag::Structural),
            id: None, // No @id tag — use sanitized name
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .build()
            .unwrap();
        assert!(json.contains("\"brand_safety_check\""));
    }

    #[test]
    fn manifest_with_source_hash() {
        let meta = ScenarioMeta {
            name: "test".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: Some(InvariantClassTag::Structural),
            id: Some("test".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .with_truth_id("test.truth")
            .with_source_hash("sha256:abc123")
            .build()
            .unwrap();
        assert!(json.contains("sha256:abc123"));
        assert!(json.contains("test.truth"));
    }

    #[test]
    fn manifest_includes_truth_governance_metadata() {
        let meta = ScenarioMeta {
            name: "approval gate".to_string(),
            kind: Some(ScenarioKind::Invariant),
            invariant_class: Some(InvariantClassTag::Acceptance),
            id: Some("approval_gate".to_string()),
            provider: None,
            is_test: false,
            raw_tags: vec![],
        };

        let governance = TruthGovernance {
            intent: Some(IntentBlock {
                outcome: Some("Ship safely".to_string()),
                goal: Some("Avoid critical regressions".to_string()),
            }),
            authority: Some(AuthorityBlock {
                actor: Some("release_manager".to_string()),
                may: vec!["approve_deployment".to_string()],
                must_not: vec![],
                requires_approval: vec!["security_owner".to_string()],
                expires: Some("2026-03-31T23:59:59Z".to_string()),
            }),
            ..TruthGovernance::default()
        };

        let json = ManifestBuilder::new()
            .from_scenario_meta(&meta)
            .from_truth_governance(&governance)
            .build()
            .unwrap();

        assert!(json.contains("\"requires_human_approval\":true"));
        assert!(json.contains("Ship safely"));
        assert!(json.contains("release_manager"));
        assert!(json.contains("security_owner"));
    }

    // =========================================================================
    // Helper tests
    // =========================================================================

    #[test]
    fn sanitize_name_handles_spaces_and_casing() {
        assert_eq!(
            sanitize_module_name("Brand Safety Check"),
            "brand_safety_check"
        );
        assert_eq!(sanitize_module_name("  test  "), "test");
        assert_eq!(sanitize_module_name("CamelCase"), "camelcase");
        assert_eq!(sanitize_module_name("hyphen-name"), "hyphen_name");
    }

    #[test]
    fn format_raw_string_simple() {
        let result = format_raw_string("hello");
        assert_eq!(result, r##"r#"hello"#"##);
    }

    #[test]
    fn format_raw_string_with_quotes() {
        let result = format_raw_string(r#"{"key":"value"}"#);
        assert_eq!(result, r##"r#"{"key":"value"}"#"##);
    }

    #[test]
    fn esc_handles_special_chars() {
        assert_eq!(esc(r#"hello "world""#), r#"hello \"world\""#);
        assert_eq!(esc("back\\slash"), "back\\\\slash");
        assert_eq!(esc("new\nline"), "new\\nline");
    }

    // =========================================================================
    // Property tests
    // =========================================================================

    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_predicate() -> impl Strategy<Value = Predicate> {
            prop_oneof![
                (1..100usize).prop_map(|min| Predicate::CountAtLeast {
                    key: "Strategies".to_string(),
                    min,
                }),
                (1..100usize).prop_map(|max| Predicate::CountAtMost {
                    key: "Seeds".to_string(),
                    max,
                }),
                Just(Predicate::HasFacts {
                    key: "Signals".to_string()
                }),
                Just(Predicate::CrossReference {
                    source_key: "Strategies".to_string(),
                    target_key: "Evaluations".to_string(),
                }),
                Just(Predicate::ContentMustContain {
                    key: "Strategies".to_string(),
                    required_field: "compliance_ref".to_string(),
                }),
                "[a-z ]{1,50}".prop_map(|desc| Predicate::Custom { description: desc }),
            ]
        }

        proptest! {
            #[test]
            fn generated_code_is_syntactically_valid_rust(
                predicates in proptest::collection::vec(arb_predicate(), 0..5)
            ) {
                let config = CodegenConfig {
                    manifest_json: r#"{"name":"t","version":"1.0.0","kind":"Invariant","invariant_class":"Structural","dependencies":[],"capabilities":[],"requires_human_approval":false}"#.to_string(),
                    module_name: "test".to_string(),
                };
                let source = generate_invariant_module(&config, &predicates);
                syn::parse_file(&source).unwrap_or_else(|e| {
                    panic!("Generated code is not valid Rust:\n{source}\nError: {e}");
                });
            }

            #[test]
            fn manifest_builder_never_panics(
                name in "[a-z]{3,10}",
                is_invariant in proptest::bool::ANY,
                has_class in proptest::bool::ANY,
            ) {
                let meta = ScenarioMeta {
                    name: name.clone(),
                    kind: Some(if is_invariant { ScenarioKind::Invariant } else { ScenarioKind::Suggestor }),
                    invariant_class: if has_class { Some(InvariantClassTag::Structural) } else { None },
                    id: Some(name),
                    provider: None,
                    is_test: false,
                    raw_tags: vec![],
                };
                // Should never panic — either Ok or Err
                let _ = ManifestBuilder::new()
                    .from_scenario_meta(&meta)
                    .build();
            }
        }
    }
}
