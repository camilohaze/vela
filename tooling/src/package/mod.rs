/*!
Package management module
*/

pub mod manifest;
pub mod resolver;
pub mod registry;
pub mod version;

// Re-export main types
pub use manifest::Manifest;
pub use resolver::DependencyResolver;
pub use registry::Registry;
pub use version::Version;
