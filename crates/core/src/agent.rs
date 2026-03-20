// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Agent trait and types for Converge.
//!
//! The `Agent` trait is defined in `converge-traits` and re-exported here.
//! `AgentId` is a core-internal type for deterministic ordering.

// Re-export the canonical Agent trait
pub use converge_traits::Agent;

/// Unique identifier for a registered agent.
///
/// Assigned monotonically at registration time.
/// Used for deterministic effect merge ordering.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ContextKey, Fact};
    use crate::effect::AgentEffect;

    /// A minimal test agent that emits one fact then stops.
    struct TestAgent {
        fact_id: String,
    }

    impl Agent for TestAgent {
        fn name(&self) -> &str {
            "TestAgent"
        }

        fn dependencies(&self) -> &[ContextKey] {
            &[ContextKey::Seeds]
        }

        fn accepts(&self, ctx: &dyn crate::ContextView) -> bool {
            !ctx.get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id == self.fact_id)
        }

        fn execute(&self, _ctx: &dyn crate::ContextView) -> AgentEffect {
            AgentEffect::with_fact(Fact::new(
                ContextKey::Seeds,
                self.fact_id.clone(),
                "test content",
            ))
        }
    }

    #[test]
    fn agent_accepts_when_fact_missing() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let ctx = crate::context::Context::new();
        assert!(agent.accepts(&ctx));
    }

    #[test]
    fn agent_rejects_when_fact_present() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let mut ctx = crate::context::Context::new();
        let _ = ctx.add_fact(Fact::new(ContextKey::Seeds, "test-1", "already here"));
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn agent_produces_effect() {
        let agent = TestAgent {
            fact_id: "test-1".into(),
        };
        let ctx = crate::context::Context::new();
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
