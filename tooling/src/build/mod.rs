/*!
Build system module
*/

pub mod config;
pub mod executor;
pub mod graph;
pub mod cache;
#[cfg(test)]
pub mod tests_ios_integration;
#[cfg(test)]
pub mod tests_android_integration;

// Re-export main types
pub use config::BuildConfig;
pub use executor::{BuildExecutor, BuildResult};
pub use graph::{BuildGraph, ModuleId};
pub use cache::BuildCache;
