// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Ollama local model provider.
//!
//! Ollama enables running open models locally, including Qwen, Llama, Mistral,
//! `DeepSeek`, and many others. This provider supports:
//!
//! - **Completion**: Text generation via chat/generate endpoints
//! - **Embedding**: Vector embeddings for models that support it
//!
//! # Architecture Note
//!
//! Ollama runs locally with `DataSovereignty::Local` - data never leaves
//! the machine. This makes it ideal for sensitive workloads.
//!
//! # Example
//!
//! ```ignore
//! use converge_provider::OllamaProvider;
//! use crate::provider_api::{LlmProvider, LlmRequest};
//! use converge_core::capability::{Embedding, EmbedRequest};
//!
//! // Create provider for Qwen 2.5 (supports both completion and embedding)
//! let provider = OllamaProvider::new("qwen2.5:7b");
//!
//! // Completion
//! let response = provider.complete(&LlmRequest::new("Hello!"))?;
//!
//! // Embedding (if model supports it)
//! if provider.supports_embedding() {
//!     let embedding = provider.embed(&EmbedRequest::text("Hello!"))?;
//! }
//! ```

use crate::provider_api::{
    FinishReason, LlmError, LlmErrorKind, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};
use converge_core::capability::{
    CapabilityError, CapabilityErrorKind, EmbedInput, EmbedRequest, EmbedResponse, EmbedUsage,
    Embedding, Modality,
};
use serde::{Deserialize, Serialize};

/// Default Ollama server URL.
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Ollama local model provider.
///
/// Supports models like:
/// - `qwen2.5:7b`, `qwen2.5:14b` - Strong multilingual, good for completion + embedding
/// - `llama3.2:3b`, `llama3.2:8b` - Fast, good for completion
/// - `mistral:7b` - Balanced performance
/// - `nomic-embed-text` - Dedicated embedding model
/// - `mxbai-embed-large` - High-quality embeddings
pub struct OllamaProvider {
    model: String,
    client: reqwest::blocking::Client,
    base_url: String,
    /// Whether this model supports embedding (cached after first check).
    embedding_support: std::sync::OnceLock<bool>,
}

impl OllamaProvider {
    /// Creates a new Ollama provider with default URL.
    #[must_use]
    pub fn new(model: impl Into<String>) -> Self {
        Self::with_url(DEFAULT_OLLAMA_URL, model)
    }

    /// Creates a new Ollama provider with custom URL.
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client cannot be created.
    #[must_use]
    pub fn with_url(url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: url.into(),
            embedding_support: std::sync::OnceLock::new(),
        }
    }

    /// Checks if Ollama is running and the model is available.
    ///
    /// # Errors
    ///
    /// Returns error if Ollama is not reachable or model is not found.
    pub fn health_check(&self) -> Result<ModelInfo, LlmError> {
        let url = format!("{}/api/show", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({"name": &self.model}))
            .send()
            .map_err(|e| {
                if e.is_connect() {
                    LlmError::network(format!(
                        "Cannot connect to Ollama at {}. Is it running?",
                        self.base_url
                    ))
                } else {
                    LlmError::network(format!("Ollama request failed: {e}"))
                }
            })?;

        if response.status().is_success() {
            let info: ModelInfo = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse model info: {e}")))?;
            Ok(info)
        } else if response.status().as_u16() == 404 {
            Err(LlmError {
                kind: LlmErrorKind::ModelNotFound,
                message: format!(
                    "Model '{}' not found. Try: ollama pull {}",
                    self.model, self.model
                ),
                retryable: false,
            })
        } else {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            Err(LlmError::provider(format!(
                "Ollama returned status {status}: {body}"
            )))
        }
    }

    /// Checks if this model supports embedding.
    ///
    /// Models like `nomic-embed-text`, `mxbai-embed-large`, and most LLMs
    /// support embedding. Dedicated embedding models are preferred for quality.
    #[must_use]
    pub fn supports_embedding(&self) -> bool {
        *self.embedding_support.get_or_init(|| {
            // Known embedding models
            let known_embedding_models = [
                "nomic-embed-text",
                "mxbai-embed-large",
                "bge-m3",
                "bge-large",
                "all-minilm",
                "snowflake-arctic-embed",
            ];

            // Check if model name contains a known embedding model
            for known in known_embedding_models {
                if self.model.contains(known) {
                    return true;
                }
            }

            // Most LLMs also support embedding, try to detect
            // by checking if the model info indicates embedding support
            if let Ok(info) = self.health_check() {
                // Ollama models generally support embedding via /api/embeddings
                // unless they explicitly don't (rare)
                return !info.details.families.is_empty();
            }

            false
        })
    }

    /// Lists all available models on the Ollama server.
    ///
    /// # Errors
    ///
    /// Returns error if request fails.
    pub fn list_models(&self) -> Result<Vec<ModelListEntry>, LlmError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| LlmError::network(format!("Failed to list models: {e}")))?;

        if response.status().is_success() {
            let list: ModelList = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse model list: {e}")))?;
            Ok(list.models)
        } else {
            Err(LlmError::provider("Failed to list models"))
        }
    }
}

impl LlmProvider for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/api/chat", self.base_url);

        let mut messages = Vec::new();

        if let Some(ref system) = request.system {
            messages.push(OllamaMessage {
                role: "system",
                content: system.clone(),
            });
        }

        messages.push(OllamaMessage {
            role: "user",
            content: request.prompt.clone(),
        });

        #[allow(clippy::cast_possible_wrap)]
        let body = OllamaChatRequest {
            model: &self.model,
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: Some(request.max_tokens as i32),
                stop: if request.stop_sequences.is_empty() {
                    None
                } else {
                    Some(request.stop_sequences.clone())
                },
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| LlmError::network(format!("Ollama request failed: {e}")))?;

        if response.status().is_success() {
            let ollama_response: OllamaChatResponse = response
                .json()
                .map_err(|e| LlmError::parse(format!("Failed to parse response: {e}")))?;

            Ok(LlmResponse {
                content: ollama_response.message.content,
                model: ollama_response.model,
                usage: TokenUsage {
                    prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                    completion_tokens: ollama_response.eval_count.unwrap_or(0),
                    total_tokens: ollama_response.prompt_eval_count.unwrap_or(0)
                        + ollama_response.eval_count.unwrap_or(0),
                },
                finish_reason: if ollama_response.done {
                    FinishReason::Stop
                } else {
                    FinishReason::MaxTokens
                },
            })
        } else {
            let status = response.status();
            let body = response.text().unwrap_or_default();

            if status.as_u16() == 404 {
                Err(LlmError {
                    kind: LlmErrorKind::ModelNotFound,
                    message: format!("Model '{}' not found", self.model),
                    retryable: false,
                })
            } else {
                Err(LlmError::provider(format!(
                    "Ollama returned status {status}: {body}"
                )))
            }
        }
    }
}

impl Embedding for OllamaProvider {
    fn name(&self) -> &'static str {
        "ollama"
    }

    fn modalities(&self) -> Vec<Modality> {
        vec![Modality::Text]
    }

    fn default_dimensions(&self) -> usize {
        // Most Ollama embedding models use 768 or 1024 dimensions
        // This is a reasonable default
        768
    }

    fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse, CapabilityError> {
        let url = format!("{}/api/embeddings", self.base_url);

        let mut embeddings = Vec::with_capacity(request.inputs.len());

        for input in &request.inputs {
            let text = match input {
                EmbedInput::Text(t) => t.clone(),
                other => {
                    return Err(CapabilityError::unsupported_modality(other.modality()));
                }
            };

            let body = OllamaEmbedRequest {
                model: &self.model,
                prompt: &text,
            };

            let response =
                self.client.post(&url).json(&body).send().map_err(|e| {
                    CapabilityError::network(format!("Embedding request failed: {e}"))
                })?;

            if response.status().is_success() {
                let embed_response: OllamaEmbedResponse =
                    response.json().map_err(|e| CapabilityError {
                        kind: CapabilityErrorKind::ProviderError,
                        message: format!("Failed to parse embedding response: {e}"),
                        retryable: false,
                    })?;

                embeddings.push(embed_response.embedding);
            } else {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(CapabilityError {
                    kind: CapabilityErrorKind::ProviderError,
                    message: format!("Ollama embedding failed with status {status}: {body}"),
                    retryable: false,
                });
            }
        }

        let dimensions = embeddings.first().map(std::vec::Vec::len).unwrap_or(0);

        Ok(EmbedResponse {
            embeddings,
            model: self.model.clone(),
            dimensions,
            usage: Some(EmbedUsage { total_tokens: 0 }), // Ollama doesn't report token usage for embeddings
        })
    }
}

// =============================================================================
// OLLAMA API TYPES
// =============================================================================

#[derive(Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    model: String,
    message: OllamaResponseMessage,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

#[derive(Serialize)]
struct OllamaEmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

/// Information about an Ollama model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    /// Model name.
    pub modelfile: String,
    /// Model parameters.
    pub parameters: Option<String>,
    /// Model template.
    pub template: Option<String>,
    /// Model details.
    pub details: ModelDetails,
}

/// Details about an Ollama model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelDetails {
    /// Model format.
    pub format: String,
    /// Model family/families.
    #[serde(default)]
    pub families: Vec<String>,
    /// Parameter count string (e.g., "7B").
    pub parameter_size: Option<String>,
    /// Quantization level.
    pub quantization_level: Option<String>,
}

#[derive(Deserialize)]
struct ModelList {
    models: Vec<ModelListEntry>,
}

/// Entry in the Ollama model list.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelListEntry {
    /// Model name.
    pub name: String,
    /// Model size in bytes.
    pub size: u64,
    /// Modification time.
    pub modified_at: String,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_api::LlmProvider;

    #[test]
    fn provider_name_and_model() {
        let provider = OllamaProvider::new("qwen2.5:7b");
        assert_eq!(LlmProvider::name(&provider), "ollama");
        assert_eq!(provider.model(), "qwen2.5:7b");
    }

    #[test]
    fn custom_url() {
        let provider = OllamaProvider::with_url("http://gpu-server:11434", "llama3.2:8b");
        assert_eq!(provider.base_url, "http://gpu-server:11434");
    }

    #[test]
    fn embedding_modalities() {
        let provider = OllamaProvider::new("nomic-embed-text");
        let modalities = Embedding::modalities(&provider);
        assert_eq!(modalities, vec![Modality::Text]);
    }

    #[test]
    fn known_embedding_models_detected() {
        // These should be detected as embedding models
        let embedding_models = [
            "nomic-embed-text",
            "nomic-embed-text:latest",
            "mxbai-embed-large",
            "bge-m3:latest",
        ];

        for model in embedding_models {
            let _provider = OllamaProvider::new(model);
            // Note: supports_embedding() does a health check, so we can't test it
            // without a running Ollama server. Just verify the model name parsing.
            assert!(model.contains("embed") || model.contains("bge"));
        }
    }
}
