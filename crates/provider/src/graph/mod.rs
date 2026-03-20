// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! Graph store implementations for Converge.
//!
//! Graph stores capture structured relationships between entities.
//! Like vector stores, they are **caches** that can be rebuilt from Context.
//!
//! # Available Stores
//!
//! - [`InMemoryGraphStore`] - In-memory store for testing and small workloads
//! - `Neo4jStore` - Neo4j graph database (requires `neo4j` feature)
//! - `NebulaStore` - `NebulaGraph` database (requires `nebula` feature)
//!
//! # Example
//!
//! ```
//! use converge_provider::graph::InMemoryGraphStore;
//! use converge_core::capability::{GraphRecall, GraphNode, GraphEdge, GraphQuery};
//!
//! let store = InMemoryGraphStore::new();
//!
//! // Add nodes
//! store.add_node(&GraphNode {
//!     id: "company-1".into(),
//!     label: "Company".into(),
//!     properties: serde_json::json!({"name": "Acme Corp"}),
//! }).unwrap();
//!
//! store.add_node(&GraphNode {
//!     id: "company-2".into(),
//!     label: "Company".into(),
//!     properties: serde_json::json!({"name": "Globex Inc"}),
//! }).unwrap();
//!
//! // Add relationship
//! store.add_edge(&GraphEdge {
//!     from: "company-1".into(),
//!     to: "company-2".into(),
//!     relationship: "COMPETES_WITH".into(),
//!     properties: None,
//! }).unwrap();
//!
//! // Traverse the graph
//! let result = store.traverse(&GraphQuery::from_node("company-1")).unwrap();
//! assert_eq!(result.nodes.len(), 2);
//! ```

mod memory;

pub use memory::InMemoryGraphStore;

#[cfg(feature = "neo4j")]
mod neo4j;

// Neo4jStore will be re-exported once implemented in neo4j.rs

// Re-export core types for convenience
pub use converge_core::capability::{
    CapabilityError, GraphEdge, GraphNode, GraphQuery, GraphRecall, GraphResult,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn basic_graph_operations() {
        let store = InMemoryGraphStore::new();

        // Add companies
        store
            .add_node(&GraphNode {
                id: "acme".into(),
                label: "Company".into(),
                properties: json!({"name": "Acme Corp", "revenue": 1_000_000}),
            })
            .unwrap();

        store
            .add_node(&GraphNode {
                id: "globex".into(),
                label: "Company".into(),
                properties: json!({"name": "Globex Inc", "revenue": 2_000_000}),
            })
            .unwrap();

        // Add person
        store
            .add_node(&GraphNode {
                id: "alice".into(),
                label: "Person".into(),
                properties: json!({"name": "Alice", "role": "CEO"}),
            })
            .unwrap();

        // Add relationships
        store
            .add_edge(&GraphEdge {
                from: "alice".into(),
                to: "acme".into(),
                relationship: "WORKS_FOR".into(),
                properties: Some(json!({"since": 2020})),
            })
            .unwrap();

        store
            .add_edge(&GraphEdge {
                from: "acme".into(),
                to: "globex".into(),
                relationship: "COMPETES_WITH".into(),
                properties: None,
            })
            .unwrap();

        // Find companies
        let companies = store.find_nodes("Company", None).unwrap();
        assert_eq!(companies.len(), 2);

        // Traverse from Alice
        let result = store
            .traverse(&GraphQuery::from_node("alice").with_max_depth(2))
            .unwrap();

        // Should find Alice -> Acme -> Globex
        assert_eq!(result.nodes.len(), 3);
        assert_eq!(result.edges.len(), 2);
    }
}
