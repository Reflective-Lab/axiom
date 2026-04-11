// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Meeting Scheduler — domain pack with constraint agents.
//!
//! Shows: domain agents, invariants, constraint satisfaction via convergence.

use converge_core::{Context, ContextKey, Engine};
use converge_domain::{
    AvailabilityRetrievalAgent, ConflictDetectionAgent, RequireParticipantAvailability,
    RequirePositiveDuration, RequireValidSlot, SlotOptimizationAgent, TimeZoneNormalizationAgent,
    WorkingHoursConstraintAgent,
};

fn main() {
    println!("=== Meeting Scheduler Example ===\n");

    let mut engine = Engine::new();

    engine.register_suggestor(AvailabilityRetrievalAgent);
    engine.register_suggestor(TimeZoneNormalizationAgent);
    engine.register_suggestor(ConflictDetectionAgent);
    engine.register_suggestor(WorkingHoursConstraintAgent);
    engine.register_suggestor(SlotOptimizationAgent);

    engine.register_invariant(RequireParticipantAvailability);
    engine.register_invariant(RequirePositiveDuration);
    engine.register_invariant(RequireValidSlot);

    let mut ctx = Context::new();
    let _ = ctx.add_input(
        ContextKey::Seeds,
        "request-1",
        serde_json::json!({
            "participants": ["alice@example.com", "bob@example.com"],
            "duration_minutes": 60,
            "preferred_window": "2026-03-25T09:00..2026-03-25T17:00",
            "timezone": "Europe/Stockholm"
        })
        .to_string(),
    );

    println!("Scheduling request seeded.\n");

    match engine.run(ctx) {
        Ok(result) => {
            println!("Converged: {}", result.converged);
            println!("Cycles:    {}", result.cycles);
            println!("Stop:      {:?}\n", result.stop_reason);

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
