// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Rope-based text buffer for efficient editing of Truths/Gherkin files.

use ropey::{LineType, Rope};
use std::sync::Arc;

const LINE_TYPE: LineType = LineType::LF_CR;

/// A text buffer optimized for editing large Gherkin/Truths files.
///
/// Uses a rope data structure for efficient:
/// - Insertions and deletions (O(log n) time)
/// - Random access (O(log n) time)
/// - Memory efficiency (only stores modified chunks)
/// - Undo/redo support
#[derive(Debug, Clone)]
pub struct TruthTextBuffer {
    rope: Arc<Rope>,
}

impl Default for TruthTextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TruthTextBuffer {
    /// Create a new empty text buffer.
    pub fn new() -> Self {
        Self {
            rope: Arc::new(Rope::new()),
        }
    }

    /// Create a new buffer with initial content.
    pub fn from_str(content: &str) -> Self {
        Self {
            rope: Arc::new(Rope::from_str(content)),
        }
    }

    /// Get the full text content.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get a slice of text by byte range.
    pub fn slice(&self, start: usize, end: usize) -> String {
        self.rope.slice(start..end).to_string()
    }

    /// Get the length in bytes.
    pub fn len(&self) -> usize {
        self.rope.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Insert text at a byte position.
    pub fn insert(&mut self, pos: usize, text: &str) {
        let rope = Arc::make_mut(&mut self.rope);
        rope.insert(pos, text);
    }

    /// Remove text from a byte range.
    pub fn remove(&mut self, start: usize, end: usize) {
        let rope = Arc::make_mut(&mut self.rope);
        rope.remove(start..end);
    }

    /// Replace text in a byte range.
    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        let rope = Arc::make_mut(&mut self.rope);
        rope.remove(start..end);
        rope.insert(start, text);
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines(LINE_TYPE)
    }

    /// Get the byte offset of a line.
    pub fn line_start(&self, line: usize) -> usize {
        self.rope.line_to_byte_idx(line, LINE_TYPE)
    }

    /// Get a line by index.
    pub fn get_line(&self, line: usize) -> String {
        strip_line_ending(self.rope.line(line, LINE_TYPE).to_string())
    }

    /// Get the byte offset range for a line.
    pub fn line_range(&self, line: usize) -> (usize, usize) {
        let start = self.rope.line_to_byte_idx(line, LINE_TYPE);
        let end_with_line_ending = if line + 1 < self.line_count() {
            self.rope.line_to_byte_idx(line + 1, LINE_TYPE)
        } else {
            self.len()
        };
        let line_text = self.rope.line(line, LINE_TYPE).to_string();
        let end = end_with_line_ending.saturating_sub(line_ending_len(&line_text));
        (start, end)
    }

    /// Convert a byte position to a line and column.
    pub fn byte_to_line_col(&self, byte: usize) -> (usize, usize) {
        let line = self.rope.byte_to_line_idx(byte.min(self.len()), LINE_TYPE);
        let col = byte.min(self.len()) - self.rope.line_to_byte_idx(line, LINE_TYPE);
        (line, col)
    }

    /// Convert a line and column to a byte position.
    pub fn line_col_to_byte(&self, line: usize, col: usize) -> usize {
        let (start, end) = self.line_range(line);
        start.saturating_add(col).min(end)
    }
}

/// Immutable snapshot of the text buffer for concurrent access.
#[derive(Debug, Clone)]
pub struct BufferSnapshot {
    rope: Arc<Rope>,
}

impl BufferSnapshot {
    /// Create a snapshot from a buffer.
    pub fn from_buffer(buffer: &TruthTextBuffer) -> Self {
        Self {
            rope: buffer.rope.clone(),
        }
    }

    /// Get the text content.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get a slice of text.
    pub fn slice(&self, start: usize, end: usize) -> String {
        self.rope.slice(start..end).to_string()
    }

    /// Get the length in bytes.
    pub fn len(&self) -> usize {
        self.rope.len()
    }

    /// Check if the snapshot is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of lines.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines(LINE_TYPE)
    }

    /// Get a line by index.
    pub fn get_line(&self, line: usize) -> String {
        strip_line_ending(self.rope.line(line, LINE_TYPE).to_string())
    }

    /// Convert byte to line/column.
    pub fn byte_to_line_col(&self, byte: usize) -> (usize, usize) {
        let line = self.rope.byte_to_line_idx(byte.min(self.len()), LINE_TYPE);
        let col = byte.min(self.len()) - self.rope.line_to_byte_idx(line, LINE_TYPE);
        (line, col)
    }
}

fn strip_line_ending(mut line: String) -> String {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    } else if line.ends_with('\r') {
        line.pop();
    }
    line
}

fn line_ending_len(line: &str) -> usize {
    if line.ends_with("\r\n") {
        2
    } else {
        usize::from(line.ends_with('\n') || line.ends_with('\r'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let buffer = TruthTextBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.line_count(), 1); // Empty rope has 1 empty line
    }

    #[test]
    fn test_buffer_with_content() {
        let content = "Truth: Test\n  Scenario: Test\n    Given a test";
        let buffer = TruthTextBuffer::from_str(content);
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), content.len());
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(0), "Truth: Test");
        assert_eq!(buffer.get_line(1), "  Scenario: Test");
        assert_eq!(buffer.get_line(2), "    Given a test");
    }

    #[test]
    fn test_insert_and_remove() {
        let original = "Feature: Test";
        let mut buffer = TruthTextBuffer::from_str(original);
        buffer.insert(original.len(), "\nScenario: Test");
        assert_eq!(buffer.text(), "Feature: Test\nScenario: Test");

        buffer.remove(original.len(), buffer.len());
        assert_eq!(buffer.text(), "Feature: Test");
    }

    #[test]
    fn test_line_col_conversion() {
        let buffer = TruthTextBuffer::from_str("Line 1\nLine 2\nLine 3");
        let (line, col) = buffer.byte_to_line_col(7);
        assert_eq!(line, 1);
        assert_eq!(col, 0);

        let byte = buffer.line_col_to_byte(1, 0);
        assert_eq!(byte, 7);
    }

    #[test]
    fn test_line_range_excludes_line_ending() {
        let buffer = TruthTextBuffer::from_str("Line 1\r\nLine 2\n");
        assert_eq!(buffer.line_range(0), (0, 6));
        assert_eq!(buffer.get_line(0), "Line 1");
        assert_eq!(buffer.get_line(1), "Line 2");
    }
}
