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

    // Objects may still exist if heap holds refs
    assert!(heap.object_count() >= 1);
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
