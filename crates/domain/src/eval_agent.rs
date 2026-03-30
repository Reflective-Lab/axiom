// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Agent that executes evals and stores results in context.
//!
//! This agent demonstrates how evals can be integrated into the convergence
//! loop. It runs registered evals when their dependencies change and stores
//! results as facts in context.

use converge_core::{Agent, AgentEffect, Context, ContextKey, Eval, EvalId, EvalRegistry, Fact};

/// Agent that executes evals and stores results in context.
///
/// This agent:
/// - Runs when eval dependencies change
/// - Executes registered evals
/// - Stores results as facts in `ContextKey::Evaluations`
/// - Is idempotent (checks for existing eval results)
pub struct EvalExecutionAgent {
    /// Registry of evals to execute.
    registry: EvalRegistry,
    /// Name of this agent instance.
    name: String,
}

impl EvalExecutionAgent {
    /// Creates a new eval execution agent with a registry.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            registry: EvalRegistry::new(),
            name: name.into(),
        }
    }

    /// Registers an eval to be executed by this agent.
    pub fn register_eval(&mut self, eval: impl Eval + 'static) -> EvalId {
        self.registry.register(eval)
    }

    /// Returns the registry (for external eval registration).
    #[must_use]
    pub fn registry(&self) -> &EvalRegistry {
        &self.registry
    }
}

impl Agent for EvalExecutionAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn dependencies(&self) -> &[ContextKey] {
        // Dependencies are determined by registered evals
        // For simplicity, we declare all common keys
        // In production, this would be computed from eval dependencies
        &[
            ContextKey::Strategies,
            ContextKey::Evaluations,
            ContextKey::Constraints,
            ContextKey::Signals,
        ]
    }

    fn accepts(&self, ctx: &dyn converge_core::ContextView) -> bool {
        // Run if:
        // 1. We have strategies or other eval inputs
        // 2. We haven't already run evals (idempotency check)
        let has_inputs = ctx.has(ContextKey::Strategies)
            || ctx.has(ContextKey::Constraints)
            || ctx.has(ContextKey::Signals);

        if !has_inputs {
            return false;
        }

        // Idempotency: check if we've already run evals
        // Look for eval results with our agent name suffix (ID format: "eval:<eval_name>:<agent_name>")
        let my_suffix = format!(":{}", self.name);
        let has_existing = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id.ends_with(&my_suffix));

        !has_existing
    }

    fn execute(&self, ctx: &dyn converge_core::ContextView) -> AgentEffect {
        // Get dirty keys from context (simplified: use all keys with data)
        let dirty_keys: Vec<ContextKey> = [
            ContextKey::Strategies,
            ContextKey::Constraints,
            ContextKey::Signals,
        ]
        .iter()
        .filter(|&&key| ctx.has(key))
        .copied()
        .collect();

        // Execute evals that depend on dirty keys
        let results = if dirty_keys.is_empty() {
            // No dependencies changed, run all evals
            self.registry.evaluate_all(ctx)
        } else {
            // Run only evals that depend on changed keys
            self.registry.evaluate_dependent(ctx, &dirty_keys)
        };

        // Convert results to facts
        let facts: Vec<Fact> = results
            .into_iter()
            .map(|result| {
                // Include agent name in eval ID for traceability
                result.to_fact(Some(&self.name))
            })
            .collect();

        AgentEffect::with_facts(facts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evals::MeetingScheduleFeasibilityEval;

    #[test]
    fn eval_agent_executes_registered_evals() {
        let mut agent = EvalExecutionAgent::new("test_eval_agent");
        agent.register_eval(MeetingScheduleFeasibilityEval);

        let mut ctx = Context::new();
        ctx.add_fact(Fact::new(
            ContextKey::Strategies,
            "strat-1",
            "email campaign",
        ))
        .unwrap();

        // Agent should accept (has inputs, no existing evals)
        assert!(agent.accepts(&ctx));

        let effect = agent.execute(&ctx);
        assert!(!effect.facts.is_empty());

        // Check that eval result was stored
        let eval_facts: Vec<&Fact> = effect
            .facts
            .iter()
            .filter(|f| f.key == ContextKey::Evaluations)
            .collect();
        assert!(!eval_facts.is_empty());
    }

    #[test]
    fn eval_agent_is_idempotent() {
        let mut agent = EvalExecutionAgent::new("test_eval_agent");
        agent.register_eval(MeetingScheduleFeasibilityEval);

        let mut ctx = Context::new();
        ctx.add_fact(Fact::new(
            ContextKey::Strategies,
            "strat-1",
            "email campaign",
        ))
        .unwrap();

        // First execution
        assert!(agent.accepts(&ctx));
        let effect1 = agent.execute(&ctx);

        // Add eval results to context (simulating merge)
        for fact in effect1.facts {
            ctx.add_fact(fact).unwrap();
        }

        // Second execution should not be accepted (idempotency)
        assert!(!agent.accepts(&ctx));
    }
}
