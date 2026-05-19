// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Editor infrastructure for Gherkin/Truths files.
//!
//! This module provides:
//! - Rope-based text buffer for efficient editing of large files
//! - Tree-sitter integration for syntax-aware operations
//! - Editor services: parsing, highlighting, completion, formatting

pub mod buffer;
pub mod parser;
pub mod service;

pub use buffer::{BufferSnapshot, TruthTextBuffer};
pub use parser::{SyntaxKind, SyntaxToken, TextRange, TruthNodeKind, TruthParser};
pub use service::{
    CompletionItem, CompletionKind, Diagnostic, DiagnosticSeverity, DocumentSymbol, FormattingEdit,
    ParsedTruthDocument, StepDefinition, SymbolKind, TruthEditorService,
};
