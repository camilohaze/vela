/*
Benchmark básico para vela-stdlib

Este benchmark proporciona una base para medir el rendimiento
de las operaciones de la librería estándar.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_basic_operations(c: &mut Criterion) {
    c.bench_function("basic_operation", |b| {
        b.iter(|| {
            // Benchmark básico - será extendido con operaciones reales
            black_box(42)
        })
    });
}

criterion_group!(benches, benchmark_basic_operations);
criterion_main!(benches);