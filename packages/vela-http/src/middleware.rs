use crate::error::Result;
use crate::types::{Request, Response};

/// Middleware trait for processing HTTP requests
#[async_trait::async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, req: Request, next: Next<'_>) -> Result<Response>;
}

/// Next middleware in the chain
pub struct Next<'a> {
    middleware: &'a [Box<dyn Middleware>],
    handler: &'a dyn crate::routing::Handler,
    index: usize,
}

impl<'a> Next<'a> {
    /// Create a new Next instance
    pub fn new(middleware: &'a [Box<dyn Middleware>], handler: &'a dyn crate::routing::Handler) -> Self {
        Next {
            middleware,
            handler,
            index: 0,
        }
    }

    /// Run the next middleware or handler in the chain
    pub async fn run(mut self, req: Request) -> Result<Response> {
        if let Some(mw) = self.middleware.get(self.index) {
            self.index += 1;
            mw.handle(req, self).await
        } else {
            self.handler.handle(req).await
        }
    }
}

/// Handler trait for final request processing - re-export from routing
pub use crate::routing::Handler;

/// Function pointer handler - re-export from routing
pub use crate::routing::FnHandler;

/// Middleware chain
#[derive(Default)]
pub struct MiddlewareChain {
    middleware: Vec<Box<dyn Middleware>>,
}

impl MiddlewareChain {
    /// Create a new empty middleware chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Add middleware to the chain
    pub fn add<M: Middleware>(mut self, middleware: M) -> Self {
        self.middleware.push(Box::new(middleware));
        self
    }

    /// Execute the middleware chain with a handler
    pub async fn execute(&self, req: Request, handler: &dyn crate::routing::Handler) -> Result<Response> {
        let next = Next::new(&self.middleware, handler);
        next.run(req).await
    }

    /// Get the number of middleware in the chain
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }
}

/// Logging middleware
pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(&self, req: Request, next: Next<'_>) -> Result<Response> {
        tracing::info!("{} {}", req.method, req.uri);
        let start = std::time::Instant::now();

        let result = next.run(req).await;

        let elapsed = start.elapsed();
        match &result {
            Ok(resp) => {
                tracing::info!("Response: {} in {:?}", resp.status.as_u16(), elapsed);
            }
            Err(err) => {
                tracing::error!("Error: {} in {:?}", err, elapsed);
            }
        }

        result
    }
}

/// CORS middleware
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
}

impl CorsMiddleware {
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["*".to_string()],
        }
    }

    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    pub fn with_methods(mut self, methods: Vec<String>) -> Self {
        self.allowed_methods = methods;
        self
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = headers;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    async fn handle(&self, req: Request, next: Next<'_>) -> Result<Response> {
        // Handle preflight requests
        if req.method == crate::types::Method::OPTIONS {
            let mut response = Response::ok();
            response.headers.insert("Access-Control-Allow-Origin".to_string(), self.allowed_origins.join(","));
            response.headers.insert("Access-Control-Allow-Methods".to_string(), self.allowed_methods.join(","));
            response.headers.insert("Access-Control-Allow-Headers".to_string(), self.allowed_headers.join(","));
            return Ok(response);
        }

        // Add CORS headers to actual response
        let mut result = next.run(req).await?;
        result.headers.insert("Access-Control-Allow-Origin".to_string(), self.allowed_origins.join(","));
        Ok(result)
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    token_header: String,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            token_header: "Authorization".to_string(),
        }
    }

    pub fn with_header(mut self, header: String) -> Self {
        self.token_header = header;
        self
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(&self, req: Request, next: Next<'_>) -> Result<Response> {
        if let Some(auth_header) = req.header(&self.token_header) {
            if auth_header.starts_with("Bearer ") {
                // In a real implementation, validate the token
                tracing::debug!("Validating token: {}", &auth_header[7..]);
                next.run(req).await
            } else {
                Ok(Response::new(crate::types::StatusCode::new(401).unwrap())
                    .with_body(crate::types::Body::from("Invalid token format")))
            }
        } else {
            Ok(Response::new(crate::types::StatusCode::new(401).unwrap())
                .with_body(crate::types::Body::from("Missing authorization header")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Response;

    async fn dummy_handler(_req: Request) -> Result<Response> {
        Ok(Response::ok().with_body(crate::types::Body::from("Hello, World!")))
    }

    #[tokio::test]
    async fn test_middleware_chain() {
        let chain = MiddlewareChain::new()
            .add(LoggingMiddleware)
            .add(CorsMiddleware::new());

        let req = Request::new(crate::types::Method::GET, "/test");
        let handler = FnHandler(dummy_handler);

        let result = chain.execute(req, &handler).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);
    }

    #[tokio::test]
    async fn test_auth_middleware_missing_token() {
        let chain = MiddlewareChain::new().add(AuthMiddleware::new());

        let req = Request::new(crate::types::Method::GET, "/protected");
        let handler = FnHandler(dummy_handler);

        let result = chain.execute(req, &handler).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 401);
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_token() {
        let chain = MiddlewareChain::new().add(AuthMiddleware::new());

        let req = Request::new(crate::types::Method::GET, "/protected")
            .with_header("Authorization", "Bearer valid-token");
        let handler = FnHandler(dummy_handler);

        let result = chain.execute(req, &handler).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status.as_u16(), 200);
    }
}