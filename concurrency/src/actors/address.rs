/*!
# Actor Address

Type-safe handle for sending messages to actors.

ActorAddress provides:
- Type-safe message sending (compile-time checked)
- Non-blocking send operations
- Cheap cloning (uses Arc internally)
- Request-response patterns via oneshot channels

## Example

```rust
let addr = my_actor.start();

// Send message (non-blocking)
addr.send(MyMessage::Ping).unwrap();

// Clone address (cheap)
let addr2 = addr.clone();
tokio::spawn(async move {
    addr2.send(MyMessage::Pong).unwrap();
});
```
*/

use super::actor::{Actor, ActorId};
use std::marker::PhantomData;
use std::fmt;
use thiserror::Error;
use tokio::sync::mpsc;

/// Error type for send operations.
#[derive(Error, Debug)]
pub enum SendError {
    /// Actor mailbox is closed (actor stopped)
    #[error("Actor mailbox closed (actor stopped)")]
    Closed,
}

/// Error type for try_send operations.
#[derive(Error, Debug)]
pub enum TrySendError {
    /// Actor mailbox is full (backpressure)
    #[error("Actor mailbox full (backpressure applied)")]
    Full,
    /// Actor mailbox is closed (actor stopped)
    #[error("Actor mailbox closed (actor stopped)")]
    Closed,
}

/// Type-safe handle to send messages to an actor.
///
/// ActorAddress can be cloned cheaply (uses Arc internally) and sent across threads.
pub struct ActorAddress<A: Actor> {
    /// Sender for the actor's mailbox
    sender: mpsc::UnboundedSender<A::Message>,
    /// Actor ID for debugging
    actor_id: ActorId,
    /// PhantomData to make ActorAddress generic over Actor type
    _phantom: PhantomData<A>,
}

impl<A: Actor> ActorAddress<A> {
    /// Create a new actor address.
    pub(crate) fn new(sender: mpsc::UnboundedSender<A::Message>, actor_id: ActorId) -> Self {
        Self {
            sender,
            actor_id,
            _phantom: PhantomData,
        }
    }

    /// Send a message to the actor (non-blocking).
    ///
    /// Returns an error if the actor's mailbox is closed (actor stopped).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::ActorAddress;
    /// # async fn example(addr: ActorAddress<MyActor>) {
    /// addr.send(MyMessage::Ping).unwrap();
    /// # }
    /// ```
    pub fn send(&self, msg: A::Message) -> Result<(), SendError> {
        self.sender
            .send(msg)
            .map_err(|_| SendError::Closed)
    }

    /// Try to send a message to the actor (non-blocking, bounded).
    ///
    /// This is useful with bounded mailboxes for backpressure.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::ActorAddress;
    /// # async fn example(addr: ActorAddress<MyActor>) {
    /// match addr.try_send(MyMessage::Ping) {
    ///     Ok(()) => println!("Message sent"),
    ///     Err(e) => println!("Failed to send: {}", e),
    /// }
    /// # }
    /// ```
    pub fn try_send(&self, msg: A::Message) -> Result<(), TrySendError> {
        // Note: UnboundedSender doesn't have try_send
        // This is a placeholder for future bounded mailbox support
        self.sender
            .send(msg)
            .map_err(|_| TrySendError::Closed)
    }

    /// Get the actor ID.
    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    /// Check if the actor is still alive (mailbox open).
    pub fn is_alive(&self) -> bool {
        !self.sender.is_closed()
    }
}

impl<A: Actor> Clone for ActorAddress<A> {
    /// Clone the address (cheap operation using Arc).
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            actor_id: self.actor_id,
            _phantom: PhantomData,
        }
    }
}

impl<A: Actor> fmt::Debug for ActorAddress<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActorAddress")
            .field("actor_id", &self.actor_id)
            .field("is_alive", &self.is_alive())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::actor::Actor;
    use crate::actors::context::ActorContext;

    struct TestActor;

    impl Actor for TestActor {
        type Message = String;

        fn handle(&mut self, _msg: Self::Message, _ctx: &mut ActorContext<Self>) {}
    }

    #[test]
    fn test_actor_address_clone() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let addr = ActorAddress::<TestActor>::new(tx, ActorId::new());
        
        let addr2 = addr.clone();
        assert_eq!(addr.actor_id(), addr2.actor_id());
    }

    #[test]
    fn test_actor_address_send() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let addr = ActorAddress::<TestActor>::new(tx, ActorId::new());
        
        addr.send("test".to_string()).unwrap();
        
        let msg = rx.try_recv().unwrap();
        assert_eq!(msg, "test");
    }

    #[test]
    fn test_actor_address_send_after_drop() {
        let (tx, rx) = mpsc::unbounded_channel();
        let addr = ActorAddress::<TestActor>::new(tx, ActorId::new());
        
        drop(rx); // Close receiver
        
        let result = addr.send("test".to_string());
        assert!(matches!(result, Err(SendError::Closed)));
    }

    #[test]
    fn test_actor_address_is_alive() {
        let (tx, rx) = mpsc::unbounded_channel();
        let addr = ActorAddress::<TestActor>::new(tx, ActorId::new());
        
        assert!(addr.is_alive());
        
        drop(rx);
        
        assert!(!addr.is_alive());
    }

    #[test]
    fn test_actor_address_debug() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let id = ActorId::new();
        let addr = ActorAddress::<TestActor>::new(tx, id);
        
        let debug_str = format!("{:?}", addr);
        assert!(debug_str.contains("ActorAddress"));
        assert!(debug_str.contains("actor_id"));
    }
}
