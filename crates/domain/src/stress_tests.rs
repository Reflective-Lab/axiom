// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Comprehensive stress tests for the Converge kernel.
//!
//! These tests stress the system in various dimensions:
//! - Parallelism: Many agents running simultaneously
//! - Determinism: Same inputs produce same outputs
//! - Convergence: System always reaches fixed point
//! - Edge cases: Empty inputs, no solutions, invalid data

use converge_core::agents::SeedAgent;
use converge_core::{Context, Engine};

// Import kernel use case agents from lib.rs exports
use crate::{
    // Meeting Scheduler
    AvailabilityRetrievalAgent,
    ConflictDetectionAgent,
    SlotOptimizationAgent,
    TimeZoneNormalizationAgent,
    WorkingHoursConstraintAgent,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: Kernel use cases converge deterministically
    #[test]
    fn kernel_use_cases_converge_deterministically() {
        // Test meeting scheduler
        {
            let run = || {
                let mut engine = Engine::new();
                engine.register(SeedAgent::new("participants", "Alice, Bob"));
                engine.register(AvailabilityRetrievalAgent);
                engine.register(TimeZoneNormalizationAgent);
                engine.register(WorkingHoursConstraintAgent);
                engine.register(SlotOptimizationAgent);
                engine.register(ConflictDetectionAgent);
                engine.run(Context::new()).expect("should converge")
            };
            let r1 = run();
            let r2 = run();
            assert_eq!(r1.cycles, r2.cycles, "meeting_scheduler: cycles must match");
            assert_eq!(
                r1.context.get(converge_core::ContextKey::Evaluations),
                r2.context.get(converge_core::ContextKey::Evaluations),
                "meeting_scheduler: evaluations must match"
            );
        }
    }

    /// Test: Multiple kernel use cases can run in sequence without interference
    #[test]
    fn multiple_kernel_use_cases_no_interference() {
        // Run meeting scheduler twice with different inputs
        let mut engine1 = Engine::new();
        engine1.register(SeedAgent::new("participants", "Alice, Bob"));
        engine1.register(AvailabilityRetrievalAgent);
        engine1.register(SlotOptimizationAgent);
        let r1 = engine1.run(Context::new()).expect("should converge");

        let mut engine2 = Engine::new();
        engine2.register(SeedAgent::new("participants", "Charlie"));
        engine2.register(AvailabilityRetrievalAgent);
        engine2.register(SlotOptimizationAgent);
        let r2 = engine2.run(Context::new()).expect("should converge");

        // Both should converge independently
        assert!(r1.converged);
        assert!(r2.converged);
    }
}
