//! Multi-Producer Single-Consumer (MPSC) channels.
//!
//! Channels for sending values from multiple producers to a single consumer.

use thiserror::Error;

/// Errors that can occur when using MPSC channels.
#[derive(Debug, Error, Clone)]
pub enum MpscError {
    /// The receiver has been dropped.
    #[error("receiver has been dropped")]
    ReceiverDropped,

    /// The channel is full (bounded channel only).
    #[error("channel is full")]
    ChannelFull,

    /// The sender has been dropped.
    #[error("sender has been dropped")]
    SenderDropped,
}

/// Create an unbounded MPSC channel.
///
/// # Returns
///
/// A tuple of (sender, receiver).
///
/// # Examples
///
/// ```rust
/// use vela_concurrency::channels::mpsc;
///
/// let (tx, mut rx) = mpsc::unbounded::<String>();
///
/// tx.send("hello".to_string()).unwrap();
///
/// tokio::spawn(async move {
///     while let Some(msg) = rx.recv().await {
///         println!("Received: {}", msg);
///     }
/// });
/// ```
pub fn unbounded<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    (UnboundedSender(tx), UnboundedReceiver(rx))
}

/// Create a bounded MPSC channel.
///
/// # Arguments
///
/// * `capacity` - Maximum number of buffered messages
///
/// # Returns
///
/// A tuple of (sender, receiver).
///
/// # Examples
///
/// ```rust
/// use vela_concurrency::channels::mpsc;
///
/// let (tx, mut rx) = mpsc::bounded::<String>(100);
///
/// tokio::spawn(async move {
///     tx.send("hello".to_string()).await.unwrap();
/// });
///
/// tokio::spawn(async move {
///     while let Some(msg) = rx.recv().await {
///         println!("Received: {}", msg);
///     }
/// });
/// ```
pub fn bounded<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = tokio::sync::mpsc::channel(capacity);
    (Sender(tx), Receiver(rx))
}

/// Sender for unbounded MPSC channel.
#[derive(Debug)]
pub struct UnboundedSender<T>(tokio::sync::mpsc::UnboundedSender<T>);

impl<T> UnboundedSender<T> {
    /// Send a value to the channel.
    ///
    /// # Errors
    ///
    /// Returns `MpscError::ReceiverDropped` if the receiver has been dropped.
    pub fn send(&self, value: T) -> Result<(), MpscError> {
        self.0.send(value).map_err(|_| MpscError::ReceiverDropped)
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Receiver for unbounded MPSC channel.
#[derive(Debug)]
pub struct UnboundedReceiver<T>(tokio::sync::mpsc::UnboundedReceiver<T>);

impl<T> UnboundedReceiver<T> {
    /// Receive a value from the channel.
    ///
    /// Returns `None` if all senders have been dropped.
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }

    /// Try to receive a value without blocking.
    ///
    /// # Errors
    ///
    /// Returns `MpscError::SenderDropped` if all senders have been dropped.
    pub fn try_recv(&mut self) -> Result<T, MpscError> {
        match self.0.try_recv() {
            Ok(value) => Ok(value),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                Err(MpscError::ChannelFull) // No message available
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                Err(MpscError::SenderDropped)
            }
        }
    }

    /// Close the channel, preventing new messages.
    pub fn close(&mut self) {
        self.0.close()
    }
}

/// Sender for bounded MPSC channel.
#[derive(Debug)]
pub struct Sender<T>(tokio::sync::mpsc::Sender<T>);

impl<T> Sender<T> {
    /// Send a value to the channel, waiting if full.
    ///
    /// # Errors
    ///
    /// Returns `MpscError::ReceiverDropped` if the receiver has been dropped.
    pub async fn send(&self, value: T) -> Result<(), MpscError> {
        self.0.send(value).await.map_err(|_| MpscError::ReceiverDropped)
    }

    /// Try to send a value without blocking.
    ///
    /// # Errors
    ///
    /// Returns `MpscError::ChannelFull` if the channel is full.
    /// Returns `MpscError::ReceiverDropped` if the receiver has been dropped.
    pub fn try_send(&self, value: T) -> Result<(), MpscError> {
        match self.0.try_send(value) {
            Ok(()) => Ok(()),
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => Err(MpscError::ChannelFull),
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                Err(MpscError::ReceiverDropped)
            }
        }
    }

    /// Check if the receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Get the current capacity of the channel.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Receiver for bounded MPSC channel.
#[derive(Debug)]
pub struct Receiver<T>(tokio::sync::mpsc::Receiver<T>);

impl<T> Receiver<T> {
    /// Receive a value from the channel.
    ///
    /// Returns `None` if all senders have been dropped.
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }

    /// Try to receive a value without blocking.
    ///
    /// # Errors
    ///
    /// Returns `MpscError::SenderDropped` if all senders have been dropped.
    pub fn try_recv(&mut self) -> Result<T, MpscError> {
        match self.0.try_recv() {
            Ok(value) => Ok(value),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                Err(MpscError::ChannelFull) // No message available
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                Err(MpscError::SenderDropped)
            }
        }
    }

    /// Close the channel, preventing new messages.
    pub fn close(&mut self) {
        self.0.close()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_unbounded_send_recv() {
        let (tx, mut rx) = unbounded::<String>();

        tx.send("hello".to_string()).unwrap();
        tx.send("world".to_string()).unwrap();

        assert_eq!(rx.recv().await, Some("hello".to_string()));
        assert_eq!(rx.recv().await, Some("world".to_string()));
    }

    #[tokio::test]
    async fn test_unbounded_multiple_senders() {
        let (tx, mut rx) = unbounded::<u32>();

        let tx1 = tx.clone();
        let tx2 = tx.clone();

        tokio::spawn(async move { tx1.send(1).unwrap() });
        tokio::spawn(async move { tx2.send(2).unwrap() });

        drop(tx);

        let mut values = vec![];
        while let Some(v) = rx.recv().await {
            values.push(v);
        }

        values.sort();
        assert_eq!(values, vec![1, 2]);
    }

    #[tokio::test]
    async fn test_unbounded_receiver_dropped() {
        let (tx, rx) = unbounded::<String>();

        drop(rx);
        sleep(Duration::from_millis(10)).await;

        assert!(tx.is_closed());
        assert!(matches!(
            tx.send("test".to_string()),
            Err(MpscError::ReceiverDropped)
        ));
    }

    #[tokio::test]
    async fn test_bounded_send_recv() {
        let (tx, mut rx) = bounded::<String>(10);

        tx.send("hello".to_string()).await.unwrap();
        tx.send("world".to_string()).await.unwrap();

        assert_eq!(rx.recv().await, Some("hello".to_string()));
        assert_eq!(rx.recv().await, Some("world".to_string()));
    }

    #[tokio::test]
    async fn test_bounded_backpressure() {
        let (tx, mut rx) = bounded::<u32>(2);

        // Fill the channel
        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();

        // Next try_send should fail
        assert!(matches!(tx.try_send(3), Err(MpscError::ChannelFull)));

        // Receive one message, making space
        assert_eq!(rx.recv().await, Some(1));

        // Now should succeed
        tx.try_send(3).unwrap();
    }

    #[tokio::test]
    async fn test_bounded_receiver_dropped() {
        let (tx, rx) = bounded::<String>(10);

        drop(rx);
        sleep(Duration::from_millis(10)).await;

        assert!(tx.is_closed());
        assert!(matches!(
            tx.send("test".to_string()).await,
            Err(MpscError::ReceiverDropped)
        ));
    }

    #[tokio::test]
    async fn test_receiver_close() {
        let (tx, mut rx) = unbounded::<String>();

        rx.close();

        assert!(tx.is_closed());
        assert!(matches!(
            tx.send("test".to_string()),
            Err(MpscError::ReceiverDropped)
        ));
    }
}
