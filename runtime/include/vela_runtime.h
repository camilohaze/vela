/**
 * Vela Runtime Library - API Header
 *
 * This header defines the public API for the Vela runtime library,
 * which provides garbage collection, reactive signals, and actor system
 * support for native Vela programs compiled to LLVM IR.
 *
 * TASK-123: Implement runtime library in C
 * VELA-620: Backend Native (LLVM)
 */

#ifndef VELA_RUNTIME_H
#define VELA_RUNTIME_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/**
 * Opaque type for Vela objects (arrays, strings, user objects)
 */
typedef struct vela_object vela_object_t;

/**
 * Opaque type for reactive signals
 */
typedef struct vela_signal vela_signal_t;

/**
 * Opaque type for computed signals
 */
typedef struct vela_computed vela_computed_t;

/**
 * Opaque type for actors
 */
typedef struct vela_actor vela_actor_t;

/**
 * Message structure for actor communication
 */
typedef struct {
    uint32_t type;        // Message type identifier
    void* data;          // Message payload
    size_t data_size;    // Size of payload in bytes
} vela_message_t;

/**
 * Function signature for computed signal computation
 */
typedef void* (*vela_compute_fn)(void);

/**
 * Function signature for actor behavior
 */
typedef void (*vela_actor_fn)(vela_actor_t* self, vela_message_t* message);

// ============================================================================
// GARBAGE COLLECTOR API
// ============================================================================

/**
 * Initialize the garbage collector
 */
void vela_gc_init(void);

/**
 * Shutdown the garbage collector
 */
void vela_gc_shutdown(void);

/**
 * Allocate memory with garbage collection tracking
 * @param size Size of memory to allocate in bytes
 * @return Pointer to allocated memory, or NULL on failure
 */
void* vela_gc_alloc(size_t size);

/**
 * Force a garbage collection cycle
 */
void vela_gc_collect(void);

/**
 * Add a root pointer to the GC root set
 * @param ptr Pointer to add to root set
 */
void vela_gc_add_root(void* ptr);

/**
 * Remove a root pointer from the GC root set
 * @param ptr Pointer to remove from root set
 */
void vela_gc_remove_root(void* ptr);

/**
 * Get current heap statistics
 * @param used_bytes Pointer to store used bytes (can be NULL)
 * @param total_bytes Pointer to store total bytes (can be NULL)
 */
void vela_gc_get_stats(size_t* used_bytes, size_t* total_bytes);

// ============================================================================
// REACTIVE SIGNALS API
// ============================================================================

/**
 * Initialize the signals system
 */
void vela_signals_init(void);

/**
 * Shutdown the signals system
 */
void vela_signals_shutdown(void);

/**
 * Create a new reactive signal
 * @param initial_value Initial value for the signal
 * @return Pointer to the created signal, or NULL on failure
 */
vela_signal_t* vela_signal_create(void* initial_value);

/**
 * Destroy a signal and free its resources
 * @param signal Signal to destroy
 */
void vela_signal_destroy(vela_signal_t* signal);

/**
 * Set the value of a signal, triggering updates to dependent signals
 * @param signal Signal to update
 * @param value New value for the signal
 */
void vela_signal_set(vela_signal_t* signal, void* value);

/**
 * Get the current value of a signal
 * @param signal Signal to read
 * @return Current value of the signal
 */
void* vela_signal_get(vela_signal_t* signal);

/**
 * Create a computed signal that derives its value from other signals
 * @param compute_fn Function that computes the signal value
 * @return Pointer to the created computed signal, or NULL on failure
 */
vela_computed_t* vela_computed_create(vela_compute_fn compute_fn);

/**
 * Destroy a computed signal
 * @param computed Computed signal to destroy
 */
void vela_computed_destroy(vela_computed_t* computed);

/**
 * Get the current value of a computed signal
 * @param computed Computed signal to read
 * @return Current computed value
 */
void* vela_computed_get(vela_computed_t* computed);

// ============================================================================
// ACTOR SYSTEM API
// ============================================================================

/**
 * Initialize the actor system
 */
void vela_actors_init(void);

/**
 * Shutdown the actor system
 */
void vela_actors_shutdown(void);

/**
 * Run the actor system (starts message processing)
 */
void vela_actors_run(void);

/**
 * Create a new actor
 * @param actor_fn Function that defines actor behavior
 * @param initial_state Initial state for the actor
 * @return Pointer to the created actor, or NULL on failure
 */
vela_actor_t* vela_actor_create(vela_actor_fn actor_fn, void* initial_state);

/**
 * Destroy an actor
 * @param actor Actor to destroy
 */
void vela_actor_destroy(vela_actor_t* actor);

/**
 * Send a message to an actor
 * @param actor Target actor
 * @param message Message to send
 * @return true on success, false on failure
 */
bool vela_actor_send(vela_actor_t* actor, vela_message_t* message);

/**
 * Get the current state of an actor
 * @param actor Actor to query
 * @return Current actor state
 */
void* vela_actor_get_state(vela_actor_t* actor);

// ============================================================================
// VELA OBJECT API
// ============================================================================

/**
 * Create a new Vela array object
 * @param element_count Number of elements in the array
 * @param element_size Size of each element in bytes
 * @return Pointer to the created array object, or NULL on failure
 */
vela_object_t* vela_array_create(size_t element_count, size_t element_size);

/**
 * Get an element from a Vela array
 * @param array Array object
 * @param index Element index
 * @return Pointer to the element, or NULL if index is out of bounds
 */
void* vela_array_get(vela_object_t* array, size_t index);

/**
 * Set an element in a Vela array
 * @param array Array object
 * @param index Element index
 * @param value Pointer to the value to set
 * @return true on success, false on failure
 */
bool vela_array_set(vela_object_t* array, size_t index, void* value);

/**
 * Get the length of a Vela array
 * @param array Array object
 * @return Number of elements in the array
 */
size_t vela_array_length(vela_object_t* array);

/**
 * Create a new Vela string object
 * @param c_string C string to copy
 * @return Pointer to the created string object, or NULL on failure
 */
vela_object_t* vela_string_create(const char* c_string);

/**
 * Get the C string from a Vela string object
 * @param string String object
 * @return C string (null-terminated), or NULL on failure
 */
const char* vela_string_get(vela_object_t* string);

/**
 * Get the length of a Vela string
 * @param string String object
 * @return String length in bytes (excluding null terminator)
 */
size_t vela_string_length(vela_object_t* string);

/**
 * Create a new Vela object (key-value map)
 * @return Pointer to the created object, or NULL on failure
 */
vela_object_t* vela_object_create(void);

/**
 * Set a property on a Vela object
 * @param object Object to modify
 * @param key Property key (string)
 * @param value Property value
 * @return true on success, false on failure
 */
bool vela_object_set(vela_object_t* object, const char* key, void* value);

/**
 * Get a property from a Vela object
 * @param object Object to query
 * @param key Property key (string)
 * @return Property value, or NULL if key doesn't exist
 */
void* vela_object_get(vela_object_t* object, const char* key);

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Initialize the entire Vela runtime
 */
void vela_runtime_init(void);

/**
 * Shutdown the entire Vela runtime
 */
void vela_runtime_shutdown(void);

/**
 * Get runtime version information
 * @return Version string
 */
const char* vela_runtime_version(void);

#endif // VELA_RUNTIME_H