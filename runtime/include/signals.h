/**
 * Vela Runtime - Reactive Signals Header
 *
 * This header defines the internal API for the Vela reactive signals system,
 * implementing reactive programming primitives for automatic dependency tracking
 * and change propagation.
 *
 * TASK-123: Implement runtime library in C
 */

#ifndef VELA_SIGNALS_H
#define VELA_SIGNALS_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// ============================================================================
// SIGNAL INTERNAL TYPES
// ============================================================================

/**
 * Signal structure
 */
typedef struct vela_signal {
    void* value;              // Current signal value
    void** dependents;        // Array of dependent signals/computeds
    size_t dependent_count;   // Number of dependents
    size_t dependent_capacity;// Capacity of dependents array
    bool is_computed;         // Is this a computed signal?
    union {
        vela_compute_fn compute_fn;  // For computed signals
        void* raw_value;             // For regular signals
    } data;
} vela_signal_t;

/**
 * Computed signal structure (inherits from signal)
 */
typedef struct vela_computed {
    vela_signal_t base;       // Base signal structure
    vela_compute_fn compute_fn; // Computation function
    bool needs_recompute;     // Does this need recomputation?
} vela_computed_t;

/**
 * Signals system state
 */
typedef struct {
    vela_signal_t** all_signals;     // Array of all signals
    size_t signal_count;             // Number of signals
    size_t signal_capacity;          // Capacity of signals array

    bool propagation_running;        // Is change propagation running?
    vela_signal_t** dirty_signals;   // Signals that need updating
    size_t dirty_count;              // Number of dirty signals
} signals_state_t;

// ============================================================================
// SIGNAL INTERNAL API
// ============================================================================

/**
 * Initialize the signals system
 * @return true on success, false on failure
 */
bool signals_init(void);

/**
 * Shutdown the signals system
 */
void signals_shutdown(void);

/**
 * Create a new signal
 * @param initial_value Initial value for the signal
 * @return Pointer to created signal, or NULL on failure
 */
vela_signal_t* signal_create(void* initial_value);

/**
 * Destroy a signal
 * @param signal Signal to destroy
 */
void signal_destroy(vela_signal_t* signal);

/**
 * Set the value of a signal
 * @param signal Signal to update
 * @param value New value
 */
void signal_set_value(vela_signal_t* signal, void* value);

/**
 * Get the current value of a signal
 * @param signal Signal to read
 * @return Current value
 */
void* signal_get_value(vela_signal_t* signal);

/**
 * Add a dependent to a signal
 * @param signal Signal to add dependent to
 * @param dependent Dependent signal/computed
 * @return true on success, false on failure
 */
bool signal_add_dependent(vela_signal_t* signal, vela_signal_t* dependent);

/**
 * Remove a dependent from a signal
 * @param signal Signal to remove dependent from
 * @param dependent Dependent to remove
 */
void signal_remove_dependent(vela_signal_t* signal, vela_signal_t* dependent);

/**
 * Mark a signal as dirty (needs recomputation)
 * @param signal Signal to mark as dirty
 */
void signal_mark_dirty(vela_signal_t* signal);

/**
 * Propagate changes through the signal graph
 */
void signals_propagate_changes(void);

/**
 * Create a computed signal
 * @param compute_fn Computation function
 * @return Pointer to created computed signal, or NULL on failure
 */
vela_computed_t* computed_create(vela_compute_fn compute_fn);

/**
 * Destroy a computed signal
 * @param computed Computed signal to destroy
 */
void computed_destroy(vela_computed_t* computed);

/**
 * Recompute a computed signal
 * @param computed Computed signal to recompute
 */
void computed_recompute(vela_computed_t* computed);

// ============================================================================
// SIGNAL UTILITIES
// ============================================================================

/**
 * Check if a signal needs recomputation
 * @param signal Signal to check
 * @return true if signal needs recomputation
 */
bool signal_needs_recompute(vela_signal_t* signal);

/**
 * Get the number of dependents for a signal
 * @param signal Signal to query
 * @return Number of dependents
 */
size_t signal_dependent_count(vela_signal_t* signal);

/**
 * Register a signal globally
 * @param signal Signal to register
 * @return true on success, false on failure
 */
bool signals_register_signal(vela_signal_t* signal);

/**
 * Unregister a signal globally
 * @param signal Signal to unregister
 */
void signals_unregister_signal(vela_signal_t* signal);

#endif // VELA_SIGNALS_H