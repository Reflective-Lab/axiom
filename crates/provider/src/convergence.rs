// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Convergence integration for LLM providers.
//!
//! This module bridges the gap between the simple `LlmProvider` invocation
//! trait and the platform-wide convergence contract (`Backend`, `Suggestor`).

use converge_pack::{AgentEffect, Context, ContextKey, ProposedFact, Suggestor};
use converge_provider_api::{Backend, BackendKind, Capability};

use crate::provider_api::{LlmProvider, LlmRequest};

/// Wraps any `LlmProvider` as a convergence [`Suggestor`].
///
/// `LlmAgent` reads from specified dependency keys, builds a prompt from
/// context, calls the wrapped provider, and emits `ProposedFact` instances
/// via `AgentEffect::with_proposal`.
pub struct LlmAgent<P: LlmProvider> {
    provider: P,
    agent_name: String,
    target_key: ContextKey,
    dependency_keys: Vec<ContextKey>,
    system_prompt: Option<String>,
}

impl<P: LlmProvider> LlmAgent<P> {
    /// Create a new LLM agent wrapping a provider.
    #[must_use]
    pub fn new(provider: P, agent_name: impl Into<String>, target_key: ContextKey) -> Self {
        Self {
            provider,
            agent_name: agent_name.into(),
            target_key,
            dependency_keys: vec![ContextKey::Seeds],
            system_prompt: None,
        }
    }

    /// Set the context keys this agent depends on.
    #[must_use]
    pub fn with_dependencies(mut self, keys: Vec<ContextKey>) -> Self {
        self.dependency_keys = keys;
        self
    }

    /// Set a system prompt for the LLM.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Build a prompt from context facts.
    fn build_prompt(&self, ctx: &dyn Context) -> String {
        let mut parts = Vec::new();

        for &key in &self.dependency_keys {
            let facts = ctx.get(key);
            if !facts.is_empty() {
                parts.push(format!("## {key:?}"));
                for fact in facts {
                    parts.push(format!("- {}", fact.content));
                }
            }
        }

        if parts.is_empty() {
            "No context available.".to_string()
        } else {
            parts.join("\n")
        }
    }

    /// Check if this agent has already contributed proposals.
    fn has_contributed(&self, ctx: &dyn Context) -> bool {
        let proposals = ctx.get_proposals(ContextKey::Proposals);
        proposals.iter().any(|p| p.provenance == self.agent_name)
    }
}

impl<P: LlmProvider> Suggestor for LlmAgent<P> {
    fn name(&self) -> &str {
        &self.agent_name
    }

    fn dependencies(&self) -> &[ContextKey] {
        &self.dependency_keys
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        if self.has_contributed(ctx) {
            return false;
        }
        self.dependency_keys.iter().any(|&key| ctx.has(key))
    }

    fn execute(&self, ctx: &dyn Context) -> AgentEffect {
        let prompt = self.build_prompt(ctx);

        let mut request = LlmRequest::new(prompt);
        if let Some(ref system) = self.system_prompt {
            request = request.with_system(system.clone());
        }

        match self.provider.complete(&request) {
            Ok(response) => {
                let proposal_id = format!(
                    "proposal:{:?}:{}-{}",
                    self.target_key, self.agent_name, response.model
                );

                let proposal = ProposedFact {
                    key: self.target_key,
                    id: proposal_id,
                    content: response.content,
                    confidence: 0.7,
                    provenance: self.agent_name.clone(),
                };

                AgentEffect::with_proposal(proposal)
            }
            Err(e) => {
                tracing::warn!(
                    agent = %self.agent_name,
                    error = %e,
                    "LLM provider call failed, emitting nothing"
                );
                AgentEffect::empty()
            }
        }
    }
}

impl<P: LlmProvider> Backend for LlmAgent<P> {
    fn name(&self) -> &str {
        &self.agent_name
    }

    fn kind(&self) -> BackendKind {
        BackendKind::Llm
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::TextGeneration]
    }

    fn supports_replay(&self) -> bool {
        false
    }

    fn requires_network(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_api::{FinishReason, LlmError, LlmResponse, TokenUsage};
    use converge_core::{Context as CoreContext, Engine};
    use converge_pack::{Context, ContextKey, Fact, ProposedFact};

    fn promoted_fact(key: ContextKey, id: &str, content: &str) -> Fact {
        let mut ctx = CoreContext::new();
        let _ = ctx.add_input(key, id, content);
        Engine::new()
            .run(ctx)
            .expect("should promote test input")
            .context
            .get(key)
            .first()
            .expect("promoted fact should exist")
            .clone()
    }

    /// Minimal in-memory context for testing.
    struct TestContext {
        facts: std::collections::HashMap<ContextKey, Vec<Fact>>,
        proposals: std::collections::HashMap<ContextKey, Vec<ProposedFact>>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                facts: std::collections::HashMap::new(),
                proposals: std::collections::HashMap::new(),
            }
        }

        fn with_seed(mut self, content: &str) -> Self {
            self.facts
                .entry(ContextKey::Seeds)
                .or_default()
                .push(promoted_fact(
                    ContextKey::Seeds,
                    &format!("seed:{content}"),
                    content,
                ));
            self
        }

        fn with_proposal(mut self, agent: &str, content: &str) -> Self {
            self.proposals
                .entry(ContextKey::Proposals)
                .or_default()
                .push(ProposedFact {
                    key: ContextKey::Hypotheses,
                    id: format!("proposal:test:{agent}"),
                    content: content.to_string(),
                    confidence: 0.8,
                    provenance: agent.to_string(),
                });
            self
        }
    }

    impl Context for TestContext {
        fn has(&self, key: ContextKey) -> bool {
            self.facts.get(&key).is_some_and(|v| !v.is_empty())
        }

        fn get(&self, key: ContextKey) -> &[Fact] {
            self.facts.get(&key).map_or(&[], Vec::as_slice)
        }

        fn get_proposals(&self, key: ContextKey) -> &[ProposedFact] {
            self.proposals.get(&key).map_or(&[], Vec::as_slice)
        }
    }

    struct StubProvider {
        response: String,
    }

    impl StubProvider {
        fn new(response: &str) -> Self {
            Self {
                response: response.to_string(),
            }
        }
    }

    impl LlmProvider for StubProvider {
        fn name(&self) -> &'static str {
            "stub"
        }

        fn model(&self) -> &str {
            "stub-model"
        }

        fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            Ok(LlmResponse {
                content: self.response.clone(),
                model: "stub-model".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
                finish_reason: FinishReason::Stop,
            })
        }
    }

    #[test]
    fn agent_accepts_when_seeds_exist() {
        let ctx = TestContext::new().with_seed("Analyze the market");
        let agent = LlmAgent::new(
            StubProvider::new("analysis"),
            "test-analyst",
            ContextKey::Hypotheses,
        );
        assert!(agent.accepts(&ctx));
    }

    #[test]
    fn agent_rejects_empty_context() {
        let ctx = TestContext::new();
        let agent = LlmAgent::new(
            StubProvider::new("analysis"),
            "test-analyst",
            ContextKey::Hypotheses,
        );
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn agent_rejects_when_already_contributed() {
        let ctx = TestContext::new()
            .with_seed("Analyze")
            .with_proposal("test-analyst", "my analysis");
        let agent = LlmAgent::new(
            StubProvider::new("analysis"),
            "test-analyst",
            ContextKey::Hypotheses,
        );
        assert!(!agent.accepts(&ctx));
    }

    #[test]
    fn agent_produces_proposals_not_facts() {
        let ctx = TestContext::new().with_seed("Analyze competitors");
        let agent = LlmAgent::new(
            StubProvider::new("Competitor X is strong"),
            "competitor-analyst",
            ContextKey::Competitors,
        );

        let effect = agent.execute(&ctx);
        assert!(!effect.is_empty());
        assert_eq!(effect.proposals.len(), 1);
        assert_eq!(effect.proposals[0].content, "Competitor X is strong");
        assert_eq!(effect.proposals[0].key, ContextKey::Competitors);
        assert_eq!(effect.proposals[0].provenance, "competitor-analyst");
    }

    #[test]
    fn agent_backend_identity() {
        let agent = LlmAgent::new(
            StubProvider::new("test"),
            "my-agent",
            ContextKey::Hypotheses,
        );
        assert_eq!(Backend::name(&agent), "my-agent");
        assert_eq!(agent.kind(), BackendKind::Llm);
        assert!(agent.has_capability(Capability::TextGeneration));
        assert!(!agent.supports_replay());
        assert!(agent.requires_network());
    }

    #[test]
    fn agent_handles_provider_failure() {
        struct FailingProvider;
        impl LlmProvider for FailingProvider {
            fn name(&self) -> &'static str {
                "failing"
            }
            fn model(&self) -> &str {
                "fail-model"
            }
            fn complete(&self, _: &LlmRequest) -> Result<LlmResponse, LlmError> {
                Err(LlmError::network("connection refused"))
            }
        }

        let ctx = TestContext::new().with_seed("test");
        let agent = LlmAgent::new(FailingProvider, "fail-test", ContextKey::Hypotheses);
        let effect = agent.execute(&ctx);

        assert!(effect.is_empty());
    }
}
