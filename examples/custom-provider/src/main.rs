// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Custom Provider — implement an LLM provider adapter.
//!
//! Shows: LlmProvider trait, request/response types, provider registration.
//! Partners use this pattern to integrate their own model backends.

use async_trait::async_trait;
use converge_provider::provider_api::{LlmProvider, LlmRequest, LlmResponse, ProviderError};
use converge_traits::{Backend, BackendKind, Capability};

/// A mock provider that echoes prompts — replace with your real API client.
struct EchoProvider {
    model_name: String,
}

impl EchoProvider {
    fn new(model: &str) -> Self {
        Self {
            model_name: model.to_string(),
        }
    }
}

impl Backend for EchoProvider {
    fn kind(&self) -> BackendKind {
        BackendKind::Llm
    }

    fn name(&self) -> &str {
        &self.model_name
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::TextGeneration, Capability::Chat]
    }
}

#[async_trait]
impl LlmProvider for EchoProvider {
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, ProviderError> {
        // Your real implementation calls an API here
        Ok(LlmResponse {
            text: format!("Echo: {}", request.prompt),
            model: self.model_name.clone(),
            input_tokens: request.prompt.len() as u32,
            output_tokens: request.prompt.len() as u32,
            ..Default::default()
        })
    }
}

#[tokio::main]
async fn main() {
    println!("=== Custom Provider Example ===\n");

    let provider = EchoProvider::new("echo-v1");

    println!("Provider: {} ({:?})", provider.name(), provider.kind());
    println!("Capabilities: {:?}\n", provider.capabilities());

    let request = LlmRequest {
        prompt: "What is the convergence model?".to_string(),
        ..Default::default()
    };

    match provider.generate(&request).await {
        Ok(response) => {
            println!("Response: {}", response.text);
            println!(
                "Tokens:   {} in / {} out",
                response.input_tokens, response.output_tokens
            );
        }
        Err(e) => println!("Error: {e}"),
    }

    println!("\n=== Done ===");
}
