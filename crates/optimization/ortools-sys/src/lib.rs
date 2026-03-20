//! FFI bindings to Google OR-Tools (CP-SAT and GLOP only)
//!
//! This crate provides low-level bindings to OR-Tools CP-SAT and GLOP solvers.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_double, c_int};

// ============================================================================
// Common Types
// ============================================================================

/// Solve status codes matching OR-Tools
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrtoolsStatus {
    Unknown = 0,
    Optimal = 1,
    Feasible = 2,
    Infeasible = 3,
    Unbounded = 4,
    ModelInvalid = 5,
    Error = 6,
}

impl OrtoolsStatus {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Optimal | Self::Feasible)
    }
}

// ============================================================================
// Opaque pointer types
// ============================================================================

#[repr(C)]
pub struct CpModelBuilder {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CpSolverResponse {
    _private: [u8; 0],
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LpSolverType {
    Glop = 0,
    Clp = 1,
    Cbc = 2,
    Scip = 3,
}

#[repr(C)]
pub struct MpSolver {
    _private: [u8; 0],
}

// ============================================================================
// FFI function declarations
// ============================================================================

#[cfg(feature = "link")]
extern "C" {
    // CP-SAT
    pub fn cpmodel_new() -> *mut CpModelBuilder;
    pub fn cpmodel_free(model: *mut CpModelBuilder);
    pub fn cpmodel_new_int_var(
        model: *mut CpModelBuilder,
        lb: i64,
        ub: i64,
        name: *const c_char,
    ) -> i32;
    pub fn cpmodel_new_bool_var(model: *mut CpModelBuilder, name: *const c_char) -> i32;
    pub fn cpmodel_add_linear_le(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        coeffs: *const i64,
        num_vars: usize,
        rhs: i64,
    );
    pub fn cpmodel_add_linear_ge(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        coeffs: *const i64,
        num_vars: usize,
        rhs: i64,
    );
    pub fn cpmodel_add_linear_eq(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        coeffs: *const i64,
        num_vars: usize,
        rhs: i64,
    );
    pub fn cpmodel_add_all_different(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        num_vars: usize,
    );
    pub fn cpmodel_minimize(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        coeffs: *const i64,
        num_vars: usize,
    );
    pub fn cpmodel_maximize(
        model: *mut CpModelBuilder,
        var_indices: *const i32,
        coeffs: *const i64,
        num_vars: usize,
    );
    pub fn cpmodel_solve(model: *mut CpModelBuilder, time_limit: c_double) -> *mut CpSolverResponse;
    pub fn cpresponse_status(response: *const CpSolverResponse) -> OrtoolsStatus;
    pub fn cpresponse_objective_value(response: *const CpSolverResponse) -> i64;
    pub fn cpresponse_value(response: *const CpSolverResponse, var_index: i32) -> i64;
    pub fn cpresponse_wall_time(response: *const CpSolverResponse) -> c_double;
    pub fn cpresponse_free(response: *mut CpSolverResponse);

    // GLOP / Linear Solver
    pub fn mpsolver_new(name: *const c_char, solver_type: LpSolverType) -> *mut MpSolver;
    pub fn mpsolver_free(solver: *mut MpSolver);
    pub fn mpsolver_num_var(
        solver: *mut MpSolver,
        lb: c_double,
        ub: c_double,
        name: *const c_char,
    ) -> i32;
    pub fn mpsolver_int_var(
        solver: *mut MpSolver,
        lb: c_double,
        ub: c_double,
        name: *const c_char,
    ) -> i32;
    pub fn mpsolver_bool_var(solver: *mut MpSolver, name: *const c_char) -> i32;
    pub fn mpsolver_add_constraint(
        solver: *mut MpSolver,
        lb: c_double,
        ub: c_double,
        name: *const c_char,
    ) -> i32;
    pub fn mpsolver_set_constraint_coeff(
        solver: *mut MpSolver,
        constraint_idx: c_int,
        var_idx: c_int,
        coeff: c_double,
    );
    pub fn mpsolver_set_objective_coeff(solver: *mut MpSolver, var_idx: c_int, coeff: c_double);
    pub fn mpsolver_minimize(solver: *mut MpSolver);
    pub fn mpsolver_maximize(solver: *mut MpSolver);
    pub fn mpsolver_solve(solver: *mut MpSolver) -> OrtoolsStatus;
    pub fn mpsolver_objective_value(solver: *const MpSolver) -> c_double;
    pub fn mpsolver_var_value(solver: *const MpSolver, var_idx: c_int) -> c_double;
}

// ============================================================================
// Safe Rust wrappers
// ============================================================================

#[cfg(feature = "link")]
pub mod safe {
    //! Safe Rust wrappers around the FFI functions.

    use super::*;
    use std::ffi::CString;
    use std::ptr::NonNull;

    /// Safe wrapper for CP-SAT model
    pub struct CpModel {
        ptr: NonNull<CpModelBuilder>,
        num_vars: usize,
    }

    impl CpModel {
        pub fn new() -> Self {
            unsafe {
                Self {
                    ptr: NonNull::new(cpmodel_new()).expect("failed to create CP model"),
                    num_vars: 0,
                }
            }
        }

        pub fn new_int_var(&mut self, lb: i64, ub: i64, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe {
                let idx = cpmodel_new_int_var(self.ptr.as_ptr(), lb, ub, c_name.as_ptr());
                self.num_vars += 1;
                idx
            }
        }

        pub fn new_bool_var(&mut self, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe {
                let idx = cpmodel_new_bool_var(self.ptr.as_ptr(), c_name.as_ptr());
                self.num_vars += 1;
                idx
            }
        }

        pub fn add_linear_le(&mut self, vars: &[i32], coeffs: &[i64], rhs: i64) {
            assert_eq!(vars.len(), coeffs.len());
            unsafe {
                cpmodel_add_linear_le(
                    self.ptr.as_ptr(),
                    vars.as_ptr(),
                    coeffs.as_ptr(),
                    vars.len(),
                    rhs,
                );
            }
        }

        pub fn add_linear_ge(&mut self, vars: &[i32], coeffs: &[i64], rhs: i64) {
            assert_eq!(vars.len(), coeffs.len());
            unsafe {
                cpmodel_add_linear_ge(
                    self.ptr.as_ptr(),
                    vars.as_ptr(),
                    coeffs.as_ptr(),
                    vars.len(),
                    rhs,
                );
            }
        }

        pub fn add_linear_eq(&mut self, vars: &[i32], coeffs: &[i64], rhs: i64) {
            assert_eq!(vars.len(), coeffs.len());
            unsafe {
                cpmodel_add_linear_eq(
                    self.ptr.as_ptr(),
                    vars.as_ptr(),
                    coeffs.as_ptr(),
                    vars.len(),
                    rhs,
                );
            }
        }

        pub fn add_all_different(&mut self, vars: &[i32]) {
            unsafe {
                cpmodel_add_all_different(self.ptr.as_ptr(), vars.as_ptr(), vars.len());
            }
        }

        pub fn minimize(&mut self, vars: &[i32], coeffs: &[i64]) {
            assert_eq!(vars.len(), coeffs.len());
            unsafe {
                cpmodel_minimize(
                    self.ptr.as_ptr(),
                    vars.as_ptr(),
                    coeffs.as_ptr(),
                    vars.len(),
                );
            }
        }

        pub fn maximize(&mut self, vars: &[i32], coeffs: &[i64]) {
            assert_eq!(vars.len(), coeffs.len());
            unsafe {
                cpmodel_maximize(
                    self.ptr.as_ptr(),
                    vars.as_ptr(),
                    coeffs.as_ptr(),
                    vars.len(),
                );
            }
        }

        pub fn solve(&self, time_limit_seconds: f64) -> CpSolution {
            unsafe {
                let response = cpmodel_solve(self.ptr.as_ptr(), time_limit_seconds);
                CpSolution {
                    response: NonNull::new(response).expect("failed to solve"),
                }
            }
        }
    }

    impl Default for CpModel {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Drop for CpModel {
        fn drop(&mut self) {
            unsafe {
                cpmodel_free(self.ptr.as_ptr());
            }
        }
    }

    /// Solution from CP-SAT solver
    pub struct CpSolution {
        response: NonNull<CpSolverResponse>,
    }

    impl CpSolution {
        pub fn status(&self) -> OrtoolsStatus {
            unsafe { cpresponse_status(self.response.as_ptr()) }
        }

        pub fn objective_value(&self) -> i64 {
            unsafe { cpresponse_objective_value(self.response.as_ptr()) }
        }

        pub fn value(&self, var_index: i32) -> i64 {
            unsafe { cpresponse_value(self.response.as_ptr(), var_index) }
        }

        pub fn wall_time(&self) -> f64 {
            unsafe { cpresponse_wall_time(self.response.as_ptr()) }
        }
    }

    impl Drop for CpSolution {
        fn drop(&mut self) {
            unsafe {
                cpresponse_free(self.response.as_ptr());
            }
        }
    }

    /// Safe wrapper for GLOP linear solver
    pub struct LinearSolver {
        ptr: NonNull<MpSolver>,
    }

    impl LinearSolver {
        pub fn new(name: &str, solver_type: LpSolverType) -> Self {
            let c_name = CString::new(name).unwrap();
            unsafe {
                Self {
                    ptr: NonNull::new(mpsolver_new(c_name.as_ptr(), solver_type))
                        .expect("failed to create solver"),
                }
            }
        }

        pub fn new_glop(name: &str) -> Self {
            Self::new(name, LpSolverType::Glop)
        }

        pub fn num_var(&mut self, lb: f64, ub: f64, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe { mpsolver_num_var(self.ptr.as_ptr(), lb, ub, c_name.as_ptr()) }
        }

        pub fn int_var(&mut self, lb: f64, ub: f64, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe { mpsolver_int_var(self.ptr.as_ptr(), lb, ub, c_name.as_ptr()) }
        }

        pub fn bool_var(&mut self, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe { mpsolver_bool_var(self.ptr.as_ptr(), c_name.as_ptr()) }
        }

        pub fn add_constraint(&mut self, lb: f64, ub: f64, name: &str) -> i32 {
            let c_name = CString::new(name).unwrap();
            unsafe { mpsolver_add_constraint(self.ptr.as_ptr(), lb, ub, c_name.as_ptr()) }
        }

        pub fn set_constraint_coeff(&mut self, constraint: i32, var: i32, coeff: f64) {
            unsafe {
                mpsolver_set_constraint_coeff(self.ptr.as_ptr(), constraint, var, coeff);
            }
        }

        pub fn set_objective_coeff(&mut self, var: i32, coeff: f64) {
            unsafe {
                mpsolver_set_objective_coeff(self.ptr.as_ptr(), var, coeff);
            }
        }

        pub fn minimize(&mut self) {
            unsafe {
                mpsolver_minimize(self.ptr.as_ptr());
            }
        }

        pub fn maximize(&mut self) {
            unsafe {
                mpsolver_maximize(self.ptr.as_ptr());
            }
        }

        pub fn solve(&mut self) -> OrtoolsStatus {
            unsafe { mpsolver_solve(self.ptr.as_ptr()) }
        }

        pub fn objective_value(&self) -> f64 {
            unsafe { mpsolver_objective_value(self.ptr.as_ptr()) }
        }

        pub fn var_value(&self, var: i32) -> f64 {
            unsafe { mpsolver_var_value(self.ptr.as_ptr(), var) }
        }
    }

    impl Drop for LinearSolver {
        fn drop(&mut self) {
            unsafe {
                mpsolver_free(self.ptr.as_ptr());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_values() {
        assert_eq!(OrtoolsStatus::Optimal as i32, 1);
        assert!(OrtoolsStatus::Optimal.is_success());
        assert!(!OrtoolsStatus::Infeasible.is_success());
    }

    #[test]
    fn test_solver_type_values() {
        assert_eq!(LpSolverType::Glop as i32, 0);
        assert_eq!(LpSolverType::Cbc as i32, 2);
    }

    #[cfg(feature = "link")]
    mod integration {
        use super::super::safe::*;
        use super::super::*;

        #[test]
        fn test_cpsat_simple() {
            // Simple problem: x + y = 10, minimize x, x,y in [0,10]
            let mut model = CpModel::new();
            let x = model.new_int_var(0, 10, "x");
            let y = model.new_int_var(0, 10, "y");

            // x + y = 10
            model.add_linear_eq(&[x, y], &[1, 1], 10);

            // Minimize x
            model.minimize(&[x], &[1]);

            let solution = model.solve(60.0);
            assert!(solution.status().is_success());
            assert_eq!(solution.value(x), 0);
            assert_eq!(solution.value(y), 10);
        }

        #[test]
        fn test_cpsat_all_different() {
            // 3 variables, all different, domain [1,3]
            let mut model = CpModel::new();
            let a = model.new_int_var(1, 3, "a");
            let b = model.new_int_var(1, 3, "b");
            let c = model.new_int_var(1, 3, "c");

            model.add_all_different(&[a, b, c]);

            let solution = model.solve(60.0);
            assert!(solution.status().is_success());

            let vals = [solution.value(a), solution.value(b), solution.value(c)];
            assert!(vals.contains(&1));
            assert!(vals.contains(&2));
            assert!(vals.contains(&3));
        }

        #[test]
        fn test_glop_simple() {
            // Simple LP: maximize x + y subject to x + y <= 10, x,y >= 0
            let mut solver = LinearSolver::new_glop("test");
            let x = solver.num_var(0.0, f64::INFINITY, "x");
            let y = solver.num_var(0.0, f64::INFINITY, "y");

            // x + y <= 10
            let c = solver.add_constraint(f64::NEG_INFINITY, 10.0, "c1");
            solver.set_constraint_coeff(c, x, 1.0);
            solver.set_constraint_coeff(c, y, 1.0);

            // Maximize x + y
            solver.set_objective_coeff(x, 1.0);
            solver.set_objective_coeff(y, 1.0);
            solver.maximize();

            let status = solver.solve();
            assert!(status.is_success());
            assert!((solver.objective_value() - 10.0).abs() < 1e-6);
        }
    }
}
