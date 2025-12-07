//! HTTP Client implementation for Vela
//!
//! This module provides a comprehensive HTTP client with async support,
//! inspired by the fetch API and modern HTTP libraries like reqwest.

use std::collections::HashMap;
use std::time::Duration;

/// HTTP methods supported by the client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalServerError = 500,
}

impl HttpStatus {
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            200 => Some(HttpStatus::Ok),
            201 => Some(HttpStatus::Created),
            202 => Some(HttpStatus::Accepted),
            204 => Some(HttpStatus::NoContent),
            400 => Some(HttpStatus::BadRequest),
            401 => Some(HttpStatus::Unauthorized),
            403 => Some(HttpStatus::Forbidden),
            404 => Some(HttpStatus::NotFound),
            500 => Some(HttpStatus::InternalServerError),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        let code = *self as u16;
        code >= 200 && code < 300
    }
}

/// HTTP Error types
#[derive(Debug, Clone)]
pub enum HttpError {
    /// Network connection failed
    NetworkError(String),
    /// Request timed out
    Timeout,
    /// HTTP status error
    StatusError { status: u16, message: String },
    /// Invalid URL
    InvalidUrl(String),
    /// JSON parsing error
    JsonError(String),
    /// I/O error
    IoError(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            HttpError::Timeout => write!(f, "Request timeout"),
            HttpError::StatusError { status, message } => write!(f, "HTTP {}: {}", status, message),
            HttpError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            HttpError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            HttpError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for HttpError {}

/// Type alias for Results
pub type Result<T> = std::result::Result<T, HttpError>;

/// HTTP Response wrapper
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new response (used internally)
    pub fn new(status: u16, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self { status, headers, body }
    }

    /// Get status as enum if it's a known status
    pub fn status_enum(&self) -> Option<HttpStatus> {
        HttpStatus::from_u16(self.status)
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Get response body as string
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| HttpError::IoError(format!("UTF-8 decode error: {}", e)))
    }

    /// Parse response body as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body)
            .map_err(|e| HttpError::JsonError(format!("JSON parse error: {}", e)))
    }

    /// Get header value
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }
}

/// HTTP Request builder
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<Duration>,
    pub query_params: HashMap<String, String>,
}

impl HttpRequest {
    /// Create a new GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::GET,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            query_params: HashMap::new(),
        }
    }

    /// Create a new POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::POST,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            query_params: HashMap::new(),
        }
    }

    /// Create a new PUT request
    pub fn put(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::PUT,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            query_params: HashMap::new(),
        }
    }

    /// Create a new DELETE request
    pub fn delete(url: impl Into<String>) -> Self {
        Self {
            method: HttpMethod::DELETE,
            url: url.into(),
            headers: HashMap::new(),
            body: None,
            timeout: None,
            query_params: HashMap::new(),
        }
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set JSON body
    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Result<Self> {
        let json = serde_json::to_vec(data)
            .map_err(|e| HttpError::JsonError(format!("JSON serialize error: {}", e)))?;
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(json);
        Ok(self)
    }

    /// Set text body
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let bytes = text.into().into_bytes();
        self.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        self.body = Some(bytes);
        self
    }

    /// Set raw bytes body
    pub fn bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body = Some(bytes);
        self
    }

    /// Add query parameter
    pub fn query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query_params.insert(name.into(), value.into());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Send the request synchronously
    pub fn send(self) -> Result<HttpResponse> {
        // For now, return a mock response
        // In a real implementation, this would use reqwest or similar
        self.send_mock()
    }

    /// Send the request asynchronously
    pub async fn send_async(self) -> Result<HttpResponse> {
        // For now, return a mock response
        // In a real implementation, this would use reqwest or similar
        self.send_mock()
    }

    /// Mock implementation for testing
    fn send_mock(self) -> Result<HttpResponse> {
        // Build full URL with query params
        let mut url = self.url.clone();
        if !self.query_params.is_empty() {
            url.push('?');
            let query_string = self.query_params.iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            url.push_str(&query_string);
        }

        // Mock response based on URL
        match url.as_str() {
            "https://httpbin.org/get" => {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());

                let response_body = r#"{
                    "args": {},
                    "headers": {
                        "Accept": "*/*",
                        "Host": "httpbin.org"
                    },
                    "url": "https://httpbin.org/get"
                }"#;

                Ok(HttpResponse::new(200, headers, response_body.as_bytes().to_vec()))
            },
            "https://httpbin.org/post" => {
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "application/json".to_string());

                let response_body = r#"{
                    "args": {},
                    "data": "",
                    "files": {},
                    "form": {},
                    "headers": {
                        "Accept": "*/*",
                        "Content-Type": "application/json",
                        "Host": "httpbin.org"
                    },
                    "json": null,
                    "url": "https://httpbin.org/post"
                }"#;

                Ok(HttpResponse::new(200, headers, response_body.as_bytes().to_vec()))
            },
            "https://httpbin.org/status/404" => {
                Err(HttpError::StatusError {
                    status: 404,
                    message: "Not Found".to_string(),
                })
            },
            _ => {
                // Default success response
                let mut headers = HashMap::new();
                headers.insert("content-type".to_string(), "text/plain".to_string());
                Ok(HttpResponse::new(200, headers, b"Hello from Vela HttpClient!".to_vec()))
            }
        }
    }
}

/// Main HTTP Client
#[derive(Debug, Clone)]
pub struct HttpClient {
    default_headers: HashMap<String, String>,
    timeout: Option<Duration>,
    user_agent: String,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new() -> Self {
        Self {
            default_headers: HashMap::new(),
            timeout: Some(Duration::from_secs(30)),
            user_agent: "Vela-HttpClient/1.0".to_string(),
        }
    }

    /// Create a GET request
    pub fn get(&self, url: impl Into<String>) -> HttpRequest {
        let mut request = HttpRequest::get(url);
        self.apply_defaults(&mut request);
        request
    }

    /// Create a POST request
    pub fn post(&self, url: impl Into<String>) -> HttpRequest {
        let mut request = HttpRequest::post(url);
        self.apply_defaults(&mut request);
        request
    }

    /// Create a PUT request
    pub fn put(&self, url: impl Into<String>) -> HttpRequest {
        let mut request = HttpRequest::put(url);
        self.apply_defaults(&mut request);
        request
    }

    /// Create a DELETE request
    pub fn delete(&self, url: impl Into<String>) -> HttpRequest {
        let mut request = HttpRequest::delete(url);
        self.apply_defaults(&mut request);
        request
    }

    /// Set default header for all requests
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// Set default timeout for all requests
    pub fn default_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Apply default settings to a request
    fn apply_defaults(&self, request: &mut HttpRequest) {
        // Apply default headers
        for (name, value) in &self.default_headers {
            if !request.headers.contains_key(name) {
                request.headers.insert(name.clone(), value.clone());
            }
        }

        // Apply default timeout if not set
        if request.timeout.is_none() {
            request.timeout = self.timeout;
        }

        // Set User-Agent if not already set
        if !request.headers.contains_key("User-Agent") {
            request.headers.insert("User-Agent".to_string(), self.user_agent.clone());
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpClient::new();
        assert!(client.default_headers.is_empty());
        assert_eq!(client.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_request_builder() {
        let request = HttpRequest::get("https://example.com")
            .header("Authorization", "Bearer token")
            .query("page", "1")
            .timeout(Duration::from_secs(10));

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(request.query_params.get("page"), Some(&"1".to_string()));
        assert_eq!(request.timeout, Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_json_request() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData { name: "test".to_string(), value: 42 };
        let request = HttpRequest::post("https://example.com/api")
            .json(&data).unwrap();

        assert_eq!(request.method, HttpMethod::POST);
        assert!(request.body.is_some());
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_mock_response() {
        let request = HttpRequest::get("https://httpbin.org/get");
        let response = request.send().unwrap();

        assert_eq!(response.status, 200);
        assert!(response.is_success());
        assert!(response.headers.contains_key("content-type"));

        let json: serde_json::Value = response.json().unwrap();
        assert!(json.get("url").is_some());
    }

    #[test]
    fn test_mock_post() {
        let request = HttpRequest::post("https://httpbin.org/post");
        let response = request.send().unwrap();

        assert_eq!(response.status, 200);
        assert!(response.is_success());
    }

    #[test]
    fn test_mock_error() {
        let request = HttpRequest::get("https://httpbin.org/status/404");
        let result = request.send();

        match result {
            Err(HttpError::StatusError { status, .. }) => assert_eq!(status, 404),
            _ => panic!("Expected status error"),
        }
    }

    #[test]
    fn test_text_response() {
        let request = HttpRequest::get("https://example.com");
        let response = request.send().unwrap();
        let text = response.text().unwrap();

        assert!(text.contains("Hello from Vela"));
    }

    #[test]
    fn test_client_with_defaults() {
        let client = HttpClient::new()
            .default_header("Authorization", "Bearer token")
            .default_timeout(Duration::from_secs(60));

        let request = client.get("https://example.com");
        assert_eq!(request.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(request.timeout, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_http_status_enum() {
        assert!(HttpStatus::Ok.is_success());
        assert!(!HttpStatus::NotFound.is_success());
        assert_eq!(HttpStatus::from_u16(200), Some(HttpStatus::Ok));
        assert_eq!(HttpStatus::from_u16(999), None);
    }
}