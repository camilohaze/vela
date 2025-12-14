//! Tests de performance para async iterators
//!
//! Benchmarks que miden throughput, latency y memory usage
//! de la Stream API básica bajo diferentes cargas.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};

/// Benchmarks de creación y suscripción básica
pub fn basic_stream_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("stream_just_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut count = 0;
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::just(42);
                    let subscription = stream.subscribe(
                        |_| count += 1,
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
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

                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(10)).await;
                subscription.unsubscribe();
                count
            })
        })
    });

    c.bench_function("stream_empty_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut completed_count = 0;
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::empty::<i32>();
                    let subscription = stream.subscribe(
                        |_| {},
                        |_| {},
                        || { completed_count += 1; },
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
                }
                completed_count
            })
        })
    });

    c.bench_function("stream_interval_throughput", |b| {
        b.iter(|| {
            rt.block_on(async {
                let stream = StreamBuilder::interval(Duration::from_micros(10))
                    .take(black_box(100));

                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(20)).await;
                subscription.unsubscribe();
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
                let stream = StreamBuilder::from_iter(black_box(0..100).collect::<Vec<_>>().into_iter());

                let results1 = Arc::new(Mutex::new(0));
                let results2 = Arc::new(Mutex::new(0));
                let results3 = Arc::new(Mutex::new(0));

                let subscription1 = {
                    let results1_clone = Arc::clone(&results1);
                    stream.subscribe(
                        move |_| { *results1_clone.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    )
                };

                let subscription2 = {
                    let results2_clone = Arc::clone(&results2);
                    stream.subscribe(
                        move |_| { *results2_clone.lock().unwrap() += 1; },
                        |_| {},
                        || {},
                    )
                };

                let subscription3 = {
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
                let stream = StreamBuilder::just(black_box(42));

                let mut subscriptions = Vec::new();
                for _ in 0..black_box(100) {
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

                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(50)).await;
                subscription.unsubscribe();
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

                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(20)).await;
                subscription.unsubscribe();
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
                let mut operations = 0;
                for _ in 0..black_box(100) {
                    let stream = StreamBuilder::just(1);
                    let subscription = stream.subscribe(
                        |_| operations += 1,
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(10)).await;
                    subscription.unsubscribe();
                }
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
                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(5)).await;
                subscription.unsubscribe();
                count
            })
        })
    });

    c.bench_function("high_frequency_interval_stream", |b| {
        b.iter(|| {
            rt.block_on(async {
                let stream = StreamBuilder::interval(Duration::from_micros(1))
                    .take(black_box(100));

                let mut count = 0;
                let subscription = stream.subscribe(
                    |_| count += 1,
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(5)).await;
                subscription.unsubscribe();
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
                let stream = StreamBuilder::just(black_box(42));

                let mut setup_count = 0;
                for _ in 0..black_box(100) {
                    let subscription = stream.subscribe(
                        |_| {},
                        |_| {},
                        || {},
                    );
                    setup_count += 1;
                    subscription.unsubscribe();
                }

                setup_count
            })
        })
    });

    c.bench_function("callback_invocation_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut invocation_count = 0;
                for _ in 0..black_box(1000) {
                    let stream = StreamBuilder::just(1);
                    let subscription = stream.subscribe(
                        |_| invocation_count += 1,
                        |_| {},
                        || {},
                    );
                    tokio::time::sleep(Duration::from_micros(1)).await;
                    subscription.unsubscribe();
                }
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
criterion_main!(benches);</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\unit\runtime\async_iterators\performance_tests.rs