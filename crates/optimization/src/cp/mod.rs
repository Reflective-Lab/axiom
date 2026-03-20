//! Constraint Programming solver built on SAT
//!
//! This module provides a simple CP solver that encodes constraints to SAT
//! using Varisat as the underlying SAT engine.
//!
//! ## Example
//!
//! ```rust,ignore
//! use converge_optimization::cp::{CpModel, CpStatus};
//!
//! let mut model = CpModel::new();
//! let x = model.new_int_var(0, 10, "x");
//! let y = model.new_int_var(0, 10, "y");
//!
//! // x + y = 10
//! model.add_linear_eq(&[x, y], &[1, 1], 10);
//!
//! // Minimize x
//! model.minimize(&[x], &[1]);
//!
//! let solution = model.solve();
//! assert_eq!(solution.status, CpStatus::Optimal);
//! ```

#[cfg(feature = "sat")]
mod solver;

#[cfg(feature = "sat")]
pub use solver::*;
