// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Agent trait and types for Converge.
//!
//! Agents are semantic capabilities that observe context and emit effects.
//! They never call each other, never control flow, and never decide termination.

use crate::context::{Context, ContextKey};
use crate::effect::AgentEffect;

/// Unique identifier for a registered agent.
///
/// Assigned monotonically at registration time.
/// Used for deterministic effect merge ordering (see DECISIONS.md §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(pub(crate) u32);

impl AgentId {
    /// Returns the raw numeric ID.
    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Agent({})", self.0)
    }
}

/// A semantic capability that observes context and emits effects.
///
/// # Contract
///
/// Agents must:
/// - Declare their dependencies (which context keys they care about)
/// - Be pure in `accepts()` — no side effects
/// - Only read context in `execute()` — never mutate
/// - Return effects, not modify state directly
///
/// Agents must NOT:
/// - Call other agents
/// - Control execution flow
/// - Decide when to terminate
///
/// # Example
///
/// ```ignore
/// struct SeedAgent;
///
/// impl Agent for SeedAgent {
///     fn name(&self) -> &str { "SeedAgent" }
///
///     fn dependencies(&self) -> &[ContextKey] { &[] }
///
///     fn accepts(&self, ctx: &Context) -> bool {
///         !ctx.has(ContextKey::Seeds)
///     }
///
///     fn execute(&self, _ctx: &Context) -> AgentEffect {
///         AgentEffect::with_fact(Fact { ... })
///     }
/// }
/// ```
pub trait Agent: Send + Sync {
    /// Human-readable name for debugging and tracing.
    fn name(&self) -> &str;

    /// Context keys this agent depends on.
    ///
    /// The engine uses this to build the dependency index.
    /// An agent is only considered for re-evaluation when
    /// one of its dependencies changes.
    ///
    /// Return `&[]` for agents that should run on every cycle
    /// (e.g., seed agents that check for absence of data).
    fn dependencies(&self) -> &[ContextKey];

    /// Returns true if this agent should execute given the current context.
    ///
    /// This must be:
    /// - Pure (no side effects)
    /// - Deterministic (same context → same result)
    /// - Fast (called frequently)
    fn accepts(&self, ctx: &Context) -> bool;

    /// Execute the agent's logic and return effects.
    ///
    /// The agent receives immutable access to context.
    /// All contributions must be returned as an `AgentEffect`.
    ///
    /// This may:
    /// - Read context
    /// - Call external tools (LLMs, APIs)
    /// - Perform computation
    ///
    /// This must NOT:
    /// - Mutate any shared state
    /// - Call other agents
    /// - Block indefinitely (respect timeouts)
    fn execute(&self, ctx: &Context) -> AgentEffect;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Fact;

    /// A minimal test agent that emits one fact then stops.
    struct TestAgent {
        fact_id: String,
    }

    impl Agent for TestAgent {
        fn name(&self) -> &'static str {
            "TestAgent"
        }

        fn dependencies(&self) -> &[ContextKey] {
            &[ContextKey::Seeds]
        }

        fn accepts(&self, ctx: &Context) -> bool {
            // Only run if our fact doesn't exist yet
            !ctx.get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id == self.fact_id)
        }

        fn execute(&self, _ctx: &Context) -> AgentEffect {
            AgentEffect::with_fact(Fact {
                key: ContextKey::Seeds,
                id: self.fact_id.clone(),
                content: "test content".into(),
            })
        }
    }

    #[test]
    fn agent_accepts_when_fact_missing() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let ctx = Context::new();

        assert!(agent.accepts(&ctx));
    }

    #[test]
    fn agent_rejects_when_fact_present() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let mut ctx = Context::new();
        let _ = ctx.add_fact(Fact {
            key: ContextKey::Seeds,
            id: "test-1".into(),
            content: "already here".into(),
        });

        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn agent_produces_effect() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let ctx = Context::new();

        let effect = agent.execute(&ctx);
        assert_eq!(effect.facts.len(), 1);
        assert_eq!(effect.facts[0].id, "test-1");
    }

    #[test]
    fn agent_id_ordering() {
        let a = AgentId(1);
        let b = AgentId(2);
        let c = AgentId(1);

        assert!(a < b);
        assert_eq!(a, c);
    }
}
