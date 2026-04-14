// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Hello Convergence — minimal example of the convergence engine.
//!
//! Shows: Engine, agents, context, facts, and the convergence loop.

use converge_core::suggestors::{ReactOnceSuggestor, SeedSuggestor};
use converge_core::{Context, ContextKey, Engine};

#[tokio::main]
async fn main() {
    println!("=== Hello Convergence ===\n");

    // 1. Create an engine
    let mut engine = Engine::new();

    // 2. Register agents
    //    SeedSuggestor:      writes a fact once, then goes idle
    //    ReactOnceSuggestor: waits for Seeds, then writes Hypotheses once
    engine.register_suggestor(SeedSuggestor::new(
        "seed-1",
        "Monthly active users grew 15%",
    ));
    engine.register_suggestor(ReactOnceSuggestor::new(
        "hypothesis-1",
        "Growth driven by new onboarding flow",
    ));

    // 3. Run until convergence (fixed point)
    let result = engine.run(Context::new()).await.expect("should converge");

    // 4. Inspect the outcome
    println!("Converged: {}", result.converged);
    println!("Cycles:    {}", result.cycles);
    println!("Stop:      {:?}\n", result.stop_reason);

    println!("Seeds:");
    for fact in result.context.get(ContextKey::Seeds) {
        println!("  [{:?}] {}: {}", fact.key(), fact.id, fact.content);
    }

    println!("\nHypotheses:");
    for fact in result.context.get(ContextKey::Hypotheses) {
        println!("  [{:?}] {}: {}", fact.key(), fact.id, fact.content);
    }

    println!("\n=== Done ===");
}
