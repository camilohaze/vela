//! HTTP client module for Vela
//!
//! Provides a modern HTTP client API inspired by fetch() and reqwest,
//! with support for async operations, JSON parsing, and comprehensive error handling.

pub mod client;

pub use client::{HttpClient, HttpRequest, HttpResponse, HttpError, HttpMethod};