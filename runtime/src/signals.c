/**
 * Vela Runtime - Reactive Signals Implementation
 *
 * Implementation of reactive signals for automatic dependency tracking
 * and change propagation in Vela programs.
 *
 * TASK-123: Implement runtime library in C
 */

#include "signals.h"
#include "gc.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>

// ============================================================================
// GLOBAL SIGNALS STATE
// ============================================================================

static signals_state_t* signals_state = NULL;

// ============================================================================
// SIGNALS SYSTEM INITIALIZATION
// ============================================================================

bool signals_init(void) {
    if (signals_state != NULL) {
        return false; // Already initialized
    }

    signals_state = (signals_state_t*)malloc(sizeof(signals_state_t));
    if (signals_state == NULL) {
        return false;
    }

    memset(signals_state, 0, sizeof(signals_state_t));

    // Initialize signal arrays
    signals_state->signal_capacity = 64;
    signals_state->all_signals = (vela_signal_t**)malloc(
        sizeof(vela_signal_t*) * signals_state->signal_capacity);
    if (signals_state->all_signals == NULL) {
        free(signals_state);
        signals_state = NULL;
        return false;
    }

    // Initialize dirty signals array
    signals_state->dirty_signals = (vela_signal_t**)malloc(
        sizeof(vela_signal_t*) * signals_state->signal_capacity);
    if (signals_state->dirty_signals == NULL) {
        free(signals_state->all_signals);
        free(signals_state);
        signals_state = NULL;
        return false;
    }

    return true;
}

void signals_shutdown(void) {
    if (signals_state == NULL) {
        return;
    }

    // Destroy all signals
    for (size_t i = 0; i < signals_state->signal_count; i++) {
        if (signals_state->all_signals[i] != NULL) {
            signal_destroy(signals_state->all_signals[i]);
        }
    }

    // Free arrays
    free(signals_state->all_signals);
    free(signals_state->dirty_signals);
    free(signals_state);

    signals_state = NULL;
}

// ============================================================================
// SIGNAL CREATION AND DESTRUCTION
// ============================================================================

vela_signal_t* signal_create(void* initial_value) {
    if (signals_state == NULL) {
        return NULL;
    }

    vela_signal_t* signal = (vela_signal_t*)vela_gc_alloc(sizeof(vela_signal_t));
    if (signal == NULL) {
        return NULL;
    }

    memset(signal, 0, sizeof(vela_signal_t));
    signal->value = initial_value;
    signal->is_computed = false;
    signal->data.raw_value = initial_value;

    // Initialize dependents array
    signal->dependent_capacity = 8;
    signal->dependents = (void**)malloc(sizeof(void*) * signal->dependent_capacity);
    if (signal->dependents == NULL) {
        return NULL;
    }

    // Register signal globally
    if (!signals_register_signal(signal)) {
        free(signal->dependents);
        return NULL;
    }

    return signal;
}

void signal_destroy(vela_signal_t* signal) {
    if (signal == NULL) {
        return;
    }

    // Remove from global registry
    signals_unregister_signal(signal);

    // Free dependents array
    if (signal->dependents != NULL) {
        free(signal->dependents);
    }

    // Note: We don't free the signal itself as it's GC-managed
}

// ============================================================================
// SIGNAL VALUE MANAGEMENT
// ============================================================================

void signal_set_value(vela_signal_t* signal, void* value) {
    if (signal == NULL || signal->is_computed) {
        return;
    }

    signal->value = value;
    signal->data.raw_value = value;

    // Mark as dirty and propagate changes
    signal_mark_dirty(signal);
    signals_propagate_changes();
}

void* signal_get_value(vela_signal_t* signal) {
    if (signal == NULL) {
        return NULL;
    }

    // For computed signals, ensure they're up to date
    if (signal->is_computed) {
        vela_computed_t* computed = (vela_computed_t*)signal;
        if (computed->needs_recompute) {
            computed_recompute(computed);
        }
    }

    return signal->value;
}

// ============================================================================
// DEPENDENCY MANAGEMENT
// ============================================================================

bool signal_add_dependent(vela_signal_t* signal, vela_signal_t* dependent) {
    if (signal == NULL || dependent == NULL) {
        return false;
    }

    // Check if already a dependent
    for (size_t i = 0; i < signal->dependent_count; i++) {
        if (signal->dependents[i] == dependent) {
            return true;
        }
    }

    // Expand dependents array if needed
    if (signal->dependent_count >= signal->dependent_capacity) {
        size_t new_capacity = signal->dependent_capacity * 2;
        void** new_dependents = (void**)realloc(signal->dependents,
                                               sizeof(void*) * new_capacity);
        if (new_dependents == NULL) {
            return false;
        }
        signal->dependents = new_dependents;
        signal->dependent_capacity = new_capacity;
    }

    // Add dependent
    signal->dependents[signal->dependent_count++] = dependent;
    return true;
}

void signal_remove_dependent(vela_signal_t* signal, vela_signal_t* dependent) {
    if (signal == NULL || dependent == NULL) {
        return;
    }

    // Find and remove dependent
    for (size_t i = 0; i < signal->dependent_count; i++) {
        if (signal->dependents[i] == dependent) {
            // Shift remaining elements
            for (size_t j = i; j < signal->dependent_count - 1; j++) {
                signal->dependents[j] = signal->dependents[j + 1];
            }
            signal->dependent_count--;
            break;
        }
    }
}

// ============================================================================
// CHANGE PROPAGATION
// ============================================================================

void signal_mark_dirty(vela_signal_t* signal) {
    if (signal == NULL || signals_state == NULL) {
        return;
    }

    // Add to dirty signals if not already there
    for (size_t i = 0; i < signals_state->dirty_count; i++) {
        if (signals_state->dirty_signals[i] == signal) {
            return; // Already marked
        }
    }

    // Expand dirty array if needed
    if (signals_state->dirty_count >= signals_state->signal_capacity) {
        size_t new_capacity = signals_state->signal_capacity * 2;
        vela_signal_t** new_dirty = (vela_signal_t**)realloc(
            signals_state->dirty_signals,
            sizeof(vela_signal_t*) * new_capacity);
        if (new_dirty == NULL) {
            return;
        }
        signals_state->dirty_signals = new_dirty;
        signals_state->signal_capacity = new_capacity;
    }

    signals_state->dirty_signals[signals_state->dirty_count++] = signal;

    // Mark all dependents as dirty recursively
    for (size_t i = 0; i < signal->dependent_count; i++) {
        vela_signal_t* dependent = (vela_signal_t*)signal->dependents[i];
        if (dependent->is_computed) {
            ((vela_computed_t*)dependent)->needs_recompute = true;
        }
        signal_mark_dirty(dependent);
    }
}

void signals_propagate_changes(void) {
    if (signals_state == NULL || signals_state->propagation_running) {
        return;
    }

    signals_state->propagation_running = true;

    // Process all dirty signals
    for (size_t i = 0; i < signals_state->dirty_count; i++) {
        vela_signal_t* signal = signals_state->dirty_signals[i];
        if (signal != NULL && signal->is_computed) {
            vela_computed_t* computed = (vela_computed_t*)signal;
            if (computed->needs_recompute) {
                computed_recompute(computed);
            }
        }
    }

    // Clear dirty signals
    signals_state->dirty_count = 0;

    signals_state->propagation_running = false;
}

// ============================================================================
// COMPUTED SIGNALS
// ============================================================================

vela_computed_t* computed_create(vela_compute_fn compute_fn) {
    if (signals_state == NULL || compute_fn == NULL) {
        return NULL;
    }

    vela_computed_t* computed = (vela_computed_t*)vela_gc_alloc(sizeof(vela_computed_t));
    if (computed == NULL) {
        return NULL;
    }

    memset(computed, 0, sizeof(vela_computed_t));
    computed->base.is_computed = true;
    computed->compute_fn = compute_fn;
    computed->needs_recompute = true;

    // Initialize dependents array for the base signal
    computed->base.dependent_capacity = 8;
    computed->base.dependents = (void**)malloc(sizeof(void*) * computed->base.dependent_capacity);
    if (computed->base.dependents == NULL) {
        return NULL;
    }

    // Register computed signal globally
    if (!signals_register_signal((vela_signal_t*)computed)) {
        free(computed->base.dependents);
        return NULL;
    }

    // Initial computation
    computed_recompute(computed);

    return computed;
}

void computed_destroy(vela_computed_t* computed) {
    if (computed == NULL) {
        return;
    }

    signal_destroy((vela_signal_t*)computed);
}

void computed_recompute(vela_computed_t* computed) {
    if (computed == NULL || computed->compute_fn == NULL) {
        return;
    }

    computed->base.value = computed->compute_fn();
    computed->needs_recompute = false;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

bool signal_needs_recompute(vela_signal_t* signal) {
    if (signal == NULL || !signal->is_computed) {
        return false;
    }

    return ((vela_computed_t*)signal)->needs_recompute;
}

size_t signal_dependent_count(vela_signal_t* signal) {
    return signal != NULL ? signal->dependent_count : 0;
}

bool signals_register_signal(vela_signal_t* signal) {
    if (signals_state == NULL || signal == NULL) {
        return false;
    }

    // Expand signals array if needed
    if (signals_state->signal_count >= signals_state->signal_capacity) {
        size_t new_capacity = signals_state->signal_capacity * 2;
        vela_signal_t** new_signals = (vela_signal_t**)realloc(
            signals_state->all_signals,
            sizeof(vela_signal_t*) * new_capacity);
        if (new_signals == NULL) {
            return false;
        }
        signals_state->all_signals = new_signals;
        signals_state->signal_capacity = new_capacity;
    }

    signals_state->all_signals[signals_state->signal_count++] = signal;
    return true;
}

void signals_unregister_signal(vela_signal_t* signal) {
    if (signals_state == NULL || signal == NULL) {
        return;
    }

    // Find and remove from global registry
    for (size_t i = 0; i < signals_state->signal_count; i++) {
        if (signals_state->all_signals[i] == signal) {
            // Shift remaining elements
            for (size_t j = i; j < signals_state->signal_count - 1; j++) {
                signals_state->all_signals[j] = signals_state->all_signals[j + 1];
            }
            signals_state->signal_count--;
            break;
        }
    }
}

// ============================================================================
// PUBLIC API WRAPPERS
// ============================================================================

void vela_signals_init(void) {
    signals_init();
}

void vela_signals_shutdown(void) {
    signals_shutdown();
}

vela_signal_t* vela_signal_create(void* initial_value) {
    return signal_create(initial_value);
}

void vela_signal_destroy(vela_signal_t* signal) {
    signal_destroy(signal);
}

void vela_signal_set(vela_signal_t* signal, void* value) {
    signal_set_value(signal, value);
}

void* vela_signal_get(vela_signal_t* signal) {
    return signal_get_value(signal);
}

vela_computed_t* vela_computed_create(vela_compute_fn compute_fn) {
    return computed_create(compute_fn);
}

void vela_computed_destroy(vela_computed_t* computed) {
    computed_destroy(computed);
}

void* vela_computed_get(vela_computed_t* computed) {
    return signal_get_value((vela_signal_t*)computed);
}