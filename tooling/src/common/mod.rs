/*!
Common utilities module
*/

pub mod error;
pub mod fs;
pub mod project;

// Re-export commonly used types
pub use error::{Error, Result};
pub use fs::FileSystem;
pub use project::Project;
