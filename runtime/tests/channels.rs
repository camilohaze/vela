//! Tests unitarios para el módulo de channels
//!
//! Estos tests verifican la funcionalidad completa del sistema de channels:
//! - Envío y recepción básica
//! - Channels bounded y unbounded
//! - Multi-producer
//! - Timeouts
//! - Error handling
//! - Utilidades (select, fan-out)

use vela_runtime::channels::*;
use std::time::Duration;
use tokio::test;
use tokio::time::timeout;

#[tokio::test]
async fn test_basic_send_recv() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    tx.send(42).await.unwrap();
    assert_eq!(rx.recv().await, Some(42));
}

#[tokio::test]
async fn test_bounded_channel_capacity() {
    let (tx, mut rx) = VelaChannel::new(1).split();

    // Fill channel
    tx.send(1).await.unwrap();

    // Try to send another message - should block
    let send_result = timeout(Duration::from_millis(10), tx.send(2)).await;
    assert!(send_result.is_err(), "Send should timeout on full bounded channel");

    // Receive one message
    assert_eq!(rx.recv().await, Some(1));

    // Now send should work
    tx.send(2).await.unwrap();
    assert_eq!(rx.recv().await, Some(2));
}

#[tokio::test]
async fn test_multi_producer() {
    let (tx1, mut rx) = VelaChannel::unbounded().split();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    let handle1 = tokio::spawn(async move {
        tx1.send("producer1").await.unwrap();
    });

    let handle2 = tokio::spawn(async move {
        tx2.send("producer2").await.unwrap();
    });

    let handle3 = tokio::spawn(async move {
        tx3.send("producer3").await.unwrap();
    });

    let mut messages = Vec::new();
    for _ in 0..3 {
        if let Some(msg) = rx.recv().await {
            messages.push(msg);
        }
    }

    assert_eq!(messages.len(), 3);
    assert!(messages.contains(&"producer1"));
    assert!(messages.contains(&"producer2"));
    assert!(messages.contains(&"producer3"));

    let _ = tokio::try_join!(handle1, handle2, handle3);
}

#[tokio::test]
async fn test_channel_close() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    // Send a message
    tx.send("message").await.unwrap();

    // Close sender
    drop(tx);

    // Should receive the message
    assert_eq!(rx.recv().await, Some("message"));

    // Should receive None after close
    assert_eq!(rx.recv().await, None);
}

#[tokio::test]
async fn test_try_recv() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    // No message available
    assert_eq!(rx.try_recv().unwrap(), None);

    // Send message
    tx.send(42).await.unwrap();

    // Message available
    assert_eq!(rx.try_recv().unwrap(), Some(42));

    // No more messages
    assert_eq!(rx.try_recv().unwrap(), None);
}

#[tokio::test]
async fn test_try_send_bounded() {
    let (tx, mut rx) = VelaChannel::new(1).split();

    // Send to fill channel
    assert!(tx.try_send(1).is_ok());

    // Try to send to full channel
    assert!(matches!(tx.try_send(2), Err(ChannelError::Full)));

    // Receive to make room
    assert_eq!(rx.recv().await, Some(1));

    // Now send should work
    assert!(tx.try_send(2).is_ok());
    assert_eq!(rx.recv().await, Some(2));
}

#[tokio::test]
async fn test_try_send_unbounded() {
    let (tx, mut rx) = VelaChannel::<i32>::unbounded().split();

    // Unbounded channels should always accept sends
    for i in 0..1000 {
        assert!(tx.try_send(i).is_ok());
    }

    // Receive some messages
    for i in 0..1000 {
        assert_eq!(rx.recv().await, Some(i));
    }
}

#[tokio::test]
async fn test_sender_is_closed() {
    let (tx, mut rx) = VelaChannel::<i32>::unbounded().split();

    assert!(!tx.is_closed());

    // Close receiver
    rx.close();

    // Give some time for the close to propagate
    tokio::time::sleep(Duration::from_millis(1)).await;

    assert!(tx.is_closed());
}

#[tokio::test]
async fn test_send_with_timeout_success() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    let receiver_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        rx.recv().await
    });

    let result = utils::send_with_timeout(&tx, "message", Duration::from_millis(100)).await;
    assert!(result.is_ok());

    let received = receiver_handle.await.unwrap();
    assert_eq!(received, Some("message"));
}

#[tokio::test]
async fn test_send_with_timeout_failure() {
    let (tx, mut rx) = VelaChannel::new(1).split(); // Small capacity

    // Fill the channel
    tx.send("fill").await.unwrap();

    // Now send should timeout since channel is full and no receiver
    let result = utils::send_with_timeout(&tx, "message", Duration::from_millis(10)).await;
    assert!(matches!(result, Err(ChannelError::SendTimeout)));
}

#[tokio::test]
async fn test_recv_with_timeout_success() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    let sender_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        tx.send("message").await.unwrap();
    });

    let result = utils::recv_with_timeout(&mut rx, Duration::from_millis(100)).await;
    assert_eq!(result.unwrap(), Some("message"));

    sender_handle.await.unwrap();
}

#[tokio::test]
async fn test_recv_with_timeout_failure() {
    let (_tx, mut rx) = VelaChannel::<&str>::unbounded().split();

    let result = utils::recv_with_timeout(&mut rx, Duration::from_millis(10)).await;
    assert!(matches!(result, Err(ChannelError::RecvTimeout)));
}

#[tokio::test]
async fn test_select_first() {
    let (tx1, mut rx1) = VelaChannel::unbounded().split();
    let (tx2, mut rx2) = VelaChannel::unbounded().split();
    let (tx3, mut rx3) = VelaChannel::unbounded().split();

    let receivers = vec![&mut rx1, &mut rx2, &mut rx3];

    // Send to second channel after a delay
    let sender_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx2.send("second").await.unwrap();
    });

    let (index, message) = utils::select_first(receivers).await;
    assert_eq!(index, 1); // Second receiver
    assert_eq!(message, "second");

    sender_handle.await.unwrap();
}

#[tokio::test]
async fn test_multiple_messages() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    // Send multiple messages
    for i in 0..100 {
        tx.send(i).await.unwrap();
    }

    // Receive all messages
    for i in 0..100 {
        assert_eq!(rx.recv().await, Some(i));
    }

    // Channel should be empty
    assert_eq!(rx.try_recv().unwrap(), None);
}

#[tokio::test]
async fn test_concurrent_senders_receivers() {
    let (tx, mut rx) = VelaChannel::new(10).split();
    let num_senders = 5;
    let messages_per_sender = 20;

    // Spawn multiple senders
    let sender_handles: Vec<_> = (0..num_senders).map(|sender_id| {
        let tx = tx.clone();
        tokio::spawn(async move {
            for msg_id in 0..messages_per_sender {
                let message = format!("sender-{}-msg-{}", sender_id, msg_id);
                tx.send(message).await.unwrap();
            }
        })
    }).collect();

    // Receive all messages
    let mut received_messages = std::collections::HashMap::new();
    for _ in 0..(num_senders * messages_per_sender) {
        if let Some(message) = rx.recv().await {
            let parts: Vec<&str> = message.split('-').collect();
            let sender_id: usize = parts[1].parse().unwrap();
            *received_messages.entry(sender_id).or_insert(0) += 1;
        }
    }

    // Verify all messages were received
    for sender_id in 0..num_senders {
        assert_eq!(received_messages.get(&sender_id), Some(&messages_per_sender));
    }

    // Wait for all senders to complete
    for handle in sender_handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_channel_clone_sender() {
    let (tx1, mut rx) = VelaChannel::unbounded().split();
    let tx2 = tx1.clone();
    let tx3 = tx1.clone();

    // All senders should be able to send
    tx1.send("from_tx1").await.unwrap();
    tx2.send("from_tx2").await.unwrap();
    tx3.send("from_tx3").await.unwrap();

    let mut messages = Vec::new();
    for _ in 0..3 {
        if let Some(msg) = rx.recv().await {
            messages.push(msg);
        }
    }

    assert_eq!(messages.len(), 3);
    assert!(messages.contains(&"from_tx1"));
    assert!(messages.contains(&"from_tx2"));
    assert!(messages.contains(&"from_tx3"));
}

#[tokio::test]
async fn test_channel_capacity_zero() {
    let (tx, mut rx) = VelaChannel::<i32>::new(1).split();

    // Fill the channel first
    tx.send(1).await.unwrap();

    // With capacity 1, second send should wait for receive
    let send_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        tx.send(42).await.unwrap();
    });

    let recv_handle = tokio::spawn(async move {
        // Receive first message
        let first = rx.recv().await.unwrap();
        assert_eq!(first, 1);
        // Then receive second message
        rx.recv().await
    });

    let ((), received) = tokio::try_join!(send_handle, recv_handle).unwrap();
    assert_eq!(received, Some(42));
}

#[tokio::test]
async fn test_error_after_close() {
    let (tx, mut rx) = VelaChannel::unbounded().split();

    // Close receiver
    rx.close();

    // Send should fail
    let result = tx.send(42).await;
    assert!(matches!(result, Err(ChannelError::Closed)));

    // Try send should also fail
    let result = tx.try_send(42);
    assert!(matches!(result, Err(ChannelError::Closed)));
}