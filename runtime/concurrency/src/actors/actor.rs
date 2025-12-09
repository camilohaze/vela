/*!
# Actor Trait

Core trait for the Vela actor system.

Actors are concurrent, isolated entities that process messages sequentially.
Each actor has:
- Private mutable state
- FIFO mailbox for incoming messages
- Lifecycle hooks (started, stopped, restarting)

## Example

```rust
use vela_concurrency::actors::Actor;

struct Counter {
    count: u32,
}

enum CounterMessage {
    Increment(u32),
    GetCount(tokio::sync::oneshot::Sender<u32>),
}

impl Actor for Counter {
    type Message = CounterMessage;
    
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
        match msg {
            CounterMessage::Increment(n) => {
                self.count += n;
            }
            CounterMessage::GetCount(tx) => {
                let _ = tx.send(self.count);
            }
        }
    }
}
```
*/

use super::context::ActorContext;
use std::fmt;

/// Core trait for actors in the Vela concurrency system.
///
/// Actors must be `Send` (transferable between threads) and `'static` (no borrowed references).
pub trait Actor: Send + 'static {
    /// Message type this actor can receive.
    /// Must be `Send` to cross thread boundaries.
    type Message: Send;

    /// Handle an incoming message.
    ///
    /// This method is called sequentially for each message in the actor's mailbox.
    /// The actor has exclusive mutable access to its state during message processing.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to process
    /// * `ctx` - Actor context for sending messages, spawning children, etc.
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>)
    where
        Self: Sized;

    /// Called when the actor is started.
    ///
    /// Use this for initialization logic that requires the actor context.
    fn started(&mut self, _ctx: &mut ActorContext<Self>)
    where
        Self: Sized,
    {}

    /// Called when the actor is stopped.
    ///
    /// Use this for cleanup logic (closing connections, releasing resources, etc.).
    fn stopped(&mut self) {}

    /// Called when the actor is restarting after a failure.
    ///
    /// Use this to reset state or re-establish connections.
    fn restarting(&mut self, _ctx: &mut ActorContext<Self>)
    where
        Self: Sized,
    {}
}

/// Unique identifier for an actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId(u64);

impl ActorId {
    /// Create a new actor ID.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ActorId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ActorId({})", self.0)
    }
}

/// Actor state in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    /// Actor is starting up
    Starting,
    /// Actor is running and processing messages
    Running,
    /// Actor is stopping gracefully
    Stopping,
    /// Actor has stopped
    Stopped,
    /// Actor is restarting after failure
    Restarting,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_id_unique() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_actor_id_sequential() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();
        assert!(id2.0 > id1.0);
    }

    #[test]
    fn test_actor_id_default() {
        let id = ActorId::default();
        assert!(id.0 > 0);
    }

    #[test]
    fn test_actor_id_display() {
        let id = ActorId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("ActorId("));
    }

    #[test]
    fn test_actor_state_equality() {
        assert_eq!(ActorState::Starting, ActorState::Starting);
        assert_ne!(ActorState::Running, ActorState::Stopped);
    }
}
