/**
 * Vela Runtime - Main Runtime Implementation
 *
 * Main runtime library implementation that integrates GC, signals, and actors
 * for native Vela program execution.
 *
 * TASK-123: Implement runtime library in C
 */

#include "vela_runtime.h"
#include "gc.h"
#include "signals.h"
#include "actors.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// RUNTIME INITIALIZATION AND SHUTDOWN
// ============================================================================

void vela_runtime_init(void) {
    // Initialize garbage collector
    vela_gc_init();

    // Initialize signals system
    vela_signals_init();

    // Initialize actor system
    vela_actors_init();
}

void vela_runtime_shutdown(void) {
    // Shutdown in reverse order
    vela_actors_shutdown();
    vela_signals_shutdown();
    vela_gc_shutdown();
}

const char* vela_runtime_version(void) {
    return "Vela Runtime v1.0.0 (TASK-123)";
}

// ============================================================================
// VELA OBJECT IMPLEMENTATION
// ============================================================================

vela_object_t* vela_array_create(size_t element_count, size_t element_size) {
    if (element_count == 0 || element_size == 0) {
        return NULL;
    }

    // Calculate total size needed
    size_t total_size = sizeof(size_t) + (element_count * element_size); // length + elements

    // Allocate through GC
    vela_object_t* array = (vela_object_t*)vela_gc_alloc(total_size);
    if (array == NULL) {
        return NULL;
    }

    // Set object type flag
    gc_header_t* header = gc_get_header(array);
    if (header != NULL) {
        header->flags |= GC_FLAG_ARRAY;
    }

    // Store array length at the beginning
    size_t* length_ptr = (size_t*)array;
    *length_ptr = element_count;

    // Initialize elements to zero
    void* elements = (char*)array + sizeof(size_t);
    memset(elements, 0, element_count * element_size);

    return array;
}

void* vela_array_get(vela_object_t* array, size_t index) {
    if (array == NULL) {
        return NULL;
    }

    size_t* length_ptr = (size_t*)array;
    size_t length = *length_ptr;

    if (index >= length) {
        return NULL; // Index out of bounds
    }

    // Calculate element size from GC header
    gc_header_t* header = gc_get_header(array);
    if (header == NULL) {
        return NULL;
    }

    size_t total_size = header->size;
    size_t element_size = (total_size - sizeof(size_t)) / length;

    // Return pointer to element
    char* elements = (char*)array + sizeof(size_t);
    return elements + (index * element_size);
}

bool vela_array_set(vela_object_t* array, size_t index, void* value) {
    if (array == NULL || value == NULL) {
        return false;
    }

    void* element_ptr = vela_array_get(array, index);
    if (element_ptr == NULL) {
        return false;
    }

    // Get element size
    size_t* length_ptr = (size_t*)array;
    size_t length = *length_ptr;
    gc_header_t* header = gc_get_header(array);
    if (header == NULL) {
        return false;
    }

    size_t total_size = header->size;
    size_t element_size = (total_size - sizeof(size_t)) / length;

    // Copy value to element
    memcpy(element_ptr, value, element_size);
    return true;
}

size_t vela_array_length(vela_object_t* array) {
    if (array == NULL) {
        return 0;
    }

    return *(size_t*)array;
}

vela_object_t* vela_string_create(const char* c_string) {
    if (c_string == NULL) {
        return NULL;
    }

    size_t length = strlen(c_string);
    size_t total_size = sizeof(size_t) + length + 1; // length + string + null terminator

    // Allocate through GC
    vela_object_t* string = (vela_object_t*)vela_gc_alloc(total_size);
    if (string == NULL) {
        return NULL;
    }

    // Set object type flag
    gc_header_t* header = gc_get_header(string);
    if (header != NULL) {
        header->flags |= GC_FLAG_STRING;
    }

    // Store string length
    size_t* length_ptr = (size_t*)string;
    *length_ptr = length;

    // Copy string data
    char* string_data = (char*)string + sizeof(size_t);
    strcpy(string_data, c_string);

    return string;
}

const char* vela_string_get(vela_object_t* string) {
    if (string == NULL) {
        return NULL;
    }

    return (const char*)string + sizeof(size_t);
}

size_t vela_string_length(vela_object_t* string) {
    if (string == NULL) {
        return 0;
    }

    return *(size_t*)string;
}

vela_object_t* vela_object_create(void) {
    // For a simple implementation, we'll use a fixed-size hash table
    // In a real implementation, this would be a dynamic hash table
    const size_t object_size = sizeof(size_t) + (256 * sizeof(void*)); // capacity + entries

    vela_object_t* object = (vela_object_t*)vela_gc_alloc(object_size);
    if (object == NULL) {
        return NULL;
    }

    // Set object type flag
    gc_header_t* header = gc_get_header(object);
    if (header != NULL) {
        header->flags |= GC_FLAG_OBJECT;
    }

    // Initialize as empty object
    memset(object, 0, object_size);

    return object;
}

bool vela_object_set(vela_object_t* object, const char* key, void* value) {
    if (object == NULL || key == NULL) {
        return false;
    }

    // Simple linear search implementation
    // In a real implementation, this would be a proper hash table
    void** entries = (void**)object;
    size_t capacity = 256; // Fixed capacity for simplicity

    // Look for existing key or empty slot
    for (size_t i = 0; i < capacity; i += 2) {
        void* existing_key = entries[i];
        if (existing_key == NULL) {
            // Empty slot - store key-value pair
            entries[i] = (void*)key;     // Key
            entries[i + 1] = value;      // Value
            return true;
        } else if (strcmp((const char*)existing_key, key) == 0) {
            // Key exists - update value
            entries[i + 1] = value;
            return true;
        }
    }

    return false; // Object is full (shouldn't happen with proper implementation)
}

void* vela_object_get(vela_object_t* object, const char* key) {
    if (object == NULL || key == NULL) {
        return NULL;
    }

    // Simple linear search
    void** entries = (void**)object;
    size_t capacity = 256;

    for (size_t i = 0; i < capacity; i += 2) {
        void* existing_key = entries[i];
        if (existing_key == NULL) {
            break; // End of entries
        } else if (strcmp((const char*)existing_key, key) == 0) {
            return entries[i + 1]; // Return value
        }
    }

    return NULL; // Key not found
}

// ============================================================================
// INTEGRATION WITH LLVM BACKEND
// ============================================================================

// These functions are called by the LLVM-generated code

void vela_init_runtime(void) {
    vela_runtime_init();
}

void vela_shutdown_runtime(void) {
    vela_runtime_shutdown();
}

// Array operations for LLVM IR
void* vela_create_array(size_t element_count, size_t element_size) {
    return vela_array_create(element_count, element_size);
}

void* vela_array_get_element(void* array, size_t index) {
    return vela_array_get((vela_object_t*)array, index);
}

void vela_array_set_element(void* array, size_t index, void* value) {
    vela_array_set((vela_object_t*)array, index, value);
}

size_t vela_get_array_length(void* array) {
    return vela_array_length((vela_object_t*)array);
}

// String operations for LLVM IR
void* vela_create_string(const char* str) {
    return vela_string_create(str);
}

const char* vela_get_string_data(void* string) {
    return vela_string_get((vela_object_t*)string);
}

size_t vela_get_string_length(void* string) {
    return vela_string_length((vela_object_t*)string);
}

// Object operations for LLVM IR
void* vela_create_object(void) {
    return vela_object_create();
}

void vela_object_set_property(void* object, const char* key, void* value) {
    vela_object_set((vela_object_t*)object, key, value);
}

void* vela_object_get_property(void* object, const char* key) {
    return vela_object_get((vela_object_t*)object, key);
}

// Signal operations for LLVM IR
void* vela_create_signal(void* initial_value) {
    return vela_signal_create(initial_value);
}

void vela_set_signal(void* signal, void* value) {
    vela_signal_set((vela_signal_t*)signal, value);
}

void* vela_get_signal(void* signal) {
    return vela_signal_get((vela_signal_t*)signal);
}

void* vela_create_computed_signal(void* (*compute_fn)(void)) {
    return vela_computed_create(compute_fn);
}

void* vela_get_computed_signal(void* computed) {
    return vela_computed_get((vela_computed_t*)computed);
}

// Actor operations for LLVM IR
void* vela_create_actor(void (*actor_fn)(void*, void*), void* initial_state) {
    return vela_actor_create((vela_actor_fn)actor_fn, initial_state);
}

int vela_send_message(void* actor, void* message) {
    // Create a message structure
    vela_message_t msg;
    memset(&msg, 0, sizeof(msg));
    msg.type = 0; // Default message type
    msg.data = message;
    msg.data_size = 0; // Unknown size

    return vela_actor_send((vela_actor_t*)actor, &msg) ? 1 : 0;
}

void* vela_get_actor_state(void* actor) {
    return vela_actor_get_state((vela_actor_t*)actor);
}

// Memory management for LLVM IR
void* vela_gc_allocate(size_t size) {
    return vela_gc_alloc(size);
}

void vela_gc_add_to_root(void* ptr) {
    vela_gc_add_root(ptr);
}

void vela_gc_remove_from_root(void* ptr) {
    vela_gc_remove_root(ptr);
}

void vela_run_gc(void) {
    vela_gc_collect();
}