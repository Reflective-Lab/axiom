// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! Convergence integration for LLM providers.
//!
//! This module bridges the gap between the simple `LlmProvider` invocation
//! trait and the platform-wide convergence contract (`Backend`, `Agent`).
//!
//! # Architecture
//!
//! ```text
//! converge-traits
//!     │
//!     ├── Backend (identity + capabilities)
//!     └── Agent (accepts + execute → AgentEffect)
//!
//! converge-provider
//!     │
//!     ├── LlmProvider (invocation: prompt → response)
//!     └── LlmAgent<P> (wraps LlmProvider as convergence Agent)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use converge_provider::{AnthropicProvider, LlmAgent};
//! use converge_traits::ContextKey;
//!
//! let provider = AnthropicProvider::new("key", "claude-sonnet-4-6");
//! let agent = LlmAgent::new(provider, "competitor-analyst", ContextKey::Competitors);
//! // agent implements converge_traits::Agent
//! ```

use converge_traits::{
    Agent, AgentEffect, Backend, BackendKind, Capability, Context, ContextKey, ProposedFact,
};

use crate::provider_api::{LlmProvider, LlmRequest};

/// Wraps any `LlmProvider` as a convergence [`Agent`].
///
/// `LlmAgent` reads from specified dependency keys, builds a prompt from
/// context, calls the wrapped provider, and emits `ProposedFact` instances
/// to `ContextKey::Proposals`.
///
/// # Design
///
/// - LLM outputs are *proposals*, never facts. The trust boundary is
///   enforced by emitting `AgentEffect::Propose`, not `AgentEffect::AddFacts`.
/// - Idempotency is context-based: if this agent has already contributed
///   proposals, it does not re-execute.
/// - The agent is `Send + Sync` because `LlmProvider` is `Send + Sync`.
pub struct LlmAgent<P: LlmProvider> {
    provider: P,
    agent_name: String,
    target_key: ContextKey,
    dependency_keys: Vec<ContextKey>,
    system_prompt: Option<String>,
}

impl<P: LlmProvider> LlmAgent<P> {
    /// Create a new LLM agent wrapping a provider.
    ///
    /// - `provider`: The LLM provider to call.
    /// - `agent_name`: Unique name for this agent in the convergence run.
    /// - `target_key`: The context key that proposals target (e.g., `Competitors`).
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
        proposals.iter().any(|p| p.source_agent == self.agent_name)
    }
}

impl<P: LlmProvider> Agent for LlmAgent<P> {
    fn name(&self) -> &str {
        &self.agent_name
    }

    fn dependencies(&self) -> &[ContextKey] {
        &self.dependency_keys
    }

    fn accepts(&self, ctx: &dyn Context) -> bool {
        // Don't re-execute if we've already contributed
        if self.has_contributed(ctx) {
            return false;
        }

        // Execute if any dependency key has facts
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
                    id: proposal_id,
                    target_key: self.target_key,
                    content: response.content,
                    source_agent: self.agent_name.clone(),
                };

                AgentEffect::Propose(vec![proposal])
            }
            Err(e) => {
                tracing::warn!(
                    agent = %self.agent_name,
                    error = %e,
                    "LLM provider call failed, emitting nothing"
                );
                AgentEffect::Nothing
            }
        }
    }
}

// ── Backend implementation for LlmAgent ──────────────────────────────

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
        false // Remote LLM APIs are non-deterministic
    }

    fn requires_network(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_api::{FinishReason, LlmError, LlmResponse, TokenUsage};
    use converge_traits::{Context, ContextKey, Fact, ProposedFact};

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
            self.facts.entry(ContextKey::Seeds).or_default().push(Fact {
                id: format!("seed:{content}"),
                key: ContextKey::Seeds,
                content: content.to_string(),
            });
            self
        }

        fn with_proposal(mut self, agent: &str, content: &str) -> Self {
            self.proposals
                .entry(ContextKey::Proposals)
                .or_default()
                .push(ProposedFact {
                    id: format!("proposal:test:{agent}"),
                    target_key: ContextKey::Hypotheses,
                    content: content.to_string(),
                    source_agent: agent.to_string(),
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

    /// Deterministic provider for convergence tests.
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

        match effect {
            AgentEffect::Propose(proposals) => {
                assert_eq!(proposals.len(), 1);
                assert_eq!(proposals[0].content, "Competitor X is strong");
                assert_eq!(proposals[0].target_key, ContextKey::Competitors);
                assert_eq!(proposals[0].source_agent, "competitor-analyst");
            }
            other => panic!("Expected Propose, got {other:?}"),
        }
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

        assert!(matches!(effect, AgentEffect::Nothing));
    }
}
