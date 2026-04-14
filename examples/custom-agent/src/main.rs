// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Custom Suggestor — implement the Suggestor trait from scratch.
//!
//! Shows: Suggestor trait, accepts/execute contract, AgentEffect, ProposedFact.

use converge_core::{AgentEffect, Context, ContextKey, Engine, ProposedFact, Suggestor};

/// A custom agent that reads Seeds and emits a summary as a Hypothesis.
struct SummaryAgent {
    agent_name: String,
}

impl SummaryAgent {
    fn new(name: &str) -> Self {
        Self {
            agent_name: name.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Suggestor for SummaryAgent {
    fn name(&self) -> &str {
        &self.agent_name
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        !ctx.get(ContextKey::Seeds).is_empty() && ctx.get(ContextKey::Hypotheses).is_empty()
    }

    async fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let summary = seeds
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        AgentEffect::with_proposal(ProposedFact {
            key: ContextKey::Hypotheses,
            id: format!("{}-summary", self.agent_name),
            content: format!("Combined signal: {summary}"),
            confidence: 0.9,
            provenance: format!("agent:{}", self.agent_name),
        })
    }
}

#[tokio::main]
async fn main() {
    println!("=== Custom Suggestor Example ===\n");

    let mut engine = Engine::new();

    engine.register_suggestor(converge_core::suggestors::SeedSuggestor::new(
        "data-a",
        "Revenue up 12%",
    ));
    engine.register_suggestor(converge_core::suggestors::SeedSuggestor::new(
        "data-b",
        "Churn down to 3.5%",
    ));

    engine.register_suggestor(SummaryAgent::new("summarizer"));

    let result = engine.run(Context::new()).await.expect("should converge");

    println!("Converged in {} cycles\n", result.cycles);

    for fact in result.context.get(ContextKey::Hypotheses) {
        println!("Hypothesis: {}", fact.content);
    }

    println!("\n=== Done ===");
}
