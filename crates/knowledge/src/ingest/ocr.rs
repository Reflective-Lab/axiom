//! OCR contracts for screenshots and photos.
//!
//! This module defines shared request/response types and backend traits used by
//! Phase 2 image ingestion specialists. Backends (Apple Vision, Tesseract, mock)
//! can implement the trait without changing ingestion orchestration logic.

use crate::Result;
use crate::error::Error;
use crate::ingest::SourceProvenance;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use tokio::process::Command;

/// Supported OCR engine families.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrEngine {
    /// Apple Vision OCR on macOS.
    AppleVision,
    /// Tesseract OCR.
    Tesseract,
    /// Test/mock backend.
    Mock,
    /// Any external/custom provider.
    External,
}

/// The source image type being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrTargetKind {
    /// A screen capture where UI chrome may be present.
    Screenshot,
    /// A photo captured by a camera.
    Photo,
    /// Generic image input where no source-specific assumptions apply.
    GenericImage,
}

/// Bounding box for OCR blocks in normalized coordinates (0.0..=1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Left X coordinate.
    pub x: f32,
    /// Top Y coordinate.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl BoundingBox {
    /// Returns `true` when the box is non-negative and inside normalized bounds.
    pub fn is_normalized(&self) -> bool {
        self.x >= 0.0
            && self.y >= 0.0
            && self.width >= 0.0
            && self.height >= 0.0
            && self.x <= 1.0
            && self.y <= 1.0
            && self.width <= 1.0
            && self.height <= 1.0
            && self.x + self.width <= 1.0
            && self.y + self.height <= 1.0
    }
}

/// OCR block classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OcrBlockKind {
    /// Paragraph-level text block.
    Paragraph,
    /// Line-level text block.
    Line,
    /// Word/token-level OCR block.
    Word,
    /// UI chrome text (menus, buttons, labels) in screenshots.
    UiChrome,
    /// Unclassified OCR block.
    Unknown,
}

/// A single OCR-detected text block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OcrTextBlock {
    /// The recognized text.
    pub text: String,
    /// Confidence from 0.0 to 1.0 when provided by the backend.
    pub confidence: Option<f32>,
    /// Normalized bounding box when provided by the backend.
    pub bbox: Option<BoundingBox>,
    /// Classification for this block.
    pub kind: OcrBlockKind,
}

impl OcrTextBlock {
    /// Create a basic line block with no confidence or bounding box.
    pub fn line(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            confidence: None,
            bbox: None,
            kind: OcrBlockKind::Line,
        }
    }
}

/// Request passed to OCR backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOcrRequest {
    /// Path to the source image file.
    pub path: PathBuf,
    /// High-level target kind for backend tuning.
    pub target_kind: OcrTargetKind,
    /// Shared provenance metadata for this image.
    pub provenance: SourceProvenance,
    /// Language hints (e.g., `en`, `de`) to improve OCR.
    pub language_hints: Vec<String>,
    /// Optional minimum confidence filter hint.
    pub min_confidence: Option<f32>,
    /// Source-specific request metadata.
    pub metadata: HashMap<String, String>,
}

impl ImageOcrRequest {
    /// Create a new OCR request.
    pub fn new(
        path: impl Into<PathBuf>,
        target_kind: OcrTargetKind,
        provenance: SourceProvenance,
    ) -> Self {
        Self {
            path: path.into(),
            target_kind,
            provenance,
            language_hints: Vec::new(),
            min_confidence: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a language hint.
    pub fn with_language_hint(mut self, language: impl Into<String>) -> Self {
        self.language_hints.push(language.into());
        self
    }

    /// Set a minimum confidence hint.
    pub fn with_min_confidence(mut self, min_confidence: f32) -> Self {
        self.min_confidence = Some(min_confidence);
        self
    }
}

/// OCR extraction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrDocument {
    /// OCR backend used to produce this result.
    pub engine: OcrEngine,
    /// The source target kind.
    pub target_kind: OcrTargetKind,
    /// Full-text result if the backend produced one directly.
    pub full_text: String,
    /// Structured OCR blocks.
    pub blocks: Vec<OcrTextBlock>,
    /// Extraction timestamp.
    pub extracted_at: DateTime<Utc>,
    /// Shared provenance for the source image.
    pub provenance: SourceProvenance,
}

impl OcrDocument {
    /// Create an empty OCR document for incremental population.
    pub fn new(
        engine: OcrEngine,
        target_kind: OcrTargetKind,
        provenance: SourceProvenance,
    ) -> Self {
        Self {
            engine,
            target_kind,
            full_text: String::new(),
            blocks: Vec::new(),
            extracted_at: Utc::now(),
            provenance,
        }
    }

    /// Returns the best available text for indexing.
    ///
    /// Uses `full_text` when present, otherwise concatenates non-empty blocks.
    pub fn effective_text(&self) -> String {
        let trimmed = self.full_text.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }

        self.blocks
            .iter()
            .map(|block| block.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns text from blocks whose confidence meets the threshold.
    pub fn filtered_text(&self, min_confidence: f32) -> String {
        self.blocks
            .iter()
            .filter(|block| block.confidence.unwrap_or(1.0) >= min_confidence)
            .map(|block| block.text.trim())
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// OCR backend contract used by screenshot/photo ingestion pipelines.
#[async_trait]
pub trait OcrBackend: Send + Sync {
    /// The engine identifier for this backend.
    fn engine(&self) -> OcrEngine;

    /// Extract OCR text and block metadata from an image.
    async fn extract(&self, request: &ImageOcrRequest) -> Result<OcrDocument>;
}

/// Configuration for the Tesseract OCR backend.
#[derive(Debug, Clone)]
pub struct TesseractOcrConfig {
    /// Path to the `tesseract` binary.
    pub binary_path: String,
    /// Default OCR language codes (e.g., `["eng"]`, `["eng", "deu"]`).
    pub default_languages: Vec<String>,
    /// Optional OCR engine mode (`--oem`).
    pub oem: Option<u8>,
    /// Optional page segmentation mode (`--psm`).
    pub psm: Option<u8>,
    /// Additional CLI arguments passed through to `tesseract`.
    pub extra_args: Vec<String>,
    /// Trim trailing whitespace/newlines from output text.
    pub trim_output: bool,
    /// Synthesize line blocks from plain-text output when no structured output is parsed.
    pub synthesize_line_blocks: bool,
}

impl Default for TesseractOcrConfig {
    fn default() -> Self {
        Self {
            binary_path: "tesseract".to_string(),
            default_languages: vec!["eng".to_string()],
            oem: None,
            psm: None,
            extra_args: Vec::new(),
            trim_output: true,
            synthesize_line_blocks: true,
        }
    }
}

impl TesseractOcrConfig {
    /// Set the Tesseract binary path.
    pub fn with_binary_path(mut self, binary_path: impl Into<String>) -> Self {
        self.binary_path = binary_path.into();
        self
    }

    /// Set the default OCR languages.
    pub fn with_default_languages(
        mut self,
        langs: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.default_languages = langs.into_iter().map(Into::into).collect();
        self
    }

    /// Set OCR engine mode (`--oem`).
    pub fn with_oem(mut self, oem: u8) -> Self {
        self.oem = Some(oem);
        self
    }

    /// Set page segmentation mode (`--psm`).
    pub fn with_psm(mut self, psm: u8) -> Self {
        self.psm = Some(psm);
        self
    }

    /// Append an extra CLI argument.
    pub fn with_extra_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Disable synthesized line blocks (text-only output).
    pub fn without_line_blocks(mut self) -> Self {
        self.synthesize_line_blocks = false;
        self
    }
}

/// Real OCR backend using the `tesseract` CLI binary.
///
/// This backend executes `tesseract <image> stdout ...` and returns the OCR text.
/// The initial implementation parses plain text output and can synthesize line
/// blocks from it; structured confidence/bounding box parsing can be added later.
#[derive(Debug, Clone, Default)]
pub struct TesseractOcrBackend {
    config: TesseractOcrConfig,
}

#[derive(Debug, Clone, Copy)]
enum TesseractOutputMode {
    PlainText,
    Tsv,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TesseractLineKey {
    page_num: u32,
    block_num: u32,
    par_num: u32,
    line_num: u32,
}

#[derive(Debug, Clone, Copy)]
struct TesseractPageDimensions {
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
struct TesseractLineAccumulator {
    words: Vec<(u32, String)>,
    confidence_sum: f32,
    confidence_count: usize,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

impl TesseractLineAccumulator {
    fn new(
        word_num: u32,
        text: String,
        confidence: f32,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            words: vec![(word_num, text)],
            confidence_sum: confidence,
            confidence_count: 1,
            left,
            top,
            right: left.saturating_add(width),
            bottom: top.saturating_add(height),
        }
    }

    fn add_word(
        &mut self,
        word_num: u32,
        text: String,
        confidence: f32,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) {
        self.words.push((word_num, text));
        self.confidence_sum += confidence;
        self.confidence_count += 1;
        self.left = self.left.min(left);
        self.top = self.top.min(top);
        self.right = self.right.max(left.saturating_add(width));
        self.bottom = self.bottom.max(top.saturating_add(height));
    }
}

#[derive(Debug, Clone)]
struct ParsedTesseractTsv {
    full_text: String,
    blocks: Vec<OcrTextBlock>,
}

impl TesseractOcrBackend {
    /// Create a Tesseract backend with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a Tesseract backend with custom configuration.
    pub fn with_config(config: TesseractOcrConfig) -> Self {
        Self { config }
    }

    /// Return the configured binary path.
    pub fn binary_path(&self) -> &str {
        &self.config.binary_path
    }

    /// Check whether the configured `tesseract` binary is available.
    pub async fn is_available(&self) -> bool {
        self.version().await.is_ok()
    }

    /// Read the `tesseract` version string.
    pub async fn version(&self) -> Result<String> {
        let output = Command::new(&self.config.binary_path)
            .arg("--version")
            .output()
            .await
            .map_err(|err| {
                Error::ingest(format!(
                    "failed to execute tesseract binary '{}': {err}",
                    self.config.binary_path
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(Error::ingest(format!(
                "tesseract --version failed (status {}): {stderr}",
                output.status
            )));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if version.is_empty() {
            return Err(Error::ingest("tesseract --version returned empty output"));
        }

        Ok(version)
    }

    fn build_args(&self, request: &ImageOcrRequest) -> Vec<String> {
        self.build_args_for_mode(request, TesseractOutputMode::PlainText)
    }

    fn build_tsv_args(&self, request: &ImageOcrRequest) -> Vec<String> {
        self.build_args_for_mode(request, TesseractOutputMode::Tsv)
    }

    fn build_args_for_mode(
        &self,
        request: &ImageOcrRequest,
        mode: TesseractOutputMode,
    ) -> Vec<String> {
        let mut args = vec![
            request.path.to_string_lossy().into_owned(),
            "stdout".to_string(),
        ];

        let languages = if request.language_hints.is_empty() {
            self.config.default_languages.clone()
        } else {
            request.language_hints.clone()
        };
        let languages = normalize_tesseract_language_hints(languages);

        if !languages.is_empty() {
            args.push("-l".to_string());
            args.push(languages.join("+"));
        }

        if let Some(oem) = self.config.oem {
            args.push("--oem".to_string());
            args.push(oem.to_string());
        }

        if let Some(psm) = self
            .config
            .psm
            .or_else(|| default_psm_for_target(request.target_kind))
        {
            args.push("--psm".to_string());
            args.push(psm.to_string());
        }

        args.extend(self.config.extra_args.iter().cloned());

        if matches!(mode, TesseractOutputMode::Tsv) {
            args.push("tsv".to_string());
        }

        args
    }

    fn normalize_output_text(&self, stdout: &[u8]) -> String {
        let text = String::from_utf8_lossy(stdout).to_string();
        if self.config.trim_output {
            text.trim().to_string()
        } else {
            text
        }
    }

    fn synthesize_blocks(&self, text: &str) -> Vec<OcrTextBlock> {
        if !self.config.synthesize_line_blocks {
            return Vec::new();
        }

        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(OcrTextBlock::line)
            .collect()
    }

    async fn run_tesseract(
        &self,
        request: &ImageOcrRequest,
        mode: TesseractOutputMode,
    ) -> Result<Vec<u8>> {
        let args = match mode {
            TesseractOutputMode::PlainText => self.build_args(request),
            TesseractOutputMode::Tsv => self.build_tsv_args(request),
        };

        let output = Command::new(&self.config.binary_path)
            .args(&args)
            .output()
            .await
            .map_err(|err| {
                Error::ingest(format!(
                    "failed to execute tesseract binary '{}': {err}",
                    self.config.binary_path
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let mode_label = match mode {
                TesseractOutputMode::PlainText => "plain-text",
                TesseractOutputMode::Tsv => "tsv",
            };
            return Err(Error::ingest(format!(
                "tesseract OCR ({mode_label}) failed for '{}'(status {}): {}",
                request.path.display(),
                output.status,
                if stderr.is_empty() {
                    "no stderr output".to_string()
                } else {
                    stderr
                }
            )));
        }

        Ok(output.stdout)
    }

    fn parse_tsv_output(&self, stdout: &[u8]) -> Result<ParsedTesseractTsv> {
        let tsv = String::from_utf8_lossy(stdout);
        parse_tesseract_tsv(&tsv)
    }

    fn build_document_from_plain_text(
        &self,
        request: &ImageOcrRequest,
        full_text: String,
    ) -> OcrDocument {
        let mut doc = OcrDocument::new(
            self.engine(),
            request.target_kind,
            request.provenance.clone(),
        );
        doc.full_text = full_text.clone();
        doc.blocks = self.synthesize_blocks(&full_text);
        doc
    }

    fn build_document_from_tsv(
        &self,
        request: &ImageOcrRequest,
        parsed: ParsedTesseractTsv,
    ) -> OcrDocument {
        let mut doc = OcrDocument::new(
            self.engine(),
            request.target_kind,
            request.provenance.clone(),
        );
        doc.full_text = if self.config.trim_output {
            parsed.full_text.trim().to_string()
        } else {
            parsed.full_text
        };
        doc.blocks = parsed.blocks;
        doc
    }
}

fn normalize_tesseract_language_hints(hints: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::with_capacity(hints.len());
    for hint in hints {
        let code = normalize_tesseract_language_hint(&hint);
        if !code.is_empty() && !normalized.iter().any(|existing| existing == &code) {
            normalized.push(code);
        }
    }
    normalized
}

fn normalize_tesseract_language_hint(hint: &str) -> String {
    let trimmed = hint.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Preserve explicit Tesseract pack identifiers (e.g. `eng`, `chi_sim`).
    if trimmed.contains('_')
        || (trimmed.len() == 3 && trimmed.chars().all(|c| c.is_ascii_alphabetic()))
    {
        return trimmed.to_ascii_lowercase();
    }

    let primary = trimmed
        .split(['-', '_'])
        .next()
        .unwrap_or(trimmed)
        .to_ascii_lowercase();

    match primary.as_str() {
        "en" => "eng",
        "de" => "deu",
        "fr" => "fra",
        "es" => "spa",
        "it" => "ita",
        "pt" => "por",
        "nl" => "nld",
        "sv" => "swe",
        "da" => "dan",
        "fi" => "fin",
        "no" | "nb" | "nn" => "nor",
        "pl" => "pol",
        "cs" => "ces",
        "sk" => "slk",
        "sl" => "slv",
        "hr" => "hrv",
        "sr" => "srp",
        "ro" => "ron",
        "hu" => "hun",
        "tr" => "tur",
        "el" => "ell",
        "ru" => "rus",
        "uk" => "ukr",
        "bg" => "bul",
        "he" | "iw" => "heb",
        "ar" => "ara",
        "fa" => "fas",
        "hi" => "hin",
        "bn" => "ben",
        "ta" => "tam",
        "te" => "tel",
        "ml" => "mal",
        "mr" => "mar",
        "gu" => "guj",
        "pa" => "pan",
        "ur" => "urd",
        "ja" => "jpn",
        "ko" => "kor",
        "zh" => "chi_sim",
        "id" => "ind",
        "ms" => "msa",
        "vi" => "vie",
        "th" => "tha",
        "ca" => "cat",
        "et" => "est",
        "lv" => "lav",
        "lt" => "lit",
        _ => return trimmed.to_ascii_lowercase(),
    }
    .to_string()
}

#[async_trait]
impl OcrBackend for TesseractOcrBackend {
    fn engine(&self) -> OcrEngine {
        OcrEngine::Tesseract
    }

    async fn extract(&self, request: &ImageOcrRequest) -> Result<OcrDocument> {
        if let Ok(tsv_stdout) = self.run_tesseract(request, TesseractOutputMode::Tsv).await {
            if let Ok(parsed) = self.parse_tsv_output(&tsv_stdout) {
                if !parsed.full_text.trim().is_empty() || !parsed.blocks.is_empty() {
                    return Ok(self.build_document_from_tsv(request, parsed));
                }
            }
        }

        let plain_stdout = self
            .run_tesseract(request, TesseractOutputMode::PlainText)
            .await?;
        let full_text = self.normalize_output_text(&plain_stdout);
        Ok(self.build_document_from_plain_text(request, full_text))
    }
}

fn default_psm_for_target(target: OcrTargetKind) -> Option<u8> {
    match target {
        OcrTargetKind::Screenshot => Some(11), // sparse text can work well for UI screens
        OcrTargetKind::Photo => Some(3),       // fully automatic page segmentation
        OcrTargetKind::GenericImage => None,
    }
}

/// Recognition quality mode for Apple Vision OCR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppleVisionRecognitionLevel {
    /// Faster recognition with potentially lower accuracy.
    Fast,
    /// Higher accuracy recognition (default).
    Accurate,
}

impl AppleVisionRecognitionLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Accurate => "accurate",
        }
    }
}

/// Configuration for the Apple Vision OCR backend.
///
/// The backend runs a small Swift script via `xcrun swift` (default) and parses
/// JSON output into `OcrDocument` blocks.
#[derive(Debug, Clone)]
pub struct AppleVisionOcrConfig {
    /// Command used to run Swift (`xcrun` by default).
    pub binary_path: String,
    /// Prefix args before the script path (defaults to `["swift"]`).
    pub runner_prefix_args: Vec<String>,
    /// Default recognition language codes (e.g., `["en-US"]`).
    pub default_languages: Vec<String>,
    /// Recognition level (`fast` or `accurate`).
    pub recognition_level: AppleVisionRecognitionLevel,
    /// Whether Vision should use language correction.
    pub uses_language_correction: bool,
    /// Trim trailing whitespace/newlines from the output text.
    pub trim_output: bool,
}

impl Default for AppleVisionOcrConfig {
    fn default() -> Self {
        Self {
            binary_path: "xcrun".to_string(),
            runner_prefix_args: vec!["swift".to_string()],
            default_languages: vec!["en-US".to_string()],
            recognition_level: AppleVisionRecognitionLevel::Accurate,
            uses_language_correction: true,
            trim_output: true,
        }
    }
}

impl AppleVisionOcrConfig {
    /// Set the command used to run Swift.
    pub fn with_binary_path(mut self, binary_path: impl Into<String>) -> Self {
        self.binary_path = binary_path.into();
        self
    }

    /// Set the runner prefix args before the script path.
    pub fn with_runner_prefix_args(
        mut self,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.runner_prefix_args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Set default recognition languages.
    pub fn with_default_languages(
        mut self,
        langs: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.default_languages = langs.into_iter().map(Into::into).collect();
        self
    }

    /// Set the recognition level.
    pub fn with_recognition_level(mut self, level: AppleVisionRecognitionLevel) -> Self {
        self.recognition_level = level;
        self
    }

    /// Set whether language correction is enabled.
    pub fn with_language_correction(mut self, enabled: bool) -> Self {
        self.uses_language_correction = enabled;
        self
    }
}

/// Real OCR backend using Apple's Vision framework via a Swift script.
///
/// This backend is macOS-only at runtime. It invokes Swift (`xcrun swift` by
/// default), runs a small Vision OCR script, and parses JSON output with line
/// blocks, confidence, and normalized bounding boxes.
#[derive(Debug, Clone, Default)]
pub struct AppleVisionOcrBackend {
    config: AppleVisionOcrConfig,
}

#[derive(Debug, Deserialize)]
struct AppleVisionScriptResponse {
    #[serde(default)]
    full_text: String,
    #[serde(default)]
    blocks: Vec<AppleVisionScriptBlock>,
}

#[derive(Debug, Deserialize)]
struct AppleVisionScriptBlock {
    text: String,
    confidence: Option<f32>,
    bbox: Option<AppleVisionScriptBoundingBox>,
}

#[derive(Debug, Deserialize)]
struct AppleVisionScriptBoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl AppleVisionOcrBackend {
    /// Create an Apple Vision backend with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an Apple Vision backend with custom configuration.
    pub fn with_config(config: AppleVisionOcrConfig) -> Self {
        Self { config }
    }

    /// Return the configured runner binary path.
    pub fn binary_path(&self) -> &str {
        &self.config.binary_path
    }

    /// Check whether the configured Swift runner is available.
    pub async fn is_available(&self) -> bool {
        self.version().await.is_ok()
    }

    /// Read the Swift runner version string (`xcrun swift --version` by default).
    pub async fn version(&self) -> Result<String> {
        self.ensure_macos_runtime()?;

        let mut args = self.config.runner_prefix_args.clone();
        args.push("--version".to_string());

        let output = Command::new(&self.config.binary_path)
            .args(&args)
            .output()
            .await
            .map_err(|err| {
                Error::ingest(format!(
                    "failed to execute Apple Vision OCR runner '{}': {err}",
                    self.config.binary_path
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(Error::ingest(format!(
                "Apple Vision OCR runner version command failed (status {}): {stderr}",
                output.status
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let version = if stdout.is_empty() {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        } else {
            stdout
        };

        if version.is_empty() {
            return Err(Error::ingest(
                "Apple Vision OCR runner version command returned empty output",
            ));
        }

        Ok(version)
    }

    fn ensure_macos_runtime(&self) -> Result<()> {
        if !cfg!(target_os = "macos") {
            return Err(Error::ingest(
                "Apple Vision OCR backend requires macOS runtime",
            ));
        }
        Ok(())
    }

    fn normalize_languages(&self, request: &ImageOcrRequest) -> String {
        let langs = if request.language_hints.is_empty() {
            self.config.default_languages.clone()
        } else {
            request.language_hints.clone()
        };

        langs
            .into_iter()
            .map(|lang| lang.trim().to_string())
            .filter(|lang| !lang.is_empty())
            .collect::<Vec<_>>()
            .join("+")
    }

    fn build_args(&self, script_path: &std::path::Path, request: &ImageOcrRequest) -> Vec<String> {
        let mut args = self.config.runner_prefix_args.clone();
        args.push(script_path.to_string_lossy().into_owned());
        args.push(request.path.to_string_lossy().into_owned());
        args.push(self.normalize_languages(request));
        args.push(self.config.recognition_level.as_str().to_string());
        args.push(self.config.uses_language_correction.to_string());
        args
    }

    async fn write_temp_script(&self) -> Result<PathBuf> {
        let ts = Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or_else(|| Utc::now().timestamp_micros() * 1000);
        let path = std::env::temp_dir().join(format!(
            "converge_apple_vision_ocr_{}_{}.swift",
            std::process::id(),
            ts
        ));

        tokio::fs::write(&path, apple_vision_swift_script())
            .await
            .map_err(Error::from)?;

        Ok(path)
    }

    async fn run_script(
        &self,
        request: &ImageOcrRequest,
        script_path: &std::path::Path,
    ) -> Result<Vec<u8>> {
        let args = self.build_args(script_path, request);
        let output = Command::new(&self.config.binary_path)
            .args(&args)
            .output()
            .await
            .map_err(|err| {
                Error::ingest(format!(
                    "failed to execute Apple Vision OCR runner '{}': {err}",
                    self.config.binary_path
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(Error::ingest(format!(
                "Apple Vision OCR failed for '{}'(status {}): {}",
                request.path.display(),
                output.status,
                if stderr.is_empty() {
                    "no stderr output".to_string()
                } else {
                    stderr
                }
            )));
        }

        Ok(output.stdout)
    }

    fn parse_json_output(&self, stdout: &[u8]) -> Result<OcrDocumentParts> {
        let response: AppleVisionScriptResponse =
            serde_json::from_slice(stdout).map_err(|err| {
                Error::ingest(format!(
                    "failed to parse Apple Vision OCR JSON output: {err}"
                ))
            })?;

        let mut blocks = Vec::with_capacity(response.blocks.len());
        for block in response.blocks {
            let text = block.text.trim().to_string();
            if text.is_empty() {
                continue;
            }

            let confidence = block.confidence.map(|c| c.clamp(0.0, 1.0));
            let bbox = block.bbox.and_then(|bbox| {
                let normalized = BoundingBox {
                    x: bbox.x.clamp(0.0, 1.0),
                    y: bbox.y.clamp(0.0, 1.0),
                    width: bbox.width.clamp(0.0, 1.0),
                    height: bbox.height.clamp(0.0, 1.0),
                };
                normalized.is_normalized().then_some(normalized)
            });

            blocks.push(OcrTextBlock {
                text,
                confidence,
                bbox,
                kind: OcrBlockKind::Line,
            });
        }

        let full_text = if response.full_text.trim().is_empty() {
            blocks
                .iter()
                .map(|block| block.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        } else if self.config.trim_output {
            response.full_text.trim().to_string()
        } else {
            response.full_text
        };

        Ok(OcrDocumentParts { full_text, blocks })
    }
}

#[async_trait]
impl OcrBackend for AppleVisionOcrBackend {
    fn engine(&self) -> OcrEngine {
        OcrEngine::AppleVision
    }

    async fn extract(&self, request: &ImageOcrRequest) -> Result<OcrDocument> {
        self.ensure_macos_runtime()?;

        let script_path = self.write_temp_script().await?;
        let stdout = self.run_script(request, &script_path).await;
        let _ = tokio::fs::remove_file(&script_path).await;
        let stdout = stdout?;

        let parts = self.parse_json_output(&stdout)?;
        let mut doc = OcrDocument::new(
            self.engine(),
            request.target_kind,
            request.provenance.clone(),
        );
        doc.full_text = parts.full_text;
        doc.blocks = parts.blocks;
        Ok(doc)
    }
}

#[derive(Debug)]
struct OcrDocumentParts {
    full_text: String,
    blocks: Vec<OcrTextBlock>,
}

fn apple_vision_swift_script() -> &'static str {
    r#"
import Foundation
import Vision
import CoreGraphics
import ImageIO

func fail(_ message: String) -> Never {
    if let data = (message + "\n").data(using: .utf8) {
        FileHandle.standardError.write(data)
    }
    exit(1)
}

func boolFromArg(_ raw: String) -> Bool {
    switch raw.lowercased() {
    case "1", "true", "yes", "y":
        return true
    default:
        return false
    }
}

let args = CommandLine.arguments
guard args.count >= 2 else {
    fail("usage: <script> <image_path> [languages] [recognition_level] [language_correction]")
}

let imagePath = args[1]
let languagesArg = args.count > 2 ? args[2] : ""
let recognitionLevelArg = args.count > 3 ? args[3].lowercased() : "accurate"
let languageCorrectionArg = args.count > 4 ? args[4] : "true"

let imageURL = URL(fileURLWithPath: imagePath)
guard let imageSource = CGImageSourceCreateWithURL(imageURL as CFURL, nil) else {
    fail("failed to create image source for \(imagePath)")
}
guard let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
    fail("failed to decode image at \(imagePath)")
}

let request = VNRecognizeTextRequest()
request.recognitionLevel = (recognitionLevelArg == "fast") ? .fast : .accurate
request.usesLanguageCorrection = boolFromArg(languageCorrectionArg)

let languages = languagesArg
    .split(separator: "+")
    .map { String($0).trimmingCharacters(in: .whitespacesAndNewlines) }
    .filter { !$0.isEmpty }
if !languages.isEmpty {
    request.recognitionLanguages = languages
}

do {
    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
    try handler.perform([request])

    let observations = request.results ?? []
    var blocks: [[String: Any]] = []

    for observation in observations {
        guard let candidate = observation.topCandidates(1).first else { continue }
        let text = candidate.string.trimmingCharacters(in: .whitespacesAndNewlines)
        if text.isEmpty { continue }

        let rect = observation.boundingBox
        // Vision uses a bottom-left origin; convert to top-left normalized coordinates.
        let topLeftY = 1.0 - rect.origin.y - rect.size.height

        blocks.append([
            "text": text,
            "confidence": Double(candidate.confidence),
            "bbox": [
                "x": Double(rect.origin.x),
                "y": Double(topLeftY),
                "width": Double(rect.size.width),
                "height": Double(rect.size.height)
            ]
        ])
    }

    let fullText = blocks.compactMap { $0["text"] as? String }.joined(separator: "\n")
    let payload: [String: Any] = [
        "full_text": fullText,
        "blocks": blocks
    ]

    let data = try JSONSerialization.data(withJSONObject: payload, options: [])
    FileHandle.standardOutput.write(data)
} catch {
    fail(String(describing: error))
}
"#
}

fn parse_tesseract_tsv(tsv: &str) -> Result<ParsedTesseractTsv> {
    let mut lines = tsv.lines();
    let header = lines
        .next()
        .ok_or_else(|| Error::ingest("empty tesseract TSV output"))?;

    let header_cols: Vec<&str> = header.split('\t').collect();
    if header_cols.is_empty() {
        return Err(Error::ingest("invalid tesseract TSV header"));
    }

    let mut header_index = HashMap::<String, usize>::new();
    for (idx, col) in header_cols.iter().enumerate() {
        header_index.insert((*col).to_string(), idx);
    }

    for required in [
        "level",
        "page_num",
        "block_num",
        "par_num",
        "line_num",
        "word_num",
        "left",
        "top",
        "width",
        "height",
        "conf",
        "text",
    ] {
        if !header_index.contains_key(required) {
            return Err(Error::ingest(format!(
                "tesseract TSV missing required column '{required}'"
            )));
        }
    }

    let mut pages = HashMap::<u32, TesseractPageDimensions>::new();
    let mut lines_by_key = BTreeMap::<TesseractLineKey, TesseractLineAccumulator>::new();

    for raw_line in lines {
        if raw_line.trim().is_empty() {
            continue;
        }

        let cols = split_tsv_row(raw_line, header_cols.len());

        let level = parse_tsv_u32(&cols, &header_index, "level")?;
        let page_num = parse_tsv_u32(&cols, &header_index, "page_num")?;

        if level == 1 {
            let width = parse_tsv_u32(&cols, &header_index, "width")?;
            let height = parse_tsv_u32(&cols, &header_index, "height")?;
            if width > 0 && height > 0 {
                pages.insert(page_num, TesseractPageDimensions { width, height });
            }
            continue;
        }

        if level != 5 {
            continue;
        }

        let text = tsv_field(&cols, &header_index, "text")?.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let conf_raw = parse_tsv_f32(&cols, &header_index, "conf")?;
        if conf_raw < 0.0 {
            continue;
        }

        let key = TesseractLineKey {
            page_num,
            block_num: parse_tsv_u32(&cols, &header_index, "block_num")?,
            par_num: parse_tsv_u32(&cols, &header_index, "par_num")?,
            line_num: parse_tsv_u32(&cols, &header_index, "line_num")?,
        };

        let word_num = parse_tsv_u32(&cols, &header_index, "word_num")?;
        let left = parse_tsv_u32(&cols, &header_index, "left")?;
        let top = parse_tsv_u32(&cols, &header_index, "top")?;
        let width = parse_tsv_u32(&cols, &header_index, "width")?;
        let height = parse_tsv_u32(&cols, &header_index, "height")?;
        let confidence = (conf_raw / 100.0).clamp(0.0, 1.0);

        use std::collections::btree_map::Entry;
        match lines_by_key.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(TesseractLineAccumulator::new(
                    word_num, text, confidence, left, top, width, height,
                ));
            }
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .add_word(word_num, text, confidence, left, top, width, height);
            }
        }
    }

    let mut blocks = Vec::with_capacity(lines_by_key.len());

    for (key, mut line) in lines_by_key {
        line.words.sort_by_key(|(word_num, _)| *word_num);
        let text = line
            .words
            .into_iter()
            .map(|(_, text)| text)
            .collect::<Vec<_>>()
            .join(" ");

        if text.trim().is_empty() {
            continue;
        }

        let confidence = if line.confidence_count > 0 {
            Some((line.confidence_sum / line.confidence_count as f32).clamp(0.0, 1.0))
        } else {
            None
        };

        let bbox = pages
            .get(&key.page_num)
            .and_then(|page| normalize_bbox(line.left, line.top, line.right, line.bottom, *page));

        blocks.push(OcrTextBlock {
            text,
            confidence,
            bbox,
            kind: OcrBlockKind::Line,
        });
    }

    let full_text = blocks
        .iter()
        .map(|block| block.text.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(ParsedTesseractTsv { full_text, blocks })
}

fn split_tsv_row(line: &str, expected_cols: usize) -> Vec<String> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() <= expected_cols {
        return parts.into_iter().map(str::to_string).collect();
    }

    let mut out = parts[..expected_cols - 1]
        .iter()
        .map(|s| (*s).to_string())
        .collect::<Vec<_>>();
    out.push(parts[expected_cols - 1..].join("\t"));
    out
}

fn tsv_field<'a>(
    cols: &'a [String],
    header_index: &HashMap<String, usize>,
    name: &str,
) -> Result<&'a str> {
    let idx = *header_index
        .get(name)
        .ok_or_else(|| Error::ingest(format!("missing TSV header index for '{name}'")))?;

    cols.get(idx)
        .map(String::as_str)
        .ok_or_else(|| Error::ingest(format!("missing TSV field '{name}' in row")))
}

fn parse_tsv_u32(
    cols: &[String],
    header_index: &HashMap<String, usize>,
    name: &str,
) -> Result<u32> {
    let value = tsv_field(cols, header_index, name)?;
    value.parse::<u32>().map_err(|err| {
        Error::ingest(format!(
            "invalid tesseract TSV integer field '{name}' value '{value}': {err}"
        ))
    })
}

fn parse_tsv_f32(
    cols: &[String],
    header_index: &HashMap<String, usize>,
    name: &str,
) -> Result<f32> {
    let value = tsv_field(cols, header_index, name)?;
    value.parse::<f32>().map_err(|err| {
        Error::ingest(format!(
            "invalid tesseract TSV float field '{name}' value '{value}': {err}"
        ))
    })
}

fn normalize_bbox(
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    page: TesseractPageDimensions,
) -> Option<BoundingBox> {
    if page.width == 0 || page.height == 0 || right <= left || bottom <= top {
        return None;
    }

    let page_w = page.width as f32;
    let page_h = page.height as f32;
    let x = (left as f32 / page_w).clamp(0.0, 1.0);
    let y = (top as f32 / page_h).clamp(0.0, 1.0);
    let width = ((right.saturating_sub(left)) as f32 / page_w).clamp(0.0, 1.0);
    let height = ((bottom.saturating_sub(top)) as f32 / page_h).clamp(0.0, 1.0);

    let bbox = BoundingBox {
        x,
        y,
        width,
        height,
    };

    bbox.is_normalized().then_some(bbox)
}

/// Fixture-driven OCR backend for tests and local scaffolding.
///
/// This backend returns pre-registered OCR results keyed by exact file path,
/// file name, or file stem (in that order). It is intended for ingestion
/// pipeline development when native OCR runtimes are unavailable.
#[derive(Debug, Clone)]
pub struct FixtureOcrBackend {
    engine: OcrEngine,
    documents: HashMap<String, OcrDocument>,
    default_document: Option<OcrDocument>,
}

impl FixtureOcrBackend {
    /// Create an empty fixture backend using the mock engine label.
    pub fn new() -> Self {
        Self {
            engine: OcrEngine::Mock,
            documents: HashMap::new(),
            default_document: None,
        }
    }

    /// Create an empty fixture backend with a custom engine label.
    pub fn with_engine(engine: OcrEngine) -> Self {
        Self {
            engine,
            documents: HashMap::new(),
            default_document: None,
        }
    }

    /// Register a fixture document for a given path-like lookup key.
    ///
    /// The key can be:
    /// - exact path string
    /// - file name (e.g., `Screenshot.png`)
    /// - file stem (e.g., `Screenshot`)
    pub fn with_document(mut self, key: impl Into<String>, document: OcrDocument) -> Self {
        self.documents.insert(key.into(), document);
        self
    }

    /// Register a default fixture returned when no key-specific fixture exists.
    pub fn with_default_document(mut self, document: OcrDocument) -> Self {
        self.default_document = Some(document);
        self
    }

    fn lookup(&self, path: &std::path::Path) -> Option<&OcrDocument> {
        let exact = path.to_string_lossy();
        if let Some(doc) = self.documents.get(exact.as_ref()) {
            return Some(doc);
        }

        if let Some(file_name) = path.file_name().and_then(|name| name.to_str())
            && let Some(doc) = self.documents.get(file_name)
        {
            return Some(doc);
        }

        if let Some(file_stem) = path.file_stem().and_then(|stem| stem.to_str())
            && let Some(doc) = self.documents.get(file_stem)
        {
            return Some(doc);
        }

        self.default_document.as_ref()
    }
}

impl Default for FixtureOcrBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OcrBackend for FixtureOcrBackend {
    fn engine(&self) -> OcrEngine {
        self.engine.clone()
    }

    async fn extract(&self, request: &ImageOcrRequest) -> Result<OcrDocument> {
        let Some(template) = self.lookup(&request.path) else {
            return Err(Error::ingest(format!(
                "no OCR fixture registered for path '{}'",
                request.path.display()
            )));
        };

        let mut doc = template.clone();
        doc.engine = self.engine();
        doc.target_kind = request.target_kind;
        doc.provenance = request.provenance.clone();
        doc.extracted_at = Utc::now();
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{SourceKind, SourceProvenance};

    #[test]
    fn effective_text_falls_back_to_blocks() {
        let provenance = SourceProvenance::new(SourceKind::Screenshot, "file:///shot.png");
        let mut doc = OcrDocument::new(OcrEngine::Mock, OcrTargetKind::Screenshot, provenance);
        doc.blocks.push(OcrTextBlock::line("First line"));
        doc.blocks.push(OcrTextBlock::line("Second line"));

        assert_eq!(doc.effective_text(), "First line\nSecond line");
    }

    #[test]
    fn filtered_text_respects_confidence() {
        let provenance = SourceProvenance::new(SourceKind::Photo, "file:///photo.jpg");
        let mut doc = OcrDocument::new(OcrEngine::Mock, OcrTargetKind::Photo, provenance);
        doc.blocks.push(OcrTextBlock {
            text: "keep".into(),
            confidence: Some(0.95),
            bbox: None,
            kind: OcrBlockKind::Word,
        });
        doc.blocks.push(OcrTextBlock {
            text: "drop".into(),
            confidence: Some(0.20),
            bbox: None,
            kind: OcrBlockKind::Word,
        });

        assert_eq!(doc.filtered_text(0.5), "keep");
    }

    #[test]
    fn bounding_box_validation_checks_normalized_range() {
        let valid = BoundingBox {
            x: 0.1,
            y: 0.2,
            width: 0.3,
            height: 0.4,
        };
        let invalid = BoundingBox {
            x: 0.8,
            y: 0.9,
            width: 0.4,
            height: 0.2,
        };

        assert!(valid.is_normalized());
        assert!(!invalid.is_normalized());
    }

    #[tokio::test]
    async fn fixture_backend_uses_filename_lookup_and_overrides_provenance() {
        let fixture_provenance = SourceProvenance::new(SourceKind::Screenshot, "fixture://shot");
        let mut fixture_doc = OcrDocument::new(
            OcrEngine::Mock,
            OcrTargetKind::Screenshot,
            fixture_provenance,
        );
        fixture_doc.full_text = "fixture text".into();

        let backend = FixtureOcrBackend::new().with_document("test-shot.png", fixture_doc);

        let request_provenance =
            SourceProvenance::new(SourceKind::Screenshot, "file:///tmp/test-shot.png")
                .with_origin_id("shot-123");
        let request = ImageOcrRequest::new(
            "/tmp/test-shot.png",
            OcrTargetKind::Screenshot,
            request_provenance.clone(),
        );

        let response = backend.extract(&request).await.unwrap();
        assert_eq!(response.full_text, "fixture text");
        assert_eq!(
            response.provenance.source_uri,
            request_provenance.source_uri
        );
        assert_eq!(response.provenance.origin_id, request_provenance.origin_id);
    }

    #[test]
    fn tesseract_backend_builds_expected_command_args() {
        let backend = TesseractOcrBackend::with_config(
            TesseractOcrConfig::default()
                .with_default_languages(["eng", "deu"])
                .with_oem(1)
                .with_psm(6)
                .with_extra_arg("--dpi")
                .with_extra_arg("300"),
        );

        let provenance = SourceProvenance::new(SourceKind::Screenshot, "file:///shot.png");
        let request = ImageOcrRequest::new("/tmp/shot.png", OcrTargetKind::Screenshot, provenance)
            .with_language_hint("fra");

        let args = backend.build_args(&request);
        assert_eq!(
            args,
            vec![
                "/tmp/shot.png",
                "stdout",
                "-l",
                "fra",
                "--oem",
                "1",
                "--psm",
                "6",
                "--dpi",
                "300",
            ]
        );
    }

    #[test]
    fn tesseract_backend_normalizes_and_dedupes_language_hints() {
        let backend = TesseractOcrBackend::new();
        let provenance = SourceProvenance::new(SourceKind::Screenshot, "file:///shot.png");
        let request = ImageOcrRequest::new("/tmp/shot.png", OcrTargetKind::Screenshot, provenance)
            .with_language_hint("en")
            .with_language_hint("en-US")
            .with_language_hint("de-DE")
            .with_language_hint("chi_sim");

        let args = backend.build_args(&request);
        let lang_idx = args.iter().position(|arg| arg == "-l").unwrap();
        assert_eq!(args[lang_idx + 1], "eng+deu+chi_sim");
    }

    #[test]
    fn tesseract_backend_builds_tsv_command_args() {
        let backend = TesseractOcrBackend::new();
        let provenance = SourceProvenance::new(SourceKind::Photo, "file:///photo.jpg");
        let request = ImageOcrRequest::new("/tmp/photo.jpg", OcrTargetKind::Photo, provenance);

        let args = backend.build_tsv_args(&request);
        assert!(args.ends_with(&["tsv".to_string()]));
        assert_eq!(args[0], "/tmp/photo.jpg");
        assert_eq!(args[1], "stdout");
    }

    #[test]
    fn tesseract_backend_synthesizes_line_blocks_from_plain_text() {
        let backend = TesseractOcrBackend::new();
        let blocks = backend.synthesize_blocks("Line 1\n\n Line 2 \n");
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, "Line 1");
        assert_eq!(blocks[1].text, "Line 2");
        assert!(blocks.iter().all(|b| b.kind == OcrBlockKind::Line));
    }

    #[test]
    fn parses_tesseract_tsv_into_line_blocks_with_bbox_and_confidence() {
        let tsv = concat!(
            "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n",
            "1\t1\t0\t0\t0\t0\t0\t0\t200\t100\t-1\t\n",
            "4\t1\t1\t1\t1\t0\t10\t10\t100\t20\t-1\t\n",
            "5\t1\t1\t1\t1\t1\t10\t10\t40\t20\t92\tHello\n",
            "5\t1\t1\t1\t1\t2\t60\t10\t50\t20\t88\tworld\n",
            "4\t1\t1\t1\t2\t0\t10\t40\t80\t20\t-1\t\n",
            "5\t1\t1\t1\t2\t1\t10\t40\t80\t20\t75\tSecond\n",
        );

        let parsed = parse_tesseract_tsv(tsv).unwrap();
        assert_eq!(parsed.blocks.len(), 2);
        assert_eq!(parsed.full_text, "Hello world\nSecond");

        let first = &parsed.blocks[0];
        assert_eq!(first.kind, OcrBlockKind::Line);
        assert_eq!(first.text, "Hello world");
        let conf = first.confidence.unwrap();
        assert!((conf - 0.90).abs() < 1e-6, "unexpected confidence: {conf}");

        let bbox = first.bbox.unwrap();
        assert!((bbox.x - 0.05).abs() < 1e-6);
        assert!((bbox.y - 0.10).abs() < 1e-6);
        assert!((bbox.width - 0.50).abs() < 1e-6);
        assert!((bbox.height - 0.20).abs() < 1e-6);
    }

    #[test]
    fn tesseract_parser_rejects_missing_header_columns() {
        let err = parse_tesseract_tsv("level\tpage_num\n1\t1\n").unwrap_err();
        assert!(err.to_string().contains("missing required column"));
    }

    #[test]
    fn apple_vision_backend_builds_runner_args() {
        let backend = AppleVisionOcrBackend::with_config(
            AppleVisionOcrConfig::default()
                .with_binary_path("xcrun")
                .with_runner_prefix_args(["swift"])
                .with_default_languages(["en-US"])
                .with_recognition_level(AppleVisionRecognitionLevel::Fast)
                .with_language_correction(false),
        );

        let provenance = SourceProvenance::new(SourceKind::Photo, "file:///photo.jpg");
        let request = ImageOcrRequest::new("/tmp/photo.jpg", OcrTargetKind::Photo, provenance)
            .with_language_hint("en-US")
            .with_language_hint("de-DE");

        let args = backend.build_args(std::path::Path::new("/tmp/vision.swift"), &request);
        assert_eq!(
            args,
            vec![
                "swift",
                "/tmp/vision.swift",
                "/tmp/photo.jpg",
                "en-US+de-DE",
                "fast",
                "false",
            ]
        );
    }

    #[test]
    fn parses_apple_vision_json_into_line_blocks() {
        let backend = AppleVisionOcrBackend::with_config(AppleVisionOcrConfig::default());
        let json = br#"{
            "full_text": "Hello\nWorld",
            "blocks": [
                {
                    "text": "Hello",
                    "confidence": 0.93,
                    "bbox": { "x": 0.1, "y": 0.2, "width": 0.3, "height": 0.1 }
                },
                {
                    "text": "World",
                    "confidence": 1.1,
                    "bbox": { "x": 0.4, "y": 0.5, "width": 0.4, "height": 0.2 }
                }
            ]
        }"#;

        let parsed = backend.parse_json_output(json).unwrap();
        assert_eq!(parsed.full_text, "Hello\nWorld");
        assert_eq!(parsed.blocks.len(), 2);
        assert_eq!(parsed.blocks[0].kind, OcrBlockKind::Line);
        assert_eq!(parsed.blocks[0].confidence, Some(0.93));
        assert_eq!(parsed.blocks[1].confidence, Some(1.0)); // clamped
        assert!(parsed.blocks[0].bbox.unwrap().is_normalized());
    }

    #[test]
    fn apple_vision_parser_rejects_invalid_json() {
        let backend = AppleVisionOcrBackend::new();
        let err = backend
            .parse_json_output(br#"{"blocks":"nope"}"#)
            .unwrap_err();
        assert!(err.to_string().contains("parse Apple Vision OCR JSON"));
    }

    #[test]
    fn default_psm_matches_target_kind() {
        assert_eq!(default_psm_for_target(OcrTargetKind::Screenshot), Some(11));
        assert_eq!(default_psm_for_target(OcrTargetKind::Photo), Some(3));
        assert_eq!(default_psm_for_target(OcrTargetKind::GenericImage), None);
    }
}
