//! SAT-based CP solver implementation
//!
//! Uses order encoding for integer variables and pseudo-boolean techniques
//! for linear constraints.

use std::collections::HashMap;
use varisat::{CnfFormula, ExtendFormula, Lit, Solver, Var};

/// Status of a CP solve
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CpStatus {
    /// Optimal solution found
    Optimal,
    /// Feasible solution found (may not be optimal)
    Feasible,
    /// Problem is infeasible
    Infeasible,
    /// Model is invalid
    Invalid,
    /// Unknown status
    Unknown,
}

impl CpStatus {
    /// Returns true if the solve found a valid solution
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Optimal | Self::Feasible)
    }
}

/// Handle to an integer variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntVarId(usize);

/// Internal representation of an integer variable using order encoding
#[derive(Debug)]
struct IntVar {
    #[allow(dead_code)] // Used by Debug impl
    name: String,
    lb: i64,
    ub: i64,
    /// Order variables: order_vars[i] represents "x <= lb + i"
    /// We have (ub - lb) such variables
    order_vars: Vec<Var>,
}

/// CP Model builder
pub struct CpModel {
    vars: Vec<IntVar>,
    constraints: Vec<Constraint>,
    objective: Option<Objective>,
    next_var: Var,
}

#[derive(Debug)]
enum Constraint {
    /// sum(coeffs[i] * vars[i]) <= rhs
    LinearLe {
        vars: Vec<IntVarId>,
        coeffs: Vec<i64>,
        rhs: i64,
    },
    /// sum(coeffs[i] * vars[i]) >= rhs
    LinearGe {
        vars: Vec<IntVarId>,
        coeffs: Vec<i64>,
        rhs: i64,
    },
    /// sum(coeffs[i] * vars[i]) == rhs
    LinearEq {
        vars: Vec<IntVarId>,
        coeffs: Vec<i64>,
        rhs: i64,
    },
    /// All variables must have different values
    AllDifferent { vars: Vec<IntVarId> },
}

#[derive(Debug)]
struct Objective {
    vars: Vec<IntVarId>,
    coeffs: Vec<i64>,
    minimize: bool,
}

/// Solution from CP solver
pub struct CpSolution {
    /// Solve status
    pub status: CpStatus,
    /// Objective value (if optimization problem)
    pub objective_value: Option<i64>,
    /// Variable values
    values: HashMap<IntVarId, i64>,
    /// Solve time in seconds
    pub wall_time: f64,
}

impl CpSolution {
    /// Get the value of a variable
    pub fn value(&self, var: IntVarId) -> i64 {
        self.values.get(&var).copied().unwrap_or(0)
    }
}

impl Default for CpModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CpModel {
    /// Create a new CP model
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            constraints: Vec::new(),
            objective: None,
            next_var: Var::from_index(0),
        }
    }

    fn alloc_var(&mut self) -> Var {
        let v = self.next_var;
        self.next_var = Var::from_index(v.index() + 1);
        v
    }

    /// Add a new integer variable with domain [lb, ub]
    pub fn new_int_var(&mut self, lb: i64, ub: i64, name: &str) -> IntVarId {
        assert!(lb <= ub, "Invalid domain: lb > ub");

        // Allocate order encoding variables
        // For domain [lb, ub], we need (ub - lb) boolean vars
        // order_vars[i] = true means x <= lb + i
        let num_order_vars = (ub - lb) as usize;
        let order_vars: Vec<Var> = (0..num_order_vars).map(|_| self.alloc_var()).collect();

        let id = IntVarId(self.vars.len());
        self.vars.push(IntVar {
            name: name.to_string(),
            lb,
            ub,
            order_vars,
        });
        id
    }

    /// Add a new boolean variable (integer with domain [0, 1])
    pub fn new_bool_var(&mut self, name: &str) -> IntVarId {
        self.new_int_var(0, 1, name)
    }

    /// Add constraint: sum(coeffs[i] * vars[i]) <= rhs
    pub fn add_linear_le(&mut self, vars: &[IntVarId], coeffs: &[i64], rhs: i64) {
        assert_eq!(vars.len(), coeffs.len());
        self.constraints.push(Constraint::LinearLe {
            vars: vars.to_vec(),
            coeffs: coeffs.to_vec(),
            rhs,
        });
    }

    /// Add constraint: sum(coeffs[i] * vars[i]) >= rhs
    pub fn add_linear_ge(&mut self, vars: &[IntVarId], coeffs: &[i64], rhs: i64) {
        assert_eq!(vars.len(), coeffs.len());
        self.constraints.push(Constraint::LinearGe {
            vars: vars.to_vec(),
            coeffs: coeffs.to_vec(),
            rhs,
        });
    }

    /// Add constraint: sum(coeffs[i] * vars[i]) == rhs
    pub fn add_linear_eq(&mut self, vars: &[IntVarId], coeffs: &[i64], rhs: i64) {
        assert_eq!(vars.len(), coeffs.len());
        self.constraints.push(Constraint::LinearEq {
            vars: vars.to_vec(),
            coeffs: coeffs.to_vec(),
            rhs,
        });
    }

    /// Add all-different constraint
    pub fn add_all_different(&mut self, vars: &[IntVarId]) {
        self.constraints.push(Constraint::AllDifferent {
            vars: vars.to_vec(),
        });
    }

    /// Set objective: minimize sum(coeffs[i] * vars[i])
    pub fn minimize(&mut self, vars: &[IntVarId], coeffs: &[i64]) {
        assert_eq!(vars.len(), coeffs.len());
        self.objective = Some(Objective {
            vars: vars.to_vec(),
            coeffs: coeffs.to_vec(),
            minimize: true,
        });
    }

    /// Set objective: maximize sum(coeffs[i] * vars[i])
    pub fn maximize(&mut self, vars: &[IntVarId], coeffs: &[i64]) {
        assert_eq!(vars.len(), coeffs.len());
        self.objective = Some(Objective {
            vars: vars.to_vec(),
            coeffs: coeffs.to_vec(),
            minimize: false,
        });
    }

    /// Build the base CNF formula (without objective bound)
    fn build_base_formula(&self) -> CnfFormula {
        let mut formula = CnfFormula::new();

        // Add order encoding constraints for each variable
        // If x <= k then x <= k+1 (monotonicity)
        for var in &self.vars {
            for i in 0..var.order_vars.len().saturating_sub(1) {
                // order_vars[i] => order_vars[i+1]
                // equivalent to: NOT order_vars[i] OR order_vars[i+1]
                formula.add_clause(&[
                    Lit::from_var(var.order_vars[i], false),
                    Lit::from_var(var.order_vars[i + 1], true),
                ]);
            }
        }

        // Encode constraints
        for constraint in &self.constraints {
            self.encode_constraint(&mut formula, constraint);
        }

        formula
    }

    /// Solve the model
    pub fn solve(&self) -> CpSolution {
        let start = std::time::Instant::now();
        let formula = self.build_base_formula();

        // For optimization, we'll use binary search
        if let Some(ref obj) = self.objective {
            self.solve_optimization(obj, start)
        } else {
            self.solve_satisfaction(&formula, start)
        }
    }

    fn solve_satisfaction(&self, formula: &CnfFormula, start: std::time::Instant) -> CpSolution {
        let mut solver = Solver::new();
        solver.add_formula(formula);

        match solver.solve() {
            Ok(true) => {
                let values = self.extract_values(&solver.model().unwrap());
                CpSolution {
                    status: CpStatus::Optimal,
                    objective_value: None,
                    values,
                    wall_time: start.elapsed().as_secs_f64(),
                }
            }
            Ok(false) => CpSolution {
                status: CpStatus::Infeasible,
                objective_value: None,
                values: HashMap::new(),
                wall_time: start.elapsed().as_secs_f64(),
            },
            Err(_) => CpSolution {
                status: CpStatus::Unknown,
                objective_value: None,
                values: HashMap::new(),
                wall_time: start.elapsed().as_secs_f64(),
            },
        }
    }

    fn solve_optimization(&self, obj: &Objective, start: std::time::Instant) -> CpSolution {
        // Compute bounds on objective
        let (obj_lb, obj_ub) = self.compute_objective_bounds(obj);

        // Binary search for optimal
        let mut lo = obj_lb;
        let mut hi = obj_ub;
        let mut best_values: Option<HashMap<IntVarId, i64>> = None;
        let mut best_obj: Option<i64> = None;

        while lo <= hi {
            let mid = if obj.minimize {
                lo + (hi - lo) / 2
            } else {
                hi - (hi - lo) / 2
            };

            // Build fresh formula for this iteration
            let mut formula = self.build_base_formula();
            self.add_objective_bound(&mut formula, obj, mid);

            let mut solver = Solver::new();
            solver.add_formula(&formula);

            match solver.solve() {
                Ok(true) => {
                    let values = self.extract_values(&solver.model().unwrap());
                    let actual_obj = self.compute_objective_value(&values, obj);
                    best_values = Some(values);
                    best_obj = Some(actual_obj);

                    if obj.minimize {
                        hi = actual_obj - 1;
                    } else {
                        lo = actual_obj + 1;
                    }
                }
                Ok(false) => {
                    if obj.minimize {
                        lo = mid + 1;
                    } else {
                        hi = mid - 1;
                    }
                }
                Err(_) => break,
            }
        }

        if let Some(values) = best_values {
            CpSolution {
                status: CpStatus::Optimal,
                objective_value: best_obj,
                values,
                wall_time: start.elapsed().as_secs_f64(),
            }
        } else {
            CpSolution {
                status: CpStatus::Infeasible,
                objective_value: None,
                values: HashMap::new(),
                wall_time: start.elapsed().as_secs_f64(),
            }
        }
    }

    fn compute_objective_bounds(&self, obj: &Objective) -> (i64, i64) {
        let mut lb = 0i64;
        let mut ub = 0i64;

        for (var_id, coeff) in obj.vars.iter().zip(&obj.coeffs) {
            let var = &self.vars[var_id.0];
            if *coeff >= 0 {
                lb += coeff * var.lb;
                ub += coeff * var.ub;
            } else {
                lb += coeff * var.ub;
                ub += coeff * var.lb;
            }
        }

        (lb, ub)
    }

    fn compute_objective_value(&self, values: &HashMap<IntVarId, i64>, obj: &Objective) -> i64 {
        obj.vars
            .iter()
            .zip(&obj.coeffs)
            .map(|(var_id, coeff)| coeff * values.get(var_id).copied().unwrap_or(0))
            .sum()
    }

    fn add_objective_bound(&self, formula: &mut CnfFormula, obj: &Objective, bound: i64) {
        // Add constraint: objective <= bound (if minimizing) or >= bound (if maximizing)
        if obj.minimize {
            self.encode_constraint(
                formula,
                &Constraint::LinearLe {
                    vars: obj.vars.clone(),
                    coeffs: obj.coeffs.clone(),
                    rhs: bound,
                },
            );
        } else {
            self.encode_constraint(
                formula,
                &Constraint::LinearGe {
                    vars: obj.vars.clone(),
                    coeffs: obj.coeffs.clone(),
                    rhs: bound,
                },
            );
        }
    }

    fn encode_constraint(&self, formula: &mut CnfFormula, constraint: &Constraint) {
        match constraint {
            Constraint::LinearLe { vars, coeffs, rhs } => {
                self.encode_linear_le(formula, vars, coeffs, *rhs);
            }
            Constraint::LinearGe { vars, coeffs, rhs } => {
                // x >= rhs is equivalent to -x <= -rhs
                let neg_coeffs: Vec<i64> = coeffs.iter().map(|c| -c).collect();
                self.encode_linear_le(formula, vars, &neg_coeffs, -rhs);
            }
            Constraint::LinearEq { vars, coeffs, rhs } => {
                // x == rhs is x <= rhs AND x >= rhs
                self.encode_linear_le(formula, vars, coeffs, *rhs);
                let neg_coeffs: Vec<i64> = coeffs.iter().map(|c| -c).collect();
                self.encode_linear_le(formula, vars, &neg_coeffs, -rhs);
            }
            Constraint::AllDifferent { vars } => {
                self.encode_all_different(formula, vars);
            }
        }
    }

    /// Encode linear <= constraint using direct encoding for binary variables
    /// and enumeration for small domains
    fn encode_linear_le(
        &self,
        formula: &mut CnfFormula,
        vars: &[IntVarId],
        coeffs: &[i64],
        rhs: i64,
    ) {
        // For small domain sizes, use enumeration-based encoding
        // Check total domain size
        let total_combinations: u64 = vars
            .iter()
            .map(|v| (self.vars[v.0].ub - self.vars[v.0].lb + 1) as u64)
            .product();

        if total_combinations <= 10000 {
            // Enumerate all combinations and forbid those that violate the constraint
            self.encode_linear_le_enumerate(formula, vars, coeffs, rhs);
        } else {
            // Use bound propagation for larger domains (weaker but tractable)
            self.encode_linear_le_bounds(formula, vars, coeffs, rhs);
        }
    }

    fn encode_linear_le_enumerate(
        &self,
        formula: &mut CnfFormula,
        vars: &[IntVarId],
        coeffs: &[i64],
        rhs: i64,
    ) {
        // Enumerate all combinations and block those that violate constraint
        let var_data: Vec<_> = vars.iter().map(|v| &self.vars[v.0]).collect();

        // Generate all combinations
        let mut indices: Vec<i64> = var_data.iter().map(|v| v.lb).collect();

        loop {
            // Check if this combination violates the constraint
            let sum: i64 = indices
                .iter()
                .zip(coeffs)
                .map(|(val, coeff)| val * coeff)
                .sum();

            if sum > rhs {
                // Block this combination
                // For each variable with value v, either x <= v-1 or x > v
                let mut clause = Vec::new();
                for (i, val) in indices.iter().enumerate() {
                    let var = var_data[i];
                    // x != val: either x < val or x > val
                    // x < val means order_vars[val - lb - 1] is true (if exists)
                    // x > val means order_vars[val - lb] is false (if exists)

                    if *val > var.lb {
                        let idx = (*val - var.lb - 1) as usize;
                        if idx < var.order_vars.len() {
                            clause.push(Lit::from_var(var.order_vars[idx], true));
                        }
                    }
                    if *val < var.ub {
                        let idx = (*val - var.lb) as usize;
                        if idx < var.order_vars.len() {
                            clause.push(Lit::from_var(var.order_vars[idx], false));
                        }
                    }
                }
                if !clause.is_empty() {
                    formula.add_clause(&clause);
                } else {
                    // No clause possible but combination violates - this would be infeasible
                    // if all vars are fixed to violating values
                }
            }

            // Move to next combination
            let mut i = indices.len();
            loop {
                if i == 0 {
                    return; // Done with all combinations
                }
                i -= 1;
                indices[i] += 1;
                if indices[i] <= var_data[i].ub {
                    break;
                }
                indices[i] = var_data[i].lb;
            }
        }
    }

    fn encode_linear_le_bounds(
        &self,
        formula: &mut CnfFormula,
        vars: &[IntVarId],
        coeffs: &[i64],
        rhs: i64,
    ) {
        // Bound propagation: for each variable, derive upper bound from constraint
        for (i, (var_id, coeff)) in vars.iter().zip(coeffs).enumerate() {
            if *coeff == 0 {
                continue;
            }

            let var = &self.vars[var_id.0];

            // Compute sum of minimums of other variables
            let mut other_min = 0i64;
            for (j, (other_id, other_coeff)) in vars.iter().zip(coeffs).enumerate() {
                if i != j {
                    let other_var = &self.vars[other_id.0];
                    if *other_coeff >= 0 {
                        other_min += other_coeff * other_var.lb;
                    } else {
                        other_min += other_coeff * other_var.ub;
                    }
                }
            }

            let bound = rhs - other_min;

            if *coeff > 0 {
                let max_x = bound / coeff;
                if max_x < var.ub && max_x >= var.lb {
                    let idx = (max_x - var.lb) as usize;
                    if idx < var.order_vars.len() {
                        formula.add_clause(&[Lit::from_var(var.order_vars[idx], true)]);
                    }
                } else if max_x < var.lb {
                    formula.add_clause(&[]);
                }
            } else {
                let min_x = (bound + (-coeff) - 1) / (-coeff);
                if min_x > var.lb && min_x <= var.ub {
                    let idx = (min_x - var.lb - 1) as usize;
                    if idx < var.order_vars.len() {
                        formula.add_clause(&[Lit::from_var(var.order_vars[idx], false)]);
                    }
                } else if min_x > var.ub {
                    formula.add_clause(&[]);
                }
            }
        }
    }

    /// Encode all-different constraint using direct encoding
    fn encode_all_different(&self, formula: &mut CnfFormula, vars: &[IntVarId]) {
        // Find the union of all domains
        let mut all_values: Vec<i64> = Vec::new();
        for var_id in vars {
            let var = &self.vars[var_id.0];
            for v in var.lb..=var.ub {
                if !all_values.contains(&v) {
                    all_values.push(v);
                }
            }
        }

        // For each value, at most one variable can take that value
        for value in &all_values {
            let mut vars_with_value: Vec<(IntVarId, usize)> = Vec::new();
            for var_id in vars {
                let var = &self.vars[var_id.0];
                if *value >= var.lb && *value <= var.ub {
                    let idx = (*value - var.lb) as usize;
                    vars_with_value.push((*var_id, idx));
                }
            }

            // Pairwise encoding: for each pair, they can't both be equal to value
            for i in 0..vars_with_value.len() {
                for j in (i + 1)..vars_with_value.len() {
                    let (var_i, idx_i) = vars_with_value[i];
                    let (var_j, idx_j) = vars_with_value[j];
                    let var_i_data = &self.vars[var_i.0];
                    let var_j_data = &self.vars[var_j.0];

                    // x_i = value AND x_j = value is forbidden
                    // x_i = value means: NOT (x_i <= value - 1) AND (x_i <= value)
                    // Simplified: add clause that blocks both being equal to value

                    // For x_i = value: order_vars[idx_i] = true (x <= value)
                    //                   order_vars[idx_i - 1] = false (NOT x <= value - 1)
                    // We need to forbid both x_i = value AND x_j = value

                    // Using a simpler approach: if both can be value, forbid it
                    // This requires auxiliary variables for "x = value"

                    // Even simpler: just forbid assignments where both equal value
                    // This is done by: (x_i != value) OR (x_j != value)

                    // x_i != value means x_i < value OR x_i > value
                    // In order encoding: (x <= value - 1) OR NOT (x <= value)

                    let mut clause = Vec::new();

                    // x_i != value
                    if idx_i > 0 {
                        clause.push(Lit::from_var(var_i_data.order_vars[idx_i - 1], true));
                    }
                    if idx_i < var_i_data.order_vars.len() {
                        clause.push(Lit::from_var(var_i_data.order_vars[idx_i], false));
                    }

                    // x_j != value
                    if idx_j > 0 {
                        clause.push(Lit::from_var(var_j_data.order_vars[idx_j - 1], true));
                    }
                    if idx_j < var_j_data.order_vars.len() {
                        clause.push(Lit::from_var(var_j_data.order_vars[idx_j], false));
                    }

                    if !clause.is_empty() {
                        formula.add_clause(&clause);
                    }
                }
            }
        }
    }

    fn extract_values(&self, model: &[Lit]) -> HashMap<IntVarId, i64> {
        let mut values = HashMap::new();
        let model_map: HashMap<Var, bool> = model
            .iter()
            .map(|lit| (lit.var(), lit.is_positive()))
            .collect();

        for (i, var) in self.vars.iter().enumerate() {
            // Find the value: it's lb + k where k is the smallest index
            // such that order_vars[k] is true
            let mut value = var.ub; // default to upper bound
            for (k, order_var) in var.order_vars.iter().enumerate() {
                if model_map.get(order_var).copied().unwrap_or(false) {
                    value = var.lb + k as i64;
                    break;
                }
            }
            values.insert(IntVarId(i), value);
        }

        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_satisfaction() {
        let mut model = CpModel::new();
        let x = model.new_int_var(0, 10, "x");
        let y = model.new_int_var(0, 10, "y");

        // x + y = 10
        model.add_linear_eq(&[x, y], &[1, 1], 10);

        let solution = model.solve();
        assert!(solution.status.is_success());
        assert_eq!(solution.value(x) + solution.value(y), 10);
    }

    #[test]
    fn test_minimize() {
        let mut model = CpModel::new();
        let x = model.new_int_var(0, 10, "x");
        let y = model.new_int_var(0, 10, "y");

        // x + y = 10
        model.add_linear_eq(&[x, y], &[1, 1], 10);

        // Minimize x
        model.minimize(&[x], &[1]);

        let solution = model.solve();
        assert_eq!(solution.status, CpStatus::Optimal);
        assert_eq!(solution.value(x), 0);
        assert_eq!(solution.value(y), 10);
    }

    #[test]
    fn test_all_different() {
        let mut model = CpModel::new();
        let a = model.new_int_var(1, 3, "a");
        let b = model.new_int_var(1, 3, "b");
        let c = model.new_int_var(1, 3, "c");

        model.add_all_different(&[a, b, c]);

        let solution = model.solve();
        assert!(solution.status.is_success());

        let vals = [solution.value(a), solution.value(b), solution.value(c)];
        assert!(vals.contains(&1));
        assert!(vals.contains(&2));
        assert!(vals.contains(&3));
    }

    #[test]
    fn test_infeasible() {
        let mut model = CpModel::new();
        let x = model.new_int_var(0, 5, "x");
        let y = model.new_int_var(0, 5, "y");

        // x + y = 20 (impossible with domains [0,5])
        model.add_linear_eq(&[x, y], &[1, 1], 20);

        let solution = model.solve();
        assert_eq!(solution.status, CpStatus::Infeasible);
    }
}
