// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Provider-specific prompt structuring and optimization.
//!
//! This module provides provider-specific enhancements to the Converge Prompt DSL,
//! including format optimizations, structured output handling, and parsing utilities.
//!
//! # Philosophy
//!
//! - Use EDN format for prompts (from converge-core)
//! - Apply provider-specific optimizations (e.g., XML tags for Claude)
//! - Parse structured outputs reliably
//! - Maintain token efficiency

use crate::provider_api::LlmResponse;
use converge_core::prompt::{
    AgentPrompt, AgentRole, Constraint, OutputContract, PromptContext, PromptFormat,
};
use converge_core::{ContextKey, ProposedFact};

/// Provider-specific prompt builder.
///
/// Enhances the base `AgentPrompt` with provider-specific optimizations.
pub struct ProviderPromptBuilder {
    base: AgentPrompt,
    /// Provider-specific format hints (e.g., XML for Claude).
    output_format_hint: Option<String>,
}

impl ProviderPromptBuilder {
    /// Creates a new builder from a base prompt.
    #[must_use]
    pub fn new(base: AgentPrompt) -> Self {
        Self {
            base,
            output_format_hint: None,
        }
    }

    /// Sets the output format hint for the provider.
    ///
    /// For Claude, use "xml" to request XML-tagged responses.
    /// For `OpenAI`, use "json" to enable JSON mode.
    #[must_use]
    pub fn with_output_format(mut self, format: impl Into<String>) -> Self {
        self.output_format_hint = Some(format.into());
        self
    }

    /// Builds the final prompt string optimized for the provider.
    ///
    /// For Claude:
    /// - Wraps EDN prompt in XML tags for better instruction following
    /// - Adds XML output format instructions
    ///
    /// For other providers:
    /// - Returns EDN as-is
    #[must_use]
    pub fn build_for_claude(&self) -> String {
        let edn_prompt = self.base.serialize(PromptFormat::Edn);

        // Claude works well with XML tags - wrap the EDN prompt
        let mut prompt = String::from("<prompt>\n");
        prompt.push_str(&edn_prompt);
        prompt.push_str("\n</prompt>\n\n");

        // Add output format instructions
        if let Some(ref format) = self.output_format_hint {
            if format.as_str() == "xml" {
                prompt.push_str("<instructions>\n");
                prompt.push_str("Respond in XML format with the following structure:\n");
                prompt.push_str("<response>\n");
                prompt.push_str("  <proposals>\n");
                prompt.push_str(
                    "    <proposal id=\"...\" confidence=\"0.0-1.0\">content</proposal>\n",
                );
                prompt.push_str("  </proposals>\n");
                prompt.push_str("</response>\n");
                prompt.push_str("</instructions>");
            } else {
                // For other formats, just note the expected format
                prompt.push_str("<instructions>Respond in ");
                prompt.push_str(format);
                prompt.push_str(" format.</instructions>");
            }
        } else {
            // Default: request structured output
            prompt.push_str("<instructions>\n");
            prompt.push_str("Respond with proposed facts in a structured format.\n");
            prompt.push_str("Each proposal should include: id, content, confidence (0.0-1.0).\n");
            prompt.push_str("</instructions>");
        }

        prompt
    }

    /// Builds EDN prompt without XML wrapping (pure EDN).
    ///
    /// This is useful for:
    /// - Testing token efficiency without XML overhead
    /// - Providers that don't benefit from XML tags
    /// - When you want maximum token savings
    #[must_use]
    pub fn build_edn_only(&self) -> String {
        self.base.serialize(PromptFormat::Edn)
    }

    /// Builds the final prompt string for `OpenAI`.
    ///
    /// `OpenAI` benefits from JSON mode, so we:
    /// - Keep EDN for input (it's compact)
    /// - Request JSON output format
    #[must_use]
    pub fn build_for_openai(&self) -> String {
        let edn_prompt = self.base.serialize(PromptFormat::Edn);

        let mut prompt = String::from("Prompt (EDN format):\n");
        prompt.push_str(&edn_prompt);
        prompt.push_str("\n\n");

        // Request JSON output
        prompt.push_str("Respond with a JSON object containing an array of proposals:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"proposals\": [\n");
        prompt.push_str("    {\"id\": \"...\", \"content\": \"...\", \"confidence\": 0.0-1.0}\n");
        prompt.push_str("  ]\n");
        prompt.push_str("}\n");

        prompt
    }

    /// Builds the prompt for a generic provider (EDN as-is).
    #[must_use]
    pub fn build_generic(&self) -> String {
        self.base.serialize(PromptFormat::Edn)
    }
}

/// Parser for structured LLM responses.
///
/// Handles provider-specific response formats (XML for Claude, JSON for `OpenAI`).
pub struct StructuredResponseParser;

impl StructuredResponseParser {
    /// Parses a Claude XML response into `ProposedFacts`.
    ///
    /// Expected XML format:
    /// ```xml
    /// <response>
    ///   <proposals>
    ///     <proposal id="..." confidence="0.85">content</proposal>
    ///   </proposals>
    /// </response>
    /// ```
    #[must_use]
    pub fn parse_claude_xml(
        response: &LlmResponse,
        target_key: ContextKey,
        model: &str,
    ) -> Vec<ProposedFact> {
        let content = &response.content;

        // Simple XML parsing (for MVP - could use a proper XML parser later)
        let mut proposals = Vec::new();
        let mut in_proposal = false;
        let mut current_id = String::new();
        let mut current_confidence = 0.7; // default
        let mut current_content = String::new();

        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            let line = line.trim();

            // Extract proposal attributes
            if line.starts_with("<proposal") {
                in_proposal = true;
                // Extract id="..." and confidence="..."
                if let Some(id_start) = line.find("id=\"") {
                    let id_end = line[id_start + 4..].find('"').unwrap_or(0);
                    current_id = line[id_start + 4..id_start + 4 + id_end].to_string();
                }
                if let Some(conf_start) = line.find("confidence=\"") {
                    let conf_end = line[conf_start + 12..].find('"').unwrap_or(0);
                    if let Ok(conf) =
                        line[conf_start + 12..conf_start + 12 + conf_end].parse::<f64>()
                    {
                        current_confidence = conf;
                    }
                }
                // Extract content between > and </proposal>
                if let Some(content_start) = line.find('>')
                    && let Some(content_end) = line.find("</proposal>")
                {
                    current_content = line[content_start + 1..content_end].trim().to_string();
                }
            } else if in_proposal
                && !line.starts_with("</proposal>")
                && !line.starts_with("<proposal")
            {
                // Multi-line content
                if !current_content.is_empty() {
                    current_content.push(' ');
                }
                current_content.push_str(line);
            }

            if line.contains("</proposal>") {
                if !current_id.is_empty() && !current_content.is_empty() {
                    proposals.push(ProposedFact {
                        key: target_key,
                        id: current_id.clone(),
                        content: current_content.clone(),
                        confidence: current_confidence,
                        provenance: format!("{}:{}", model, response.model),
                    });
                }
                in_proposal = false;
                current_id.clear();
                current_content.clear();
                current_confidence = 0.7;
            }
        }

        proposals
    }

    /// Parses an `OpenAI` JSON response into `ProposedFacts`.
    ///
    /// Expected JSON format:
    /// ```json
    /// {
    ///   "proposals": [
    ///     {"id": "...", "content": "...", "confidence": 0.85}
    ///   ]
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if JSON parsing fails or expected fields are missing.
    pub fn parse_openai_json(
        response: &LlmResponse,
        target_key: ContextKey,
        model: &str,
    ) -> Result<Vec<ProposedFact>, String> {
        use serde_json::Value;

        let json: Value = serde_json::from_str(&response.content)
            .map_err(|e| format!("Failed to parse JSON: {e}"))?;

        let mut proposals = Vec::new();

        if let Some(proposals_array) = json.get("proposals").and_then(|v| v.as_array()) {
            for proposal in proposals_array {
                let id = proposal
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing or invalid 'id' field".to_string())?
                    .to_string();

                let content = proposal
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing or invalid 'content' field".to_string())?
                    .to_string();

                let confidence = proposal
                    .get("confidence")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.7);

                proposals.push(ProposedFact {
                    key: target_key,
                    id,
                    content,
                    confidence,
                    provenance: format!("{}:{}", model, response.model),
                });
            }
        } else {
            // Fallback: try to parse as a single proposal
            if let (Some(id), Some(content)) = (
                json.get("id").and_then(|v| v.as_str()),
                json.get("content").and_then(|v| v.as_str()),
            ) {
                let confidence = json
                    .get("confidence")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.7);
                proposals.push(ProposedFact {
                    key: target_key,
                    id: id.to_string(),
                    content: content.to_string(),
                    confidence,
                    provenance: format!("{}:{}", model, response.model),
                });
            } else {
                return Err("No proposals found in JSON response".to_string());
            }
        }

        Ok(proposals)
    }

    /// Parses a generic response (fallback to simple parsing).
    #[must_use]
    pub fn parse_generic(
        response: &LlmResponse,
        target_key: ContextKey,
        model: &str,
    ) -> Vec<ProposedFact> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Generate a simple ID from timestamp
        let id = SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
            |_| "proposal-0".to_string(),
            |d| format!("proposal-{:x}", d.as_nanos() % 0xFFFF_FFFF),
        );

        // Fallback: treat entire response as a single proposal
        vec![ProposedFact {
            key: target_key,
            id,
            content: response.content.clone(),
            confidence: 0.7,
            provenance: format!("{}:{}", model, response.model),
        }]
    }
}

/// Helper function to build a prompt for Claude with XML optimization.
pub fn build_claude_prompt(
    role: AgentRole,
    objective: impl Into<String>,
    context: PromptContext,
    output_contract: OutputContract,
    constraints: impl IntoIterator<Item = Constraint>,
) -> String {
    let base =
        AgentPrompt::new(role, objective, context, output_contract).with_constraints(constraints);

    ProviderPromptBuilder::new(base)
        .with_output_format("xml")
        .build_for_claude()
}

/// Helper function to build a prompt for `OpenAI` with JSON optimization.
pub fn build_openai_prompt(
    role: AgentRole,
    objective: impl Into<String>,
    context: PromptContext,
    output_contract: OutputContract,
    constraints: impl IntoIterator<Item = Constraint>,
) -> String {
    let base =
        AgentPrompt::new(role, objective, context, output_contract).with_constraints(constraints);

    ProviderPromptBuilder::new(base).build_for_openai()
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Known constant values in tests
mod tests {
    use super::*;
    use converge_core::{Context, Engine, Fact};

    fn promoted_fact(key: ContextKey, id: &str, content: &str) -> Fact {
        let mut ctx = Context::new();
        let _ = ctx.add_input(key, id, content);
        Engine::new()
            .run(ctx)
            .expect("should promote test input")
            .context
            .get(key)
            .first()
            .expect("promoted fact should exist")
            .clone()
    }

    #[test]
    fn test_claude_prompt_building() {
        let mut ctx = PromptContext::new();
        ctx.add_facts(
            ContextKey::Signals,
            vec![promoted_fact(ContextKey::Signals, "s1", "Test signal")],
        );

        let prompt = build_claude_prompt(
            AgentRole::Proposer,
            "test-objective",
            ctx,
            OutputContract::new("proposed-fact", ContextKey::Competitors),
            vec![Constraint::NoInvent, Constraint::NoHallucinate],
        );

        assert!(prompt.contains("<prompt>"));
        assert!(prompt.contains(":r :proposer"));
        assert!(prompt.contains("<instructions>"));
        assert!(prompt.contains("XML format"));
    }

    #[test]
    fn test_openai_prompt_building() {
        let ctx = PromptContext::new();

        let prompt = build_openai_prompt(
            AgentRole::Proposer,
            "test-objective",
            ctx,
            OutputContract::new("proposed-fact", ContextKey::Strategies),
            vec![Constraint::NoInvent],
        );

        assert!(prompt.contains("EDN format"));
        assert!(prompt.contains("JSON"));
        assert!(prompt.contains("proposals"));
    }

    #[test]
    fn test_claude_xml_parsing() {
        let xml_response = r#"
<response>
  <proposals>
    <proposal id="p1" confidence="0.85">Test content 1</proposal>
    <proposal id="p2" confidence="0.90">Test content 2</proposal>
  </proposals>
</response>
"#;

        let response = LlmResponse {
            content: xml_response.to_string(),
            model: "claude-sonnet-4-6".to_string(),
            usage: crate::provider_api::TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
            finish_reason: crate::provider_api::FinishReason::Stop,
        };

        let proposals = StructuredResponseParser::parse_claude_xml(
            &response,
            ContextKey::Competitors,
            "anthropic",
        );

        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].id, "p1");
        assert_eq!(proposals[0].confidence, 0.85);
        assert_eq!(proposals[1].id, "p2");
        assert_eq!(proposals[1].confidence, 0.90);
    }

    #[test]
    fn test_openai_json_parsing() {
        let json_response = r#"
{
  "proposals": [
    {"id": "p1", "content": "Test content 1", "confidence": 0.85},
    {"id": "p2", "content": "Test content 2", "confidence": 0.90}
  ]
}
"#;

        let response = LlmResponse {
            content: json_response.to_string(),
            model: "gpt-4".to_string(),
            usage: crate::provider_api::TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
            finish_reason: crate::provider_api::FinishReason::Stop,
        };

        let proposals = StructuredResponseParser::parse_openai_json(
            &response,
            ContextKey::Strategies,
            "openai",
        )
        .unwrap();

        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].id, "p1");
        assert_eq!(proposals[0].confidence, 0.85);
        assert_eq!(proposals[1].id, "p2");
        assert_eq!(proposals[1].confidence, 0.90);
    }
}
