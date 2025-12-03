/*!
# Vela Standard Library

The Vela standard library provides essential functionality for Vela programs,
including collections, I/O, networking, and utility functions.
*/

pub mod collections;
pub mod io;
pub mod net;
pub mod time;
pub mod math;
pub mod string;
pub mod fs;

/// Re-export commonly used types
pub use collections::{List, Dict, Set};
pub use io::{stdin, stdout, stderr};
pub use time::DateTime;

/// Initialize the standard library
pub fn init() {
    // Initialize standard library modules
    collections::init();
    io::init();
    net::init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_stdlib_init() {
        init(); // Should not panic
    }
}