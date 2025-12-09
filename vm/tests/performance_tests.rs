/*!
Performance Tests for VelaVM Memory Management

Benchmarks and performance validation for GC operations.
*/

use vela_vm::gc::{GcHeap, GcObject};
use vela_vm::bytecode::Value;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_gc_performance_small_heap() {
        let mut heap = GcHeap::new();

        // Allocate small number of objects
        for i in 0..100 {
            let _ = heap.alloc_string(format!("obj_{}", i));
        }

        // Measure collection time
        let start = Instant::now();
        let freed = heap.force_collect().unwrap();
        let duration = start.elapsed();

        // Should be fast (< 1ms)
        assert!(duration < Duration::from_millis(1));
        assert_eq!(freed, 100);
    }

    #[test]
    fn test_gc_performance_large_heap() {
        let mut heap = GcHeap::new();

        // Allocate larger number of objects
        for i in 0..1000 {
            let _ = heap.alloc_string(format!("obj_{}", i));
        }

        // Measure collection time
        let start = Instant::now();
        let freed = heap.force_collect().unwrap();
        let duration = start.elapsed();

        // Should be reasonable (< 10ms)
        assert!(duration < Duration::from_millis(10));
        assert_eq!(freed, 1000);
    }

    #[test]
    fn test_allocation_performance() {
        let mut heap = GcHeap::new();

        let start = Instant::now();
        for i in 0..10000 {
            let _ = heap.alloc_string(format!("fast_obj_{}", i));
        }
        let duration = start.elapsed();

        // Should allocate fast (< 50ms for 10k objects)
        assert!(duration < Duration::from_millis(50));
    }

    #[test]
    fn test_reactive_allocation_performance() {
        let mut heap = GcHeap::new();

        let start = Instant::now();
        for i in 0..1000 {
            let signal = heap.alloc_reactive_signal(Value::int(i));
            let computed = heap.alloc_reactive_computed(Value::int(0));
            heap.add_reactive_dependency(&computed, &signal).unwrap();
        }
        let duration = start.elapsed();

        // Should be reasonable (< 100ms for 1000 pairs with dependencies)
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
    fn test_cycle_detection_performance() {
        let mut heap = GcHeap::new();

        // Create objects that might have cycles
        let mut objects = Vec::new();
        for i in 0..500 {
            objects.push(heap.alloc_string(format!("cycle_test_{}", i)));
        }

        let start = Instant::now();
        let freed = heap.force_collect().unwrap();
        let duration = start.elapsed();

        // Should detect cycles quickly
        assert!(duration < Duration::from_millis(5));
        assert_eq!(freed, 500);
    }

    #[test]
    fn test_memory_throughput() {
        let mut heap = GcHeap::with_threshold(100);

        let start = Instant::now();

        // Simulate high allocation rate
        for batch in 0..10 {
            for i in 0..100 {
                let _ = heap.alloc_list(vec![Value::int(batch * 100 + i)]);
            }
            // Automatic collection may trigger
        }

        let duration = start.elapsed();

        // Should handle throughput well
        assert!(duration < Duration::from_millis(200));
    }

    #[test]
    fn test_dependency_management_performance() {
        let mut heap = GcHeap::new();

        // Create many reactive objects
        let mut signals = Vec::new();
        let mut computeds = Vec::new();

        for i in 0..200 {
            signals.push(heap.alloc_reactive_signal(Value::int(i)));
            computeds.push(heap.alloc_reactive_computed(Value::int(0)));
        }

        // Add dependencies
        let start = Instant::now();
        for i in 0..199 {
            heap.add_reactive_dependency(&computeds[i], &signals[i]).unwrap();
            heap.add_reactive_dependency(&signals[i], &computeds[i + 1]).unwrap();
        }
        let dep_duration = start.elapsed();

        // Should be fast (< 20ms)
        assert!(dep_duration < Duration::from_millis(20));

        // Test dependency removal performance
        let start = Instant::now();
        for i in 0..199 {
            heap.remove_reactive_dependency(&computeds[i], &signals[i]).unwrap();
            heap.remove_reactive_dependency(&signals[i], &computeds[i + 1]).unwrap();
        }
        let remove_duration = start.elapsed();

        // Should be fast (< 20ms)
        assert!(remove_duration < Duration::from_millis(20));
    }

    #[test]
    fn test_gc_scalability() {
        // Test with increasing heap sizes
        for size in [100, 500, 1000, 2000] {
            let mut heap = GcHeap::new();

            // Allocate objects
            for i in 0..size {
                let _ = heap.alloc_string(format!("scale_{}", i));
            }

            let start = Instant::now();
            let freed = heap.force_collect().unwrap();
            let duration = start.elapsed();

            // Collection time should scale reasonably (not exponentially)
            // For size N, time should be roughly O(N) or better
            let max_expected = Duration::from_millis((size / 50) as u64);
            assert!(duration < max_expected, "GC too slow for size {}: {:?}", size, duration);
            assert_eq!(freed, size);
        }
    }

    #[test]
    fn test_memory_fragmentation_impact() {
        let mut heap = GcHeap::new();

        // Create fragmentation pattern
        let mut objects = Vec::new();

        for i in 0..1000 {
            objects.push(heap.alloc_string(format!("frag_{}", i)));
        }

        // Free every other object
        for i in (0..1000).step_by(2) {
            objects[i] = heap.alloc_string("replacement".to_string());
        }

        // Measure collection performance
        let start = Instant::now();
        let freed = heap.force_collect().unwrap();
        let duration = start.elapsed();

        // Should handle fragmentation well
        assert!(duration < Duration::from_millis(20));
        assert!(freed > 0);
    }

    #[test]
    fn test_concurrent_collection_simulation() {
        let mut heap = GcHeap::new();

        // Simulate concurrent allocations during collection
        let mut objects = Vec::new();

        for phase in 0..5 {
            // Allocate batch
            for i in 0..200 {
                objects.push(heap.alloc_list(vec![Value::int(phase * 200 + i)]));
            }

            // Force collection (simulating concurrent GC)
            let start = Instant::now();
            let freed = heap.force_collect().unwrap();
            let duration = start.elapsed();

            assert!(duration < Duration::from_millis(10));
            assert_eq!(freed, 200);
        }
    }

    #[test]
    fn test_peak_memory_tracking_accuracy() {
        let mut heap = GcHeap::new();

        let initial_peak = heap.peak_heap_size();

        // Allocate increasing amounts
        for size in [10, 50, 100, 500, 1000] {
            let large_obj = heap.alloc_string("x".repeat(size));

            // Peak should be at least as large as current allocation
            assert!(heap.peak_heap_size() >= heap.heap_size());

            // Don't drop yet
            std::mem::forget(large_obj);
        }

        // Force collection
        heap.force_collect().unwrap();

        // Peak should be preserved
        assert!(heap.peak_heap_size() > initial_peak);
    }

    #[test]
    fn test_threshold_based_collection_efficiency() {
        // Test different threshold settings
        for threshold in [10, 50, 100, 500] {
            let mut heap = GcHeap::with_threshold(threshold);

            let mut total_collections = 0;
            let mut total_allocations = 0;

            // Allocate until we trigger collections
            for i in 0..(threshold * 3) {
                let _ = heap.alloc_string(format!("thresh_{}", i));
                total_allocations += 1;

                if heap.object_count() == 0 {
                    total_collections += 1;
                }
            }

            // Should have triggered collections
            assert!(total_collections > 0);

            // Final collection
            let freed = heap.force_collect().unwrap();
            assert_eq!(freed, heap.object_count());
        }
    }

    #[test]
    fn test_memory_reclamation_efficiency() {
        let mut heap = GcHeap::new();

        // Allocate and free repeatedly
        let iterations = 100;
        let objects_per_iteration = 50;

        let start = Instant::now();

        for iter in 0..iterations {
            let mut temp_objects = Vec::new();

            for i in 0..objects_per_iteration {
                temp_objects.push(heap.alloc_string(format!("temp_{}_{}", iter, i)));
            }

            // Drop all objects in this iteration
            drop(temp_objects);

            // Force collection
            heap.force_collect().unwrap();
        }

        let duration = start.elapsed();

        // Should handle repeated allocation/deallocation efficiently
        assert!(duration < Duration::from_millis(500));

        // Should have freed everything
        assert_eq!(heap.object_count(), 0);

        let stats = heap.statistics();
        assert_eq!(stats.freed_total, iterations * objects_per_iteration);
    }

    #[test]
    fn test_large_object_allocation_performance() {
        let mut heap = GcHeap::new();

        let start = Instant::now();

        // Allocate large objects
        for size in [1000, 5000, 10000, 50000] {
            let _ = heap.alloc_string("x".repeat(size));
        }

        let duration = start.elapsed();

        // Should handle large objects reasonably
        assert!(duration < Duration::from_millis(100));

        // Clean up
        heap.force_collect().unwrap();
    }
}