//! Photo OCR ingestion scaffold.
//!
//! This module turns photo image files into searchable text chunks using an
//! `OcrBackend` implementation (e.g., Apple Vision, Tesseract, or fixtures).
//! It preserves photo-specific file metadata and produces a `KnowledgeEntry`
//! scaffold for indexing pipelines.

use crate::core::KnowledgeEntry;
use crate::error::{Error, Result};
use crate::ingest::{
    AppleVisionOcrBackend, AppleVisionOcrConfig, ImageOcrRequest, OcrBackend, OcrBlockKind,
    OcrDocument, OcrTargetKind, SourceKind, SourceProvenance, TesseractOcrBackend,
    TesseractOcrConfig,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A photo text chunk derived from OCR output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoTextChunk {
    /// The text content for this chunk.
    pub content: String,
    /// The OCR block kind this chunk originated from.
    pub block_kind: OcrBlockKind,
    /// OCR confidence when available.
    pub confidence: Option<f32>,
    /// Relative weight hint for later ranking/indexing logic.
    pub weight: f32,
}

/// Structured photo OCR ingestion result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoDocument {
    /// Original photo path.
    pub path: PathBuf,
    /// Human-readable title inferred from path/metadata.
    pub title: String,
    /// OCR extraction result.
    pub ocr: OcrDocument,
    /// Photo-specific metadata.
    pub metadata: HashMap<String, String>,
    /// Text chunks prepared for indexing.
    pub chunks: Vec<PhotoTextChunk>,
}

impl PhotoDocument {
    /// Best-effort text for indexing, concatenating chunks in order.
    pub fn indexing_text(&self) -> String {
        self.chunks
            .iter()
            .map(|chunk| chunk.content.trim())
            .filter(|content| !content.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert this photo OCR result into a knowledge entry scaffold.
    pub fn to_knowledge_entry(&self) -> KnowledgeEntry {
        let content = self.indexing_text();
        let mut entry = KnowledgeEntry::new(self.title.clone(), content)
            .with_category("Photo")
            .with_tags(["photo", "ocr", "image"])
            .with_source(self.path.to_string_lossy().into_owned());

        for (key, value) in self.ocr.provenance.metadata_pairs() {
            entry = entry.with_metadata(key, value);
        }

        for (key, value) in &self.metadata {
            entry = entry.with_metadata(format!("photo.{key}"), value.clone());
        }

        entry
            .with_metadata("photo.chunk_count", self.chunks.len().to_string())
            .with_metadata(
                "photo.ocr_engine",
                match self.ocr.engine {
                    crate::ingest::OcrEngine::AppleVision => "apple_vision",
                    crate::ingest::OcrEngine::Tesseract => "tesseract",
                    crate::ingest::OcrEngine::Mock => "mock",
                    crate::ingest::OcrEngine::External => "external",
                },
            )
    }
}

/// Configuration for photo ingestion.
#[derive(Debug, Clone)]
pub struct PhotoIngesterConfig {
    /// Language hints passed to OCR backends by default.
    pub language_hints: Vec<String>,
    /// Drop OCR blocks below this confidence when set.
    pub min_confidence: Option<f32>,
    /// Weight applied to all OCR-derived content blocks.
    pub content_weight: f32,
}

impl Default for PhotoIngesterConfig {
    fn default() -> Self {
        Self {
            language_hints: vec!["en".to_string()],
            min_confidence: Some(0.5),
            content_weight: 1.0,
        }
    }
}

impl PhotoIngesterConfig {
    /// Disable default confidence filtering.
    pub fn without_confidence_filter(mut self) -> Self {
        self.min_confidence = None;
        self
    }
}

/// Photo ingester backed by an OCR backend implementation.
#[derive(Clone)]
pub struct PhotoIngester {
    backend: Arc<dyn OcrBackend>,
    config: PhotoIngesterConfig,
}

impl PhotoIngester {
    /// Create a new photo ingester with default config.
    pub fn new(backend: Arc<dyn OcrBackend>) -> Self {
        Self {
            backend,
            config: PhotoIngesterConfig::default(),
        }
    }

    /// Create a photo ingester with custom config.
    pub fn with_config(backend: Arc<dyn OcrBackend>, config: PhotoIngesterConfig) -> Self {
        Self { backend, config }
    }

    /// Create a photo ingester backed by the real Tesseract OCR backend.
    pub fn with_tesseract() -> Self {
        Self::new(Arc::new(TesseractOcrBackend::new()))
    }

    /// Create a photo ingester backed by Tesseract with custom OCR config.
    pub fn with_tesseract_config(ocr_config: TesseractOcrConfig) -> Self {
        Self::new(Arc::new(TesseractOcrBackend::with_config(ocr_config)))
    }

    /// Create a photo ingester with custom ingestion + Tesseract OCR configs.
    pub fn with_tesseract_and_config(
        ocr_config: TesseractOcrConfig,
        config: PhotoIngesterConfig,
    ) -> Self {
        Self::with_config(
            Arc::new(TesseractOcrBackend::with_config(ocr_config)),
            config,
        )
    }

    /// Create a photo ingester backed by Apple's Vision OCR (macOS-only at runtime).
    pub fn with_apple_vision() -> Self {
        Self::new(Arc::new(AppleVisionOcrBackend::new()))
    }

    /// Create a photo ingester backed by Apple Vision with custom OCR config.
    pub fn with_apple_vision_config(ocr_config: AppleVisionOcrConfig) -> Self {
        Self::new(Arc::new(AppleVisionOcrBackend::with_config(ocr_config)))
    }

    /// Create a photo ingester with custom ingestion + Apple Vision OCR configs.
    pub fn with_apple_vision_and_config(
        ocr_config: AppleVisionOcrConfig,
        config: PhotoIngesterConfig,
    ) -> Self {
        Self::with_config(
            Arc::new(AppleVisionOcrBackend::with_config(ocr_config)),
            config,
        )
    }

    /// Ingest a photo image file into structured OCR chunks.
    pub async fn ingest_file(&self, path: &Path) -> Result<PhotoDocument> {
        if !path.exists() {
            return Err(Error::ingest(format!(
                "photo file does not exist: {}",
                path.display()
            )));
        }

        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(Error::ingest(format!(
                "photo path is not a file: {}",
                path.display()
            )));
        }

        let provenance = self.build_provenance(path, &metadata);
        let mut request =
            ImageOcrRequest::new(path.to_path_buf(), OcrTargetKind::Photo, provenance);
        request.min_confidence = self.config.min_confidence;
        request.language_hints = self.config.language_hints.clone();

        let ocr = self.backend.extract(&request).await?;
        let photo_metadata = self.build_photo_metadata(path, &metadata);
        let chunks = self.build_chunks(&ocr);
        let title = infer_title(path);

        Ok(PhotoDocument {
            path: path.to_path_buf(),
            title,
            ocr,
            metadata: photo_metadata,
            chunks,
        })
    }

    /// Ingest a photo and convert directly to a knowledge entry scaffold.
    pub async fn ingest_as_entry(&self, path: &Path) -> Result<KnowledgeEntry> {
        let doc = self.ingest_file(path).await?;
        Ok(doc.to_knowledge_entry())
    }

    fn build_provenance(&self, path: &Path, fs_meta: &fs::Metadata) -> SourceProvenance {
        let mut provenance =
            SourceProvenance::new(SourceKind::Photo, path.to_string_lossy().into_owned())
                .with_metadata("filename", file_name_string(path))
                .with_metadata("extension", file_extension_string(path))
                .with_metadata("file_size_bytes", fs_meta.len().to_string());

        if let Some(captured_at) = system_time_to_utc(fs_meta.modified().ok()) {
            provenance = provenance.with_captured_at(captured_at);
        }

        provenance
    }

    fn build_photo_metadata(&self, path: &Path, fs_meta: &fs::Metadata) -> HashMap<String, String> {
        let mut out = HashMap::new();
        out.insert("filename".to_string(), file_name_string(path));
        out.insert("extension".to_string(), file_extension_string(path));
        out.insert("file_size_bytes".to_string(), fs_meta.len().to_string());
        if let Some(min_conf) = self.config.min_confidence {
            out.insert("min_confidence".to_string(), min_conf.to_string());
        }
        out
    }

    fn build_chunks(&self, ocr: &OcrDocument) -> Vec<PhotoTextChunk> {
        if ocr.blocks.is_empty() {
            let text = ocr.effective_text();
            if text.trim().is_empty() {
                return Vec::new();
            }
            return vec![PhotoTextChunk {
                content: text,
                block_kind: OcrBlockKind::Unknown,
                confidence: None,
                weight: self.config.content_weight,
            }];
        }

        ocr.blocks
            .iter()
            .filter(|block| {
                self.config
                    .min_confidence
                    .is_none_or(|min| block.confidence.unwrap_or(1.0) >= min)
            })
            .filter_map(|block| {
                let content = block.text.trim();
                if content.is_empty() {
                    return None;
                }

                Some(PhotoTextChunk {
                    content: content.to_string(),
                    block_kind: block.kind,
                    confidence: block.confidence,
                    weight: self.config.content_weight,
                })
            })
            .collect()
    }
}

fn infer_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(clean_photo_title)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Photo".to_string())
}

fn clean_photo_title(raw: &str) -> String {
    raw.replace('_', " ").trim().to_string()
}

fn file_name_string(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn file_extension_string(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_string()
}

fn system_time_to_utc(time: Option<std::time::SystemTime>) -> Option<DateTime<Utc>> {
    time.map(DateTime::<Utc>::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::{FixtureOcrBackend, OcrDocument, OcrEngine, OcrTextBlock};
    use tempfile::TempDir;

    async fn create_temp_photo(temp_dir: &TempDir, name: &str) -> PathBuf {
        let path = temp_dir.path().join(name);
        tokio::fs::write(&path, b"fakejpg").await.unwrap();
        path
    }

    #[tokio::test]
    async fn ingest_photo_builds_chunks_and_entry_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_photo(&temp_dir, "IMG_2042.jpg").await;

        let fixture_provenance = SourceProvenance::new(SourceKind::Photo, "fixture://p1");
        let mut fixture_doc =
            OcrDocument::new(OcrEngine::Mock, OcrTargetKind::Photo, fixture_provenance);
        fixture_doc.blocks = vec![
            OcrTextBlock {
                text: "Menu".into(),
                confidence: Some(0.98),
                bbox: None,
                kind: OcrBlockKind::Line,
            },
            OcrTextBlock {
                text: "blurry".into(),
                confidence: Some(0.20),
                bbox: None,
                kind: OcrBlockKind::Word,
            },
        ];

        let backend = Arc::new(FixtureOcrBackend::new().with_document("IMG_2042.jpg", fixture_doc));
        let ingester = PhotoIngester::new(backend);

        let doc = ingester.ingest_file(&path).await.unwrap();
        assert_eq!(doc.title, "IMG 2042");
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.chunks[0].content, "Menu");
        assert!(doc.indexing_text().contains("Menu"));

        let entry = doc.to_knowledge_entry();
        assert_eq!(entry.category.as_deref(), Some("Photo"));
        assert_eq!(entry.metadata.get("source.kind"), Some("photo"));
        assert_eq!(entry.metadata.get("photo.chunk_count"), Some("1"));
        assert_eq!(entry.metadata.get("photo.ocr_engine"), Some("mock"));
    }

    #[tokio::test]
    async fn ingest_photo_falls_back_to_full_text_when_no_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_photo(&temp_dir, "receipt.png").await;

        let fixture_provenance = SourceProvenance::new(SourceKind::Photo, "fixture://p2");
        let mut fixture_doc =
            OcrDocument::new(OcrEngine::Mock, OcrTargetKind::Photo, fixture_provenance);
        fixture_doc.full_text = "Receipt total 12.95".into();

        let backend = Arc::new(FixtureOcrBackend::new().with_default_document(fixture_doc));
        let ingester = PhotoIngester::new(backend);

        let doc = ingester.ingest_file(&path).await.unwrap();
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.chunks[0].content, "Receipt total 12.95");
    }

    #[tokio::test]
    async fn ingest_photo_can_disable_confidence_filter() {
        let temp_dir = TempDir::new().unwrap();
        let path = create_temp_photo(&temp_dir, "label.jpg").await;

        let fixture_provenance = SourceProvenance::new(SourceKind::Photo, "fixture://p3");
        let mut fixture_doc =
            OcrDocument::new(OcrEngine::Mock, OcrTargetKind::Photo, fixture_provenance);
        fixture_doc.blocks = vec![OcrTextBlock {
            text: "low".into(),
            confidence: Some(0.1),
            bbox: None,
            kind: OcrBlockKind::Word,
        }];

        let backend = Arc::new(FixtureOcrBackend::new().with_default_document(fixture_doc));
        let config = PhotoIngesterConfig::default().without_confidence_filter();
        let ingester = PhotoIngester::with_config(backend, config);

        let doc = ingester.ingest_file(&path).await.unwrap();
        assert_eq!(doc.chunks.len(), 1);
        assert_eq!(doc.chunks[0].content, "low");
    }

    #[tokio::test]
    async fn ingest_photo_errors_for_missing_file() {
        let backend = Arc::new(FixtureOcrBackend::new());
        let ingester = PhotoIngester::new(backend);

        let err = ingester
            .ingest_file(Path::new("/tmp/does-not-exist-photo.jpg"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
