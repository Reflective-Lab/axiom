// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Editor services for Gherkin/Truths files.
//!
//! This module provides high-level editor services including:
//! - Syntax highlighting
//! - Code completion
//! - Formatting
//! - Navigation
//! - Diagnostics

use std::collections::HashMap;

use super::buffer::TruthTextBuffer;
use super::parser::{SyntaxKind, SyntaxToken, TextRange};
use crate::gherkin::preprocess_truths;
use crate::truths::parse_truth_document;

const STEP_KEYWORDS: [&str; 5] = ["Given", "When", "Then", "And", "But"];
const GOVERNANCE_BLOCKS: [&str; 5] = ["Intent", "Authority", "Constraint", "Evidence", "Exception"];
const GOVERNANCE_FIELDS: [&str; 13] = [
    "Outcome",
    "Goal",
    "Actor",
    "May",
    "Must Not",
    "Requires Approval",
    "Expires",
    "Budget",
    "Cost Limit",
    "Requires",
    "Provenance",
    "Audit",
    "Escalates To",
];
const STRUCTURE_KEYWORDS: [&str; 8] = [
    "Scenario Outline:",
    "Scenario:",
    "Feature:",
    "Truth:",
    "Background:",
    "Examples:",
    "Example:",
    "Rule:",
];
const DEFAULT_TAGS: [&str; 8] = [
    "invariant",
    "structural",
    "semantic",
    "acceptance",
    "id:",
    "llm",
    "test",
    "policy",
];

/// Step definition for completion and validation.
#[derive(Debug, Clone)]
pub struct StepDefinition {
    /// The step keyword (Given, When, Then, And, But).
    pub keyword: String,
    /// The step text pattern.
    pub pattern: String,
    /// Description of what the step does.
    pub description: Option<String>,
}

/// Completion item for code completion.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The text to insert.
    pub label: String,
    /// Kind of completion.
    pub kind: CompletionKind,
    /// Detail description.
    pub detail: Option<String>,
    /// Documentation.
    pub documentation: Option<String>,
}

/// Kind of completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// Feature, Truth, Scenario, Background, Examples, Rule.
    StructureKeyword,
    /// Step keyword (Given, When, Then, And, But).
    StepKeyword,
    /// Governance block keyword (Intent, Authority, etc.).
    GovernanceKeyword,
    /// Tag.
    Tag,
    /// Step text (from step definitions).
    Step,
    /// Snippet (multi-line template).
    Snippet,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Byte range of the diagnostic.
    pub range: TextRange,
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Message.
    pub message: String,
    /// Code (optional).
    pub code: Option<String>,
}

/// Formatting edit.
#[derive(Debug, Clone)]
pub struct FormattingEdit {
    /// Byte range to replace.
    pub range: TextRange,
    /// New text.
    pub new_text: String,
}

/// Editor service for Gherkin/Truths files.
#[derive(Debug)]
pub struct TruthEditorService {
    /// Registered step definitions.
    step_definitions: HashMap<String, StepDefinition>,
    /// Registered tags.
    known_tags: Vec<String>,
    /// Governance block keywords.
    governance_keywords: Vec<String>,
}

impl Default for TruthEditorService {
    fn default() -> Self {
        Self::new()
    }
}

impl TruthEditorService {
    /// Create a new editor service.
    pub fn new() -> Self {
        let mut service = Self {
            step_definitions: HashMap::new(),
            known_tags: DEFAULT_TAGS.iter().map(ToString::to_string).collect(),
            governance_keywords: GOVERNANCE_BLOCKS.iter().map(ToString::to_string).collect(),
        };

        service.register_step_definitions(default_step_definitions());
        service
    }

    /// Register step definitions.
    pub fn register_step_definitions(&mut self, definitions: Vec<StepDefinition>) {
        for def in definitions {
            self.step_definitions.insert(def.pattern.clone(), def);
        }
    }

    /// Register known tags.
    pub fn register_tags(&mut self, tags: Vec<String>) {
        self.known_tags = tags
            .into_iter()
            .map(|tag| tag.trim_start_matches('@').to_string())
            .collect();
    }

    /// Parse and validate the document.
    pub fn parse_document(
        &self,
        buffer: &TruthTextBuffer,
    ) -> Result<ParsedTruthDocument, Vec<Diagnostic>> {
        let text = buffer.text();
        let preprocessed = preprocess_truths(&text);

        let doc = match parse_truth_document(&text) {
            Ok(doc) => doc,
            Err(error) => return Err(vec![parse_error_diagnostic(&text, error.to_string())]),
        };

        if let Err(error) =
            ::gherkin::Feature::parse(&doc.gherkin, ::gherkin::GherkinEnv::default())
        {
            return Err(vec![parse_error_diagnostic(
                &text,
                format!("Gherkin parse error: {error}"),
            )]);
        }

        Ok(ParsedTruthDocument {
            text,
            preprocessed,
            governance: doc.governance,
            gherkin_content: doc.gherkin,
            errors: Vec::new(),
        })
    }

    /// Get diagnostics for the document.
    pub fn diagnostics(&self, buffer: &TruthTextBuffer) -> Vec<Diagnostic> {
        self.parse_document(buffer)
            .map_or_else(|diagnostics| diagnostics, |document| document.errors)
    }

    /// Get syntax highlighting tokens for the entire document.
    pub fn highlight(&self, buffer: &TruthTextBuffer) -> Vec<SyntaxToken> {
        self.highlight_without_tree_sitter(&buffer.text())
    }

    /// Highlight without tree-sitter. This is the deterministic fallback until
    /// the compiled grammar is wired into [`TruthParser`](super::parser::TruthParser).
    fn highlight_without_tree_sitter(&self, text: &str) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let mut offset = 0;

        for segment in text.split_inclusive('\n') {
            let line = trim_line_ending(segment);
            self.highlight_line(line, offset, &mut tokens);
            offset += segment.len();
        }

        tokens
    }

    fn highlight_line(&self, line: &str, line_start: usize, tokens: &mut Vec<SyntaxToken>) {
        let leading = leading_whitespace_len(line);
        let trimmed = &line[leading..];

        if trimmed.is_empty() {
            return;
        }

        if trimmed.starts_with('#') {
            push_token(
                tokens,
                line_start + leading,
                line_start + line.len(),
                SyntaxKind::Comment,
            );
            return;
        }

        highlight_tags(line, line_start, tokens);

        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("```") {
            push_token(
                tokens,
                line_start + leading,
                line_start + line.len(),
                SyntaxKind::DocString,
            );
            return;
        }

        if trimmed.starts_with('|') {
            push_token(
                tokens,
                line_start + leading,
                line_start + line.len(),
                SyntaxKind::Table,
            );
            return;
        }

        if let Some(block) = self.match_governance_block(trimmed) {
            let end = line_start + leading + block.len() + 1;
            push_token(
                tokens,
                line_start + leading,
                end,
                SyntaxKind::GovernanceKeyword,
            );
        } else if let Some(keyword) = match_prefixed_keyword(trimmed, &STRUCTURE_KEYWORDS) {
            push_token(
                tokens,
                line_start + leading,
                line_start + leading + keyword.len(),
                SyntaxKind::Keyword,
            );
        } else if let Some(keyword) = match_step_keyword(trimmed) {
            push_token(
                tokens,
                line_start + leading,
                line_start + leading + keyword.len(),
                SyntaxKind::StepKeyword,
            );
        } else if let Some(field) = match_governance_field(trimmed) {
            push_token(
                tokens,
                line_start + leading,
                line_start + leading + field.len() + 1,
                SyntaxKind::GovernanceKeyword,
            );
        }

        let comment_start = find_inline_comment(line).unwrap_or(line.len());
        highlight_strings(&line[..comment_start], line_start, tokens);

        if comment_start < line.len() {
            push_token(
                tokens,
                line_start + comment_start,
                line_start + line.len(),
                SyntaxKind::Comment,
            );
        }
    }

    fn match_governance_block<'a>(&'a self, trimmed: &str) -> Option<&'a str> {
        self.governance_keywords.iter().find_map(|keyword| {
            trimmed
                .strip_prefix(keyword)
                .filter(|rest| rest.starts_with(':'))
                .map(|_| keyword.as_str())
        })
    }

    /// Get code completions at a specific position.
    pub fn complete(&self, buffer: &TruthTextBuffer, offset: usize) -> Vec<CompletionItem> {
        let (line, col) = buffer.byte_to_line_col(offset);
        let line_text = buffer.get_line(line);
        let prefix = prefix_at_byte(&line_text, col);
        let context = self.detect_context(prefix);

        match context {
            CompletionContext::Tag => self.complete_tags(prefix),
            CompletionContext::GovernanceKeyword => self.complete_governance_keywords(prefix),
            CompletionContext::StepKeyword => Self::complete_step_keywords(prefix),
            CompletionContext::StepText => self.complete_steps(prefix),
            CompletionContext::FeatureKeyword => {
                let mut items = Self::complete_feature_keywords(prefix);
                items.extend(self.complete_governance_keywords(prefix));
                items.extend(Self::complete_step_keywords(prefix));
                items.extend(Self::complete_snippets(prefix));
                items
            }
            CompletionContext::Unknown => {
                let mut items = Self::complete_feature_keywords(prefix);
                items.extend(Self::complete_step_keywords(prefix));
                items.extend(self.complete_governance_keywords(prefix));
                items
            }
        }
    }

    /// Detect the completion context.
    fn detect_context(&self, prefix: &str) -> CompletionContext {
        if current_tag_prefix(prefix).is_some() {
            return CompletionContext::Tag;
        }

        let trimmed = prefix.trim_start();
        if trimmed.is_empty() {
            return CompletionContext::FeatureKeyword;
        }

        if step_text_query(trimmed).is_some() {
            return CompletionContext::StepText;
        }

        if STEP_KEYWORDS
            .iter()
            .any(|keyword| keyword.starts_with(trimmed))
        {
            return CompletionContext::StepKeyword;
        }

        if self
            .governance_keywords
            .iter()
            .any(|keyword| keyword.starts_with(trimmed.trim_end_matches(':')))
        {
            return CompletionContext::GovernanceKeyword;
        }

        CompletionContext::Unknown
    }

    /// Complete feature and Gherkin structure keywords.
    fn complete_feature_keywords(prefix: &str) -> Vec<CompletionItem> {
        let fragment = completion_fragment(prefix);
        STRUCTURE_KEYWORDS
            .iter()
            .filter(|keyword| keyword_matches(keyword, fragment))
            .map(|keyword| CompletionItem {
                label: (*keyword).to_string(),
                kind: CompletionKind::StructureKeyword,
                detail: Some("Gherkin structure".to_string()),
                documentation: Some("Insert a Gherkin/Truths structural keyword.".to_string()),
            })
            .collect()
    }

    /// Complete step keywords.
    fn complete_step_keywords(prefix: &str) -> Vec<CompletionItem> {
        let fragment = completion_fragment(prefix);
        STEP_KEYWORDS
            .iter()
            .filter(|keyword| keyword_matches(keyword, fragment))
            .map(|keyword| CompletionItem {
                label: (*keyword).to_string(),
                kind: CompletionKind::StepKeyword,
                detail: Some("Step keyword".to_string()),
                documentation: Some(format!("BDD step keyword: {keyword}")),
            })
            .collect()
    }

    /// Complete governance keywords.
    fn complete_governance_keywords(&self, prefix: &str) -> Vec<CompletionItem> {
        let fragment = completion_fragment(prefix).trim_end_matches(':');
        self.governance_keywords
            .iter()
            .filter(|keyword| keyword_matches(keyword, fragment))
            .map(|keyword| CompletionItem {
                label: format!("{keyword}:"),
                kind: CompletionKind::GovernanceKeyword,
                detail: Some("Governance block".to_string()),
                documentation: Some(format!("Converge governance block: {keyword}")),
            })
            .collect()
    }

    /// Complete tags.
    fn complete_tags(&self, prefix: &str) -> Vec<CompletionItem> {
        let fragment = current_tag_prefix(prefix).unwrap_or_default();
        self.known_tags
            .iter()
            .filter(|tag| tag.starts_with(fragment))
            .map(|tag| CompletionItem {
                label: tag.clone(),
                kind: CompletionKind::Tag,
                detail: Some("Scenario tag".to_string()),
                documentation: None,
            })
            .collect()
    }

    /// Complete steps from definitions.
    fn complete_steps(&self, prefix: &str) -> Vec<CompletionItem> {
        let trimmed = prefix.trim_start();
        let query = step_text_query(trimmed)
            .unwrap_or(trimmed)
            .to_ascii_lowercase();

        self.step_definitions
            .values()
            .filter(|def| {
                let pattern = def.pattern.to_ascii_lowercase();
                query.trim().is_empty() || pattern.contains(query.trim())
            })
            .map(|def| CompletionItem {
                label: def.pattern.clone(),
                kind: CompletionKind::Step,
                detail: def.description.clone(),
                documentation: Some(format!("Step definition: {}", def.pattern)),
            })
            .collect()
    }

    fn complete_snippets(prefix: &str) -> Vec<CompletionItem> {
        if !completion_fragment(prefix).is_empty() {
            return Vec::new();
        }

        vec![CompletionItem {
            label: "Truth template".to_string(),
            kind: CompletionKind::Snippet,
            detail: Some("Governed Truths document".to_string()),
            documentation: Some(
                "Intent, Authority, Evidence, and Scenario scaffold for a new .truths file."
                    .to_string(),
            ),
        }]
    }

    /// Format the document.
    pub fn format(&self, buffer: &TruthTextBuffer) -> Vec<FormattingEdit> {
        let mut edits = Vec::new();

        for line_idx in 0..buffer.line_count() {
            let line = buffer.get_line(line_idx);
            let formatted = self.format_line(&line);
            if formatted != line {
                let (start, end) = buffer.line_range(line_idx);
                edits.push(FormattingEdit {
                    range: TextRange::new(start, end),
                    new_text: formatted,
                });
            }
        }

        edits
    }

    /// Format a single line.
    fn format_line(&self, line: &str) -> String {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return String::new();
        }

        if trimmed.starts_with('#') {
            return format_comment(trimmed);
        }

        if trimmed.starts_with("Feature:") || trimmed.starts_with("Truth:") {
            return trimmed.to_string();
        }

        if trimmed.starts_with('@')
            || matches_prefixed_keyword(
                trimmed,
                &[
                    "Rule:",
                    "Background:",
                    "Scenario:",
                    "Scenario Outline:",
                    "Examples:",
                    "Example:",
                ],
            )
            || self.match_governance_block(trimmed).is_some()
        {
            return format!("  {trimmed}");
        }

        if match_step_keyword(trimmed).is_some()
            || trimmed.starts_with('|')
            || trimmed.starts_with("\"\"\"")
            || trimmed.starts_with("```")
            || match_governance_field(trimmed).is_some()
        {
            return format!("    {trimmed}");
        }

        line.to_string()
    }

    /// Navigate to definition (e.g., step to step definition).
    pub fn navigate_to_definition(
        &self,
        _buffer: &TruthTextBuffer,
        _offset: usize,
    ) -> Option<TextRange> {
        None
    }

    /// Get document symbols for navigation (outline view).
    pub fn get_document_symbols(&self, buffer: &TruthTextBuffer) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        let mut current_feature: Option<usize> = None;

        for line_idx in 0..buffer.line_count() {
            let line = buffer.get_line(line_idx);
            let trimmed = line.trim();
            let (start, end) = buffer.line_range(line_idx);
            let range = TextRange::new(start, end);

            if trimmed.starts_with("Truth:") || trimmed.starts_with("Feature:") {
                current_feature = Some(symbols.len());
                symbols.push(DocumentSymbol {
                    name: trimmed.to_string(),
                    kind: SymbolKind::Feature,
                    range,
                    children: Vec::new(),
                });
            } else if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
                push_symbol(
                    &mut symbols,
                    current_feature,
                    DocumentSymbol {
                        name: trimmed.to_string(),
                        kind: SymbolKind::Scenario,
                        range,
                        children: Vec::new(),
                    },
                );
            } else if self.match_governance_block(trimmed).is_some() {
                push_symbol(
                    &mut symbols,
                    current_feature,
                    DocumentSymbol {
                        name: trimmed.to_string(),
                        kind: SymbolKind::GovernanceBlock,
                        range,
                        children: Vec::new(),
                    },
                );
            } else if trimmed.starts_with('@') {
                push_symbol(
                    &mut symbols,
                    current_feature,
                    DocumentSymbol {
                        name: trimmed.to_string(),
                        kind: SymbolKind::Tag,
                        range,
                        children: Vec::new(),
                    },
                );
            }
        }

        symbols
    }
}

/// Completion context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    Tag,
    GovernanceKeyword,
    StepKeyword,
    StepText,
    FeatureKeyword,
    Unknown,
}

/// Document symbol for navigation.
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: TextRange,
    pub children: Vec<DocumentSymbol>,
}

/// Kind of document symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Feature,
    Scenario,
    Step,
    GovernanceBlock,
    Tag,
    Table,
    Comment,
}

/// Parsed document with structure.
#[derive(Debug, Clone)]
pub struct ParsedTruthDocument {
    pub text: String,
    pub preprocessed: String,
    pub governance: crate::truths::TruthGovernance,
    pub gherkin_content: String,
    pub errors: Vec<Diagnostic>,
}

fn default_step_definitions() -> Vec<StepDefinition> {
    vec![
        StepDefinition {
            keyword: "Given".to_string(),
            pattern: "Given I have a valid user account".to_string(),
            description: Some("Setup authenticated user".to_string()),
        },
        StepDefinition {
            keyword: "Given".to_string(),
            pattern: "Given the system is initialized".to_string(),
            description: Some("Setup initialized system".to_string()),
        },
        StepDefinition {
            keyword: "When".to_string(),
            pattern: "When I perform the action".to_string(),
            description: Some("Trigger main action".to_string()),
        },
        StepDefinition {
            keyword: "Then".to_string(),
            pattern: "Then the result should be successful".to_string(),
            description: Some("Verify success".to_string()),
        },
    ]
}

fn parse_error_diagnostic(text: &str, message: String) -> Diagnostic {
    Diagnostic {
        range: TextRange::new(0, text.len()),
        severity: DiagnosticSeverity::Error,
        message,
        code: Some("truths.parse".to_string()),
    }
}

fn push_token(tokens: &mut Vec<SyntaxToken>, start: usize, end: usize, kind: SyntaxKind) {
    if start < end {
        tokens.push(SyntaxToken { start, end, kind });
    }
}

fn push_symbol(symbols: &mut Vec<DocumentSymbol>, parent: Option<usize>, symbol: DocumentSymbol) {
    if let Some(parent_idx) = parent {
        symbols[parent_idx].children.push(symbol);
    } else {
        symbols.push(symbol);
    }
}

fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line)
}

fn leading_whitespace_len(line: &str) -> usize {
    line.char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
        .unwrap_or(line.len())
}

fn highlight_tags(line: &str, line_start: usize, tokens: &mut Vec<SyntaxToken>) {
    let mut search_start = 0;
    while let Some(relative_start) = line[search_start..].find('@') {
        let start = search_start + relative_start;
        let rest = &line[start..];
        let end = rest
            .char_indices()
            .skip(1)
            .find_map(|(idx, ch)| (!is_tag_char(ch)).then_some(start + idx))
            .unwrap_or(line.len());

        if end > start + 1 {
            push_token(
                tokens,
                line_start + start,
                line_start + end,
                SyntaxKind::Tag,
            );
        }

        search_start = end;
    }
}

fn highlight_strings(line: &str, line_start: usize, tokens: &mut Vec<SyntaxToken>) {
    let mut search_start = 0;
    while let Some(open_relative) = line[search_start..].find('"') {
        let open = search_start + open_relative;
        let after_open = open + 1;
        if let Some(close_relative) = line[after_open..].find('"') {
            let close = after_open + close_relative + 1;
            push_token(
                tokens,
                line_start + open,
                line_start + close,
                SyntaxKind::String,
            );
            search_start = close;
        } else {
            break;
        }
    }
}

fn is_tag_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':')
}

fn find_inline_comment(line: &str) -> Option<usize> {
    line.char_indices()
        .find_map(|(idx, ch)| (ch == '#').then_some(idx))
}

fn match_prefixed_keyword<'a>(trimmed: &str, keywords: &'a [&str]) -> Option<&'a str> {
    keywords
        .iter()
        .copied()
        .find(|keyword| trimmed.starts_with(keyword))
}

fn matches_prefixed_keyword(trimmed: &str, keywords: &[&str]) -> bool {
    match_prefixed_keyword(trimmed, keywords).is_some()
}

fn match_step_keyword(trimmed: &str) -> Option<&'static str> {
    STEP_KEYWORDS.iter().copied().find(|keyword| {
        trimmed == *keyword
            || trimmed
                .strip_prefix(keyword)
                .is_some_and(|rest| rest.chars().next().is_some_and(char::is_whitespace))
    })
}

fn match_governance_field(trimmed: &str) -> Option<&'static str> {
    GOVERNANCE_FIELDS.iter().copied().find(|field| {
        trimmed
            .strip_prefix(field)
            .is_some_and(|rest| rest.starts_with(':'))
    })
}

fn prefix_at_byte(line: &str, byte: usize) -> &str {
    let end = previous_char_boundary(line, byte.min(line.len()));
    &line[..end]
}

fn previous_char_boundary(value: &str, mut byte: usize) -> usize {
    while byte > 0 && !value.is_char_boundary(byte) {
        byte -= 1;
    }
    byte
}

fn current_tag_prefix(prefix: &str) -> Option<&str> {
    let at = prefix.rfind('@')?;
    let tag = &prefix[at + 1..];
    (!tag.chars().any(char::is_whitespace)).then_some(tag)
}

fn completion_fragment(prefix: &str) -> &str {
    let trimmed = prefix.trim_start();
    trimmed
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or(trimmed)
}

fn keyword_matches(keyword: &str, fragment: &str) -> bool {
    fragment.is_empty()
        || keyword
            .to_ascii_lowercase()
            .starts_with(&fragment.to_ascii_lowercase())
}

fn step_text_query(trimmed_prefix: &str) -> Option<&str> {
    STEP_KEYWORDS.iter().find_map(|keyword| {
        let rest = trimmed_prefix.strip_prefix(keyword)?;
        let first = rest.chars().next()?;
        first.is_whitespace().then_some(&rest[first.len_utf8()..])
    })
}

fn format_comment(trimmed: &str) -> String {
    if trimmed.len() > 1 && !trimmed.chars().nth(1).is_some_and(char::is_whitespace) {
        format!("# {}", &trimmed[1..])
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = TruthEditorService::new();
        assert!(!service.governance_keywords.is_empty());
        assert!(!service.known_tags.is_empty());
    }

    #[test]
    fn test_complete_step_keywords() {
        let service = TruthEditorService::new();
        let buffer = TruthTextBuffer::from_str("    G");
        let completions = service.complete(&buffer, 5);

        assert!(!completions.is_empty());
        assert!(
            completions
                .iter()
                .any(|completion| completion.label == "Given")
        );
    }

    #[test]
    fn test_complete_governance_keywords() {
        let service = TruthEditorService::new();
        let buffer = TruthTextBuffer::from_str("  I");
        let completions = service.complete(&buffer, 3);

        assert!(!completions.is_empty());
        assert!(
            completions
                .iter()
                .any(|completion| completion.label == "Intent:")
        );
    }

    #[test]
    fn test_complete_step_text_after_keyword() {
        let service = TruthEditorService::new();
        let buffer = TruthTextBuffer::from_str("    Given I");
        let completions = service.complete(&buffer, buffer.len());

        assert!(
            completions
                .iter()
                .any(|completion| completion.label == "Given I have a valid user account")
        );
    }

    #[test]
    fn test_highlight_returns_line_based_tokens() {
        let service = TruthEditorService::new();
        let content = "Truth: Test\n  @invariant\n  Intent:\n    Outcome: safe\n  Scenario: Test\n    Given a \"test\"\n";
        let buffer = TruthTextBuffer::from_str(content);
        let tokens = service.highlight(&buffer);

        assert!(tokens.iter().any(|token| token.kind == SyntaxKind::Keyword));
        assert!(tokens.iter().any(|token| token.kind == SyntaxKind::Tag));
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == SyntaxKind::GovernanceKeyword)
        );
        assert!(
            tokens
                .iter()
                .any(|token| token.kind == SyntaxKind::StepKeyword)
        );
        assert!(tokens.iter().any(|token| token.kind == SyntaxKind::String));
    }

    #[test]
    fn test_parse_document_reports_gherkin_errors() {
        let service = TruthEditorService::new();
        let buffer = TruthTextBuffer::from_str("not a truth");

        let diagnostics = service.parse_document(&buffer).unwrap_err();
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_document_symbols_are_nested_under_feature() {
        let service = TruthEditorService::new();
        let content =
            "Truth: Test\n  Intent:\n    Outcome: test\n  Scenario: Test\n    Given a test";
        let buffer = TruthTextBuffer::from_str(content);
        let symbols = service.get_document_symbols(&buffer);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].kind, SymbolKind::Feature);
        assert!(
            symbols[0]
                .children
                .iter()
                .any(|symbol| symbol.kind == SymbolKind::Scenario)
        );
        assert!(
            symbols[0]
                .children
                .iter()
                .any(|symbol| symbol.kind == SymbolKind::GovernanceBlock)
        );
    }
}
