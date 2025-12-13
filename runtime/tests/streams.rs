//! Tests unitarios para la Stream API
//!
//! Tests completos para validar la funcionalidad de la Stream API
//! incluyendo operadores funcionales, backpressure y composición.

use std::time::Duration;
use tokio::time::timeout;

use vela_runtime::streams::*;

/// Tests básicos de creación de streams
#[cfg(test)]
mod stream_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_just() {
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

        // Wait a bit for async processing
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(*values.lock().unwrap(), vec![42]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_empty() {
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

        assert!(values.lock().unwrap().is_empty());
        assert!(*completed.lock().unwrap());
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_from_iter() {
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
}

/// Tests de operadores funcionales
#[cfg(test)]
mod stream_operators_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_map() {
        let stream = StreamBuilder::just(21).map(|x| x * 2);
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
    async fn test_stream_filter() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter())
            .filter(|&x| x % 2 == 0);

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

        assert_eq!(*values.lock().unwrap(), vec![2, 4]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_take() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5].into_iter())
            .take(3);

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

        assert_eq!(*values.lock().unwrap(), vec![1, 2, 3]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_drop() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5])
            .drop(2);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![3, 4, 5]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_take_while() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5])
            .take_while(|&x| x < 4);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![1, 2, 3]);
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_buffer() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5])
            .buffer(2);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![vec![1, 2], vec![3, 4], vec![5]]);
        subscription.unsubscribe();
    }
}

/// Tests de composición de operadores
#[cfg(test)]
mod stream_composition_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_map_filter_composition() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5, 6])
            .filter(|&x| x % 2 == 0)
            .map(|x| x * 3);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![6, 12, 18]); // 2*3, 4*3, 6*3
        subscription.unsubscribe();
    }

    #[tokio::test]
    async fn test_stream_take_map_filter() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5, 6, 7, 8])
            .take(5)
            .filter(|&x| x % 2 == 0)
            .map(|x| x * 10);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![20, 40]); // 2*10, 4*10 (from first 5: 1,2,3,4,5)
        subscription.unsubscribe();
    }
}

/// Tests de backpressure
#[cfg(test)]
mod backpressure_tests {
    use super::*;

    #[test]
    fn test_backpressure_buffer_basic() {
        let mut buffer = BackpressureBuffer::new(3);

        assert!(buffer.offer(1));
        assert!(buffer.offer(2));
        assert!(buffer.offer(3));
        assert!(!buffer.offer(4)); // Buffer full

        assert_eq!(buffer.poll(), Some(1));
        assert_eq!(buffer.poll(), Some(2));
        assert!(buffer.offer(4)); // Now can offer

        assert_eq!(buffer.size(), 2);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_backpressure_buffer_empty() {
        let mut buffer = BackpressureBuffer::<i32>::new(5);

        assert_eq!(buffer.poll(), None);
        assert!(buffer.is_empty());
        assert_eq!(buffer.size(), 0);
    }

    #[test]
    fn test_backpressure_buffer_capacity_zero() {
        let mut buffer = BackpressureBuffer::<i32>::new(0);

        assert!(!buffer.offer(1));
        assert_eq!(buffer.poll(), None);
    }
}

/// Tests de subscription
#[cfg(test)]
mod subscription_tests {
    use super::*;

    #[test]
    fn test_subscription_basic() {
        let subscription = Subscription::new();

        assert!(subscription.is_subscribed());

        subscription.unsubscribe();
        assert!(!subscription.is_subscribed());
    }

    #[test]
    fn test_subscription_clone() {
        let subscription1 = Subscription::new();
        let subscription2 = subscription1.clone();

        assert!(subscription1.is_subscribed());
        assert!(subscription2.is_subscribed());

        subscription1.unsubscribe();

        assert!(!subscription1.is_subscribed());
        assert!(!subscription2.is_subscribed()); // Should be shared state
    }
}

/// Tests de interval stream
#[cfg(test)]
mod interval_tests {
    use super::*;

    #[tokio::test]
    async fn test_interval_stream() {
        let stream = StreamBuilder::interval(Duration::from_millis(10));
        let mut values = Vec::new();

        let subscription = stream.take(3).subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        // Wait for 3 intervals + some buffer
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(values.len(), 3);
        assert_eq!(values, vec![0, 1, 2]);
        subscription.unsubscribe();
    }
}

/// Tests de reduce operation
#[cfg(test)]
mod reduce_tests {
    use super::*;

    #[tokio::test]
    async fn test_reduce_sum() {
        let stream = StreamBuilder::from_iter(vec![1, 2, 3, 4, 5]);

        let result = timeout(
            Duration::from_millis(100),
            stream.reduce(0, |acc, x| acc + x)
        ).await;

        match result {
            Ok(sum) => assert_eq!(sum, 15),
            Err(_) => panic!("Reduce operation timed out"),
        }
    }

    #[tokio::test]
    async fn test_reduce_max() {
        let stream = StreamBuilder::from_iter(vec![3, 1, 4, 1, 5, 9, 2, 6]);

        let result = timeout(
            Duration::from_millis(100),
            stream.reduce(i32::MIN, |acc, x| if x > acc { x } else { acc })
        ).await;

        match result {
            Ok(max) => assert_eq!(max, 9),
            Err(_) => panic!("Reduce operation timed out"),
        }
    }
}

/// Tests de error handling
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_error_propagation() {
        // Test that errors are properly propagated through operators
        // This would require a stream that can emit errors
        // For now, just test basic error handling setup
        let stream = StreamBuilder::just(42);
        let mut errors = Vec::new();
        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |error| errors.push(error),
            || {},
        );

        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(values, vec![42]);
        assert!(errors.is_empty());
        subscription.unsubscribe();
    }
}

/// Tests de performance y límites
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_large_stream_processing() {
        let data: Vec<i32> = (0..1000).collect();
        let stream = StreamBuilder::from_iter(data.into_iter())
            .filter(|&x| x % 2 == 0)
            .map(|x| x * 2)
            .take(10);

        let mut values = Vec::new();

        let subscription = stream.subscribe(
            |value| values.push(value),
            |_| {},
            || {},
        );

        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should have first 10 even numbers, doubled: 0, 4, 8, 12, 16, 20, 24, 28, 32, 36
        let expected: Vec<i32> = (0..10).map(|x| x * 4).collect();
        assert_eq!(values, expected);
        subscription.unsubscribe();
    }
}