//! I/O and networking APIs for Vela
//!
//! This module provides comprehensive I/O capabilities including:
//! - File operations (read, write, copy, move, delete)
//! - Directory operations (create, list, remove)
//! - HTTP client for REST APIs
//! - WebSocket client for real-time communication

pub mod file;
pub mod directory;
// pub mod http;       // TODO: TASK-089
// pub mod websocket;  // TODO: TASK-090

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, std::io::Error>;