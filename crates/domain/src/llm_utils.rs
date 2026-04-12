// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Utilities for creating LLM-enabled agents with model selection.
//!
//! This module provides helpers for setting up LLM agents that follow
//! the Converge pattern for model selection based on agent requirements.

use crate::mock::{MockProvider, MockResponse};
use converge_core::{
    ContextKey,
    llm::{LlmAgent, LlmAgentConfig},
    model_selection::{AgentRequirements, CostClass},
    prompt::PromptFormat,
};
use std::sync::Arc;

/// Creates an LLM agent with a mock provider (for testing).
///
/// This bypasses model selection and uses a mock provider directly.
/// Returns both the agent and the mock provider so you can configure responses.
#[must_use]
pub fn create_mock_llm_agent(
    name: impl Into<String>,
    system_prompt: impl Into<String>,
    prompt_template: impl Into<String>,
    target_key: ContextKey,
    dependencies: Vec<ContextKey>,
    _requirements: AgentRequirements,
    mock_responses: Vec<MockResponse>,
) -> (LlmAgent, Arc<MockProvider>) {
    // Create mock provider with responses
    let mock_provider = Arc::new(MockProvider::new(mock_responses));

    // Create agent config
    let config = LlmAgentConfig {
        system_prompt: system_prompt.into(),
        prompt_template: prompt_template.into(),
        prompt_format: PromptFormat::Edn,
        target_key,
        dependencies,
        default_confidence: 0.7,
        max_tokens: 1024,
        temperature: 0.7,
        requirements: None,
    };

    let name_str = name.into();
    let agent = LlmAgent::new(name_str, mock_provider.clone(), config);
    (agent, mock_provider)
}

/// Common requirement presets for different agent types.
pub mod requirements {
    use super::{AgentRequirements, CostClass};

    /// Requirements for fast, high-volume agents (e.g., data extraction).
    #[must_use]
    pub fn fast_extraction() -> AgentRequirements {
        AgentRequirements::fast_cheap()
    }

    /// Requirements for analysis agents (e.g., market analysis, strategy synthesis).
    #[must_use]
    pub fn analysis() -> AgentRequirements {
        AgentRequirements::balanced().with_min_quality(0.75)
    }

    /// Requirements for deep research agents (e.g., competitor analysis, risk assessment).
    #[must_use]
    pub fn deep_research() -> AgentRequirements {
        AgentRequirements::deep_research()
    }

    /// Requirements for synthesis agents (e.g., strategy synthesis, consolidation).
    #[must_use]
    pub fn synthesis() -> AgentRequirements {
        AgentRequirements::new(CostClass::Medium, 10000, true).with_min_quality(0.8)
    }

    /// Requirements for validation agents (e.g., compliance checking, quality gates).
    #[must_use]
    pub fn validation() -> AgentRequirements {
        AgentRequirements::balanced().with_min_quality(0.85)
    }

    /// Requirements for categorization agents (e.g., category inference, classification).
    #[must_use]
    pub fn categorization() -> AgentRequirements {
        AgentRequirements::fast_cheap().with_min_quality(0.7)
    }
}
