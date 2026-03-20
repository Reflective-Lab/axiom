// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Form Filler pack agents (PDF-first).
//!
//! This module produces a governed, reviewable fill plan and proposed field values.
//! It is intentionally minimal and deterministic to keep invariants in focus.

use converge_core::{Agent, AgentEffect, Context, ContextKey, Fact, ProposedFact};
use serde::{Deserialize, Serialize};

const FORM_REQUEST_SEED_ID: &str = "form_filler:request";
const SCHEMA_FACT_ID: &str = "form_filler:schema";
const MAPPINGS_FACT_ID: &str = "form_filler:field_mappings";
const NORMALIZED_FACT_ID: &str = "form_filler:normalized_fields";
const COMPLETENESS_FACT_ID: &str = "form_filler:completeness";
const RISK_FACT_ID: &str = "form_filler:risk_classification";
const FILL_PLAN_FACT_ID: &str = "form_filler:fill_plan";
const PROPOSAL_PREFIX: &str = "form_filler:proposed_field:";

fn has_fact(ctx: &dyn converge_core::ContextView, key: ContextKey, id: &str) -> bool {
    ctx.get(key).iter().any(|fact| fact.id == id)
}

fn parse_form_request(ctx: &dyn converge_core::ContextView) -> Option<FormRequestSeed> {
    ctx.get(ContextKey::Seeds)
        .iter()
        .find(|seed| seed.id == FORM_REQUEST_SEED_ID)
        .and_then(|seed| serde_json::from_str::<FormRequestSeed>(&seed.content).ok())
}

#[derive(Debug, Clone, Deserialize)]
struct FormRequestSeed {
    #[serde(default)]
    form_id: Option<String>,
    #[serde(default)]
    fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldMapping {
    field_id: String,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NormalizedField {
    field_id: String,
    normalized_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompletenessStatus {
    missing_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RiskClassification {
    high_risk_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct FillPlan {
    form_id: String,
    missing_fields: Vec<String>,
    high_risk_fields: Vec<String>,
    ready_for_submit: bool,
}

fn classify_risk(field_id: &str) -> bool {
    let lower = field_id.to_lowercase();
    ["ssn", "bank", "account", "passport", "tax", "salary"]
        .iter()
        .any(|keyword| lower.contains(keyword))
}

/// Extracts a schema from the seed request (PDF-first entry).
pub struct FormSchemaAgent;

impl Agent for FormSchemaAgent {
    fn name(&self) -> &str {
        "FormSchemaAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Seeds) && !has_fact(ctx, ContextKey::Signals, SCHEMA_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let request = match parse_form_request(ctx) {
            Some(request) => request,
            None => return AgentEffect::empty(),
        };

        let payload = serde_json::json!({
            "form_id": request.form_id.unwrap_or_else(|| "unknown".to_string()),
            "fields": request.fields,
        });

        AgentEffect::with_fact(Fact {
            key: ContextKey::Signals,
            id: SCHEMA_FACT_ID.to_string(),
            content: payload.to_string(),
        })
    }
}

/// Maps schema fields to candidate sources (deterministic placeholder).
pub struct FieldMappingAgent;

impl Agent for FieldMappingAgent {
    fn name(&self) -> &str {
        "FieldMappingAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !has_fact(ctx, ContextKey::Hypotheses, MAPPINGS_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let schema = ctx
            .get(ContextKey::Signals)
            .iter()
            .find(|fact| fact.id == SCHEMA_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());

        let fields = schema
            .and_then(|value| value.get("fields").cloned())
            .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
            .unwrap_or_default();

        let mappings: Vec<FieldMapping> = fields
            .iter()
            .map(|field_id| FieldMapping {
                field_id: field_id.to_string(),
                source: "unknown".to_string(),
            })
            .collect();

        let payload = serde_json::json!({ "mappings": mappings });
        AgentEffect::with_fact(Fact {
            key: ContextKey::Hypotheses,
            id: MAPPINGS_FACT_ID.to_string(),
            content: payload.to_string(),
        })
    }
}

/// Normalizes candidate values (placeholder deterministic normalization).
pub struct NormalizationAgent;

impl Agent for NormalizationAgent {
    fn name(&self) -> &str {
        "NormalizationAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Hypotheses)
            && !has_fact(ctx, ContextKey::Hypotheses, NORMALIZED_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let mappings = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|fact| fact.id == MAPPINGS_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok())
            .and_then(|value| value.get("mappings").cloned())
            .and_then(|value| serde_json::from_value::<Vec<FieldMapping>>(value).ok())
            .unwrap_or_default();

        let normalized: Vec<NormalizedField> = mappings
            .into_iter()
            .map(|mapping| NormalizedField {
                field_id: mapping.field_id,
                normalized_value: String::new(),
            })
            .collect();

        let payload = serde_json::json!({ "normalized": normalized });
        AgentEffect::with_fact(Fact {
            key: ContextKey::Hypotheses,
            id: NORMALIZED_FACT_ID.to_string(),
            content: payload.to_string(),
        })
    }
}

/// Detects missing required fields (based on empty normalized values).
pub struct CompletenessAgent;

impl Agent for CompletenessAgent {
    fn name(&self) -> &str {
        "CompletenessAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Hypotheses)
            && !has_fact(ctx, ContextKey::Constraints, COMPLETENESS_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let normalized = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|fact| fact.id == NORMALIZED_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok())
            .and_then(|value| value.get("normalized").cloned())
            .and_then(|value| serde_json::from_value::<Vec<NormalizedField>>(value).ok())
            .unwrap_or_default();

        let missing_fields: Vec<String> = normalized
            .iter()
            .filter(|field| field.normalized_value.trim().is_empty())
            .map(|field| field.field_id.clone())
            .collect();

        let payload = CompletenessStatus { missing_fields };
        AgentEffect::with_fact(Fact {
            key: ContextKey::Constraints,
            id: COMPLETENESS_FACT_ID.to_string(),
            content: serde_json::to_string(&payload).unwrap_or_default(),
        })
    }
}

/// Classifies high-risk fields that require approval.
pub struct RiskClassifierAgent;

impl Agent for RiskClassifierAgent {
    fn name(&self) -> &str {
        "RiskClassifierAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals) && !has_fact(ctx, ContextKey::Constraints, RISK_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let schema = ctx
            .get(ContextKey::Signals)
            .iter()
            .find(|fact| fact.id == SCHEMA_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());

        let fields = schema
            .and_then(|value| value.get("fields").cloned())
            .and_then(|value| serde_json::from_value::<Vec<String>>(value).ok())
            .unwrap_or_default();

        let high_risk_fields = fields
            .into_iter()
            .filter(|field| classify_risk(field))
            .collect::<Vec<_>>();

        let payload = RiskClassification { high_risk_fields };
        AgentEffect::with_fact(Fact {
            key: ContextKey::Constraints,
            id: RISK_FACT_ID.to_string(),
            content: serde_json::to_string(&payload).unwrap_or_default(),
        })
    }
}

/// Produces a consolidated fill plan.
pub struct FillPlanAgent;

impl Agent for FillPlanAgent {
    fn name(&self) -> &str {
        "FillPlanAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals, ContextKey::Constraints]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Signals)
            && ctx.has(ContextKey::Constraints)
            && !has_fact(ctx, ContextKey::Strategies, FILL_PLAN_FACT_ID)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let schema = ctx
            .get(ContextKey::Signals)
            .iter()
            .find(|fact| fact.id == SCHEMA_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok());

        let form_id = schema
            .and_then(|value| {
                value
                    .get("form_id")
                    .and_then(|id| id.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        let missing_fields = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id == COMPLETENESS_FACT_ID)
            .and_then(|fact| serde_json::from_str::<CompletenessStatus>(&fact.content).ok())
            .map(|status| status.missing_fields)
            .unwrap_or_default();

        let high_risk_fields = ctx
            .get(ContextKey::Constraints)
            .iter()
            .find(|fact| fact.id == RISK_FACT_ID)
            .and_then(|fact| serde_json::from_str::<RiskClassification>(&fact.content).ok())
            .map(|status| status.high_risk_fields)
            .unwrap_or_default();

        let ready_for_submit = missing_fields.is_empty() && high_risk_fields.is_empty();
        let plan = FillPlan {
            form_id,
            missing_fields,
            high_risk_fields,
            ready_for_submit,
        };

        AgentEffect::with_fact(Fact {
            key: ContextKey::Strategies,
            id: FILL_PLAN_FACT_ID.to_string(),
            content: serde_json::to_string(&plan).unwrap_or_default(),
        })
    }
}

/// Emits proposed field values (for approval and promotion).
pub struct ProposalEmitterAgent;

impl Agent for ProposalEmitterAgent {
    fn name(&self) -> &str {
        "ProposalEmitterAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses, ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Hypotheses) && ctx.has(ContextKey::Signals)
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let normalized = ctx
            .get(ContextKey::Hypotheses)
            .iter()
            .find(|fact| fact.id == NORMALIZED_FACT_ID)
            .and_then(|fact| serde_json::from_str::<serde_json::Value>(&fact.content).ok())
            .and_then(|value| value.get("normalized").cloned())
            .and_then(|value| serde_json::from_value::<Vec<NormalizedField>>(value).ok())
            .unwrap_or_default();

        let proposals: Vec<ProposedFact> = normalized
            .into_iter()
            .filter(|field| !field.normalized_value.trim().is_empty())
            .map(|field| ProposedFact {
                key: ContextKey::Proposals,
                id: format!("{}{}", PROPOSAL_PREFIX, field.field_id),
                content: serde_json::json!({
                    "field_id": field.field_id,
                    "value": field.normalized_value,
                    "provenance": "form_filler:deterministic",
                    "risk": "unknown",
                })
                .to_string(),
                confidence: 0.8,
                provenance: "form_filler:deterministic".to_string(),
            })
            .collect();

        let mut effect = AgentEffect::empty();
        effect.proposals = proposals;
        effect
    }
}
