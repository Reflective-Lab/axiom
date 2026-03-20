// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! LLM-enabled drafting flow with explicit providers:
//! - Perplexity for research (web search)
//! - Anthropic for drafting (composition)
//!
//! NOTE: Temporarily disabled. The `create_provider` function returns
//! `converge_provider::LlmProvider` which is a different trait from
//! `converge_core::llm::LlmProvider`. This module needs updating once
//! the trait unification lands. See REF-36.

// TODO: Re-enable once converge-provider's LlmProvider trait is unified with converge-core's.
// The build_agent function used create_provider() which returns Arc<dyn converge_provider::LlmProvider>
// but LlmAgent::new expects Arc<dyn converge_core::llm::LlmProvider>.
