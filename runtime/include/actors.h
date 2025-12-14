/**
 * Vela Runtime - Actor System Header
 *
 * This header defines the internal API for the Vela actor system,
 * implementing message-passing concurrency with isolated actor processes.
 *
 * TASK-123: Implement runtime library in C
 */

#ifndef VELA_ACTORS_H
#define VELA_ACTORS_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>

// ============================================================================
// ACTOR INTERNAL TYPES
// ============================================================================

/**
 * Message structure for actor communication
 */
typedef struct {
    uint32_t type;        // Message type identifier
    void* data;          // Message payload
    size_t data_size;    // Size of payload in bytes
    struct vela_actor* sender; // Sending actor (can be NULL)
} vela_message_t;

/**
 * Message queue structure
 */
typedef struct {
    vela_message_t** messages;    // Array of messages
    size_t count;                 // Number of messages
    size_t capacity;              // Capacity of message array
    size_t head;                  // Queue head index
    size_t tail;                  // Queue tail index
    pthread_mutex_t mutex;        // Mutex for thread safety
    pthread_cond_t not_empty;     // Condition for non-empty queue
    pthread_cond_t not_full;      // Condition for non-full queue
} message_queue_t;

/**
 * Actor structure
 */
typedef struct vela_actor {
    uint32_t id;                  // Unique actor ID
    vela_actor_fn behavior;       // Actor behavior function
    void* state;                  // Actor state
    message_queue_t* mailbox;     // Actor's message queue
    bool running;                 // Is actor running?
    bool stopped;                 // Has actor been stopped?
    pthread_t thread;             // Actor thread
    struct vela_actor_system* system; // Parent actor system
} vela_actor_t;

/**
 * Actor system structure
 */
typedef struct vela_actor_system {
    vela_actor_t** actors;        // Array of all actors
    size_t actor_count;           // Number of actors
    size_t actor_capacity;        // Capacity of actors array

    uint32_t next_actor_id;       // Next actor ID to assign

    bool running;                 // Is system running?
    pthread_t scheduler_thread;   // Scheduler thread

    // Thread pool for actor execution
    pthread_t* worker_threads;    // Worker thread pool
    size_t worker_count;          // Number of worker threads
} vela_actor_system_t;

/**
 * Actor system configuration
 */
typedef struct {
    size_t max_actors;            // Maximum number of actors
    size_t max_mailbox_size;      // Maximum mailbox size per actor
    size_t worker_threads;        // Number of worker threads
} actor_system_config_t;

// ============================================================================
// ACTOR INTERNAL API
// ============================================================================

/**
 * Initialize the actor system
 * @param config System configuration
 * @return true on success, false on failure
 */
bool actors_init(const actor_system_config_t* config);

/**
 * Shutdown the actor system
 */
void actors_shutdown(void);

/**
 * Start the actor system (begin processing messages)
 * @return true on success, false on failure
 */
bool actors_start(void);

/**
 * Stop the actor system
 */
void actors_stop(void);

/**
 * Create a new actor
 * @param behavior Actor behavior function
 * @param initial_state Initial actor state
 * @return Pointer to created actor, or NULL on failure
 */
vela_actor_t* actor_create(vela_actor_fn behavior, void* initial_state);

/**
 * Destroy an actor
 * @param actor Actor to destroy
 */
void actor_destroy(vela_actor_t* actor);

/**
 * Send a message to an actor
 * @param actor Target actor
 * @param message Message to send
 * @return true on success, false on failure
 */
bool actor_send(vela_actor_t* actor, vela_message_t* message);

/**
 * Get the current state of an actor
 * @param actor Actor to query
 * @return Current actor state
 */
void* actor_get_state(vela_actor_t* actor);

/**
 * Set the state of an actor
 * @param actor Actor to modify
 * @param state New state
 */
void actor_set_state(vela_actor_t* actor, void* state);

// ============================================================================
// MESSAGE QUEUE API
// ============================================================================

/**
 * Create a new message queue
 * @param capacity Maximum capacity of the queue
 * @return Pointer to created queue, or NULL on failure
 */
message_queue_t* message_queue_create(size_t capacity);

/**
 * Destroy a message queue
 * @param queue Queue to destroy
 */
void message_queue_destroy(message_queue_t* queue);

/**
 * Add a message to the queue
 * @param queue Queue to add to
 * @param message Message to add
 * @return true on success, false on failure (queue full)
 */
bool message_queue_put(message_queue_t* queue, vela_message_t* message);

/**
 * Remove a message from the queue
 * @param queue Queue to remove from
 * @param message Pointer to store the removed message
 * @return true on success, false on failure (queue empty)
 */
bool message_queue_get(message_queue_t* queue, vela_message_t** message);

/**
 * Check if queue is empty
 * @param queue Queue to check
 * @return true if empty, false otherwise
 */
bool message_queue_empty(message_queue_t* queue);

/**
 * Check if queue is full
 * @param queue Queue to check
 * @return true if full, false otherwise
 */
bool message_queue_full(message_queue_t* queue);

/**
 * Get the current size of the queue
 * @param queue Queue to check
 * @return Number of messages in queue
 */
size_t message_queue_size(message_queue_t* queue);

// ============================================================================
// ACTOR SCHEDULER
// ============================================================================

/**
 * Actor scheduler function (runs in separate thread)
 * @param arg Actor system pointer
 * @return NULL
 */
void* actor_scheduler(void* arg);

/**
 * Actor worker function (runs in worker threads)
 * @param arg Actor pointer
 * @return NULL
 */
void* actor_worker(void* arg);

/**
 * Schedule an actor for execution
 * @param actor Actor to schedule
 * @return true on success, false on failure
 */
bool actor_schedule(vela_actor_t* actor);

// ============================================================================
// ACTOR UTILITIES
// ============================================================================

/**
 * Generate a unique actor ID
 * @return New unique actor ID
 */
uint32_t actor_generate_id(void);

/**
 * Find an actor by ID
 * @param id Actor ID to search for
 * @return Pointer to actor, or NULL if not found
 */
vela_actor_t* actor_find_by_id(uint32_t id);

/**
 * Get the number of active actors
 * @return Number of active actors
 */
size_t actors_count(void);

/**
 * Check if an actor is running
 * @param actor Actor to check
 * @return true if running, false otherwise
 */
bool actor_is_running(vela_actor_t* actor);

#endif // VELA_ACTORS_H