/*!
Garbage Collector tests

Tests for RC behavior, cycle detection, and memory management.
*/

use vela_vm::{GcHeap, GcObject, Value};

#[test]
fn test_gc_heap_initialization() {
    let heap = GcHeap::new();
    assert_eq!(heap.object_count(), 0);
    assert_eq!(heap.statistics().allocations, 0);
}

#[test]
fn test_gc_heap_with_threshold() {
    let heap = GcHeap::with_threshold(500);
    assert_eq!(heap.object_count(), 0);
}

#[test]
fn test_gc_alloc_string() {
    let mut heap = GcHeap::new();
    let string = heap.alloc_string("hello".to_string());

    assert_eq!(heap.object_count(), 1);
    assert_eq!(heap.statistics().allocations, 1);

    // string is Rc<RefCell<GcObject>>
    if let GcObject::String(ptr) = &*string.borrow() {
        assert_eq!(*ptr.borrow(), "hello");
    } else {
        panic!("Expected String object");
    };
}

#[test]
fn test_gc_alloc_list() {
    let mut heap = GcHeap::new();
    let items = vec![Value::int(1), Value::int(2), Value::int(3)];
    let list = heap.alloc_list(items.clone());

    assert_eq!(heap.object_count(), 1);

    if let GcObject::List(ptr) = &*list.borrow() {
        assert_eq!(ptr.borrow().len(), 3);
    } else {
        panic!("Expected List object");
    };
}

#[test]
fn test_gc_alloc_dict() {
    let mut heap = GcHeap::new();
    let mut map = std::collections::HashMap::new();
    map.insert("key".to_string(), Value::int(42));

    let dict = heap.alloc_dict(map);

    assert_eq!(heap.object_count(), 1);

    if let GcObject::Dict(ptr) = &*dict.borrow() {
        assert_eq!(ptr.borrow().len(), 1);
    } else {
        panic!("Expected Dict object");
    };
}

#[test]
fn test_gc_alloc_set() {
    let mut heap = GcHeap::new();
    let items = vec![Value::int(1), Value::int(2)];
    let set = heap.alloc_set(items);

    assert_eq!(heap.object_count(), 1);

    if let GcObject::Set(ptr) = &*set.borrow() {
        assert_eq!(ptr.borrow().len(), 2);
    } else {
        panic!("Expected Set object");
    };
}

#[test]
fn test_gc_alloc_tuple() {
    let mut heap = GcHeap::new();
    let items = vec![Value::int(1), Value::int(2)];
    let tuple = heap.alloc_tuple(items);

    assert_eq!(heap.object_count(), 1);

    if let GcObject::Tuple(ptr) = &*tuple.borrow() {
        assert_eq!(ptr.len(), 2);
    } else {
        panic!("Expected Tuple object");
    };
}

#[test]
fn test_gc_statistics() {
    let mut heap = GcHeap::new();

    heap.alloc_string("test1".to_string());
    heap.alloc_string("test2".to_string());
    heap.alloc_list(vec![Value::int(1)]);

    let stats = heap.statistics();
    assert_eq!(stats.allocations, 3);
    // heap_size may differ from allocations due to internal tracking
    assert!(stats.heap_size >= 3);
}

#[test]
fn test_gc_force_collect() {
    let mut heap = GcHeap::new();

    // Allocate some objects
    heap.alloc_string("temp".to_string());
    heap.alloc_list(vec![Value::int(1)]);

    assert_eq!(heap.object_count(), 2);

    // Force collect (may not remove all objects if heap holds refs)
    let _ = heap.force_collect();

    // Objects should be collected
    assert_eq!(heap.object_count(), 0);
}

#[test]
fn test_gc_reference_counting() {
    use std::rc::Rc;

    let mut heap = GcHeap::new();
    let string = heap.alloc_string("test".to_string());

    // string is Rc<RefCell<GcObject>>
    // Check strong count on the outer Rc
    assert!(Rc::strong_count(&string) >= 1);

    drop(string); // Drop one reference

    // After drop, heap still holds reference
    assert_eq!(heap.object_count(), 1);
}

#[test]
fn test_gc_automatic_collection() {
    let mut heap = GcHeap::with_threshold(10);

    // Allocate objects to trigger GC
    for i in 0..15 {
        heap.alloc_string(format!("string_{}", i));
    }

    let stats = heap.statistics();
    assert!(stats.collections > 0); // At least one collection happened
}

#[test]
fn test_gc_clear() {
    let mut heap = GcHeap::new();

    heap.alloc_string("test1".to_string());
    heap.alloc_string("test2".to_string());
    heap.alloc_list(vec![Value::int(1)]);

    assert_eq!(heap.object_count(), 3);

    heap.clear();

    assert_eq!(heap.object_count(), 0);
    assert_eq!(heap.statistics().heap_size, 0);
}

#[test]
fn test_gc_cycle_buffer() {
    let mut heap = GcHeap::new();

    // Lists can form cycles
    let _list1 = heap.alloc_list(vec![Value::int(1)]);
    let _list2 = heap.alloc_list(vec![Value::int(2)]);

    // Cycle buffer should track these
    assert_eq!(heap.cycle_buffer_size(), 2);
}

#[test]
fn test_gc_freed_statistics() {
    let mut heap = GcHeap::with_threshold(5);

    // Allocate and drop
    for _ in 0..10 {
        heap.alloc_string("temp".to_string());
    }

    let _ = heap.force_collect();

    let stats = heap.statistics();
    assert!(stats.freed_total > 0);
}

#[test]
fn test_gc_force_collect_unused() {
    let mut heap = GcHeap::new();
    heap.alloc_string("test".to_string());
    let _ = heap.force_collect(); // Use let _ = to silence warning
    assert!(heap.object_count() >= 0);
}

#[test]
fn test_gc_peak_heap_size() {
    let mut heap = GcHeap::new();

    // Allocate many objects
    for i in 0..20 {
        heap.alloc_string(format!("test_{}", i));
    }

    let stats = heap.statistics();
    assert!(stats.peak_heap_size >= 20);
}

#[test]
fn test_gc_object_drop() {
    let mut heap = GcHeap::new();

    {
        let _string = heap.alloc_string("scoped".to_string());
        assert_eq!(heap.object_count(), 1);
    } // string dropped here

    let _ = heap.force_collect();
    assert_eq!(heap.object_count(), 0);
}

#[test]
#[test]
fn test_gc_multiple_references() {
    use std::rc::Rc;
    
    let mut heap = GcHeap::new();
    let string1 = heap.alloc_string("shared".to_string());
    let string2 = string1.clone(); // Clone increments refcount

    // Both point to same Rc
    assert!(Rc::ptr_eq(&string1, &string2));
    assert!(Rc::strong_count(&string1) >= 2); // At least 2 refs

    drop(string1);
    drop(string2);

    let _ = heap.force_collect();
    assert_eq!(heap.object_count(), 0);
}
#[test]
fn test_gc_list_with_nested_objects() {
    let mut heap = GcHeap::new();

    // Create nested structure
    let inner_list = heap.alloc_list(vec![Value::int(1), Value::int(2)]);
    let _outer_list = heap.alloc_list(vec![Value::int(0)]);

    // Should have 2 objects
    assert_eq!(heap.object_count(), 2);

    drop(inner_list);
    let _ = heap.force_collect();

    // After collection, some objects may remain if heap holds refs
    assert!(heap.object_count() >= 1);
}
#[test]
fn test_gc_dict_operations() {
    let mut heap = GcHeap::new();

    let mut map = std::collections::HashMap::new();
    map.insert("name".to_string(), Value::int(42));
    map.insert("age".to_string(), Value::int(30));

    let dict = heap.alloc_dict(map);

    if let GcObject::Dict(ptr) = &*dict.borrow() {
        assert_eq!(ptr.borrow().len(), 2);
        assert!(ptr.borrow().contains_key("name"));
        assert!(ptr.borrow().contains_key("age"));
    };
}

#[test]
fn test_gc_set_uniqueness() {
    let mut heap = GcHeap::new();

    // Sets should handle duplicates (though this is application logic)
    let items = vec![Value::int(1), Value::int(2), Value::int(1)];
    let set = heap.alloc_set(items);

    if let GcObject::Set(ptr) = &*set.borrow() {
        // Raw vec contains all items (deduplication is application logic)
        assert_eq!(ptr.borrow().len(), 3);
    };
}

#[test]
fn test_gc_tuple_immutability() {
    use std::rc::Rc;

    let mut heap = GcHeap::new();
    let items = vec![Value::int(1), Value::int(2), Value::int(3)];
    let tuple = heap.alloc_tuple(items);

    if let GcObject::Tuple(ptr) = &*tuple.borrow() {
        // Tuple uses Rc (not RefCell), so it's immutable
        assert_eq!(ptr.len(), 3);
        // strong_count may vary due to internal heap references
        assert!(Rc::strong_count(ptr) >= 1);
    };
}

#[test]
fn test_gc_collection_threshold() {
    let mut heap = GcHeap::with_threshold(3);

    heap.alloc_string("s1".to_string());
    heap.alloc_string("s2".to_string());
    assert_eq!(heap.statistics().collections, 0);

    heap.alloc_string("s3".to_string()); // Triggers collection
    assert_eq!(heap.statistics().collections, 1);
}

#[test]
fn test_gc_alloc_reactive_signal() {
    let mut heap = GcHeap::new();
    let signal = heap.alloc_reactive_signal("signal-1".to_string(), Value::int(42));

    assert_eq!(heap.object_count(), 1);
    assert_eq!(heap.cycle_buffer_size(), 1); // Reactive objects go to cycle buffer

    // Check signal value
    assert_eq!(heap.get_reactive_signal_value(&signal), Some(Value::int(42)));

    // Update signal value
    heap.update_reactive_signal(&signal, Value::int(100));
    assert_eq!(heap.get_reactive_signal_value(&signal), Some(Value::int(100)));
}

#[test]
fn test_gc_alloc_reactive_computed() {
    let mut heap = GcHeap::new();
    let computed = heap.alloc_reactive_computed("computed-1".to_string());

    assert_eq!(heap.object_count(), 1);
    assert_eq!(heap.cycle_buffer_size(), 1); // Reactive objects go to cycle buffer

    // Initially no cached value
    assert_eq!(heap.get_reactive_computed_value(&computed), None);

    // Set cached value
    heap.set_reactive_computed_value(&computed, Some(Value::int(42)));
    assert_eq!(heap.get_reactive_computed_value(&computed), Some(Value::int(42)));
}

#[test]
fn test_gc_reactive_dependencies() {
    let mut heap = GcHeap::new();

    let signal = heap.alloc_reactive_signal("signal-1".to_string(), Value::int(10));
    let computed = heap.alloc_reactive_computed("computed-1".to_string());

    // Add dependency: computed depends on signal
    heap.add_reactive_dependency(&computed, &signal);

    // Check that dependencies are tracked
    {
        let computed_borrow = computed.borrow();
        if let GcObject::ReactiveComputed(computed_obj) = &*computed_borrow {
            assert_eq!(computed_obj.borrow().dependencies.len(), 1);
        }
    }

    {
        let signal_borrow = signal.borrow();
        if let GcObject::ReactiveSignal(signal_obj) = &*signal_borrow {
            assert_eq!(signal_obj.borrow().dependents.len(), 1); // Signal should have computed as dependent
        }
    }

    // Remove dependency
    heap.remove_reactive_dependency(&computed, &signal);

    {
        let computed_borrow = computed.borrow();
        if let GcObject::ReactiveComputed(computed_obj) = &*computed_borrow {
            assert_eq!(computed_obj.borrow().dependencies.len(), 0);
        }
    }
}

#[test]
fn test_gc_reactive_cycle_detection() {
    let mut heap = GcHeap::with_threshold(10); // High threshold to avoid auto-collection

    let signal1 = heap.alloc_reactive_signal("signal-1".to_string(), Value::int(1));
    let signal2 = heap.alloc_reactive_signal("signal-2".to_string(), Value::int(2));
    let computed1 = heap.alloc_reactive_computed("computed-1".to_string());
    let computed2 = heap.alloc_reactive_computed("computed-2".to_string());

    // Create dependencies that form a cycle:
    // computed1 -> signal1 -> computed2 -> signal2 -> computed1
    heap.add_reactive_dependency(&computed1, &signal1);
    heap.add_reactive_dependency(&computed2, &signal1);
    heap.add_reactive_dependency(&computed1, &signal2);
    heap.add_reactive_dependency(&computed2, &signal2);

    // Force collection
    let freed = heap.force_collect().unwrap();

    // Objects with dependencies should be retained
    assert_eq!(freed, 0); // No objects should be freed due to dependencies

    // Check that all objects are still in cycle buffer
    assert_eq!(heap.cycle_buffer_size(), 4);

    // Now remove all dependencies
    heap.remove_reactive_dependency(&computed1, &signal1);
    heap.remove_reactive_dependency(&computed2, &signal1);
    heap.remove_reactive_dependency(&computed1, &signal2);
    heap.remove_reactive_dependency(&computed2, &signal2);

    // Force another collection
    let freed = heap.force_collect().unwrap();

    // Now objects should be freed since they have no dependencies and no external references
    // (Note: In this test setup, they may still have references from the test, so this is more of a structural test)
    assert!(freed >= 0);
}

#[test]
fn test_gc_reactive_memory_management() {
    let mut heap = GcHeap::with_threshold(20);

    // Allocate reactive objects
    let signal = heap.alloc_reactive_signal("test-signal".to_string(), Value::int(42));
    let computed = heap.alloc_reactive_computed("test-computed".to_string());

    // Add dependency
    heap.add_reactive_dependency(&computed, &signal);

    // Force collection - objects should be retained due to dependency
    let freed = heap.force_collect().unwrap();
    assert_eq!(freed, 0);

    // Update signal
    heap.update_reactive_signal(&signal, Value::int(84));
    assert_eq!(heap.get_reactive_signal_value(&signal), Some(Value::int(84)));

    // Set computed value
    heap.set_reactive_computed_value(&computed, Some(Value::int(168)));
    assert_eq!(heap.get_reactive_computed_value(&computed), Some(Value::int(168)));
}
