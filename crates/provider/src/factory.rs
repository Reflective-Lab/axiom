// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Provider factory for creating providers from runtime selection.

#[cfg(feature = "anthropic")]
use crate::AnthropicProvider;
#[cfg(feature = "apertus")]
use crate::ApertusProvider;
#[cfg(feature = "baidu")]
use crate::BaiduProvider;
#[cfg(feature = "deepseek")]
use crate::DeepSeekProvider;
#[cfg(feature = "gemini")]
use crate::GeminiProvider;
#[cfg(feature = "grok")]
use crate::GrokProvider;
#[cfg(feature = "kimi")]
use crate::KimiProvider;
#[cfg(feature = "minmax")]
use crate::MinMaxProvider;
#[cfg(feature = "mistral")]
use crate::MistralProvider;
#[cfg(feature = "perplexity")]
use crate::PerplexityProvider;
#[cfg(feature = "qwen")]
use crate::QwenProvider;
#[cfg(feature = "zhipu")]
use crate::ZhipuProvider;
#[cfg(feature = "kong")]
use crate::kong::{KongProvider, KongRoute};
use crate::provider_api::{LlmError, LlmProvider};
use crate::secret::SecretProvider;
use crate::tools::{ToolAwareProvider, ToolFormat, ToolRegistry};
#[cfg(feature = "openai")]
use crate::{OpenAiProvider, OpenRouterProvider};
use std::sync::Arc;

/// Creates a provider instance from provider name and model ID.
///
/// This factory is used after runtime model selection to instantiate
/// the actual provider.
///
/// # Errors
///
/// Returns error if:
/// - Provider name is unknown
/// - Required environment variables are not set
/// - Provider creation fails
pub fn create_provider(
    provider_name: &str,
    model_id: &str,
) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match provider_name {
        #[cfg(feature = "anthropic")]
        "anthropic" => {
            let provider = AnthropicProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "openai")]
        "openai" => {
            let provider = OpenAiProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "gemini")]
        "gemini" => {
            let provider = GeminiProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "perplexity")]
        "perplexity" => {
            let provider = PerplexityProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "openai")]
        "openrouter" => {
            let provider = OpenRouterProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "qwen")]
        "qwen" => {
            let provider = QwenProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "minmax")]
        "minmax" => {
            let provider = MinMaxProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "grok")]
        "grok" => {
            let provider = GrokProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "mistral")]
        "mistral" => {
            let provider = MistralProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "deepseek")]
        "deepseek" => {
            let provider = DeepSeekProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "baidu")]
        "baidu" => {
            let provider = BaiduProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "zhipu")]
        "zhipu" => {
            let provider = ZhipuProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "kimi")]
        "kimi" => {
            let provider = KimiProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "apertus")]
        "apertus" => {
            let provider = ApertusProvider::from_env(model_id)?;
            Ok(Arc::new(provider))
        }
        #[cfg(feature = "kong")]
        "kong" => {
            let route = KongRoute::new(model_id);
            let provider = KongProvider::from_env(route)?;
            Ok(Arc::new(provider))
        }
        _ => Err(LlmError::provider(format!(
            "Unknown or disabled provider: {provider_name}"
        ))),
    }
}

/// Creates a provider using a custom `SecretProvider` for key loading.
///
/// This allows production deployments to load keys from Vault, GCP
/// Secret Manager, or any other backend.
///
/// # Errors
///
/// Returns error if the provider name is unknown or secret loading fails.
pub fn create_provider_with_secrets(
    provider_name: &str,
    model_id: &str,
    secrets: &dyn SecretProvider,
) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match provider_name {
        #[cfg(feature = "anthropic")]
        "anthropic" => Ok(Arc::new(AnthropicProvider::from_secret_provider(
            secrets, model_id,
        )?)),
        #[cfg(feature = "openai")]
        "openai" => Ok(Arc::new(OpenAiProvider::from_secret_provider(
            secrets, model_id,
        )?)),
        #[cfg(feature = "gemini")]
        "gemini" => Ok(Arc::new(GeminiProvider::from_secret_provider(
            secrets, model_id,
        )?)),
        #[cfg(feature = "qwen")]
        "qwen" => Ok(Arc::new(QwenProvider::from_secret_provider(
            secrets, model_id,
        )?)),
        #[cfg(feature = "baidu")]
        "baidu" => Ok(Arc::new(BaiduProvider::from_secret_provider(
            secrets, model_id,
        )?)),
        // Providers without from_secret_provider fall back to from_env
        _ => create_provider(provider_name, model_id),
    }
}

/// Checks if a provider can be created (has required API keys).
///
/// Returns `true` if the provider can be instantiated.
#[must_use]
pub fn can_create_provider(provider_name: &str) -> bool {
    match provider_name {
        #[cfg(feature = "anthropic")]
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
        #[cfg(feature = "openai")]
        "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
        #[cfg(feature = "gemini")]
        "gemini" => std::env::var("GEMINI_API_KEY").is_ok(),
        #[cfg(feature = "perplexity")]
        "perplexity" => std::env::var("PERPLEXITY_API_KEY").is_ok(),
        #[cfg(feature = "openai")]
        "openrouter" => std::env::var("OPENROUTER_API_KEY").is_ok(),
        #[cfg(feature = "qwen")]
        "qwen" => std::env::var("QWEN_API_KEY").is_ok(),
        #[cfg(feature = "minmax")]
        "minmax" => std::env::var("MINMAX_API_KEY").is_ok(),
        #[cfg(feature = "grok")]
        "grok" => std::env::var("GROK_API_KEY").is_ok(),
        #[cfg(feature = "mistral")]
        "mistral" => std::env::var("MISTRAL_API_KEY").is_ok(),
        #[cfg(feature = "deepseek")]
        "deepseek" => std::env::var("DEEPSEEK_API_KEY").is_ok(),
        #[cfg(feature = "baidu")]
        "baidu" => {
            std::env::var("BAIDU_API_KEY").is_ok() && std::env::var("BAIDU_SECRET_KEY").is_ok()
        }
        #[cfg(feature = "zhipu")]
        "zhipu" => std::env::var("ZHIPU_API_KEY").is_ok(),
        #[cfg(feature = "kimi")]
        "kimi" => std::env::var("KIMI_API_KEY").is_ok(),
        #[cfg(feature = "apertus")]
        "apertus" => std::env::var("APERTUS_API_KEY").is_ok(),
        #[cfg(feature = "kong")]
        "kong" => {
            std::env::var("KONG_AI_GATEWAY_URL").is_ok() && std::env::var("KONG_API_KEY").is_ok()
        }
        _ => false,
    }
}

/// Creates a tool-aware provider that wraps an LLM provider with tool calling capabilities.
///
/// This combines an LLM provider with a tool registry to enable tool use in LLM interactions.
/// The returned provider can inject tool definitions into requests and parse tool calls from
/// responses.
///
/// # Errors
///
/// Returns error if the underlying provider creation fails.
pub fn create_tool_aware_provider(
    provider_name: &str,
    model_id: &str,
    registry: ToolRegistry,
) -> Result<ToolAwareProvider, LlmError> {
    let provider = create_provider(provider_name, model_id)?;
    let format = match provider_name {
        "anthropic" => ToolFormat::Anthropic,
        _ => ToolFormat::OpenAi,
    };
    Ok(ToolAwareProvider::with_shared(provider, registry).with_format(format))
}
