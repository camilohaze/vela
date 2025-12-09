/*!
# Vela Language Server

Language Server Protocol implementation for Vela, providing IDE features
like syntax highlighting, error diagnostics, code completion, and refactoring.
*/

pub mod server;
pub mod handlers;
pub mod diagnostics;
pub mod completion;

/// Re-export main server type
pub use server::LanguageServer;

/// Initialize the language server
pub fn init() -> anyhow::Result<LanguageServer> {
    LanguageServer::new()
}

#[cfg(test)]
mod tests {
    use super::init;

    #[test]
    fn test_lsp_init() {
        let _server = init().expect("Failed to initialize language server");
        // TODO: Add more tests
    }
}