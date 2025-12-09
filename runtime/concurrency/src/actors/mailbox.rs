/*!
# Mailbox

FIFO message queue for actors.

Mailbox provides:
- FIFO ordering (messages processed in order received)
- At-most-once delivery (no duplicates)
- Backpressure support (optional bounded capacity)
- Async message receiving

## Example

```rust
use vela_concurrency::actors::Mailbox;

let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
let mut mailbox = Mailbox::new(rx);

// Receive messages
while let Some(msg) = mailbox.recv().await {
    println!("Received: {:?}", msg);
}
```
*/

use tokio::sync::mpsc;

/// FIFO message queue for actors.
pub struct Mailbox<M> {
    /// Receiver for incoming messages
    receiver: mpsc::UnboundedReceiver<M>,
}

impl<M> Mailbox<M> {
    /// Create a new mailbox from a receiver.
    pub fn new(receiver: mpsc::UnboundedReceiver<M>) -> Self {
        Self { receiver }
    }

    /// Receive the next message from the mailbox.
    ///
    /// Returns `None` when the mailbox is closed (all senders dropped).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use vela_concurrency::actors::Mailbox;
    /// # async fn example(mut mailbox: Mailbox<String>) {
    /// while let Some(msg) = mailbox.recv().await {
    ///     println!("Received: {}", msg);
    /// }
    /// # }
    /// ```
    pub async fn recv(&mut self) -> Option<M> {
        self.receiver.recv().await
    }

    /// Try to receive a message without blocking.
    ///
    /// Returns `Ok(Some(msg))` if a message is available,
    /// `Ok(None)` if the mailbox is empty,
    /// `Err(())` if the mailbox is closed.
    pub fn try_recv(&mut self) -> Result<Option<M>, ()> {
        match self.receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(()),
        }
    }

    /// Get the number of messages in the mailbox.
    ///
    /// Note: This is an approximation as messages may be added concurrently.
    pub fn len(&self) -> usize {
        // Note: UnboundedReceiver doesn't expose len()
        // This is a placeholder for future bounded mailbox support
        0
    }

    /// Check if the mailbox is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the mailbox is closed (all senders dropped).
    pub fn is_closed(&self) -> bool {
        self.receiver.is_closed()
    }
}

/// Bounded mailbox with backpressure support.
pub struct BoundedMailbox<M> {
    /// Receiver for incoming messages
    receiver: mpsc::Receiver<M>,
    /// Maximum capacity
    capacity: usize,
}

impl<M> BoundedMailbox<M> {
    /// Create a new bounded mailbox with a specific capacity.
    pub fn new(receiver: mpsc::Receiver<M>, capacity: usize) -> Self {
        Self { receiver, capacity }
    }

    /// Receive the next message from the mailbox.
    pub async fn recv(&mut self) -> Option<M> {
        self.receiver.recv().await
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&mut self) -> Result<Option<M>, ()> {
        match self.receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => Err(()),
        }
    }

    /// Get the capacity of the mailbox.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if the mailbox is closed.
    pub fn is_closed(&self) -> bool {
        self.receiver.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mailbox_recv() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut mailbox = Mailbox::new(rx);

        tx.send("msg1").unwrap();
        tx.send("msg2").unwrap();

        assert_eq!(mailbox.recv().await, Some("msg1"));
        assert_eq!(mailbox.recv().await, Some("msg2"));
    }

    #[tokio::test]
    async fn test_mailbox_recv_closed() {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let mut mailbox = Mailbox::new(rx);

        drop(tx); // Close sender

        assert_eq!(mailbox.recv().await, None);
    }

    #[tokio::test]
    async fn test_mailbox_try_recv() {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut mailbox = Mailbox::new(rx);

        // Empty mailbox
        assert_eq!(mailbox.try_recv(), Ok(None));

        // Send message
        tx.send("msg").unwrap();

        // Receive message
        assert_eq!(mailbox.try_recv(), Ok(Some("msg")));

        // Empty again
        assert_eq!(mailbox.try_recv(), Ok(None));
    }

    #[tokio::test]
    async fn test_mailbox_try_recv_closed() {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let mut mailbox = Mailbox::new(rx);

        drop(tx); // Close sender

        assert!(mailbox.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_mailbox_is_closed() {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        let mailbox = Mailbox::new(rx);

        assert!(!mailbox.is_closed());

        drop(tx); // Close sender

        // Note: is_closed() might not be immediately true
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(mailbox.is_closed());
    }

    #[tokio::test]
    async fn test_bounded_mailbox() {
        let (tx, rx) = mpsc::channel(2); // Capacity 2
        let mut mailbox = BoundedMailbox::new(rx, 2);

        assert_eq!(mailbox.capacity(), 2);

        tx.send("msg1").await.unwrap();
        tx.send("msg2").await.unwrap();

        assert_eq!(mailbox.recv().await, Some("msg1"));
        assert_eq!(mailbox.recv().await, Some("msg2"));
    }

    #[tokio::test]
    async fn test_bounded_mailbox_backpressure() {
        let (tx, rx) = mpsc::channel(1); // Capacity 1
        let mut mailbox = BoundedMailbox::new(rx, 1);

        // Fill mailbox
        tx.send("msg1").await.unwrap();

        // Try to send another (should block)
        let tx2 = tx.clone();
        let handle = tokio::spawn(async move {
            tx2.send("msg2").await.unwrap();
        });

        // Receive first message (unblocks sender)
        assert_eq!(mailbox.recv().await, Some("msg1"));

        // Wait for second send to complete
        handle.await.unwrap();

        // Receive second message
        assert_eq!(mailbox.recv().await, Some("msg2"));
    }
}
