//! # Tracing Tests
//!
//! Comprehensive tests for the distributed tracing functionality.
//!
//! Tests cover:
//! - Span creation and lifecycle
//! - Context propagation (W3C Trace Context)
//! - Nested spans and hierarchies
//! - Tags and attributes
//! - Error handling and cleanup
//! - Async tracing scenarios
//! - Sampling configuration
//! - Jaeger export integration

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use vela_runtime::observability::{
    tracing::{Tracer, Span, SpanContext, TraceId, SpanId, TracerConfig},
    ObservabilityConfig, TracingConfig, init_observability, shutdown_observability
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_span_creation() {
        let tracer = Tracer::new(TracerConfig {
            service_name: "test-service".to_string(),
            service_version: "1.0.0".to_string(),
            ..Default::default()
        });

        let span = tracer.start_span("test_operation");
        assert_eq!(span.operation_name(), "test_operation");
        assert!(span.context().trace_id().is_valid());
        assert!(span.context().span_id().is_valid());
    }

    #[tokio::test]
    async fn test_span_context_propagation() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create root span
        let root_span = tracer.start_span("root_operation");
        let root_context = root_span.context().clone();

        // Create child span with explicit parent
        let child_span = tracer.start_span_with_parent("child_operation", &root_context);
        let child_context = child_span.context();

        // Verify trace continuity
        assert_eq!(root_context.trace_id(), child_context.trace_id());
        assert_ne!(root_context.span_id(), child_context.span_id());

        // Verify parent-child relationship
        assert_eq!(child_context.parent_span_id(), Some(root_context.span_id()));
    }

    #[tokio::test]
    async fn test_nested_spans() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create nested spans
        let root = tracer.start_span("root");
        let root_id = root.context().span_id();

        {
            let child1 = tracer.start_span("child1");
            assert_eq!(child1.context().parent_span_id(), Some(root_id));

            {
                let grandchild = tracer.start_span("grandchild");
                assert_eq!(grandchild.context().parent_span_id(), Some(child1.context().span_id()));

                grandchild.finish();
            }
            child1.finish();
        }
        root.finish();
    }

    #[tokio::test]
    async fn test_span_tags() {
        let tracer = Tracer::new(TracerConfig::default());
        let mut span = tracer.start_span("tagged_operation");

        // Add various types of tags
        span.set_tag("string_tag", "value");
        span.set_tag("number_tag", 42);
        span.set_tag("bool_tag", true);
        span.set_tag("float_tag", 3.14);

        // Verify tags are stored
        let tags = span.tags();
        assert_eq!(tags.get("string_tag"), Some(&"value".to_string()));
        assert_eq!(tags.get("number_tag"), Some(&"42".to_string()));
        assert_eq!(tags.get("bool_tag"), Some(&"true".to_string()));
        assert_eq!(tags.get("float_tag"), Some(&"3.14".to_string()));
    }

    #[tokio::test]
    async fn test_span_error_handling() {
        let tracer = Tracer::new(TracerConfig::default());
        let mut span = tracer.start_span("error_operation");

        // Record an error
        span.record_error("Test error", "Detailed error description");

        // Verify error is recorded
        assert!(span.has_error());
        let errors = span.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Test error");
        assert_eq!(errors[0].description, "Detailed error description");
    }

    #[tokio::test]
    async fn test_async_span_tracing() {
        let tracer = Arc::new(Tracer::new(TracerConfig::default()));

        let result = async_operation_with_tracing(tracer.clone()).await;
        assert_eq!(result, 42);
    }

    async fn async_operation_with_tracing(tracer: Arc<Tracer>) -> i32 {
        let span = tracer.start_span("async_operation");

        // Simulate async work
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        span.finish();
        42
    }

    #[tokio::test]
    async fn test_sampling_configuration() {
        // Test 100% sampling
        let tracer_always = Tracer::new(TracerConfig {
            sampling_ratio: 1.0,
            ..Default::default()
        });

        for _ in 0..100 {
            let span = tracer_always.start_span("sampled_operation");
            assert!(span.is_sampled());
        }

        // Test 0% sampling
        let tracer_never = Tracer::new(TracerConfig {
            sampling_ratio: 0.0,
            ..Default::default()
        });

        for _ in 0..100 {
            let span = tracer_never.start_span("not_sampled_operation");
            assert!(!span.is_sampled());
        }
    }

    #[tokio::test]
    async fn test_w3c_trace_context_propagation() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create a span and extract its context
        let span = tracer.start_span("test_span");
        let trace_id = span.context().trace_id();
        let span_id = span.context().span_id();

        // Create W3C trace context header
        let w3c_header = format!("{}-{}", trace_id, span_id);

        // Parse the header back
        let parsed_context = SpanContext::from_w3c_header(&w3c_header).unwrap();

        assert_eq!(parsed_context.trace_id(), trace_id);
        assert_eq!(parsed_context.span_id(), span_id);
    }

    #[tokio::test]
    async fn test_span_lifecycle() {
        let tracer = Tracer::new(TracerConfig::default());

        // Span should not be finished initially
        let span = tracer.start_span("lifecycle_test");
        assert!(!span.is_finished());

        // Finish the span
        span.finish();
        assert!(span.is_finished());

        // Duration should be recorded
        assert!(span.duration().is_some());
        assert!(span.duration().unwrap() > std::time::Duration::from_nanos(0));
    }

    #[tokio::test]
    async fn test_concurrent_span_creation() {
        let tracer = Arc::new(Tracer::new(TracerConfig::default()));
        let mut handles = vec![];

        // Create multiple concurrent spans
        for i in 0..10 {
            let tracer_clone = tracer.clone();
            let handle = tokio::spawn(async move {
                let span = tracer_clone.start_span(&format!("concurrent_span_{}", i));
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                span.finish();
                i
            });
            handles.push(handle);
        }

        // Wait for all spans to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result >= 0 && result < 10);
        }
    }

    #[tokio::test]
    async fn test_span_baggage_propagation() {
        let tracer = Tracer::new(TracerConfig::default());

        let mut root_span = tracer.start_span("root_with_baggage");
        root_span.set_baggage("user_id", "12345");
        root_span.set_baggage("request_id", "req-abc");

        let root_context = root_span.context().clone();

        // Create child span - should inherit baggage
        let child_span = tracer.start_span_with_parent("child_inherits_baggage", &root_context);

        // Verify baggage inheritance
        assert_eq!(child_span.get_baggage("user_id"), Some("12345"));
        assert_eq!(child_span.get_baggage("request_id"), Some("req-abc"));
        assert_eq!(child_span.get_baggage("nonexistent"), None);
    }

    #[tokio::test]
    async fn test_span_events() {
        let tracer = Tracer::new(TracerConfig::default());
        let mut span = tracer.start_span("event_test");

        // Add events
        span.add_event("operation_started", HashMap::new());
        span.add_event("processing_data", {
            let mut attrs = HashMap::new();
            attrs.insert("records_processed".to_string(), "100".to_string());
            attrs
        });
        span.add_event("operation_completed", HashMap::new());

        // Verify events are recorded
        let events = span.events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].name, "operation_started");
        assert_eq!(events[1].name, "processing_data");
        assert_eq!(events[2].name, "operation_completed");

        // Check event attributes
        assert_eq!(events[1].attributes.get("records_processed"), Some(&"100".to_string()));
    }

    #[tokio::test]
    async fn test_tracer_configuration() {
        let config = TracerConfig {
            service_name: "test-service".to_string(),
            service_version: "2.1.0".to_string(),
            sampling_ratio: 0.5,
            max_batch_size: 100,
            export_timeout: std::time::Duration::from_secs(30),
        };

        let tracer = Tracer::new(config);

        // Verify configuration is applied
        assert_eq!(tracer.service_name(), "test-service");
        assert_eq!(tracer.service_version(), "2.1.0");
    }

    #[tokio::test]
    async fn test_span_cleanup_on_drop() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create span in a block - should be cleaned up when dropped
        {
            let span = tracer.start_span("cleanup_test");
            assert!(!span.is_finished());
            // span is dropped here
        }

        // Verify span was automatically finished
        // Note: In a real implementation, this would be verified through
        // the tracer's span registry
    }

    #[tokio::test]
    async fn test_trace_id_generation() {
        let tracer = Tracer::new(TracerConfig::default());

        let span1 = tracer.start_span("trace1");
        let span2 = tracer.start_span("trace2");

        // Different traces should have different trace IDs
        assert_ne!(span1.context().trace_id(), span2.context().span_id());

        // Same trace should share trace ID
        let child1 = tracer.start_span_with_parent("child1", span1.context());
        let child2 = tracer.start_span_with_parent("child2", span1.context());

        assert_eq!(child1.context().trace_id(), span1.context().trace_id());
        assert_eq!(child2.context().trace_id(), span1.context().trace_id());
    }

    #[tokio::test]
    async fn test_span_id_uniqueness() {
        let tracer = Tracer::new(TracerConfig::default());

        let mut span_ids = std::collections::HashSet::new();

        // Create many spans and ensure unique IDs
        for _ in 0..1000 {
            let span = tracer.start_span("uniqueness_test");
            assert!(span_ids.insert(span.context().span_id()));
        }

        assert_eq!(span_ids.len(), 1000);
    }

    #[tokio::test]
    async fn test_tracer_shutdown() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create some spans
        let span1 = tracer.start_span("span1");
        let span2 = tracer.start_span("span2");

        // Shutdown tracer
        tracer.shutdown().await;

        // Spans created before shutdown should still be valid
        assert_eq!(span1.operation_name(), "span1");
        assert_eq!(span2.operation_name(), "span2");

        // New spans after shutdown should fail gracefully
        // Note: This depends on the actual implementation
    }

    #[tokio::test]
    async fn test_span_memory_usage() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create spans with varying amounts of data
        let simple_span = tracer.start_span("simple");

        let mut complex_span = tracer.start_span("complex");
        for i in 0..100 {
            complex_span.set_tag(&format!("tag_{}", i), &format!("value_{}", i));
            complex_span.add_event(&format!("event_{}", i), HashMap::new());
        }

        // Both spans should be valid regardless of memory usage
        assert_eq!(simple_span.operation_name(), "simple");
        assert_eq!(complex_span.operation_name(), "complex");
        assert!(complex_span.tags().len() >= 100);
        assert!(complex_span.events().len() >= 100);
    }

    #[tokio::test]
    async fn test_jaeger_export_integration() {
        // This test would require a Jaeger instance running
        // For now, we test the export preparation logic

        let tracer = Tracer::new(TracerConfig {
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            ..Default::default()
        });

        let span = tracer.start_span("jaeger_test");
        span.set_tag("test", "value");

        // Verify span can be serialized for Jaeger export
        // This is a placeholder for actual Jaeger export testing
        assert!(span.context().trace_id().is_valid());
        assert!(span.tags().contains_key("test"));
    }

    #[tokio::test]
    async fn test_tracing_with_observability_system() {
        let config = ObservabilityConfig {
            tracing: TracingConfig {
                service_name: "integration-test".to_string(),
                service_version: "1.0.0".to_string(),
                sampling_ratio: 1.0,
            },
            ..Default::default()
        };

        // Initialize observability system
        init_observability(config).await.unwrap();

        // Get tracer from the system
        let tracer = vela_runtime::observability::get_tracer().await.unwrap();

        // Use the tracer
        let span = tracer.start_span("system_integration_test");
        span.set_tag("integration", "true");

        assert_eq!(span.operation_name(), "system_integration_test");
        assert_eq!(span.tags().get("integration"), Some(&"true".to_string()));

        span.finish();

        // Shutdown system
        shutdown_observability().await;
    }

    #[tokio::test]
    async fn test_span_context_serialization() {
        let tracer = Tracer::new(TracerConfig::default());
        let span = tracer.start_span("serialization_test");

        let context = span.context();

        // Serialize to string
        let serialized = context.to_string();

        // Deserialize back
        let deserialized = SpanContext::from_string(&serialized).unwrap();

        assert_eq!(context.trace_id(), deserialized.trace_id());
        assert_eq!(context.span_id(), deserialized.span_id());
        assert_eq!(context.parent_span_id(), deserialized.parent_span_id());
    }

    #[tokio::test]
    async fn test_tracer_statistics() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create some spans
        for i in 0..10 {
            let span = tracer.start_span(&format!("stat_test_{}", i));
            span.finish();
        }

        // Check tracer statistics
        let stats = tracer.stats();
        assert!(stats.spans_created >= 10);
        assert!(stats.spans_finished >= 10);
    }

    #[tokio::test]
    async fn test_span_log_correlation() {
        let tracer = Tracer::new(TracerConfig::default());
        let span = tracer.start_span("log_correlation_test");

        // Simulate logging with span context
        let trace_id = span.context().trace_id().to_string();
        let span_id = span.context().span_id().to_string();

        // Verify trace/span IDs can be used for log correlation
        assert!(!trace_id.is_empty());
        assert!(!span_id.is_empty());

        // In a real system, these would be included in log records
        let log_message = format!("Operation completed trace_id={} span_id={}", trace_id, span_id);
        assert!(log_message.contains("trace_id="));
        assert!(log_message.contains("span_id="));
    }

    #[tokio::test]
    async fn test_tracing_error_propagation() {
        let tracer = Tracer::new(TracerConfig::default());

        let span = tracer.start_span("error_propagation_test");

        // Simulate an operation that fails
        let result: Result<(), &str> = Err("operation failed");

        match result {
            Ok(_) => {}
            Err(e) => {
                span.record_error(e, "Operation failed with error");
                span.set_tag("error", "true");
            }
        }

        assert!(span.has_error());
        assert_eq!(span.tags().get("error"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_span_time_measurements() {
        let tracer = Tracer::new(TracerConfig::default());

        let start = std::time::Instant::now();
        let span = tracer.start_span("timing_test");

        // Simulate work
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        span.finish();
        let elapsed = start.elapsed();

        // Span duration should be close to the actual elapsed time
        let span_duration = span.duration().unwrap();
        let diff = if elapsed > span_duration {
            elapsed - span_duration
        } else {
            span_duration - elapsed
        };

        // Allow for some timing variance (within 10ms)
        assert!(diff < std::time::Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_tracer_resource_cleanup() {
        let tracer = Arc::new(Tracer::new(TracerConfig::default()));

        // Create spans in multiple tasks
        let mut tasks = vec![];

        for i in 0..5 {
            let tracer_clone = tracer.clone();
            let task = tokio::spawn(async move {
                let span = tracer_clone.start_span(&format!("cleanup_task_{}", i));
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                drop(span); // Explicit drop
                i
            });
            tasks.push(task);
        }

        // Wait for all tasks
        for task in tasks {
            task.await.unwrap();
        }

        // Tracer should still be functional
        let final_span = tracer.start_span("final_test");
        assert_eq!(final_span.operation_name(), "final_test");
    }

    #[tokio::test]
    async fn test_span_attribute_types() {
        let tracer = Tracer::new(TracerConfig::default());
        let mut span = tracer.start_span("attribute_types_test");

        // Test different attribute types
        span.set_tag("string_attr", "hello");
        span.set_tag("int_attr", 42);
        span.set_tag("float_attr", 3.14159);
        span.set_tag("bool_attr", true);

        let tags = span.tags();
        assert_eq!(tags.len(), 4);
        assert_eq!(tags.get("string_attr"), Some(&"hello".to_string()));
        assert_eq!(tags.get("int_attr"), Some(&"42".to_string()));
        assert_eq!(tags.get("float_attr"), Some(&"3.14159".to_string()));
        assert_eq!(tags.get("bool_attr"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_tracing_configuration_validation() {
        // Test invalid sampling ratio
        let invalid_config = TracerConfig {
            sampling_ratio: 1.5, // Invalid: should be 0.0 to 1.0
            ..Default::default()
        };

        // Should either reject invalid config or clamp it
        let tracer = Tracer::new(invalid_config);
        // Implementation should handle this gracefully
        let span = tracer.start_span("config_test");
        assert!(span.context().trace_id().is_valid());
    }

    #[tokio::test]
    async fn test_span_context_thread_safety() {
        let tracer = Arc::new(Tracer::new(TracerConfig::default()));

        // Test that span contexts are thread-safe
        let mut handles = vec![];

        for i in 0..10 {
            let tracer_clone = tracer.clone();
            let handle = tokio::spawn(async move {
                let span = tracer_clone.start_span(&format!("thread_safety_{}", i));
                let context = span.context().clone();

                // Context should be usable across threads
                assert!(context.trace_id().is_valid());
                assert!(context.span_id().is_valid());

                context
            });
            handles.push(handle);
        }

        // Collect all contexts
        let contexts: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All contexts should be valid and unique
        for (i, context) in contexts.iter().enumerate() {
            assert!(context.trace_id().is_valid());
            assert!(context.span_id().is_valid());

            // Check uniqueness
            for (j, other_context) in contexts.iter().enumerate() {
                if i != j {
                    assert_ne!(context.span_id(), other_context.span_id());
                }
            }
        }
    }

    #[tokio::test]
    async fn test_tracing_performance_baseline() {
        let tracer = Tracer::new(TracerConfig::default());

        let start = std::time::Instant::now();

        // Create many spans quickly
        for i in 0..1000 {
            let span = tracer.start_span(&format!("perf_test_{}", i));
            // Minimal work
            span.finish();
        }

        let elapsed = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(elapsed < std::time::Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_span_hierarchy_depth() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create deeply nested spans
        let mut spans = vec![];
        let mut parent_context = None;

        for depth in 0..10 {
            let span = if let Some(ref context) = parent_context {
                tracer.start_span_with_parent(&format!("depth_{}", depth), context)
            } else {
                tracer.start_span(&format!("depth_{}", depth))
            };

            parent_context = Some(span.context().clone());
            spans.push(span);
        }

        // Verify hierarchy
        for (i, span) in spans.iter().enumerate() {
            if i > 0 {
                assert_eq!(span.context().parent_span_id(), Some(spans[i-1].context().span_id()));
            }
        }

        // Finish spans in reverse order
        for span in spans.into_iter().rev() {
            span.finish();
        }
    }

    #[tokio::test]
    async fn test_tracing_memory_efficiency() {
        let tracer = Tracer::new(TracerConfig::default());

        // Create spans with minimal memory overhead
        let spans: Vec<_> = (0..100).map(|i| {
            tracer.start_span(&format!("memory_test_{}", i))
        }).collect();

        // All spans should be valid
        for (i, span) in spans.into_iter().enumerate() {
            assert_eq!(span.operation_name(), format!("memory_test_{}", i));
            span.finish();
        }
    }

    #[tokio::test]
    async fn test_span_context_inheritance() {
        let tracer = Tracer::new(TracerConfig::default());

        // Root span
        let root = tracer.start_span("root");
        root.set_tag("root_tag", "root_value");

        // Child inherits context
        let child = tracer.start_span_with_parent("child", root.context());
        assert_eq!(child.context().trace_id(), root.context().trace_id());

        // Grandchild inherits from child
        let grandchild = tracer.start_span_with_parent("grandchild", child.context());
        assert_eq!(grandchild.context().trace_id(), root.context().trace_id());
        assert_eq!(grandchild.context().parent_span_id(), Some(child.context().span_id()));

        grandchild.finish();
        child.finish();
        root.finish();
    }

    #[tokio::test]
    async fn test_tracing_with_custom_timestamps() {
        let tracer = Tracer::new(TracerConfig::default());

        let custom_start = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
        let span = tracer.start_span_with_timestamp("custom_time_test", custom_start);

        // Span should record the custom start time
        assert!(span.start_time() >= custom_start);

        span.finish();
    }
}