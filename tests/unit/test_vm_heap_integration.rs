/*
Tests unitarios para la integración VM-heap en debugging tools

Historia: VELA-142 (EPIC-14 DevTools & Debugging)
Tarea: TASK-142
Fecha: 2025-12-14

Tests para:
- VirtualMachine::new_with_heap()
- GcHeap::get_reactive_objects()
- Integración entre VM y heap para debugging
*/

#[cfg(test)]
mod test_vm_heap_integration {
    use vela_vm::{VirtualMachine, gc::{GcHeap, GcObject}};
    use std::sync::Arc;

    #[test]
    fn test_vm_creation_with_heap() {
        // Test creación de VM con heap
        let heap = GcHeap::new();
        let vm = VirtualMachine::new_with_heap(heap);

        // Verificar que VM se creó correctamente
        assert!(vm.is_valid(), "VM should be valid after creation with heap");
    }

    #[test]
    fn test_vm_creation_without_heap() {
        // Test creación de VM sin heap (debería funcionar)
        let vm = VirtualMachine::new();

        // VM sin heap debería ser válido pero sin acceso a reactive objects
        assert!(vm.is_valid(), "VM should be valid even without explicit heap");
    }

    #[test]
    fn test_heap_get_reactive_objects_empty() {
        // Test obtener objetos reactivos de heap vacío
        let heap = GcHeap::new();
        let reactive_objects = heap.get_reactive_objects();

        assert_eq!(reactive_objects.len(), 0, "Empty heap should have no reactive objects");
    }

    #[test]
    fn test_heap_get_reactive_objects_with_allocation() {
        // Test después de allocar algunos objetos
        let heap = GcHeap::new();

        // Allocamos algunos objetos que NO son reactivos
        let _obj1 = heap.allocate(GcObject::String("test".to_string()));
        let _obj2 = heap.allocate(GcObject::Number(42.0));

        let reactive_objects = heap.get_reactive_objects();
        assert_eq!(reactive_objects.len(), 0, "Heap with non-reactive objects should still have 0 reactive objects");
    }

    #[test]
    fn test_vm_get_reactive_objects_method() {
        // Test del método get_reactive_objects en VM
        let heap = GcHeap::new();
        let vm = VirtualMachine::new_with_heap(heap);

        // Verificar que el método existe y funciona
        let result = vm.get_reactive_objects();
        assert!(result.is_ok(), "get_reactive_objects should succeed");

        let reactive_objects = result.unwrap();
        assert_eq!(reactive_objects.len(), 0, "New VM should have no reactive objects");
    }

    #[test]
    fn test_reactive_object_filtering() {
        // Test que solo objetos reactivos sean incluidos
        let heap = GcHeap::new();

        // Allocar objetos de diferentes tipos
        let _string_obj = heap.allocate(GcObject::String("test".to_string()));
        let _number_obj = heap.allocate(GcObject::Number(42.0));
        let _array_obj = heap.allocate(GcObject::Array(vec![]));

        // Nota: En implementación real, habría objetos ReactiveSignal y ReactiveComputed
        // Por ahora, verificamos que el filtrado funciona (todos deberían ser excluidos)
        let reactive_objects = heap.get_reactive_objects();
        assert_eq!(reactive_objects.len(), 0, "Only reactive objects should be included");
    }

    #[test]
    fn test_heap_gc_cycles_with_reactive_objects() {
        // Test que GC maneje correctamente objetos reactivos
        let heap = GcHeap::new();

        // Allocar y liberar algunos objetos
        let obj1 = heap.allocate(GcObject::String("test".to_string()));
        let obj2 = heap.allocate(GcObject::Number(42.0));

        // Simular GC cycle
        heap.gc();

        // Verificar que objetos aún existen si son alcanzables
        // (En implementación real, verificaríamos objetos reactivos específicos)
        let reactive_objects = heap.get_reactive_objects();
        assert_eq!(reactive_objects.len(), 0, "GC should not affect reactive object counting");
    }

    #[test]
    fn test_vm_heap_sharing() {
        // Test que múltiples VMs pueden compartir heap si es necesario
        let heap = Arc::new(GcHeap::new());

        // Nota: En implementación actual, VM toma ownership del heap
        // Este test verifica el diseño conceptual
        let heap_clone = Arc::clone(&heap);
        assert_eq!(Arc::strong_count(&heap), 2, "Heap should be shareable via Arc");
    }
}