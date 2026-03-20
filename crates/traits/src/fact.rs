// Copyright 2024-2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Facts and proposed facts — the type boundary.
//!
//! This is the most important design decision in Converge: LLMs suggest,
//! the engine validates. `ProposedFact` is not `Fact`. There is no implicit
//! conversion between them.

use serde::{Deserialize, Serialize};

use crate::context::ContextKey;

/// A validated, authoritative assertion in the context.
///
/// Facts are append-only. Once added to the context, they are never
/// mutated or removed (within a convergence run). History is preserved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    /// Which context key this fact belongs to.
    pub key: ContextKey,
    /// Unique identifier within the context key namespace.
    pub id: String,
    /// The fact's content as a string. Interpretation is key-dependent.
    pub content: String,
}

impl Fact {
    /// Creates a new fact.
    #[must_use]
    pub fn new(key: ContextKey, id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            key,
            id: id.into(),
            content: content.into(),
        }
    }
}

/// An unvalidated suggestion from a non-authoritative source.
///
/// Proposed facts live in `ContextKey::Proposals` until a `ValidationAgent`
/// promotes them to `Fact`. The proposal tracks its origin for audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedFact {
    /// The context key this proposal targets.
    pub key: ContextKey,
    /// Unique identifier encoding origin and target.
    pub id: String,
    /// The proposed content.
    pub content: String,
    /// Confidence hint from the source (0.0 - 1.0).
    pub confidence: f64,
    /// Provenance information (e.g., model ID, prompt hash).
    pub provenance: String,
}

/// Error when a `ProposedFact` fails validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Reason the proposal was rejected.
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "validation failed: {}", self.reason)
    }
}

impl std::error::Error for ValidationError {}

impl TryFrom<ProposedFact> for Fact {
    type Error = ValidationError;

    fn try_from(proposed: ProposedFact) -> Result<Self, Self::Error> {
        if !proposed.confidence.is_finite()
            || proposed.confidence < 0.0
            || proposed.confidence > 1.0
        {
            return Err(ValidationError {
                reason: "confidence must be a finite number between 0.0 and 1.0".into(),
            });
        }

        if proposed.content.trim().is_empty() {
            return Err(ValidationError {
                reason: "content cannot be empty".into(),
            });
        }

        Ok(Fact {
            key: proposed.key,
            id: proposed.id,
            content: proposed.content,
        })
    }
}
