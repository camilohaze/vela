//! # Vela Channels Module
//!
//! Asynchronous channels for inter-task communication in the Vela runtime.
//! Based on Tokio's mpsc channels with additional utilities.
//!
//! This module provides:
//! - `VelaChannel<T>`: Main channel with sender/receiver pair
//! - `VelaSender<T>`: Multi-producer sender
//! - `VelaReceiver<T>`: Single-consumer receiver
//! - Utilities for timeouts, selection, and fan-out operations

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use thiserror::Error;
use tracing::{error};

/// Errors that can occur in channel operations
#[derive(Debug, Error)]
pub enum ChannelError {
    #[error("Channel is closed")]
    Closed,

    #[error("Channel is full")]
    Full,

    #[error("Send operation timed out")]
    SendTimeout,

    #[error("Receive operation timed out")]
    RecvTimeout,

    #[error("Try send failed: {0}")]
    TrySendError(String),

    #[error("Try receive failed: {0}")]
    TryRecvError(String),
}

/// Result type for channel operations
pub type ChannelResult<T> = Result<T, ChannelError>;

/// A Vela channel for asynchronous communication between tasks
///
/// This is a multi-producer, single-consumer channel based on Tokio's mpsc.
/// It provides both bounded and unbounded variants.
pub struct VelaChannel<T> {
    sender: VelaSender<T>,
    receiver: VelaReceiver<T>,
}

impl<T> VelaChannel<T>
where
    T: Send + 'static,
{
    /// Create a new bounded channel with the specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of messages that can be buffered
    ///
    /// # Returns
    /// A new `VelaChannel` with bounded capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            sender: VelaSender::new(tx),
            receiver: VelaReceiver::new(rx),
        }
    }

    /// Create a new unbounded channel
    ///
    /// # Returns
    /// A new `VelaChannel` with unlimited capacity
    pub fn unbounded() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            sender: VelaSender::new_unbounded(tx),
            receiver: VelaReceiver::new_unbounded(rx),
        }
    }

    /// Split the channel into sender and receiver parts
    ///
    /// # Returns
    /// A tuple of `(VelaSender<T>, VelaReceiver<T>)`
    pub fn split(self) -> (VelaSender<T>, VelaReceiver<T>) {
        (self.sender, self.receiver)
    }

    /// Get a reference to the sender
    pub fn sender(&self) -> &VelaSender<T> {
        &self.sender
    }

    /// Get a mutable reference to the receiver
    pub fn receiver(&mut self) -> &mut VelaReceiver<T> {
        &mut self.receiver
    }
}

/// Sender half of a Vela channel
///
/// Can be cloned to allow multiple producers to send messages to the same channel.
pub struct VelaSender<T> {
    inner: SenderImpl<T>,
}

enum SenderImpl<T> {
    Bounded(mpsc::Sender<T>),
    Unbounded(mpsc::UnboundedSender<T>),
}

impl<T> VelaSender<T> {
    fn new(tx: mpsc::Sender<T>) -> Self {
        Self {
            inner: SenderImpl::Bounded(tx),
        }
    }

    fn new_unbounded(tx: mpsc::UnboundedSender<T>) -> Self {
        Self {
            inner: SenderImpl::Unbounded(tx),
        }
    }

    /// Send a message asynchronously
    ///
    /// # Arguments
    /// * `value` - The value to send
    ///
    /// # Returns
    /// `Ok(())` if the message was sent, `Err(ChannelError::Closed)` if the receiver is closed
    pub async fn send(&self, value: T) -> ChannelResult<()> {
        match &self.inner {
            SenderImpl::Bounded(tx) => {
                tx.send(value).await.map_err(|_| ChannelError::Closed)?;
            }
            SenderImpl::Unbounded(tx) => {
                tx.send(value).map_err(|_| ChannelError::Closed)?;
            }
        }
        Ok(())
    }

    /// Try to send a message without blocking
    ///
    /// # Arguments
    /// * `value` - The value to send
    ///
    /// # Returns
    /// `Ok(())` if sent successfully, `Err` if the channel is full or closed
    pub fn try_send(&self, value: T) -> ChannelResult<()> {
        match &self.inner {
            SenderImpl::Bounded(tx) => {
                tx.try_send(value).map_err(|e| match e {
                    mpsc::error::TrySendError::Full(_) => ChannelError::Full,
                    mpsc::error::TrySendError::Closed(_) => ChannelError::Closed,
                })?;
            }
            SenderImpl::Unbounded(tx) => {
                tx.send(value).map_err(|_| ChannelError::Closed)?;
            }
        }
        Ok(())
    }

    /// Check if the channel is closed
    ///
    /// # Returns
    /// `true` if the receiver has been dropped, `false` otherwise
    pub fn is_closed(&self) -> bool {
        match &self.inner {
            SenderImpl::Bounded(tx) => tx.is_closed(),
            SenderImpl::Unbounded(tx) => tx.is_closed(),
        }
    }

    /// Clone the sender to allow multiple producers
    pub fn clone(&self) -> Self {
        match &self.inner {
            SenderImpl::Bounded(tx) => Self::new(tx.clone()),
            SenderImpl::Unbounded(tx) => Self::new_unbounded(tx.clone()),
        }
    }
}

impl<T> Clone for VelaSender<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

/// Receiver half of a Vela channel
///
/// Only one receiver can exist for each channel.
pub struct VelaReceiver<T> {
    inner: ReceiverImpl<T>,
}

enum ReceiverImpl<T> {
    Bounded(mpsc::Receiver<T>),
    Unbounded(mpsc::UnboundedReceiver<T>),
}

impl<T> VelaReceiver<T> {
    fn new(rx: mpsc::Receiver<T>) -> Self {
        Self {
            inner: ReceiverImpl::Bounded(rx),
        }
    }

    fn new_unbounded(rx: mpsc::UnboundedReceiver<T>) -> Self {
        Self {
            inner: ReceiverImpl::Unbounded(rx),
        }
    }

    /// Receive a message asynchronously
    ///
    /// # Returns
    /// `Some(value)` if a message was received, `None` if the channel is closed
    pub async fn recv(&mut self) -> Option<T> {
        match &mut self.inner {
            ReceiverImpl::Bounded(rx) => rx.recv().await,
            ReceiverImpl::Unbounded(rx) => rx.recv().await,
        }
    }

    /// Try to receive a message without blocking
    ///
    /// # Returns
    /// `Ok(Some(value))` if a message was received,
    /// `Ok(None)` if no message is available,
    /// `Err` if the channel is closed
    pub fn try_recv(&mut self) -> Result<Option<T>, ChannelError> {
        match &mut self.inner {
            ReceiverImpl::Bounded(rx) => match rx.try_recv() {
                Ok(value) => Ok(Some(value)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(ChannelError::Closed),
            },
            ReceiverImpl::Unbounded(rx) => match rx.try_recv() {
                Ok(value) => Ok(Some(value)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(ChannelError::Closed),
            },
        }
    }

    /// Close the receiver
    ///
    /// This will cause any pending or future send operations to fail.
    pub fn close(&mut self) {
        match &mut self.inner {
            ReceiverImpl::Bounded(rx) => rx.close(),
            ReceiverImpl::Unbounded(rx) => rx.close(),
        }
    }
}

/// Channel utilities for common operations
pub mod utils {
    use super::*;

    /// Send a message with a timeout
    ///
    /// # Arguments
    /// * `sender` - The sender to use
    /// * `value` - The value to send
    /// * `timeout_duration` - How long to wait before timing out
    ///
    /// # Returns
    /// `Ok(())` if sent successfully, `Err(ChannelError::SendTimeout)` if timed out
    pub async fn send_with_timeout<T>(
        sender: &VelaSender<T>,
        value: T,
        timeout_duration: Duration,
    ) -> ChannelResult<()> {
        match timeout(timeout_duration, sender.send(value)).await {
            Ok(result) => result,
            Err(_) => Err(ChannelError::SendTimeout),
        }
    }

    /// Receive a message with a timeout
    ///
    /// # Arguments
    /// * `receiver` - The receiver to use
    /// * `timeout_duration` - How long to wait before timing out
    ///
    /// # Returns
    /// `Ok(Some(value))` if received, `Ok(None)` if channel closed, `Err` if timed out
    pub async fn recv_with_timeout<T>(
        receiver: &mut VelaReceiver<T>,
        timeout_duration: Duration,
    ) -> Result<Option<T>, ChannelError> {
        match timeout(timeout_duration, receiver.recv()).await {
            Ok(value) => Ok(value),
            Err(_) => Err(ChannelError::RecvTimeout),
        }
    }

    /// Select the first message from multiple receivers
    ///
    /// # Arguments
    /// * `receivers` - Vector of mutable references to receivers
    ///
    /// # Returns
    /// `(index, value)` where index is the position in the vector and value is the received message
    pub async fn select_first<T>(
        mut receivers: Vec<&mut VelaReceiver<T>>,
    ) -> (usize, T) {
        // Simple implementation - check receivers in order until one has a message
        // In a real implementation, you'd want to use tokio::select! or futures::select_all
        loop {
            for (i, rx) in receivers.iter_mut().enumerate() {
                if let Ok(Some(value)) = rx.try_recv() {
                    return (i, value);
                }
            }
            // If no message available, yield to allow other tasks to run
            tokio::task::yield_now().await;
        }
    }

    /// Create multiple senders that broadcast to the same receiver
    ///
    /// # Arguments
    /// * `capacity` - Capacity for the channel (0 for unbounded)
    /// * `num_senders` - Number of senders to create
    ///
    /// # Returns
    /// Vector of senders and a single receiver
    pub fn fan_out<T>(
        capacity: usize,
        num_senders: usize,
    ) -> (Vec<VelaSender<T>>, VelaReceiver<T>) {
        let (tx, rx) = if capacity == 0 {
            let (tx, rx) = mpsc::unbounded_channel();
            (VelaSender::new_unbounded(tx), VelaReceiver::new_unbounded(rx))
        } else {
            let (tx, rx) = mpsc::channel(capacity);
            (VelaSender::new(tx), VelaReceiver::new(rx))
        };

        let mut senders = Vec::with_capacity(num_senders);
        for _ in 0..num_senders {
            senders.push(tx.clone());
        }

        (senders, rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_basic_send_recv() {
        let (tx, mut rx) = VelaChannel::unbounded().split();

        tx.send(42).await.unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }

    #[tokio::test]
    async fn test_bounded_channel() {
        let (tx, mut rx) = VelaChannel::new(1).split();

        // Fill channel
        tx.send(1).await.unwrap();

        // Try to send another message - should work since we receive immediately
        let send_future = tx.send(2);
        let recv_future = rx.recv();

        tokio::select! {
            _ = send_future => panic!("Should not be able to send to full channel"),
            Some(val) = recv_future => assert_eq!(val, 1),
        }
    }

    #[tokio::test]
    async fn test_multi_producer() {
        let (tx1, mut rx) = VelaChannel::unbounded().split();
        let tx2 = tx1.clone();

        let handle1 = tokio::spawn(async move {
            tx1.send("hello").await.unwrap();
        });

        let handle2 = tokio::spawn(async move {
            tx2.send("world").await.unwrap();
        });

        let mut messages = Vec::new();
        messages.push(rx.recv().await.unwrap());
        messages.push(rx.recv().await.unwrap());

        // Should receive both messages (order not guaranteed)
        assert_eq!(messages.len(), 2);
        assert!(messages.contains(&"hello"));
        assert!(messages.contains(&"world"));

        let _ = handle1.await;
        let _ = handle2.await;
    }

    #[tokio::test]
    async fn test_channel_close() {
        let (tx, mut rx) = VelaChannel::<i32>::unbounded().split();

        drop(tx); // Close sender

        assert_eq!(rx.recv().await, None);
    }

    #[tokio::test]
    async fn test_try_recv() {
        let (tx, mut rx) = VelaChannel::<i32>::unbounded().split();

        // No message available
        assert_eq!(rx.try_recv().unwrap(), None);

        tx.send(42).await.unwrap();

        // Message available
        assert_eq!(rx.try_recv().unwrap(), Some(42));
    }

    #[tokio::test]
    async fn test_send_with_timeout() {
        let (tx, mut rx) = VelaChannel::<i32>::new(0).split(); // Zero capacity

        // This should timeout since channel is full and no receiver
        let result = utils::send_with_timeout(&tx, 42, Duration::from_millis(10)).await;
        assert!(matches!(result, Err(ChannelError::SendTimeout)));
    }

    #[tokio::test]
    async fn test_recv_with_timeout() {
        let (_tx, mut rx) = VelaChannel::<i32>::unbounded().split();

        // This should timeout since no message is sent
        let result = utils::recv_with_timeout(&mut rx, Duration::from_millis(10)).await;
        assert!(matches!(result, Err(ChannelError::RecvTimeout)));
    }
}