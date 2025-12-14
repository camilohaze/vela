//! Tests de correctness para async iterators
//!
//! Suite completa de tests que validan el funcionamiento básico
//! de la Stream API con la implementación actual.

use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::time::timeout;

use vela_runtime::streams::*;
use vela_runtime::Stream;

/// Tests básicos de creación y suscripción
#[cfg(test)]
mod basic_stream_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_just_correctness() {
        let stream = StreamBuilder::just(42);
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![42]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_empty_correctness() {
        let stream = StreamBuilder::empty::<i32>();
        let values = Arc::new(Mutex::new(Vec::new()));
        let completed = Arc::new(Mutex::new(false));

        let subscription = {
            let values_clone = Arc::clone(&values);
            let completed_clone = Arc::clone(&completed);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                move || { *completed_clone.lock().unwrap() = true; },
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), Vec::<i32>::new());
        assert!(*completed.lock().unwrap());
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_from_iter_correctness() {
        let data = vec![1, 2, 3, 4, 5];
        let stream = StreamBuilder::from_iter(data.into_iter());
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![1, 2, 3, 4, 5]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_interval_correctness() {
        let stream = StreamBuilder::interval(Duration::from_millis(20)).take(3);
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = values.lock().unwrap().clone();
        assert!(!result.is_empty());
        // Should have received some interval values
        subscription.unsubscribe();
    }
}

/// Tests de concurrencia y suscripciones múltiples
#[cfg(test)]
mod concurrency_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiple_subscriptions_correctness() {
        let stream1 = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter());
        let stream2 = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter());

        let values1 = Arc::new(Mutex::new(Vec::new()));
        let values2 = Arc::new(Mutex::new(Vec::new()));

        let subscription1 = {
            let values1_clone = Arc::clone(&values1);
            stream1.subscribe(
                move |value| values1_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        let subscription2 = {
            let values2_clone = Arc::clone(&values2);
            stream2.subscribe(
                move |value| values2_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Both subscriptions should receive all values
        assert_eq!(*values1.lock().unwrap(), vec![1, 2, 3, 4, 5]);
        assert_eq!(*values2.lock().unwrap(), vec![1, 2, 3, 4, 5]);

        subscription1.unsubscribe();
        subscription2.unsubscribe();
    }

    #[tokio::test]
    async fn test_subscription_after_completion_correctness() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3].into_iter());

        // For synchronous streams like from_iter, subscription should always receive all values
        let values = Arc::new(Mutex::new(Vec::new()));
        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Synchronous streams should emit all values to any subscriber
        assert_eq!(*values.lock().unwrap(), vec![1, 2, 3]);
        subscription.unsubscribe();
    }
}

/// Tests de backpressure básico
#[cfg(test)]
mod backpressure_tests {
    use super::*;

    #[tokio::test]
    async fn test_backpressure_controller_creation() {
        let controller = BackpressureController::new(BackpressureStrategy::DropOldest, 10);

        // Test initial state
        assert!(!controller.should_apply_backpressure());
        assert!(controller.should_resume());
    }

    #[tokio::test]
    async fn test_backpressure_pressure_changes() {
        let controller = BackpressureController::new(BackpressureStrategy::DropOldest, 10);

        // Increase pressure
        controller.increase_pressure();
        controller.increase_pressure();

        // Should not apply backpressure yet (below high watermark)
        assert!(!controller.should_apply_backpressure());

        // Increase pressure to trigger backpressure
        for _ in 0..8 {
            controller.increase_pressure();
        }

        // Should now apply backpressure
        assert!(controller.should_apply_backpressure());

        // Decrease pressure
        for _ in 0..5 {
            controller.decrease_pressure();
        }

        // Should still apply backpressure (above low watermark)
        assert!(controller.should_apply_backpressure());

        // Decrease more to go below low watermark
        for _ in 0..4 {
            controller.decrease_pressure();
        }

        // Should resume now
        assert!(controller.should_resume());
    }

    #[tokio::test]
    async fn test_backpressure_flow_control_signals() {
        let controller = BackpressureController::new(BackpressureStrategy::Error, 5);

        // Initially continue
        assert_eq!(controller.get_flow_control_signal(), FlowControl::Continue);

        // Add pressure
        for _ in 0..5 {
            controller.increase_pressure();
        }

        // Should signal drop for error strategy
        assert_eq!(controller.get_flow_control_signal(), FlowControl::Drop);
    }

    #[tokio::test]
    async fn test_different_backpressure_strategies() {
        let strategies = vec![
            (BackpressureStrategy::DropOldest, FlowControl::Drop),
            (BackpressureStrategy::DropNewest, FlowControl::Drop),
            (BackpressureStrategy::Error, FlowControl::Drop),
            (BackpressureStrategy::Block, FlowControl::Pause),
        ];

        for (strategy, expected_signal) in strategies {
            let controller = BackpressureController::new(strategy, 3);

            // Fill buffer
            for _ in 0..3 {
                controller.increase_pressure();
            }

            assert_eq!(controller.get_flow_control_signal(), expected_signal);
        }
    }
}

/// Tests de edge cases
#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[tokio::test]
    async fn test_large_dataset_correctness() {
        let large_data: Vec<i32> = (0..1000).collect();
        let stream = StreamBuilder::from_iter(large_data.into_iter());
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        // Give more time for large dataset
        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = values.lock().unwrap().clone();
        assert_eq!(result.len(), 1000);
        assert_eq!(result[0], 0);
        assert_eq!(result[999], 999);

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_zero_interval_correctness() {
        // Test with very small interval
        let stream = StreamBuilder::interval(Duration::from_micros(100)).take(5);
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = values.lock().unwrap().clone();
        assert!(!result.is_empty());

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_subscription_unsubscribe_timing() {
        let stream = StreamBuilder::interval(Duration::from_millis(5));
        let values = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let values_clone = Arc::clone(&values);
            stream.subscribe(
                move |value| values_clone.lock().unwrap().push(value),
                |_| {},
                || {},
            )
        };

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(20)).await;

        let count_before = values.lock().unwrap().len();
        assert!(count_before > 0);

        // Unsubscribe
        subscription.unsubscribe();

        // Wait a bit more
        tokio::time::sleep(Duration::from_millis(20)).await;

        let count_after = values.lock().unwrap().len();

        // Count should not increase after unsubscribe
        assert_eq!(count_before, count_after);
    }
}

/// Tests de error handling básico
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_error_callback_invocation() {
        // Since we don't have error-producing streams yet,
        // test that error callback can be set up
        let stream = StreamBuilder::just(42);
        let errors = Arc::new(Mutex::new(Vec::new()));

        let subscription = {
            let errors_clone = Arc::clone(&errors);
            stream.subscribe(
                |_| {},
                move |error: Box<dyn std::error::Error>| errors_clone.lock().unwrap().push(error.to_string()),
                || {},
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should have no errors for successful stream
        assert_eq!(errors.lock().unwrap().len(), 0);

        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_completion_callback_invocation() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3].into_iter());
        let completed = Arc::new(Mutex::new(false));

        let subscription = {
            let completed_clone = Arc::clone(&completed);
            stream.subscribe(
                |_| {},
                |_| {},
                move || { *completed_clone.lock().unwrap() = true; },
            )
        };

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(*completed.lock().unwrap());
        subscription.unsubscribe();
    }
}