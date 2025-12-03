/*
Benchmark para el lexer del compilador Vela

Este benchmark mide el rendimiento del análisis léxico
y tokenización del código fuente.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_lexer_tokenization(c: &mut Criterion) {
    let source = "fn add(a: Number, b: Number) -> Number { return a + b }";

    c.bench_function("lexer_basic_tokenization", |b| {
        b.iter(|| {
            // Benchmark básico de tokenización - será extendido con lexer real
            black_box(source.len())
        })
    });
}

criterion_group!(benches, benchmark_lexer_tokenization);
criterion_main!(benches);