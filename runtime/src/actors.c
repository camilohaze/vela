/**
 * Vela Runtime - Actor System Implementation
 *
 * Implementation of the actor system for message-passing concurrency
 * in Vela programs compiled to native code.
 *
 * TASK-123: Implement runtime library in C
 */

#include "actors.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>
#include <unistd.h>

// ============================================================================
// GLOBAL ACTOR SYSTEM STATE
// ============================================================================

static vela_actor_system_t* actor_system = NULL;

// ============================================================================
// DEFAULT CONFIGURATION
// ============================================================================

static const actor_system_config_t default_config = {
    .max_actors = 1024,
    .max_mailbox_size = 256,
    .worker_threads = 4
};

// ============================================================================
// ACTOR SYSTEM INITIALIZATION
// ============================================================================

bool actors_init(const actor_system_config_t* config) {
    if (actor_system != NULL) {
        return false; // Already initialized
    }

    if (config == NULL) {
        config = &default_config;
    }

    actor_system = (vela_actor_system_t*)malloc(sizeof(vela_actor_system_t));
    if (actor_system == NULL) {
        return false;
    }

    memset(actor_system, 0, sizeof(vela_actor_system_t));

    // Initialize actor array
    actor_system->actor_capacity = config->max_actors;
    actor_system->actors = (vela_actor_t**)malloc(
        sizeof(vela_actor_t*) * actor_system->actor_capacity);
    if (actor_system->actors == NULL) {
        free(actor_system);
        actor_system = NULL;
        return false;
    }

    // Initialize worker threads array
    actor_system->worker_count = config->worker_threads;
    actor_system->worker_threads = (pthread_t*)malloc(
        sizeof(pthread_t) * actor_system->worker_count);
    if (actor_system->worker_threads == NULL) {
        free(actor_system->actors);
        free(actor_system);
        actor_system = NULL;
        return false;
    }

    actor_system->next_actor_id = 1; // Start IDs from 1
    return true;
}

void actors_shutdown(void) {
    if (actor_system == NULL) {
        return;
    }

    // Stop the system first
    actors_stop();

    // Destroy all actors
    for (size_t i = 0; i < actor_system->actor_count; i++) {
        if (actor_system->actors[i] != NULL) {
            actor_destroy(actor_system->actors[i]);
        }
    }

    // Free arrays
    free(actor_system->actors);
    free(actor_system->worker_threads);
    free(actor_system);

    actor_system = NULL;
}

bool actors_start(void) {
    if (actor_system == NULL || actor_system->running) {
        return false;
    }

    actor_system->running = true;

    // Start scheduler thread
    if (pthread_create(&actor_system->scheduler_thread, NULL,
                       actor_scheduler, actor_system) != 0) {
        actor_system->running = false;
        return false;
    }

    // Start worker threads
    for (size_t i = 0; i < actor_system->worker_count; i++) {
        if (pthread_create(&actor_system->worker_threads[i], NULL,
                           actor_worker, actor_system) != 0) {
            actor_system->running = false;
            return false;
        }
    }

    return true;
}

void actors_stop(void) {
    if (actor_system == NULL || !actor_system->running) {
        return;
    }

    actor_system->running = false;

    // Wait for scheduler thread to finish
    pthread_join(actor_system->scheduler_thread, NULL);

    // Wait for worker threads to finish
    for (size_t i = 0; i < actor_system->worker_count; i++) {
        pthread_join(actor_system->worker_threads[i], NULL);
    }
}

// ============================================================================
// ACTOR CREATION AND DESTRUCTION
// ============================================================================

vela_actor_t* actor_create(vela_actor_fn behavior, void* initial_state) {
    if (actor_system == NULL || behavior == NULL) {
        return NULL;
    }

    // Check if we have space for more actors
    if (actor_system->actor_count >= actor_system->actor_capacity) {
        return NULL;
    }

    vela_actor_t* actor = (vela_actor_t*)malloc(sizeof(vela_actor_t));
    if (actor == NULL) {
        return NULL;
    }

    memset(actor, 0, sizeof(vela_actor_t));
    actor->id = actor_generate_id();
    actor->behavior = behavior;
    actor->state = initial_state;
    actor->system = actor_system;
    actor->running = true;

    // Create mailbox
    actor->mailbox = message_queue_create(default_config.max_mailbox_size);
    if (actor->mailbox == NULL) {
        free(actor);
        return NULL;
    }

    // Add to system
    actor_system->actors[actor_system->actor_count++] = actor;

    return actor;
}

void actor_destroy(vela_actor_t* actor) {
    if (actor == NULL) {
        return;
    }

    // Stop the actor
    actor->running = false;
    actor->stopped = true;

    // Destroy mailbox
    if (actor->mailbox != NULL) {
        message_queue_destroy(actor->mailbox);
    }

    // Remove from system
    for (size_t i = 0; i < actor_system->actor_count; i++) {
        if (actor_system->actors[i] == actor) {
            // Shift remaining elements
            for (size_t j = i; j < actor_system->actor_count - 1; j++) {
                actor_system->actors[j] = actor_system->actors[j + 1];
            }
            actor_system->actor_count--;
            break;
        }
    }

    free(actor);
}

// ============================================================================
// MESSAGE PASSING
// ============================================================================

bool actor_send(vela_actor_t* actor, vela_message_t* message) {
    if (actor == NULL || message == NULL || actor->mailbox == NULL) {
        return false;
    }

    return message_queue_put(actor->mailbox, message);
}

void* actor_get_state(vela_actor_t* actor) {
    return actor != NULL ? actor->state : NULL;
}

void actor_set_state(vela_actor_t* actor, void* state) {
    if (actor != NULL) {
        actor->state = state;
    }
}

// ============================================================================
// MESSAGE QUEUE IMPLEMENTATION
// ============================================================================

message_queue_t* message_queue_create(size_t capacity) {
    message_queue_t* queue = (message_queue_t*)malloc(sizeof(message_queue_t));
    if (queue == NULL) {
        return NULL;
    }

    memset(queue, 0, sizeof(message_queue_t));
    queue->capacity = capacity;
    queue->messages = (vela_message_t**)malloc(sizeof(vela_message_t*) * capacity);
    if (queue->messages == NULL) {
        free(queue);
        return NULL;
    }

    // Initialize mutex and condition variables
    if (pthread_mutex_init(&queue->mutex, NULL) != 0 ||
        pthread_cond_init(&queue->not_empty, NULL) != 0 ||
        pthread_cond_init(&queue->not_full, NULL) != 0) {
        free(queue->messages);
        free(queue);
        return NULL;
    }

    return queue;
}

void message_queue_destroy(message_queue_t* queue) {
    if (queue == NULL) {
        return;
    }

    // Free all messages
    while (!message_queue_empty(queue)) {
        vela_message_t* message;
        if (message_queue_get(queue, &message)) {
            // Note: We don't free message data here as it's owned by sender
            free(message);
        }
    }

    // Destroy synchronization primitives
    pthread_mutex_destroy(&queue->mutex);
    pthread_cond_destroy(&queue->not_empty);
    pthread_cond_destroy(&queue->not_full);

    free(queue->messages);
    free(queue);
}

bool message_queue_put(message_queue_t* queue, vela_message_t* message) {
    if (queue == NULL || message == NULL) {
        return false;
    }

    pthread_mutex_lock(&queue->mutex);

    // Wait for space if queue is full
    while (message_queue_full(queue) && actor_system->running) {
        pthread_cond_wait(&queue->not_full, &queue->mutex);
    }

    if (!actor_system->running) {
        pthread_mutex_unlock(&queue->mutex);
        return false;
    }

    // Add message to queue
    queue->messages[queue->tail] = message;
    queue->tail = (queue->tail + 1) % queue->capacity;
    queue->count++;

    // Signal that queue is not empty
    pthread_cond_signal(&queue->not_empty);
    pthread_mutex_unlock(&queue->mutex);

    return true;
}

bool message_queue_get(message_queue_t* queue, vela_message_t** message) {
    if (queue == NULL || message == NULL) {
        return false;
    }

    pthread_mutex_lock(&queue->mutex);

    // Wait for message if queue is empty
    while (message_queue_empty(queue) && actor_system->running) {
        pthread_cond_wait(&queue->not_empty, &queue->mutex);
    }

    if (!actor_system->running || message_queue_empty(queue)) {
        pthread_mutex_unlock(&queue->mutex);
        return false;
    }

    // Get message from queue
    *message = queue->messages[queue->head];
    queue->head = (queue->head + 1) % queue->capacity;
    queue->count--;

    // Signal that queue is not full
    pthread_cond_signal(&queue->not_full);
    pthread_mutex_unlock(&queue->mutex);

    return true;
}

bool message_queue_empty(message_queue_t* queue) {
    return queue == NULL || queue->count == 0;
}

bool message_queue_full(message_queue_t* queue) {
    return queue == NULL || queue->count >= queue->capacity;
}

size_t message_queue_size(message_queue_t* queue) {
    return queue != NULL ? queue->count : 0;
}

// ============================================================================
// SCHEDULER AND WORKER THREADS
// ============================================================================

void* actor_scheduler(void* arg) {
    vela_actor_system_t* system = (vela_actor_system_t*)arg;

    while (system->running) {
        // Simple round-robin scheduling
        for (size_t i = 0; i < system->actor_count && system->running; i++) {
            vela_actor_t* actor = system->actors[i];
            if (actor != NULL && actor->running && !message_queue_empty(actor->mailbox)) {
                actor_schedule(actor);
            }
        }

        // Small delay to prevent busy waiting
        usleep(1000); // 1ms
    }

    return NULL;
}

void* actor_worker(void* arg) {
    vela_actor_system_t* system = (vela_actor_system_t*)arg;

    while (system->running) {
        // Wait for an actor to be scheduled
        // In a real implementation, this would use a work queue
        // For now, we'll just yield
        usleep(10000); // 10ms
    }

    return NULL;
}

bool actor_schedule(vela_actor_t* actor) {
    if (actor == NULL || !actor->running) {
        return false;
    }

    // Process one message from the actor's mailbox
    vela_message_t* message;
    if (!message_queue_get(actor->mailbox, &message)) {
        return false;
    }

    // Execute actor behavior
    if (actor->behavior != NULL) {
        actor->behavior(actor, message);
    }

    // Free the message (data is owned by sender)
    free(message);

    return true;
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

uint32_t actor_generate_id(void) {
    if (actor_system == NULL) {
        return 0;
    }

    return actor_system->next_actor_id++;
}

vela_actor_t* actor_find_by_id(uint32_t id) {
    if (actor_system == NULL) {
        return NULL;
    }

    for (size_t i = 0; i < actor_system->actor_count; i++) {
        if (actor_system->actors[i] != NULL && actor_system->actors[i]->id == id) {
            return actor_system->actors[i];
        }
    }

    return NULL;
}

size_t actors_count(void) {
    return actor_system != NULL ? actor_system->actor_count : 0;
}

bool actor_is_running(vela_actor_t* actor) {
    return actor != NULL && actor->running && !actor->stopped;
}

// ============================================================================
// PUBLIC API WRAPPERS
// ============================================================================

void vela_actors_init(void) {
    actors_init(NULL);
}

void vela_actors_shutdown(void) {
    actors_shutdown();
}

void vela_actors_run(void) {
    actors_start();
}

vela_actor_t* vela_actor_create(vela_actor_fn actor_fn, void* initial_state) {
    return actor_create(actor_fn, initial_state);
}

void vela_actor_destroy(vela_actor_t* actor) {
    actor_destroy(actor);
}

bool vela_actor_send(vela_actor_t* actor, vela_message_t* message) {
    return actor_send(actor, message);
}

void* vela_actor_get_state(vela_actor_t* actor) {
    return actor_get_state(actor);
}