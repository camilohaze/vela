//! Router Engine para API Gateway
//!
//! Implementa routing basado en tries con soporte para:
//! - Wildcards (*)
//! - Parámetros nombrados (:param)
//! - Métodos HTTP
//! - Middleware chaining

use std::collections::HashMap;
use crate::gateway::{Request, Response, Context, GatewayError};

/// Ruta definida
#[derive(Debug, Clone)]
pub struct Route {
    pub path: String,
    pub methods: Vec<String>,
    pub service: String,
    pub middlewares: Vec<String>,
}

/// Nodo del trie de routing
#[derive(Debug)]
struct TrieNode {
    children: HashMap<String, TrieNode>,
    routes: HashMap<String, Route>, // method -> route
    is_wildcard: bool,
    param_name: Option<String>,
}

/// Router principal
#[derive(Debug)]
pub struct Router {
    root: TrieNode,
    routes: Vec<Route>,
}

impl Router {
    /// Crear nuevo router
    pub fn new() -> Self {
        Self {
            root: TrieNode {
                children: HashMap::new(),
                routes: HashMap::new(),
                is_wildcard: false,
                param_name: None,
            },
            routes: Vec::new(),
        }
    }

    /// Agregar una ruta
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route.clone());
        self.insert_route(&route);
    }

    /// Insertar ruta en el trie
    fn insert_route(&mut self, route: &Route) {
        let segments: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut self.root;

        for segment in segments {
            let key = if segment.starts_with(':') {
                // Parámetro nombrado
                ":param".to_string()
            } else if segment == "*" {
                // Wildcard
                "*".to_string()
            } else {
                segment.to_string()
            };

            current = current.children.entry(key).or_insert_with(|| {
                let param_name = if segment.starts_with(':') {
                    Some(segment[1..].to_string())
                } else {
                    None
                };

                TrieNode {
                    children: HashMap::new(),
                    routes: HashMap::new(),
                    is_wildcard: segment == "*",
                    param_name,
                }
            });
        }

        // Agregar métodos HTTP
        for method in &route.methods {
            current.routes.insert(method.clone(), route.clone());
        }
    }

    /// Encontrar ruta que coincide con path y método
    pub fn match_route(&self, path: &str, method: &str) -> Option<Route> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.root;
        let mut params = HashMap::new();

        for segment in segments {
            // Primero intentar match exacto
            if let Some(node) = current.children.get(segment) {
                current = node;
                continue;
            }

            // Intentar match con parámetro
            if let Some(param_node) = current.children.get(":param") {
                if let Some(ref param_name) = param_node.param_name {
                    params.insert(param_name.clone(), segment.to_string());
                }
                current = param_node;
                continue;
            }

            // Intentar match con wildcard
            if let Some(wildcard_node) = current.children.get("*") {
                current = wildcard_node;
                break; // Wildcard consume el resto
            }

            // No match
            return None;
        }

        // Verificar si el método está soportado
        current.routes.get(method).cloned()
    }

    /// Obtener todas las rutas
    pub fn get_routes(&self) -> &[Route] {
        &self.routes
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert!(router.get_routes().is_empty());
    }

    #[test]
    fn test_add_route() {
        let mut router = Router::new();

        let route = Route {
            path: "/api/users".to_string(),
            methods: vec!["GET".to_string(), "POST".to_string()],
            service: "user-service".to_string(),
            middlewares: vec!["auth".to_string()],
        };

        router.add_route(route.clone());

        assert_eq!(router.get_routes().len(), 1);
        assert_eq!(router.get_routes()[0].path, "/api/users");
    }

    #[test]
    fn test_match_route() {
        let mut router = Router::new();

        let route = Route {
            path: "/api/users".to_string(),
            methods: vec!["GET".to_string()],
            service: "user-service".to_string(),
            middlewares: vec![],
        };

        router.add_route(route.clone());

        let matched = router.match_route("/api/users", "GET");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().service, "user-service");

        // Método no soportado
        let not_matched = router.match_route("/api/users", "PUT");
        assert!(not_matched.is_none());

        // Ruta no existente
        let not_matched = router.match_route("/api/posts", "GET");
        assert!(not_matched.is_none());
    }

    #[test]
    fn test_param_route() {
        let mut router = Router::new();

        let route = Route {
            path: "/api/users/:id".to_string(),
            methods: vec!["GET".to_string()],
            service: "user-service".to_string(),
            middlewares: vec![],
        };

        router.add_route(route.clone());

        let matched = router.match_route("/api/users/123", "GET");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().service, "user-service");
    }

    #[test]
    fn test_wildcard_route() {
        let mut router = Router::new();

        let route = Route {
            path: "/api/*".to_string(),
            methods: vec!["GET".to_string()],
            service: "api-service".to_string(),
            middlewares: vec![],
        };

        router.add_route(route.clone());

        let matched = router.match_route("/api/users/123/posts", "GET");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().service, "api-service");
    }
}