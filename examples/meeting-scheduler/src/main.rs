// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Meeting Scheduler — domain pack with constraint agents.
//!
//! Shows: domain agents, invariants, constraint satisfaction via convergence.

use converge_core::{Context, ContextKey, Engine, Fact};
use converge_domain::{
    AvailabilityRetrievalAgent, ConflictDetectionAgent, RequireParticipantAvailability,
    RequirePositiveDuration, RequireValidSlot, SlotOptimizationAgent, TimeZoneNormalizationAgent,
    WorkingHoursConstraintAgent,
};

fn main() {
    println!("=== Meeting Scheduler Example ===\n");

    let mut engine = Engine::new();

    // Register the scheduling agent pipeline
    engine.register(AvailabilityRetrievalAgent::new());
    engine.register(TimeZoneNormalizationAgent::new());
    engine.register(ConflictDetectionAgent::new());
    engine.register(WorkingHoursConstraintAgent::new());
    engine.register(SlotOptimizationAgent::new());

    // Register invariants — these MUST hold for convergence to succeed
    engine.add_invariant(RequireParticipantAvailability);
    engine.add_invariant(RequirePositiveDuration);
    engine.add_invariant(RequireValidSlot);

    // Seed the context with a scheduling request
    let mut ctx = Context::new();
    let _ = ctx.add_fact(Fact {
        key: ContextKey::Seeds,
        id: "request-1".to_string(),
        content: serde_json::json!({
            "participants": ["alice@example.com", "bob@example.com"],
            "duration_minutes": 60,
            "preferred_window": "2026-03-25T09:00..2026-03-25T17:00",
            "timezone": "Europe/Stockholm"
        })
        .to_string(),
    });

    println!("Scheduling request seeded.\n");

    // Run until convergence
    match engine.run(ctx) {
        Ok(result) => {
            println!("Converged: {}", result.converged);
            println!("Cycles:    {}", result.cycles);
            println!("Stop:      {:?}\n", result.stop_reason);

            // Print proposed slots
            for fact in result.context.get(ContextKey::Hypotheses) {
                println!("Proposed slot: {}", fact.content);
            }
        }
        Err(e) => {
            println!("Scheduling failed: {e}");
            println!("(This is expected if availability data is not seeded)");
        }
    }

    println!("\n=== Done ===");
}
