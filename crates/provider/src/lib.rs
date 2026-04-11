// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

// Pedantic clippy lints — allow stylistic ones that don't improve correctness
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::doc_link_with_quotes)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::unused_self)]
#![allow(clippy::needless_pass_by_value)]

//! Capability adapters for the Converge runtime.
//!
//! > **Providers produce observations, never decisions.**
//! > **Converge converges; providers adapt.**
//!
//! This crate provides capability adapters (providers) that connect Converge
//! workflows to external systems. Each provider implements
//! [`converge_provider_api::Backend`]
//! for identity and capability declaration, and [`provider_api::LlmProvider`] for
//! invocation. Providers return structured observations with provenance.
//!
//! # What Is a Provider?
//!
//! A provider is an **adapter** that:
//! - Implements capability traits (`LlmProvider`, `Embedding`, `VectorRecall`, etc.)
//! - Returns observations (not facts, not decisions)
//! - Includes provenance metadata for tracing
//! - Is stateless (no hidden lifecycle state)
//!
//! A provider is **NOT**:
//! - An agent (agents live in `converge-core`)
//! - Orchestration (no workflows, no scheduling)
//! - Domain logic (business rules live in `converge-domain`)
//!
//! # Available Providers
//!
//! ## Remote Providers
//! - [`AnthropicProvider`] - Claude API (Anthropic)
//! - [`OpenAiProvider`] - GPT-4, GPT-3.5 (`OpenAI`)
//! - [`GeminiProvider`] - Gemini Pro (Google)
//! - [`PerplexityProvider`] - Perplexity AI
//! - [`QwenProvider`] - Qwen models (Alibaba Cloud)
//! - [`OpenRouterProvider`] - Multi-provider aggregator
//! - [`MinMaxProvider`] - `MinMax` AI
//! - [`GrokProvider`] - Grok (xAI)
//! - [`MistralProvider`] - Mistral AI
//! - [`DeepSeekProvider`] - `DeepSeek` AI
//! - [`BaiduProvider`] - Baidu ERNIE
//! - [`ZhipuProvider`] - Zhipu GLM
//! - [`KimiProvider`] - Kimi (Moonshot AI)
//! - [`ApertusProvider`] - Apertus (Switzerland, EU digital sovereignty)
//!
//! ## Gateway Providers
//! - [`KongProvider`] - Kong AI Gateway (routes to any upstream model with governance)
//!
//! ## Local Providers
//! - [`OllamaProvider`] - Local models via Ollama (Qwen, Llama, Mistral, etc.)
//!
//! # Prompt Structuring
//!
//! This crate provides provider-specific prompt structuring and optimization:
//!
//! - [`ProviderPromptBuilder`]: Builds prompts optimized for specific providers
//! - [`StructuredResponseParser`]: Parses structured responses (XML/JSON)
//! - Helper functions: [`build_claude_prompt`], [`build_openai_prompt`]
//!
//! # Examples
//!
//! ## Using Anthropic (Claude)
//!
//! ```ignore
//! use converge_provider::{AnthropicProvider, build_claude_prompt, StructuredResponseParser};
//! use crate::provider_api::{LlmProvider, LlmRequest};
//! use converge_core::prompt::{AgentRole, OutputContract, PromptContext};
//! use converge_core::context::ContextKey;
//!
//! let provider = AnthropicProvider::from_env("claude-sonnet-4-6")?;
//!
//! // Build optimized prompt with XML structure
//! let prompt = build_claude_prompt(
//!     AgentRole::Proposer,
//!     "extract-competitors",
//!     PromptContext::new(),
//!     OutputContract::new("proposed-fact", ContextKey::Competitors),
//!     vec![],
//! );
//!
//! let response = provider.complete(&LlmRequest::new(prompt))?;
//!
//! // Parse structured XML response
//! let proposals = StructuredResponseParser::parse_claude_xml(
//!     &response,
//!     ContextKey::Competitors,
//!     "anthropic",
//! );
//! ```
//!
//! ## Using `OpenAI`
//!
//! ```ignore
//! use converge_provider::OpenAiProvider;
//! use crate::provider_api::{LlmProvider, LlmRequest};
//!
//! let provider = OpenAiProvider::from_env("gpt-4")?;
//! let response = provider.complete(&LlmRequest::new("Hello!"))?;
//! ```
//!
//! ## Using `OpenRouter` (Multi-Provider)
//!
//! ```ignore
//! use converge_provider::OpenRouterProvider;
//! use crate::provider_api::{LlmProvider, LlmRequest};
//!
//! // Access any provider through OpenRouter
//! let provider = OpenRouterProvider::from_env("anthropic/claude-3-opus")?;
//! let response = provider.complete(&LlmRequest::new("Hello!"))?;
//! ```

// Secret management (SecretProvider trait, EnvSecretProvider default)
pub mod secret;

// LLM provider invocation types (migrated from converge-traits 0.1)
pub mod provider_api;

// Fallback-aware provider wrappers
pub mod fallback;

// Convergence agent wrapper for LLM providers
pub mod convergence;

// Core contract types
pub mod contract;

// LLM Backend implementations (unified LlmBackend trait from converge-core)
pub mod llm;

// LLM providers (simple LlmProvider trait)
#[cfg(feature = "anthropic")]
mod anthropic;
#[cfg(feature = "apertus")]
mod apertus;
#[cfg(feature = "baidu")]
mod baidu;

// Search providers
#[cfg(feature = "brave")]
pub mod brave;

// OCR / Document AI providers
mod capability_registry;
mod common;
#[cfg(feature = "deepseek")]
mod deepseek;
mod factory;
mod fake;
#[cfg(feature = "gemini")]
mod gemini;
#[cfg(feature = "grok")]
mod grok;
#[cfg(feature = "kimi")]
mod kimi;
#[cfg(feature = "kong")]
pub mod kong;
#[cfg(feature = "minmax")]
mod minmax;
#[cfg(feature = "mistral")]
mod mistral;
mod model_selection;
pub mod ocr;
#[cfg(feature = "ollama")]
mod ollama;
#[cfg(feature = "openai")]
mod openai;
#[cfg(feature = "openai")]
mod openrouter;
#[cfg(feature = "perplexity")]
mod perplexity;
mod prompt;
#[cfg(feature = "qwen")]
mod qwen;
#[cfg(feature = "zhipu")]
mod zhipu;

// Patent providers
#[cfg(feature = "patent")]
pub mod patent;

// Tool integration (MCP, OpenAPI, GraphQL)
pub mod tools;

// LinkedIn providers
#[cfg(feature = "linkedin")]
mod linkedin;

// Capability providers
pub mod embedding;
pub mod graph;
#[cfg(feature = "registry")]
pub mod registry_loader;
pub mod reranker;
pub mod vector;

// Re-exports
#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicProvider;
#[cfg(feature = "apertus")]
pub use apertus::ApertusProvider;
#[cfg(feature = "baidu")]
pub use baidu::BaiduProvider;
pub use capability_registry::{
    CapabilityRegistry, CapabilityRequirements, SearchProviderMeta, WebSearchRequirements,
};
pub use common::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChatUsage, HttpProviderConfig,
    OpenAiCompatibleProvider, OpenAiStyleError, OpenAiStyleErrorDetail,
    chat_response_to_llm_response, handle_openai_style_error, make_chat_completion_request,
    parse_finish_reason,
};
#[cfg(feature = "deepseek")]
pub use deepseek::DeepSeekProvider;
pub use factory::{
    can_create_provider, create_provider, create_provider_with_secrets, create_tool_aware_provider,
};
pub use fallback::{FallbackLlmProvider, try_with_fallback};
#[cfg(feature = "gemini")]
pub use gemini::GeminiProvider;
#[cfg(feature = "grok")]
pub use grok::GrokProvider;
#[cfg(feature = "kimi")]
pub use kimi::KimiProvider;
#[cfg(feature = "kong")]
pub use kong::{KongGateway, KongProvider, KongRoute};
#[cfg(feature = "minmax")]
pub use minmax::MinMaxProvider;
#[cfg(feature = "mistral")]
pub use mistral::MistralProvider;
pub use model_selection::{
    FitnessBreakdown, ModelMetadata, ModelSelector, ProviderRegistry, RejectionReason,
    SelectionResult, is_brave_available, is_provider_available,
};
#[cfg(feature = "ollama")]
pub use ollama::{
    DEFAULT_OLLAMA_URL, ModelInfo as OllamaModelInfo, ModelListEntry as OllamaModelEntry,
    OllamaProvider,
};
#[cfg(feature = "openai")]
pub use openai::OpenAiProvider;
#[cfg(feature = "openai")]
pub use openrouter::OpenRouterProvider;
#[cfg(feature = "perplexity")]
pub use perplexity::PerplexityProvider;
pub use prompt::{
    ProviderPromptBuilder, StructuredResponseParser, build_claude_prompt, build_openai_prompt,
};
#[cfg(feature = "qwen")]
pub use qwen::QwenProvider;
#[cfg(feature = "zhipu")]
pub use zhipu::ZhipuProvider;

// Secret management
pub use secret::{
    EnvSecretProvider, SecretError, SecretProvider, SecretString, StaticSecretProvider,
};

// Testing utilities
pub use fake::FakeProvider;

// Provider API types (LlmProvider trait, request/response, errors)
pub use provider_api::{
    AgentRequirements, FinishReason, LlmError, LlmErrorKind, LlmProvider, LlmRequest, LlmResponse,
    ModelSelectorTrait, TokenUsage as LlmTokenUsage,
};

// Convergence integration (Backend + Suggestor wrappers)
pub use convergence::LlmAgent;

// Contract types (re-exported for convenience)
pub use contract::{
    CallTimer, Capability, ProviderCallContext, ProviderMeta, ProviderObservation, Region,
    TokenUsage, canonical_hash,
};

// LLM Backend (unified LlmBackend trait implementations)
#[cfg(feature = "anthropic")]
pub use llm::AnthropicBackend;

// Patent providers
#[cfg(feature = "patent")]
pub use patent::{
    CompositePatentProvider, PatentOperator, PatentSearchProvider, PatentSearchRequest,
    PatentSearchResponse, PatentSearchResult, StubPatentProvider,
};

// LinkedIn providers
#[cfg(feature = "linkedin")]
pub use linkedin::{
    LinkedInApiProvider, LinkedInGetRequest, LinkedInProvider, StubLinkedInProvider,
};

// Search providers
#[cfg(feature = "brave")]
pub use brave::{
    BraveCapability, BraveSearchError, BraveSearchProvider, BraveSearchRequest,
    BraveSearchResponse, BraveSearchResult,
};

// OCR / Document AI providers
pub use ocr::{
    DeepSeekOcrProvider,
    LightOnOcrProvider,
    // Cloud providers
    MistralOcrProvider,
    OcrConfidence,
    OcrError,
    OcrImage,
    OcrInput,
    OcrOutputFormat,
    OcrPreprocessing,
    // Provenance & tracing
    OcrProvenance,
    // Core types
    OcrProvider,
    OcrRequest,
    OcrResult,
    OcrSpan,
    OcrTable,
    TesseractConfig,
    // Local providers (stubs for now)
    TesseractOcrProvider,
    TesseractOutputFormat,
    compute_hash,
    with_trace_hashes,
};

// Tool integration
pub use tools::{
    GraphQlConfig, GraphQlConverter, GraphQlOperationType, InlineToolConfig, InputSchema,
    McpClient, McpClientBuilder, McpServerConfig, McpTransport, McpTransportType, OpenApiConfig,
    OpenApiConverter, ParsedToolCall, SourceFilter, ToolAwareProvider, ToolAwareResponse, ToolCall,
    ToolDefinition, ToolError, ToolErrorKind, ToolFormat, ToolHandler, ToolRegistry, ToolResult,
    ToolResultContent, ToolSource, ToolsConfig, ToolsConfigError,
};
