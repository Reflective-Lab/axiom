// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Meeting Scheduler — domain pack with constraint agents.
//!
//! Shows: domain agents, invariants, constraint satisfaction via convergence.

use std::collections::BTreeMap;
use std::sync::Arc;

use converge_core::{
    Context, ContextKey, Engine, EventQuery, ExperienceEvent, ExperienceEventKind, ExperienceStore,
};
use converge_domain::{
    AvailabilityRetrievalAgent, ConflictDetectionAgent, RequireParticipantAvailability,
    RequirePositiveDuration, RequireValidSlot, SlotOptimizationAgent, TimeZoneNormalizationAgent,
    WorkingHoursConstraintAgent,
};
use converge_experience::{InMemoryExperienceStore, StoreObserver};

fn event_kind_label(kind: ExperienceEventKind) -> &'static str {
    match kind {
        ExperienceEventKind::ProposalCreated => "proposal_created",
        ExperienceEventKind::ProposalValidated => "proposal_validated",
        ExperienceEventKind::FactPromoted => "fact_promoted",
        ExperienceEventKind::RecallExecuted => "recall_executed",
        ExperienceEventKind::ReplayTraceRecorded => "trace_link_recorded",
        ExperienceEventKind::ReplayabilityDowngraded => "replayability_downgraded",
        ExperienceEventKind::ArtifactStateTransitioned => "artifact_state_transitioned",
        ExperienceEventKind::ArtifactRollbackRecorded => "artifact_rollback_recorded",
        ExperienceEventKind::BackendInvoked => "backend_invoked",
        ExperienceEventKind::OutcomeRecorded => "outcome_recorded",
        ExperienceEventKind::BudgetExceeded => "budget_exceeded",
        ExperienceEventKind::PolicySnapshotCaptured => "policy_snapshot_captured",
    }
}

fn print_experience_summary(store: &Arc<InMemoryExperienceStore>) {
    let Ok(events) = store.query_events(&EventQuery::default()) else {
        println!("\nExperience capture unavailable.");
        return;
    };

    if events.is_empty() {
        println!("\nNo experience events captured.");
        return;
    }

    let mut counts = BTreeMap::new();
    let mut promoted_facts = Vec::new();

    for envelope in events {
        let label = event_kind_label(envelope.event.kind());
        *counts.entry(label).or_insert(0usize) += 1;

        if let ExperienceEvent::FactPromoted {
            fact_id, reason, ..
        } = envelope.event
        {
            promoted_facts.push(format!("{fact_id} ({reason})"));
        }
    }

    println!("\nExperience capture:");
    for (label, count) in counts {
        println!("  {label}: {count}");
    }

    if !promoted_facts.is_empty() {
        println!("  promoted facts:");
        for fact in promoted_facts.into_iter().take(5) {
            println!("    - {fact}");
        }
    }
}

fn main() {
    println!("=== Meeting Scheduler Example ===\n");

    let mut engine = Engine::new();
    let experience_store = Arc::new(InMemoryExperienceStore::new());

    engine.register_suggestor(AvailabilityRetrievalAgent);
    engine.register_suggestor(TimeZoneNormalizationAgent);
    engine.register_suggestor(ConflictDetectionAgent);
    engine.register_suggestor(WorkingHoursConstraintAgent);
    engine.register_suggestor(SlotOptimizationAgent);

    engine.register_invariant(RequireParticipantAvailability);
    engine.register_invariant(RequirePositiveDuration);
    engine.register_invariant(RequireValidSlot);
    engine.set_event_observer(Arc::new(StoreObserver::new(experience_store.clone())));

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

    print_experience_summary(&experience_store);

    println!("\n=== Done ===");
}
