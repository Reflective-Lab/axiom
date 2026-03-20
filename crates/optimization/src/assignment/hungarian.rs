//! Hungarian Algorithm for the Linear Assignment Problem
//!
//! The Hungarian algorithm (also known as Kuhn-Munkres) finds the optimal
//! assignment in O(n³) time for an n×n cost matrix.
//!
//! ## Algorithm Overview
//!
//! 1. Subtract row minima from each row
//! 2. Subtract column minima from each column
//! 3. Cover all zeros with minimum lines
//! 4. If lines < n, create more zeros and repeat
//! 5. Find optimal assignment using covered zeros

use super::{AssignmentProblem, AssignmentSolution, AssignmentSolver};
use crate::{Cost, Error, Result, SolverParams, SolverStats, SolverStatus};
use std::time::Instant;

/// Hungarian algorithm solver
pub struct HungarianSolver;

impl AssignmentSolver for HungarianSolver {
    fn solve(&self, problem: &AssignmentProblem, params: &SolverParams) -> Result<AssignmentSolution> {
        solve_hungarian(problem, params)
    }

    fn name(&self) -> &'static str {
        "hungarian"
    }
}

/// Solve using Hungarian algorithm
pub fn solve(problem: &AssignmentProblem) -> Result<AssignmentSolution> {
    solve_hungarian(problem, &SolverParams::default())
}

fn solve_hungarian(problem: &AssignmentProblem, params: &SolverParams) -> Result<AssignmentSolution> {
    let start = Instant::now();

    let n = problem.num_agents;
    let m = problem.num_tasks;

    if n == 0 || m == 0 {
        return Err(Error::invalid_input("empty problem"));
    }

    // For rectangular problems, pad to square
    let size = n.max(m);
    let mut cost = vec![vec![0i64; size]; size];

    // Copy costs (use large value for padding)
    let large = problem.costs.iter()
        .flat_map(|row| row.iter())
        .max()
        .copied()
        .unwrap_or(0)
        .saturating_add(1);

    for i in 0..size {
        for j in 0..size {
            cost[i][j] = if i < n && j < m {
                problem.costs[i][j]
            } else {
                large // Padding cost
            };
        }
    }

    // u[i] and v[j] are the dual variables (potentials)
    let mut u = vec![0i64; size + 1];
    let mut v = vec![0i64; size + 1];

    // p[j] = agent assigned to task j (0 means unassigned, 1-indexed)
    let mut p = vec![0usize; size + 1];

    // way[j] = previous task in augmenting path to j
    let mut way = vec![0usize; size + 1];

    let mut iterations = 0;

    for i in 1..=size {
        // Check timeout
        if params.has_time_limit() && start.elapsed().as_secs_f64() > params.time_limit_seconds {
            return Err(Error::timeout(params.time_limit_seconds));
        }

        p[0] = i;
        let mut j0 = 0usize; // Current task (0 = virtual)

        let mut minv = vec![i64::MAX; size + 1];
        let mut used = vec![false; size + 1];

        // Find augmenting path
        loop {
            iterations += 1;
            used[j0] = true;
            let i0 = p[j0];
            let mut delta = i64::MAX;
            let mut j1 = 0usize;

            for j in 1..=size {
                if !used[j] {
                    let cur = cost[i0 - 1][j - 1] - u[i0] - v[j];
                    if cur < minv[j] {
                        minv[j] = cur;
                        way[j] = j0;
                    }
                    if minv[j] < delta {
                        delta = minv[j];
                        j1 = j;
                    }
                }
            }

            // Update potentials
            for j in 0..=size {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }

            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }

        // Reconstruct path
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }

    // Extract solution (convert from task->agent to agent->task)
    let mut assignments = vec![0usize; n];
    let mut total_cost: Cost = 0;

    for j in 1..=size {
        if p[j] != 0 && p[j] <= n && j <= m {
            let agent = p[j] - 1;
            let task = j - 1;
            assignments[agent] = task;
            total_cost += problem.costs[agent][task];
        }
    }

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
    fn test_3x3() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![10, 5, 13],
            vec![3, 9, 18],
            vec![14, 8, 7],
        ]);
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_cost, 15);
        assert_eq!(solution.status, SolverStatus::Optimal);
    }

    #[test]
    fn test_1x1() {
        let problem = AssignmentProblem::from_costs(vec![vec![42]]);
        let solution = solve(&problem).unwrap();
        assert_eq!(solution.total_cost, 42);
        assert_eq!(solution.assignments, vec![0]);
    }

    #[test]
    fn test_2x2() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![1, 2],
            vec![3, 4],
        ]);
        let solution = solve(&problem).unwrap();
        // Optimal: (0,0)=1 + (1,1)=4 = 5, or (0,1)=2 + (1,0)=3 = 5
        assert_eq!(solution.total_cost, 5);
    }

    #[test]
    fn test_negative_costs() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![-1, -2],
            vec![-3, -4],
        ]);
        let solution = solve(&problem).unwrap();
        // Optimal: minimize -> most negative = -1 + -4 = -5 or -2 + -3 = -5
        assert_eq!(solution.total_cost, -5);
    }

    #[test]
    fn test_larger() {
        let problem = AssignmentProblem::from_costs(vec![
            vec![7, 53, 183, 439],
            vec![497, 383, 563, 79],
            vec![627, 343, 773, 959],
            vec![447, 283, 463, 29],
        ]);
        let solution = solve(&problem).unwrap();
        // Known optimal: 7 + 383 + 773 + 29 = 1192? Let's verify
        // Actually: 7 + 79 + 343 + 463 = 892 is better
        // Best: agent0->task0(7), agent1->task3(79), agent2->task1(343), agent3->task2(463) = 892
        assert!(solution.total_cost <= 892);
    }
}
