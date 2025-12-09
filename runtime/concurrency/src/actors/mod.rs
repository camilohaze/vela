/*!
# Actors Module

Actor model implementation for Vela concurrency system.

This module provides:
- `Actor` trait for defining actor behavior
- `ActorAddress` for type-safe message sending
- `ActorContext` for actor lifecycle and spawning
- `Mailbox` for FIFO message queue
- `Supervisor` for fault tolerance

## Example

```rust
use vela_concurrency::actors::{Actor, ActorContext, ActorAddress};

// Define actor
struct Counter {
    count: u32,
}

enum CounterMsg {
    Add(u32),
    Get(tokio::sync::oneshot::Sender<u32>),
}

impl Actor for Counter {
    type Message = CounterMsg;
    
    fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
        match msg {
            CounterMsg::Add(n) => self.count += n,
            CounterMsg::Get(tx) => { let _ = tx.send(self.count); }
        }
    }
}

// Use actor
#[tokio::main]
async fn main() {
    let counter = Counter { count: 0 };
    let addr = spawn_actor(counter);
    
    addr.send(CounterMsg::Add(5)).unwrap();
    
    let (tx, rx) = tokio::sync::oneshot::channel();
    addr.send(CounterMsg::Get(tx)).unwrap();
    
    let count = rx.await.unwrap();
    println!("Count: {}", count); // 5
}
```
*/

mod actor;
mod address;
mod context;
mod mailbox;
mod supervisor;

pub use actor::{Actor, ActorId, ActorState};
pub use address::{ActorAddress, SendError, TrySendError};
pub use context::ActorContext;
pub use mailbox::Mailbox;
pub use supervisor::{Supervisor, SupervisionStrategy};

/// Spawn an actor and return its address.
///
/// This is a convenience function for starting actors.
///
/// # Example
///
/// ```rust,no_run
/// # use vela_concurrency::actors::{Actor, ActorContext, spawn_actor};
/// # struct MyActor;
/// # impl Actor for MyActor { type Message = (); fn handle(&mut self, _: (), _: &mut ActorContext<Self>) {} }
/// let actor = MyActor::new();
/// let addr = spawn_actor(actor);
/// ```
pub fn spawn_actor<A: Actor>(actor: A) -> ActorAddress<A> {
    let actor_id = ActorId::new();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let address = ActorAddress::new(tx.clone(), actor_id);
    
    // Spawn actor runtime
    tokio::spawn(async move {
        let mut actor = actor;
        let mut mailbox = Mailbox::new(rx);
        let mut ctx = ActorContext::new(ActorAddress::new(tx, actor_id), actor_id);
        
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    struct TestActor {
        counter: u32,
    }

    enum TestMessage {
        Increment,
        GetCount(oneshot::Sender<u32>),
        Stop,
    }

    impl Actor for TestActor {
        type Message = TestMessage;

        fn handle(&mut self, msg: Self::Message, ctx: &mut ActorContext<Self>) {
            match msg {
                TestMessage::Increment => {
                    self.counter += 1;
                }
                TestMessage::GetCount(tx) => {
                    let _ = tx.send(self.counter);
                }
                TestMessage::Stop => {
                    ctx.stop();
                }
            }
        }
    }

    #[tokio::test]
    async fn test_spawn_actor_basic() {
        let actor = TestActor { counter: 0 };
        let addr = spawn_actor(actor);
        
        // Send increment
        addr.send(TestMessage::Increment).unwrap();
        
        // Get count
        let (tx, rx) = oneshot::channel();
        addr.send(TestMessage::GetCount(tx)).unwrap();
        
        let count = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            rx
        ).await.unwrap().unwrap();
        
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_spawn_actor_multiple_messages() {
        let actor = TestActor { counter: 0 };
        let addr = spawn_actor(actor);
        
        // Send 10 increments
        for _ in 0..10 {
            addr.send(TestMessage::Increment).unwrap();
        }
        
        // Wait a bit for processing
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Get count
        let (tx, rx) = oneshot::channel();
        addr.send(TestMessage::GetCount(tx)).unwrap();
        
        let count = rx.await.unwrap();
        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn test_spawn_actor_stop() {
        let actor = TestActor { counter: 0 };
        let addr = spawn_actor(actor);
        
        // Increment
        addr.send(TestMessage::Increment).unwrap();
        
        // Stop
        addr.send(TestMessage::Stop).unwrap();
        
        // Wait for actor to stop
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Verify actor is stopped
        assert!(!addr.is_alive());
    }

    #[tokio::test]
    async fn test_spawn_actor_address_clone() {
        let actor = TestActor { counter: 0 };
        let addr1 = spawn_actor(actor);
        let addr2 = addr1.clone();
        
        // Send from both addresses
        addr1.send(TestMessage::Increment).unwrap();
        addr2.send(TestMessage::Increment).unwrap();
        
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Get count
        let (tx, rx) = oneshot::channel();
        addr1.send(TestMessage::GetCount(tx)).unwrap();
        
        let count = rx.await.unwrap();
        assert_eq!(count, 2);
    }
}
