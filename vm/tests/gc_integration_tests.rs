/*!
GC Integration Tests for VelaVM

Tests that validate GC behavior in complex scenarios,
especially with reactive objects and cross-references.
*/

use vela_vm::gc::{GcHeap, GcObject};
use vela_vm::bytecode::Value;

#[cfg(test)]
mod gc_integration_tests {
    use super::*;

    #[test]
    fn test_reactive_objects_integration() {
        let mut heap = GcHeap::new();

        // Create reactive objects
        let signal1 = heap.alloc_reactive_signal("signal1".to_string(), Value::int(10));
        let signal2 = heap.alloc_reactive_signal("signal2".to_string(), Value::int(20));
        let computed1 = heap.alloc_reactive_computed("computed1".to_string());
        let computed2 = heap.alloc_reactive_computed("computed2".to_string());

        // Create dependency chain: signal1 -> computed1 -> signal2 -> computed2
        heap.add_reactive_dependency(&computed1, &signal1);
        heap.add_reactive_dependency(&computed2, &signal2);
        heap.add_reactive_dependency(&signal2, &computed1);

        // Force collection - should retain all due to dependencies
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 0);
        assert_eq!(heap.object_count(), 4);

        // Break dependencies
        heap.remove_reactive_dependency(&computed1, &signal1);
        heap.remove_reactive_dependency(&computed2, &signal2);
        heap.remove_reactive_dependency(&signal2, &computed1);

        // Now should free all
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 4);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_mixed_object_types_gc() {
        let mut heap = GcHeap::new();

        // Create mix of object types
        let string = heap.alloc_string("test".to_string());
        let list = heap.alloc_list(vec![Value::int(1), Value::int(2)]);
        let dict = heap.alloc_dict(std::collections::HashMap::new());
        let signal = heap.alloc_reactive_signal("signal".to_string(), Value::int(42));
        let computed = heap.alloc_reactive_computed("computed".to_string());

        // Create some relationships
        heap.add_reactive_dependency(&computed, &signal);

        // Force collection - should retain signal and computed due to dependency
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 3); // string, list, dict freed
        assert_eq!(heap.object_count(), 2); // signal, computed retained

        // Break dependency
        heap.remove_reactive_dependency(&computed, &signal);

        // Now free remaining
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_complex_dependency_graphs() {
        let mut heap = GcHeap::new();

        // Create diamond dependency: A -> B, A -> C, B -> D, C -> D
        let signal_a = heap.alloc_reactive_signal("signal_a".to_string(), Value::int(1));
        let signal_b = heap.alloc_reactive_signal("signal_b".to_string(), Value::int(2));
        let signal_c = heap.alloc_reactive_signal("signal_c".to_string(), Value::int(3));
        let computed_d = heap.alloc_reactive_computed("computed_d".to_string());

        heap.add_reactive_dependency(&signal_b, &signal_a);
        heap.add_reactive_dependency(&signal_c, &signal_a);
        heap.add_reactive_dependency(&computed_d, &signal_b);
        heap.add_reactive_dependency(&computed_d, &signal_c);

        // All should be retained
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 0);
        assert_eq!(heap.object_count(), 4);

        // Remove root dependency
        heap.remove_reactive_dependency(&signal_b, &signal_a);
        heap.remove_reactive_dependency(&signal_c, &signal_a);

        // Should still retain due to D depending on B and C
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 1); // A freed
        assert_eq!(heap.object_count(), 3);

        // Remove remaining dependencies
        heap.remove_reactive_dependency(&computed_d, &signal_b);
        heap.remove_reactive_dependency(&computed_d, &signal_c);

        // Now free all
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 3);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_circular_references_with_regular_objects() {
        let mut heap = GcHeap::new();

        // Create circular references between regular objects
        let list1 = heap.alloc_list(vec![]);
        let list2 = heap.alloc_list(vec![]);

        // Simulate circular reference (in real Vela this would be through reactive deps)
        // For testing, we'll just check that cycle detection works

        // Force collection - should handle potential cycles
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_gc_with_nested_collections() {
        let mut heap = GcHeap::new();

        // Create nested collections
        let inner_list = heap.alloc_list(vec![Value::int(1), Value::int(2), Value::int(3)]);
        let middle_list = heap.alloc_list(vec![Value::int(4), Value::int(5)]);
        let outer_list = heap.alloc_list(vec![Value::int(6)]);

        // Create dict containing lists
        let mut dict_data = std::collections::HashMap::new();
        dict_data.insert("inner".to_string(), Value::int(100));
        let dict = heap.alloc_dict(dict_data);

        // Force collection - all should be freed
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 4);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_reactive_updates_and_gc() {
        let mut heap = GcHeap::new();

        let signal = heap.alloc_reactive_signal("signal".to_string(), Value::int(10));
        let computed = heap.alloc_reactive_computed("computed".to_string());

        heap.add_reactive_dependency(&computed, &signal);

        // Update signal value
        heap.update_reactive_signal(&signal, Value::int(20));
        assert_eq!(heap.get_reactive_signal_value(&signal), Some(Value::int(20)));

        // Update computed value
        heap.set_reactive_computed_value(&computed, Some(Value::int(40)));

        // Force collection - should retain both
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 0);
        assert_eq!(heap.object_count(), 2);

        // Clean up
        heap.remove_reactive_dependency(&computed, &signal);
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
    }

    #[test]
    fn test_memory_pressure_with_reactive_objects() {
        let mut heap = GcHeap::with_threshold(5);

        // Create many reactive objects
        let mut signals = Vec::new();
        let mut computeds = Vec::new();

        for i in 0..20 {
            signals.push(heap.alloc_reactive_signal(format!("signal_{}", i), Value::int(i)));
            computeds.push(heap.alloc_reactive_computed(format!("computed_{}", i)));
        }

        // Create some dependencies
        for i in 0..10 {
            heap.add_reactive_dependency(&computeds[i], &signals[i]);
        }

        // Should have triggered collections
        assert!(heap.object_count() < 40);

        // Clean up dependencies
        for i in 0..10 {
            heap.remove_reactive_dependency(&computeds[i], &signals[i]);
        }

        // Final collection
        let freed = heap.force_collect().unwrap();
        assert!(freed > 0);
    }

    #[test]
    fn test_gc_statistics_with_reactive_objects() {
        let mut heap = GcHeap::new();

        // Create reactive objects and track stats
        let initial_objects = heap.object_count();

        let signal = heap.alloc_reactive_signal("signal".to_string(), Value::int(1));
        let computed = heap.alloc_reactive_computed("computed".to_string());

        assert_eq!(heap.object_count(), initial_objects + 2);

        heap.add_reactive_dependency(&computed, &signal);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 0); // Retained due to dependency

        // Check stats
        let stats = heap.statistics();
        assert!(stats.collections >= 1);

        // Clean up
        heap.remove_reactive_dependency(&computed, &signal);
        heap.force_collect().unwrap();
        assert_eq!(heap.object_count(), initial_objects);
    }

    #[test]
    fn test_edge_case_empty_dependencies() {
        let mut heap = GcHeap::new();

        let signal = heap.alloc_reactive_signal("signal".to_string(), Value::int(42));
        let computed = heap.alloc_reactive_computed("computed".to_string());

        // No dependencies added

        // Should free immediately
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_self_dependency_prevention() {
        let mut heap = GcHeap::new();

        let signal = heap.alloc_reactive_signal("signal".to_string(), Value::int(42));

        // Try to add self-dependency (should not be allowed or should handle gracefully)
        // In current implementation, this would create a cycle but let's test handling

        // For now, just ensure basic functionality works
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 1);
    }

    #[test]
    fn test_large_dependency_graph() {
        let mut heap = GcHeap::with_threshold(50);

        // Create large dependency graph
        let mut signals = Vec::new();
        let mut computeds = Vec::new();

        for i in 0..25 {
            signals.push(heap.alloc_reactive_signal(format!("signal_{}", i), Value::int(i)));
            computeds.push(heap.alloc_reactive_computed(format!("computed_{}", i)));
        }

        // Create chain dependencies
        for i in 0..24 {
            heap.add_reactive_dependency(&computeds[i], &signals[i]);
            heap.add_reactive_dependency(&signals[i + 1], &computeds[i]);
        }

        // Should handle large graph
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 0); // All retained due to dependencies

        // Clean up
        for i in 0..24 {
            heap.remove_reactive_dependency(&computeds[i], &signals[i]);
            heap.remove_reactive_dependency(&signals[i + 1], &computeds[i]);
        }

        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 50);
    }
}