// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Observation types - raw provider output (evidence ledger).
//!
//! Observations are the first tier in the 3-tier hierarchy:
//! Observation -> Proposal -> Fact
//!
//! # Design
//!
//! - Observations are raw provider outputs (API responses, user input, etc.)
//! - They carry CaptureContext for auditability
//! - They reference raw payloads via ContentHash (not inline)
//! - "Providers return observations, never facts"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::id::{ContentHash, ObservationId, Timestamp};

// ============================================================================
// ObservationKind - Type of observation source
// ============================================================================

/// Kind of observation source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObservationKind {
    /// Response from an external API
    ApiResponse,
    /// Input from a human user
    UserInput,
    /// Event from the system (logs, metrics, etc.)
    SystemEvent,
    /// Data from an external feed (RSS, webhooks, etc.)
    ExternalFeed,
}

// ============================================================================
// CaptureContext - How the observation was captured
// ============================================================================

/// Context of how the observation was captured.
///
/// Carries enough information to understand and replay the capture conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureContext {
    /// Parameters used in the request (query params, body, etc.)
    pub request_params: serde_json::Value,
    /// Relevant environment variables (sanitized)
    pub environment: HashMap<String, String>,
    /// Session identifier for grouping related observations
    pub session_id: Option<String>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
}

impl Default for CaptureContext {
    fn default() -> Self {
        Self {
            request_params: serde_json::Value::Null,
            environment: HashMap::new(),
            session_id: None,
            correlation_id: None,
        }
    }
}

impl CaptureContext {
    /// Create a new empty capture context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add request parameters.
    pub fn with_request_params(mut self, params: serde_json::Value) -> Self {
        self.request_params = params;
        self
    }

    /// Add an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Set session ID.
    pub fn with_session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Set correlation ID.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

// ============================================================================
// ProviderIdentity - Who provided the observation
// ============================================================================

/// Provider identity and version.
///
/// Identifies the source of an observation for audit and version tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderIdentity {
    /// Provider name (e.g., "openai", "anthropic", "internal-metrics")
    pub name: String,
    /// Provider version or API version
    pub version: String,
    /// Adapter identifier if using an adapter layer
    pub adapter_id: Option<String>,
}

impl ProviderIdentity {
    /// Create a new provider identity.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            adapter_id: None,
        }
    }

    /// Set adapter ID.
    pub fn with_adapter(mut self, adapter_id: impl Into<String>) -> Self {
        self.adapter_id = Some(adapter_id.into());
        self
    }
}

// ============================================================================
// Observation - Raw provider output
// ============================================================================

/// Raw provider output - the evidence ledger.
///
/// Observations are immutable records of what a provider returned.
/// They do NOT become facts directly - they must go through the
/// proposal/promotion pipeline.
///
/// # Example
///
/// ```
/// use converge_core::types::{
///     Observation, ObservationKind, ObservationId, ContentHash,
///     CaptureContext, ProviderIdentity,
/// };
///
/// let obs = Observation::from_api_response(
///     ObservationId::new("obs-001"),
///     ContentHash::zero(),
///     ProviderIdentity::new("openai", "v1"),
///     CaptureContext::default(),
/// );
///
/// assert_eq!(obs.kind, ObservationKind::ApiResponse);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier for this observation
    pub id: ObservationId,
    /// What kind of source this came from
    pub kind: ObservationKind,
    /// Hash/pointer to the raw payload (content-addressed)
    pub raw_payload_ref: ContentHash,
    /// How the observation was captured
    pub capture_context: CaptureContext,
    /// Who provided this observation
    pub provider: ProviderIdentity,
    /// When this was captured
    pub captured_at: Timestamp,
}

impl Observation {
    /// Create an observation from an API response.
    pub fn from_api_response(
        id: ObservationId,
        raw_payload_ref: ContentHash,
        provider: ProviderIdentity,
        capture_context: CaptureContext,
    ) -> Self {
        Self {
            id,
            kind: ObservationKind::ApiResponse,
            raw_payload_ref,
            capture_context,
            provider,
            captured_at: Timestamp::now(),
        }
    }

    /// Create an observation from user input.
    pub fn from_user_input(
        id: ObservationId,
        raw_payload_ref: ContentHash,
        capture_context: CaptureContext,
    ) -> Self {
        Self {
            id,
            kind: ObservationKind::UserInput,
            raw_payload_ref,
            capture_context,
            provider: ProviderIdentity::new("user", "direct"),
            captured_at: Timestamp::now(),
        }
    }

    /// Create an observation from a system event.
    pub fn from_system_event(
        id: ObservationId,
        raw_payload_ref: ContentHash,
        capture_context: CaptureContext,
    ) -> Self {
        Self {
            id,
            kind: ObservationKind::SystemEvent,
            raw_payload_ref,
            capture_context,
            provider: ProviderIdentity::new("system", "internal"),
            captured_at: Timestamp::now(),
        }
    }

    /// Create an observation from an external feed.
    pub fn from_external_feed(
        id: ObservationId,
        raw_payload_ref: ContentHash,
        provider: ProviderIdentity,
        capture_context: CaptureContext,
    ) -> Self {
        Self {
            id,
            kind: ObservationKind::ExternalFeed,
            raw_payload_ref,
            capture_context,
            provider,
            captured_at: Timestamp::now(),
        }
    }

    /// Create a generic observation with explicit kind.
    pub fn new(
        id: ObservationId,
        kind: ObservationKind,
        raw_payload_ref: ContentHash,
        provider: ProviderIdentity,
        capture_context: CaptureContext,
        captured_at: Timestamp,
    ) -> Self {
        Self {
            id,
            kind,
            raw_payload_ref,
            capture_context,
            provider,
            captured_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_from_api_response() {
        let obs = Observation::from_api_response(
            ObservationId::new("obs-001"),
            ContentHash::zero(),
            ProviderIdentity::new("openai", "v1"),
            CaptureContext::default(),
        );

        assert_eq!(obs.id.as_str(), "obs-001");
        assert_eq!(obs.kind, ObservationKind::ApiResponse);
        assert_eq!(obs.provider.name, "openai");
    }

    #[test]
    fn capture_context_builder() {
        let ctx = CaptureContext::new()
            .with_request_params(serde_json::json!({"query": "test"}))
            .with_session_id("session-123")
            .with_correlation_id("corr-456");

        assert_eq!(ctx.session_id, Some("session-123".to_string()));
        assert_eq!(ctx.correlation_id, Some("corr-456".to_string()));
    }

    #[test]
    fn provider_identity_with_adapter() {
        let provider =
            ProviderIdentity::new("anthropic", "2024-01").with_adapter("llm/grounded@1.0.0");

        assert_eq!(provider.name, "anthropic");
        assert_eq!(provider.adapter_id, Some("llm/grounded@1.0.0".to_string()));
    }
}
