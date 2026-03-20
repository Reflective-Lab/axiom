//! Scheduling algorithms and constraints
//!
//! This module provides scheduling primitives that can be used
//! with constraint solvers or standalone heuristics.
//!
//! ## Concepts
//!
//! - **Interval**: A task with start, duration, and end
//! - **Disjunctive**: Tasks that cannot overlap (single resource)
//! - **Cumulative**: Tasks sharing resource capacity

use crate::Result;
use serde::{Deserialize, Serialize};

/// An interval (task) in a schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interval {
    /// Interval identifier
    pub id: usize,
    /// Earliest start time
    pub earliest_start: i64,
    /// Latest end time
    pub latest_end: i64,
    /// Duration (fixed for now)
    pub duration: i64,
    /// Resource consumption (for cumulative)
    pub demand: i64,
}

impl Interval {
    /// Create a new interval
    pub fn new(id: usize, earliest_start: i64, latest_end: i64, duration: i64) -> Self {
        Self {
            id,
            earliest_start,
            latest_end,
            duration,
            demand: 1,
        }
    }

    /// Create interval with resource demand
    pub fn with_demand(mut self, demand: i64) -> Self {
        self.demand = demand;
        self
    }

    /// Latest possible start time
    pub fn latest_start(&self) -> i64 {
        self.latest_end - self.duration
    }

    /// Earliest possible end time
    pub fn earliest_end(&self) -> i64 {
        self.earliest_start + self.duration
    }

    /// Check if interval can be scheduled
    pub fn is_feasible(&self) -> bool {
        self.earliest_start + self.duration <= self.latest_end
    }
}

/// A scheduled interval (with assigned start time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledInterval {
    /// The interval
    pub interval: Interval,
    /// Assigned start time
    pub start: i64,
}

impl ScheduledInterval {
    /// End time of scheduled interval
    pub fn end(&self) -> i64 {
        self.start + self.interval.duration
    }
}

/// A scheduling problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingProblem {
    /// Tasks to schedule
    pub intervals: Vec<Interval>,
    /// Resource capacity (for cumulative)
    pub capacity: i64,
    /// Whether tasks are disjunctive (no overlap)
    pub disjunctive: bool,
}

impl SchedulingProblem {
    /// Create a disjunctive scheduling problem
    pub fn disjunctive(intervals: Vec<Interval>) -> Self {
        Self {
            intervals,
            capacity: 1,
            disjunctive: true,
        }
    }

    /// Create a cumulative scheduling problem
    pub fn cumulative(intervals: Vec<Interval>, capacity: i64) -> Self {
        Self {
            intervals,
            capacity,
            disjunctive: false,
        }
    }
}

/// Solution to a scheduling problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingSolution {
    /// Scheduled intervals with start times
    pub schedule: Vec<ScheduledInterval>,
    /// Makespan (latest end time)
    pub makespan: i64,
}

/// Simple list scheduling heuristic for disjunctive problems
pub fn list_schedule(problem: &SchedulingProblem) -> Result<SchedulingSolution> {
    let mut schedule = Vec::new();
    let mut current_time = 0i64;

    // Sort by earliest start (EDD - Earliest Due Date variant)
    let mut intervals = problem.intervals.clone();
    intervals.sort_by_key(|i| i.earliest_start);

    for interval in intervals {
        // Find earliest feasible start
        let start = current_time.max(interval.earliest_start);

        if start + interval.duration > interval.latest_end {
            return Err(crate::Error::infeasible(format!(
                "Interval {} cannot be scheduled within time window",
                interval.id
            )));
        }

        schedule.push(ScheduledInterval {
            start,
            interval: interval.clone(),
        });

        if problem.disjunctive {
            current_time = start + interval.duration;
        }
    }

    let makespan = schedule.iter()
        .map(|s| s.end())
        .max()
        .unwrap_or(0);

    Ok(SchedulingSolution { schedule, makespan })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_disjunctive() {
        let intervals = vec![
            Interval::new(0, 0, 100, 10),
            Interval::new(1, 0, 100, 20),
            Interval::new(2, 0, 100, 15),
        ];

        let problem = SchedulingProblem::disjunctive(intervals);
        let solution = list_schedule(&problem).unwrap();

        assert_eq!(solution.schedule.len(), 3);
        assert_eq!(solution.makespan, 45); // 10 + 20 + 15
    }

    #[test]
    fn test_interval_feasibility() {
        let interval = Interval::new(0, 0, 10, 5);
        assert!(interval.is_feasible());

        let infeasible = Interval::new(0, 0, 3, 5);
        assert!(!infeasible.is_feasible());
    }
}
