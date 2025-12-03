//! HTTP client implementation using reqwest

use crate::http::error::{HttpError, Result};
use crate::http::types::{Body, Method, Request, Response, StatusCode};
use reqwest::{Client as ReqwestClient, Response as ReqwestResponse};
use std::time::Duration;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout: Duration,
    pub user_agent: String,
    pub max_connections: usize,
    pub max_connections_per_host: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: "Vela-HTTP-Client/1.0".to_string(),
            max_connections: 100,
            max_connections_per_host: 10,
        }
    }
}

/// HTTP client
#[derive(Clone)]
pub struct HttpClient {
    client: ReqwestClient,
    config: ClientConfig,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .pool_max_idle_per_host(config.max_connections_per_host)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()?;

        Ok(Self { client, config })
    }

    /// Send a GET request
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.request(Method::GET, url).await
    }

    /// Send a POST request
    pub async fn post(&self, url: &str, body: Body) -> Result<Response> {
        self.request_with_body(Method::POST, url, body).await
    }

    /// Send a PUT request
    pub async fn put(&self, url: &str, body: Body) -> Result<Response> {
        self.request_with_body(Method::PUT, url, body).await
    }

    /// Send a PATCH request
    pub async fn patch(&self, url: &str, body: Body) -> Result<Response> {
        self.request_with_body(Method::PATCH, url, body).await
    }

    /// Send a DELETE request
    pub async fn delete(&self, url: &str) -> Result<Response> {
        self.request(Method::DELETE, url).await
    }

    /// Send a HEAD request
    pub async fn head(&self, url: &str) -> Result<Response> {
        self.request(Method::HEAD, url).await
    }

    /// Send a custom request
    pub async fn request(&self, method: Method, url: &str) -> Result<Response> {
        self.request_with_body(method, url, Body::empty()).await
    }

    /// Send a custom request with body
    pub async fn request_with_body(&self, method: Method, url: &str, body: Body) -> Result<Response> {
        let mut req_builder = match method {
            Method::GET => self.client.get(url),
            Method::POST => self.client.post(url),
            Method::PUT => self.client.put(url),
            Method::DELETE => self.client.delete(url),
            Method::PATCH => self.client.patch(url),
            Method::HEAD => self.client.head(url),
            Method::OPTIONS => self.client.request(reqwest::Method::OPTIONS, url),
            Method::CONNECT => self.client.request(reqwest::Method::CONNECT, url),
            Method::TRACE => self.client.request(reqwest::Method::TRACE, url),
        };

        if !body.is_empty() {
            req_builder = req_builder.body(body.into_bytes());
        }

        let reqwest_resp = req_builder.send().await?;
        convert_reqwest_response(reqwest_resp).await
    }

    /// Send a custom request using Request struct
    pub async fn send(&self, request: Request) -> Result<Response> {
        let reqwest_method = match request.method {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::DELETE => reqwest::Method::DELETE,
            Method::PATCH => reqwest::Method::PATCH,
            Method::HEAD => reqwest::Method::HEAD,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::CONNECT => reqwest::Method::CONNECT,
            Method::TRACE => reqwest::Method::TRACE,
        };

        let mut req_builder = self.client.request(reqwest_method, &request.uri);

        // Add headers
        for (key, value) in request.headers {
            req_builder = req_builder.header(key, value);
        }

        // Add body if present
        if !request.body.is_empty() {
            req_builder = req_builder.body(request.body.into_bytes());
        }

        let reqwest_resp = req_builder.send().await?;
        convert_reqwest_response(reqwest_resp).await
    }

    /// Get JSON response and deserialize
    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.get(url).await?;
        if !resp.status.is_success() {
            return Err(HttpError::Other(format!("HTTP {}: {}", resp.status.as_u16(), String::from_utf8_lossy(resp.body.as_bytes()))));
        }

        serde_json::from_slice(resp.body.as_bytes()).map_err(Into::into)
    }

    /// Post JSON data
    pub async fn post_json<T: serde::Serialize>(&self, url: &str, data: &T) -> Result<Response> {
        let json_body = serde_json::to_vec(data)?;
        let body = Body::from(json_body);
        self.post(url, body).await
    }

    /// Post JSON and get JSON response
    pub async fn post_json_response<T: serde::de::DeserializeOwned, U: serde::Serialize>(
        &self,
        url: &str,
        data: &U,
    ) -> Result<T> {
        let resp = self.post_json(url, data).await?;
        if !resp.status.is_success() {
            return Err(HttpError::Other(format!("HTTP {}: {}", resp.status.as_u16(), String::from_utf8_lossy(resp.body.as_bytes()))));
        }

        serde_json::from_slice(resp.body.as_bytes()).map_err(Into::into)
    }
}

/// Convert reqwest response to vela response
async fn convert_reqwest_response(reqwest_resp: ReqwestResponse) -> Result<Response> {
    let status = StatusCode::new(reqwest_resp.status().as_u16())
        .map_err(|e| HttpError::Other(e))?;

    // Convert reqwest version to http version
    let version = match reqwest_resp.version() {
        reqwest::Version::HTTP_09 => http::Version::HTTP_09,
        reqwest::Version::HTTP_10 => http::Version::HTTP_10,
        reqwest::Version::HTTP_11 => http::Version::HTTP_11,
        reqwest::Version::HTTP_2 => http::Version::HTTP_2,
        reqwest::Version::HTTP_3 => http::Version::HTTP_3,
        _ => http::Version::HTTP_11, // default fallback
    };

    // Collect headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in reqwest_resp.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Read body
    let body_bytes = reqwest_resp.bytes().await?;
    let body = Body::from(body_bytes);

    Ok(Response {
        status,
        version,
        headers,
        body,
    })
}