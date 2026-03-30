// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Capability abstractions for Converge providers.
//!
//! This module defines the abstract capability traits that providers can implement.
//! A single provider may implement multiple capabilities (e.g., Qwen3-VL supports
//! Embedding + Reranking + Vision).
//!
//! # Architecture
//!
//! ```text
//! Capabilities (what)          Providers (who/where)
//! ──────────────────          ────────────────────
//! Completion                   Anthropic, OpenAI, Ollama
//! Embedding                    OpenAI, Qwen3-VL, Ollama/nomic
//! Reranking                    Qwen3-VL, Cohere
//! VectorRecall                 LanceDB, Qdrant
//! GraphRecall                  Neo4j, NebulaGraph
//! Vision                       Claude, GPT-4V, Qwen-VL
//! ```
//!
//! # Design Principles
//!
//! 1. **Capabilities produce candidates, not decisions** - Aligned with Converge's
//!    "LLMs suggest, never decide" principle. Embeddings, reranking, and recall
//!    operations return scored candidates that must go through validation.
//!
//! 2. **Stores are caches, not truth** - Vector and graph stores can be rebuilt
//!    from the authoritative Context at any time.
//!
//! 3. **Explicit provenance** - Every operation tracks its source for auditability.
//!
//! # Example
//!
//! ```ignore
//! use converge_core::capability::{Embedding, EmbedInput, EmbedRequest};
//!
//! // Embed text and images in a shared space (Qwen3-VL style)
//! let request = EmbedRequest::new(vec![
//!     EmbedInput::Text("Product description".into()),
//!     EmbedInput::image_path("/screenshots/dashboard.png"),
//! ]);
//!
//! let response = embedder.embed(&request).await?;
//! // response.embeddings contains vectors in unified semantic space
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// =============================================================================
// COMMON TYPES
// =============================================================================

/// Input modalities that capabilities can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    /// Plain text input.
    Text,
    /// Image input (PNG, JPEG, etc.).
    Image,
    /// Video input (frames or full video).
    Video,
    /// Audio input (speech, sound).
    Audio,
    /// Structured data (JSON, tables).
    Structured,
}

/// Error from a capability operation.
#[derive(Debug, Clone)]
pub struct CapabilityError {
    /// Error kind.
    pub kind: CapabilityErrorKind,
    /// Human-readable message.
    pub message: String,
    /// Whether the operation can be retried.
    pub retryable: bool,
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for CapabilityError {}

/// Kind of capability error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityErrorKind {
    /// Authentication failed.
    Authentication,
    /// Rate limit exceeded.
    RateLimit,
    /// Invalid input.
    InvalidInput,
    /// Unsupported modality.
    UnsupportedModality,
    /// Network error.
    Network,
    /// Provider returned an error.
    ProviderError,
    /// Store operation failed.
    StoreError,
    /// Resource not found.
    NotFound,
    /// Operation timed out.
    Timeout,
}

impl CapabilityError {
    /// Creates an invalid input error.
    #[must_use]
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            kind: CapabilityErrorKind::InvalidInput,
            message: message.into(),
            retryable: false,
        }
    }

    /// Creates an unsupported modality error.
    #[must_use]
    pub fn unsupported_modality(modality: Modality) -> Self {
        Self {
            kind: CapabilityErrorKind::UnsupportedModality,
            message: format!("Modality {modality:?} is not supported by this provider"),
            retryable: false,
        }
    }

    /// Creates a store error.
    #[must_use]
    pub fn store(message: impl Into<String>) -> Self {
        Self {
            kind: CapabilityErrorKind::StoreError,
            message: message.into(),
            retryable: false,
        }
    }

    /// Creates a network error.
    #[must_use]
    pub fn network(message: impl Into<String>) -> Self {
        Self {
            kind: CapabilityErrorKind::Network,
            message: message.into(),
            retryable: true,
        }
    }

    /// Creates an authentication error.
    #[must_use]
    pub fn auth(message: impl Into<String>) -> Self {
        Self {
            kind: CapabilityErrorKind::Authentication,
            message: message.into(),
            retryable: false,
        }
    }

    /// Creates a not found error.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: CapabilityErrorKind::NotFound,
            message: message.into(),
            retryable: false,
        }
    }
}

// =============================================================================
// EMBEDDING CAPABILITY
// =============================================================================

/// Input for embedding operations.
///
/// Supports multiple modalities for multimodal embedders like Qwen3-VL.
#[derive(Debug, Clone)]
pub enum EmbedInput {
    /// Plain text input.
    Text(String),
    /// Raw image bytes with MIME type.
    ImageBytes {
        /// Image data.
        data: Vec<u8>,
        /// MIME type (e.g., "image/png").
        mime_type: String,
    },
    /// Path to an image file (for lazy loading).
    ImagePath(PathBuf),
    /// Video frame at a specific timestamp.
    VideoFrame {
        /// Path to video file.
        path: PathBuf,
        /// Timestamp in milliseconds.
        timestamp_ms: u64,
    },
    /// Mixed modality input (e.g., text + image together).
    /// Qwen3-VL supports this for joint embedding.
    Mixed(Vec<EmbedInput>),
}

impl EmbedInput {
    /// Creates a text input.
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Creates an image path input.
    #[must_use]
    pub fn image_path(path: impl Into<PathBuf>) -> Self {
        Self::ImagePath(path.into())
    }

    /// Creates an image bytes input.
    #[must_use]
    pub fn image_bytes(data: Vec<u8>, mime_type: impl Into<String>) -> Self {
        Self::ImageBytes {
            data,
            mime_type: mime_type.into(),
        }
    }

    /// Returns the primary modality of this input.
    #[must_use]
    pub fn modality(&self) -> Modality {
        match self {
            Self::Text(_) => Modality::Text,
            Self::ImageBytes { .. } | Self::ImagePath(_) => Modality::Image,
            Self::VideoFrame { .. } => Modality::Video,
            Self::Mixed(_) => Modality::Structured, // Mixed is its own thing
        }
    }
}

/// Request for embedding operation.
#[derive(Debug, Clone)]
pub struct EmbedRequest {
    /// Inputs to embed.
    pub inputs: Vec<EmbedInput>,
    /// Desired embedding dimensions (if configurable).
    /// Qwen3-VL supports configurable dimensions.
    pub dimensions: Option<usize>,
    /// Task-specific instruction for the embedder.
    /// Helps the model understand the embedding purpose.
    pub task_instruction: Option<String>,
    /// Whether to normalize the output vectors.
    pub normalize: bool,
}

impl EmbedRequest {
    /// Creates a new embedding request.
    #[must_use]
    pub fn new(inputs: Vec<EmbedInput>) -> Self {
        Self {
            inputs,
            dimensions: None,
            task_instruction: None,
            normalize: true,
        }
    }

    /// Creates a request for a single text input.
    #[must_use]
    pub fn text(s: impl Into<String>) -> Self {
        Self::new(vec![EmbedInput::text(s)])
    }

    /// Sets the desired dimensions.
    #[must_use]
    pub fn with_dimensions(mut self, dim: usize) -> Self {
        self.dimensions = Some(dim);
        self
    }

    /// Sets the task instruction.
    #[must_use]
    pub fn with_task(mut self, instruction: impl Into<String>) -> Self {
        self.task_instruction = Some(instruction.into());
        self
    }

    /// Sets normalization preference.
    #[must_use]
    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

/// Response from an embedding operation.
#[derive(Debug, Clone)]
pub struct EmbedResponse {
    /// Embedding vectors, one per input.
    pub embeddings: Vec<Vec<f32>>,
    /// Model that generated the embeddings.
    pub model: String,
    /// Dimensions of each embedding.
    pub dimensions: usize,
    /// Token/unit usage statistics.
    pub usage: Option<EmbedUsage>,
}

/// Usage statistics for embedding operations.
#[derive(Debug, Clone, Default)]
pub struct EmbedUsage {
    /// Total tokens processed.
    pub total_tokens: u32,
}

/// Trait for providers that can generate embeddings.
///
/// Embeddings map inputs (text, images, etc.) to dense vectors in a
/// shared semantic space. These vectors enable similarity search.
pub trait Embedding: Send + Sync {
    /// Name of this embedding provider.
    fn name(&self) -> &str;

    /// Modalities this embedder supports.
    fn modalities(&self) -> Vec<Modality>;

    /// Default embedding dimensions.
    fn default_dimensions(&self) -> usize;

    /// Generates embeddings for the given inputs.
    ///
    /// # Errors
    ///
    /// Returns error if embedding fails.
    fn embed(&self, request: &EmbedRequest) -> Result<EmbedResponse, CapabilityError>;

    /// Checks if this embedder supports a given modality.
    fn supports(&self, modality: Modality) -> bool {
        self.modalities().contains(&modality)
    }
}

// =============================================================================
// RERANKING CAPABILITY
// =============================================================================

/// Request for reranking operation.
#[derive(Debug, Clone)]
pub struct RerankRequest {
    /// The query to rank against.
    pub query: EmbedInput,
    /// Candidate items to rerank.
    pub candidates: Vec<EmbedInput>,
    /// Maximum number of results to return.
    pub top_k: Option<usize>,
    /// Minimum score threshold (0.0-1.0).
    pub min_score: Option<f64>,
}

impl RerankRequest {
    /// Creates a new rerank request.
    #[must_use]
    pub fn new(query: EmbedInput, candidates: Vec<EmbedInput>) -> Self {
        Self {
            query,
            candidates,
            top_k: None,
            min_score: None,
        }
    }

    /// Creates a text-only rerank request.
    #[must_use]
    pub fn text(query: impl Into<String>, candidates: Vec<String>) -> Self {
        Self::new(
            EmbedInput::text(query),
            candidates.into_iter().map(EmbedInput::text).collect(),
        )
    }

    /// Sets the top-k limit.
    #[must_use]
    pub fn with_top_k(mut self, k: usize) -> Self {
        self.top_k = Some(k);
        self
    }

    /// Sets the minimum score threshold.
    #[must_use]
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }
}

/// A single ranked item from reranking.
#[derive(Debug, Clone)]
pub struct RankedItem {
    /// Index in the original candidates list.
    pub index: usize,
    /// Relevance score (0.0-1.0, higher = more relevant).
    pub score: f64,
}

/// Response from a reranking operation.
#[derive(Debug, Clone)]
pub struct RerankResponse {
    /// Ranked items, sorted by score descending.
    pub ranked: Vec<RankedItem>,
    /// Model that performed the reranking.
    pub model: String,
}

/// Trait for providers that can rerank candidates by relevance.
///
/// Reranking takes a query and a list of candidates, returning
/// fine-grained relevance scores. This is the second stage in
/// two-stage retrieval (embedding recall → reranking).
pub trait Reranking: Send + Sync {
    /// Name of this reranker.
    fn name(&self) -> &str;

    /// Modalities this reranker supports.
    fn modalities(&self) -> Vec<Modality>;

    /// Reranks candidates by relevance to the query.
    ///
    /// # Errors
    ///
    /// Returns error if reranking fails.
    fn rerank(&self, request: &RerankRequest) -> Result<RerankResponse, CapabilityError>;
}

// =============================================================================
// VECTOR RECALL CAPABILITY (Vector Store)
// =============================================================================

/// A stored vector with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorRecord {
    /// Unique identifier.
    pub id: String,
    /// The embedding vector.
    pub vector: Vec<f32>,
    /// Associated metadata (JSON-serializable).
    pub payload: serde_json::Value,
}

/// Query for vector similarity search.
#[derive(Debug, Clone)]
pub struct VectorQuery {
    /// Query vector.
    pub vector: Vec<f32>,
    /// Maximum number of results.
    pub top_k: usize,
    /// Metadata filter (provider-specific).
    pub filter: Option<serde_json::Value>,
    /// Minimum similarity threshold (0.0-1.0).
    pub min_score: Option<f64>,
}

impl VectorQuery {
    /// Creates a new vector query.
    #[must_use]
    pub fn new(vector: Vec<f32>, top_k: usize) -> Self {
        Self {
            vector,
            top_k,
            filter: None,
            min_score: None,
        }
    }

    /// Sets a metadata filter.
    #[must_use]
    pub fn with_filter(mut self, filter: serde_json::Value) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Sets minimum similarity threshold.
    #[must_use]
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }
}

/// A match from vector similarity search.
#[derive(Debug, Clone)]
pub struct VectorMatch {
    /// ID of the matched record.
    pub id: String,
    /// Similarity score (0.0-1.0 for cosine, higher = more similar).
    pub score: f64,
    /// Payload from the matched record.
    pub payload: serde_json::Value,
}

/// Trait for vector stores that enable similarity search.
///
/// Vector stores are **caches**, not authoritative state. They can
/// always be rebuilt from the Context which is the source of truth.
///
/// # Design Note
///
/// This follows Converge's principle: vector stores expand what
/// agents can *see*, not what they are allowed to *decide*.
pub trait VectorRecall: Send + Sync {
    /// Name of this vector store.
    fn name(&self) -> &str;

    /// Insert or update a vector record.
    ///
    /// # Errors
    ///
    /// Returns error if upsert fails.
    fn upsert(&self, record: &VectorRecord) -> Result<(), CapabilityError>;

    /// Batch upsert multiple records.
    ///
    /// Default implementation calls `upsert` for each record.
    ///
    /// # Errors
    ///
    /// Returns error if any upsert fails.
    fn upsert_batch(&self, records: &[VectorRecord]) -> Result<(), CapabilityError> {
        for record in records {
            self.upsert(record)?;
        }
        Ok(())
    }

    /// Query for similar vectors.
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    fn query(&self, query: &VectorQuery) -> Result<Vec<VectorMatch>, CapabilityError>;

    /// Delete a record by ID.
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails.
    fn delete(&self, id: &str) -> Result<(), CapabilityError>;

    /// Clear all records from the store.
    ///
    /// This is safe because vector stores are regenerable caches.
    ///
    /// # Errors
    ///
    /// Returns error if clear fails.
    fn clear(&self) -> Result<(), CapabilityError>;

    /// Count of records in the store.
    ///
    /// # Errors
    ///
    /// Returns error if count fails.
    fn count(&self) -> Result<usize, CapabilityError>;
}

// =============================================================================
// GRAPH RECALL CAPABILITY (Graph Store)
// =============================================================================

/// A node in a knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier.
    pub id: String,
    /// Node label/type (e.g., "Company", "Person", "Product").
    pub label: String,
    /// Node properties.
    pub properties: serde_json::Value,
}

/// An edge in a knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Relationship type (e.g., "`WORKS_FOR`", "`COMPETES_WITH`").
    pub relationship: String,
    /// Edge properties (optional).
    pub properties: Option<serde_json::Value>,
}

/// Query for graph traversal.
#[derive(Debug, Clone)]
pub struct GraphQuery {
    /// Starting node ID(s).
    pub start_nodes: Vec<String>,
    /// Relationship types to traverse (empty = all).
    pub relationships: Vec<String>,
    /// Maximum traversal depth.
    pub max_depth: usize,
    /// Maximum results to return.
    pub limit: usize,
}

impl GraphQuery {
    /// Creates a new graph query starting from a single node.
    #[must_use]
    pub fn from_node(id: impl Into<String>) -> Self {
        Self {
            start_nodes: vec![id.into()],
            relationships: Vec::new(),
            max_depth: 2,
            limit: 100,
        }
    }

    /// Sets the relationships to traverse.
    #[must_use]
    pub fn with_relationships(mut self, rels: Vec<String>) -> Self {
        self.relationships = rels;
        self
    }

    /// Sets the maximum depth.
    #[must_use]
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Sets the result limit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Result from a graph query.
#[derive(Debug, Clone)]
pub struct GraphResult {
    /// Nodes in the result.
    pub nodes: Vec<GraphNode>,
    /// Edges connecting the nodes.
    pub edges: Vec<GraphEdge>,
}

/// Trait for graph stores that enable knowledge graph operations.
///
/// Graph stores capture structured relationships between entities.
/// Like vector stores, they are caches that can be rebuilt.
pub trait GraphRecall: Send + Sync {
    /// Name of this graph store.
    fn name(&self) -> &str;

    /// Add a node to the graph.
    ///
    /// # Errors
    ///
    /// Returns error if operation fails.
    fn add_node(&self, node: &GraphNode) -> Result<(), CapabilityError>;

    /// Add an edge between nodes.
    ///
    /// # Errors
    ///
    /// Returns error if operation fails.
    fn add_edge(&self, edge: &GraphEdge) -> Result<(), CapabilityError>;

    /// Query the graph by traversal.
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    fn traverse(&self, query: &GraphQuery) -> Result<GraphResult, CapabilityError>;

    /// Find nodes by label and properties.
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    fn find_nodes(
        &self,
        label: &str,
        properties: Option<&serde_json::Value>,
    ) -> Result<Vec<GraphNode>, CapabilityError>;

    /// Get a node by ID.
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    fn get_node(&self, id: &str) -> Result<Option<GraphNode>, CapabilityError>;

    /// Delete a node and its edges.
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails.
    fn delete_node(&self, id: &str) -> Result<(), CapabilityError>;

    /// Clear all nodes and edges.
    ///
    /// # Errors
    ///
    /// Returns error if clear fails.
    fn clear(&self) -> Result<(), CapabilityError>;
}

// =============================================================================
// CAPABILITY METADATA
// =============================================================================

/// Metadata about a capability provider.
#[derive(Debug, Clone)]
pub struct CapabilityMetadata {
    /// Provider name.
    pub provider: String,
    /// Capabilities offered.
    pub capabilities: Vec<CapabilityKind>,
    /// Supported modalities.
    pub modalities: Vec<Modality>,
    /// Whether this is a local/on-premises provider.
    pub is_local: bool,
    /// Typical latency in milliseconds.
    pub typical_latency_ms: u32,
}

/// Kinds of capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityKind {
    /// Text/image generation (LLM completion).
    Completion,
    /// Vector embedding generation.
    Embedding,
    /// Relevance reranking.
    Reranking,
    /// Vector similarity search.
    VectorRecall,
    /// Graph traversal and querying.
    GraphRecall,
    /// Full-text document search.
    DocRecall,
    /// Vision/image understanding.
    Vision,
    /// Audio processing (speech-to-text, etc.).
    Audio,
    /// Code execution.
    CodeExecution,
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_input_modality() {
        assert_eq!(EmbedInput::text("hello").modality(), Modality::Text);
        assert_eq!(
            EmbedInput::image_path("/foo.png").modality(),
            Modality::Image
        );
    }

    #[test]
    fn embed_request_builder() {
        let req = EmbedRequest::text("test")
            .with_dimensions(512)
            .with_task("retrieval")
            .with_normalize(false);

        assert_eq!(req.inputs.len(), 1);
        assert_eq!(req.dimensions, Some(512));
        assert_eq!(req.task_instruction, Some("retrieval".into()));
        assert!(!req.normalize);
    }

    #[test]
    fn rerank_request_builder() {
        let req = RerankRequest::text("query", vec!["a".into(), "b".into()])
            .with_top_k(5)
            .with_min_score(0.5);

        assert_eq!(req.candidates.len(), 2);
        assert_eq!(req.top_k, Some(5));
        assert_eq!(req.min_score, Some(0.5));
    }

    #[test]
    fn vector_query_builder() {
        let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10)
            .with_min_score(0.8)
            .with_filter(serde_json::json!({"type": "document"}));

        assert_eq!(query.top_k, 10);
        assert_eq!(query.min_score, Some(0.8));
        assert!(query.filter.is_some());
    }

    #[test]
    fn graph_query_builder() {
        let query = GraphQuery::from_node("company-1")
            .with_relationships(vec!["COMPETES_WITH".into()])
            .with_max_depth(3)
            .with_limit(50);

        assert_eq!(query.start_nodes, vec!["company-1"]);
        assert_eq!(query.max_depth, 3);
        assert_eq!(query.limit, 50);
    }

    #[test]
    fn capability_error_creation() {
        let err = CapabilityError::unsupported_modality(Modality::Video);
        assert_eq!(err.kind, CapabilityErrorKind::UnsupportedModality);
        assert!(!err.retryable);

        let err = CapabilityError::network("connection refused");
        assert_eq!(err.kind, CapabilityErrorKind::Network);
        assert!(err.retryable);
    }
}
