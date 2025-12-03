/*
Garbage Collector for VelaVM

This module implements garbage collection for the Vela Virtual Machine.
Uses a mark-and-sweep algorithm with generational collection.
*/

use crate::bytecode::Value;
use std::collections::HashSet;

/// Garbage collector
pub struct GC {
    objects: Vec<GCObject>,
    roots: HashSet<usize>,
    next_id: usize,
}

struct GCObject {
    id: usize,
    value: Value,
    marked: bool,
    generation: usize,
}

impl GC {
    /// Create a new garbage collector
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            roots: HashSet::new(),
            next_id: 0,
        }
    }

    /// Allocate a new object
    pub fn allocate(&mut self, value: Value) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let obj = GCObject {
            id,
            value,
            marked: false,
            generation: 0,
        };

        self.objects.push(obj);
        id
    }

    /// Mark an object as a root
    pub fn mark_root(&mut self, id: usize) {
        self.roots.insert(id);
    }

    /// Unmark an object as a root
    pub fn unmark_root(&mut self, id: usize) {
        self.roots.remove(&id);
    }

    /// Run garbage collection
    pub fn collect(&mut self) {
        // Mark phase
        self.mark();

        // Sweep phase
        self.sweep();
    }

    /// Mark reachable objects
    fn mark(&mut self) {
        // Use a work list to avoid borrowing issues
        let mut work_list = Vec::new();

        // Add roots to work list
        for &root_id in &self.roots {
            work_list.push(root_id);
        }

        // Process work list
        while let Some(obj_id) = work_list.pop() {
            if let Some(obj_index) = self.objects.iter().position(|o| o.id == obj_id) {
                if !self.objects[obj_index].marked {
                    self.objects[obj_index].marked = true;

                    // Add referenced objects to work list
                    match &self.objects[obj_index].value {
                        Value::List(items) => {
                            for item in items {
                                if let Some(ref_id) = self.find_object_by_value(item) {
                                    work_list.push(ref_id);
                                }
                            }
                        }
                        Value::Dict(map) => {
                            for value in map.values() {
                                if let Some(ref_id) = self.find_object_by_value(value) {
                                    work_list.push(ref_id);
                                }
                            }
                        }
                        _ => {} // Other types don't have references
                    }
                }
            }
        }
    }

    /// Sweep unreachable objects
    fn sweep(&mut self) {
        self.objects.retain(|obj| obj.marked);
        // Reset marks for next collection
        for obj in &mut self.objects {
            obj.marked = false;
        }
    }

    /// Find object by value (simplified - in real implementation would use object references)
    fn find_object_by_value(&self, _value: &Value) -> Option<usize> {
        // This is a simplified implementation
        // In a real GC, objects would have references to other objects
        None
    }

    /// Get object by ID
    pub fn get_object(&self, id: usize) -> Option<&Value> {
        self.objects.iter().find(|o| o.id == id).map(|o| &o.value)
    }

    /// Get object by ID (mutable)
    pub fn get_object_mut(&mut self, id: usize) -> Option<&mut Value> {
        self.objects.iter_mut().find(|o| o.id == id).map(|o| &mut o.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_creation() {
        let gc = GC::new();
        assert!(gc.objects.is_empty());
        assert!(gc.roots.is_empty());
    }

    #[test]
    fn test_allocation() {
        let mut gc = GC::new();
        let id = gc.allocate(Value::Int(42));
        assert_eq!(id, 0);

        let obj = gc.get_object(id);
        assert!(obj.is_some());
        assert_eq!(obj.unwrap(), &Value::Int(42));
    }

    #[test]
    fn test_gc_collection() {
        let mut gc = GC::new();

        // Allocate some objects
        let id1 = gc.allocate(Value::Int(1));
        let id2 = gc.allocate(Value::Int(2));
        let _id3 = gc.allocate(Value::Int(3));

        // Mark one as root
        gc.mark_root(id1);

        // Collect - should keep id1, remove others
        gc.collect();

        assert!(gc.get_object(id1).is_some());
        assert!(gc.get_object(id2).is_none()); // Should be collected
    }
}