//! WebSocket client module for Vela
//!
//! Provides a modern WebSocket client API for bidirectional communication,
//! inspired by the browser WebSocket API with async support and event handling.

pub mod client;

pub use client::{WebSocket, WebSocketConnection, Message, WebSocketError, WebSocketConfig, ConnectionState};