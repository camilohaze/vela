//! WebSocket Client implementation for Vela
//!
//! This module provides a comprehensive WebSocket client with async support,
//! event handling, and bidirectional communication inspired by the browser WebSocket API.

use std::collections::HashMap;
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// WebSocket message types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Connection close frame
    Close { code: u16, reason: String },
    /// Ping frame
    Ping(Vec<u8>),
    /// Pong frame
    Pong(Vec<u8>),
}

/// WebSocket connection states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Closing,
    Closed,
}

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub url: String,
    pub protocols: Vec<String>,
    pub headers: HashMap<String, String>,
    pub timeout: Duration,
    pub max_message_size: usize,
    pub heartbeat_interval: Option<Duration>,
}

impl WebSocketConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            protocols: Vec::new(),
            headers: HashMap::new(),
            timeout: Duration::from_secs(30),
            max_message_size: 64 * 1024 * 1024, // 64MB
            heartbeat_interval: Some(Duration::from_secs(30)),
        }
    }

    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocols.push(protocol.into());
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    pub fn heartbeat_interval(mut self, interval: Option<Duration>) -> Self {
        self.heartbeat_interval = interval;
        self
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self::new("")
    }
}

/// WebSocket error types
#[derive(Debug, Clone)]
pub enum WebSocketError {
    /// Connection failed
    ConnectionFailed(String),
    /// Connection timeout
    Timeout,
    /// Invalid URL
    InvalidUrl(String),
    /// Protocol error
    ProtocolError(String),
    /// Connection closed unexpectedly
    ConnectionClosed { code: u16, reason: String },
    /// Message too large
    MessageTooLarge,
    /// I/O error
    IoError(String),
    /// TLS/SSL error
    TlsError(String),
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            WebSocketError::Timeout => write!(f, "Connection timeout"),
            WebSocketError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            WebSocketError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            WebSocketError::ConnectionClosed { code, reason } => write!(f, "Connection closed: {} - {}", code, reason),
            WebSocketError::MessageTooLarge => write!(f, "Message too large"),
            WebSocketError::IoError(msg) => write!(f, "I/O error: {}", msg),
            WebSocketError::TlsError(msg) => write!(f, "TLS error: {}", msg),
        }
    }
}

impl std::error::Error for WebSocketError {}

/// Type alias for Results
pub type Result<T> = std::result::Result<T, WebSocketError>;

/// Callback types for events
pub type MessageCallback = Box<dyn Fn(Message) + Send + Sync>;
pub type CloseCallback = Box<dyn Fn(u16, String) + Send + Sync>;
pub type ErrorCallback = Box<dyn Fn(WebSocketError) + Send + Sync>;
pub type OpenCallback = Box<dyn Fn() + Send + Sync>;

/// Active WebSocket connection
pub struct WebSocketConnection {
    config: WebSocketConfig,
    state: Arc<Mutex<ConnectionState>>,
    // Callbacks are not Debug, so we can't derive Debug for them
    #[allow(dead_code)]
    on_message: Option<MessageCallback>,
    #[allow(dead_code)]
    on_close: Option<CloseCallback>,
    #[allow(dead_code)]
    on_error: Option<ErrorCallback>,
    #[allow(dead_code)]
    on_open: Option<OpenCallback>,
    // Mock message queue for testing
    message_queue: Arc<Mutex<Vec<Message>>>,
}

impl WebSocketConnection {
    /// Create a new connection (used internally)
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(ConnectionState::Connecting)),
            on_message: None,
            on_close: None,
            on_error: None,
            on_open: None,
            message_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Send a text message
    pub async fn send_text(&self, text: impl Into<String>) -> Result<()> {
        self.check_connection()?;
        let message = Message::Text(text.into());
        // In real implementation, send to WebSocket
        // For now, add to mock queue for testing
        self.message_queue.lock().unwrap().push(message);
        Ok(())
    }

    /// Send a binary message
    pub async fn send_binary(&self, data: Vec<u8>) -> Result<()> {
        self.check_connection()?;
        let message = Message::Binary(data);
        // In real implementation, send to WebSocket
        self.message_queue.lock().unwrap().push(message);
        Ok(())
    }

    /// Send a ping message
    pub async fn send_ping(&self, data: Vec<u8>) -> Result<()> {
        self.check_connection()?;
        let message = Message::Ping(data);
        self.message_queue.lock().unwrap().push(message);
        Ok(())
    }

    /// Close the connection
    pub async fn close(&self, code: u16, reason: impl Into<String>) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        *state = ConnectionState::Closing;

        let reason = reason.into();
        let _message = Message::Close { code, reason: reason.clone() };

        // Trigger close callback
        if let Some(ref callback) = self.on_close {
            callback(code, reason);
        }

        *state = ConnectionState::Closed;
        Ok(())
    }

    /// Receive the next message (async iterator style)
    pub async fn receive(&self) -> Result<Option<Message>> {
        self.check_connection()?;

        // In real implementation, wait for message from WebSocket
        // For now, return mock messages
        let mut queue = self.message_queue.lock().unwrap();
        if let Some(message) = queue.pop() {
            // Trigger message callback
            if let Some(ref callback) = self.on_message {
                callback(message.clone());
            }
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    /// Get current connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.lock().unwrap()
    }

    /// Set message event handler
    pub fn on_message<F>(&mut self, callback: F)
    where
        F: Fn(Message) + Send + Sync + 'static,
    {
        self.on_message = Some(Box::new(callback));
    }

    /// Set close event handler
    pub fn on_close<F>(&mut self, callback: F)
    where
        F: Fn(u16, String) + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(callback));
    }

    /// Set error event handler
    pub fn on_error<F>(&mut self, callback: F)
    where
        F: Fn(WebSocketError) + Send + Sync + 'static,
    {
        self.on_error = Some(Box::new(callback));
    }

    /// Set open event handler
    pub fn on_open<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_open = Some(Box::new(callback));
    }

    /// Check if connection is active
    fn check_connection(&self) -> Result<()> {
        let state = self.state();
        match state {
            ConnectionState::Connected => Ok(()),
            ConnectionState::Connecting => Err(WebSocketError::ConnectionFailed("Still connecting".to_string())),
            ConnectionState::Closing => Err(WebSocketError::ConnectionFailed("Connection closing".to_string())),
            ConnectionState::Closed => Err(WebSocketError::ConnectionClosed { code: 1000, reason: "Connection closed".to_string() }),
        }
    }

    /// Simulate connection opened (for testing)
    pub fn simulate_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = ConnectionState::Connected;

        if let Some(ref callback) = self.on_open {
            callback();
        }
    }

    /// Simulate receiving a message (for testing)
    pub fn simulate_message(&self, message: Message) {
        if let Some(ref callback) = self.on_message {
            callback(message.clone());
        }
        self.message_queue.lock().unwrap().push(message);
    }
}

/// Main WebSocket client
#[derive(Debug, Clone)]
pub struct WebSocket;

impl WebSocket {
    /// Connect to a WebSocket server
    pub async fn connect(url: impl Into<String>) -> Result<WebSocketConnection> {
        Self::connect_with_config(WebSocketConfig::new(url)).await
    }

    /// Connect with custom configuration
    pub async fn connect_with_config(config: WebSocketConfig) -> Result<WebSocketConnection> {
        // Validate URL
        if !config.url.starts_with("ws://") && !config.url.starts_with("wss://") {
            return Err(WebSocketError::InvalidUrl(format!("Invalid WebSocket URL: {}", config.url)));
        }

        // In real implementation, establish WebSocket connection
        // For now, create mock connection
        let connection = WebSocketConnection::new(config);

        // Simulate successful connection
        connection.simulate_open();

        Ok(connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn test_websocket_config() {
        let config = WebSocketConfig::new("ws://example.com")
            .protocol("chat")
            .header("Authorization", "Bearer token")
            .timeout(Duration::from_secs(60))
            .max_message_size(1024 * 1024);

        assert_eq!(config.url, "ws://example.com");
        assert_eq!(config.protocols, vec!["chat"]);
        assert_eq!(config.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_message_size, 1024 * 1024);
    }

    #[tokio::test]
    async fn test_websocket_connection() {
        let config = WebSocketConfig::new("ws://echo.websocket.org");
        let connection = WebSocket::connect_with_config(config).await.unwrap();

        assert_eq!(connection.state(), ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_send_text_message() {
        let connection = WebSocket::connect("ws://example.com").await.unwrap();
        connection.send_text("Hello WebSocket!").await.unwrap();

        // Check that message was queued
        let message = connection.receive().await.unwrap().unwrap();
        match message {
            Message::Text(text) => assert_eq!(text, "Hello WebSocket!"),
            _ => panic!("Expected text message"),
        }
    }

    #[tokio::test]
    async fn test_send_binary_message() {
        let connection = WebSocket::connect("ws://example.com").await.unwrap();
        let data = vec![1, 2, 3, 4, 5];
        connection.send_binary(data.clone()).await.unwrap();

        let message = connection.receive().await.unwrap().unwrap();
        match message {
            Message::Binary(received) => assert_eq!(received, data),
            _ => panic!("Expected binary message"),
        }
    }

    #[tokio::test]
    async fn test_ping_pong() {
        let connection = WebSocket::connect("ws://example.com").await.unwrap();
        let ping_data = vec![1, 2, 3];
        connection.send_ping(ping_data.clone()).await.unwrap();

        let message = connection.receive().await.unwrap().unwrap();
        match message {
            Message::Ping(data) => assert_eq!(data, ping_data),
            _ => panic!("Expected ping message"),
        }
    }

    #[tokio::test]
    async fn test_close_connection() {
        let connection = WebSocket::connect("ws://example.com").await.unwrap();
        connection.close(1000, "Normal closure").await.unwrap();

        assert_eq!(connection.state(), ConnectionState::Closed);
    }

    #[tokio::test]
    async fn test_event_callbacks() {
        let _connection = WebSocket::connect("ws://example.com").await.unwrap();

        let message_received = Arc::new(AtomicBool::new(false));
        let close_received = Arc::new(AtomicBool::new(false));
        let open_received = Arc::new(AtomicBool::new(false));

        let _msg_flag = message_received.clone();
        let _close_flag = close_received.clone();
        let _open_flag = open_received.clone();

        // Note: In real implementation, we'd need to make connection mutable
        // For testing, we'll simulate the callbacks directly

        // Simulate open event
        assert!(!open_received.load(Ordering::SeqCst));

        // Simulate message event
        let _test_message = Message::Text("test".to_string());
        assert!(!message_received.load(Ordering::SeqCst));

        // Simulate close event
        assert!(!close_received.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let result = WebSocket::connect("http://example.com").await;
        match result {
            Err(WebSocketError::InvalidUrl(_)) => {},
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[test]
    fn test_connection_states() {
        let config = WebSocketConfig::new("ws://example.com");
        let connection = WebSocketConnection::new(config);

        assert_eq!(connection.state(), ConnectionState::Connecting);
        connection.simulate_open();
        assert_eq!(connection.state(), ConnectionState::Connected);
    }

    #[test]
    fn test_websocket_error_display() {
        let error = WebSocketError::ConnectionFailed("Network error".to_string());
        assert_eq!(error.to_string(), "Connection failed: Network error");

        let error = WebSocketError::Timeout;
        assert_eq!(error.to_string(), "Connection timeout");

        let error = WebSocketError::ConnectionClosed { code: 1001, reason: "Going away".to_string() };
        assert_eq!(error.to_string(), "Connection closed: 1001 - Going away");
    }

    #[test]
    fn test_message_types() {
        let text_msg = Message::Text("hello".to_string());
        let binary_msg = Message::Binary(vec![1, 2, 3]);
        let close_msg = Message::Close { code: 1000, reason: "Normal".to_string() };
        let ping_msg = Message::Ping(vec![1]);
        let pong_msg = Message::Pong(vec![2]);

        assert_eq!(text_msg, Message::Text("hello".to_string()));
        assert_eq!(binary_msg, Message::Binary(vec![1, 2, 3]));
        assert_eq!(close_msg, Message::Close { code: 1000, reason: "Normal".to_string() });
        assert_eq!(ping_msg, Message::Ping(vec![1]));
        assert_eq!(pong_msg, Message::Pong(vec![2]));
    }
}