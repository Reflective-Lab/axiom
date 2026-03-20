// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! `LinkedIn` Provider - Stub implementation for testing.

use crate::contract::{ProviderCallContext, ProviderObservation, canonical_hash};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `LinkedIn` search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInGetRequest {
    pub endpoint: String,
    pub query: HashMap<String, String>,
}

impl LinkedInGetRequest {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            query: HashMap::new(),
        }
    }
}

/// `LinkedIn` search result content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInProfile {
    pub profile_id: String,
    pub name: String,
    pub title: Option<String>,
    pub company: Option<String>,
    pub payload: serde_json::Value,
}

/// `LinkedIn` search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInGetResponse {
    pub records: Vec<ProviderObservation<LinkedInProfile>>,
}

/// `LinkedIn` provider trait.
pub trait LinkedInProvider: Send + Sync {
    fn name(&self) -> &str;
    fn get(
        &self,
        request: &LinkedInGetRequest,
        ctx: &ProviderCallContext,
    ) -> Result<LinkedInGetResponse, String>;
}

/// Real `LinkedIn` API provider (placeholder - uses stub responses for now).
#[derive(Debug, Clone, Default)]
pub struct LinkedInApiProvider;

impl LinkedInApiProvider {
    /// Create from environment variables.
    /// In a real implementation, this would read `LINKEDIN_API_KEY` etc.
    pub fn from_env() -> Result<Self, String> {
        // Placeholder: in production this would validate API credentials
        Ok(Self)
    }
}

impl LinkedInProvider for LinkedInApiProvider {
    fn name(&self) -> &'static str {
        "linkedin_api"
    }

    fn get(
        &self,
        request: &LinkedInGetRequest,
        _ctx: &ProviderCallContext,
    ) -> Result<LinkedInGetResponse, String> {
        // Placeholder implementation - in production this would call the LinkedIn API
        if request.endpoint.trim().is_empty() {
            return Err("Empty endpoint".to_string());
        }

        let profile = LinkedInProfile {
            profile_id: "LI-API-001".to_string(),
            name: "API User".to_string(),
            title: Some("Director".to_string()),
            company: Some("Tech Corp".to_string()),
            payload: serde_json::json!({
                "profile_id": "LI-API-001",
                "name": "API User",
                "title": "Director",
                "company": "Tech Corp"
            }),
        };

        let hash_input = format!("{}:{:?}", request.endpoint, request.query);
        let obs = ProviderObservation {
            observation_id: format!("obs:linkedin:{}", canonical_hash(&hash_input)),
            request_hash: canonical_hash(&hash_input),
            vendor: "linkedin_api".to_string(),
            model: "api".to_string(),
            latency_ms: 100,
            cost_estimate: None,
            tokens: None,
            content: profile,
            raw_response: None,
        };

        Ok(LinkedInGetResponse { records: vec![obs] })
    }
}

/// Stub `LinkedIn` provider for testing.
#[derive(Debug, Clone, Default)]
pub struct StubLinkedInProvider;

impl LinkedInProvider for StubLinkedInProvider {
    fn name(&self) -> &'static str {
        "stub_linkedin"
    }

    fn get(
        &self,
        request: &LinkedInGetRequest,
        _ctx: &ProviderCallContext,
    ) -> Result<LinkedInGetResponse, String> {
        if request.endpoint.trim().is_empty() {
            return Err("Empty endpoint".to_string());
        }

        let profile = LinkedInProfile {
            profile_id: "LI-STUB-001".to_string(),
            name: "Jane Doe".to_string(),
            title: Some("VP Engineering".to_string()),
            company: Some("Acme Corp".to_string()),
            payload: serde_json::json!({
                "profile_id": "LI-STUB-001",
                "name": "Jane Doe",
                "title": "VP Engineering",
                "company": "Acme Corp"
            }),
        };

        let hash_input = format!("{}:{:?}", request.endpoint, request.query);
        let obs = ProviderObservation {
            observation_id: format!("obs:linkedin:{}", canonical_hash(&hash_input)),
            request_hash: canonical_hash(&hash_input),
            vendor: "stub_linkedin".to_string(),
            model: "stub".to_string(),
            latency_ms: 10,
            cost_estimate: None,
            tokens: None,
            content: profile,
            raw_response: None,
        };

        Ok(LinkedInGetResponse { records: vec![obs] })
    }
}
