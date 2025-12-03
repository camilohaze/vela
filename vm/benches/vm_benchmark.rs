/*
Benchmark básico para VelaVM

Este benchmark proporciona una base para medir el rendimiento
de la máquina virtual y el interprete de bytecode.
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_vm_execution(c: &mut Criterion) {
    c.bench_function("vm_basic_execution", |b| {
        b.iter(|| {
            // Benchmark básico de ejecución VM - será extendido
            black_box(42)
        })
    });
}

criterion_group!(benches, benchmark_vm_execution);
criterion_main!(benches);