// Copyright 2024-2025 Aprio One AB, Sweden
// SPDX-License-Identifier: MIT

//! Patent search provider contracts and stub implementation.
//!
//! Providers return observations with provenance; agents decide.

use crate::contract::{CallTimer, ProviderCallContext, ProviderObservation, canonical_hash};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use converge_core::capability::CapabilityError;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Supported patent search operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatentOperator {
    Uspto,
    Epo,
    Wipo,
    GooglePatents,
    Lens,
}

impl PatentOperator {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Uspto => "uspto",
            Self::Epo => "epo",
            Self::Wipo => "wipo",
            Self::GooglePatents => "google_patents",
            Self::Lens => "lens",
        }
    }
}

/// Patent search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatentSearchRequest {
    pub query: String,
    pub operators: Vec<PatentOperator>,
    pub include_paid: bool,
    pub account_id: Option<String>,
    pub filters: serde_json::Value,
}

impl PatentSearchRequest {
    #[must_use]
    pub fn new(query: impl Into<String>, operators: Vec<PatentOperator>) -> Self {
        Self {
            query: query.into(),
            operators,
            include_paid: false,
            account_id: None,
            filters: serde_json::json!({}),
        }
    }
}

/// A paid action offered by a provider (e.g., bulk export).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaidAction {
    pub action_id: String,
    pub operator: PatentOperator,
    pub description: String,
    pub estimated_cost_usd: f64,
    pub requires_approval: bool,
}

/// A single patent search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatentSearchResult {
    pub publication_id: String,
    pub title: String,
    pub abstract_text: String,
    pub operator: PatentOperator,
    pub url: String,
    pub raw_source: serde_json::Value,
}

/// Response from a patent search provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatentSearchResponse {
    pub results: Vec<ProviderObservation<PatentSearchResult>>,
    pub paid_actions: Vec<PaidAction>,
}

/// Provider trait for patent search.
pub trait PatentSearchProvider: Send + Sync {
    fn name(&self) -> &str;

    /// Execute a patent search request.
    ///
    /// # Errors
    ///
    /// Returns error when the provider fails or request is invalid.
    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError>;
}

fn extract_results_array(payload: &serde_json::Value) -> Vec<&serde_json::Value> {
    let keys = ["results", "data", "patents", "documents", "items"];
    for key in keys {
        if let Some(arr) = payload.get(key).and_then(|v| v.as_array()) {
            return arr.iter().collect();
        }
    }
    Vec::new()
}

fn pick_string(payload: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = payload.get(*key)
            && let Some(text) = value.as_str()
            && !text.trim().is_empty()
        {
            return Some(text.to_string());
        }
    }
    None
}

// =============================================================================
// PatentsView Provider (USPTO-backed)
// =============================================================================

#[derive(Debug, Clone)]
pub struct PatentsViewProvider {
    client: Client,
    base_url: String,
}

impl PatentsViewProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    #[must_use]
    pub fn from_env() -> Self {
        let base_url = std::env::var("PATENTSVIEW_BASE_URL")
            .unwrap_or_else(|_| "https://api.patentsview.org/patents/query".to_string());
        Self::new(base_url)
    }

    fn build_query(&self, query: &str) -> String {
        let trimmed = query.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            trimmed.to_string()
        } else {
            serde_json::json!({
                "_text_any": {
                    "patent_title": trimmed,
                    "patent_abstract": trimmed
                }
            })
            .to_string()
        }
    }
}

impl PatentSearchProvider for PatentsViewProvider {
    fn name(&self) -> &'static str {
        "patentsview"
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        if request.query.trim().is_empty() {
            return Err(CapabilityError::invalid_input("query cannot be empty"));
        }

        let timer = CallTimer::start();
        let q = self.build_query(&request.query);
        let fields = serde_json::json!(["patent_number", "patent_title", "patent_abstract"]);

        let response = self
            .client
            .get(&self.base_url)
            .query(&[("q", q.as_str()), ("f", fields.to_string().as_str())])
            .send()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        if !status.is_success() {
            return Err(CapabilityError::invalid_input(format!(
                "PatentsView error: {status}"
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CapabilityError::invalid_input(e.to_string()))?;
        let patents = parsed
            .get("patents")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for patent in patents {
            let number = patent
                .get("patent_number")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let title = patent
                .get("patent_title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let abstract_text = patent
                .get("patent_abstract")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let result = PatentSearchResult {
                publication_id: number.to_string(),
                title: title.to_string(),
                abstract_text: abstract_text.to_string(),
                operator: PatentOperator::Uspto,
                url: format!("https://patentsview.org/patent/{number}"),
                raw_source: patent.clone(),
            };

            let observation =
                ProviderObservation::new(self.name(), "patentsview", result, timer.elapsed_ms())
                    .with_request_hash(canonical_hash(&format!(
                        "{}:{}:{}",
                        request.query, ctx.trace_id, number
                    )));
            results.push(observation);
        }

        Ok(PatentSearchResponse {
            results,
            paid_actions: Vec::new(),
        })
    }
}

// =============================================================================
// WIPO PATENTSCOPE Provider
// =============================================================================

#[derive(Debug, Clone)]
pub struct WipoPatentscopeProvider {
    client: Client,
    base_url: String,
    api_key: crate::secret::SecretString,
}

impl WipoPatentscopeProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: crate::secret::SecretString::new(api_key),
        }
    }

    /// Load configuration from environment.
    ///
    /// Required:
    /// - `WIPO_PATENTSCOPE_API_KEY`
    ///
    /// Optional:
    /// - `WIPO_PATENTSCOPE_BASE_URL`
    pub fn from_env() -> Result<Self, CapabilityError> {
        let base_url = std::env::var("WIPO_PATENTSCOPE_BASE_URL")
            .unwrap_or_else(|_| "https://patentscope.wipo.int/search/en/".to_string());
        let api_key = std::env::var("WIPO_PATENTSCOPE_API_KEY")
            .map_err(|_| CapabilityError::auth("WIPO_PATENTSCOPE_API_KEY missing"))?;
        Ok(Self::new(base_url, api_key))
    }
}

impl PatentSearchProvider for WipoPatentscopeProvider {
    fn name(&self) -> &'static str {
        "wipo_patentscope"
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        if request.query.trim().is_empty() {
            return Err(CapabilityError::invalid_input("query cannot be empty"));
        }

        let timer = CallTimer::start();
        let response = self
            .client
            .get(&self.base_url)
            .query(&[("q", request.query.as_str())])
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key.expose()))
            .header(ACCEPT, "application/json")
            .send()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        if !status.is_success() {
            return Err(CapabilityError::invalid_input(format!(
                "WIPO PATENTSCOPE error: {status}"
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CapabilityError::invalid_input(e.to_string()))?;
        let items = extract_results_array(&parsed);

        let mut results = Vec::new();
        for item in items {
            let publication_id = pick_string(
                item,
                &[
                    "publication_id",
                    "publicationNumber",
                    "doc_number",
                    "publication_number",
                ],
            )
            .unwrap_or_else(|| "unknown".to_string());
            let title = pick_string(item, &["title", "invention_title", "patent_title", "name"])
                .unwrap_or_else(|| "WIPO result".to_string());
            let abstract_text =
                pick_string(item, &["abstract", "abstract_text", "summary"]).unwrap_or_default();
            let url = pick_string(item, &["url", "link"]).unwrap_or_else(|| self.base_url.clone());

            let result = PatentSearchResult {
                publication_id: publication_id.clone(),
                title,
                abstract_text,
                operator: PatentOperator::Wipo,
                url,
                raw_source: item.clone(),
            };

            let observation = ProviderObservation::new(
                self.name(),
                "wipo_patentscope",
                result,
                timer.elapsed_ms(),
            )
            .with_request_hash(canonical_hash(&format!(
                "{}:{}:{}",
                request.query, ctx.trace_id, publication_id
            )));

            results.push(observation);
        }

        Ok(PatentSearchResponse {
            results,
            paid_actions: Vec::new(),
        })
    }
}

// =============================================================================
// EPO OPS Provider
// =============================================================================

#[derive(Debug, Clone)]
pub struct EpoOpsProvider {
    client: Client,
    base_url: String,
    token_url: String,
    consumer_key: String,
    consumer_secret: String,
    token_cache: Arc<Mutex<Option<String>>>,
}

impl EpoOpsProvider {
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        token_url: impl Into<String>,
        consumer_key: impl Into<String>,
        consumer_secret: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            token_url: token_url.into(),
            consumer_key: consumer_key.into(),
            consumer_secret: consumer_secret.into(),
            token_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_env() -> Result<Self, CapabilityError> {
        let base_url = std::env::var("EPO_OPS_BASE_URL").unwrap_or_else(|_| {
            "https://ops.epo.org/3.2/rest-services/published-data/search".to_string()
        });
        let token_url = std::env::var("EPO_OPS_TOKEN_URL")
            .unwrap_or_else(|_| "https://ops.epo.org/3.2/auth/accesstoken".to_string());
        let consumer_key = std::env::var("EPO_OPS_CONSUMER_KEY")
            .map_err(|_| CapabilityError::auth("EPO_OPS_CONSUMER_KEY missing"))?;
        let consumer_secret = std::env::var("EPO_OPS_CONSUMER_SECRET")
            .map_err(|_| CapabilityError::auth("EPO_OPS_CONSUMER_SECRET missing"))?;

        Ok(Self::new(
            base_url,
            token_url,
            consumer_key,
            consumer_secret,
        ))
    }

    fn fetch_token(&self) -> Result<String, CapabilityError> {
        let auth = BASE64.encode(format!("{}:{}", self.consumer_key, self.consumer_secret));
        let response = self
            .client
            .post(&self.token_url)
            .header(AUTHORIZATION, format!("Basic {auth}"))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e| CapabilityError::network(e.to_string()))?;
        if !status.is_success() {
            return Err(CapabilityError::invalid_input(format!(
                "EPO OPS token error: {status}"
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CapabilityError::invalid_input(e.to_string()))?;
        let token = parsed
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CapabilityError::invalid_input("EPO OPS token missing"))?;
        Ok(token.to_string())
    }

    fn get_token(&self) -> Result<String, CapabilityError> {
        if let Ok(cache) = self.token_cache.lock()
            && let Some(token) = cache.as_ref()
        {
            return Ok(token.clone());
        }

        let token = self.fetch_token()?;
        if let Ok(mut cache) = self.token_cache.lock() {
            *cache = Some(token.clone());
        }
        Ok(token)
    }

    fn cql_query(query: &str) -> String {
        format!("ti=\"{query}\" or ab=\"{query}\"")
    }

    fn extract_first(xml: &str, tag: &str) -> Option<String> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = xml.find(&open)? + open.len();
        let end = xml[start..].find(&close)? + start;
        Some(xml[start..end].to_string())
    }
}

impl PatentSearchProvider for EpoOpsProvider {
    fn name(&self) -> &'static str {
        "epo_ops"
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        if request.query.trim().is_empty() {
            return Err(CapabilityError::invalid_input("query cannot be empty"));
        }

        let token = self.get_token()?;
        let timer = CallTimer::start();
        let cql = Self::cql_query(&request.query);

        let response = self
            .client
            .get(&self.base_url)
            .query(&[("q", cql.as_str())])
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .header(ACCEPT, "application/xml")
            .send()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e| CapabilityError::network(e.to_string()))?;
        if !status.is_success() {
            return Err(CapabilityError::invalid_input(format!(
                "EPO OPS error: {status}"
            )));
        }

        let doc_number =
            Self::extract_first(&text, "doc-number").unwrap_or_else(|| "unknown".into());
        let title =
            Self::extract_first(&text, "invention-title").unwrap_or_else(|| "EPO result".into());

        let result = PatentSearchResult {
            publication_id: doc_number.clone(),
            title,
            abstract_text: String::new(),
            operator: PatentOperator::Epo,
            url: "https://ops.epo.org".to_string(),
            raw_source: serde_json::json!({
                "xml": text,
                "trace_id": ctx.trace_id,
            }),
        };

        let observation = ProviderObservation::new(self.name(), "ops", result, timer.elapsed_ms())
            .with_request_hash(canonical_hash(&format!(
                "{}:{}:{}",
                request.query, ctx.trace_id, doc_number
            )));

        Ok(PatentSearchResponse {
            results: vec![observation],
            paid_actions: Vec::new(),
        })
    }
}

// =============================================================================
// Lens Provider
// =============================================================================

#[derive(Debug, Clone)]
pub struct LensProvider {
    client: Client,
    base_url: String,
    api_key: crate::secret::SecretString,
}

impl LensProvider {
    #[must_use]
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            api_key: crate::secret::SecretString::new(api_key),
        }
    }

    /// Load configuration from environment.
    ///
    /// Required:
    /// - `LENS_API_KEY`
    ///
    /// Optional:
    /// - `LENS_API_BASE_URL`
    pub fn from_env() -> Result<Self, CapabilityError> {
        let base_url = std::env::var("LENS_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.lens.org/patent/search".to_string());
        let api_key = std::env::var("LENS_API_KEY")
            .map_err(|_| CapabilityError::auth("LENS_API_KEY missing"))?;
        Ok(Self::new(base_url, api_key))
    }
}

impl PatentSearchProvider for LensProvider {
    fn name(&self) -> &'static str {
        "lens_api"
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        if request.query.trim().is_empty() {
            return Err(CapabilityError::invalid_input("query cannot be empty"));
        }

        let timer = CallTimer::start();
        let body = serde_json::json!({
            "query": request.query,
            "size": 5
        });

        let response = self
            .client
            .post(&self.base_url)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key.expose()))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .json(&body)
            .send()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .map_err(|e| CapabilityError::network(e.to_string()))?;

        if !status.is_success() {
            return Err(CapabilityError::invalid_input(format!(
                "Lens error: {status}"
            )));
        }

        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| CapabilityError::invalid_input(e.to_string()))?;
        let items = extract_results_array(&parsed);

        let mut results = Vec::new();
        for item in items {
            let publication_id = pick_string(
                item,
                &[
                    "publication_id",
                    "publicationNumber",
                    "doc_number",
                    "publication_number",
                ],
            )
            .unwrap_or_else(|| "unknown".to_string());
            let title = pick_string(item, &["title", "invention_title", "patent_title", "name"])
                .unwrap_or_else(|| "Lens result".to_string());
            let abstract_text =
                pick_string(item, &["abstract", "abstract_text", "summary"]).unwrap_or_default();
            let url = pick_string(item, &["url", "link"]).unwrap_or_else(|| self.base_url.clone());

            let result = PatentSearchResult {
                publication_id: publication_id.clone(),
                title,
                abstract_text,
                operator: PatentOperator::Lens,
                url,
                raw_source: item.clone(),
            };

            let observation =
                ProviderObservation::new(self.name(), "lens", result, timer.elapsed_ms())
                    .with_request_hash(canonical_hash(&format!(
                        "{}:{}:{}",
                        request.query, ctx.trace_id, publication_id
                    )));

            results.push(observation);
        }

        Ok(PatentSearchResponse {
            results,
            paid_actions: Vec::new(),
        })
    }
}

// =============================================================================
// Composite Provider
// =============================================================================

/// Composite provider that delegates to operator-specific providers.
#[derive(Clone, Default)]
pub struct CompositePatentProvider {
    providers: HashMap<PatentOperator, Arc<dyn PatentSearchProvider>>,
}

impl std::fmt::Debug for CompositePatentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositePatentProvider")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl CompositePatentProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_provider(
        mut self,
        operator: PatentOperator,
        provider: Arc<dyn PatentSearchProvider>,
    ) -> Self {
        self.providers.insert(operator, provider);
        self
    }

    pub fn from_env() -> Result<Self, CapabilityError> {
        let mut composite = Self::new().with_provider(
            PatentOperator::Uspto,
            Arc::new(PatentsViewProvider::from_env()),
        );

        if let Ok(epo) = EpoOpsProvider::from_env() {
            composite = composite.with_provider(PatentOperator::Epo, Arc::new(epo));
        }
        if let Ok(wipo) = WipoPatentscopeProvider::from_env() {
            composite = composite.with_provider(PatentOperator::Wipo, Arc::new(wipo));
        }
        if let Ok(lens) = LensProvider::from_env() {
            composite = composite.with_provider(PatentOperator::Lens, Arc::new(lens));
        }

        Ok(composite)
    }
}

impl PatentSearchProvider for CompositePatentProvider {
    fn name(&self) -> &'static str {
        "composite_patent"
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        let mut results = Vec::new();
        let mut paid_actions = Vec::new();
        let mut matched = false;

        for operator in &request.operators {
            let Some(provider) = self.providers.get(operator) else {
                continue;
            };
            matched = true;

            let response = provider.search(request, ctx)?;
            results.extend(response.results);
            paid_actions.extend(response.paid_actions);
        }

        if !matched {
            return Err(CapabilityError::not_found(
                "No provider configured for requested operators",
            ));
        }

        Ok(PatentSearchResponse {
            results,
            paid_actions,
        })
    }
}

/// Stub patent provider for deterministic local runs.
#[derive(Debug, Clone, Default)]
pub struct StubPatentProvider {
    vendor: String,
}

impl StubPatentProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vendor: "stub-patent".to_string(),
        }
    }
}

impl PatentSearchProvider for StubPatentProvider {
    fn name(&self) -> &str {
        &self.vendor
    }

    fn search(
        &self,
        request: &PatentSearchRequest,
        ctx: &ProviderCallContext,
    ) -> Result<PatentSearchResponse, CapabilityError> {
        if request.query.trim().is_empty() {
            return Err(CapabilityError::invalid_input("query cannot be empty"));
        }

        let timer = CallTimer::start();
        let request_hash = canonical_hash(&format!(
            "{}:{}:{:?}",
            request.query, ctx.trace_id, request.operators
        ));

        let operator = request
            .operators
            .first()
            .copied()
            .unwrap_or(PatentOperator::Uspto);
        let result = PatentSearchResult {
            publication_id: format!("{}-demo-001", operator.as_str()),
            title: "Demo patent result".to_string(),
            abstract_text: "Stub abstract for deterministic testing.".to_string(),
            operator,
            url: format!("https://example.com/patents/{}", operator.as_str()),
            raw_source: serde_json::json!({
                "query": request.query,
                "operator": operator.as_str(),
                "trace_id": ctx.trace_id,
            }),
        };

        let observation =
            ProviderObservation::new(self.vendor.clone(), "stub", result, timer.elapsed_ms())
                .with_request_hash(request_hash);

        let paid_actions = if request.include_paid {
            vec![PaidAction {
                action_id: format!("paid:{}:export", operator.as_str()),
                operator,
                description: "Paid export of bulk results".to_string(),
                estimated_cost_usd: 25.0,
                requires_approval: true,
            }]
        } else {
            Vec::new()
        };

        Ok(PatentSearchResponse {
            results: vec![observation],
            paid_actions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_provider_returns_results() {
        let provider = StubPatentProvider::new();
        let request = PatentSearchRequest::new(
            "battery electrolyte",
            vec![PatentOperator::Uspto, PatentOperator::Epo],
        );
        let ctx = ProviderCallContext::default();

        let response = provider.search(&request, &ctx).expect("should succeed");
        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].content.title.contains("Demo"));
    }
}
