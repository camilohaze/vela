//! # Navigation Module - Routing and Navigation for Vela UI
//!
//! This module provides routing capabilities for Vela applications,
//! including declarative route definitions, dynamic parameters,
//! and programmatic navigation.

pub mod router;
pub mod service;

pub use router::{Router, Route, RouteMatch, RouteMatcher, NavigationContext};
pub use service::{NavigationService, NavigationEntry, NavigationError, NavigationGuard, NavigationGuardContext};