/**
 * Vela Runtime - Garbage Collector Header
 *
 * This header defines the internal API for the Vela garbage collector,
 * implementing a mark-and-sweep algorithm for automatic memory management.
 *
 * TASK-123: Implement runtime library in C
 */

#ifndef VELA_GC_H
#define VELA_GC_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// GC INTERNAL TYPES
// ============================================================================

/**
 * GC object header - prepended to all GC-managed objects
 */
typedef struct gc_header {
    uint32_t flags;       // Object flags (marked, etc.)
    size_t size;         // Object size in bytes
    struct gc_header* next; // Next object in heap (for sweeping)
} gc_header_t;

/**
 * GC heap structure
 */
typedef struct {
    void* heap_start;     // Start of heap memory
    void* heap_end;       // End of heap memory
    size_t heap_size;     // Total heap size
    void* free_ptr;       // Next free allocation pointer (bump allocator)

    gc_header_t* objects; // Linked list of all objects
    size_t object_count;  // Number of live objects

    void** roots;         // Root set array
    size_t root_count;    // Number of roots
    size_t root_capacity; // Capacity of root array

    bool gc_running;      // Is GC currently running?
} gc_heap_t;

/**
 * Object flags
 */
#define GC_FLAG_MARKED   0x01  // Object is marked (reachable)
#define GC_FLAG_ROOT     0x02  // Object is in root set
#define GC_FLAG_ARRAY    0x04  // Object is an array
#define GC_FLAG_STRING   0x08  // Object is a string
#define GC_FLAG_OBJECT   0x10  // Object is a key-value object

// ============================================================================
// GC INTERNAL API
// ============================================================================

/**
 * Initialize the GC heap
 * @param heap_size Initial heap size in bytes
 * @return true on success, false on failure
 */
bool gc_init(size_t heap_size);

/**
 * Shutdown the GC heap
 */
void gc_shutdown(void);

/**
 * Allocate memory from the GC heap
 * @param size Size of allocation in bytes
 * @param flags Object flags
 * @return Pointer to allocated memory (after header), or NULL on failure
 */
void* gc_alloc(size_t size, uint32_t flags);

/**
 * Run a garbage collection cycle
 */
void gc_collect(void);

/**
 * Mark phase of garbage collection
 */
void gc_mark(void);

/**
 * Sweep phase of garbage collection
 */
void gc_sweep(void);

/**
 * Add a root pointer to the root set
 * @param ptr Root pointer to add
 * @return true on success, false on failure
 */
bool gc_add_root(void* ptr);

/**
 * Remove a root pointer from the root set
 * @param ptr Root pointer to remove
 */
void gc_remove_root(void* ptr);

/**
 * Check if a pointer is within the heap
 * @param ptr Pointer to check
 * @return true if pointer is in heap, false otherwise
 */
bool gc_is_heap_ptr(void* ptr);

/**
 * Get the header for a GC-managed object
 * @param ptr Object pointer (after header)
 * @return Pointer to the object header
 */
gc_header_t* gc_get_header(void* ptr);

/**
 * Mark an object as reachable
 * @param ptr Object to mark
 */
void gc_mark_object(void* ptr);

// ============================================================================
// GC STATISTICS
// ============================================================================

/**
 * GC statistics structure
 */
typedef struct {
    size_t heap_size;        // Total heap size
    size_t used_bytes;       // Bytes currently used
    size_t free_bytes;       // Bytes currently free
    size_t object_count;     // Number of live objects
    size_t collection_count; // Number of GC cycles run
    size_t total_allocated;  // Total bytes allocated since start
    size_t total_collected;  // Total bytes collected since start
} gc_stats_t;

/**
 * Get current GC statistics
 * @param stats Pointer to stats structure to fill
 */
void gc_get_stats(gc_stats_t* stats);

#endif // VELA_GC_H