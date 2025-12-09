//! HTTP server implementation using hyper

use crate::error::Result;
use crate::middleware::MiddlewareChain;
use crate::routing::RouteTable;
use crate::types::{Method, Request, Response};
use hyper::service::service_fn;
use hyper::{body::Incoming, Request as HyperRequest, Response as HyperResponse};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// HTTP server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub max_connections: usize,
    pub timeout: std::time::Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().unwrap(),
            max_connections: 1000,
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// HTTP server
pub struct HttpServer {
    config: ServerConfig,
    routes: RouteTable,
    middleware: MiddlewareChain,
}

impl HttpServer {
    /// Create a new HTTP server with default configuration
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            routes: RouteTable::new(),
            middleware: MiddlewareChain::new(),
        }
    }

    /// Create a new HTTP server with custom configuration
    pub fn with_config(config: ServerConfig) -> Self {
        Self {
            config,
            routes: RouteTable::new(),
            middleware: MiddlewareChain::new(),
        }
    }

    /// Bind server to address
    pub fn bind(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.config.addr = addr.into();
        self
    }

    /// Add a route
    pub fn route<F, Fut>(mut self, method: Method, path: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response>> + Send + 'static,
    {
        self.routes.insert(method, path, crate::middleware::FnHandler(handler));
        self
    }

    /// Add middleware
    pub fn middleware<M: crate::middleware::Middleware>(mut self, middleware: M) -> Self {
        self.middleware = self.middleware.add(middleware);
        self
    }

    /// Start the server
    pub async fn serve(self) -> Result<()> {
        let listener = TcpListener::bind(self.config.addr).await?;
        let routes = Arc::new(self.routes);
        let middleware = Arc::new(self.middleware);

        tracing::info!("Server listening on {}", self.config.addr);

        loop {
            let (stream, _) = listener.accept().await?;
            let routes = routes.clone();
            let middleware = middleware.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: HyperRequest<Incoming>| {
                    let routes = routes.clone();
                    let middleware = middleware.clone();
                    async move {
                        handle_request(req, routes, middleware).await
                    }
                });

                if let Err(err) = hyper::server::conn::http1::Builder::new()
                    .serve_connection(TokioIo::new(stream), service)
                    .with_upgrades()
                    .await
                {
                    tracing::error!("Error serving connection: {}", err);
                }
            });
        }
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle an incoming HTTP request
async fn handle_request(
    hyper_req: HyperRequest<Incoming>,
    routes: Arc<RouteTable>,
    middleware: Arc<MiddlewareChain>,
) -> std::result::Result<HyperResponse<Full<hyper::body::Bytes>>, Infallible> {
    // Convert hyper request to our request type
    let vela_req = convert_hyper_request(hyper_req).await;

    // Find route
    let method = crate::types::Method::from(vela_req.method.clone());
    let (handler, _params) = match routes.find(&method, &vela_req.uri) {
        Some((handler, params)) => (handler, params),
        None => {
            let response = Response::not_found()
                .with_body(crate::types::Body::from("Not Found"));
            return Ok(convert_vela_response(response));
        }
    };

    // Execute middleware chain
    let result = middleware.execute(vela_req, handler).await;

    match result {
        Ok(vela_resp) => Ok(convert_vela_response(vela_resp)),
        Err(err) => {
            tracing::error!("Handler error: {}", err);
            let error_resp = Response::internal_server_error()
                .with_body(crate::types::Body::from("Internal Server Error"));
            Ok(convert_vela_response(error_resp))
        }
    }
}

/// Convert hyper request to vela request
async fn convert_hyper_request(hyper_req: HyperRequest<Incoming>) -> Request {
    let (parts, body) = hyper_req.into_parts();

    // Read body
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => hyper::body::Bytes::new(),
    };

    // Convert headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in parts.headers {
        if let (Some(name), Ok(value_str)) = (name, value.to_str()) {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    Request {
        method: crate::types::Method::from(parts.method),
        uri: parts.uri.to_string(),
        version: parts.version,
        headers,
        body: crate::types::Body::from(body_bytes),
    }
}

/// Convert vela response to hyper response
fn convert_vela_response(vela_resp: Response) -> HyperResponse<Full<hyper::body::Bytes>> {
    let mut builder = HyperResponse::builder()
        .status(hyper::StatusCode::from_u16(vela_resp.status.as_u16()).unwrap())
        .version(vela_resp.version);

    // Add headers
    for (key, value) in vela_resp.headers {
        builder = builder.header(key, value);
    }

    builder
        .body(Full::from(vela_resp.body.into_bytes()))
        .unwrap()
}