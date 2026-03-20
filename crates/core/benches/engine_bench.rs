// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: LicenseRef-Proprietary

//! Engine benchmarks.
//!
//! Run with: `cargo bench`

use criterion::{Criterion, criterion_group, criterion_main};

fn engine_benchmark(_c: &mut Criterion) {
    // Placeholder for engine benchmarks
    // Will be populated in Phase 4 (Performance Infrastructure)
}

criterion_group!(benches, engine_benchmark);
criterion_main!(benches);
