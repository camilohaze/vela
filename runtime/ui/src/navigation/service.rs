//! # Navigation Service - High-level Navigation API
//!
//! Provides a convenient API for programmatic navigation,
//! abstracting the details of the Router widget.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use crate::widget::Widget;
use crate::navigation::router::Router;

/// Entry in the navigation history
#[derive(Debug, Clone, PartialEq)]
pub struct NavigationEntry {
    /// The full path including query string
    pub path: String,
    /// Path parameters extracted from route
    pub params: HashMap<String, String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Timestamp when navigation occurred
    pub timestamp: SystemTime,
}

impl NavigationEntry {
    pub fn new(path: String, params: HashMap<String, String>, query: HashMap<String, String>) -> Self {
        Self {
            path,
            params,
            query,
            timestamp: SystemTime::now(),
        }
    }
}

/// Errors that can occur during navigation
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationError {
    RouteNotFound(String),
    GuardBlocked(String),
    InvalidPath(String),
    RouterNotAvailable,
    InvalidParameters(String),
}

impl std::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NavigationError::RouteNotFound(path) => write!(f, "Route not found: {}", path),
            NavigationError::GuardBlocked(path) => write!(f, "Navigation blocked by guard: {}", path),
            NavigationError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            NavigationError::RouterNotAvailable => write!(f, "Router not available"),
            NavigationError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
        }
    }
}

impl std::error::Error for NavigationError {}

/// Context passed to navigation guards
#[derive(Debug, Clone)]
pub struct NavigationGuardContext {
    pub current_entry: Option<NavigationEntry>,
    pub target_entry: NavigationEntry,
}

/// Trait for navigation guards that can prevent navigation
pub trait NavigationGuard {
    fn can_activate(&self, context: &NavigationGuardContext) -> bool;
}

/// High-level navigation service
pub struct NavigationService<T, F> {
    router: Arc<Mutex<Router<T, Box<dyn Fn(T) -> Box<dyn Widget> + Send + Sync>>>>,
    builder: F,
    history: Vec<NavigationEntry>,
    current_index: usize,
    guards: Vec<Box<dyn NavigationGuard>>,
    max_history_size: usize,
}

impl<T, F> NavigationService<T, F>
where
    T: Clone + Send + Sync + 'static,
    F: Fn(crate::navigation::router::RouteMatch<T>) -> Box<dyn Widget> + Send + Sync + 'static,
{
    /// Create a new navigation service
    pub fn new(router: Arc<Mutex<Router<T, Box<dyn Fn(T) -> Box<dyn Widget> + Send + Sync>>>>, builder: F) -> Self {
        Self {
            router,
            builder,
            history: Vec::new(),
            current_index: 0,
            guards: Vec::new(),
            max_history_size: 50, // Limit history to prevent memory issues
        }
    }

    /// Add a navigation guard
    pub fn add_guard(&mut self, guard: Box<dyn NavigationGuard>) {
        self.guards.push(guard);
    }

    /// Build a widget for the given route match
    pub fn build_widget(&self, route_match: crate::navigation::router::RouteMatch<T>) -> Box<dyn Widget> {
        (self.builder)(route_match)
    }

    /// Navigate to a new path (adds to history)
    pub fn push(&mut self, path: &str) -> Result<(), NavigationError> {
        self.push_with_params(path, HashMap::new())
    }

    /// Navigate to a path with parameters
    pub fn push_with_params(&mut self, path: &str, params: HashMap<String, String>) -> Result<(), NavigationError> {
        let full_path = self.build_path_with_params(path, &params)?;
        let entry = self.create_entry(&full_path)?;

        // Check guards
        if !self.check_guards(&entry)? {
            return Err(NavigationError::GuardBlocked(full_path));
        }

        // Navigate router
        self.navigate_router(&full_path)?;

        // Update history
        self.add_to_history(entry);

        Ok(())
    }

    /// Replace current entry in history
    pub fn replace(&mut self, path: &str) -> Result<(), NavigationError> {
        self.replace_with_params(path, HashMap::new())
    }

    /// Replace current entry with path and parameters
    pub fn replace_with_params(&mut self, path: &str, params: HashMap<String, String>) -> Result<(), NavigationError> {
        let full_path = self.build_path_with_params(path, &params)?;
        let entry = self.create_entry(&full_path)?;

        // Check guards
        if !self.check_guards(&entry)? {
            return Err(NavigationError::GuardBlocked(full_path));
        }

        // Navigate router
        self.navigate_router(&full_path)?;

        // Replace current history entry
        self.replace_current_history(entry);

        Ok(())
    }

    /// Go back in history
    pub fn pop(&mut self) -> Result<(), NavigationError> {
        if !self.can_go_back() {
            return Err(NavigationError::InvalidPath("Cannot go back".to_string()));
        }

        self.go(-1)
    }

    /// Go forward in history
    pub fn go_forward(&mut self) -> Result<(), NavigationError> {
        if !self.can_go_forward() {
            return Err(NavigationError::InvalidPath("Cannot go forward".to_string()));
        }

        self.go(1)
    }

    /// Go back in history (alias for pop)
    pub fn go_back(&mut self) -> Result<(), NavigationError> {
        self.pop()
    }

    /// Navigate by delta in history
    pub fn go(&mut self, delta: i32) -> Result<(), NavigationError> {
        let new_index = (self.current_index as i32 + delta) as usize;

        if new_index >= self.history.len() {
            return Err(NavigationError::InvalidPath("History index out of bounds".to_string()));
        }

        let entry = &self.history[new_index];
        self.navigate_router(&entry.path)?;
        self.current_index = new_index;

        Ok(())
    }

    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        self.current_index > 0
    }

    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        self.current_index < self.history.len() - 1
    }

    /// Get current path
    pub fn get_current_path(&self) -> Option<String> {
        self.history.get(self.current_index).map(|entry| entry.path.clone())
    }

    /// Get navigation history
    pub fn get_history(&self) -> &[NavigationEntry] {
        &self.history
    }

    /// Get current history index
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    // Private helper methods

    fn build_path_with_params(&self, template: &str, params: &HashMap<String, String>) -> Result<String, NavigationError> {
        let mut result = template.to_string();

        for (key, value) in params {
            let placeholder = format!(":{}", key);
            result = result.replace(&placeholder, value);
        }

        // Check if all placeholders were replaced
        if result.contains(':') {
            return Err(NavigationError::InvalidParameters(
                "Not all path parameters were provided".to_string()
            ));
        }

        Ok(result)
    }

    fn create_entry(&self, path: &str) -> Result<NavigationEntry, NavigationError> {
        // Parse path and query
        let (path_part, query_part) = if let Some(pos) = path.find('?') {
            (&path[..pos], Some(&path[pos + 1..]))
        } else {
            (path, None)
        };

        let query = if let Some(query_str) = query_part {
            self.parse_query(query_str)
        } else {
            HashMap::new()
        };

        // For now, params are empty - router will fill them during matching
        let params = HashMap::new();

        Ok(NavigationEntry::new(path.to_string(), params, query))
    }

    fn parse_query(&self, query_str: &str) -> HashMap<String, String> {
        let mut query = HashMap::new();
        for pair in query_str.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                query.insert(key.to_string(), value.to_string());
            }
        }
        query
    }

    fn check_guards(&self, entry: &NavigationEntry) -> Result<bool, NavigationError> {
        let current_entry = self.history.get(self.current_index).cloned();
        let context = NavigationGuardContext {
            current_entry,
            target_entry: entry.clone(),
        };

        for guard in &self.guards {
            if !guard.can_activate(&context) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn navigate_router(&self, path: &str) -> Result<(), NavigationError> {
        let mut router = self.router.lock().map_err(|_| NavigationError::RouterNotAvailable)?;
        router.navigate_to(path);
        Ok(())
    }

    fn add_to_history(&mut self, entry: NavigationEntry) {
        // Remove forward history when adding new entry
        self.history.truncate(self.current_index + 1);
        self.history.push(entry);
        self.current_index = self.history.len() - 1;

        // Limit history size
        if self.history.len() > self.max_history_size {
            let remove_count = self.history.len() - self.max_history_size;
            self.history.drain(0..remove_count);
            self.current_index -= remove_count;
        }
    }

    fn replace_current_history(&mut self, entry: NavigationEntry) {
        if let Some(current) = self.history.get_mut(self.current_index) {
            *current = entry;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigation::router::Route;

    #[derive(Clone, Debug, PartialEq)]
    enum TestRoute {
        Home,
        User { id: String },
    }

    fn create_test_service() -> NavigationService<TestRoute, impl Fn(crate::navigation::router::RouteMatch<TestRoute>) -> Box<dyn Widget>> {
        let routes: Vec<Box<dyn crate::navigation::router::RouteMatcher<Route = TestRoute>>> = vec![
            Box::new(Route::new("/", TestRoute::Home)),
            Box::new(Route::new("/users/:id", TestRoute::User { id: "".to_string() })),
        ];

        let router_builder: Box<dyn Fn(TestRoute) -> Box<dyn Widget> + Send + Sync> = Box::new(|route: TestRoute| -> Box<dyn Widget> {
            match route {
                TestRoute::Home => Box::new(crate::display_widgets::Text::new("Home")),
                TestRoute::User { .. } => Box::new(crate::display_widgets::Text::new("User")),
            }
        });

        let router = Arc::new(Mutex::new(Router::new(routes, router_builder)));

        let service_builder = |route_match: crate::navigation::router::RouteMatch<TestRoute>| -> Box<dyn Widget> {
            match route_match.route {
                TestRoute::Home => Box::new(crate::display_widgets::Text::new("Home")),
                TestRoute::User { .. } => Box::new(crate::display_widgets::Text::new("User")),
            }
        };

        NavigationService::new(router, service_builder)
    }

    #[test]
    fn test_navigation_push() {
        let mut service = create_test_service();
        assert!(service.push("/").is_ok());
        assert_eq!(service.get_current_path(), Some("/".to_string()));
        assert_eq!(service.get_history().len(), 1);
    }

    #[test]
    fn test_navigation_push_with_params() {
        let mut service = create_test_service();
        let params = HashMap::from([("id".to_string(), "123".to_string())]);
        assert!(service.push_with_params("/users/:id", params).is_ok());
        assert_eq!(service.get_current_path(), Some("/users/123".to_string()));
    }

    #[test]
    fn test_navigation_pop() {
        let mut service = create_test_service();
        service.push("/").unwrap();
        service.push("/users/123").unwrap();

        assert!(service.can_go_back());
        assert!(service.pop().is_ok());
        assert_eq!(service.get_current_path(), Some("/".to_string()));
    }

    #[test]
    fn test_navigation_replace() {
        let mut service = create_test_service();
        service.push("/").unwrap();
        service.replace("/users/123").unwrap();

        assert_eq!(service.get_current_path(), Some("/users/123".to_string()));
        assert_eq!(service.get_history().len(), 1); // Should replace, not add
    }

    #[test]
    fn test_navigation_go() {
        let mut service = create_test_service();
        service.push("/").unwrap();
        service.push("/users/123").unwrap();
        service.push("/about").unwrap();

        service.go(-2).unwrap();
        assert_eq!(service.get_current_path(), Some("/".to_string()));

        service.go(1).unwrap();
        assert_eq!(service.get_current_path(), Some("/users/123".to_string()));
    }

    #[test]
    fn test_navigation_guards() {
        let mut service = create_test_service();

        struct TestGuard;
        impl NavigationGuard for TestGuard {
            fn can_activate(&self, _context: &NavigationGuardContext) -> bool {
                false // Block all navigation
            }
        }

        service.add_guard(Box::new(TestGuard));
        assert!(matches!(service.push("/"), Err(NavigationError::GuardBlocked(_))));
    }

    #[test]
    fn test_path_building() {
        let service = create_test_service();
        let params = HashMap::from([("id".to_string(), "123".to_string())]);
        let result = service.build_path_with_params("/users/:id", &params);
        assert_eq!(result, Ok("/users/123".to_string()));
    }

    #[test]
    fn test_path_building_missing_params() {
        let service = create_test_service();
        let params = HashMap::new();
        let result = service.build_path_with_params("/users/:id", &params);
        assert!(matches!(result, Err(NavigationError::InvalidParameters(_))));
    }
}