// Copyright 2024-2026 Reflective Labs

// SPDX-License-Identifier: MIT

//! Capability declarations for backends.
//!
//! Capabilities describe *what a backend can do*, independent of its kind.
//! A single backend may support multiple capabilities (e.g., a multimodal
//! LLM supports TextGeneration + Vision + CodeGeneration).
//!
//! # Design
//!
//! Capabilities are intentionally fine-grained. Selection logic matches
//! required capabilities against declared capabilities. This avoids the
//! need to know *which specific backend* you need — just declare what
//! you need and let the selector find it.

use serde::{Deserialize, Serialize};

/// A capability that a backend declares it supports.
///
/// Organized by domain but not restricted to any single backend kind.
/// A backend of any kind can declare any capability it genuinely supports.
///
/// # Extensibility
///
/// The `Other(String)` variant allows declaring capabilities not yet
/// enumerated. Use it for experimental or domain-specific capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // ── LLM / Generation ──────────────────────────────────────────────
    /// Generate natural language text.
    TextGeneration,
    /// Multi-step reasoning and chain-of-thought.
    Reasoning,
    /// Generate or analyze source code.
    CodeGeneration,
    /// Understand and generate text in multiple languages.
    MultilingualText,
    /// Search the web for current information.
    WebSearch,
    /// Understand images, screenshots, diagrams.
    ImageUnderstanding,
    /// Generate structured output (JSON, schemas).
    StructuredOutput,
    /// Use external tools / function calling.
    ToolUse,
    /// Stream partial results as they are generated.
    Streaming,

    // ── Policy / Governance ───────────────────────────────────────────
    /// Evaluate access control rules (who can do what).
    AccessControl,
    /// Check regulatory compliance (GDPR, HIPAA, SOC2).
    ComplianceCheck,
    /// Generate audit trail entries.
    AuditTrail,
    /// Evaluate business rules and constraints.
    RuleEvaluation,

    // ── Optimization / Solving ────────────────────────────────────────
    /// Solve constraint satisfaction problems.
    ConstraintSolving,
    /// Allocate resources under constraints.
    ResourceAllocation,
    /// Schedule tasks/events with dependencies.
    Scheduling,
    /// Linear/integer programming.
    MathematicalProgramming,

    // ── Analytics / ML ────────────────────────────────────────────────
    /// Generate vector embeddings from text/images.
    Embedding,
    /// Rerank candidates by relevance.
    Reranking,
    /// Find similar vectors (nearest neighbor search).
    VectorSearch,
    /// Group data points by similarity.
    Clustering,
    /// Predict continuous values.
    Regression,
    /// Assign categories to data.
    Classification,
    /// Detect anomalies in data patterns.
    AnomalyDetection,

    // ── Search / Recall ───────────────────────────────────────────────
    /// Full-text document search.
    FullTextSearch,
    /// Graph traversal and relationship queries.
    GraphTraversal,
    /// Semantic search using embeddings.
    SemanticSearch,

    // ── Storage / Persistence ─────────────────────────────────────────
    /// Key-value storage.
    KeyValue,
    /// Document storage (JSON, BSON).
    DocumentStore,
    /// Append-only event sourcing.
    EventSourcing,

    // ── Infrastructure ────────────────────────────────────────────────
    /// Deterministic replay of operations.
    Replay,
    /// Operate without network access.
    Offline,

    /// Extension point for capabilities not yet enumerated.
    Other(String),
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(name) => write!(f, "other:{}", name),
            other => write!(f, "{:?}", other),
        }
    }
}
