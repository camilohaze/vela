//! Tests de stress para async iterators
//!
//! Tests que validan el comportamiento del sistema bajo alta carga,
//! memory pressure, long-running operations y escenarios extremos.

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};

/// Tests de alta carga
#[cfg(test)]
mod high_load_tests {
    use super::*;

    #[tokio::test]
    async fn test_large_stream_processing() {
        let start = Instant::now();
        let large_data: Vec<i32> = (0..100_000).collect();
        let stream = StreamBuilder::from_iter(large_data.into_iter());

        let counter = Arc::new(AtomicUsize::new(0));
        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        // Wait for processing to complete
        tokio::time::sleep(Duration::from_secs(5)).await;

        let duration = start.elapsed();
        let processed_count = counter.load(Ordering::SeqCst);

        println!("Large stream processing took: {:?}, processed: {}", duration, processed_count);

        // Should have processed all elements
        assert_eq!(processed_count, 100_000);

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_high_frequency_stream() {
        let start = Instant::now();
        let stream = StreamBuilder::interval(Duration::from_micros(100))
            .take(10_000);

        let counter = Arc::new(AtomicUsize::new(0));
        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        // Wait for all values to be processed
        tokio::time::sleep(Duration::from_secs(2)).await;

        let duration = start.elapsed();
        let processed_count = counter.load(Ordering::SeqCst);

        println!("High frequency stream processed {} values in {:?}", processed_count, duration);

        // Should have processed most values within reasonable time
        assert!(processed_count >= 9000); // Allow some tolerance for timing

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_concurrent_subscriptions_high_load() {
        let data: Vec<i32> = (0..50_000).collect();
        let stream = StreamBuilder::from_iter(data.into_iter());

        let counters: Vec<Arc<AtomicUsize>> = (0..10)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        let mut subscriptions = Vec::new();

        // Create 10 concurrent subscriptions
        for i in 0..10 {
            let counter_clone = Arc::clone(&counters[i]);
            let subscription = stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            );
            subscriptions.push(subscription);
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_secs(3)).await;

        // All subscriptions should have received all values
        for (i, counter) in counters.iter().enumerate() {
            let count = counter.load(Ordering::SeqCst);
            println!("Subscription {} processed {} values", i, count);
            assert_eq!(count, 50_000);
        }

        // Clean up
        for subscription in subscriptions {
            subscription.unsubscribe();
        }
    }

    #[tokio::test]
    async fn test_memory_pressure_with_large_data() {
        let large_data: Vec<Vec<i32>> = (0..10_000)
            .map(|x| vec![x; 1000]) // Create large vectors
            .collect();
        let stream = StreamBuilder::from_iter(large_data.into_iter());

        let counter = Arc::new(AtomicUsize::new(0));
        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        // Process with timeout to avoid hanging
        let result = tokio::time::timeout(Duration::from_secs(10), async {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }).await;

        let processed_count = counter.load(Ordering::SeqCst);
        println!("Memory pressure test processed {} values", processed_count);

        // Should process all values
        assert_eq!(processed_count, 10_000);

        subscription.unsubscribe();
    }
}

/// Tests de long-running operations
#[cfg(test)]
mod long_running_tests {
    use super::*;

    #[tokio::test]
    async fn test_long_running_subscription() {
        let stream = StreamBuilder::interval(Duration::from_millis(100))
            .take(1000); // 100 seconds worth of values

        let counter = Arc::new(AtomicUsize::new(0));
        let start_time = Instant::now();

        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        // Run for 10 seconds
        tokio::time::sleep(Duration::from_secs(10)).await;

        let elapsed = start_time.elapsed();
        let processed_count = counter.load(Ordering::SeqCst);

        println!("Long running test processed {} values in {:?}", processed_count, elapsed);

        // Should have processed approximately 100 values (10 seconds / 100ms)
        assert!(processed_count >= 90 && processed_count <= 110);

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_subscription_lifecycle_stress() {
        let mut subscriptions = Vec::new();
        let stream = StreamBuilder::interval(Duration::from_millis(10));

        // Create and destroy subscriptions repeatedly
        for i in 0..50 {
            let counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = Arc::clone(&counter);

            let subscription = stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            );

            subscriptions.push((subscription, counter));

            // Let it run briefly
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Unsubscribe
            if let Some((subscription, counter)) = subscriptions.pop() {
                let count = counter.load(Ordering::SeqCst);
                println!("Subscription {} processed {} values before unsubscribe", i, count);
                subscription.unsubscribe();
            }
        }

        // Clean up any remaining subscriptions
        for (subscription, _) in subscriptions {
            subscription.unsubscribe();
        }
    }

    #[tokio::test]
    async fn test_sustained_interval_stream() {
        let stream = StreamBuilder::interval(Duration::from_millis(1));

        let counter = Arc::new(AtomicUsize::new(0));
        let start_time = Instant::now();

        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        // Run for 5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;

        let elapsed = start_time.elapsed();
        let processed_count = counter.load(Ordering::SeqCst);

        println!("Sustained stream processed {} values in {:?}", processed_count, elapsed);

        // Should have processed approximately 5000 values (5 seconds / 1ms)
        assert!(processed_count >= 4500 && processed_count <= 5500);

        subscription.unsubscribe();
    }
}

/// Tests de error recovery básico
#[cfg(test)]
mod error_recovery_tests {
    use super::*;

    #[tokio::test]
    async fn test_error_callback_invocation() {
        // Since we don't have error-producing streams yet,
        // test that error callbacks can be set up and don't interfere
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter());

        let counter = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        let subscription = {
            let counter_clone = Arc::clone(&counter);
            let error_clone = Arc::clone(&error_count);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                move |_| { error_clone.fetch_add(1, Ordering::SeqCst); },
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(50)).await;

        let processed_count = counter.load(Ordering::SeqCst);
        let errors = error_count.load(Ordering::SeqCst);

        println!("Error callback test: {} processed, {} errors", processed_count, errors);

        // Should have processed all values without errors
        assert_eq!(processed_count, 5);
        assert_eq!(errors, 0);

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_completion_callback_stress() {
        let completed_count = Arc::new(AtomicUsize::new(0));

        // Test multiple streams completing
        for i in 0..100 {
            let completed_clone = Arc::clone(&completed_count);
            let stream = StreamBuilder::from_iter(vec![i].into_iter());

            let subscription = stream.subscribe(
                |_| {},
                |_| {},
                move || { completed_clone.fetch_add(1, Ordering::SeqCst); },
            );

            tokio::time::sleep(Duration::from_micros(100)).await;
            subscription.unsubscribe();
        }

        let total_completed = completed_count.load(Ordering::SeqCst);
        println!("Completion callback stress test: {} completions", total_completed);
        assert_eq!(total_completed, 100);
    }
}

/// Tests de resource cleanup
#[cfg(test)]
mod resource_cleanup_tests {
    use super::*;

    #[tokio::test]
    async fn test_subscription_cleanup_on_drop() {
        let stream = StreamBuilder::interval(Duration::from_millis(10));

        let counter = Arc::new(AtomicUsize::new(0));

        // Create subscription in a block to test cleanup on drop
        {
            let counter_clone = Arc::clone(&counter);
            let _subscription = stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            );

            // Let it run briefly
            tokio::time::sleep(Duration::from_millis(50)).await;
        } // Subscription should be dropped here

        let initial_count = counter.load(Ordering::SeqCst);

        // Wait a bit more - should not receive more values after drop
        tokio::time::sleep(Duration::from_millis(50)).await;

        let final_count = counter.load(Ordering::SeqCst);

        println!("Cleanup test: initial {}, final {}", initial_count, final_count);

        // Count should not increase after subscription is dropped
        assert_eq!(initial_count, final_count);
    }

    #[tokio::test]
    async fn test_multiple_cleanup_cycles() {
        let stream = StreamBuilder::interval(Duration::from_millis(5));

        // Run multiple create/destroy cycles
        for cycle in 0..10 {
            let counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = Arc::clone(&counter);

            {
                let _subscription = stream.subscribe(
                    move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                    |_| {},
                    || {},
                );

                tokio::time::sleep(Duration::from_millis(20)).await;
            } // Subscription dropped

            let count = counter.load(Ordering::SeqCst);
            println!("Cycle {} processed {} values", cycle, count);
            assert!(count > 0); // Should have processed some values
        }
    }

    #[tokio::test]
    async fn test_subscription_memory_overhead() {
        // Test that creating many subscriptions doesn't cause issues
        let stream = StreamBuilder::just(42);

        let mut subscriptions = Vec::new();

        // Create many subscriptions
        for i in 0..1000 {
            let subscription = stream.subscribe(
                |_| {},
                |_| {},
                || {},
            );
            subscriptions.push(subscription);

            if i % 100 == 0 {
                println!("Created {} subscriptions", i);
            }
        }

        // Let them process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Clean up subscriptions
        for subscription in subscriptions {
            subscription.unsubscribe();
        }

        println!("Successfully managed 1000 subscriptions");
    }
}

/// Tests de boundary conditions
#[cfg(test)]
mod boundary_condition_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_stream_stress() {
        // Test many empty streams
        let completed_count = Arc::new(AtomicUsize::new(0));

        for _ in 0..1000 {
            let completed_clone = Arc::clone(&completed_count);
            let stream = StreamBuilder::empty::<i32>();

            let subscription = stream.subscribe(
                |_| {},
                |_| {},
                move || { completed_clone.fetch_add(1, Ordering::SeqCst); },
            );

            tokio::time::sleep(Duration::from_micros(10)).await;
            subscription.unsubscribe();
        }

        let total_completed = completed_count.load(Ordering::SeqCst);
        println!("Empty stream stress test: {} completions", total_completed);
        assert_eq!(total_completed, 1000);
    }

    #[tokio::test]
    async fn test_single_value_high_concurrency() {
        let stream = StreamBuilder::just(42);

        let counters: Vec<Arc<AtomicUsize>> = (0..1000)
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect();

        let mut subscriptions = Vec::new();

        // Create 1000 concurrent subscriptions to a single value
        for i in 0..1000 {
            let counter_clone = Arc::clone(&counters[i]);
            let subscription = stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            );
            subscriptions.push(subscription);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        // All subscriptions should have received the single value
        for (i, counter) in counters.iter().enumerate() {
            let count = counter.load(Ordering::SeqCst);
            assert_eq!(count, 1, "Subscription {} should have received 1 value", i);
        }

        // Clean up
        for subscription in subscriptions {
            subscription.unsubscribe();
        }
    }

    #[tokio::test]
    async fn test_rapid_subscription_creation() {
        let stream = StreamBuilder::interval(Duration::from_millis(1));

        let mut subscriptions = Vec::new();
        let total_created = Arc::new(AtomicUsize::new(0));

        // Rapidly create subscriptions
        for i in 0..100 {
            let total_clone = Arc::clone(&total_created);
            let subscription = stream.subscribe(
                move |_| { total_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            );
            subscriptions.push(subscription);

            // Don't wait between creations
        }

        tokio::time::sleep(Duration::from_millis(50)).await;

        let total_processed = total_created.load(Ordering::SeqCst);
        println!("Rapid creation test processed {} total values", total_processed);

        // Should have processed some values
        assert!(total_processed > 0);

        // Clean up
        for subscription in subscriptions {
            subscription.unsubscribe();
        }
    }

    #[tokio::test]
    async fn test_extreme_interval_timing() {
        // Test with very small intervals
        let stream = StreamBuilder::interval(Duration::from_nanos(100))
            .take(1000);

        let counter = Arc::new(AtomicUsize::new(0));
        let subscription = {
            let counter_clone = Arc::clone(&counter);
            stream.subscribe(
                move |_| { counter_clone.fetch_add(1, Ordering::SeqCst); },
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_micros(2000)).await;

        let count = counter.load(Ordering::SeqCst);
        println!("Extreme timing test processed {} values", count);

        // Should have processed many values quickly
        assert!(count >= 500);

        subscription.unsubscribe();
    }
}</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\tests\unit\runtime\async_iterators\stress_tests.rs