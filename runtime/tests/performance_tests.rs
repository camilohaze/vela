//! Tests de performance para async iterators
//!
//! Benchmarks que miden throughput, latency y memory usage
//! de la Stream API básica bajo diferentes cargas.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};
use vela_runtime::streams::{StreamBuilder, BackpressureController, BackpressureStrategy};
use vela_runtime::Stream;

/// Benchmarks de creación y suscripción básica
pub fn basic_stream_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("stream_just_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut count = 0;
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::just(42);
                    let count_clone = Arc::new(Mutex::new(0));
                    let count_clone_inner = Arc::clone(&count_clone);
                    let subscription = stream.subscribe(
                        move |_| { *count_clone_inner.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
                    let final_count = *count_clone.lock().unwrap();
                    count += final_count;
                }
                count
            })
        })
    });

    c.bench_function("stream_from_iter_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let data: Vec<i32> = (0..black_box(1000)).collect();
                let stream = StreamBuilder::from_iter(data.into_iter());

                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(10)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });

    c.bench_function("stream_empty_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let completed_count_clone = Arc::new(Mutex::new(0));
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::empty::<i32>();
                    let completed_count_inner = Arc::clone(&completed_count_clone);
                    let subscription = stream.subscribe(
                        |_| {},
                        |_| {},
                        move || { *completed_count_inner.lock().unwrap() += 1; },
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
                }
                let count = *completed_count_clone.lock().unwrap();
                count
            })
        })
    });

    c.bench_function("stream_interval_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let stream = StreamBuilder::interval(Duration::from_micros(10))
                    .take(black_box(100));

                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(20)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });
}

/// Benchmarks de concurrencia y suscripciones múltiples
pub fn concurrency_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("multiple_subscriptions_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let data: Vec<i32> = black_box(0..100).collect();

                let results1 = Arc::new(Mutex::new(0));
                let results2 = Arc::new(Mutex::new(0));
                let results3 = Arc::new(Mutex::new(0));

                let subscription1 = {
                    let stream = StreamBuilder::from_iter(data.clone().into_iter());
                    let results1_clone = Arc::clone(&results1);
                    stream.subscribe(
                        move |_| { *results1_clone.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    )
                };

                let subscription2 = {
                    let stream = StreamBuilder::from_iter(data.clone().into_iter());
                    let results2_clone = Arc::clone(&results2);
                    stream.subscribe(
                        move |_| { *results2_clone.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    )
                };

                let subscription3 = {
                    let stream = StreamBuilder::from_iter(data.into_iter());
                    let results3_clone = Arc::clone(&results3);
                    stream.subscribe(
                        move |_| { *results3_clone.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    )
                };

                tokio::time::sleep(Duration::from_millis(10)).await;

                let total = *results1.lock().unwrap() +
                           *results2.lock().unwrap() +
                           *results3.lock().unwrap();

                subscription1.unsubscribe();
                subscription2.unsubscribe();
                subscription3.unsubscribe();

                total
            })
        })
    });

    c.bench_function("subscription_creation_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut subscriptions = Vec::new();
                for _ in 0..black_box(100) {
                    let stream = StreamBuilder::just(black_box(42));
                    let subscription = stream.subscribe(
                        |_| {},
                        |_| {},
                        || {},
                    );
                    subscriptions.push(subscription);
                }

                tokio::time::sleep(Duration::from_millis(1)).await;

                for subscription in subscriptions {
                    subscription.unsubscribe();
                }

                100
            })
        })
    });
}

/// Benchmarks de memory usage
pub fn memory_usage_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("large_dataset_memory_usage", |b| {
        b.iter(|| {
            rt.block_on(async {
                let large_data: Vec<i32> = (0..black_box(10000)).collect();
                let stream = StreamBuilder::from_iter(large_data.into_iter());

                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(50)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });

    c.bench_function("string_stream_memory_usage", |b| {
        b.iter(|| {
            rt.block_on(async {
                let string_data: Vec<String> = (0..black_box(1000))
                    .map(|i| format!("item_{}", i))
                    .collect();
                let stream = StreamBuilder::from_iter(string_data.into_iter());

                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(20)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });
}

/// Benchmarks de backpressure controller
pub fn backpressure_controller_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("backpressure_controller_pressure_changes", |b| {
        b.iter(|| {
            let mut controller = BackpressureController::new(BackpressureStrategy::DropOldest, 100);

            for _ in 0..black_box(1000) {
                controller.increase_pressure();
                controller.decrease_pressure();
            }

            controller.should_apply_backpressure()
        })
    });

    c.bench_function("backpressure_controller_flow_signals", |b| {
        b.iter(|| {
            let controller = BackpressureController::new(BackpressureStrategy::DropOldest, 10);

            // Fill buffer
            for _ in 0..black_box(10) {
                controller.increase_pressure();
            }

            controller.get_flow_control_signal()
        })
    });

    c.bench_function("different_strategies_flow_signals", |b| {
        b.iter(|| {
            let strategies = vec![
                BackpressureStrategy::DropOldest,
                BackpressureStrategy::DropNewest,
                BackpressureStrategy::Error,
                BackpressureStrategy::Block,
            ];

            let mut results = Vec::new();
            for strategy in strategies {
                let controller = BackpressureController::new(strategy, 5);
                for _ in 0..5 {
                    controller.increase_pressure();
                }
                results.push(controller.get_flow_control_signal());
            }

            results.len()
        })
    });
}

/// Benchmarks de edge cases
pub fn edge_cases_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("rapid_subscription_unsubscription", |b| {
        b.iter(|| {
            rt.block_on(async {
                let operations_clone = Arc::new(Mutex::new(0));
                for _ in 0..black_box(100) {
                    let stream = StreamBuilder::just(1);
                    let operations_inner = Arc::clone(&operations_clone);
                    let subscription = stream.subscribe(
                        move |_| { *operations_inner.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(10)).await;
                    subscription.unsubscribe();
                }
                let operations = *operations_clone.lock().unwrap();
                operations
            })
        })
    });

    c.bench_function("subscription_after_stream_completion", |b| {
        b.iter(|| {
            rt.block_on(async {
                let stream = StreamBuilder::from_iter(vec![1, 2, 3].into_iter());

                // Let stream complete
                tokio::time::sleep(Duration::from_millis(5)).await;

                // Subscribe after completion
                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(5)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });

    c.bench_function("high_frequency_interval_stream", |b| {
        b.iter(|| {
            rt.block_on(async {
                let stream = StreamBuilder::interval(Duration::from_micros(1))
                    .take(black_box(100));

                let count_clone = Arc::new(Mutex::new(0));
                let count_clone_inner = Arc::clone(&count_clone);
                let subscription = stream.subscribe(
                    move |_| { *count_clone_inner.lock().unwrap() += 1; },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(5)).await;
                subscription.unsubscribe();
                let count = *count_clone.lock().unwrap();
                count
            })
        })
    });
}

/// Benchmarks de latency
pub fn latency_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("stream_creation_latency", |b| {
        b.iter(|| {
            for _ in 0..black_box(1000) {
                let _stream = StreamBuilder::just(42);
            }
            1000
        })
    });

    c.bench_function("subscription_setup_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let setup_count_clone = Arc::new(Mutex::new(0));
                for _ in 0..black_box(100) {
                    let stream = StreamBuilder::just(black_box(42));
                    let setup_count_inner = Arc::clone(&setup_count_clone);
                    let subscription = stream.subscribe(
                        |_| {},
                        |_| {},
                        move || { *setup_count_inner.lock().unwrap() += 1; },
                    );
                    subscription.unsubscribe();
                }

                let setup_count = *setup_count_clone.lock().unwrap();
                setup_count
            })
        })
    });

    c.bench_function("callback_invocation_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let invocation_count_clone = Arc::new(Mutex::new(0));
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::just(1);
                    let invocation_count_inner = Arc::clone(&invocation_count_clone);
                    let subscription = stream.subscribe(
                        move |_| { *invocation_count_inner.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
                }
                let invocation_count = *invocation_count_clone.lock().unwrap();
                invocation_count
            })
        })
    });
}

criterion_group!(
    benches,
    basic_stream_benchmarks,
    concurrency_benchmarks,
    memory_usage_benchmarks,
    backpressure_controller_benchmarks,
    edge_cases_benchmarks,
    latency_benchmarks
);
criterion_main!(benches);