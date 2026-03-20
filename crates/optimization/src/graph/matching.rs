//! Graph matching algorithms
//!
//! - Bipartite matching
//! - Maximum cardinality matching

use crate::Result;

// TODO: Implement bipartite matching (Hopcroft-Karp)
// TODO: Implement maximum cardinality matching

/// Matching result
#[derive(Debug, Clone)]
pub struct Matching {
    /// Matched pairs (node_a, node_b)
    pub pairs: Vec<(usize, usize)>,
    /// Size of matching
    pub size: usize,
}

/// Placeholder for bipartite matching
pub fn bipartite_matching() -> Result<Matching> {
    todo!("Bipartite matching not yet implemented")
}
