// Copyright 2024-2026 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Bridge between converge-provider LLM providers and converge-core LLM traits.

use converge_core::llm as core;
use converge_provider::provider_api as provider;
use tokio::runtime::Handle;
use tokio::task;

#[derive(Debug)]
pub struct ProviderBridge<P> {
    inner: P,
}

impl<P> ProviderBridge<P> {
    #[must_use]
    pub fn new(inner: P) -> Self {
        Self { inner }
    }
}

impl<P> core::LlmProvider for ProviderBridge<P>
where
    P: provider::LlmProvider + Send + Sync,
{
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    fn complete(&self, request: &core::LlmRequest) -> Result<core::LlmResponse, core::LlmError> {
        let provider_request = provider::LlmRequest {
            prompt: request.prompt.clone(),
            system: request.system.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop_sequences: request.stop_sequences.clone(),
        };

        let response = if Handle::try_current().is_ok() {
            task::block_in_place(|| self.inner.complete(&provider_request))
        } else {
            self.inner.complete(&provider_request)
        }
        .map_err(provider_error_to_core)?;

        Ok(core::LlmResponse {
            content: response.content,
            model: response.model,
            usage: core::TokenUsage {
                prompt_tokens: response.usage.prompt_tokens,
                completion_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
            finish_reason: match response.finish_reason {
                provider::FinishReason::Stop => core::FinishReason::Stop,
                provider::FinishReason::MaxTokens => core::FinishReason::MaxTokens,
                provider::FinishReason::StopSequence => core::FinishReason::StopSequence,
                provider::FinishReason::ContentFilter => core::FinishReason::ContentFilter,
            },
        })
    }
}

fn provider_error_to_core(err: provider::LlmError) -> core::LlmError {
    core::LlmError {
        kind: match err.kind {
            provider::LlmErrorKind::Authentication => core::LlmErrorKind::Authentication,
            provider::LlmErrorKind::RateLimit => core::LlmErrorKind::RateLimit,
            provider::LlmErrorKind::InvalidRequest => core::LlmErrorKind::InvalidRequest,
            provider::LlmErrorKind::ModelNotFound => core::LlmErrorKind::ModelNotFound,
            provider::LlmErrorKind::Network => core::LlmErrorKind::Network,
            provider::LlmErrorKind::ProviderError => core::LlmErrorKind::ProviderError,
            provider::LlmErrorKind::ParseError => core::LlmErrorKind::ParseError,
            provider::LlmErrorKind::Timeout => core::LlmErrorKind::Timeout,
        },
        message: err.message,
        retryable: err.retryable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use converge_core::llm::LlmProvider;
    use converge_provider::FakeProvider;

    #[test]
    fn bridge_adapts_provider_response() {
        let provider = FakeProvider::new().with_default_response("VALID");
        let bridge = ProviderBridge::new(provider);
        let response = bridge.complete(&core::LlmRequest::new("hello")).unwrap();
        assert_eq!(response.content, "VALID");
    }
}
