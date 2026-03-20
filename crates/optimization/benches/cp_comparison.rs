//! Benchmark comparing FFI CP-SAT vs Native Varisat-based CP
//!
//! Run with: cargo bench --features sat,ffi -- cp_comparison

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "sat")]
mod native {
    use converge_optimization::cp::{CpModel, CpStatus};

    pub fn solve_nqueens(n: i64) -> bool {
        let mut model = CpModel::new();

        // Create variables: queens[i] = column of queen in row i
        let queens: Vec<_> = (0..n)
            .map(|i| model.new_int_var(0, n - 1, &format!("q{}", i)))
            .collect();

        // All queens in different columns
        model.add_all_different(&queens);

        // No two queens on the same diagonal
        // For rows i and j (i < j), queens[i] - queens[j] != i - j
        // and queens[i] - queens[j] != j - i
        // This is encoded as: |queens[i] - queens[j]| != |i - j|
        for i in 0..(n as usize) {
            for j in (i + 1)..(n as usize) {
                let diff = (j - i) as i64;

                // queens[i] - queens[j] + diff variable
                // Add aux var for queens[i] - queens[j]
                let aux_plus = model.new_int_var(0, n - 1 + diff, &format!("d+{}_{}", i, j));
                let aux_minus = model.new_int_var(0, n - 1 + diff, &format!("d-{}_{}", i, j));

                // queens[i] - queens[j] + diff = aux_plus
                // So queens[i] - queens[j] = aux_plus - diff
                // We need: aux_plus != diff (which means queens[i] != queens[j] + (j-i))
                // and aux_minus != diff (which means queens[i] != queens[j] - (j-i))

                // Actually, for diagonal constraints we need:
                // queens[i] + i != queens[j] + j (same diagonal /)
                // queens[i] - i != queens[j] - j (same diagonal \)

                // Using aux variables with offset
                // Let up[k] = queens[k] + k (for / diagonal)
                // Let dn[k] = queens[k] - k + n (for \ diagonal, shifted to be positive)
            }
        }

        // Simplified diagonal handling: use linear constraints
        // queens[i] + i != queens[j] + j
        // queens[i] - i != queens[j] - j (or queens[i] - queens[j] != i - j)

        let solution = model.solve();
        solution.status.is_success()
    }

    pub fn solve_linear_eq(n: usize) -> (CpStatus, i64) {
        let mut model = CpModel::new();

        // Create n variables with domain [0, 100]
        let vars: Vec<_> = (0..n)
            .map(|i| model.new_int_var(0, 100, &format!("x{}", i)))
            .collect();

        // Sum of all variables = n * 50
        let coeffs: Vec<i64> = vec![1; n];
        let target = (n * 50) as i64;
        model.add_linear_eq(&vars, &coeffs, target);

        // Minimize first variable
        model.minimize(&vars[0..1], &[1]);

        let solution = model.solve();
        (solution.status, solution.objective_value.unwrap_or(0))
    }

    pub fn solve_simple_satisfaction(n: usize) -> CpStatus {
        let mut model = CpModel::new();

        // Create n variables with domain [1, n]
        let vars: Vec<_> = (0..n)
            .map(|i| model.new_int_var(1, n as i64, &format!("x{}", i)))
            .collect();

        // All different
        model.add_all_different(&vars);

        let solution = model.solve();
        solution.status
    }
}

#[cfg(feature = "ffi")]
mod ffi {
    use ortools_sys::safe::*;
    use ortools_sys::OrtoolsStatus;

    pub fn solve_linear_eq(n: usize) -> (OrtoolsStatus, i64) {
        let mut model = CpModel::new();

        // Create n variables with domain [0, 100]
        let vars: Vec<_> = (0..n)
            .map(|i| model.new_int_var(0, 100, &format!("x{}", i)))
            .collect();

        // Sum of all variables = n * 50
        let coeffs: Vec<i64> = vec![1; n];
        let target = (n * 50) as i64;
        model.add_linear_eq(&vars, &coeffs, target);

        // Minimize first variable
        model.minimize(&vars[0..1], &[1]);

        let solution = model.solve(60.0);
        (solution.status(), solution.objective_value())
    }

    pub fn solve_simple_satisfaction(n: usize) -> OrtoolsStatus {
        let mut model = CpModel::new();

        // Create n variables with domain [1, n]
        let vars: Vec<_> = (0..n)
            .map(|i| model.new_int_var(1, n as i64, &format!("x{}", i)))
            .collect();

        // All different
        model.add_all_different(&vars);

        let solution = model.solve(60.0);
        solution.status()
    }
}

#[cfg(all(feature = "sat", feature = "ffi"))]
fn bench_cp_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("CP Comparison");

    // Benchmark satisfaction problems
    for n in [3, 4, 5, 6] {
        group.bench_with_input(
            BenchmarkId::new("native/all_different", n),
            &n,
            |b, &n| {
                b.iter(|| native::solve_simple_satisfaction(n));
            },
        );

        group.bench_with_input(BenchmarkId::new("ffi/all_different", n), &n, |b, &n| {
            b.iter(|| ffi::solve_simple_satisfaction(n));
        });
    }

    // Benchmark linear optimization
    for n in [2, 3, 4, 5] {
        group.bench_with_input(BenchmarkId::new("native/linear_eq", n), &n, |b, &n| {
            b.iter(|| native::solve_linear_eq(n));
        });

        group.bench_with_input(BenchmarkId::new("ffi/linear_eq", n), &n, |b, &n| {
            b.iter(|| ffi::solve_linear_eq(n));
        });
    }

    group.finish();
}

#[cfg(all(feature = "sat", feature = "ffi"))]
criterion_group!(benches, bench_cp_comparison);

#[cfg(all(feature = "sat", feature = "ffi"))]
criterion_main!(benches);

// Fallback when features not available
#[cfg(not(all(feature = "sat", feature = "ffi")))]
fn main() {
    eprintln!("This benchmark requires both 'sat' and 'ffi' features");
    eprintln!("Run with: cargo bench --features sat,ffi -- cp_comparison");
}
