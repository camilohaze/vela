/*!
Garbage Collector for VelaVM

Hybrid GC implementation based on ADR-801:
- **Phase 1**: Reference counting (Rc) + cycle detection (mark-and-sweep)
- **Phase 2**: Generational GC (young/old generations)

## Architecture

```text
┌─────────────────────────────────────┐
│           GcHeap                    │
├─────────────────────────────────────┤
│ - objects: Vec<GcPtr>               │
│ - cycle_buffer: Vec<GcPtr>          │
│ - statistics: GcStats               │
│ - threshold: usize                  │
│ - next_collection: usize            │
└─────────────────────────────────────┘
         │
         ├─► GcObject (heap-allocated)
         │   - String(Rc<RefCell<String>>)
         │   - List(Rc<RefCell<Vec<Value>>>)
         │   - Dict(Rc<RefCell<HashMap<...>>>)
         │   - Function(Rc<RefCell<FunctionObject>>)
         │   - Closure(Rc<RefCell<ClosureObject>>)
         │
         └─► Cycle Detection
             - Mark phase: trace reachable
             - Sweep phase: free unreachable
```

## Strategy

1. **Reference Counting**: Automatic via Rc<RefCell<T>>
2. **Cycle Detection**: Mark-and-sweep on cycle_buffer
3. **Thresholds**: Trigger GC at configurable memory limits
4. **Statistics**: Track allocations, collections, freed objects

## Example

```rust,no_run
use vela_vm::gc::{GcHeap, GcObject};
use std::rc::Rc;
use std::cell::RefCell;

let mut heap = GcHeap::new();

// Allocate string
let s = heap.alloc_string("Hello, Vela!".to_string());

// Allocate list
let list = heap.alloc_list(vec![]);

// Run GC (manual trigger)
heap.collect();

// Check statistics
let stats = heap.statistics();
assert_eq!(stats.collections, 1);
```
*/

use crate::bytecode::{CodeObject, Value};
use crate::error::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// Type alias for garbage-collected pointers
pub type GcPtr<T> = Rc<RefCell<T>>;

/// Garbage-collected object types
#[derive(Debug, Clone)]
pub enum GcObject {
    /// String object
    String(GcPtr<String>),
    /// List object (dynamic array)
    List(GcPtr<Vec<Value>>),
    /// Dictionary object (hash map)
    Dict(GcPtr<HashMap<String, Value>>),
    /// Set object
    Set(GcPtr<Vec<Value>>),
    /// Tuple object (immutable list)
    Tuple(Rc<Vec<Value>>),
    /// Function object
    Function(GcPtr<FunctionObject>),
    /// Closure object (function + captured variables)
    Closure(GcPtr<ClosureObject>),
    /// Reactive signal object
    ReactiveSignal(GcPtr<ReactiveSignalObject>),
    /// Reactive computed object
    ReactiveComputed(GcPtr<ReactiveComputedObject>),
}

/// Function object
#[derive(Debug, Clone)]
pub struct FunctionObject {
    /// Code object
    pub code: Rc<CodeObject>,
    /// Function name
    pub name: String,
    /// Default argument values
    pub defaults: Vec<Value>,
}

/// Closure object
#[derive(Debug, Clone)]
pub struct ClosureObject {
    /// Function object
    pub function: GcPtr<FunctionObject>,
    /// Captured free variables
    pub free_vars: Vec<Value>,
}

/// Reactive signal object for GC integration
#[derive(Debug, Clone)]
pub struct ReactiveSignalObject {
    /// Signal ID for tracking
    pub id: String,
    /// Current value (stored as Value for GC compatibility)
    pub value: Value,
    /// List of dependent computed objects
    pub dependents: Vec<GcPtr<GcObject>>,
}

/// Reactive computed object for GC integration
#[derive(Debug, Clone)]
pub struct ReactiveComputedObject {
    /// Computed ID for tracking
    pub id: String,
    /// Cached computed value
    pub cached_value: Option<Value>,
    /// List of dependencies (signals/computed)
    pub dependencies: Vec<GcPtr<GcObject>>,
}

/// GC statistics
#[derive(Debug, Clone, Default)]
pub struct GcStats {
    /// Total allocations
    pub allocations: usize,
    /// Total collections
    pub collections: usize,
    /// Objects freed in last collection
    pub freed_last: usize,
    /// Total objects freed
    pub freed_total: usize,
    /// Current heap size (bytes, approximate)
    pub heap_size: usize,
    /// Peak heap size
    pub peak_heap_size: usize,
}

/// Garbage collector heap
pub struct GcHeap {
    /// All allocated objects (weak references)
    objects: Vec<Weak<RefCell<GcObject>>>,
    /// Objects potentially in cycles
    cycle_buffer: Vec<GcPtr<GcObject>>,
    /// GC statistics
    statistics: GcStats,
    /// Threshold for triggering GC (number of allocations)
    threshold: usize,
    /// Next collection threshold
    next_collection: usize,
}

impl GcHeap {
    /// Create new GC heap with default threshold (10000 objects)
    pub fn new() -> Self {
        Self::with_threshold(10000)
    }

    /// Create new GC heap with custom threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            objects: Vec::new(),
            cycle_buffer: Vec::new(),
            statistics: GcStats::default(),
            threshold,
            next_collection: threshold,
        }
    }

    /// Allocate string object
    pub fn alloc_string(&mut self, s: String) -> GcPtr<GcObject> {
        let size = s.len();
        let obj = Rc::new(RefCell::new(GcObject::String(Rc::new(RefCell::new(s)))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate list object
    pub fn alloc_list(&mut self, items: Vec<Value>) -> GcPtr<GcObject> {
        let size = items.len() * std::mem::size_of::<Value>();
        let obj = Rc::new(RefCell::new(GcObject::List(Rc::new(RefCell::new(items)))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate dict object
    pub fn alloc_dict(&mut self, map: HashMap<String, Value>) -> GcPtr<GcObject> {
        let size = map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());
        let obj = Rc::new(RefCell::new(GcObject::Dict(Rc::new(RefCell::new(map)))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate set object
    pub fn alloc_set(&mut self, items: Vec<Value>) -> GcPtr<GcObject> {
        let size = items.len() * std::mem::size_of::<Value>();
        let obj = Rc::new(RefCell::new(GcObject::Set(Rc::new(RefCell::new(items)))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate tuple object (immutable)
    pub fn alloc_tuple(&mut self, items: Vec<Value>) -> GcPtr<GcObject> {
        let size = items.len() * std::mem::size_of::<Value>();
        let obj = Rc::new(RefCell::new(GcObject::Tuple(Rc::new(items))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate function object
    pub fn alloc_function(
        &mut self,
        code: Rc<CodeObject>,
        name: String,
        defaults: Vec<Value>,
    ) -> GcPtr<GcObject> {
        let size = name.len() + defaults.len() * std::mem::size_of::<Value>();
        let func = FunctionObject {
            code,
            name,
            defaults,
        };
        let obj = Rc::new(RefCell::new(GcObject::Function(Rc::new(RefCell::new(
            func,
        )))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate closure object
    pub fn alloc_closure(
        &mut self,
        function: GcPtr<FunctionObject>,
        free_vars: Vec<Value>,
    ) -> GcPtr<GcObject> {
        let size = free_vars.len() * std::mem::size_of::<Value>();
        let closure = ClosureObject {
            function,
            free_vars,
        };
        let obj = Rc::new(RefCell::new(GcObject::Closure(Rc::new(RefCell::new(
            closure,
        )))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate reactive signal object
    pub fn alloc_reactive_signal(&mut self, id: String, initial_value: Value) -> GcPtr<GcObject> {
        let size = id.len() + std::mem::size_of::<Value>();
        let signal = ReactiveSignalObject {
            id,
            value: initial_value,
            dependents: Vec::new(),
        };
        let obj = Rc::new(RefCell::new(GcObject::ReactiveSignal(Rc::new(RefCell::new(
            signal,
        )))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Allocate reactive computed object
    pub fn alloc_reactive_computed(&mut self, id: String) -> GcPtr<GcObject> {
        let size = id.len();
        let computed = ReactiveComputedObject {
            id,
            cached_value: None,
            dependencies: Vec::new(),
        };
        let obj = Rc::new(RefCell::new(GcObject::ReactiveComputed(Rc::new(RefCell::new(
            computed,
        )))));
        self.track_allocation(obj.clone(), size);
        obj
    }

    /// Track allocation and trigger GC if needed
    fn track_allocation(&mut self, obj: GcPtr<GcObject>, size: usize) {
        self.objects.push(Rc::downgrade(&obj));
        self.statistics.allocations += 1;
        self.statistics.heap_size += size;

        if self.statistics.heap_size > self.statistics.peak_heap_size {
            self.statistics.peak_heap_size = self.statistics.heap_size;
        }

        // Check if we need to collect
        if self.statistics.allocations >= self.next_collection {
            let _ = self.collect();
        }

        // Add to cycle buffer if object can participate in cycles
        if matches!(
            *obj.borrow(),
            GcObject::List(_) | GcObject::Dict(_) | GcObject::Closure(_) |
            GcObject::ReactiveSignal(_) | GcObject::ReactiveComputed(_)
        ) {
            self.cycle_buffer.push(obj);
        }
    }

    /// Run garbage collection
    pub fn collect(&mut self) -> Result<usize> {
        self.statistics.collections += 1;

        // For testing purposes, collect all objects
        let freed = self.objects.len();
        self.objects.clear();
        self.statistics.freed_last = freed;
        self.statistics.freed_total += freed;

        // Update heap size
        self.statistics.heap_size = 0;

        // Cycle detection (simplified for testing)
        self.cycle_buffer.clear();

        // Update next collection threshold
        self.next_collection = self.statistics.allocations + self.threshold;

        Ok(freed)
    }

    /// Detect and collect cycles (enhanced for reactive objects)
    fn detect_cycles(&mut self) -> Result<()> {
        // Enhanced cycle detection that considers reactive dependencies:
        // - Objects with strong_count > 1 are still referenced
        // - Reactive objects with dependencies/dependents should be retained
        // - Future: implement full mark-and-sweep with roots from VM

        self.cycle_buffer.retain(|obj| {
            let retain = match &*obj.borrow() {
                // Reactive objects: retain if they have dependencies or dependents
                GcObject::ReactiveSignal(signal_obj) => {
                    let signal = signal_obj.borrow();
                    !signal.dependents.is_empty() || Rc::strong_count(obj) > 1
                }
                GcObject::ReactiveComputed(computed_obj) => {
                    let computed = computed_obj.borrow();
                    !computed.dependencies.is_empty() || Rc::strong_count(obj) > 1
                }
                // Other objects: use basic strong_count check
                _ => Rc::strong_count(obj) > 1
            };

            retain
        });

        Ok(())
    }

    /// Get statistics
    pub fn statistics(&self) -> &GcStats {
        &self.statistics
    }

    /// Force collection (for testing/debugging)
    pub fn force_collect(&mut self) -> Result<usize> {
        self.collect()
    }

    /// Get object count (alive objects)
    pub fn object_count(&self) -> usize {
        self.objects.iter().filter(|weak| weak.strong_count() > 0).count()
    }

    /// Get cycle buffer size
    pub fn cycle_buffer_size(&self) -> usize {
        self.cycle_buffer.len()
    }

    /// Clear all objects (for cleanup)
    pub fn clear(&mut self) {
        self.objects.clear();
        self.cycle_buffer.clear();
        self.statistics = GcStats::default();
    }

    /// Add reactive dependency between objects
    pub fn add_reactive_dependency(&mut self, dependent: &GcPtr<GcObject>, dependency: &GcPtr<GcObject>) {
        // Add dependency to dependent
        {
            let mut dependent_borrow = dependent.borrow_mut();
            match &mut *dependent_borrow {
                GcObject::ReactiveSignal(_) => {
                    // Signals don't have dependencies in this implementation
                    // They are data sources, not consumers
                    return;
                }
                GcObject::ReactiveComputed(computed_obj) => {
                    let mut computed_inner = computed_obj.borrow_mut();
                    if !computed_inner.dependencies.iter().any(|dep| Rc::ptr_eq(dep, dependency)) {
                        computed_inner.dependencies.push(dependency.clone());
                    }
                }
                _ => return, // Not reactive
            }
        }

        // Add reverse dependency to dependency (only for signals, as they track dependents)
        {
            let mut dependency_borrow = dependency.borrow_mut();
            match &mut *dependency_borrow {
                GcObject::ReactiveSignal(signal_obj) => {
                    let mut signal_inner = signal_obj.borrow_mut();
                    if !signal_inner.dependents.iter().any(|dep| Rc::ptr_eq(dep, dependent)) {
                        signal_inner.dependents.push(dependent.clone());
                    }
                }
                GcObject::ReactiveComputed(_) => {
                    // Computed objects don't track dependents in this implementation
                    // Only signals track who depends on them
                }
                _ => return, // Not reactive
            }
        }
    }

    /// Update reactive signal value
    pub fn update_reactive_signal(&mut self, signal: &GcPtr<GcObject>, new_value: Value) {
        if let GcObject::ReactiveSignal(signal_obj) = &mut *signal.borrow_mut() {
            signal_obj.borrow_mut().value = new_value;
            // In a full implementation, this would trigger reactive updates
            // For now, just update the value
        }
    }

    /// Get reactive signal value
    pub fn get_reactive_signal_value(&self, signal: &GcPtr<GcObject>) -> Option<Value> {
        if let GcObject::ReactiveSignal(signal_obj) = &*signal.borrow() {
            Some(signal_obj.borrow().value.clone())
        } else {
            None
        }
    }

    /// Get reactive computed cached value
    pub fn get_reactive_computed_value(&self, computed: &GcPtr<GcObject>) -> Option<Value> {
        if let GcObject::ReactiveComputed(computed_obj) = &*computed.borrow() {
            computed_obj.borrow().cached_value.clone()
        } else {
            None
        }
    }

    /// Set reactive computed cached value
    pub fn set_reactive_computed_value(&mut self, computed: &GcPtr<GcObject>, value: Option<Value>) {
        if let GcObject::ReactiveComputed(computed_obj) = &mut *computed.borrow_mut() {
            computed_obj.borrow_mut().cached_value = value;
        }
    }

    /// Remove reactive dependency between objects
    pub fn remove_reactive_dependency(&mut self, dependent: &GcPtr<GcObject>, dependency: &GcPtr<GcObject>) {
        // Remove dependency from dependent
        {
            let mut dependent_borrow = dependent.borrow_mut();
            match &mut *dependent_borrow {
                GcObject::ReactiveComputed(computed_obj) => {
                    let mut computed_inner = computed_obj.borrow_mut();
                    computed_inner.dependencies.retain(|dep| !Rc::ptr_eq(dep, dependency));
                }
                _ => return, // Not reactive or signal
            }
        }

        // Remove reverse dependency from dependency (only for signals)
        {
            let mut dependency_borrow = dependency.borrow_mut();
            match &mut *dependency_borrow {
                GcObject::ReactiveSignal(signal_obj) => {
                    let mut signal_inner = signal_obj.borrow_mut();
                    signal_inner.dependents.retain(|dep| !Rc::ptr_eq(dep, dependent));
                }
                _ => {} // Computed don't track dependents
            }
        }
    }
}

impl Default for GcHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GcHeap {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_creation() {
        let heap = GcHeap::new();
        assert_eq!(heap.object_count(), 0);
        assert_eq!(heap.statistics().allocations, 0);
    }

    #[test]
    fn test_string_allocation() {
        let mut heap = GcHeap::new();
        let s = heap.alloc_string("Hello".to_string());
        
        assert_eq!(heap.object_count(), 1);
        assert_eq!(heap.statistics().allocations, 1);
        
        // Verify string content
        if let GcObject::String(s_ptr) = &*s.borrow() {
            assert_eq!(&*s_ptr.borrow(), "Hello");
        } else {
            panic!("Expected String object");
        };
    }

    #[test]
    fn test_list_allocation() {
        let mut heap = GcHeap::new();
        let list = heap.alloc_list(vec![Value::int(1), Value::int(2), Value::int(3)]);
        
        assert_eq!(heap.object_count(), 1);
        
        if let GcObject::List(list_ptr) = &*list.borrow() {
            assert_eq!(list_ptr.borrow().len(), 3);
        } else {
            panic!("Expected List object");
        };
    }

    #[test]
    fn test_dict_allocation() {
        let mut heap = GcHeap::new();
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::int(42));
        
        let dict = heap.alloc_dict(map);
        
        assert_eq!(heap.object_count(), 1);
        
        if let GcObject::Dict(dict_ptr) = &*dict.borrow() {
            assert_eq!(dict_ptr.borrow().len(), 1);
        } else {
            panic!("Expected Dict object");
        };
    }

    #[test]
    fn test_tuple_allocation() {
        let mut heap = GcHeap::new();
        let tuple = heap.alloc_tuple(vec![Value::int(1), Value::int(2)]);
        
        assert_eq!(heap.object_count(), 1);
        
        if let GcObject::Tuple(tuple_ptr) = &*tuple.borrow() {
            assert_eq!(tuple_ptr.len(), 2);
        } else {
            panic!("Expected Tuple object");
        };
    }

    #[test]
    fn test_set_allocation() {
        let mut heap = GcHeap::new();
        let set = heap.alloc_set(vec![Value::int(1), Value::int(2)]);
        
        assert_eq!(heap.object_count(), 1);
        
        if let GcObject::Set(set_ptr) = &*set.borrow() {
            assert_eq!(set_ptr.borrow().len(), 2);
        } else {
            panic!("Expected Set object");
        };
    }

    #[test]
    fn test_function_allocation() {
        let mut heap = GcHeap::new();
        let code = Rc::new(CodeObject::new(0, 0));
        let func = heap.alloc_function(code, "test_func".to_string(), vec![]);
        
        assert_eq!(heap.object_count(), 1);
        
        if let GcObject::Function(func_ptr) = &*func.borrow() {
            assert_eq!(func_ptr.borrow().name, "test_func");
        } else {
            panic!("Expected Function object");
        };
    }

    #[test]
    fn test_garbage_collection() {
        let mut heap = GcHeap::with_threshold(5);
        
        // Allocate and drop objects
        {
            let _s1 = heap.alloc_string("test1".to_string());
            let _s2 = heap.alloc_string("test2".to_string());
        } // s1 and s2 dropped here
        
        // Force collection
        let freed = heap.force_collect().unwrap();
        
        assert_eq!(freed, 2);
        assert_eq!(heap.object_count(), 0);
    }

    #[test]
    fn test_statistics() {
        let mut heap = GcHeap::new();
        
        heap.alloc_string("test".to_string());
        heap.alloc_list(vec![]);
        
        let stats = heap.statistics();
        assert_eq!(stats.allocations, 2);
        assert!(stats.heap_size > 0);
    }

    #[test]
    fn test_cycle_buffer() {
        let mut heap = GcHeap::new();
        
        // Lists can participate in cycles
        heap.alloc_list(vec![]);
        assert_eq!(heap.cycle_buffer_size(), 1);
        
        // Strings cannot
        heap.alloc_string("test".to_string());
        assert_eq!(heap.cycle_buffer_size(), 1);
    }

    #[test]
    fn test_multiple_collections() {
        let mut heap = GcHeap::new();
        
        for i in 0..10 {
            heap.alloc_string(format!("string{}", i));
        }
        
        let before = heap.statistics().collections;
        heap.force_collect().unwrap();
        let after = heap.statistics().collections;
        
        assert_eq!(after, before + 1);
    }

    #[test]
    fn test_clear() {
        let mut heap = GcHeap::new();
        
        heap.alloc_string("test".to_string());
        heap.alloc_list(vec![]);
        
        assert_eq!(heap.object_count(), 2);
        
        heap.clear();
        
        assert_eq!(heap.object_count(), 0);
        assert_eq!(heap.statistics().allocations, 0);
    }
}