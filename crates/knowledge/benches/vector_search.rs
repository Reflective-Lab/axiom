//! Benchmarks for vector search operations.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn embedding_benchmark(c: &mut Criterion) {
    use converge_knowledge::EmbeddingEngine;

    let engine = EmbeddingEngine::new(384);
    let texts = vec![
        "Rust programming language",
        "Vector databases enable semantic search",
        "Graph neural networks learn from graph structures",
        "Machine learning models require training data",
    ];

    let mut group = c.benchmark_group("embedding");

    for text in texts {
        group.bench_with_input(BenchmarkId::new("embed", text.len()), &text, |b, text| {
            b.iter(|| engine.embed(text).unwrap())
        });
    }

    group.finish();
}

fn similarity_benchmark(c: &mut Criterion) {
    use converge_knowledge::EmbeddingEngine;

    let engine = EmbeddingEngine::new(384);
    let emb1 = engine.embed("rust programming").unwrap();
    let emb2 = engine.embed("rust development").unwrap();

    c.bench_function("similarity", |b| b.iter(|| engine.similarity(&emb1, &emb2)));
}

criterion_group!(benches, embedding_benchmark, similarity_benchmark);
criterion_main!(benches);
