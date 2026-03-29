// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Custom Agent — implement the Agent trait from scratch.
//!
//! Shows: Agent trait, accepts/execute contract, AgentEffect, ProposedFact.

use converge_core::{Context, ContextKey, Engine};
use converge_traits::{Agent, AgentEffect, Fact, ProposedFact};

/// A custom agent that reads Seeds and emits a summary as a Hypothesis.
struct SummaryAgent {
    name: String,
}

impl SummaryAgent {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Agent for SummaryAgent {
    fn name(&self) -> &str {
        &self.name
    }

    /// Only fire when Seeds exist but no Hypotheses yet.
    fn accepts(&self, ctx: &Context) -> bool {
        !ctx.get(ContextKey::Seeds).is_empty() && ctx.get(ContextKey::Hypotheses).is_empty()
    }

    /// Read all seeds and produce a summary hypothesis.
    fn execute(&self, ctx: &Context) -> AgentEffect {
        let seeds = ctx.get(ContextKey::Seeds);
        let summary = seeds
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("; ");

        AgentEffect {
            facts: vec![ProposedFact {
                key: ContextKey::Hypotheses,
                id: format!("{}-summary", self.name),
                content: format!("Combined signal: {summary}"),
            }],
        }
    }
}

fn main() {
    println!("=== Custom Agent Example ===\n");

    let mut engine = Engine::new();

    // Seed agents provide initial facts
    engine.register(converge_core::agents::SeedAgent::new(
        "data-a",
        "Revenue up 12%",
    ));
    engine.register(converge_core::agents::SeedAgent::new(
        "data-b",
        "Churn down to 3.5%",
    ));

    // Our custom agent reacts to those seeds
    engine.register(SummaryAgent::new("summarizer"));

    let result = engine.run(Context::new()).expect("should converge");

    println!("Converged in {} cycles\n", result.cycles);

    for fact in result.context.get(ContextKey::Hypotheses) {
        println!("Hypothesis: {}", fact.content);
    }

    println!("\n=== Done ===");
}
