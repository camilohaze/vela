//! Integration tests for resilience patterns
//!
//! This module tests the complete integration of resilience decorators
//! from Vela source code through compilation to runtime execution.

use std::time::Duration;
use tokio::time::timeout;
use vela_compiler::Compiler;
use vela_runtime::vm::VirtualMachine;
use vela_runtime::resilience::{CircuitBreaker, CircuitBreakerConfig, RetryConfig, TimeoutConfig, BulkheadConfig, FallbackConfig};

/// Test circuit breaker integration
#[tokio::test]
async fn test_circuit_breaker_integration() {
    // Test circuit breaker state transitions
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        success_threshold: 1,
        call_timeout: Duration::from_millis(50),
    };

    let cb = CircuitBreaker::new("test_cb".to_string(), config);

    // Initially closed
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Closed);

    // Success calls should keep it closed
    for _ in 0..5 {
        let result = cb.call(|| async { Ok::<&str, &str>("success") }).await;
        assert_eq!(result, Ok("success"));
        assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Closed);
    }

    // Failure calls should eventually open it
    for i in 0..3 {
        let result = cb.call(|| async { Err::<&str, &str>("failure") }).await;
        if i >= 1 { // After 2 failures (threshold)
            assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Open);
        }
    }

    // Calls when open should fail fast
    let result = cb.call(|| async { Ok::<&str, &str>("should not execute") }).await;
    assert!(result.is_err());

    // Wait for recovery timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should transition to half-open
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::HalfOpen);

    // Success in half-open should close it
    let result = cb.call(|| async { Ok::<&str, &str>("recovery success") }).await;
    assert_eq!(result, Ok("recovery success"));
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Closed);
}

/// Test retry with exponential backoff
#[tokio::test]
async fn test_retry_with_backoff_integration() {
    use vela_runtime::resilience::with_retry;

    let config = RetryConfig {
        max_attempts: 3,
        backoff_multiplier: 2.0,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
    };

    let mut attempt_count = 0;

    let result = with_retry(config, || async {
        attempt_count += 1;
        if attempt_count < 3 {
            Err::<&str, &str>("temporary failure")
        } else {
            Ok("success after retry")
        }
    }).await;

    assert_eq!(result, Ok("success after retry"));
    assert_eq!(attempt_count, 3);
}

/// Test timeout functionality
#[tokio::test]
async fn test_timeout_integration() {
    use vela_runtime::resilience::with_timeout;

    // Test successful completion within timeout
    let result = with_timeout(Duration::from_millis(100), || async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<&str, &str>("completed")
    }).await;

    assert_eq!(result, Ok("completed"));

    // Test timeout expiration
    let result = with_timeout(Duration::from_millis(50), || async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok::<&str, &str>("should not complete")
    }).await;

    assert!(result.is_err());
}

/// Test bulkhead concurrency limits
#[tokio::test]
async fn test_bulkhead_integration() {
    use vela_runtime::resilience::Bulkhead;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let bulkhead = Arc::new(Bulkhead::new(2)); // Limit to 2 concurrent operations
    let mut handles = vec![];

    // Launch 5 concurrent operations
    for i in 0..5 {
        let bulkhead_clone = Arc::clone(&bulkhead);
        let handle = tokio::spawn(async move {
            bulkhead_clone.execute(|| async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<usize, &str>(i)
            }).await
        });
        handles.push(handle);
    }

    // Collect results
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    // Should have 3 errors (bulkhead limit exceeded) and 2 successes
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    let error_count = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(success_count, 2); // Only 2 concurrent allowed
    assert_eq!(error_count, 3);   // 3 should be rejected
}

/// Test fallback functionality
#[tokio::test]
async fn test_fallback_integration() {
    use vela_runtime::resilience::with_fallback;

    let config = FallbackConfig::default();

    // Test successful primary function
    let result = with_fallback(config.clone(),
        || async { Ok::<&str, &str>("primary success") },
        || async { Ok::<&str, &str>("fallback") }
    ).await;

    assert_eq!(result, Ok("primary success"));

    // Test fallback execution on primary failure
    let result = with_fallback(config,
        || async { Err::<&str, &str>("primary failed") },
        || async { Ok::<&str, &str>("fallback success") }
    ).await;

    assert_eq!(result, Ok("fallback success"));
}

/// Test combined resilience patterns
#[tokio::test]
async fn test_combined_resilience_patterns() {
    use vela_runtime::resilience::{with_timeout, with_retry, CircuitBreaker};
    use std::sync::Arc;

    // Create a circuit breaker
    let cb_config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout: Duration::from_millis(200),
        success_threshold: 1,
        call_timeout: Duration::from_millis(1000),
    };

    let cb = Arc::new(CircuitBreaker::new("combined_test".to_string(), cb_config));

    let mut call_count = 0;

    // Function that fails initially but succeeds after retries
    let test_fn = || async {
        call_count += 1;

        // Fail first 2 calls, succeed on 3rd
        if call_count < 3 {
            tokio::time::sleep(Duration::from_millis(50)).await; // Within timeout
            Err::<&str, &str>("temporary failure")
        } else {
            Ok("eventual success")
        }
    };

    // Apply timeout, then retry, then circuit breaker
    let result = cb.call(|| async {
        with_timeout(Duration::from_millis(200), || async {
            with_retry(RetryConfig {
                max_attempts: 3,
                backoff_multiplier: 1.5,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(50),
            }, test_fn).await
        }).await
    }).await;

    assert_eq!(result, Ok("eventual success"));
    assert_eq!(call_count, 3); // Should have been called 3 times total
}

/// Test resilience under load
#[tokio::test]
async fn test_resilience_under_load() {
    use vela_runtime::resilience::Bulkhead;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::time::Instant;

    let bulkhead = Arc::new(Bulkhead::new(5)); // Allow 5 concurrent
    let shared_counter = Arc::new(Mutex::new(0));
    let start_time = Instant::now();

    // Launch 20 concurrent operations
    let mut handles = vec![];
    for _ in 0..20 {
        let bulkhead_clone = Arc::clone(&bulkhead);
        let counter_clone = Arc::clone(&shared_counter);

        let handle = tokio::spawn(async move {
            bulkhead_clone.execute(|| async move {
                let mut counter = counter_clone.lock().await;
                *counter += 1;
                let execution_order = *counter;

                // Simulate variable execution time
                let sleep_time = 10 + (execution_order % 5) * 20; // 10-90ms
                tokio::time::sleep(Duration::from_millis(sleep_time)).await;

                Ok::<usize, &str>(execution_order)
            }).await
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }

    let elapsed = start_time.elapsed();

    // Should complete within reasonable time (bulkhead should allow controlled parallelism)
    assert!(elapsed < Duration::from_millis(500));

    // Should have exactly 5 successful operations (bulkhead limit)
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    let error_count = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(success_count, 5);
    assert_eq!(error_count, 15);
}

/// Test circuit breaker state persistence across multiple calls
#[tokio::test]
async fn test_circuit_breaker_state_persistence() {
    use vela_runtime::resilience::CircuitBreaker;

    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        recovery_timeout: Duration::from_millis(100),
        success_threshold: 2,
        call_timeout: Duration::from_millis(50),
    };

    let cb = CircuitBreaker::new("persistence_test".to_string(), config);

    // Initial state
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Closed);

    // Cause failures to open circuit
    for _ in 0..3 {
        let _ = cb.call(|| async { Err::<&str, &str>("failure") }).await;
    }

    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Open);

    // Multiple calls while open should all fail fast
    for _ in 0..5 {
        let result = cb.call(|| async { Ok::<&str, &str>("should not execute") }).await;
        assert!(result.is_err());
        assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Open);
    }

    // Wait for recovery
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be half-open
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::HalfOpen);

    // First success should keep it half-open (need success_threshold successes)
    let result = cb.call(|| async { Ok::<&str, &str>("first success") }).await;
    assert_eq!(result, Ok("first success"));
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::HalfOpen);

    // Second success should close it
    let result = cb.call(|| async { Ok::<&str, &str>("second success") }).await;
    assert_eq!(result, Ok("second success"));
    assert_eq!(cb.get_state().await, vela_runtime::resilience::CircuitBreakerState::Closed);
}

/// Test retry backoff timing
#[tokio::test]
async fn test_retry_backoff_timing() {
    use vela_runtime::resilience::with_retry;
    use std::time::Instant;

    let config = RetryConfig {
        max_attempts: 4,
        backoff_multiplier: 2.0,
        initial_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(200),
    };

    let start_time = Instant::now();
    let mut attempt_times = vec![];

    let result = with_retry(config, || async {
        attempt_times.push(start_time.elapsed());

        // Always fail to test timing
        Err::<&str, &str>("always fail")
    }).await;

    assert!(result.is_err());
    assert_eq!(attempt_times.len(), 4); // Should have 4 attempts

    // Check that delays are increasing (exponential backoff)
    // attempt_times[1] - attempt_times[0] should be ~50ms (initial)
    // attempt_times[2] - attempt_times[1] should be ~100ms (50 * 2)
    // attempt_times[3] - attempt_times[2] should be ~200ms (100 * 2, but capped at max_delay)

    let delay1 = attempt_times[1] - attempt_times[0];
    let delay2 = attempt_times[2] - attempt_times[1];
    let delay3 = attempt_times[3] - attempt_times[2];

    assert!(delay1 >= Duration::from_millis(40)); // Allow some variance
    assert!(delay2 >= delay1); // Should be increasing
    assert!(delay3 >= delay2); // Should be increasing
}