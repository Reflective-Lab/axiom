// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Example agents for testing and demonstration.
//!
// Agent trait returns &str, but we return literals. This is fine.
#![allow(clippy::unnecessary_literal_bound)]
//!
//! These agents prove the core convergence properties:
//! - `SeedAgent`: Emits initial facts, stops when done
//! - `ReactOnceAgent`: Reacts to changes, stops after one contribution
//!
//! # Example
//!
//! ```
//! use converge_core::{Engine, Context, ContextKey};
//! use converge_core::agents::{SeedAgent, ReactOnceAgent};
//!
//! let mut engine = Engine::new();
//! engine.register(SeedAgent::new("seed-1", "initial value"));
//! engine.register(ReactOnceAgent::new("hyp-1", "derived insight"));
//!
//! let result = engine.run(Context::new()).expect("converges");
//! assert!(result.converged);
//! assert!(result.context.has(ContextKey::Seeds));
//! assert!(result.context.has(ContextKey::Hypotheses));
//! ```

use crate::agent::Agent;
use crate::context::{ContextKey, Fact};
use crate::effect::AgentEffect;

/// An agent that emits an initial seed fact once.
///
/// Demonstrates:
/// - Agent with no dependencies (runs first)
/// - Self-terminating behavior (checks if already contributed)
/// - Monotonic context evolution
pub struct SeedAgent {
    fact_id: String,
    content: String,
}

impl SeedAgent {
    /// Creates a new seed agent.
    #[must_use]
    pub fn new(fact_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            fact_id: fact_id.into(),
            content: content.into(),
        }
    }
}

impl Agent for SeedAgent {
    fn name(&self) -> &str {
        "SeedAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[] // No dependencies = eligible on first cycle
    }

    fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
        // Only run if we haven't contributed yet
        !ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id == self.fact_id)
    }

    fn execute(&self, _ctx: &dyn crate::ContextView) -> AgentEffect {
        AgentEffect::with_fact(Fact {
            key: ContextKey::Seeds,
            id: self.fact_id.clone(),
            content: self.content.clone(),
        })
    }
}

/// An agent that reacts to seeds by emitting a hypothesis once.
///
/// Demonstrates:
/// - Dependency-driven activation (only runs when Seeds change)
/// - Data-driven behavior (reads context to decide)
/// - Self-terminating (checks if already contributed)
pub struct ReactOnceAgent {
    fact_id: String,
    content: String,
}

impl ReactOnceAgent {
    /// Creates a new reactive agent.
    #[must_use]
    pub fn new(fact_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            fact_id: fact_id.into(),
            content: content.into(),
        }
    }
}

impl Agent for ReactOnceAgent {
    fn name(&self) -> &str {
        "ReactOnceAgent"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds] // Only wake when Seeds change
    }

    fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
        // Run if: seeds exist AND we haven't contributed
        ctx.has(ContextKey::Seeds)
            && !ctx
                .get(ContextKey::Hypotheses)
                .iter()
                .any(|f| f.id == self.fact_id)
    }

    fn execute(&self, _ctx: &dyn crate::ContextView) -> AgentEffect {
        AgentEffect::with_fact(Fact {
            key: ContextKey::Hypotheses,
            id: self.fact_id.clone(),
            content: self.content.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::engine::Engine;

    #[test]
    fn seed_agent_emits_once() {
        let mut engine = Engine::new();
        engine.register(SeedAgent::new("s1", "value"));

        let result = engine.run(Context::new()).expect("converges");

        assert!(result.converged);
        assert_eq!(result.context.get(ContextKey::Seeds).len(), 1);
    }

    #[test]
    fn react_once_agent_chains_from_seed() {
        let mut engine = Engine::new();
        engine.register(SeedAgent::new("s1", "seed"));
        engine.register(ReactOnceAgent::new("h1", "hypothesis"));

        let result = engine.run(Context::new()).expect("converges");

        assert!(result.converged);
        assert!(result.context.has(ContextKey::Seeds));
        assert!(result.context.has(ContextKey::Hypotheses));
    }

    #[test]
    fn multiple_seeds_all_converge() {
        let mut engine = Engine::new();
        engine.register(SeedAgent::new("s1", "first"));
        engine.register(SeedAgent::new("s2", "second"));
        engine.register(SeedAgent::new("s3", "third"));

        let result = engine.run(Context::new()).expect("converges");

        assert!(result.converged);
        assert_eq!(result.context.get(ContextKey::Seeds).len(), 3);
    }

    #[test]
    fn chain_of_three_converges() {
        /// Third agent in the chain.
        struct StrategyAgent;

        impl Agent for StrategyAgent {
            fn name(&self) -> &str {
                "StrategyAgent"
            }

            fn dependencies(&self) -> &[ContextKey] {
                &[ContextKey::Hypotheses]
            }

            fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
                ctx.has(ContextKey::Hypotheses) && !ctx.has(ContextKey::Strategies)
            }

            fn execute(&self, _ctx: &dyn crate::ContextView) -> AgentEffect {
                AgentEffect::with_fact(Fact {
                    key: ContextKey::Strategies,
                    id: "strat-1".into(),
                    content: "derived strategy".into(),
                })
            }
        }

        let mut engine = Engine::new();
        engine.register(SeedAgent::new("s1", "seed"));
        engine.register(ReactOnceAgent::new("h1", "hypothesis"));
        engine.register(StrategyAgent);

        let result = engine.run(Context::new()).expect("converges");

        assert!(result.converged);
        assert!(result.context.has(ContextKey::Seeds));
        assert!(result.context.has(ContextKey::Hypotheses));
        assert!(result.context.has(ContextKey::Strategies));
    }
}
