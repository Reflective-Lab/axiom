// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Truth heading guidance — LLM-powered and local heuristic rewriting.
//!
//! Evaluates the `Truth:` heading in a Converge Truth spec and suggests
//! stronger formulations. A good heading states a durable governed truth,
//! not a topic or initiative label.

use std::sync::Arc;

use converge_provider::{ChatBackendSelectionConfig, select_healthy_chat_backend};
use converge_provider_api::{ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat};
use serde::{Deserialize, Serialize};

use crate::truths::parse_truth_document;

/// Configuration for heading guidance.
#[derive(Debug, Clone, Default)]
pub struct GuidanceConfig {
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
}

/// The result of evaluating a Truth heading.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GuidanceResponse {
    pub current_title: String,
    pub suggested_title: String,
    pub should_rewrite: bool,
    pub source: String,
    pub source_label: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub rationale: Vec<String>,
    pub description_hints: Vec<String>,
    pub note: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LlmGuidance {
    #[serde(default)]
    should_rewrite: bool,
    #[serde(default)]
    suggested_title: String,
    #[serde(default)]
    rationale: Vec<String>,
    #[serde(default)]
    description_hints: Vec<String>,
}

/// Draft context extracted from the spec for guidance prompts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftContext {
    pub title: String,
    pub description_line_count: usize,
    pub scenario_count: usize,
    pub has_intent: bool,
    pub has_authority: bool,
    pub has_constraint: bool,
    pub has_evidence: bool,
    pub has_exception: bool,
}

/// Extract the Truth/Feature title from a spec.
pub fn extract_title(spec: &str) -> Option<String> {
    spec.lines().find_map(|line| {
        let trimmed = line.trim_start();
        trimmed
            .strip_prefix("Truth:")
            .or_else(|| trimmed.strip_prefix("Feature:"))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// Full guidance flow: try LLM first, fall back to local heuristics.
pub async fn guide_heading(spec: &str, config: &GuidanceConfig) -> Option<GuidanceResponse> {
    let current_title = extract_title(spec)?;

    let response = match request_live_guidance(spec, &current_title, config).await {
        Ok(response) => response,
        Err(error) => local_heading_guidance(
            spec,
            &current_title,
            format!("Live guidance failed, showing local rewrite: {error}"),
        ),
    };

    Some(response)
}

/// LLM-powered heading guidance using the configured chat backend.
async fn request_live_guidance(
    spec: &str,
    current_title: &str,
    config: &GuidanceConfig,
) -> Result<GuidanceResponse, String> {
    let ctx = draft_context(spec, current_title);
    let selected = select_backend(config).await?;
    let prompt = build_prompt(current_title, &ctx, spec)?;

    let response = selected
        .backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt,
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some("You are a strict Converge Truth editor. Respond with JSON only.".into()),
            tools: Vec::new(),
            response_format: ResponseFormat::Json,
            max_tokens: Some(300),
            temperature: Some(0.2),
            stop_sequences: Vec::new(),
            model: config.model_override.clone(),
        })
        .await
        .map_err(|error| format!("ChatBackend request failed: {error}"))?;

    let parsed = parse_llm_guidance(&response.content)?;
    let suggested_title = sanitize_suggested_title(&parsed.suggested_title, current_title);

    Ok(GuidanceResponse {
        current_title: current_title.to_string(),
        suggested_title,
        should_rewrite: parsed.should_rewrite,
        source: "live-chat-backend".into(),
        source_label: provider_label(&selected.provider).to_string(),
        provider: Some(selected.provider.clone()),
        model: Some(selected.model.clone()),
        rationale: normalize_rationale(
            parsed.rationale,
            "The current heading was evaluated against the full Truth context.".to_string(),
        ),
        description_hints: normalize_description_hints(parsed.description_hints),
        note: format!(
            "Live guidance is active through {} using model `{}`.",
            provider_label(&selected.provider),
            selected.model
        ),
    })
}

struct SelectedBackend {
    backend: Arc<dyn DynChatBackend>,
    provider: String,
    model: String,
}

async fn select_backend(config: &GuidanceConfig) -> Result<SelectedBackend, String> {
    let mut selection = ChatBackendSelectionConfig::from_env()
        .map_err(|error| format!("ChatBackend selection configuration failed: {error}"))?;
    if let Some(provider) = &config.provider_override {
        selection = selection.with_provider_override(provider.clone());
    }
    let selected = select_healthy_chat_backend(&selection)
        .await
        .map_err(|error| format!("No live chat backend is available: {error}"))?;
    let provider = selected.provider().to_string();
    let model = selected.model().to_string();
    Ok(SelectedBackend {
        backend: selected.backend,
        provider,
        model,
    })
}

fn build_prompt(current_title: &str, ctx: &DraftContext, spec: &str) -> Result<String, String> {
    Ok(format!(
        r#"You are improving a Converge Truth heading.

A strong heading states a durable business truth, decision rule, or governed outcome.
A weak heading reads like a topic, workstream, or initiative label.

Current heading:
{current_title}

Parsed draft context:
{draft_context}

Spec excerpt:
{spec_excerpt}

Return ONLY a JSON object with this exact schema:
{{
  "shouldRewrite": true,
  "suggestedTitle": "Enterprise AI vendor selection is auditable, constrained, and approval-gated",
  "rationale": [
    "Current heading reads like a topic, not a governed truth."
  ],
  "descriptionHints": [
    "Vendor choice must be reproducible from explicit evidence.",
    "Final selection must stay within policy, budget, and approval boundaries."
  ]
}}

Rules:
- Do not include `Truth:` in suggestedTitle.
- Keep suggestedTitle concise.
- Prefer declarative language such as `is`, `must`, `requires`, `remains`, or `produces`.
- Align the rewrite with the governance, evidence, and approval context in the spec.
- If the current heading is already strong, set shouldRewrite to false and keep suggestedTitle equal to the current heading.
- descriptionHints should be 0-2 concise lines suitable immediately below the Truth header."#,
        draft_context = serde_json::to_string_pretty(ctx)
            .map_err(|e| format!("Failed to serialize draft context: {e}"))?,
        spec_excerpt = truncated_excerpt(spec)
    ))
}

fn parse_llm_guidance(content: &str) -> Result<LlmGuidance, String> {
    let payload = content.trim();
    let json = match (payload.find('{'), payload.rfind('}')) {
        (Some(start), Some(end)) if start <= end => &payload[start..=end],
        _ => payload,
    };
    serde_json::from_str(json).map_err(|e| format!("LLM returned invalid guidance JSON: {e}"))
}

/// Local heuristic-based heading guidance (no LLM needed).
pub fn local_heading_guidance(spec: &str, current_title: &str, note: String) -> GuidanceResponse {
    let ctx = draft_context(spec, current_title);
    let title_lower = current_title.trim().to_ascii_lowercase();
    let spec_lower = spec.to_ascii_lowercase();
    let mentions_approval = spec_lower.contains("approval");
    let mentions_traceability = spec_lower.contains("traceable")
        || spec_lower.contains("audit")
        || spec_lower.contains("provenance")
        || spec_lower.contains("compliance");
    let mentions_policy = spec_lower.contains("governance")
        || spec_lower.contains("policy")
        || spec_lower.contains("budget")
        || spec_lower.contains("cost")
        || ctx.has_constraint;
    let has_assertive_verb = title_is_declarative(&title_lower);
    let sounds_like_topic = title_lower.contains(" for ")
        || title_lower.contains(" workflow")
        || title_lower.contains(" rollout")
        || title_lower.contains(" process")
        || !has_assertive_verb;

    let subject = normalize_subject(current_title.trim());
    let predicate = quality_predicate(
        ctx.has_authority || mentions_approval,
        ctx.has_constraint || mentions_policy,
        ctx.has_evidence || mentions_traceability,
    );
    let suggested_title = if sounds_like_topic {
        format!("{subject} {predicate}")
    } else {
        current_title.trim().to_string()
    };

    let mut rationale = Vec::new();
    if title_lower.contains(" for ") {
        rationale.push(
            "The current heading reads like a topic scoped to an initiative, not a governed truth."
                .into(),
        );
    }
    if !has_assertive_verb {
        rationale.push("A Converge Truth heading should state a claim or rule, usually with language like `is`, `must`, or `requires`.".into());
    }
    if !ctx.has_constraint {
        rationale.push("Vendor-selection truths are stronger when the title is backed by explicit constraints.".into());
    }
    if !ctx.has_evidence {
        rationale.push("Vendor-selection truths should usually imply what evidence makes the decision auditable.".into());
    }

    let description_hints = build_description_hints(&ctx, spec);

    GuidanceResponse {
        current_title: current_title.trim().to_string(),
        suggested_title,
        should_rewrite: sounds_like_topic,
        source: "local-heuristic".into(),
        source_label: "Local".into(),
        provider: None,
        model: None,
        rationale: normalize_rationale(
            rationale,
            "The editor is checking whether the heading is written as a stable truth instead of a topic label.".into(),
        ),
        description_hints,
        note,
    }
}

/// Extract draft context from a spec.
pub fn draft_context(spec: &str, current_title: &str) -> DraftContext {
    if let Ok(document) = parse_truth_document(spec) {
        return DraftContext {
            title: current_title.trim().to_string(),
            description_line_count: description_line_count(spec),
            scenario_count: document
                .gherkin
                .lines()
                .filter(|line| line.trim_start().starts_with("Scenario:"))
                .count(),
            has_intent: document.governance.intent.is_some(),
            has_authority: document.governance.authority.is_some(),
            has_constraint: document.governance.constraint.is_some(),
            has_evidence: document.governance.evidence.is_some(),
            has_exception: document.governance.exception.is_some(),
        };
    }

    DraftContext {
        title: current_title.trim().to_string(),
        description_line_count: description_line_count(spec),
        scenario_count: spec
            .lines()
            .filter(|line| line.trim_start().starts_with("Scenario:"))
            .count(),
        has_intent: spec.contains("\nIntent:"),
        has_authority: spec.contains("\nAuthority:"),
        has_constraint: spec.contains("\nConstraint:"),
        has_evidence: spec.contains("\nEvidence:"),
        has_exception: spec.contains("\nException:"),
    }
}

// ─── Helpers ───

fn provider_label(provider: &str) -> &'static str {
    match provider {
        "openrouter" => "OpenRouter",
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        "gemini" => "Gemini",
        "mistral" => "Mistral",
        _ => "Live",
    }
}

fn sanitize_suggested_title(suggested: &str, fallback: &str) -> String {
    let trimmed = suggested.trim();
    let stripped = trimmed
        .strip_prefix("Truth:")
        .or_else(|| trimmed.strip_prefix("Feature:"))
        .map_or(trimmed, str::trim);
    if stripped.is_empty() {
        fallback.to_string()
    } else {
        stripped.to_string()
    }
}

fn normalize_description_hints(mut hints: Vec<String>) -> Vec<String> {
    hints.retain(|h| !h.trim().is_empty());
    hints.truncate(2);
    hints
}

fn normalize_rationale(mut rationale: Vec<String>, fallback: String) -> Vec<String> {
    rationale.retain(|r| !r.trim().is_empty());
    if rationale.is_empty() {
        rationale.push(fallback);
    }
    rationale
}

fn build_description_hints(ctx: &DraftContext, spec: &str) -> Vec<String> {
    let mut hints = Vec::new();
    if ctx.description_line_count == 0 {
        hints.push("Vendor choice must be reproducible from explicit evidence.".into());
    }
    if ctx.has_authority || spec.to_ascii_lowercase().contains("approval") {
        hints.push("Final selection must stay within accountable approval boundaries.".into());
    } else if !ctx.has_constraint {
        hints.push("Selection must stay within policy, cost, and risk boundaries.".into());
    }
    if !ctx.has_evidence && hints.len() < 2 {
        hints
            .push("The recommended vendor must be justified by traceable review artifacts.".into());
    }
    normalize_description_hints(hints)
}

fn quality_predicate(has_authority: bool, has_constraint: bool, has_evidence: bool) -> String {
    let mut q = Vec::new();
    if has_evidence {
        q.push("auditable");
    }
    if has_constraint {
        q.push("constrained");
    }
    if has_authority {
        q.push("approval-gated");
    }
    if q.is_empty() {
        q.push("explicit");
        q.push("reviewable");
    }
    format!("is {}", join_phrases(&q))
}

fn join_phrases(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_string(),
        [a, b] => format!("{a} and {b}"),
        [first, middle @ .., last] => format!("{first}, {}, and {last}", middle.join(", ")),
    }
}

fn title_is_declarative(title_lower: &str) -> bool {
    [
        " is ",
        " must ",
        " requires ",
        " remains ",
        " produces ",
        " blocks ",
        " allows ",
    ]
    .iter()
    .any(|n| title_lower.contains(n))
}

fn normalize_subject(title: &str) -> String {
    let trimmed = title.trim().trim_end_matches('.');
    if let Some((left, right)) = trimmed.split_once(" for ") {
        let left = left.trim();
        if reorderable_subject(left) {
            let right = strip_context_suffix(right.trim());
            return uppercase_first(&format!("{} {}", right, left.to_ascii_lowercase()));
        }
    }
    uppercase_first(trimmed)
}

fn reorderable_subject(left: &str) -> bool {
    let l = left.to_ascii_lowercase();
    [
        "selection",
        "evaluation",
        "approval",
        "review",
        "screening",
        "comparison",
    ]
    .iter()
    .any(|s| l.ends_with(s))
}

fn strip_context_suffix(value: &str) -> String {
    for suffix in [" rollout", " workflow", " process", " program"] {
        if let Some(stripped) = value.strip_suffix(suffix) {
            return stripped.trim().to_string();
        }
    }
    value.trim().to_string()
}

fn uppercase_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.as_str())
}

fn description_line_count(spec: &str) -> usize {
    let lines: Vec<&str> = spec.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Truth:") || trimmed.starts_with("Feature:") {
            let mut count = 0;
            for next in lines.iter().skip(idx + 1) {
                let t = next.trim();
                if t.is_empty() {
                    if count > 0 {
                        break;
                    }
                    continue;
                }
                if is_heading_boundary(t) {
                    break;
                }
                count += 1;
            }
            return count;
        }
    }
    0
}

fn is_heading_boundary(line: &str) -> bool {
    matches!(
        line,
        "Intent:" | "Authority:" | "Constraint:" | "Evidence:" | "Exception:"
    ) || line.starts_with('@')
        || line.starts_with("Background:")
        || line.starts_with("Scenario:")
        || line.starts_with("Rule:")
        || line.starts_with("Example:")
        || line.starts_with("Examples:")
}

fn truncated_excerpt(spec: &str) -> String {
    const MAX_LINES: usize = 20;
    const MAX_CHARS: usize = 2200;
    let mut excerpt = spec.lines().take(MAX_LINES).collect::<Vec<_>>().join("\n");
    if excerpt.chars().count() > MAX_CHARS {
        excerpt = excerpt.chars().take(MAX_CHARS).collect::<String>();
        excerpt.push_str("\n...");
    }
    excerpt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_rewrites_topic_titles() {
        let guidance = local_heading_guidance(
            r#"Truth: Vendor selection for enterprise AI rollout

Authority:
  Actor: governance_review_board
  Requires Approval: final_vendor_selection

Scenario: Candidate vendors produce traceable evaluation outcomes
  Given candidate vendors "Acme AI, Beta ML, Gamma LLM"
  When the governance workflow evaluates each vendor
  Then each vendor should produce a compliance screening result
"#,
            "Vendor selection for enterprise AI rollout",
            "local".into(),
        );

        assert!(guidance.should_rewrite);
        assert_eq!(
            guidance.suggested_title,
            "Enterprise AI vendor selection is auditable, constrained, and approval-gated"
        );
    }

    #[test]
    fn local_keeps_declarative_titles() {
        let guidance = local_heading_guidance(
            r"Truth: Enterprise AI vendor selection is auditable and approval-gated

Constraint:
  Cost Limit: first-year spend stays within procurement budget

Evidence:
  Requires: security_assessment
",
            "Enterprise AI vendor selection is auditable and approval-gated",
            "local".into(),
        );

        assert!(!guidance.should_rewrite);
    }

    #[test]
    fn extract_title_works() {
        assert_eq!(
            extract_title("Truth: Governed vendor selection\n\nScenario: test"),
            Some("Governed vendor selection".into())
        );
        assert_eq!(extract_title("no heading here"), None);
    }
}
