/*!
# Actor Context

Context provided to actors during message handling.

ActorContext provides:
- Actor's own address (for self-messaging)
- Ability to spawn child actors
- Ability to stop the actor
- Access to supervisor (if any)

## Example

```rust
impl Actor for MyActor {
    type Message = MyMessage;
    
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
        match msg {
            MyMessage::SpawnChild => {
                let child = ChildActor::new();
                let child_addr = ctx.spawn(child);
                // Store child_addr for later use
            }
            MyMessage::Stop => {
                ctx.stop();
            }
        }
    }
}
```
*/

use super::actor::{Actor, ActorId};
use super::address::ActorAddress;
use std::marker::PhantomData;

/// Context provided to actors during message handling.
pub struct ActorContext<A: Actor> {
    /// Actor's own address
    address: ActorAddress<A>,
    /// Actor ID
    actor_id: ActorId,
    /// Whether actor should stop after current message
    should_stop: bool,
    /// PhantomData for generic type
    _phantom: PhantomData<A>,
}

impl<A: Actor> ActorContext<A> {
    /// Create a new actor context.
    pub(crate) fn new(address: ActorAddress<A>, actor_id: ActorId) -> Self {
        Self {
            address,
            actor_id,
            should_stop: false,
            _phantom: PhantomData,
        }
    }

    /// Get the actor's own address.
    ///
    /// Useful for self-messaging or passing address to other actors.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::{Actor, ActorContext};
    /// # impl Actor for MyActor {
    /// #     type Message = ();
    /// #     fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
    /// let self_addr = ctx.address();
    /// // Pass self_addr to another actor
    /// #     }
    /// # }
    /// ```
    pub fn address(&self) -> ActorAddress<A> {
        self.address.clone()
    }

    /// Get the actor ID.
    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }

    /// Stop the actor after the current message is processed.
    ///
    /// The actor's `stopped()` lifecycle hook will be called.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::{Actor, ActorContext};
    /// # impl Actor for MyActor {
    /// #     type Message = ();
    /// #     fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
    /// ctx.stop(); // Stop actor gracefully
    /// #     }
    /// # }
    /// ```
    pub fn stop(&mut self) {
        self.should_stop = true;
    }

    /// Check if actor should stop.
    pub(crate) fn should_stop(&self) -> bool {
        self.should_stop
    }

    /// Spawn a child actor.
    ///
    /// Returns the address of the spawned actor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::{Actor, ActorContext};
    /// # struct ChildActor;
    /// # impl ChildActor { fn new() -> Self { ChildActor } }
    /// # impl Actor for ChildActor { type Message = (); fn handle(&mut self, _: (), _: &mut ActorContext<Self>) {} }
    /// # impl Actor for MyActor {
    /// #     type Message = ();
    /// #     fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
    /// let child = ChildActor::new();
    /// let child_addr = ctx.spawn(child);
    /// #     }
    /// # }
    /// ```
    pub fn spawn<C: Actor>(&mut self, actor: C) -> ActorAddress<C> {
        // Create actor runtime
        let actor_id = ActorId::new();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let address = ActorAddress::new(tx, actor_id);
        
        // Spawn actor task
        tokio::spawn(async move {
            ActorRuntime::new(actor, rx, actor_id).run().await;
        });
        
        address
    }
}

/// Internal actor runtime that processes messages.
struct ActorRuntime<A: Actor> {
    actor: A,
    mailbox: tokio::sync::mpsc::UnboundedReceiver<A::Message>,
    actor_id: ActorId,
}

impl<A: Actor> ActorRuntime<A> {
    fn new(
        actor: A,
        mailbox: tokio::sync::mpsc::UnboundedReceiver<A::Message>,
        actor_id: ActorId,
    ) -> Self {
        Self {
            actor,
            mailbox,
            actor_id,
        }
    }

    async fn run(mut self) {
        // Create context
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let address = ActorAddress::new(tx, self.actor_id);
        let mut ctx = ActorContext::new(address, self.actor_id);
        
        // Call started hook
        self.actor.started(&mut ctx);
        
        // Process messages
        while let Some(msg) = self.mailbox.recv().await {
            self.actor.handle(msg, &mut ctx);
            
            if ctx.should_stop() {
                break;
            }
        }
        
        // Call stopped hook
        self.actor.stopped();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::actor::{Actor, ActorId};

    struct TestActor {
        stopped_called: bool,
    }

    impl Actor for TestActor {
        type Message = String;

        fn handle(&mut self, _msg: Self::Message, _ctx: &mut ActorContext<Self>) {}

        fn stopped(&mut self) {
            self.stopped_called = true;
        }
    }

    #[test]
    fn test_context_actor_id() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let id = ActorId::new();
        let addr = ActorAddress::new(tx, id);
        let ctx = ActorContext::<TestActor>::new(addr, id);
        
        assert_eq!(ctx.actor_id(), id);
    }

    #[test]
    fn test_context_stop() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let id = ActorId::new();
        let addr = ActorAddress::new(tx, id);
        let mut ctx = ActorContext::<TestActor>::new(addr, id);
        
        assert!(!ctx.should_stop());
        
        ctx.stop();
        
        assert!(ctx.should_stop());
    }

    #[test]
    fn test_context_address() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let id = ActorId::new();
        let addr = ActorAddress::new(tx, id);
        let ctx = ActorContext::<TestActor>::new(addr.clone(), id);
        
        let addr2 = ctx.address();
        assert_eq!(addr.actor_id(), addr2.actor_id());
    }

    #[tokio::test]
    async fn test_actor_runtime_lifecycle() {
        struct LifecycleActor {
            started: bool,
            stopped: bool,
        }

        impl Actor for LifecycleActor {
            type Message = ();

            fn handle(&mut self, _: Self::Message, ctx: &mut ActorContext<Self>) {
                ctx.stop();
            }

            fn started(&mut self, _: &mut ActorContext<Self>) {
                self.started = true;
            }

            fn stopped(&mut self) {
                self.stopped = true;
            }
        }

        let actor = LifecycleActor {
            started: false,
            stopped: false,
        };

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let id = ActorId::new();

        let runtime = ActorRuntime::new(actor, rx, id);

        // Send a message to trigger stop
        tx.send(()).unwrap();
        drop(tx);

        runtime.run().await;
    }
}
