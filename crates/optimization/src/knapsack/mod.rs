//! Knapsack problem solvers
//!
//! The knapsack problem: select items to maximize value within capacity.
//!
//! ## Variants
//!
//! - 0-1 Knapsack: Take or leave each item
//! - Bounded: Limited copies of each item
//! - Unbounded: Unlimited copies
//! - Multidimensional: Multiple capacity constraints

use crate::{Error, Result, SolverParams, SolverStats, SolverStatus, Value, Weight};
use serde::{Deserialize, Serialize};

/// A knapsack problem instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnapsackProblem {
    /// Weight of each item
    pub weights: Vec<Weight>,
    /// Value of each item
    pub values: Vec<Value>,
    /// Knapsack capacity
    pub capacity: Weight,
}

impl KnapsackProblem {
    /// Create a new knapsack problem
    pub fn new(weights: Vec<Weight>, values: Vec<Value>, capacity: Weight) -> Result<Self> {
        if weights.len() != values.len() {
            return Err(Error::dimension_mismatch(weights.len(), values.len()));
        }
        Ok(Self { weights, values, capacity })
    }

    /// Number of items
    pub fn num_items(&self) -> usize {
        self.weights.len()
    }

    /// Validate the problem
    pub fn validate(&self) -> Result<()> {
        if self.weights.len() != self.values.len() {
            return Err(Error::dimension_mismatch(
                self.weights.len(),
                self.values.len(),
            ));
        }
        if self.capacity < 0 {
            return Err(Error::invalid_input("negative capacity"));
        }
        for &w in &self.weights {
            if w < 0 {
                return Err(Error::invalid_input("negative weight"));
            }
        }
        Ok(())
    }
}

/// Solution to a knapsack problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnapsackSolution {
    /// Selected items (indices)
    pub selected: Vec<usize>,
    /// Total value of selected items
    pub total_value: Value,
    /// Total weight of selected items
    pub total_weight: Weight,
    /// Solver status
    pub status: SolverStatus,
    /// Solver statistics
    pub stats: SolverStats,
}

/// Trait for knapsack solvers
pub trait KnapsackSolver {
    /// Solve the knapsack problem
    fn solve(&self, problem: &KnapsackProblem, params: &SolverParams) -> Result<KnapsackSolution>;

    /// Solver name
    fn name(&self) -> &'static str;
}

/// Dynamic programming solver for 0-1 knapsack
pub struct DynamicProgrammingSolver;

impl KnapsackSolver for DynamicProgrammingSolver {
    fn solve(&self, problem: &KnapsackProblem, _params: &SolverParams) -> Result<KnapsackSolution> {
        solve_dp(problem)
    }

    fn name(&self) -> &'static str {
        "dynamic_programming"
    }
}

/// Solve 0-1 knapsack using dynamic programming
pub fn solve(problem: &KnapsackProblem) -> Result<KnapsackSolution> {
    problem.validate()?;
    solve_dp(problem)
}

fn solve_dp(problem: &KnapsackProblem) -> Result<KnapsackSolution> {
    let start = std::time::Instant::now();
    let n = problem.num_items();
    let capacity = problem.capacity as usize;

    if n == 0 || capacity == 0 {
        return Ok(KnapsackSolution {
            selected: vec![],
            total_value: 0,
            total_weight: 0,
            status: SolverStatus::Optimal,
            stats: SolverStats::default(),
        });
    }

    // Check for overflow potential
    if capacity > 10_000_000 {
        return Err(Error::invalid_input(
            "capacity too large for DP (use branch-and-bound instead)"
        ));
    }

    // dp[w] = max value achievable with capacity w
    let mut dp = vec![0i64; capacity + 1];

    // Track which items were used
    let mut keep = vec![vec![false; capacity + 1]; n];

    for i in 0..n {
        let w = problem.weights[i] as usize;
        let v = problem.values[i];

        // Process in reverse to avoid using item multiple times
        for c in (w..=capacity).rev() {
            if dp[c - w] + v > dp[c] {
                dp[c] = dp[c - w] + v;
                keep[i][c] = true;
            }
        }
    }

    // Backtrack to find selected items
    let mut selected = Vec::new();
    let mut remaining = capacity;

    for i in (0..n).rev() {
        if keep[i][remaining] {
            selected.push(i);
            remaining -= problem.weights[i] as usize;
        }
    }

    selected.reverse();

    let total_value = dp[capacity];
    let total_weight: Weight = selected.iter()
        .map(|&i| problem.weights[i])
        .sum();

    let elapsed = start.elapsed().as_secs_f64();

    Ok(KnapsackSolution {
        selected,
        total_value,
        total_weight,
        status: SolverStatus::Optimal,
        stats: SolverStats {
            solve_time_seconds: elapsed,
            iterations: n * capacity,
            objective_value: Some(total_value as f64),
            ..Default::default()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_knapsack() {
        let problem = KnapsackProblem::new(
            vec![10, 20, 30],
            vec![60, 100, 120],
            50,
        ).unwrap();

        let solution = solve(&problem).unwrap();

        // Optimal: items 1 and 2 (weights 20+30=50, values 100+120=220)
        assert_eq!(solution.total_value, 220);
        assert_eq!(solution.total_weight, 50);
        assert!(solution.selected.contains(&1));
        assert!(solution.selected.contains(&2));
    }

    #[test]
    fn test_empty_knapsack() {
        let problem = KnapsackProblem::new(vec![], vec![], 100).unwrap();
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_value, 0);
        assert!(solution.selected.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let problem = KnapsackProblem::new(vec![10, 20], vec![100, 200], 0).unwrap();
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_value, 0);
    }

    #[test]
    fn test_single_item_fits() {
        let problem = KnapsackProblem::new(vec![5], vec![10], 10).unwrap();
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_value, 10);
        assert_eq!(solution.selected, vec![0]);
    }

    #[test]
    fn test_single_item_too_heavy() {
        let problem = KnapsackProblem::new(vec![15], vec![10], 10).unwrap();
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_value, 0);
        assert!(solution.selected.is_empty());
    }
}
