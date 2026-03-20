// Copyright 2024-2026 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! WASM compilation pipeline for Converge truth files.
//!
//! Compiles generated Rust source code to `.wasm` binaries by creating a
//! temporary Cargo project and invoking `cargo build --target wasm32-unknown-unknown`.
//!
//! # Pipeline
//!
//! ```text
//! .truth file → parse → predicates → Rust source → cargo build → .wasm bytes
//! ```
//!
//! # Requirements
//!
//! - Rust toolchain with `cargo` in PATH
//! - `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`

use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use crate::codegen::{
    CodegenConfig, ManifestBuilder, generate_invariant_module, sanitize_module_name,
};
use crate::gherkin::extract_scenario_meta;
use crate::predicate::parse_steps;
use crate::truths::parse_truth_document;

// ============================================================================
// Types
// ============================================================================

/// WASM compilation target triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WasmTarget {
    /// Standard WASM target without WASI (default for Converge modules).
    #[default]
    Wasm32UnknownUnknown,
    /// WASI target for modules needing system interface.
    Wasm32Wasip1,
}

impl WasmTarget {
    fn as_str(self) -> &'static str {
        match self {
            Self::Wasm32UnknownUnknown => "wasm32-unknown-unknown",
            Self::Wasm32Wasip1 => "wasm32-wasip1",
        }
    }
}

/// Optimization level for WASM compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptLevel {
    /// No optimization (debug build).
    Debug,
    /// Full optimization.
    Release,
    /// Optimize for binary size (default for WASM modules).
    #[default]
    Size,
}

/// Configuration for WASM compilation.
#[derive(Debug, Clone, Default)]
pub struct CompileConfig {
    /// Target triple.
    pub target: WasmTarget,
    /// Optimization level.
    pub opt_level: OptLevel,
}

/// Result of compiling a `.truth` file to WASM.
#[derive(Debug, Clone)]
pub struct CompiledModule {
    /// Raw `.wasm` bytes.
    pub wasm_bytes: Vec<u8>,
    /// Manifest JSON embedded in the module.
    pub manifest_json: String,
    /// SHA-256 hash of the source `.truth` file content.
    pub source_hash: String,
    /// Module name derived from scenario tags or sanitized scenario name.
    pub module_name: String,
}

/// Error during WASM compilation.
#[derive(Debug)]
pub enum CompileError {
    /// WASM target not installed.
    MissingTarget(String),
    /// `cargo build` failed.
    BuildFailed { stdout: String, stderr: String },
    /// Compiled `.wasm` file not found in target directory.
    WasmNotFound(PathBuf),
    /// IO error during file operations.
    Io(std::io::Error),
    /// Gherkin parsing or predicate extraction error.
    GherkinParse(String),
    /// Manifest building error.
    ManifestBuild(String),
    /// No compilable scenarios found in truth file.
    NoScenarios,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTarget(target) => write!(
                f,
                "WASM target '{target}' not installed. Run: rustup target add {target}"
            ),
            Self::BuildFailed { stderr, .. } => write!(f, "cargo build failed:\n{stderr}"),
            Self::WasmNotFound(path) => {
                write!(f, "compiled .wasm not found at: {}", path.display())
            }
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::GherkinParse(msg) => write!(f, "Gherkin parse error: {msg}"),
            Self::ManifestBuild(msg) => write!(f, "manifest build error: {msg}"),
            Self::NoScenarios => write!(f, "no compilable scenarios found in truth file"),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ============================================================================
// Compiler
// ============================================================================

/// WASM module compiler.
///
/// Compiles generated Rust source code to `.wasm` binaries by creating a
/// temporary Cargo project and invoking `cargo build`.
///
/// # Examples
///
/// ```ignore
/// use converge_tool::compile::{WasmCompiler, CompileConfig};
///
/// let source = "/* generated Rust source */";
/// let wasm_bytes = WasmCompiler::compile(source, &CompileConfig::default())?;
/// assert_eq!(&wasm_bytes[0..4], b"\0asm");
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct WasmCompiler;

impl WasmCompiler {
    /// Compile Rust source code to WASM bytes.
    ///
    /// Creates a temporary Cargo project, writes the source as `lib.rs`,
    /// and runs `cargo build --target <target> --release`.
    ///
    /// # Errors
    ///
    /// Returns `CompileError::MissingTarget` if the WASM target is not installed.
    /// Returns `CompileError::BuildFailed` if cargo compilation fails.
    pub fn compile(source: &str, config: &CompileConfig) -> Result<Vec<u8>, CompileError> {
        Self::check_target(config)?;

        let tmp_dir = std::env::temp_dir().join(format!("converge-wasm-{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir)?;

        let result = Self::compile_in_dir(source, config, &tmp_dir);

        // Clean up temp directory
        let _ = std::fs::remove_dir_all(&tmp_dir);

        result
    }

    /// Compile a `.truth` file end-to-end: parse → codegen → compile → hash.
    ///
    /// Reads the truth file, parses Gherkin scenarios, extracts predicates,
    /// generates Rust source, and compiles to WASM bytes. Returns a
    /// `CompiledModule` containing the .wasm bytes, manifest, source hash,
    /// and module name.
    ///
    /// Currently compiles the first compilable scenario (non-test, with a kind
    /// tag). Future versions may compile all scenarios into a multi-invariant
    /// module.
    ///
    /// # Errors
    ///
    /// Returns errors for any stage of the pipeline.
    pub fn compile_truth_file(path: &Path) -> Result<CompiledModule, CompileError> {
        let content = std::fs::read_to_string(path)?;
        let source_hash = content_hash(content.as_bytes());

        // Parse Gherkin
        let document =
            parse_truth_document(&content).map_err(|e| CompileError::GherkinParse(e.to_string()))?;
        let feature = gherkin::Feature::parse(&document.gherkin, gherkin::GherkinEnv::default())
            .map_err(|e| CompileError::GherkinParse(format!("{e}")))?;

        // Extract scenario metadata
        let metas: Vec<_> = feature
            .scenarios
            .iter()
            .map(|s| extract_scenario_meta(&s.name, &s.tags))
            .collect();

        // Find first compilable scenario (has kind, not a test)
        let compilable_idx = metas
            .iter()
            .enumerate()
            .find(|(_, m)| !m.is_test && m.kind.is_some())
            .map(|(i, _)| i)
            .ok_or(CompileError::NoScenarios)?;

        let meta = &metas[compilable_idx];
        let scenario = &feature.scenarios[compilable_idx];

        // Convert gherkin steps to predicate parser format
        let step_tuples = steps_to_tuples(&scenario.steps);
        let step_refs: Vec<(&str, &str, Vec<Vec<String>>)> = step_tuples
            .iter()
            .map(|(kw, text, table)| (kw.as_str(), text.as_str(), table.clone()))
            .collect();

        let predicates = parse_steps(&step_refs)
            .map_err(|e| CompileError::GherkinParse(format!("predicate parse: {e}")))?;

        // Build manifest
        let truth_id = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let manifest_json = ManifestBuilder::new()
            .from_scenario_meta(meta)
            .from_predicates(&predicates)
            .from_truth_governance(&document.governance)
            .with_truth_id(&truth_id)
            .with_source_hash(&source_hash)
            .build()
            .map_err(|e| CompileError::ManifestBuild(e.to_string()))?;

        let module_name = meta
            .id
            .clone()
            .unwrap_or_else(|| sanitize_module_name(&meta.name));

        // Generate Rust source
        let codegen_config = CodegenConfig {
            manifest_json: manifest_json.clone(),
            module_name: module_name.clone(),
        };
        let rust_source = generate_invariant_module(&codegen_config, &predicates);

        // Compile to WASM
        let wasm_bytes = Self::compile(&rust_source, &CompileConfig::default())?;

        Ok(CompiledModule {
            wasm_bytes,
            manifest_json,
            source_hash,
            module_name,
        })
    }

    /// Compute SHA-256 content hash for WASM bytes (for `ModuleId`).
    #[must_use]
    pub fn content_hash_wasm(bytes: &[u8]) -> String {
        content_hash(bytes)
    }

    fn compile_in_dir(
        source: &str,
        config: &CompileConfig,
        dir: &Path,
    ) -> Result<Vec<u8>, CompileError> {
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Write Cargo.toml
        std::fs::write(dir.join("Cargo.toml"), generate_cargo_toml(config))?;

        // Write lib.rs
        std::fs::write(src_dir.join("lib.rs"), source)?;

        // Build
        let target = config.target.as_str();
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--target")
            .arg(target)
            .arg("--lib")
            .current_dir(dir);

        if config.opt_level != OptLevel::Debug {
            cmd.arg("--release");
        }

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CompileError::MissingTarget("cargo not found in PATH".to_string())
            } else {
                CompileError::Io(e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            if stderr.contains("target may not be installed")
                || stderr.contains("can't find crate for `std`")
            {
                return Err(CompileError::MissingTarget(target.to_string()));
            }

            return Err(CompileError::BuildFailed { stdout, stderr });
        }

        // Read .wasm output
        let profile = if config.opt_level == OptLevel::Debug {
            "debug"
        } else {
            "release"
        };
        let wasm_path = dir
            .join("target")
            .join(target)
            .join(profile)
            .join("converge_wasm_module.wasm");

        if !wasm_path.exists() {
            return Err(CompileError::WasmNotFound(wasm_path));
        }

        Ok(std::fs::read(&wasm_path)?)
    }

    fn check_target(config: &CompileConfig) -> Result<(), CompileError> {
        let target = config.target.as_str();
        let output = Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let installed = String::from_utf8_lossy(&out.stdout);
                if !installed.lines().any(|line| line.trim() == target) {
                    return Err(CompileError::MissingTarget(target.to_string()));
                }
                Ok(())
            }
            // rustup not available — try compilation anyway
            _ => Ok(()),
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Generate a minimal `Cargo.toml` for the temporary compilation crate.
fn generate_cargo_toml(config: &CompileConfig) -> String {
    let opt_level = match config.opt_level {
        OptLevel::Debug => "0",
        OptLevel::Release => "2",
        OptLevel::Size => "s",
    };

    format!(
        r#"[package]
name = "converge-wasm-module"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"

[profile.release]
opt-level = "{opt_level}"
lto = true
strip = true
codegen-units = 1
"#
    )
}

/// Convert `gherkin::Step` list to the tuple format expected by `parse_steps`.
///
/// The gherkin crate resolves `And`/`But` keywords into their parent
/// `StepType` (Given/When/Then), but the `keyword` field preserves the
/// original keyword. We use `keyword` to maintain And/But distinction
/// since `parse_steps` handles them differently.
///
/// The gherkin crate includes header rows in `Table.rows`. The predicate
/// parser expects data rows only, so we skip the first row (header) of
/// each table.
fn steps_to_tuples(steps: &[gherkin::Step]) -> Vec<(String, String, Vec<Vec<String>>)> {
    steps
        .iter()
        .map(|step| {
            // Use raw keyword (preserves And/But), trim trailing whitespace
            let keyword = step.keyword.trim().to_string();
            let table = step
                .table
                .as_ref()
                .map(|t| {
                    // Skip header row — predicate parser expects data rows only
                    if t.rows.len() > 1 {
                        t.rows[1..].to_vec()
                    } else {
                        Vec::new()
                    }
                })
                .unwrap_or_default();
            (keyword, step.value.clone(), table)
        })
        .collect()
}

/// Compute SHA-256 content hash.
pub fn content_hash(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    format!("sha256:{hash:x}")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Content hashing
    // =========================================================================

    #[test]
    fn content_hash_produces_sha256_prefix() {
        let hash = content_hash(b"hello world");
        assert!(hash.starts_with("sha256:"));
        // "sha256:" (7 chars) + 64 hex chars
        assert_eq!(hash.len(), 7 + 64);
    }

    #[test]
    fn content_hash_is_deterministic() {
        let h1 = content_hash(b"test data");
        let h2 = content_hash(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn content_hash_differs_for_different_input() {
        let h1 = content_hash(b"hello");
        let h2 = content_hash(b"world");
        assert_ne!(h1, h2);
    }

    // =========================================================================
    // Cargo.toml generation
    // =========================================================================

    #[test]
    fn cargo_toml_includes_required_deps() {
        let toml = generate_cargo_toml(&CompileConfig::default());
        assert!(toml.contains("serde"));
        assert!(toml.contains("serde_json"));
        assert!(toml.contains("cdylib"));
        assert!(toml.contains("edition = \"2024\""));
    }

    #[test]
    fn cargo_toml_uses_size_opt_for_default() {
        let toml = generate_cargo_toml(&CompileConfig::default());
        assert!(toml.contains(r#"opt-level = "s""#));
    }

    #[test]
    fn cargo_toml_respects_opt_level() {
        let release = CompileConfig {
            opt_level: OptLevel::Release,
            ..Default::default()
        };
        assert!(generate_cargo_toml(&release).contains(r#"opt-level = "2""#));

        let debug = CompileConfig {
            opt_level: OptLevel::Debug,
            ..Default::default()
        };
        assert!(generate_cargo_toml(&debug).contains(r#"opt-level = "0""#));
    }

    // =========================================================================
    // Target
    // =========================================================================

    #[test]
    fn wasm_target_as_str() {
        assert_eq!(
            WasmTarget::Wasm32UnknownUnknown.as_str(),
            "wasm32-unknown-unknown"
        );
        assert_eq!(WasmTarget::Wasm32Wasip1.as_str(), "wasm32-wasip1");
    }

    #[test]
    fn default_config() {
        let config = CompileConfig::default();
        assert_eq!(config.target, WasmTarget::Wasm32UnknownUnknown);
        assert_eq!(config.opt_level, OptLevel::Size);
    }

    // =========================================================================
    // Error types
    // =========================================================================

    #[test]
    fn compile_error_display_missing_target() {
        let err = CompileError::MissingTarget("wasm32-unknown-unknown".to_string());
        let msg = err.to_string();
        assert!(msg.contains("rustup target add"));
        assert!(msg.contains("wasm32-unknown-unknown"));
    }

    #[test]
    fn compile_error_display_build_failed() {
        let err = CompileError::BuildFailed {
            stdout: String::new(),
            stderr: "error[E0432]: unresolved import".to_string(),
        };
        assert!(err.to_string().contains("unresolved import"));
    }

    #[test]
    fn compile_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = CompileError::from(io_err);
        assert!(matches!(err, CompileError::Io(_)));
    }

    // =========================================================================
    // Step conversion
    // =========================================================================

    // =========================================================================
    // Integration tests (require wasm32 target and cargo)
    // =========================================================================

    #[test]
    #[ignore] // Requires wasm32-unknown-unknown target installed
    fn compile_minimal_invariant() {
        use crate::predicate::Predicate;

        let config = CodegenConfig {
            manifest_json: r#"{"name":"test","version":"0.1.0","kind":"Invariant","invariant_class":"Structural","dependencies":["Strategies"],"capabilities":["ReadContext"],"requires_human_approval":false}"#.to_string(),
            module_name: "test_invariant".to_string(),
        };

        let source = generate_invariant_module(
            &config,
            &[Predicate::CountAtLeast {
                key: "Strategies".to_string(),
                min: 2,
            }],
        );

        let wasm_bytes = WasmCompiler::compile(&source, &CompileConfig::default()).unwrap();

        // Verify WASM magic bytes: \0asm
        assert!(wasm_bytes.len() > 8);
        assert_eq!(&wasm_bytes[0..4], b"\0asm");

        // Content hash should work on the output
        let hash = content_hash(&wasm_bytes);
        assert!(hash.starts_with("sha256:"));
    }

    #[test]
    #[ignore] // Requires wasm32-unknown-unknown target installed
    fn compile_truth_file_end_to_end() {
        let truth_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("specs")
            .join("growth-strategy.truth");

        assert!(
            truth_path.exists(),
            "Test truth file not found: {}",
            truth_path.display()
        );

        let module = WasmCompiler::compile_truth_file(&truth_path).unwrap();

        // Verify WASM magic bytes
        assert!(module.wasm_bytes.len() > 8);
        assert_eq!(&module.wasm_bytes[0..4], b"\0asm");

        // First compilable scenario should be brand_safety
        assert_eq!(module.module_name, "brand_safety");
        assert!(module.manifest_json.contains("brand_safety"));
        assert!(module.source_hash.starts_with("sha256:"));
    }

    #[test]
    #[ignore] // Requires wasm32-unknown-unknown target installed
    fn compile_invalid_rust_returns_build_error() {
        let result = WasmCompiler::compile("this is not valid rust", &CompileConfig::default());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CompileError::BuildFailed { .. }
        ));
    }

    #[test]
    fn compile_truth_file_nonexistent_path() {
        let result = WasmCompiler::compile_truth_file(Path::new("/nonexistent/file.truth"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CompileError::Io(_)));
    }
}
