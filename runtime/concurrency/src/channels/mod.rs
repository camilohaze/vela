//! Channel implementations for inter-task communication.
//!
//! This module provides type-safe channels for different communication patterns:
//!
//! - [`mpsc`]: Multi-Producer Single-Consumer channels
//!
//! # Examples
//!
//! ## MPSC Unbounded
//!
//! ```rust
//! use vela_concurrency::channels::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, mut rx) = mpsc::unbounded::<String>();
//!
//!     tx.send("hello".to_string()).unwrap();
//!
//!     assert_eq!(rx.recv().await, Some("hello".to_string()));
//! }
//! ```
//!
//! ## MPSC Bounded (with backpressure)
//!
//! ```rust
//! use vela_concurrency::channels::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, mut rx) = mpsc::bounded::<String>(10);
//!
//!     tx.send("hello".to_string()).await.unwrap();
//!
//!     assert_eq!(rx.recv().await, Some("hello".to_string()));
//! }
//! ```

pub mod mpsc;

pub use mpsc::{MpscError};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mpsc_unbounded_basic() {
        let (tx, mut rx) = mpsc::unbounded::<u32>();

        tx.send(42).unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }

    #[tokio::test]
    async fn test_mpsc_bounded_basic() {
        let (tx, mut rx) = mpsc::bounded::<u32>(10);

        tx.send(42).await.unwrap();
        assert_eq!(rx.recv().await, Some(42));
    }

    #[tokio::test]
    async fn test_mpsc_multiple_producers() {
        let (tx, mut rx) = mpsc::unbounded::<u32>();

        let tx1 = tx.clone();
        let tx2 = tx.clone();
        let tx3 = tx.clone();

        tokio::spawn(async move { tx1.send(1).unwrap() });
        tokio::spawn(async move { tx2.send(2).unwrap() });
        tokio::spawn(async move { tx3.send(3).unwrap() });

        drop(tx); // Drop original sender

        let mut values = vec![];
        while let Some(v) = rx.recv().await {
            values.push(v);
        }

        values.sort();
        assert_eq!(values, vec![1, 2, 3]);
    }
}
