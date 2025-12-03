/*
Benchmark para el Garbage Collector de VelaVM

Este benchmark mide el rendimiento del sistema de recolección
de basura y gestión de memoria.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_gc_operations(c: &mut Criterion) {
    c.bench_function("gc_basic_allocation", |b| {
        b.iter(|| {
            // Benchmark básico de operaciones GC - será extendido
            black_box(vec![1, 2, 3, 4, 5])
        })
    });
}

criterion_group!(benches, benchmark_gc_operations);
criterion_main!(benches);