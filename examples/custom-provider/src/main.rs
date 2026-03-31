// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Custom Provider — implement an LLM provider adapter.
//!
//! Shows: LlmProvider trait, request/response types, provider registration.
//! Partners use this pattern to integrate their own model backends.

use converge_provider::provider_api::{
    FinishReason, LlmError, LlmErrorKind, LlmProvider, LlmRequest, LlmResponse, TokenUsage,
};

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

impl LlmProvider for EchoProvider {
    fn name(&self) -> &'static str {
        "echo-provider"
    }

    fn model(&self) -> &str {
        &self.model_name
    }

    fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: format!("Echo: {}", request.prompt),
            model: self.model_name.clone(),
            usage: TokenUsage {
                prompt_tokens: request.prompt.len() as u32,
                completion_tokens: request.prompt.len() as u32,
                total_tokens: (request.prompt.len() * 2) as u32,
            },
            finish_reason: FinishReason::Stop,
        })
    }
}

fn main() {
    println!("=== Custom Provider Example ===\n");

    let provider = EchoProvider::new("echo-v1");

    println!(
        "Provider: {} (model: {})",
        provider.name(),
        provider.model()
    );

    let request = LlmRequest::new("What is the convergence model?");

    match provider.complete(&request) {
        Ok(response) => {
            println!("Response: {}", response.content);
            println!(
                "Tokens:   {} in / {} out",
                response.usage.prompt_tokens, response.usage.completion_tokens
            );
        }
        Err(e) => println!("Error: {e}"),
    }

    println!("\n=== Done ===");
}
