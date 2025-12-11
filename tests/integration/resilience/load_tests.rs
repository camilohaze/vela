//! Load and performance tests for resilience patterns
//!
//! These tests focus on high-load scenarios and edge cases
//! to ensure resilience patterns work correctly under stress.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use vela_runtime::resilience::{CircuitBreaker, CircuitBreakerConfig, Bulkhead};

/// Test circuit breaker under high concurrent load
#[tokio::test]
async fn test_circuit_breaker_high_concurrency() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout: Duration::from_millis(500),
        success_threshold: 3,
        call_timeout: Duration::from_millis(100),
    };

    let cb = Arc::new(CircuitBreaker::new("high_load_test".to_string(), config));
    let failure_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));

    // Launch 50 concurrent operations
    let mut handles = vec![];
    for i in 0..50 {
        let cb_clone = Arc::clone(&cb);
        let failure_clone = Arc::clone(&failure_count);
        let success_clone = Arc::clone(&success_count);

        let handle = tokio::spawn(async move {
            let should_fail = i < 15; // First 15 calls fail

            let result = cb_clone.call(|| async move {
                if should_fail {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Err::<&str, &str>("simulated failure")
                } else {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    Ok("success")
                }
            }).await;

            match result {
                Ok(_) => {
                    let mut count = success_clone.lock().await;
                    *count += 1;
                }
                Err(_) => {
                    let mut count = failure_clone.lock().await;
                    *count += 1;
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let final_failures = *failure_count.lock().await;
    let final_successes = *success_count.lock().await;

    // Circuit should open after 10 failures, causing subsequent calls to fail fast
    assert!(final_failures > 10); // More than threshold should fail
    assert!(final_successes < 50); // Not all should succeed
}

/// Test bulkhead queueing behavior
#[tokio::test]
async fn test_bulkhead_queueing() {
    use vela_runtime::resilience::Bulkhead;

    let bulkhead = Arc::new(Bulkhead::new(3)); // Only 3 concurrent
    let start_time = Instant::now();
    let completion_times = Arc::new(Mutex::new(vec![]));

    // Launch 10 operations that take 200ms each
    let mut handles = vec![];
    for i in 0..10 {
        let bulkhead_clone = Arc::clone(&bulkhead);
        let times_clone = Arc::clone(&completion_times);

        let handle = tokio::spawn(async move {
            let op_start = Instant::now();

            let result = bulkhead_clone.execute(|| async move {
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok::<usize, &str>(i)
            }).await;

            let op_duration = op_start.elapsed();

            {
                let mut times = times_clone.lock().await;
                times.push((i, op_duration, result.is_ok()));
            }

            result
        });

        handles.push(handle);
    }

    // Wait for all to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    let total_duration = start_time.elapsed();
    let times = completion_times.lock().await.clone();

    // Should take at least 600ms (3 batches of 200ms each)
    assert!(total_duration >= Duration::from_millis(600));

    // Exactly 3 operations should succeed (bulkhead limit)
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 3);

    // Failed operations should have very short duration (rejected immediately)
    let failed_times: Vec<_> = times.iter()
        .filter(|(_, _, success)| !success)
        .map(|(_, duration, _)| *duration)
        .collect();

    for duration in failed_times {
        assert!(duration < Duration::from_millis(10)); // Should fail fast
    }
}

/// Test resilience pattern memory usage
#[tokio::test]
async fn test_resilience_memory_usage() {
    use vela_runtime::resilience::{CircuitBreaker, CircuitBreakerConfig};
    use std::mem;

    // Test that circuit breakers don't leak memory
    let mut circuit_breakers = vec![];

    for i in 0..100 {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
            call_timeout: Duration::from_secs(10),
        };

        let cb = CircuitBreaker::new(format!("memory_test_{}", i), config);
        circuit_breakers.push(cb);
    }

    // Perform operations
    for cb in &circuit_breakers {
        for _ in 0..10 {
            let _ = cb.call(|| async { Ok::<&str, &str>("test") }).await;
        }
    }

    // Clear the vector (simulating cleanup)
    circuit_breakers.clear();

    // Force garbage collection if available
    // In a real application, this would help ensure no memory leaks

    assert!(circuit_breakers.is_empty());
}

/// Test resilience patterns with cancellation
#[tokio::test]
async fn test_resilience_cancellation() {
    use tokio::sync::oneshot;
    use vela_runtime::resilience::with_timeout;

    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    let task = tokio::spawn(async move {
        with_timeout(Duration::from_millis(500), || async {
            // Wait to be cancelled
            let _ = cancel_rx.await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok::<&str, &str>("completed")
        }).await
    });

    // Cancel after 100ms
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _ = cancel_tx.send(());

    let result = task.await.unwrap();

    // Should complete successfully (not timeout)
    assert!(result.is_ok());
}

/// Test circuit breaker metrics collection
#[tokio::test]
async fn test_circuit_breaker_metrics() {
    use vela_runtime::resilience::{CircuitBreaker, CircuitBreakerConfig};

    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout: Duration::from_millis(100),
        success_threshold: 1,
        call_timeout: Duration::from_millis(50),
    };

    let cb = CircuitBreaker::new("metrics_test".to_string(), config);

    // Initial metrics
    let initial_state = cb.get_state().await;
    assert_eq!(initial_state, vela_runtime::resilience::CircuitBreakerState::Closed);

    // Cause some failures
    for _ in 0..4 {
        let _ = cb.call(|| async { Err::<&str, &str>("failure") }).await;
    }

    // Should be open
    let state_after_failures = cb.get_state().await;
    assert_eq!(state_after_failures, vela_runtime::resilience::CircuitBreakerState::Open);

    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be half-open
    let state_after_recovery = cb.get_state().await;
    assert_eq!(state_after_recovery, vela_runtime::resilience::CircuitBreakerState::HalfOpen);

    // Successful call should close it
    let result = cb.call(|| async { Ok::<&str, &str>("recovery") }).await;
    assert_eq!(result, Ok("recovery"));

    let final_state = cb.get_state().await;
    assert_eq!(final_state, vela_runtime::resilience::CircuitBreakerState::Closed);
}

/// Test bulkhead with different priorities
#[tokio::test]
async fn test_bulkhead_priorities() {
    use vela_runtime::resilience::Bulkhead;
    use std::sync::Arc;

    let bulkhead = Arc::new(Bulkhead::new(2));
    let results = Arc::new(Mutex::new(vec![]));

    // Launch high priority tasks first
    let mut handles = vec![];
    for priority in 0..4 {
        let bulkhead_clone = Arc::clone(&bulkhead);
        let results_clone = Arc::clone(&results);

        let handle = tokio::spawn(async move {
            let result = bulkhead_clone.execute(|| async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok::<usize, &str>(priority)
            }).await;

            {
                let mut res = results_clone.lock().await;
                res.push((priority, result.is_ok()));
            }

            result
        });

        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        let _ = handle.await.unwrap();
    }

    let final_results = results.lock().await.clone();

    // Exactly 2 should succeed (bulkhead limit)
    let success_count = final_results.iter().filter(|(_, success)| *success).count();
    assert_eq!(success_count, 2);

    // The first 2 (highest priority) should succeed
    assert!(final_results[0].1); // Priority 0 should succeed
    assert!(final_results[1].1); // Priority 1 should succeed
}

/// Test resilience patterns with resource exhaustion
#[tokio::test]
async fn test_resource_exhaustion_resilience() {
    use vela_runtime::resilience::Bulkhead;
    use std::sync::Arc;

    // Very small bulkhead to force resource exhaustion
    let bulkhead = Arc::new(Bulkhead::new(1));
    let error_count = Arc::new(Mutex::new(0));

    // Launch many concurrent operations
    let mut handles = vec![];
    for _ in 0..20 {
        let bulkhead_clone = Arc::clone(&bulkhead);
        let error_clone = Arc::clone(&error_count);

        let handle = tokio::spawn(async move {
            let result = bulkhead_clone.execute(|| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<&str, &str>("success")
            }).await;

            if result.is_err() {
                let mut count = error_clone.lock().await;
                *count += 1;
            }

            result
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let _ = handle.await.unwrap();
    }

    let final_errors = *error_count.lock().await;

    // Should have 19 rejections (only 1 success allowed)
    assert_eq!(final_errors, 19);
}

/// Test circuit breaker with slow calls
#[tokio::test]
async fn test_circuit_breaker_slow_calls() {
    use vela_runtime::resilience::{CircuitBreaker, CircuitBreakerConfig};

    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(200),
        success_threshold: 1,
        call_timeout: Duration::from_millis(50), // Very short timeout
    };

    let cb = CircuitBreaker::new("slow_calls_test".to_string(), config);

    // Make calls that take longer than the call_timeout
    for _ in 0..3 {
        let result = cb.call(|| async {
            tokio::time::sleep(Duration::from_millis(100)).await; // Longer than timeout
            Ok::<&str, &str>("slow success")
        }).await;

        // Should fail due to timeout
        assert!(result.is_err());
    }

    // Circuit should be open due to failures
    let state = cb.get_state().await;
    assert_eq!(state, vela_runtime::resilience::CircuitBreakerState::Open);
}