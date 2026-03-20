//! Auction Algorithm for the Linear Assignment Problem
//!
//! The auction algorithm uses a market-based approach where agents
//! "bid" for tasks. It's particularly efficient for sparse problems.
//!
//! ## Algorithm Overview
//!
//! 1. Agents bid on preferred tasks
//! 2. Tasks are "sold" to highest bidder
//! 3. Prices increase for contested tasks
//! 4. Repeat until all agents assigned
//!
//! Time complexity: O(n³) worst case, often faster in practice.

use super::{AssignmentProblem, AssignmentSolution, AssignmentSolver};
use crate::{Cost, Result, SolverParams, SolverStats, SolverStatus};
use std::time::Instant;

/// Auction algorithm solver
pub struct AuctionSolver {
    /// Epsilon for ε-complementary slackness (default: 1)
    pub epsilon: i64,
}

impl Default for AuctionSolver {
    fn default() -> Self {
        Self { epsilon: 1 }
    }
}

impl AssignmentSolver for AuctionSolver {
    fn solve(&self, problem: &AssignmentProblem, params: &SolverParams) -> Result<AssignmentSolution> {
        solve_auction(problem, params, self.epsilon)
    }

    fn name(&self) -> &'static str {
        "auction"
    }
}

/// Solve using auction algorithm with default epsilon
pub fn solve(problem: &AssignmentProblem) -> Result<AssignmentSolution> {
    solve_auction(problem, &SolverParams::default(), 1)
}

fn solve_auction(
    problem: &AssignmentProblem,
    params: &SolverParams,
    epsilon: i64,
) -> Result<AssignmentSolution> {
    let start = Instant::now();
    let n = problem.num_agents;

    if n == 0 || problem.num_tasks == 0 {
        return Err(crate::Error::invalid_input("empty problem"));
    }

    if !problem.is_square() {
        // For now, fall back to Hungarian for non-square
        return super::hungarian::solve(problem);
    }

    // Convert to benefits (negate costs for maximization)
    let max_cost: i64 = problem.costs.iter()
        .flat_map(|row| row.iter())
        .max()
        .copied()
        .unwrap_or(0);

    let benefit: Vec<Vec<i64>> = problem.costs.iter()
        .map(|row| row.iter().map(|&c| max_cost - c).collect())
        .collect();

    // Prices of tasks
    let mut prices = vec![0i64; n];

    // Assignment: assignment[agent] = task (-1 means unassigned)
    let mut assignment = vec![-1i32; n];

    // Reverse assignment: task_owner[task] = agent (-1 means unowned)
    let mut task_owner = vec![-1i32; n];

    let mut iterations = 0;
    let mut unassigned: Vec<usize> = (0..n).collect();

    while !unassigned.is_empty() {
        iterations += 1;

        // Check timeout
        if params.has_time_limit() && start.elapsed().as_secs_f64() > params.time_limit_seconds {
            return Err(crate::Error::timeout(params.time_limit_seconds));
        }

        // Process one unassigned agent
        let agent = unassigned.pop().unwrap();

        // Find best and second-best task values
        let mut best_task = 0;
        let mut best_value = i64::MIN;
        let mut second_best_value = i64::MIN;

        for task in 0..n {
            let value = benefit[agent][task] - prices[task];
            if value > best_value {
                second_best_value = best_value;
                best_value = value;
                best_task = task;
            } else if value > second_best_value {
                second_best_value = value;
            }
        }

        // Handle case where there's only one task
        if second_best_value == i64::MIN {
            second_best_value = best_value;
        }

        // Calculate bid increment
        let bid_increment = best_value - second_best_value + epsilon;

        // If task is owned by another agent, unassign them
        let prev_owner = task_owner[best_task];
        if prev_owner >= 0 {
            assignment[prev_owner as usize] = -1;
            unassigned.push(prev_owner as usize);
        }

        // Assign agent to task
        assignment[agent] = best_task as i32;
        task_owner[best_task] = agent as i32;
        prices[best_task] += bid_increment;
    }

    // Convert assignment to solution
    let assignments: Vec<usize> = assignment.iter().map(|&t| t as usize).collect();
    let total_cost: Cost = assignments.iter()
        .enumerate()
        .map(|(agent, &task)| problem.costs[agent][task])
        .sum();

    let elapsed = start.elapsed().as_secs_f64();

    Ok(AssignmentSolution {
        assignments,
        total_cost,
        status: SolverStatus::Optimal,
        stats: SolverStats {
            solve_time_seconds: elapsed,
            iterations,
            objective_value: Some(total_cost as f64),
            ..Default::default()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3x3_auction() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![10, 5, 13],
            vec![3, 9, 18],
            vec![14, 8, 7],
        ]);
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_cost, 15);
    }

    #[test]
    fn test_2x2_auction() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![1, 2],
            vec![3, 4],
        ]);
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_cost, 5);
    }
}
