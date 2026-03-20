//! Set Cover problem solvers
//!
//! The set cover problem: find minimum cost collection of sets
//! that covers all elements.
//!
//! ## Algorithms
//!
//! - Greedy: O(n²) approximation with ln(n) guarantee
//! - Local search: Improve greedy solution

use crate::{Cost, Error, Result, SolverStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A set cover problem instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCoverProblem {
    /// Number of elements to cover
    pub num_elements: usize,
    /// Sets: each set is (cost, elements)
    pub sets: Vec<(Cost, Vec<usize>)>,
}

impl SetCoverProblem {
    /// Create a new set cover problem
    pub fn new(num_elements: usize, sets: Vec<(Cost, Vec<usize>)>) -> Result<Self> {
        // Validate elements are in range
        for (_, elements) in &sets {
            for &e in elements {
                if e >= num_elements {
                    return Err(Error::invalid_input(format!(
                        "element {} out of range [0, {})",
                        e, num_elements
                    )));
                }
            }
        }
        Ok(Self { num_elements, sets })
    }

    /// Create problem with unit costs
    pub fn unit_cost(num_elements: usize, sets: Vec<Vec<usize>>) -> Result<Self> {
        let sets_with_cost = sets.into_iter()
            .map(|s| (1, s))
            .collect();
        Self::new(num_elements, sets_with_cost)
    }

    /// Number of sets
    pub fn num_sets(&self) -> usize {
        self.sets.len()
    }
}

/// Solution to a set cover problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCoverSolution {
    /// Selected set indices
    pub selected: Vec<usize>,
    /// Total cost
    pub total_cost: Cost,
    /// Status
    pub status: SolverStatus,
}

/// Greedy set cover algorithm
///
/// At each step, select the set with best cost-effectiveness
/// (cost / number of new elements covered).
pub fn greedy(problem: &SetCoverProblem) -> Result<SetCoverSolution> {
    let mut uncovered: HashSet<usize> = (0..problem.num_elements).collect();
    let mut selected = Vec::new();
    let mut total_cost: Cost = 0;

    while !uncovered.is_empty() {
        // Find best set (minimum cost per new element)
        let mut best_set = None;
        let mut best_ratio = f64::INFINITY;

        for (idx, (cost, elements)) in problem.sets.iter().enumerate() {
            if selected.contains(&idx) {
                continue;
            }

            let new_covered: usize = elements.iter()
                .filter(|e| uncovered.contains(e))
                .count();

            if new_covered == 0 {
                continue;
            }

            let ratio = *cost as f64 / new_covered as f64;
            if ratio < best_ratio {
                best_ratio = ratio;
                best_set = Some(idx);
            }
        }

        match best_set {
            Some(idx) => {
                let (cost, elements) = &problem.sets[idx];
                selected.push(idx);
                total_cost += cost;
                for &e in elements {
                    uncovered.remove(&e);
                }
            }
            None => {
                return Err(Error::infeasible(
                    "not all elements can be covered"
                ));
            }
        }
    }

    Ok(SetCoverSolution {
        selected,
        total_cost,
        status: SolverStatus::Feasible, // Greedy is not optimal
    })
}

/// Solve set cover using greedy algorithm
pub fn solve(problem: &SetCoverProblem) -> Result<SetCoverSolution> {
    greedy(problem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_cover() {
        // Elements: {0, 1, 2, 3, 4}
        // Sets:
        //   0: {0, 1, 2} cost 1
        //   1: {2, 3} cost 1
        //   2: {3, 4} cost 1
        //   3: {4, 0} cost 1
        let problem = SetCoverProblem::new(
            5,
            vec![
                (1, vec![0, 1, 2]),
                (1, vec![2, 3]),
                (1, vec![3, 4]),
                (1, vec![4, 0]),
            ],
        ).unwrap();

        let solution = solve(&problem).unwrap();

        // Verify all elements covered
        let mut covered = HashSet::new();
        for &idx in &solution.selected {
            for &e in &problem.sets[idx].1 {
                covered.insert(e);
            }
        }
        assert_eq!(covered.len(), 5);
    }

    #[test]
    fn test_unit_cost() {
        let problem = SetCoverProblem::unit_cost(
            3,
            vec![vec![0, 1], vec![1, 2], vec![0, 2]],
        ).unwrap();

        let solution = solve(&problem).unwrap();
        assert!(solution.total_cost <= 2); // Can be done with 2 sets
    }

    #[test]
    fn test_infeasible() {
        // Element 2 not in any set
        let problem = SetCoverProblem::new(
            3,
            vec![(1, vec![0]), (1, vec![1])],
        ).unwrap();

        let result = solve(&problem);
        assert!(result.is_err());
    }
}
