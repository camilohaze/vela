//! HTTP types and utilities

use bytes::Bytes;
use http::{Method as HttpMethod, StatusCode as HttpStatusCode, Version};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP method
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::CONNECT => "CONNECT",
            Method::TRACE => "TRACE",
        };
        write!(f, "{}", s)
    }
}

impl From<Method> for HttpMethod {
    fn from(method: Method) -> Self {
        match method {
            Method::GET => HttpMethod::GET,
            Method::POST => HttpMethod::POST,
            Method::PUT => HttpMethod::PUT,
            Method::DELETE => HttpMethod::DELETE,
            Method::PATCH => HttpMethod::PATCH,
            Method::HEAD => HttpMethod::HEAD,
            Method::OPTIONS => HttpMethod::OPTIONS,
            Method::CONNECT => HttpMethod::CONNECT,
            Method::TRACE => HttpMethod::TRACE,
        }
    }
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::HEAD => Method::HEAD,
            HttpMethod::OPTIONS => Method::OPTIONS,
            HttpMethod::CONNECT => Method::CONNECT,
            HttpMethod::TRACE => Method::TRACE,
            _ => Method::GET, // fallback
        }
    }
}

/// HTTP status code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCode(HttpStatusCode);

impl StatusCode {
    pub fn new(code: u16) -> Result<Self, String> {
        HttpStatusCode::from_u16(code)
            .map(StatusCode)
            .map_err(|_| format!("Invalid status code: {}", code))
    }

    pub fn as_u16(&self) -> u16 {
        self.0.as_u16()
    }

    pub fn is_success(&self) -> bool {
        self.0.is_success()
    }

    pub fn is_client_error(&self) -> bool {
        self.0.is_client_error()
    }

    pub fn is_server_error(&self) -> bool {
        self.0.is_server_error()
    }
}

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode(HttpStatusCode::OK)
    }
}

/// HTTP body
#[derive(Debug, Clone, Default)]
pub struct Body {
    inner: Bytes,
}

impl Body {
    pub fn new(data: impl Into<Bytes>) -> Self {
        Body {
            inner: data.into(),
        }
    }

    pub fn empty() -> Self {
        Body::new(Bytes::new())
    }

    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Body::new(s.into_bytes())
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Body::new(s.to_string().into_bytes())
    }
}

impl From<Vec<u8>> for Body {
    fn from(v: Vec<u8>) -> Self {
        Body::new(v)
    }
}

impl From<Bytes> for Body {
    fn from(bytes: Bytes) -> Self {
        Body { inner: bytes }
    }
}

/// HTTP headers
pub type Headers = HashMap<String, String>;

/// HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: String,
    pub version: Version,
    pub headers: Headers,
    pub body: Body,
}

impl Request {
    pub fn new(method: Method, uri: impl Into<String>) -> Self {
        Request {
            method,
            uri: uri.into(),
            version: Version::HTTP_11,
            headers: HashMap::new(),
            body: Body::empty(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub version: Version,
    pub headers: Headers,
    pub body: Body,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Response {
            status,
            version: Version::HTTP_11,
            headers: HashMap::new(),
            body: Body::empty(),
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::new(200).unwrap())
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::new(404).unwrap())
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::new(500).unwrap())
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }

    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(data)?;
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(self.with_body(Body::from(json)))
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}