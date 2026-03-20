// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! In-memory graph store for testing and small workloads.

use converge_core::capability::{
    CapabilityError, GraphEdge, GraphNode, GraphQuery, GraphRecall, GraphResult,
};
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// In-memory graph store.
///
/// This is a simple graph store that keeps all nodes and edges in memory.
/// Suitable for:
/// - Testing and development
/// - Small knowledge graphs (< 10k nodes)
/// - Ephemeral graph caches
///
/// For production workloads, use `Neo4jStore` or similar.
///
/// # Thread Safety
///
/// This store is thread-safe and can be shared across threads.
pub struct InMemoryGraphStore {
    nodes: RwLock<HashMap<String, GraphNode>>,
    // Edges indexed by source node ID for efficient traversal
    outgoing_edges: RwLock<HashMap<String, Vec<GraphEdge>>>,
    // Also index by target for reverse traversal
    incoming_edges: RwLock<HashMap<String, Vec<GraphEdge>>>,
}

impl Default for InMemoryGraphStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryGraphStore {
    /// Creates a new empty in-memory graph store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            outgoing_edges: RwLock::new(HashMap::new()),
            incoming_edges: RwLock::new(HashMap::new()),
        }
    }

    /// Returns the count of nodes.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn node_count(&self) -> usize {
        self.nodes.read().expect("Lock poisoned").len()
    }

    /// Returns the count of edges.
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    pub fn edge_count(&self) -> usize {
        self.outgoing_edges
            .read()
            .expect("Lock poisoned")
            .values()
            .map(std::vec::Vec::len)
            .sum()
    }

    /// Helper: Collect nodes reachable from start within `max_depth`
    fn collect_reachable(
        &self,
        start_nodes: &[String],
        relationships: &[String],
        max_depth: usize,
        limit: usize,
    ) -> (Vec<GraphNode>, Vec<GraphEdge>) {
        let nodes_lock = self.nodes.read().expect("Lock poisoned");
        let edges_lock = self.outgoing_edges.read().expect("Lock poisoned");

        let mut visited_nodes: HashSet<String> = HashSet::new();
        let mut visited_edges: HashSet<(String, String, String)> = HashSet::new();
        let mut result_nodes: Vec<GraphNode> = Vec::new();
        let mut result_edges: Vec<GraphEdge> = Vec::new();

        // BFS traversal
        // depth represents how many edges we've traversed
        let mut current_level: Vec<String> = start_nodes.to_vec();
        let mut depth = 0;

        while !current_level.is_empty() && result_nodes.len() < limit {
            let mut next_level: Vec<String> = Vec::new();

            for node_id in current_level {
                if visited_nodes.contains(&node_id) {
                    continue;
                }
                visited_nodes.insert(node_id.clone());

                // Add the node if it exists
                if let Some(node) = nodes_lock.get(&node_id) {
                    result_nodes.push(node.clone());
                    if result_nodes.len() >= limit {
                        break;
                    }
                }

                // Only traverse edges if we haven't reached max depth
                if depth < max_depth
                    && let Some(edges) = edges_lock.get(&node_id)
                {
                    for edge in edges {
                        // Filter by relationship type if specified
                        if !relationships.is_empty() && !relationships.contains(&edge.relationship)
                        {
                            continue;
                        }

                        let edge_key = (
                            edge.from.clone(),
                            edge.to.clone(),
                            edge.relationship.clone(),
                        );

                        if !visited_edges.contains(&edge_key) {
                            visited_edges.insert(edge_key);
                            result_edges.push(edge.clone());
                            next_level.push(edge.to.clone());
                        }
                    }
                }
            }

            current_level = next_level;
            depth += 1;
        }

        (result_nodes, result_edges)
    }
}

impl GraphRecall for InMemoryGraphStore {
    fn name(&self) -> &'static str {
        "in-memory-graph"
    }

    fn add_node(&self, node: &GraphNode) -> Result<(), CapabilityError> {
        let mut nodes = self.nodes.write().expect("Lock poisoned");
        nodes.insert(node.id.clone(), node.clone());
        Ok(())
    }

    fn add_edge(&self, edge: &GraphEdge) -> Result<(), CapabilityError> {
        // Verify source node exists
        {
            let nodes = self.nodes.read().expect("Lock poisoned");
            if !nodes.contains_key(&edge.from) {
                return Err(CapabilityError::not_found(format!(
                    "Source node '{}' not found",
                    edge.from
                )));
            }
            if !nodes.contains_key(&edge.to) {
                return Err(CapabilityError::not_found(format!(
                    "Target node '{}' not found",
                    edge.to
                )));
            }
        }

        // Add to outgoing edges
        {
            let mut outgoing = self.outgoing_edges.write().expect("Lock poisoned");
            outgoing
                .entry(edge.from.clone())
                .or_default()
                .push(edge.clone());
        }

        // Add to incoming edges
        {
            let mut incoming = self.incoming_edges.write().expect("Lock poisoned");
            incoming
                .entry(edge.to.clone())
                .or_default()
                .push(edge.clone());
        }

        Ok(())
    }

    fn traverse(&self, query: &GraphQuery) -> Result<GraphResult, CapabilityError> {
        let (nodes, edges) = self.collect_reachable(
            &query.start_nodes,
            &query.relationships,
            query.max_depth,
            query.limit,
        );

        Ok(GraphResult { nodes, edges })
    }

    fn find_nodes(
        &self,
        label: &str,
        properties: Option<&serde_json::Value>,
    ) -> Result<Vec<GraphNode>, CapabilityError> {
        let nodes = self.nodes.read().expect("Lock poisoned");

        let result: Vec<GraphNode> = nodes
            .values()
            .filter(|node| {
                // Match label
                if node.label != label {
                    return false;
                }

                // Match properties if specified
                if let Some(required_props) = properties
                    && let Some(required_obj) = required_props.as_object()
                {
                    if let Some(node_obj) = node.properties.as_object() {
                        for (key, value) in required_obj {
                            if node_obj.get(key) != Some(value) {
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        Ok(result)
    }

    fn get_node(&self, id: &str) -> Result<Option<GraphNode>, CapabilityError> {
        let nodes = self.nodes.read().expect("Lock poisoned");
        Ok(nodes.get(id).cloned())
    }

    fn delete_node(&self, id: &str) -> Result<(), CapabilityError> {
        // Remove the node
        {
            let mut nodes = self.nodes.write().expect("Lock poisoned");
            nodes.remove(id);
        }

        // Remove outgoing edges
        {
            let mut outgoing = self.outgoing_edges.write().expect("Lock poisoned");
            outgoing.remove(id);
        }

        // Remove incoming edges that point to this node
        {
            let mut incoming = self.incoming_edges.write().expect("Lock poisoned");
            incoming.remove(id);

            // Also remove from other nodes' outgoing edges
            let mut outgoing = self.outgoing_edges.write().expect("Lock poisoned");
            for edges in outgoing.values_mut() {
                edges.retain(|e| e.to != id);
            }
        }

        Ok(())
    }

    fn clear(&self) -> Result<(), CapabilityError> {
        {
            let mut nodes = self.nodes.write().expect("Lock poisoned");
            nodes.clear();
        }
        {
            let mut outgoing = self.outgoing_edges.write().expect("Lock poisoned");
            outgoing.clear();
        }
        {
            let mut incoming = self.incoming_edges.write().expect("Lock poisoned");
            incoming.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_graph() -> InMemoryGraphStore {
        let store = InMemoryGraphStore::new();

        // Create a simple company graph
        // Alice -[WORKS_FOR]-> Acme -[COMPETES_WITH]-> Globex
        // Bob -[WORKS_FOR]-> Globex

        store
            .add_node(&GraphNode {
                id: "alice".into(),
                label: "Person".into(),
                properties: json!({"name": "Alice", "role": "CEO"}),
            })
            .unwrap();

        store
            .add_node(&GraphNode {
                id: "bob".into(),
                label: "Person".into(),
                properties: json!({"name": "Bob", "role": "CTO"}),
            })
            .unwrap();

        store
            .add_node(&GraphNode {
                id: "acme".into(),
                label: "Company".into(),
                properties: json!({"name": "Acme Corp"}),
            })
            .unwrap();

        store
            .add_node(&GraphNode {
                id: "globex".into(),
                label: "Company".into(),
                properties: json!({"name": "Globex Inc"}),
            })
            .unwrap();

        store
            .add_edge(&GraphEdge {
                from: "alice".into(),
                to: "acme".into(),
                relationship: "WORKS_FOR".into(),
                properties: None,
            })
            .unwrap();

        store
            .add_edge(&GraphEdge {
                from: "bob".into(),
                to: "globex".into(),
                relationship: "WORKS_FOR".into(),
                properties: None,
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

        store
    }

    #[test]
    fn add_and_get_node() {
        let store = InMemoryGraphStore::new();

        store
            .add_node(&GraphNode {
                id: "node-1".into(),
                label: "Test".into(),
                properties: json!({"key": "value"}),
            })
            .unwrap();

        let node = store.get_node("node-1").unwrap();
        assert!(node.is_some());
        assert_eq!(node.unwrap().label, "Test");

        let missing = store.get_node("missing").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn add_edge_requires_nodes() {
        let store = InMemoryGraphStore::new();

        let result = store.add_edge(&GraphEdge {
            from: "missing".into(),
            to: "also-missing".into(),
            relationship: "TEST".into(),
            properties: None,
        });

        assert!(result.is_err());
    }

    #[test]
    fn traverse_depth_1() {
        let store = create_test_graph();

        let result = store
            .traverse(&GraphQuery::from_node("alice").with_max_depth(1))
            .unwrap();

        // Should find: Alice, Acme
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
    }

    #[test]
    fn traverse_depth_2() {
        let store = create_test_graph();

        let result = store
            .traverse(&GraphQuery::from_node("alice").with_max_depth(2))
            .unwrap();

        // Should find: Alice, Acme, Globex
        assert_eq!(result.nodes.len(), 3);
        assert_eq!(result.edges.len(), 2);
    }

    #[test]
    fn traverse_with_relationship_filter() {
        let store = create_test_graph();

        let result = store
            .traverse(
                &GraphQuery::from_node("alice")
                    .with_relationships(vec!["WORKS_FOR".into()])
                    .with_max_depth(10),
            )
            .unwrap();

        // Should find: Alice, Acme (COMPETES_WITH is filtered out)
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);
    }

    #[test]
    fn find_by_label() {
        let store = create_test_graph();

        let companies = store.find_nodes("Company", None).unwrap();
        assert_eq!(companies.len(), 2);

        let people = store.find_nodes("Person", None).unwrap();
        assert_eq!(people.len(), 2);
    }

    #[test]
    fn find_by_label_and_properties() {
        let store = create_test_graph();

        let ceos = store
            .find_nodes("Person", Some(&json!({"role": "CEO"})))
            .unwrap();
        assert_eq!(ceos.len(), 1);
        assert_eq!(ceos[0].id, "alice");
    }

    #[test]
    fn delete_node() {
        let store = create_test_graph();

        assert_eq!(store.node_count(), 4);

        store.delete_node("alice").unwrap();

        assert_eq!(store.node_count(), 3);
        assert!(store.get_node("alice").unwrap().is_none());
    }

    #[test]
    fn clear() {
        let store = create_test_graph();

        assert!(store.node_count() > 0);
        assert!(store.edge_count() > 0);

        store.clear().unwrap();

        assert_eq!(store.node_count(), 0);
        assert_eq!(store.edge_count(), 0);
    }
}
