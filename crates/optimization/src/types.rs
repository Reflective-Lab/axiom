//! Common types used across optimization modules

use serde::{Deserialize, Serialize};

/// Cost type for optimization problems (signed to allow negative costs)
pub type Cost = i64;

/// Weight type for capacity constraints
pub type Weight = i64;

/// Value type for objective functions
pub type Value = i64;

/// Index type for nodes/variables
pub type Index = usize;

/// Solver status after optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverStatus {
    /// Optimal solution found
    Optimal,
    /// Feasible solution found (may not be optimal)
    Feasible,
    /// Problem is infeasible
    Infeasible,
    /// Problem is unbounded
    Unbounded,
    /// Solver timed out
    Timeout,
    /// Solver hit iteration limit
    IterationLimit,
    /// Unknown status
    Unknown,
}

impl SolverStatus {
    /// Returns true if a solution was found
    pub fn has_solution(self) -> bool {
        matches!(self, Self::Optimal | Self::Feasible)
    }

    /// Returns true if the solution is proven optimal
    pub fn is_optimal(self) -> bool {
        matches!(self, Self::Optimal)
    }
}

/// Statistics from a solver run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SolverStats {
    /// Time spent solving (seconds)
    pub solve_time_seconds: f64,
    /// Number of iterations
    pub iterations: usize,
    /// Number of nodes explored (for tree search)
    pub nodes_explored: usize,
    /// Best objective value found
    pub objective_value: Option<f64>,
    /// Best bound (for MIP)
    pub best_bound: Option<f64>,
    /// Gap between objective and bound
    pub gap: Option<f64>,
}

/// Common solver parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverParams {
    /// Maximum solve time in seconds (0 = unlimited)
    pub time_limit_seconds: f64,
    /// Maximum iterations (0 = unlimited)
    pub iteration_limit: usize,
    /// Number of threads to use (0 = auto)
    pub num_threads: usize,
    /// Random seed for reproducibility
    pub random_seed: u64,
    /// Verbosity level (0 = silent)
    pub verbosity: u32,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            time_limit_seconds: 0.0,
            iteration_limit: 0,
            num_threads: 0,
            random_seed: 0,
            verbosity: 0,
        }
    }
}

impl SolverParams {
    /// Create params with a time limit
    pub fn with_time_limit(seconds: f64) -> Self {
        Self {
            time_limit_seconds: seconds,
            ..Default::default()
        }
    }

    /// Check if time limit is set
    pub fn has_time_limit(&self) -> bool {
        self.time_limit_seconds > 0.0
    }

    /// Check if iteration limit is set
    pub fn has_iteration_limit(&self) -> bool {
        self.iteration_limit > 0
    }
}
