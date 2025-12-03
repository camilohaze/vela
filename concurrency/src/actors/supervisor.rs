/*!
# Supervisor

Fault tolerance system for actors using supervision trees.

Supervisors monitor child actors and handle failures by:
- Restarting failed actors (with backoff)
- Stopping permanently failed actors
- Applying supervision strategies (OneForOne, OneForAll, RestForOne)

## Example

```rust
use vela_concurrency::actors::{Supervisor, SupervisionStrategy};

let supervisor = Supervisor::new(SupervisionStrategy::OneForOne {
    max_restarts: 3,
    within_seconds: 60,
});

// Spawn supervised actors
let child1 = supervisor.spawn(Actor1::new());
let child2 = supervisor.spawn(Actor2::new());

// If child1 fails, only child1 is restarted (OneForOne)
```
*/

use super::actor::{Actor, ActorId, ActorState};
use super::address::ActorAddress;
use super::context::ActorContext;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Supervision strategy for handling actor failures.
#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    /// Restart only the failed actor.
    ///
    /// Other children continue running.
    OneForOne {
        /// Maximum number of restarts allowed
        max_restarts: u32,
        /// Time window for counting restarts (seconds)
        within_seconds: u64,
    },

    /// Restart all children when one fails.
    ///
    /// Use when children are interdependent.
    OneForAll {
        /// Maximum number of restarts allowed
        max_restarts: u32,
        /// Time window for counting restarts (seconds)
        within_seconds: u64,
    },

    /// Restart the failed actor and all actors started after it.
    ///
    /// Use when actors form a processing pipeline.
    RestForOne {
        /// Maximum number of restarts allowed
        max_restarts: u32,
        /// Time window for counting restarts (seconds)
        within_seconds: u64,
    },
}

/// Message sent to supervisor when a child actor fails.
#[derive(Debug)]
pub struct ActorFailure {
    /// ID of the failed actor
    pub actor_id: ActorId,
    /// Error message
    pub error: String,
    /// When the failure occurred
    pub timestamp: Instant,
}

/// Supervisor actor that monitors child actors.
pub struct Supervisor {
    /// Supervision strategy
    strategy: SupervisionStrategy,
    /// Child actors being supervised
    children: Vec<SupervisedChild>,
    /// Restart counts per actor
    restart_counts: HashMap<ActorId, RestartHistory>,
}

/// Information about a supervised child actor.
struct SupervisedChild {
    actor_id: ActorId,
    state: ActorState,
    spawn_order: usize,
}

/// Restart history for an actor.
struct RestartHistory {
    count: u32,
    first_restart: Instant,
}

impl Supervisor {
    /// Create a new supervisor with the given strategy.
    pub fn new(strategy: SupervisionStrategy) -> Self {
        Self {
            strategy,
            children: Vec::new(),
            restart_counts: HashMap::new(),
        }
    }

    /// Spawn a child actor under supervision.
    pub fn spawn<A: Actor>(&mut self, actor: A) -> ActorAddress<A> {
        let actor_id = ActorId::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let address = ActorAddress::new(tx.clone(), actor_id);

        // Add to children
        self.children.push(SupervisedChild {
            actor_id,
            state: ActorState::Starting,
            spawn_order: self.children.len(),
        });

        // Clone address before moving into async block
        let addr_clone = address.clone();
        
        // Spawn actor runtime (with supervision)
        tokio::spawn(async move {
            let mut actor = actor;
            let mut mailbox = super::mailbox::Mailbox::new(rx);
            let mut ctx = ActorContext::new(addr_clone, actor_id);

            // Started hook
            actor.started(&mut ctx);

            // Message loop
            while let Some(msg) = mailbox.recv().await {
                actor.handle(msg, &mut ctx);

                if ctx.should_stop() {
                    break;
                }
            }

            // Stopped hook
            actor.stopped();
        });

        address
    }

    /// Handle an actor failure.
    pub fn handle_failure(&mut self, failure: ActorFailure) -> RestartDecision {
        let actor_id = failure.actor_id;

        // Check restart limits
        if self.should_restart(actor_id, failure.timestamp) {
            // Apply strategy
            match self.strategy {
                SupervisionStrategy::OneForOne { .. } => {
                    RestartDecision::Restart
                }
                SupervisionStrategy::OneForAll { .. } => {
                    RestartDecision::RestartAll
                }
                SupervisionStrategy::RestForOne { .. } => {
                    let spawn_order = self
                        .children
                        .iter()
                        .find(|c| c.actor_id == actor_id)
                        .map(|c| c.spawn_order)
                        .unwrap_or(0);

                    RestartDecision::RestartFrom(spawn_order)
                }
            }
        } else {
            RestartDecision::Stop
        }
    }

    /// Check if an actor should be restarted based on restart limits.
    fn should_restart(&mut self, actor_id: ActorId, now: Instant) -> bool {
        let (max_restarts, within_seconds) = match self.strategy {
            SupervisionStrategy::OneForOne {
                max_restarts,
                within_seconds,
            } => (max_restarts, within_seconds),
            SupervisionStrategy::OneForAll {
                max_restarts,
                within_seconds,
            } => (max_restarts, within_seconds),
            SupervisionStrategy::RestForOne {
                max_restarts,
                within_seconds,
            } => (max_restarts, within_seconds),
        };

        let within = Duration::from_secs(within_seconds);

        // Get or create restart history
        let history = self.restart_counts.entry(actor_id).or_insert_with(|| {
            RestartHistory {
                count: 0,
                first_restart: now,
            }
        });

        // Reset count if outside time window
        if now.duration_since(history.first_restart) > within {
            history.count = 0;
            history.first_restart = now;
        }

        // Check limit
        if history.count >= max_restarts {
            false
        } else {
            history.count += 1;
            true
        }
    }

    /// Get number of supervised children.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

/// Decision made by supervisor when an actor fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartDecision {
    /// Restart only the failed actor
    Restart,
    /// Restart all children
    RestartAll,
    /// Restart from specific spawn order (RestForOne)
    RestartFrom(usize),
    /// Stop the actor permanently
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_new() {
        let supervisor = Supervisor::new(SupervisionStrategy::OneForOne {
            max_restarts: 3,
            within_seconds: 60,
        });

        assert_eq!(supervisor.child_count(), 0);
    }

    #[test]
    fn test_supervision_strategy_one_for_one() {
        let mut supervisor = Supervisor::new(SupervisionStrategy::OneForOne {
            max_restarts: 3,
            within_seconds: 60,
        });

        let failure = ActorFailure {
            actor_id: ActorId::new(),
            error: "test error".to_string(),
            timestamp: Instant::now(),
        };

        let decision = supervisor.handle_failure(failure);
        assert_eq!(decision, RestartDecision::Restart);
    }

    #[test]
    fn test_supervision_strategy_one_for_all() {
        let mut supervisor = Supervisor::new(SupervisionStrategy::OneForAll {
            max_restarts: 3,
            within_seconds: 60,
        });

        let failure = ActorFailure {
            actor_id: ActorId::new(),
            error: "test error".to_string(),
            timestamp: Instant::now(),
        };

        let decision = supervisor.handle_failure(failure);
        assert_eq!(decision, RestartDecision::RestartAll);
    }

    #[test]
    fn test_restart_limit_exceeded() {
        let mut supervisor = Supervisor::new(SupervisionStrategy::OneForOne {
            max_restarts: 2,
            within_seconds: 60,
        });

        let actor_id = ActorId::new();

        // First restart - OK
        let failure1 = ActorFailure {
            actor_id,
            error: "error 1".to_string(),
            timestamp: Instant::now(),
        };
        assert_eq!(
            supervisor.handle_failure(failure1),
            RestartDecision::Restart
        );

        // Second restart - OK
        let failure2 = ActorFailure {
            actor_id,
            error: "error 2".to_string(),
            timestamp: Instant::now(),
        };
        assert_eq!(
            supervisor.handle_failure(failure2),
            RestartDecision::Restart
        );

        // Third restart - STOP (limit exceeded)
        let failure3 = ActorFailure {
            actor_id,
            error: "error 3".to_string(),
            timestamp: Instant::now(),
        };
        assert_eq!(supervisor.handle_failure(failure3), RestartDecision::Stop);
    }

    #[test]
    fn test_restart_count_reset_after_time_window() {
        let mut supervisor = Supervisor::new(SupervisionStrategy::OneForOne {
            max_restarts: 2,
            within_seconds: 1, // 1 second window
        });

        let actor_id = ActorId::new();

        // First restart
        let failure1 = ActorFailure {
            actor_id,
            error: "error 1".to_string(),
            timestamp: Instant::now(),
        };
        assert_eq!(
            supervisor.handle_failure(failure1),
            RestartDecision::Restart
        );

        // Wait for time window to expire
        std::thread::sleep(Duration::from_secs(2));

        // Second restart (after window) - should be OK
        let failure2 = ActorFailure {
            actor_id,
            error: "error 2".to_string(),
            timestamp: Instant::now(),
        };
        assert_eq!(
            supervisor.handle_failure(failure2),
            RestartDecision::Restart
        );
    }
}
