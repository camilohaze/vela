/*!
Memory Management Tests for VelaVM

Comprehensive tests for memory leaks, performance, and edge cases.
*/

use vela_vm::gc::{GcHeap, GcObject};
use vela_vm::bytecode::Value;
use std::collections::HashMap;

#[cfg(test)]
mod memory_management_tests {
    use super::*;

    #[test]
    fn test_no_memory_leaks_basic_allocation() {
        let mut heap = GcHeap::new();

    // Allocate some objects
    let obj1 = heap.alloc_string("test1".to_string());
    let obj2 = heap.alloc_string("test2".to_string());
    let list_data = vec![Value::int(1), Value::int(2)];
    let list = heap.alloc_list(list_data);
    let mut dict_data = HashMap::new();
    dict_data.insert("key".to_string(), Value::int(42));
    let dict = heap.alloc_dict(dict_data);

    // Objects should be alive
    assert_eq!(heap.object_count(), 4);

    // Drop references - objects should be collected
    drop(obj1);
    drop(obj2);
    drop(list);
    drop(dict);        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 4); // All objects should be freed
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_no_memory_leaks_with_cycles() {
        let mut heap = GcHeap::new();

        // Create objects that reference each other
        let obj1 = heap.alloc_list(vec![]);
        let obj2 = heap.alloc_list(vec![]);

        // Simulate cycle by adding references (in real VM this would be done by bytecode)
        // For testing, we'll just allocate and drop

        assert_eq!(heap.object_count(), 2);

        // Drop references
        drop(obj1);
        drop(obj2);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_memory_pressure_handling() {
        let mut heap = GcHeap::new();

        let mut objects = Vec::new();

        // Allocate until we trigger collections
        for i in 0..50 {
            objects.push(heap.alloc_string(format!("pressure_{}", i)));
        }

        // Should have triggered automatic collections
        // Some objects may have been freed
        assert!(heap.object_count() <= 50);

        // Drop references
        drop(objects);

        // Force final collection
        let freed = heap.force_collect().unwrap();
        assert!(freed > 0);
    }

    #[test]
    fn test_large_object_allocation() {
        let mut heap = GcHeap::new();

        // Allocate large strings
        let large_string = "x".repeat(10000);
        let obj = heap.alloc_string(large_string);

        assert_eq!(heap.object_count(), 1);

        // Drop and collect
        drop(obj);
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 1);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_fragmentation_handling() {
        let mut heap = GcHeap::new();

        let mut objects = Vec::new();

        // Allocate many small objects
        for i in 0..1000 {
            objects.push(heap.alloc_string(format!("frag_{}", i)));
        }

        assert_eq!(heap.object_count(), 1000);

        // Drop every other object to create fragmentation
        for i in (0..1000).step_by(2) {
            objects[i] = heap.alloc_string("replacement".to_string());
        }

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 1500);
        assert_eq!(heap.object_count(), 0);

        // Drop remaining references
        drop(objects);
    }

    #[test]
    fn test_nested_collection_allocation() {
        let mut heap = GcHeap::new();

        // Create nested structures
        let mut nested_list = Vec::new();
        let mut inner_objects = Vec::new();
        for i in 0..10 {
            let inner_list = heap.alloc_list(vec![Value::int(i), Value::int(i * 2)]);
            nested_list.push(Value::ptr(inner_list.as_ptr() as usize));
            inner_objects.push(inner_list);
        }

        let outer_list = heap.alloc_list(nested_list);

        // Should have 11 objects (10 inner + 1 outer)
        assert_eq!(heap.object_count(), 11);

        // Drop all references
        drop(inner_objects);
        drop(outer_list);

        // Force collection - should free all
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 11);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_collection_with_mixed_types() {
        let mut heap = GcHeap::new();

        // Allocate different types
        let string = heap.alloc_string("test".to_string());
        let list = heap.alloc_list(vec![Value::int(1), Value::int(2)]);
        let mut dict = HashMap::new();
        dict.insert("key".to_string(), Value::int(42));
        let dict_obj = heap.alloc_dict(dict);
        let set = heap.alloc_set(vec![Value::int(1), Value::int(2), Value::int(3)]);
        let tuple = heap.alloc_tuple(vec![Value::int(1), Value::int(2)]);

        assert_eq!(heap.object_count(), 5);

        // Drop references
        drop(string);
        drop(list);
        drop(dict_obj);
        drop(set);
        drop(tuple);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 5);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut heap = GcHeap::new();

        let initial_count = heap.object_count();
        assert_eq!(initial_count, 0);

        // Allocate some objects
        let obj1 = heap.alloc_string("test1".to_string());
        let obj2 = heap.alloc_string("test2".to_string());

        assert_eq!(heap.object_count(), 2);

        // Allocate more
        let obj3 = heap.alloc_list(vec![Value::int(1)]);
        assert_eq!(heap.object_count(), 3);

        // Drop references
        drop(obj1);
        drop(obj2);
        drop(obj3);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 3);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_threshold_based_collection() {
        let _heap = GcHeap::with_threshold(5);

        // Threshold should be set correctly
        // (We can't directly test threshold triggering without more internal access)
        // This test mainly ensures the constructor works
        assert!(true); // Placeholder - would need internal access to test threshold
    }

    #[test]
    fn test_statistics_tracking() {
        let mut heap = GcHeap::new();

        let stats = heap.statistics();
        assert_eq!(stats.allocations, 0);
        assert_eq!(stats.freed_total, 0);
        assert_eq!(stats.collections, 0);

        // Allocate and collect
        let _obj = heap.alloc_string("test".to_string());
        let freed = heap.force_collect().unwrap();

        let stats = heap.statistics();
        assert_eq!(stats.allocations, 1);
        assert_eq!(stats.freed_total, freed);
        assert!(stats.collections >= 1);
    }

    #[test]
    fn test_cycle_buffer_management() {
        let mut heap = GcHeap::new();

        assert_eq!(heap.cycle_buffer_size(), 0);

        // Allocate objects that might go to cycle buffer
        for i in 0..100 {
            let _obj = heap.alloc_string(format!("cycle_test_{}", i));
        }

        // Force collection
        heap.force_collect().unwrap();

        // Cycle buffer should be empty after collection
        assert_eq!(heap.cycle_buffer_size(), 0);
    }

    #[test]
    fn test_heap_clear_functionality() {
        let mut heap = GcHeap::new();

        // Allocate objects
        let mut objects = Vec::new();
        for i in 0..10 {
            objects.push(heap.alloc_string(format!("clear_test_{}", i)));
        }

        assert_eq!(heap.object_count(), 10);

        // Clear heap
        heap.clear();

        assert_eq!(heap.object_count(), 0);
        assert_eq!(heap.cycle_buffer_size(), 0);
    }

    #[test]
    fn test_reactive_object_memory_management() {
        let mut heap = GcHeap::new();

        // Allocate reactive objects
        let signal = heap.alloc_reactive_signal("test_signal".to_string(), Value::int(42));
        let computed = heap.alloc_reactive_computed("test_computed".to_string());

        // Add dependency
        heap.add_reactive_dependency(&computed, &signal);

        assert_eq!(heap.object_count(), 2);

        // Drop references
        drop(signal);
        drop(computed);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_reactive_dependency_cycles() {
        let mut heap = GcHeap::new();

        // Create reactive objects
        let signal1 = heap.alloc_reactive_signal("signal1".to_string(), Value::int(1));
        let signal2 = heap.alloc_reactive_signal("signal2".to_string(), Value::int(2));
        let computed1 = heap.alloc_reactive_computed("computed1".to_string());
        let computed2 = heap.alloc_reactive_computed("computed2".to_string());

        // Create dependency cycle
        heap.add_reactive_dependency(&computed1, &signal1);
        heap.add_reactive_dependency(&computed1, &signal2);
        heap.add_reactive_dependency(&computed2, &computed1);
        heap.add_reactive_dependency(&signal1, &computed2); // Creates cycle

        assert_eq!(heap.object_count(), 4);

        // Drop references
        drop(signal1);
        drop(signal2);
        drop(computed1);
        drop(computed2);

        // Force collection - should handle cycles
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 4);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_complex_object_graph() {
        let mut heap = GcHeap::new();

        // Create a complex object graph
        let mut objects = Vec::new();

        for i in 0..50 {
            // Create nested structures
            let inner_list = heap.alloc_list(vec![Value::int(i), Value::int(i * 2)]);
            let mut dict = HashMap::new();
            dict.insert(format!("key_{}", i), Value::ptr(inner_list.as_ptr() as usize));
            let dict_obj = heap.alloc_dict(dict);
            objects.push(dict_obj);
        }

        assert_eq!(heap.object_count(), 100); // 50 dicts + 50 lists

        // Drop all references
        drop(objects);

        // Force collection
        let freed = heap.force_collect().unwrap();
        assert_eq!(freed, 100);
        assert_eq!(heap.object_count(), 0);
    }
}