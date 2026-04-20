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
    use proptest::prelude::*;

    // ─── extract_title ───

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

    #[test]
    fn extract_title_feature_prefix() {
        assert_eq!(
            extract_title("Feature: My cool feature\n"),
            Some("My cool feature".into())
        );
    }

    #[test]
    fn extract_title_skips_empty_value() {
        assert_eq!(
            extract_title("Truth:   \nFeature: Real title"),
            Some("Real title".into())
        );
    }

    #[test]
    fn extract_title_with_leading_whitespace() {
        assert_eq!(
            extract_title("  Truth: Indented heading"),
            Some("Indented heading".into())
        );
    }

    #[test]
    fn extract_title_empty_spec() {
        assert_eq!(extract_title(""), None);
    }

    #[test]
    fn extract_title_special_characters() {
        assert_eq!(
            extract_title("Truth: Héllo wörld — «special» ñ"),
            Some("Héllo wörld — «special» ñ".into())
        );
    }

    // ─── GuidanceConfig ───

    #[test]
    fn guidance_config_default() {
        let cfg = GuidanceConfig::default();
        assert!(cfg.provider_override.is_none());
        assert!(cfg.model_override.is_none());
    }

    #[test]
    fn guidance_config_with_overrides() {
        let cfg = GuidanceConfig {
            provider_override: Some("openai".into()),
            model_override: Some("gpt-4o".into()),
        };
        assert_eq!(cfg.provider_override.as_deref(), Some("openai"));
        assert_eq!(cfg.model_override.as_deref(), Some("gpt-4o"));
    }

    // ─── GuidanceResponse ───

    #[test]
    fn guidance_response_serializes_to_camel_case() {
        let resp = GuidanceResponse {
            current_title: "t".into(),
            suggested_title: "s".into(),
            should_rewrite: true,
            source: "local-heuristic".into(),
            source_label: "Local".into(),
            provider: None,
            model: None,
            rationale: vec!["r".into()],
            description_hints: vec![],
            note: "n".into(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("shouldRewrite").is_some());
        assert!(json.get("currentTitle").is_some());
        assert!(json.get("suggestedTitle").is_some());
        assert!(json.get("sourceLabel").is_some());
        assert!(json.get("descriptionHints").is_some());
    }

    #[test]
    fn guidance_response_equality() {
        let a = GuidanceResponse {
            current_title: "x".into(),
            suggested_title: "y".into(),
            should_rewrite: false,
            source: "s".into(),
            source_label: "l".into(),
            provider: None,
            model: None,
            rationale: vec![],
            description_hints: vec![],
            note: String::new(),
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ─── DraftContext ───

    #[test]
    fn draft_context_from_minimal_spec() {
        let ctx = draft_context("Truth: Simple\n\nScenario: test\n  Given x", "Simple");
        assert_eq!(ctx.title, "Simple");
        assert_eq!(ctx.scenario_count, 1);
        assert!(!ctx.has_intent);
        assert!(!ctx.has_authority);
        assert!(!ctx.has_constraint);
        assert!(!ctx.has_evidence);
        assert!(!ctx.has_exception);
    }

    #[test]
    fn draft_context_with_governance_blocks() {
        let spec = "\
Truth: Governed truth

Intent:
  Outcome: clarity

Authority:
  Actor: board

Constraint:
  Cost Limit: budget

Evidence:
  Requires: audit_trail

Exception:
  Override: emergency

Scenario: First
  Given x
Scenario: Second
  Given y
";
        let ctx = draft_context(spec, "Governed truth");
        assert_eq!(ctx.title, "Governed truth");
        assert!(ctx.has_intent);
        assert!(ctx.has_authority);
        assert!(ctx.has_constraint);
        assert!(ctx.has_evidence);
        assert!(ctx.has_exception);
        assert_eq!(ctx.scenario_count, 2);
    }

    #[test]
    fn draft_context_empty_spec() {
        let ctx = draft_context("", "");
        assert_eq!(ctx.title, "");
        assert_eq!(ctx.scenario_count, 0);
        assert_eq!(ctx.description_line_count, 0);
    }

    #[test]
    fn draft_context_trims_title() {
        let ctx = draft_context("Truth:  padded  \n", "  padded  ");
        assert_eq!(ctx.title, "padded");
    }

    #[test]
    fn draft_context_description_lines() {
        let spec = "\
Truth: Has description
  This is line one
  This is line two

Scenario: test
  Given x
";
        let ctx = draft_context(spec, "Has description");
        assert_eq!(ctx.description_line_count, 2);
    }

    // ─── sanitize_suggested_title ───

    #[test]
    fn sanitize_strips_truth_prefix() {
        assert_eq!(
            sanitize_suggested_title("Truth: Good title", "fallback"),
            "Good title"
        );
    }

    #[test]
    fn sanitize_strips_feature_prefix() {
        assert_eq!(
            sanitize_suggested_title("Feature: Good title", "fallback"),
            "Good title"
        );
    }

    #[test]
    fn sanitize_empty_falls_back() {
        assert_eq!(sanitize_suggested_title("", "fallback"), "fallback");
        assert_eq!(sanitize_suggested_title("   ", "fallback"), "fallback");
    }

    #[test]
    fn sanitize_preserves_clean_title() {
        assert_eq!(
            sanitize_suggested_title("Already clean", "x"),
            "Already clean"
        );
    }

    #[test]
    fn sanitize_truth_prefix_only() {
        assert_eq!(sanitize_suggested_title("Truth:", "fallback"), "fallback");
    }

    // ─── parse_llm_guidance ───

    #[test]
    fn parse_valid_llm_json() {
        let json = r#"{"shouldRewrite":true,"suggestedTitle":"Better","rationale":["r1"],"descriptionHints":["h1"]}"#;
        let g = parse_llm_guidance(json).unwrap();
        assert!(g.should_rewrite);
        assert_eq!(g.suggested_title, "Better");
        assert_eq!(g.rationale, vec!["r1"]);
        assert_eq!(g.description_hints, vec!["h1"]);
    }

    #[test]
    fn parse_llm_guidance_extracts_json_from_surrounding_text() {
        let raw =
            "Here is the result:\n{\"shouldRewrite\":false,\"suggestedTitle\":\"Same\"}\nDone.";
        let g = parse_llm_guidance(raw).unwrap();
        assert!(!g.should_rewrite);
        assert_eq!(g.suggested_title, "Same");
    }

    #[test]
    fn parse_llm_guidance_defaults_on_missing_fields() {
        let json = "{}";
        let g = parse_llm_guidance(json).unwrap();
        assert!(!g.should_rewrite);
        assert!(g.suggested_title.is_empty());
        assert!(g.rationale.is_empty());
        assert!(g.description_hints.is_empty());
    }

    #[test]
    fn parse_llm_guidance_rejects_garbage() {
        assert!(parse_llm_guidance("not json at all").is_err());
    }

    #[test]
    fn parse_llm_guidance_empty_string() {
        assert!(parse_llm_guidance("").is_err());
    }

    // ─── normalize_rationale ───

    #[test]
    fn normalize_rationale_filters_empty() {
        let result = normalize_rationale(vec![String::new(), "  ".into()], "fallback".into());
        assert_eq!(result, vec!["fallback"]);
    }

    #[test]
    fn normalize_rationale_keeps_non_empty() {
        let result = normalize_rationale(vec!["good reason".into()], "fallback".into());
        assert_eq!(result, vec!["good reason"]);
    }

    // ─── normalize_description_hints ───

    #[test]
    fn normalize_hints_truncates_to_two() {
        let result = normalize_description_hints(vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn normalize_hints_removes_blank() {
        let result = normalize_description_hints(vec![String::new(), "  ".into(), "real".into()]);
        assert_eq!(result, vec!["real"]);
    }

    // ─── provider_label ───

    #[test]
    fn known_provider_labels() {
        assert_eq!(provider_label("openrouter"), "OpenRouter");
        assert_eq!(provider_label("openai"), "OpenAI");
        assert_eq!(provider_label("anthropic"), "Anthropic");
        assert_eq!(provider_label("gemini"), "Gemini");
        assert_eq!(provider_label("mistral"), "Mistral");
    }

    #[test]
    fn unknown_provider_label_fallback() {
        assert_eq!(provider_label("deepseek"), "Live");
        assert_eq!(provider_label(""), "Live");
    }

    // ─── title_is_declarative ───

    #[test]
    fn declarative_verbs_detected() {
        assert!(title_is_declarative("selection is auditable"));
        assert!(title_is_declarative("policy must be enforced"));
        assert!(title_is_declarative("access requires approval"));
        assert!(title_is_declarative("state remains consistent"));
        assert!(title_is_declarative("pipeline produces artifacts"));
        assert!(title_is_declarative("gate blocks unapproved changes"));
        assert!(title_is_declarative("policy allows exceptions"));
    }

    #[test]
    fn non_declarative_titles() {
        assert!(!title_is_declarative("vendor selection"));
        assert!(!title_is_declarative("evaluation workflow"));
        assert!(!title_is_declarative(""));
    }

    // ─── join_phrases ───

    #[test]
    fn join_empty() {
        assert_eq!(join_phrases(&[]), "");
    }

    #[test]
    fn join_single() {
        assert_eq!(join_phrases(&["auditable"]), "auditable");
    }

    #[test]
    fn join_two() {
        assert_eq!(
            join_phrases(&["auditable", "constrained"]),
            "auditable and constrained"
        );
    }

    #[test]
    fn join_three() {
        assert_eq!(
            join_phrases(&["auditable", "constrained", "approval-gated"]),
            "auditable, constrained, and approval-gated"
        );
    }

    #[test]
    fn join_four() {
        assert_eq!(join_phrases(&["a", "b", "c", "d"]), "a, b, c, and d");
    }

    // ─── quality_predicate ───

    #[test]
    fn predicate_all_false() {
        assert_eq!(
            quality_predicate(false, false, false),
            "is explicit and reviewable"
        );
    }

    #[test]
    fn predicate_evidence_only() {
        assert_eq!(quality_predicate(false, false, true), "is auditable");
    }

    #[test]
    fn predicate_all_true() {
        assert_eq!(
            quality_predicate(true, true, true),
            "is auditable, constrained, and approval-gated"
        );
    }

    #[test]
    fn predicate_authority_and_constraint() {
        assert_eq!(
            quality_predicate(true, true, false),
            "is constrained and approval-gated"
        );
    }

    // ─── normalize_subject ───

    #[test]
    fn subject_reorders_for_pattern() {
        assert_eq!(
            normalize_subject("Selection for enterprise AI"),
            "Enterprise AI selection"
        );
    }

    #[test]
    fn subject_strips_suffix() {
        assert_eq!(
            normalize_subject("Evaluation for vendor rollout"),
            "Vendor evaluation"
        );
    }

    #[test]
    fn subject_non_reorderable() {
        assert_eq!(normalize_subject("Budget controls"), "Budget controls");
    }

    #[test]
    fn subject_trailing_dot() {
        assert_eq!(normalize_subject("Controls."), "Controls");
    }

    #[test]
    fn subject_empty() {
        assert_eq!(normalize_subject(""), "");
    }

    // ─── uppercase_first ───

    #[test]
    fn uppercase_first_normal() {
        assert_eq!(uppercase_first("hello"), "Hello");
    }

    #[test]
    fn uppercase_first_empty() {
        assert_eq!(uppercase_first(""), "");
    }

    #[test]
    fn uppercase_first_already_upper() {
        assert_eq!(uppercase_first("Already"), "Already");
    }

    // ─── description_line_count ───

    #[test]
    fn description_count_no_heading() {
        assert_eq!(description_line_count("just text\nno heading"), 0);
    }

    #[test]
    fn description_count_with_blank_lines_before_content() {
        let spec = "Truth: Title\n\n  Description line\n\nScenario: test";
        assert_eq!(description_line_count(spec), 1);
    }

    #[test]
    fn description_count_stops_at_governance() {
        let spec = "Truth: Title\n  desc 1\n  desc 2\nIntent:\n  Outcome: x";
        assert_eq!(description_line_count(spec), 2);
    }

    #[test]
    fn description_count_stops_at_tag() {
        let spec = "Truth: Title\n  desc\n@slow\nScenario: x";
        assert_eq!(description_line_count(spec), 1);
    }

    // ─── is_heading_boundary ───

    #[test]
    fn heading_boundaries() {
        assert!(is_heading_boundary("Intent:"));
        assert!(is_heading_boundary("Authority:"));
        assert!(is_heading_boundary("Constraint:"));
        assert!(is_heading_boundary("Evidence:"));
        assert!(is_heading_boundary("Exception:"));
        assert!(is_heading_boundary("@tag"));
        assert!(is_heading_boundary("Background: setup"));
        assert!(is_heading_boundary("Scenario: test"));
        assert!(is_heading_boundary("Rule: something"));
        assert!(is_heading_boundary("Example: one"));
        assert!(is_heading_boundary("Examples: table"));
    }

    #[test]
    fn non_boundaries() {
        assert!(!is_heading_boundary("just text"));
        assert!(!is_heading_boundary(""));
        assert!(!is_heading_boundary("  Intent:"));
    }

    // ─── truncated_excerpt ───

    #[test]
    fn excerpt_short_spec_unchanged() {
        let spec = "Truth: Short\nScenario: x";
        assert_eq!(truncated_excerpt(spec), spec);
    }

    #[test]
    fn excerpt_truncates_many_lines() {
        let lines: Vec<String> = (0..50).map(|i| format!("Line {i}")).collect();
        let spec = lines.join("\n");
        let excerpt = truncated_excerpt(&spec);
        assert_eq!(excerpt.lines().count(), 20);
    }

    // ─── local_heading_guidance branches ───

    #[test]
    fn local_guidance_with_workflow_in_title() {
        let spec = "Truth: Onboarding workflow\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Onboarding workflow", "note".into());
        assert!(g.should_rewrite);
        assert!(g.rationale.iter().any(|r| r.contains("claim or rule")));
    }

    #[test]
    fn local_guidance_with_process_in_title() {
        let spec = "Truth: Review process\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Review process", "note".into());
        assert!(g.should_rewrite);
    }

    #[test]
    fn local_guidance_with_for_pattern() {
        let spec = "Truth: Screening for vendors\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Screening for vendors", "note".into());
        assert!(g.should_rewrite);
        assert!(g.rationale.iter().any(|r| r.contains("initiative")));
    }

    #[test]
    fn local_guidance_source_fields() {
        let spec = "Truth: Topic title\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Topic title", "my note".into());
        assert_eq!(g.source, "local-heuristic");
        assert_eq!(g.source_label, "Local");
        assert!(g.provider.is_none());
        assert!(g.model.is_none());
        assert_eq!(g.note, "my note");
    }

    #[test]
    fn local_guidance_with_approval_in_spec() {
        let spec = "Truth: Budget is constrained\n\nApproval required before proceeding.\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Budget is constrained", "note".into());
        assert!(!g.should_rewrite);
    }

    #[test]
    fn local_guidance_with_traceable_in_spec() {
        let spec =
            "Truth: Selection topic\n\nAll steps must be traceable.\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Selection topic", "note".into());
        assert!(g.should_rewrite);
        let title_lower = g.suggested_title.to_ascii_lowercase();
        assert!(title_lower.contains("auditable"));
    }

    #[test]
    fn local_guidance_with_governance_in_spec() {
        let spec =
            "Truth: Policy topic\n\nGovernance framework applies.\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Policy topic", "note".into());
        assert!(g.should_rewrite);
        let title_lower = g.suggested_title.to_ascii_lowercase();
        assert!(title_lower.contains("constrained"));
    }

    #[test]
    fn local_guidance_missing_evidence_adds_rationale() {
        let spec = "Truth: Vendor selection topic\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Vendor selection topic", "note".into());
        assert!(g.rationale.iter().any(|r| r.contains("evidence")));
    }

    #[test]
    fn local_guidance_missing_constraint_adds_rationale() {
        let spec = "Truth: Vendor selection topic\n\nScenario: test\n  Given x";
        let g = local_heading_guidance(spec, "Vendor selection topic", "note".into());
        assert!(g.rationale.iter().any(|r| r.contains("constraints")));
    }

    // ─── build_description_hints ───

    #[test]
    fn hints_for_no_description() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 0,
            scenario_count: 0,
            has_intent: false,
            has_authority: false,
            has_constraint: false,
            has_evidence: false,
            has_exception: false,
        };
        let hints = build_description_hints(&ctx, "Truth: T");
        assert!(hints.iter().any(|h| h.contains("reproducible")));
    }

    #[test]
    fn hints_for_authority() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 1,
            scenario_count: 0,
            has_intent: false,
            has_authority: true,
            has_constraint: false,
            has_evidence: true,
            has_exception: false,
        };
        let hints = build_description_hints(&ctx, "Truth: T");
        assert!(hints.iter().any(|h| h.contains("approval")));
    }

    #[test]
    fn hints_no_constraint_no_authority() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 1,
            scenario_count: 0,
            has_intent: false,
            has_authority: false,
            has_constraint: false,
            has_evidence: true,
            has_exception: false,
        };
        let hints = build_description_hints(&ctx, "Truth: T");
        assert!(hints.iter().any(|h| h.contains("policy")));
    }

    #[test]
    fn hints_no_evidence_adds_traceable() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 1,
            scenario_count: 0,
            has_intent: false,
            has_authority: false,
            has_constraint: true,
            has_evidence: false,
            has_exception: false,
        };
        let hints = build_description_hints(&ctx, "Truth: T");
        assert!(hints.iter().any(|h| h.contains("traceable")));
    }

    #[test]
    fn hints_max_two() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 0,
            scenario_count: 0,
            has_intent: false,
            has_authority: true,
            has_constraint: false,
            has_evidence: false,
            has_exception: false,
        };
        let hints = build_description_hints(&ctx, "Truth: T");
        assert!(hints.len() <= 2);
    }

    // ─── guide_heading (async, no backend) ───

    #[tokio::test]
    async fn guide_heading_returns_none_for_no_title() {
        let config = GuidanceConfig::default();
        let result = guide_heading("no heading here", &config).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn guide_heading_returns_none_for_empty_spec() {
        let config = GuidanceConfig::default();
        let result = guide_heading("", &config).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn guide_heading_falls_back_to_local_on_no_backend() {
        let config = GuidanceConfig::default();
        let spec = "Truth: Vendor selection for AI rollout\n\nScenario: test\n  Given x";
        let result = guide_heading(spec, &config).await;
        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.source, "local-heuristic");
        assert!(resp.note.contains("Live guidance failed"));
    }

    // ─── build_prompt ───

    #[test]
    fn build_prompt_includes_title_and_context() {
        let ctx = draft_context("Truth: Title\nScenario: test\n  Given x", "Title");
        let prompt =
            build_prompt("Title", &ctx, "Truth: Title\nScenario: test\n  Given x").unwrap();
        assert!(prompt.contains("Title"));
        assert!(prompt.contains("shouldRewrite"));
        assert!(prompt.contains("suggestedTitle"));
    }

    // ─── DraftContext serialization ───

    #[test]
    fn draft_context_serializes_to_camel_case() {
        let ctx = DraftContext {
            title: "T".into(),
            description_line_count: 0,
            scenario_count: 1,
            has_intent: true,
            has_authority: false,
            has_constraint: false,
            has_evidence: false,
            has_exception: false,
        };
        let json = serde_json::to_value(&ctx).unwrap();
        assert!(json.get("descriptionLineCount").is_some());
        assert!(json.get("scenarioCount").is_some());
        assert!(json.get("hasIntent").is_some());
    }

    // ─── reorderable_subject ───

    #[test]
    fn reorderable_subjects() {
        assert!(reorderable_subject("Selection"));
        assert!(reorderable_subject("Vendor Evaluation"));
        assert!(reorderable_subject("Final Approval"));
        assert!(reorderable_subject("Peer Review"));
        assert!(reorderable_subject("Initial Screening"));
        assert!(reorderable_subject("Feature Comparison"));
    }

    #[test]
    fn non_reorderable_subjects() {
        assert!(!reorderable_subject("Budget"));
        assert!(!reorderable_subject("Controls"));
        assert!(!reorderable_subject(""));
    }

    // ─── strip_context_suffix ───

    #[test]
    fn strip_known_suffixes() {
        assert_eq!(
            strip_context_suffix("enterprise AI rollout"),
            "enterprise AI"
        );
        assert_eq!(strip_context_suffix("onboarding workflow"), "onboarding");
        assert_eq!(strip_context_suffix("review process"), "review");
        assert_eq!(strip_context_suffix("compliance program"), "compliance");
    }

    #[test]
    fn strip_no_suffix() {
        assert_eq!(strip_context_suffix("plain text"), "plain text");
    }

    // ─── Property tests ───

    proptest! {
        #[test]
        fn extract_title_never_panics(s in "\\PC*") {
            let _ = extract_title(&s);
        }

        #[test]
        fn draft_context_never_panics(s in "\\PC*") {
            let _ = draft_context(&s, "any title");
        }

        #[test]
        fn local_heading_guidance_never_panics(s in "\\PC{0,500}") {
            let title = extract_title(&s).unwrap_or_default();
            let _ = local_heading_guidance(&s, &title, "test".into());
        }

        #[test]
        fn sanitize_never_panics(s in "\\PC*") {
            let result = sanitize_suggested_title(&s, "fallback");
            assert!(!result.is_empty());
        }

        #[test]
        fn parse_llm_guidance_never_panics(s in "\\PC*") {
            let _ = parse_llm_guidance(&s);
        }

        #[test]
        fn any_valid_spec_produces_response(
            title in "[A-Za-z ]{3,30}",
            scenario in "[A-Za-z ]{3,30}"
        ) {
            let spec = format!("Truth: {title}\n\nScenario: {scenario}\n  Given x");
            let g = local_heading_guidance(&spec, &title, "test".into());
            assert_eq!(g.current_title, title.trim());
            assert!(!g.source.is_empty());
        }

        #[test]
        fn truncated_excerpt_bounded(s in "\\PC{0,5000}") {
            let excerpt = truncated_excerpt(&s);
            assert!(excerpt.lines().count() <= 21);
        }

        #[test]
        fn description_line_count_never_panics(s in "\\PC*") {
            let _ = description_line_count(&s);
        }
    }
}
