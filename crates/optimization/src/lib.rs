//! # converge-optimization
//!
//! Optimization algorithms for converge.zone - a Rust reimplementation of
//! key OR-Tools algorithms optimized for the converge platform.
//!
//! ## Modules
//!
//! - [`assignment`] - Linear assignment problem (Hungarian, Goldberg-Kennedy)
//! - [`graph`] - Graph algorithms (shortest path, max flow, min cost flow)
//! - [`knapsack`] - Knapsack problems (0-1, bounded, multidimensional)
//! - [`scheduling`] - Scheduling constraints and solvers
//! - [`setcover`] - Set cover heuristics
//! - [`provider`] - Converge platform integration
//!
//! ## Quick Start
//!
//! ```rust
//! use converge_optimization::assignment::{hungarian, AssignmentProblem};
//!
//! // Cost matrix: agent i to task j
//! let costs = vec![
//!     vec![10, 5, 13],
//!     vec![3, 9, 18],
//!     vec![14, 8, 7],
//! ];
//!
//! let problem = AssignmentProblem::from_costs(costs);
//! let solution = hungarian::solve(&problem).unwrap();
//! println!("Total cost: {}", solution.total_cost);
//! ```
//!
//! ## Feature Flags
//!
//! - `ffi` - Enable C++ OR-Tools bindings for complex algorithms
//! - `full` - Enable all features

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod assignment;
pub mod gate;
pub mod graph;
pub mod knapsack;
pub mod packs;
pub mod scheduling;
pub mod setcover;
pub mod provider;

#[cfg(feature = "sat")]
pub mod cp;

mod error;
mod types;

pub use error::{Error, Result};
pub use types::*;

/// Prelude for common imports
pub mod prelude {
    pub use crate::assignment::{AssignmentProblem, AssignmentSolution, AssignmentSolver};
    pub use crate::gate::{ProblemSpec, ProposedPlan, SolverReport, PromotionGate, GateDecision};
    pub use crate::graph::{Graph, NodeId, EdgeId};
    pub use crate::knapsack::{KnapsackProblem, KnapsackSolution, KnapsackSolver};
    pub use crate::packs::{Pack, PackRegistry, PackSolveResult};
    pub use crate::Error;
    pub use crate::Result;
}
