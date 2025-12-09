//! # Reactive System Benchmarks
//!
//! Benchmarks for measuring the performance of the reactive system.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use vela_reactive::{signal, Signal, Computed, Effect};

fn bench_signal_creation(c: &mut Criterion) {
    c.bench_function("signal_creation", |b| {
        b.iter(|| {
            let _signal = signal(black_box(42));
        });
    });
}

fn bench_signal_get(c: &mut Criterion) {
    let signal = signal(42);
    c.bench_function("signal_get", |b| {
        b.iter(|| {
            black_box(signal.get());
        });
    });
}

fn bench_signal_set(c: &mut Criterion) {
    let signal = signal(42);
    c.bench_function("signal_set", |b| {
        b.iter(|| {
            signal.set(black_box(43));
        });
    });
}

fn bench_computed_creation(c: &mut Criterion) {
    c.bench_function("computed_creation", |b| {
        b.iter(|| {
            let _computed = Computed::new(|| black_box(42));
        });
    });
}

fn bench_computed_get(c: &mut Criterion) {
    let computed = Computed::new(|| 42);
    c.bench_function("computed_get", |b| {
        b.iter(|| {
            black_box(computed.get());
        });
    });
}

fn bench_effect_creation(c: &mut Criterion) {
    c.bench_function("effect_creation", |b| {
        b.iter(|| {
            let _effect = Effect::new(|| {});
        });
    });
}

fn bench_signal_with_subscribers(c: &mut Criterion) {
    let signal = Arc::new(Signal::new(0));
    let mut subscribers = Vec::new();

    // Create 100 subscribers
    for _ in 0..100 {
        let signal_clone = Arc::clone(&signal);
        let unsubscribe = signal_clone.subscribe(move |_| {});
        subscribers.push(unsubscribe);
    }

    c.bench_function("signal_set_with_100_subscribers", |b| {
        b.iter(|| {
            signal.set(black_box(1));
        });
    });
}

fn bench_computed_chain(c: &mut Criterion) {
    let signal = Arc::new(Signal::new(1));
    let computed1 = Arc::new(Computed::new({
        let signal = Arc::clone(&signal);
        move || signal.get() * 2
    }));
    let computed2 = Arc::new(Computed::new({
        let computed1 = Arc::clone(&computed1);
        move || computed1.get() + 1
    }));
    let computed3 = Arc::new(Computed::new({
        let computed2 = Arc::clone(&computed2);
        move || computed2.get() * 3
    }));

    c.bench_function("computed_chain_get", |b| {
        b.iter(|| {
            black_box(computed3.get());
        });
    });
}

criterion_group!(
    benches,
    bench_signal_creation,
    bench_signal_get,
    bench_signal_set,
    bench_computed_creation,
    bench_computed_get,
    bench_effect_creation,
    bench_signal_with_subscribers,
    bench_computed_chain
);
criterion_main!(benches);