/*
Benchmark para el parser del compilador Vela

Este benchmark mide el rendimiento del análisis sintáctico
y construcción del AST.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_parser_ast_construction(c: &mut Criterion) {
    let source = "fn add(a: Number, b: Number) -> Number { return a + b }";

    c.bench_function("parser_basic_construction", |b| {
        b.iter(|| {
            // Benchmark básico de construcción AST - será extendido con parser real
            black_box(source.chars().count())
        })
    });
}

criterion_group!(benches, benchmark_parser_ast_construction);
criterion_main!(benches);