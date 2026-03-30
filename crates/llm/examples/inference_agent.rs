// Copyright (c) 2026 Aprio One AB
// Author: Kenneth Pernyer, kenneth@pernyer.se

//! Example: LLM Agent Integration
//!
//! Demonstrates how to use the LlmAgent within a Converge context.
//! This example shows the agent API without requiring actual model weights.

use converge_core::{Agent, Context, ContextKey, Fact};
use converge_llm::{GenerationParams, LlmAgent, LlmConfig, PromptTemplate};

fn main() {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Converge LLM Agent Example ===\n");

    // Create an LLM agent with small config (for demo)
    let config = LlmConfig::small();
    println!("Model config:");
    println!("  - Model: {}", config.model_id);
    println!("  - Context: {} tokens", config.max_context_length);
    println!("  - Precision: {:?}", config.precision);
    println!("  - Est. memory: {:.1} GB\n", config.estimated_memory_gb());

    // Configure the agent
    let agent = LlmAgent::new("reasoning-agent", config)
        .with_params(GenerationParams::agent())
        .with_template(PromptTemplate::reasoning())
        .with_output_key(ContextKey::Hypotheses);

    println!("Agent: {}", agent.name());
    println!("Dependencies: {:?}", agent.dependencies());
    println!();

    // Create a context with some seed facts
    let mut ctx = Context::new();

    // Add seed facts (simulating data from analytics pipeline)
    let _ = ctx.add_fact(Fact {
        key: ContextKey::Seeds,
        id: "metric-1".to_string(),
        content: "Monthly active users increased by 15% in December".to_string(),
    });

    let _ = ctx.add_fact(Fact {
        key: ContextKey::Seeds,
        id: "metric-2".to_string(),
        content: "Customer churn rate decreased from 5% to 3.5%".to_string(),
    });

    let _ = ctx.add_fact(Fact {
        key: ContextKey::Signals,
        id: "signal-1".to_string(),
        content: "New onboarding flow launched November 15th".to_string(),
    });

    println!(
        "Context prepared with {} seeds and {} signals\n",
        ctx.get(ContextKey::Seeds).len(),
        ctx.get(ContextKey::Signals).len()
    );

    // Check if agent accepts this context
    println!("Agent accepts context: {}", agent.accepts(&ctx));

    // Execute the agent (this will fail gracefully without loaded model)
    println!("\nExecuting agent...");
    let effect = agent.execute(&ctx);

    // Examine the effect
    println!("\nAgent produced {} fact(s):", effect.facts.len());
    for fact in &effect.facts {
        println!(
            "  [{:?}] {}: {}",
            fact.key,
            fact.id,
            if fact.content.len() > 80 {
                format!("{}...", &fact.content[..80])
            } else {
                fact.content.clone()
            }
        );
    }

    // Demonstrate that agent won't run twice (already has output)
    let _ = ctx.add_fact(effect.facts.into_iter().next().unwrap());
    println!("\nAfter adding output to context:");
    println!("Agent accepts context: {}", agent.accepts(&ctx));

    println!("\n=== Example Complete ===");
    println!("\nNote: To run with actual inference, load model weights first.");
    println!("See README.md for instructions on downloading pretrained models.");
}
