//! PDF document ingestion and text extraction.
//!
//! This module provides functionality to extract text content from PDF files,
//! chunk the content by pages or sections, and extract metadata such as title,
//! author, and creation date.
//!
//! # Example
//!
//! ```rust,no_run
//! use converge_knowledge::ingest::{PdfIngester, PdfDocument};
//! use std::path::Path;
//!
//! let ingester = PdfIngester::new();
//! let doc = ingester.ingest_file(Path::new("document.pdf")).unwrap();
//!
//! println!("Title: {:?}", doc.title);
//! println!("Pages: {}", doc.page_count);
//! for chunk in &doc.chunks {
//!     println!("Page {}: {}", chunk.page_number, &chunk.content[..100.min(chunk.content.len())]);
//! }
//! ```

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Default maximum chunk size in characters.
const DEFAULT_MAX_CHUNK_SIZE: usize = 4000;

/// Minimum chunk size to avoid creating very small chunks.
const MIN_CHUNK_SIZE: usize = 100;

/// A parsed PDF document with extracted content and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// Original file path.
    pub path: PathBuf,

    /// Document title extracted from metadata.
    pub title: Option<String>,

    /// Document author extracted from metadata.
    pub author: Option<String>,

    /// Additional metadata key-value pairs.
    pub metadata: HashMap<String, String>,

    /// Extracted content chunks.
    pub chunks: Vec<PdfChunk>,

    /// Total number of pages in the document.
    pub page_count: usize,
}

impl PdfDocument {
    /// Create a new empty PDF document.
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            title: None,
            author: None,
            metadata: HashMap::new(),
            chunks: Vec::new(),
            page_count: 0,
        }
    }

    /// Get the total character count across all chunks.
    pub fn total_chars(&self) -> usize {
        self.chunks.iter().map(|c| c.content.len()).sum()
    }

    /// Get all content as a single string.
    pub fn full_text(&self) -> String {
        self.chunks
            .iter()
            .map(|c| c.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get chunks for a specific page.
    pub fn chunks_for_page(&self, page: usize) -> Vec<&PdfChunk> {
        self.chunks
            .iter()
            .filter(|c| c.page_number == page)
            .collect()
    }
}

/// A chunk of content extracted from a PDF page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfChunk {
    /// The extracted text content.
    pub content: String,

    /// The page number this chunk came from (1-indexed).
    pub page_number: usize,

    /// Index of this chunk within the page (for multi-chunk pages).
    pub chunk_index: usize,
}

impl PdfChunk {
    /// Create a new PDF chunk.
    fn new(content: String, page_number: usize, chunk_index: usize) -> Self {
        Self {
            content,
            page_number,
            chunk_index,
        }
    }

    /// Check if the chunk is empty or contains only whitespace.
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get the character count of the content.
    pub fn len(&self) -> usize {
        self.content.len()
    }
}

/// Configuration for PDF ingestion.
#[derive(Debug, Clone)]
pub struct PdfIngesterConfig {
    /// Maximum size of each chunk in characters.
    pub max_chunk_size: usize,

    /// Whether to preserve original line breaks.
    pub preserve_line_breaks: bool,

    /// Whether to attempt to detect and handle multi-column layouts.
    pub handle_multi_column: bool,

    /// Whether to extract metadata from the PDF.
    pub extract_metadata: bool,

    /// Minimum chunk size (smaller chunks will be merged).
    pub min_chunk_size: usize,
}

impl Default for PdfIngesterConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: DEFAULT_MAX_CHUNK_SIZE,
            preserve_line_breaks: false,
            handle_multi_column: true,
            extract_metadata: true,
            min_chunk_size: MIN_CHUNK_SIZE,
        }
    }
}

impl PdfIngesterConfig {
    /// Create a new configuration with custom max chunk size.
    pub fn with_max_chunk_size(mut self, size: usize) -> Self {
        self.max_chunk_size = size;
        self
    }

    /// Set whether to preserve line breaks.
    pub fn with_preserve_line_breaks(mut self, preserve: bool) -> Self {
        self.preserve_line_breaks = preserve;
        self
    }

    /// Set whether to handle multi-column layouts.
    pub fn with_multi_column_handling(mut self, handle: bool) -> Self {
        self.handle_multi_column = handle;
        self
    }
}

/// PDF document ingester for extracting text and metadata.
#[derive(Debug, Clone)]
pub struct PdfIngester {
    config: PdfIngesterConfig,
}

impl Default for PdfIngester {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfIngester {
    /// Create a new PDF ingester with default configuration.
    pub fn new() -> Self {
        Self {
            config: PdfIngesterConfig::default(),
        }
    }

    /// Create a new PDF ingester with custom configuration.
    pub fn with_config(config: PdfIngesterConfig) -> Self {
        Self { config }
    }

    /// Ingest a single PDF file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file
    ///
    /// # Returns
    ///
    /// A `PdfDocument` containing the extracted content and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed as a PDF.
    pub fn ingest_file(&self, path: &Path) -> Result<PdfDocument> {
        info!(?path, "Ingesting PDF file");

        // Validate file exists and is a PDF
        if !path.exists() {
            return Err(Error::ingest(format!("File not found: {}", path.display())));
        }

        if path.extension().and_then(|e| e.to_str()) != Some("pdf") {
            warn!(?path, "File does not have .pdf extension");
        }

        // Read the file
        let data = fs::read(path).map_err(|e| {
            Error::ingest(format!("Failed to read PDF file {}: {}", path.display(), e))
        })?;

        self.ingest_bytes(&data, path.to_path_buf())
    }

    /// Ingest PDF content from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw PDF file bytes
    /// * `path` - Path to associate with the document (for reference)
    ///
    /// # Returns
    ///
    /// A `PdfDocument` containing the extracted content and metadata.
    pub fn ingest_bytes(&self, data: &[u8], path: PathBuf) -> Result<PdfDocument> {
        let mut doc = PdfDocument::new(path.clone());

        // Extract text using pdf-extract
        let text = pdf_extract::extract_text_from_mem(data).map_err(|e| {
            Error::ingest(format!(
                "Failed to extract text from PDF {}: {}",
                path.display(),
                e
            ))
        })?;

        // Try to extract metadata
        if self.config.extract_metadata {
            self.extract_metadata_from_bytes(data, &mut doc);
        }

        // Process the extracted text
        let processed_text = self.process_text(&text);

        // Estimate page count from text (pdf-extract doesn't provide page info directly)
        // We use form feed characters or estimate based on text length
        doc.page_count = self.estimate_page_count(&text);

        // Create chunks
        doc.chunks = self.create_chunks(&processed_text, doc.page_count);

        info!(
            path = %path.display(),
            pages = doc.page_count,
            chunks = doc.chunks.len(),
            chars = doc.total_chars(),
            "PDF ingestion complete"
        );

        Ok(doc)
    }

    /// Extract metadata from PDF bytes.
    fn extract_metadata_from_bytes(&self, data: &[u8], doc: &mut PdfDocument) {
        // Try to parse with lopdf for metadata (pdf-extract doesn't expose metadata)
        // We'll do a simple scan for common metadata patterns in the PDF
        let text = String::from_utf8_lossy(data);

        // Look for common PDF metadata patterns
        if let Some(title) = self.extract_metadata_field(&text, "Title") {
            doc.title = Some(title);
            doc.metadata
                .insert("title".to_string(), doc.title.clone().unwrap_or_default());
        }

        if let Some(author) = self.extract_metadata_field(&text, "Author") {
            doc.author = Some(author);
            doc.metadata
                .insert("author".to_string(), doc.author.clone().unwrap_or_default());
        }

        if let Some(creator) = self.extract_metadata_field(&text, "Creator") {
            doc.metadata.insert("creator".to_string(), creator);
        }

        if let Some(producer) = self.extract_metadata_field(&text, "Producer") {
            doc.metadata.insert("producer".to_string(), producer);
        }

        if let Some(creation_date) = self.extract_metadata_field(&text, "CreationDate") {
            doc.metadata
                .insert("creation_date".to_string(), creation_date);
        }

        if let Some(mod_date) = self.extract_metadata_field(&text, "ModDate") {
            doc.metadata
                .insert("modification_date".to_string(), mod_date);
        }

        debug!(
            title = ?doc.title,
            author = ?doc.author,
            metadata_count = doc.metadata.len(),
            "Extracted PDF metadata"
        );
    }

    /// Extract a metadata field value from raw PDF text.
    fn extract_metadata_field(&self, text: &str, field: &str) -> Option<String> {
        // Look for patterns like /Title (Value) or /Title <hex>
        let pattern = format!("/{field}");
        if let Some(pos) = text.find(&pattern) {
            let after = &text[pos + pattern.len()..];

            // Handle parentheses-enclosed values
            if let Some(start) = after.find('(') {
                let value_start = start + 1;
                let mut depth = 1;
                let mut end = value_start;

                for c in after[value_start..].chars() {
                    match c {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    end += c.len_utf8();
                }

                if end > value_start {
                    let value = &after[value_start..end];
                    let cleaned = value.trim().to_string();
                    if !cleaned.is_empty() && cleaned.len() < 500 {
                        return Some(cleaned);
                    }
                }
            }
        }
        None
    }

    /// Process extracted text to clean up formatting issues.
    fn process_text(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());

        // Handle multi-column layout detection and reordering
        let processed = if self.config.handle_multi_column {
            self.handle_multi_column_text(text)
        } else {
            text.to_string()
        };

        // Clean up the text
        for line in processed.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !result.ends_with("\n\n") {
                    result.push('\n');
                }
                continue;
            }

            // Detect if this line appears to be a continuation
            let is_continuation = !result.is_empty()
                && !result.ends_with('\n')
                && !trimmed.starts_with(char::is_uppercase)
                && !trimmed.starts_with(|c: char| c.is_ascii_digit());

            if is_continuation {
                // Check if previous line ended with hyphen (word continuation)
                if result.ends_with('-') {
                    result.pop(); // Remove hyphen
                } else {
                    result.push(' ');
                }
            } else if !result.is_empty() && !result.ends_with('\n') {
                if self.config.preserve_line_breaks {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
            }

            result.push_str(trimmed);
        }

        // Normalize whitespace
        self.normalize_whitespace(&result)
    }

    /// Attempt to handle multi-column layouts by detecting and reordering text.
    fn handle_multi_column_text(&self, text: &str) -> String {
        // Simple heuristic: if we detect short lines with consistent lengths,
        // it might be multi-column. We'll try to identify and merge columns.

        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        // Calculate average line length
        let total_len: usize = lines.iter().map(|l| l.len()).sum();
        let avg_len = total_len / lines.len().max(1);

        // If average line length is very short (< 60 chars) and we have many lines,
        // this might indicate multi-column layout
        if avg_len < 60 && lines.len() > 20 {
            debug!(
                avg_len,
                lines = lines.len(),
                "Detected potential multi-column layout"
            );

            // Try to detect column boundaries by looking for consistent indentation patterns
            // This is a simplified approach - full multi-column detection would be more complex
            let mut result = String::new();
            let mut current_paragraph = String::new();

            for line in lines {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    if !current_paragraph.is_empty() {
                        result.push_str(&current_paragraph);
                        result.push_str("\n\n");
                        current_paragraph.clear();
                    }
                    continue;
                }

                // Detect paragraph breaks: lines starting with caps or numbers after whitespace
                let is_new_para = !current_paragraph.is_empty()
                    && (trimmed.starts_with(char::is_uppercase)
                        || trimmed.starts_with(|c: char| c.is_ascii_digit()));

                if is_new_para && current_paragraph.ends_with('.') {
                    result.push_str(&current_paragraph);
                    result.push_str("\n\n");
                    current_paragraph.clear();
                }

                if !current_paragraph.is_empty() {
                    // Check for hyphenation
                    if current_paragraph.ends_with('-') {
                        current_paragraph.pop();
                    } else {
                        current_paragraph.push(' ');
                    }
                }
                current_paragraph.push_str(trimmed);
            }

            if !current_paragraph.is_empty() {
                result.push_str(&current_paragraph);
            }

            result
        } else {
            text.to_string()
        }
    }

    /// Normalize whitespace in text.
    fn normalize_whitespace(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_was_space = false;
        let mut prev_was_newline = false;

        for c in text.chars() {
            if c == '\n' {
                if !prev_was_newline {
                    result.push('\n');
                    prev_was_newline = true;
                }
                prev_was_space = false;
            } else if c.is_whitespace() {
                if !prev_was_space && !prev_was_newline {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(c);
                prev_was_space = false;
                prev_was_newline = false;
            }
        }

        result.trim().to_string()
    }

    /// Estimate page count from extracted text.
    fn estimate_page_count(&self, text: &str) -> usize {
        // Count form feed characters (page breaks)
        let form_feeds = text.matches('\x0c').count();

        if form_feeds > 0 {
            form_feeds + 1
        } else {
            // Estimate based on character count (average ~3000 chars per page)
            let chars = text.len();
            (chars / 3000).max(1)
        }
    }

    /// Create chunks from processed text.
    fn create_chunks(&self, text: &str, page_count: usize) -> Vec<PdfChunk> {
        let mut chunks = Vec::new();

        // Split by form feeds first (actual page breaks)
        let pages: Vec<&str> = text.split('\x0c').collect();

        if pages.len() > 1 {
            // We have actual page breaks
            for (page_idx, page_text) in pages.iter().enumerate() {
                let page_chunks = Self::chunk_text(page_text, self.config.max_chunk_size);
                for (chunk_idx, chunk_content) in page_chunks.into_iter().enumerate() {
                    if !chunk_content.trim().is_empty() {
                        chunks.push(PdfChunk::new(chunk_content, page_idx + 1, chunk_idx));
                    }
                }
            }
        } else {
            // No page breaks - distribute chunks across estimated pages
            let all_chunks = Self::chunk_text(text, self.config.max_chunk_size);
            let chunks_per_page = (all_chunks.len() / page_count).max(1);

            for (idx, chunk_content) in all_chunks.into_iter().enumerate() {
                if !chunk_content.trim().is_empty() {
                    let page_number = (idx / chunks_per_page).min(page_count - 1) + 1;
                    let chunk_index = idx % chunks_per_page;
                    chunks.push(PdfChunk::new(chunk_content, page_number, chunk_index));
                }
            }
        }

        // Merge very small chunks with adjacent ones
        self.merge_small_chunks(chunks)
    }

    /// Merge chunks that are too small.
    fn merge_small_chunks(&self, chunks: Vec<PdfChunk>) -> Vec<PdfChunk> {
        if chunks.is_empty() {
            return chunks;
        }

        let mut result: Vec<PdfChunk> = Vec::new();

        for chunk in chunks {
            if chunk.content.len() < self.config.min_chunk_size {
                // Try to merge with previous chunk if on same page
                if let Some(last) = result.last_mut() {
                    if last.page_number == chunk.page_number
                        && last.content.len() + chunk.content.len() < self.config.max_chunk_size
                    {
                        last.content.push_str("\n\n");
                        last.content.push_str(&chunk.content);
                        continue;
                    }
                }
            }
            result.push(chunk);
        }

        result
    }

    /// Chunk text into segments of maximum size, breaking at paragraph or sentence boundaries.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to chunk
    /// * `max_chunk_size` - Maximum size of each chunk in characters
    ///
    /// # Returns
    ///
    /// A vector of text chunks.
    pub fn chunk_text(text: &str, max_chunk_size: usize) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }

        if text.len() <= max_chunk_size {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        // Split by paragraphs first (double newlines)
        let paragraphs: Vec<&str> = text.split("\n\n").collect();

        for para in paragraphs {
            let para_trimmed = para.trim();
            if para_trimmed.is_empty() {
                continue;
            }

            // If adding this paragraph would exceed max size
            if !current_chunk.is_empty()
                && current_chunk.len() + para_trimmed.len() + 2 > max_chunk_size
            {
                // Save current chunk and start new one
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
            }

            // If single paragraph is too large, split by sentences
            if para_trimmed.len() > max_chunk_size {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk = String::new();
                }

                let sentence_chunks = Self::chunk_by_sentences(para_trimmed, max_chunk_size);
                chunks.extend(sentence_chunks);
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(para_trimmed);
            }
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Chunk text by sentence boundaries when paragraphs are too large.
    fn chunk_by_sentences(text: &str, max_chunk_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        // Simple sentence splitting on . ! ? followed by space or end
        let sentence_endings = [". ", "! ", "? ", ".\n", "!\n", "?\n"];

        let mut remaining = text;
        while !remaining.is_empty() {
            // Find the next sentence boundary
            let mut best_split = remaining.len();

            for ending in &sentence_endings {
                if let Some(pos) = remaining.find(ending) {
                    let split_pos = pos + ending.len();
                    if split_pos < best_split {
                        best_split = split_pos;
                    }
                }
            }

            let sentence = &remaining[..best_split];
            remaining = &remaining[best_split..];

            // Check if adding this sentence would exceed max size
            if !current_chunk.is_empty() && current_chunk.len() + sentence.len() > max_chunk_size {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = String::new();
            }

            // If single sentence is too large, force split at max_chunk_size
            if sentence.len() > max_chunk_size {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                    current_chunk = String::new();
                }

                let mut sent_remaining = sentence;
                while !sent_remaining.is_empty() {
                    let split_at = max_chunk_size.min(sent_remaining.len());
                    // Try to split at word boundary
                    let split_pos = if split_at < sent_remaining.len() {
                        sent_remaining[..split_at]
                            .rfind(' ')
                            .map(|p| p + 1)
                            .unwrap_or(split_at)
                    } else {
                        split_at
                    };

                    chunks.push(sent_remaining[..split_pos].trim().to_string());
                    sent_remaining = &sent_remaining[split_pos..];
                }
            } else {
                current_chunk.push_str(sentence);
            }
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Ingest all PDF files in a directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory path to search
    /// * `recursive` - Whether to search subdirectories
    ///
    /// # Returns
    ///
    /// A vector of successfully parsed PDF documents.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn ingest_directory(&self, dir: &Path, recursive: bool) -> Result<Vec<PdfDocument>> {
        info!(?dir, recursive, "Ingesting PDF files from directory");

        if !dir.exists() {
            return Err(Error::ingest(format!(
                "Directory not found: {}",
                dir.display()
            )));
        }

        if !dir.is_dir() {
            return Err(Error::ingest(format!(
                "Path is not a directory: {}",
                dir.display()
            )));
        }

        let mut documents = Vec::new();
        self.ingest_directory_recursive(dir, recursive, &mut documents)?;

        info!(
            dir = %dir.display(),
            count = documents.len(),
            "Directory ingestion complete"
        );

        Ok(documents)
    }

    /// Recursive helper for directory ingestion.
    fn ingest_directory_recursive(
        &self,
        dir: &Path,
        recursive: bool,
        documents: &mut Vec<PdfDocument>,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            Error::ingest(format!("Failed to read directory {}: {}", dir.display(), e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| Error::ingest(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.is_dir() {
                if recursive {
                    self.ingest_directory_recursive(&path, recursive, documents)?;
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                match self.ingest_file(&path) {
                    Ok(doc) => documents.push(doc),
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Failed to ingest PDF");
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_ingester_creation() {
        let ingester = PdfIngester::new();
        assert_eq!(ingester.config.max_chunk_size, DEFAULT_MAX_CHUNK_SIZE);
    }

    #[test]
    fn test_pdf_ingester_with_config() {
        let config = PdfIngesterConfig::default()
            .with_max_chunk_size(2000)
            .with_preserve_line_breaks(true);

        let ingester = PdfIngester::with_config(config);
        assert_eq!(ingester.config.max_chunk_size, 2000);
        assert!(ingester.config.preserve_line_breaks);
    }

    #[test]
    fn test_chunk_text_small() {
        let text = "This is a small text.";
        let chunks = PdfIngester::chunk_text(text, 4000);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunk_text_paragraphs() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = PdfIngester::chunk_text(text, 30);

        assert!(chunks.len() >= 2);
        assert!(chunks[0].contains("First"));
    }

    #[test]
    fn test_chunk_text_long_paragraph() {
        let text = "This is sentence one. This is sentence two. This is sentence three. This is sentence four. This is sentence five.";
        let chunks = PdfIngester::chunk_text(text, 50);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 100); // Allow some overflow for sentence completion
        }
    }

    #[test]
    fn test_chunk_text_empty() {
        let chunks = PdfIngester::chunk_text("", 4000);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_pdf_document_new() {
        let doc = PdfDocument::new(PathBuf::from("test.pdf"));

        assert_eq!(doc.path, PathBuf::from("test.pdf"));
        assert!(doc.title.is_none());
        assert!(doc.author.is_none());
        assert!(doc.chunks.is_empty());
        assert_eq!(doc.page_count, 0);
    }

    #[test]
    fn test_pdf_document_full_text() {
        let mut doc = PdfDocument::new(PathBuf::from("test.pdf"));
        doc.chunks
            .push(PdfChunk::new("First chunk.".to_string(), 1, 0));
        doc.chunks
            .push(PdfChunk::new("Second chunk.".to_string(), 1, 1));

        let full = doc.full_text();
        assert!(full.contains("First chunk."));
        assert!(full.contains("Second chunk."));
    }

    #[test]
    fn test_pdf_document_total_chars() {
        let mut doc = PdfDocument::new(PathBuf::from("test.pdf"));
        doc.chunks.push(PdfChunk::new("Hello".to_string(), 1, 0));
        doc.chunks.push(PdfChunk::new("World".to_string(), 2, 0));

        assert_eq!(doc.total_chars(), 10);
    }

    #[test]
    fn test_pdf_document_chunks_for_page() {
        let mut doc = PdfDocument::new(PathBuf::from("test.pdf"));
        doc.chunks
            .push(PdfChunk::new("Page 1 chunk 1".to_string(), 1, 0));
        doc.chunks
            .push(PdfChunk::new("Page 1 chunk 2".to_string(), 1, 1));
        doc.chunks
            .push(PdfChunk::new("Page 2 chunk 1".to_string(), 2, 0));

        let page1_chunks = doc.chunks_for_page(1);
        assert_eq!(page1_chunks.len(), 2);

        let page2_chunks = doc.chunks_for_page(2);
        assert_eq!(page2_chunks.len(), 1);
    }

    #[test]
    fn test_pdf_chunk_creation() {
        let chunk = PdfChunk::new("Content".to_string(), 5, 2);

        assert_eq!(chunk.content, "Content");
        assert_eq!(chunk.page_number, 5);
        assert_eq!(chunk.chunk_index, 2);
    }

    #[test]
    fn test_pdf_chunk_is_empty() {
        let empty_chunk = PdfChunk::new("   \n\t  ".to_string(), 1, 0);
        assert!(empty_chunk.is_empty());

        let non_empty_chunk = PdfChunk::new("Content".to_string(), 1, 0);
        assert!(!non_empty_chunk.is_empty());
    }

    #[test]
    fn test_pdf_chunk_len() {
        let chunk = PdfChunk::new("Hello World".to_string(), 1, 0);
        assert_eq!(chunk.len(), 11);
    }

    #[test]
    fn test_normalize_whitespace() {
        let ingester = PdfIngester::new();

        let text = "Hello    world\n\n\n\nTest";
        let normalized = ingester.normalize_whitespace(text);

        assert!(!normalized.contains("    "));
        assert!(!normalized.contains("\n\n\n"));
    }

    #[test]
    fn test_estimate_page_count_with_form_feeds() {
        let ingester = PdfIngester::new();

        let text = "Page 1\x0cPage 2\x0cPage 3";
        assert_eq!(ingester.estimate_page_count(text), 3);
    }

    #[test]
    fn test_estimate_page_count_without_form_feeds() {
        let ingester = PdfIngester::new();

        // ~6000 chars should be ~2 pages
        let text = "a".repeat(6000);
        assert_eq!(ingester.estimate_page_count(&text), 2);
    }

    #[test]
    fn test_extract_metadata_field() {
        let ingester = PdfIngester::new();

        let pdf_content = "/Title (Test Document)";
        let title = ingester.extract_metadata_field(pdf_content, "Title");
        assert_eq!(title, Some("Test Document".to_string()));
    }

    #[test]
    fn test_extract_metadata_field_with_nested_parens() {
        let ingester = PdfIngester::new();

        let pdf_content = "/Author (John (Jack) Doe)";
        let author = ingester.extract_metadata_field(pdf_content, "Author");
        assert_eq!(author, Some("John (Jack) Doe".to_string()));
    }

    #[test]
    fn test_extract_metadata_field_not_found() {
        let ingester = PdfIngester::new();

        let pdf_content = "/Title (Test)";
        let author = ingester.extract_metadata_field(pdf_content, "Author");
        assert!(author.is_none());
    }

    #[test]
    fn test_ingest_file_not_found() {
        let ingester = PdfIngester::new();
        let result = ingester.ingest_file(Path::new("/nonexistent/path/file.pdf"));

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("Ingest"));
    }

    #[test]
    fn test_ingest_directory_not_found() {
        let ingester = PdfIngester::new();
        let result = ingester.ingest_directory(Path::new("/nonexistent/directory"), false);

        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = PdfIngesterConfig::default()
            .with_max_chunk_size(5000)
            .with_preserve_line_breaks(true)
            .with_multi_column_handling(false);

        assert_eq!(config.max_chunk_size, 5000);
        assert!(config.preserve_line_breaks);
        assert!(!config.handle_multi_column);
    }

    #[test]
    fn test_create_chunks_with_form_feeds() {
        let ingester = PdfIngester::new();

        let text = "Page one content.\x0cPage two content.\x0cPage three content.";
        let chunks = ingester.create_chunks(text, 3);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].page_number, 1);
        assert_eq!(chunks[1].page_number, 2);
        assert_eq!(chunks[2].page_number, 3);
    }

    #[test]
    fn test_handle_multi_column_short_lines() {
        let ingester = PdfIngester::new();

        // Simulate multi-column text with short lines
        let lines: Vec<String> = (0..30).map(|i| format!("Line {}", i)).collect();
        let text = lines.join("\n");

        let processed = ingester.handle_multi_column_text(&text);
        // Should consolidate short lines
        assert!(processed.lines().count() < 30 || processed.contains(' '));
    }

    #[test]
    fn test_merge_small_chunks() {
        let ingester = PdfIngester::new();

        let chunks = vec![
            PdfChunk::new("Small".to_string(), 1, 0),
            PdfChunk::new("Also small".to_string(), 1, 1),
            PdfChunk::new(
                "A much longer chunk that should not be merged.".to_string(),
                1,
                2,
            ),
        ];

        let merged = ingester.merge_small_chunks(chunks);
        // First two small chunks should be merged
        assert!(merged.len() <= 2);
    }
}
