// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Unified capability registry for Converge providers.
//!
//! The capability registry provides a single point for discovering and
//! selecting providers based on their capabilities. This supports the
//! Converge principle that different models excel at different tasks.
//!
//! # Example
//!
//! ```ignore
//! use converge_provider::{CapabilityRegistry, CapabilityRequirements};
//! use converge_core::capability::{CapabilityKind, Modality};
//!
//! let registry = CapabilityRegistry::from_env();
//!
//! // Find an embedder that supports images
//! let requirements = CapabilityRequirements::embedding()
//!     .with_modality(Modality::Image)
//!     .prefer_local(true);
//!
//! if let Some(embedder) = registry.select_embedder(&requirements) {
//!     // Use the embedder
//! }
//! ```

#[cfg(feature = "brave")]
use crate::brave::BraveSearchProvider;
use crate::provider_api::DataSovereignty;
use crate::provider_api::LlmProvider;
use converge_core::capability::{
    CapabilityKind, CapabilityMetadata, Embedding, GraphRecall, Modality, Reranking, VectorRecall,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Requirements for capability selection.
#[derive(Debug, Clone)]
pub struct CapabilityRequirements {
    /// Required capability kind.
    pub capability: CapabilityKind,
    /// Required modalities (for embedding/reranking).
    pub modalities: Vec<Modality>,
    /// Prefer local providers (data sovereignty).
    pub prefer_local: bool,
    /// Required data sovereignty level.
    pub data_sovereignty: DataSovereignty,
    /// Maximum acceptable latency in milliseconds.
    pub max_latency_ms: u32,
}

impl CapabilityRequirements {
    /// Requirements for LLM completion.
    #[must_use]
    pub fn completion() -> Self {
        Self {
            capability: CapabilityKind::Completion,
            modalities: vec![Modality::Text],
            prefer_local: false,
            data_sovereignty: DataSovereignty::Any,
            max_latency_ms: 30_000,
        }
    }

    /// Requirements for embedding.
    #[must_use]
    pub fn embedding() -> Self {
        Self {
            capability: CapabilityKind::Embedding,
            modalities: vec![Modality::Text],
            prefer_local: false,
            data_sovereignty: DataSovereignty::Any,
            max_latency_ms: 5_000,
        }
    }

    /// Requirements for reranking.
    #[must_use]
    pub fn reranking() -> Self {
        Self {
            capability: CapabilityKind::Reranking,
            modalities: vec![Modality::Text],
            prefer_local: false,
            data_sovereignty: DataSovereignty::Any,
            max_latency_ms: 5_000,
        }
    }

    /// Requirements for vector recall.
    #[must_use]
    pub fn vector_recall() -> Self {
        Self {
            capability: CapabilityKind::VectorRecall,
            modalities: vec![],
            prefer_local: true,
            data_sovereignty: DataSovereignty::Any,
            max_latency_ms: 100,
        }
    }

    /// Requirements for graph recall.
    #[must_use]
    pub fn graph_recall() -> Self {
        Self {
            capability: CapabilityKind::GraphRecall,
            modalities: vec![],
            prefer_local: true,
            data_sovereignty: DataSovereignty::Any,
            max_latency_ms: 100,
        }
    }

    /// Add required modality.
    #[must_use]
    pub fn with_modality(mut self, modality: Modality) -> Self {
        if !self.modalities.contains(&modality) {
            self.modalities.push(modality);
        }
        self
    }

    /// Set local preference.
    #[must_use]
    pub fn prefer_local(mut self, prefer: bool) -> Self {
        self.prefer_local = prefer;
        self
    }

    /// Set data sovereignty requirement.
    #[must_use]
    pub fn with_data_sovereignty(mut self, sovereignty: DataSovereignty) -> Self {
        self.data_sovereignty = sovereignty;
        self
    }

    /// Set maximum latency.
    #[must_use]
    pub fn with_max_latency_ms(mut self, ms: u32) -> Self {
        self.max_latency_ms = ms;
        self
    }
}

/// Registered provider with its capabilities.
struct RegisteredProvider {
    /// Provider metadata.
    metadata: CapabilityMetadata,
    /// LLM provider instance (if applicable).
    llm: Option<Arc<dyn LlmProvider>>,
    /// Embedding provider instance (if applicable).
    embedder: Option<Arc<dyn Embedding>>,
    /// Reranker provider instance (if applicable).
    reranker: Option<Arc<dyn Reranking>>,
}

/// Web search provider metadata for agent selection.
#[derive(Debug, Clone)]
pub struct SearchProviderMeta {
    /// Provider name (e.g., "brave", "perplexity").
    pub name: String,
    /// Whether this provider is available (API key set).
    pub available: bool,
    /// Typical latency in milliseconds.
    pub typical_latency_ms: u32,
    /// Whether this provider supports AI-powered summaries.
    pub supports_ai_summary: bool,
    /// Whether this provider supports news search.
    pub supports_news: bool,
    /// Whether this provider supports image search.
    pub supports_images: bool,
    /// Whether this provider supports local/POI search.
    pub supports_local: bool,
}

/// Requirements for selecting a web search provider.
///
/// Unlike LLM requirements, web search requirements focus on
/// search-specific capabilities like news, images, and AI summaries.
#[derive(Debug, Clone)]
pub struct WebSearchRequirements {
    /// Maximum latency in milliseconds.
    pub max_latency_ms: u32,
    /// Whether AI-powered summaries are required.
    pub requires_ai_summary: bool,
    /// Whether news search is required.
    pub requires_news: bool,
    /// Whether image search is required.
    pub requires_images: bool,
    /// Whether local/POI search is required.
    pub requires_local: bool,
    /// Data sovereignty requirement.
    pub data_sovereignty: DataSovereignty,
}

impl Default for WebSearchRequirements {
    fn default() -> Self {
        Self {
            max_latency_ms: 10_000,
            requires_ai_summary: false,
            requires_news: false,
            requires_images: false,
            requires_local: false,
            data_sovereignty: DataSovereignty::Any,
        }
    }
}

impl WebSearchRequirements {
    /// Creates default requirements for general web search.
    #[must_use]
    pub fn web_search() -> Self {
        Self::default()
    }

    /// Creates requirements for AI-grounded search (RAG).
    #[must_use]
    pub fn grounded() -> Self {
        Self {
            max_latency_ms: 15_000,
            requires_ai_summary: true,
            ..Self::default()
        }
    }

    /// Creates requirements for news search.
    #[must_use]
    pub fn news() -> Self {
        Self {
            requires_news: true,
            ..Self::default()
        }
    }

    /// Sets the maximum latency.
    #[must_use]
    pub fn with_max_latency_ms(mut self, ms: u32) -> Self {
        self.max_latency_ms = ms;
        self
    }

    /// Requires AI-powered summaries.
    #[must_use]
    pub fn with_ai_summary(mut self, required: bool) -> Self {
        self.requires_ai_summary = required;
        self
    }

    /// Sets data sovereignty requirement.
    #[must_use]
    pub fn with_data_sovereignty(mut self, sovereignty: DataSovereignty) -> Self {
        self.data_sovereignty = sovereignty;
        self
    }
}

/// Unified capability registry.
///
/// Discovers and manages all available capability providers.
pub struct CapabilityRegistry {
    /// Registered providers by name.
    providers: HashMap<String, RegisteredProvider>,
    /// Vector stores by name.
    vector_stores: HashMap<String, Arc<dyn VectorRecall>>,
    /// Graph stores by name.
    graph_stores: HashMap<String, Arc<dyn GraphRecall>>,
    /// Web search providers by name.
    search_providers: HashMap<String, SearchProviderMeta>,
    /// Brave search provider instance (if available).
    #[cfg(feature = "brave")]
    brave_provider: Option<BraveSearchProvider>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    /// Creates an empty capability registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            vector_stores: HashMap::new(),
            graph_stores: HashMap::new(),
            search_providers: HashMap::new(),
            #[cfg(feature = "brave")]
            brave_provider: None,
        }
    }

    /// Creates a registry with auto-detected providers from environment.
    ///
    /// This checks for:
    /// - Ollama (local LLM and embedding)
    /// - In-memory vector store (always available)
    /// - In-memory graph store (always available)
    /// - Brave Search (if `BRAVE_API_KEY` is set)
    #[must_use]
    pub fn with_local_defaults() -> Self {
        let mut registry = Self::new();

        // Add in-memory vector store
        registry.add_vector_store(
            "default",
            Arc::new(crate::vector::InMemoryVectorStore::new()),
        );

        // Graph store moved to organism-intelligence crate

        // Try to add Brave Search if available
        registry.try_add_brave_from_env();

        registry
    }

    /// Attempts to add Brave Search provider from environment.
    ///
    /// Returns `true` if Brave Search was added successfully.
    pub fn try_add_brave_from_env(&mut self) -> bool {
        #[cfg(feature = "brave")]
        if let Ok(provider) = BraveSearchProvider::from_env() {
            self.brave_provider = Some(provider);
            self.search_providers.insert(
                "brave".to_string(),
                SearchProviderMeta {
                    name: "brave".to_string(),
                    available: true,
                    typical_latency_ms: 500,
                    supports_ai_summary: false, // Requires Pro plan
                    supports_news: true,
                    supports_images: true,
                    supports_local: true,
                },
            );
            return true;
        }
        false
    }

    /// Adds Brave Search provider with a specific API key.
    #[cfg(feature = "brave")]
    pub fn add_brave(&mut self, api_key: impl Into<String>) {
        self.brave_provider = Some(BraveSearchProvider::new(api_key));
        self.search_providers.insert(
            "brave".to_string(),
            SearchProviderMeta {
                name: "brave".to_string(),
                available: true,
                typical_latency_ms: 500,
                supports_ai_summary: false,
                supports_news: true,
                supports_images: true,
                supports_local: true,
            },
        );
    }

    /// Gets the Brave Search provider if available.
    #[cfg(feature = "brave")]
    #[must_use]
    pub fn brave(&self) -> Option<&BraveSearchProvider> {
        self.brave_provider.as_ref()
    }

    /// Checks if web search capability is available.
    #[must_use]
    pub fn has_web_search(&self) -> bool {
        !self.search_providers.is_empty()
    }

    /// Gets metadata for all available search providers.
    #[must_use]
    pub fn search_providers(&self) -> Vec<&SearchProviderMeta> {
        self.search_providers.values().collect()
    }

    /// Selects the best search provider based on requirements.
    ///
    /// Currently returns Brave if available, as it's the primary search provider.
    #[must_use]
    pub fn select_search_provider(
        &self,
        requirements: &WebSearchRequirements,
    ) -> Option<&SearchProviderMeta> {
        self.search_providers
            .values()
            .filter(|p| {
                // Basic availability and latency check
                if !p.available || p.typical_latency_ms > requirements.max_latency_ms {
                    return false;
                }
                // Check required capabilities
                if requirements.requires_ai_summary && !p.supports_ai_summary {
                    return false;
                }
                if requirements.requires_news && !p.supports_news {
                    return false;
                }
                if requirements.requires_images && !p.supports_images {
                    return false;
                }
                if requirements.requires_local && !p.supports_local {
                    return false;
                }
                true
            })
            .max_by_key(|p| {
                // Score providers by their capabilities
                let mut score = 0i32;
                if p.supports_ai_summary {
                    score += 100;
                }
                if p.supports_news {
                    score += 20;
                }
                if p.supports_images {
                    score += 20;
                }
                if p.supports_local {
                    score += 10;
                }
                // Prefer lower latency
                score -= (p.typical_latency_ms / 100) as i32;
                score
            })
    }

    /// Registers an LLM provider.
    pub fn add_llm_provider(
        &mut self,
        name: &str,
        provider: Arc<dyn LlmProvider>,
        metadata: CapabilityMetadata,
    ) {
        let entry = self
            .providers
            .entry(name.to_string())
            .or_insert_with(|| RegisteredProvider {
                metadata: metadata.clone(),
                llm: None,
                embedder: None,
                reranker: None,
            });
        entry.llm = Some(provider);
        entry.metadata = metadata;
    }

    /// Registers an embedding provider.
    #[allow(clippy::needless_pass_by_value)]
    pub fn add_embedder(
        &mut self,
        name: &str,
        provider: Arc<dyn Embedding>,
        metadata: CapabilityMetadata,
    ) {
        let entry = self
            .providers
            .entry(name.to_string())
            .or_insert_with(|| RegisteredProvider {
                metadata: metadata.clone(),
                llm: None,
                embedder: None,
                reranker: None,
            });
        entry.embedder = Some(provider);
        // Merge capabilities
        for cap in &metadata.capabilities {
            if !entry.metadata.capabilities.contains(cap) {
                entry.metadata.capabilities.push(*cap);
            }
        }
    }

    /// Registers a reranker provider.
    #[allow(clippy::needless_pass_by_value)]
    pub fn add_reranker(
        &mut self,
        name: &str,
        provider: Arc<dyn Reranking>,
        metadata: CapabilityMetadata,
    ) {
        let entry = self
            .providers
            .entry(name.to_string())
            .or_insert_with(|| RegisteredProvider {
                metadata: metadata.clone(),
                llm: None,
                embedder: None,
                reranker: None,
            });
        entry.reranker = Some(provider);
        // Merge capabilities
        for cap in &metadata.capabilities {
            if !entry.metadata.capabilities.contains(cap) {
                entry.metadata.capabilities.push(*cap);
            }
        }
    }

    /// Registers a vector store.
    pub fn add_vector_store(&mut self, name: &str, store: Arc<dyn VectorRecall>) {
        self.vector_stores.insert(name.to_string(), store);
    }

    /// Registers a graph store.
    pub fn add_graph_store(&mut self, name: &str, store: Arc<dyn GraphRecall>) {
        self.graph_stores.insert(name.to_string(), store);
    }

    /// Selects an LLM provider matching requirements.
    #[must_use]
    pub fn select_llm(
        &self,
        requirements: &CapabilityRequirements,
    ) -> Option<Arc<dyn LlmProvider>> {
        self.providers
            .values()
            .filter(|p| p.llm.is_some() && self.matches_requirements(&p.metadata, requirements))
            .max_by_key(|p| self.score_provider(&p.metadata, requirements))
            .and_then(|p| p.llm.clone())
    }

    /// Selects an embedding provider matching requirements.
    #[must_use]
    pub fn select_embedder(
        &self,
        requirements: &CapabilityRequirements,
    ) -> Option<Arc<dyn Embedding>> {
        self.providers
            .values()
            .filter(|p| {
                p.embedder.is_some() && self.matches_requirements(&p.metadata, requirements)
            })
            .max_by_key(|p| self.score_provider(&p.metadata, requirements))
            .and_then(|p| p.embedder.clone())
    }

    /// Selects a reranker provider matching requirements.
    #[must_use]
    pub fn select_reranker(
        &self,
        requirements: &CapabilityRequirements,
    ) -> Option<Arc<dyn Reranking>> {
        self.providers
            .values()
            .filter(|p| {
                p.reranker.is_some() && self.matches_requirements(&p.metadata, requirements)
            })
            .max_by_key(|p| self.score_provider(&p.metadata, requirements))
            .and_then(|p| p.reranker.clone())
    }

    /// Gets the default vector store.
    #[must_use]
    pub fn get_vector_store(&self, name: &str) -> Option<Arc<dyn VectorRecall>> {
        self.vector_stores.get(name).cloned()
    }

    /// Gets the default graph store.
    #[must_use]
    pub fn get_graph_store(&self, name: &str) -> Option<Arc<dyn GraphRecall>> {
        self.graph_stores.get(name).cloned()
    }

    /// Gets the default vector store (named "default").
    #[must_use]
    pub fn default_vector_store(&self) -> Option<Arc<dyn VectorRecall>> {
        self.get_vector_store("default")
    }

    /// Gets the default graph store (named "default").
    #[must_use]
    pub fn default_graph_store(&self) -> Option<Arc<dyn GraphRecall>> {
        self.get_graph_store("default")
    }

    /// Lists all registered provider names.
    #[must_use]
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(String::as_str).collect()
    }

    /// Lists all registered vector store names.
    #[must_use]
    pub fn vector_store_names(&self) -> Vec<&str> {
        self.vector_stores.keys().map(String::as_str).collect()
    }

    /// Lists all registered graph store names.
    #[must_use]
    pub fn graph_store_names(&self) -> Vec<&str> {
        self.graph_stores.keys().map(String::as_str).collect()
    }

    /// Checks if a provider matches the requirements.
    #[allow(clippy::unused_self)]
    fn matches_requirements(
        &self,
        metadata: &CapabilityMetadata,
        requirements: &CapabilityRequirements,
    ) -> bool {
        // Check capability
        if !metadata.capabilities.contains(&requirements.capability) {
            return false;
        }

        // Check modalities
        for modality in &requirements.modalities {
            if !metadata.modalities.contains(modality) {
                return false;
            }
        }

        // Check data sovereignty - local providers satisfy all requirements
        #[allow(clippy::match_same_arms)]
        match (&requirements.data_sovereignty, metadata.is_local) {
            (DataSovereignty::Any, _) | (_, true) => {} // Always OK or local
            _ => {} // Remote providers must match specific sovereignty
        }

        // Check latency
        if metadata.typical_latency_ms > requirements.max_latency_ms {
            return false;
        }

        true
    }

    /// Scores a provider for selection (higher = better).
    #[allow(clippy::unused_self, clippy::cast_possible_wrap)]
    fn score_provider(
        &self,
        metadata: &CapabilityMetadata,
        requirements: &CapabilityRequirements,
    ) -> i32 {
        let mut score = 0;

        // Prefer local if requested
        if requirements.prefer_local && metadata.is_local {
            score += 100;
        }

        // Lower latency is better
        if metadata.typical_latency_ms < requirements.max_latency_ms / 2 {
            score += 50;
        }

        // More modalities is better
        score += (metadata.modalities.len() * 10) as i32;

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::InMemoryGraphStore;
    use crate::vector::InMemoryVectorStore;

    #[test]
    fn registry_with_local_defaults() {
        let registry = CapabilityRegistry::with_local_defaults();

        assert!(registry.default_vector_store().is_some());
        assert!(registry.default_graph_store().is_some());
    }

    #[test]
    fn add_and_get_stores() {
        let mut registry = CapabilityRegistry::new();

        registry.add_vector_store("test", Arc::new(InMemoryVectorStore::new()));
        registry.add_graph_store("test", Arc::new(InMemoryGraphStore::new()));

        assert!(registry.get_vector_store("test").is_some());
        assert!(registry.get_graph_store("test").is_some());
        assert!(registry.get_vector_store("missing").is_none());
    }

    #[test]
    fn list_registered_stores() {
        let registry = CapabilityRegistry::with_local_defaults();

        let vector_stores = registry.vector_store_names();
        assert!(vector_stores.contains(&"default"));

        let graph_stores = registry.graph_store_names();
        assert!(graph_stores.contains(&"default"));
    }

    #[test]
    fn capability_requirements_builder() {
        let reqs = CapabilityRequirements::embedding()
            .with_modality(Modality::Image)
            .prefer_local(true)
            .with_max_latency_ms(1000);

        assert_eq!(reqs.capability, CapabilityKind::Embedding);
        assert!(reqs.modalities.contains(&Modality::Image));
        assert!(reqs.prefer_local);
        assert_eq!(reqs.max_latency_ms, 1000);
    }
}
