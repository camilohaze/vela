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
    use lsp_types::{DiagnosticSeverity, Url};

    #[test]
    fn test_lsp_init() {
        let _server = init().expect("Failed to initialize language server");
        // TODO: Add more tests
    }

    #[test]
    fn test_analyze_diagnostics_brace_mismatch() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    println(\"hello\"\n}"; // Missing closing brace

        let diagnostics = server.analyze_diagnostics(content, &uri);

        assert!(!diagnostics.is_empty(), "Should detect brace mismatch");
        assert!(diagnostics.iter().any(|d| d.severity == Some(DiagnosticSeverity::ERROR)));
    }

    #[test]
    fn test_analyze_diagnostics_todo_comment() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    // TODO: implement this\n}";

        let diagnostics = server.analyze_diagnostics(content, &uri);

        assert!(!diagnostics.is_empty(), "Should detect TODO comment");
        assert!(diagnostics.iter().any(|d| d.severity == Some(DiagnosticSeverity::WARNING)));
    }

    #[test]
    fn test_analyze_diagnostics_long_line() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let long_line = "a".repeat(121); // Line longer than 120 characters
        let content = format!("fn test() {{\n    {}\n}}", long_line);

        let diagnostics = server.analyze_diagnostics(&content, &uri);

        assert!(!diagnostics.is_empty(), "Should detect long line");
        assert!(diagnostics.iter().any(|d| d.severity == Some(DiagnosticSeverity::WARNING)));
    }

    #[test]
    fn test_analyze_diagnostics_no_issues() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    println(\"hello\");\n}";

        let diagnostics = server.analyze_diagnostics(content, &uri);

        assert!(diagnostics.is_empty(), "Should not detect any issues in valid code");
    }
}