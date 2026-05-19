// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Tree-sitter based parser for Gherkin/Truths files.
//!
//! This module provides syntax-aware parsing using tree-sitter,
//! enabling accurate syntax highlighting, code completion, and navigation.

use tree_sitter::{Language, LanguageError, Parser, Tree};

/// A parser for Gherkin/Truths files.
///
/// Uses tree-sitter for incremental parsing and accurate syntax trees.
pub struct TruthParser {
    parser: Parser,
    language: Language,
}

impl std::fmt::Debug for TruthParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TruthParser")
            .field("language_version", &self.language.abi_version())
            .finish_non_exhaustive()
    }
}

impl TruthParser {
    /// Create a new parser with a tree-sitter language.
    ///
    /// # Arguments
    /// * `language` - Compiled tree-sitter language for Gherkin/Truths.
    pub fn new(language: Language) -> Result<Self, LanguageError> {
        let mut parser = Parser::new();
        parser.set_language(&language)?;
        Ok(Self { parser, language })
    }

    /// Parse the given source code and return a syntax tree.
    pub fn parse(&mut self, source: &str) -> Option<Tree> {
        self.parser.parse(source, None)
    }

    /// Parse source with old tree for incremental parsing.
    pub fn parse_with_tree(&mut self, source: &str, old_tree: &Tree) -> Option<Tree> {
        self.parser.parse(source, Some(old_tree))
    }

    /// Get the language.
    pub fn language(&self) -> Language {
        self.language.clone()
    }
}

/// Node types in the Gherkin/Truths syntax tree.
///
/// These correspond to the tree-sitter grammar node types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TruthNodeKind {
    // Document
    Document,

    // Feature/Truth
    Feature,
    FeatureHeader,
    FeatureLine,
    TruthLine, // Converge extension

    // Governance blocks (Converge extensions)
    IntentBlock,
    AuthorityBlock,
    ConstraintBlock,
    EvidenceBlock,
    ExceptionBlock,

    // Scenario
    Scenario,
    ScenarioDefinition,
    ScenarioLine,
    ScenarioOutline,
    ScenarioOutlineLine,

    // Steps
    Steps,
    Step,
    Given,
    When,
    Then,
    And,
    But,

    // Background
    Background,
    BackgroundLine,

    // Examples
    Examples,
    ExamplesDefinition,
    ExamplesLine,
    ExamplesTable,
    TableRow,
    TableCell,

    // Tags
    Tags,
    Tag,

    // Comments
    Comment,

    // Text
    Description,
    Context,

    // Tables
    DataTable,

    // DocStrings
    DocString,

    // Language
    Language,
    LanguageLine,

    // Rule (Gherkin 6+)
    Rule,
    RuleLine,

    // Unknown
    Error,
}

impl TruthNodeKind {
    /// Convert from a tree-sitter node kind string.
    pub fn from_str(kind: &str) -> Self {
        match kind {
            "document" => Self::Document,
            "feature" => Self::Feature,
            "feature_header" => Self::FeatureHeader,
            "feature_line" => Self::FeatureLine,
            "truth_line" => Self::TruthLine,
            "intent_block" => Self::IntentBlock,
            "authority_block" => Self::AuthorityBlock,
            "constraint_block" => Self::ConstraintBlock,
            "evidence_block" => Self::EvidenceBlock,
            "exception_block" => Self::ExceptionBlock,
            "scenario" => Self::Scenario,
            "scenario_definition" => Self::ScenarioDefinition,
            "scenario_line" => Self::ScenarioLine,
            "scenario_outline" => Self::ScenarioOutline,
            "scenario_outline_line" => Self::ScenarioOutlineLine,
            "steps" => Self::Steps,
            "step" => Self::Step,
            "given" => Self::Given,
            "when" => Self::When,
            "then" => Self::Then,
            "and" => Self::And,
            "but" => Self::But,
            "background" => Self::Background,
            "background_line" => Self::BackgroundLine,
            "examples" => Self::Examples,
            "examples_definition" => Self::ExamplesDefinition,
            "examples_line" => Self::ExamplesLine,
            "examples_table" => Self::ExamplesTable,
            "table_row" => Self::TableRow,
            "table_cell" => Self::TableCell,
            "tags" => Self::Tags,
            "tag" => Self::Tag,
            "comment" => Self::Comment,
            "description" => Self::Description,
            "context" => Self::Context,
            "data_table" => Self::DataTable,
            "doc_string" => Self::DocString,
            "language" => Self::Language,
            "language_line" => Self::LanguageLine,
            "rule" => Self::Rule,
            "rule_line" => Self::RuleLine,
            _ => Self::Error,
        }
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Feature => "feature",
            Self::FeatureHeader => "feature_header",
            Self::FeatureLine => "feature_line",
            Self::TruthLine => "truth_line",
            Self::IntentBlock => "intent_block",
            Self::AuthorityBlock => "authority_block",
            Self::ConstraintBlock => "constraint_block",
            Self::EvidenceBlock => "evidence_block",
            Self::ExceptionBlock => "exception_block",
            Self::Scenario => "scenario",
            Self::ScenarioDefinition => "scenario_definition",
            Self::ScenarioLine => "scenario_line",
            Self::ScenarioOutline => "scenario_outline",
            Self::ScenarioOutlineLine => "scenario_outline_line",
            Self::Steps => "steps",
            Self::Step => "step",
            Self::Given => "given",
            Self::When => "when",
            Self::Then => "then",
            Self::And => "and",
            Self::But => "but",
            Self::Background => "background",
            Self::BackgroundLine => "background_line",
            Self::Examples => "examples",
            Self::ExamplesDefinition => "examples_definition",
            Self::ExamplesLine => "examples_line",
            Self::ExamplesTable => "examples_table",
            Self::TableRow => "table_row",
            Self::TableCell => "table_cell",
            Self::Tags => "tags",
            Self::Tag => "tag",
            Self::Comment => "comment",
            Self::Description => "description",
            Self::Context => "context",
            Self::DataTable => "data_table",
            Self::DocString => "doc_string",
            Self::Language => "language",
            Self::LanguageLine => "language_line",
            Self::Rule => "rule",
            Self::RuleLine => "rule_line",
            Self::Error => "error",
        }
    }
}

/// Syntax kind for classification in syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    /// Keywords: Feature, Scenario, Given, When, Then, etc.
    Keyword,
    /// Governance block keywords: Intent, Authority, Constraint, etc.
    GovernanceKeyword,
    /// Step keywords: Given, When, Then, And, But
    StepKeyword,
    /// Tags: @tag
    Tag,
    /// Comments
    Comment,
    /// Strings (quoted text in steps)
    String,
    /// Table headers and cells
    Table,
    /// DocStrings
    DocString,
    /// Punctuation: :, |, etc.
    Punctuation,
    /// Whitespace
    Whitespace,
    /// Error/invalid syntax
    Error,
    /// Default/unknown
    Default,
}

/// A syntax token for highlighting.
#[derive(Debug, Clone)]
pub struct SyntaxToken {
    /// Byte start position.
    pub start: usize,
    /// Byte end position.
    pub end: usize,
    /// The kind of syntax.
    pub kind: SyntaxKind,
}

/// A range in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_kind_from_str() {
        assert_eq!(TruthNodeKind::from_str("feature"), TruthNodeKind::Feature);
        assert_eq!(TruthNodeKind::from_str("scenario"), TruthNodeKind::Scenario);
        assert_eq!(TruthNodeKind::from_str("given"), TruthNodeKind::Given);
        assert_eq!(TruthNodeKind::from_str("unknown"), TruthNodeKind::Error);
    }

    #[test]
    fn test_text_range() {
        let range = TextRange::new(10, 20);
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(!range.contains(20));
        assert!(!range.contains(5));
        assert_eq!(range.len(), 10);
    }
}
