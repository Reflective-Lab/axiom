// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Deterministic drafting flow (short path).
//!
//! Seeds -> Signals (research notes) -> Strategies (draft output)

use converge_core::{Agent, AgentEffect, Context, ContextKey, Fact};

const DRAFT_RESEARCH_PREFIX: &str = "drafting_research:";
const DRAFT_OUTPUT_PREFIX: &str = "drafting_output:";

/// Drafting research agent (deterministic fallback).
pub struct DraftingResearchAgent;

impl Agent for DraftingResearchAgent {
    fn name(&self) -> &str {
        "DraftingResearchAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.has(ContextKey::Seeds)
            && !ctx
                .get(ContextKey::Signals)
                .iter()
                .any(|fact| fact.id.starts_with(DRAFT_RESEARCH_PREFIX))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let summary = ctx
            .get(ContextKey::Seeds)
            .iter()
            .map(|seed| seed.content.clone())
            .collect::<Vec<_>>()
            .join(" | ");

        AgentEffect::with_facts(vec![Fact {
            key: ContextKey::Signals,
            id: format!("{DRAFT_RESEARCH_PREFIX}notes"),
            content: format!("Drafting research notes: {summary}"),
        }])
    }
}

/// Drafting composer agent (deterministic fallback).
pub struct DraftingComposerAgent;

impl Agent for DraftingComposerAgent {
    fn name(&self) -> &str {
        "DraftingComposerAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        ctx.get(ContextKey::Signals)
            .iter()
            .any(|fact| fact.id.starts_with(DRAFT_RESEARCH_PREFIX))
            && !ctx
                .get(ContextKey::Strategies)
                .iter()
                .any(|fact| fact.id.starts_with(DRAFT_OUTPUT_PREFIX))
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let notes = ctx
            .get(ContextKey::Signals)
            .iter()
            .filter(|fact| fact.id.starts_with(DRAFT_RESEARCH_PREFIX))
            .map(|fact| fact.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        AgentEffect::with_facts(vec![Fact {
            key: ContextKey::Strategies,
            id: format!("{DRAFT_OUTPUT_PREFIX}v0"),
            content: format!("Draft output (deterministic):\n{notes}"),
        }])
    }
}
