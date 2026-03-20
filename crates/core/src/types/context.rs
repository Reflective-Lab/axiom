// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary
// All rights reserved. This source code is proprietary and confidential.
// Unauthorized copying, modification, or distribution is strictly prohibited.

//! Context types with builder pattern.
//!
//! Per CONTEXT.md:
//! - ContextBuilder -> immutable Context (fluent add_fact, set_intent)
//! - Append-only Context; Facts are immutable
//!
//! This module provides TypesContextSnapshot for serialization/snapshots.
//! The existing context::Context remains for runtime mutation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::frame::IntentId;
use super::id::FactId;

// ============================================================================
// TypesContextKey - Key for context entries
// ============================================================================

/// Key for context entries (simplified from existing ContextKey).
///
/// Uses "Types" prefix to avoid collision with existing ContextKey.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypesContextKey(String);

impl TypesContextKey {
    /// Create a new context key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Get the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Create a "seeds" key.
    pub fn seeds() -> Self {
        Self("seeds".to_string())
    }

    /// Create a "hypotheses" key.
    pub fn hypotheses() -> Self {
        Self("hypotheses".to_string())
    }

    /// Create a "strategies" key.
    pub fn strategies() -> Self {
        Self("strategies".to_string())
    }

    /// Create a "constraints" key.
    pub fn constraints() -> Self {
        Self("constraints".to_string())
    }

    /// Create a "signals" key.
    pub fn signals() -> Self {
        Self("signals".to_string())
    }

    /// Create a "observations" key.
    pub fn observations() -> Self {
        Self("observations".to_string())
    }

    /// Create a "proposals" key.
    pub fn proposals() -> Self {
        Self("proposals".to_string())
    }

    /// Create a "facts" key.
    pub fn facts() -> Self {
        Self("facts".to_string())
    }
}

impl std::fmt::Display for TypesContextKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TypesContextKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for TypesContextKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

// ============================================================================
// ContextBuilder - Builder for constructing immutable Context
// ============================================================================

/// Builder for constructing immutable Context.
///
/// Provides fluent API for building context with facts and intent.
///
/// # Example
///
/// ```
/// use converge_core::types::{ContextBuilder, TypesContextKey, FactId, IntentId};
///
/// let context = ContextBuilder::new()
///     .with_intent(IntentId::new("intent-1"))
///     .add_fact(TypesContextKey::seeds(), FactId::new("fact-1"))
///     .add_fact(TypesContextKey::seeds(), FactId::new("fact-2"))
///     .with_metadata("session_id", "sess-123")
///     .build();
///
/// assert!(context.has(&TypesContextKey::seeds()));
/// assert_eq!(context.get(&TypesContextKey::seeds()).len(), 2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ContextBuilder {
    facts: HashMap<TypesContextKey, Vec<FactId>>,
    intent_id: Option<IntentId>,
    metadata: HashMap<String, String>,
}

impl ContextBuilder {
    /// Create a new empty context builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fact reference to the context.
    pub fn add_fact(mut self, key: TypesContextKey, fact_id: FactId) -> Self {
        self.facts.entry(key).or_default().push(fact_id);
        self
    }

    /// Add multiple facts under a key.
    pub fn add_facts(mut self, key: TypesContextKey, fact_ids: Vec<FactId>) -> Self {
        self.facts.entry(key).or_default().extend(fact_ids);
        self
    }

    /// Set the root intent.
    pub fn with_intent(mut self, intent_id: IntentId) -> Self {
        self.intent_id = Some(intent_id);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the context snapshot.
    pub fn build(self) -> TypesContextSnapshot {
        TypesContextSnapshot {
            facts: self.facts,
            intent_id: self.intent_id,
            metadata: self.metadata,
            version: 0,
        }
    }

    /// Build with a specific version.
    pub fn build_versioned(self, version: u64) -> TypesContextSnapshot {
        TypesContextSnapshot {
            facts: self.facts,
            intent_id: self.intent_id,
            metadata: self.metadata,
            version,
        }
    }
}

// ============================================================================
// TypesContextSnapshot - Immutable context snapshot
// ============================================================================

/// Immutable context snapshot.
///
/// This represents a point-in-time view of context. The existing context::Context
/// remains for runtime mutation; this is for snapshot/serialization.
///
/// Uses "Types" prefix to avoid collision with existing Context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesContextSnapshot {
    /// Facts organized by key.
    pub facts: HashMap<TypesContextKey, Vec<FactId>>,
    /// Root intent ID.
    pub intent_id: Option<IntentId>,
    /// Metadata.
    pub metadata: HashMap<String, String>,
    /// Version number for optimistic concurrency.
    pub version: u64,
}

impl Default for TypesContextSnapshot {
    fn default() -> Self {
        Self {
            facts: HashMap::new(),
            intent_id: None,
            metadata: HashMap::new(),
            version: 0,
        }
    }
}

impl TypesContextSnapshot {
    /// Create an empty context snapshot.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Get fact IDs for a key.
    pub fn get(&self, key: &TypesContextKey) -> &[FactId] {
        self.facts.get(key).map_or(&[], Vec::as_slice)
    }

    /// Check if key has facts.
    pub fn has(&self, key: &TypesContextKey) -> bool {
        self.facts.get(key).is_some_and(|v| !v.is_empty())
    }

    /// Get total number of facts across all keys.
    pub fn total_facts(&self) -> usize {
        self.facts.values().map(Vec::len).sum()
    }

    /// Get all keys that have facts.
    pub fn keys(&self) -> impl Iterator<Item = &TypesContextKey> {
        self.facts.keys()
    }

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    /// Create a new version with incremented version number.
    pub fn increment_version(&self) -> Self {
        Self {
            facts: self.facts.clone(),
            intent_id: self.intent_id.clone(),
            metadata: self.metadata.clone(),
            version: self.version + 1,
        }
    }

    /// Convert to builder for modifications.
    pub fn to_builder(&self) -> ContextBuilder {
        ContextBuilder {
            facts: self.facts.clone(),
            intent_id: self.intent_id.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_key_display() {
        let key = TypesContextKey::new("test-key");
        assert_eq!(key.to_string(), "test-key");
        assert_eq!(key.as_str(), "test-key");
    }

    #[test]
    fn context_key_helpers() {
        assert_eq!(TypesContextKey::seeds().as_str(), "seeds");
        assert_eq!(TypesContextKey::hypotheses().as_str(), "hypotheses");
        assert_eq!(TypesContextKey::facts().as_str(), "facts");
    }

    #[test]
    fn context_builder_basic() {
        let context = ContextBuilder::new()
            .add_fact(TypesContextKey::seeds(), FactId::new("f1"))
            .add_fact(TypesContextKey::seeds(), FactId::new("f2"))
            .build();

        assert!(context.has(&TypesContextKey::seeds()));
        assert_eq!(context.get(&TypesContextKey::seeds()).len(), 2);
        assert!(!context.has(&TypesContextKey::hypotheses()));
    }

    #[test]
    fn context_builder_with_intent() {
        let context = ContextBuilder::new()
            .with_intent(IntentId::new("intent-1"))
            .build();

        assert_eq!(
            context.intent_id.as_ref().map(IntentId::as_str),
            Some("intent-1")
        );
    }

    #[test]
    fn context_builder_with_metadata() {
        let context = ContextBuilder::new()
            .with_metadata("session_id", "sess-123")
            .with_metadata("user_id", "user-456")
            .build();

        assert_eq!(context.get_metadata("session_id"), Some("sess-123"));
        assert_eq!(context.get_metadata("user_id"), Some("user-456"));
        assert_eq!(context.get_metadata("nonexistent"), None);
    }

    #[test]
    fn context_builder_add_facts() {
        let context = ContextBuilder::new()
            .add_facts(
                TypesContextKey::proposals(),
                vec![FactId::new("p1"), FactId::new("p2"), FactId::new("p3")],
            )
            .build();

        assert_eq!(context.get(&TypesContextKey::proposals()).len(), 3);
    }

    #[test]
    fn context_snapshot_total_facts() {
        let context = ContextBuilder::new()
            .add_fact(TypesContextKey::seeds(), FactId::new("f1"))
            .add_fact(TypesContextKey::seeds(), FactId::new("f2"))
            .add_fact(TypesContextKey::hypotheses(), FactId::new("h1"))
            .build();

        assert_eq!(context.total_facts(), 3);
    }

    #[test]
    fn context_snapshot_versioning() {
        let context = ContextBuilder::new()
            .add_fact(TypesContextKey::seeds(), FactId::new("f1"))
            .build_versioned(5);

        assert_eq!(context.version, 5);

        let next = context.increment_version();
        assert_eq!(next.version, 6);
    }

    #[test]
    fn context_snapshot_to_builder() {
        let original = ContextBuilder::new()
            .add_fact(TypesContextKey::seeds(), FactId::new("f1"))
            .with_intent(IntentId::new("intent-1"))
            .build();

        let modified = original
            .to_builder()
            .add_fact(TypesContextKey::hypotheses(), FactId::new("h1"))
            .build();

        assert!(modified.has(&TypesContextKey::seeds()));
        assert!(modified.has(&TypesContextKey::hypotheses()));
    }

    #[test]
    fn context_snapshot_serialization() {
        let context = ContextBuilder::new()
            .add_fact(TypesContextKey::seeds(), FactId::new("f1"))
            .with_intent(IntentId::new("intent-1"))
            .build_versioned(1);

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("\"seeds\""));
        assert!(json.contains("\"f1\""));
        assert!(json.contains("\"version\":1"));

        let deserialized: TypesContextSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, 1);
        assert!(deserialized.has(&TypesContextKey::seeds()));
    }

    #[test]
    fn context_snapshot_empty() {
        let empty = TypesContextSnapshot::empty();
        assert_eq!(empty.total_facts(), 0);
        assert!(empty.intent_id.is_none());
        assert_eq!(empty.version, 0);
    }
}
