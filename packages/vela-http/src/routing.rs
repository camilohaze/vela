//! HTTP routing system

use crate::error::Result;
use crate::types::{Method, Request, Response};
use regex::Regex;
use std::collections::HashMap;

/// Route handler trait
#[async_trait::async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, req: Request) -> Result<Response>;
}

/// Function pointer handler
pub struct FnHandler<F>(pub F);

#[async_trait::async_trait]
impl<F, Fut> Handler for FnHandler<F>
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Response>> + Send + 'static,
{
    async fn handle(&self, req: Request) -> Result<Response> {
        (self.0)(req).await
    }
}

/// Route table for storing and matching routes
#[derive(Default)]
pub struct RouteTable {
    /// Static routes: (method, path) -> handler
    static_routes: HashMap<(Method, String), Box<dyn Handler>>,
    /// Dynamic routes: (method, regex, param_names) -> handler
    dynamic_routes: Vec<(Method, Regex, Vec<String>, Box<dyn Handler>)>,
}

impl RouteTable {
    /// Create a new empty route table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a route with a handler
    pub fn insert<H: Handler>(&mut self, method: Method, path: &str, handler: H) {
        if path.contains(':') || path.contains('*') {
            // Dynamic route
            let (regex, param_names) = Self::parse_dynamic_path(path);
            self.dynamic_routes.push((method, regex, param_names, Box::new(handler)));
        } else {
            // Static route
            self.static_routes.insert((method, path.to_string()), Box::new(handler));
        }
    }

    /// Find a handler for the given method and path
    pub fn find(&self, method: &Method, path: &str) -> Option<(&dyn Handler, HashMap<String, String>)> {
        // Try static routes first
        if let Some(handler) = self.static_routes.get(&(method.clone(), path.to_string())) {
            return Some((handler.as_ref(), HashMap::new()));
        }

        // Try dynamic routes
        for (route_method, regex, param_names, handler) in &self.dynamic_routes {
            if route_method == method {
                if let Some(captures) = regex.captures(path) {
                    let mut params = HashMap::new();
                    for (i, param_name) in param_names.iter().enumerate() {
                        if let Some(value) = captures.get(i + 1) {
                            params.insert(param_name.clone(), value.as_str().to_string());
                        }
                    }
                    return Some((handler.as_ref(), params));
                }
            }
        }

        None
    }

    /// Parse a dynamic path into regex and parameter names
    /// Examples:
    /// - "/users/:id" -> regex: "^/users/([^/]+)$", params: ["id"]
    /// - "/posts/:year/:month" -> regex: "^/posts/([^/]+)/([^/]+)$", params: ["year", "month"]
    fn parse_dynamic_path(path: &str) -> (Regex, Vec<String>) {
        let mut regex_pattern = String::from("^");
        let mut param_names = Vec::new();
        let mut in_param = false;
        let mut param_start = 0;

        for (i, ch) in path.char_indices() {
            match ch {
                ':' => {
                    if !in_param {
                        in_param = true;
                        param_start = i + 1;
                    }
                }
                '/' | '?' | '#' | '\0' => {
                    if in_param {
                        let param_name = &path[param_start..i];
                        param_names.push(param_name.to_string());
                        regex_pattern.push_str("([^/]+)");
                        in_param = false;
                    }
                    if ch != '\0' {
                        regex_pattern.push(ch);
                    }
                }
                _ => {
                    if !in_param {
                        regex_pattern.push(ch);
                    }
                }
            }
        }

        // Handle parameter at the end
        if in_param {
            let param_name = &path[param_start..];
            param_names.push(param_name.to_string());
            regex_pattern.push_str("([^/]+)");
        }

        regex_pattern.push('$');

        let regex = Regex::new(&regex_pattern).expect("Invalid regex pattern");
        (regex, param_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Response;

    async fn dummy_handler(_req: Request) -> Result<Response> {
        Ok(Response::ok())
    }

    #[test]
    fn test_static_route() {
        let mut table = RouteTable::new();
        table.insert(Method::GET, "/users", FnHandler(dummy_handler));

        let (_handler, params) = table.find(&Method::GET, "/users").unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_dynamic_route() {
        let mut table = RouteTable::new();
        table.insert(Method::GET, "/users/:id", FnHandler(dummy_handler));

        let (_handler, params) = table.find(&Method::GET, "/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_dynamic_route_multiple_params() {
        let mut table = RouteTable::new();
        table.insert(Method::GET, "/posts/:year/:month", FnHandler(dummy_handler));

        let (_handler, params) = table.find(&Method::GET, "/posts/2023/12").unwrap();
        assert_eq!(params.get("year"), Some(&"2023".to_string()));
        assert_eq!(params.get("month"), Some(&"12".to_string()));
    }

    #[test]
    fn test_route_not_found() {
        let table = RouteTable::new();
        assert!(table.find(&Method::GET, "/nonexistent").is_none());
    }
}