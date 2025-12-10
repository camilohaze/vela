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
    use lsp_types::{DiagnosticSeverity, Url, RenameParams, Position};

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

    #[test]
    fn test_compute_rename_variable() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    let old_name = 42;\n    println(old_name);\n}";
        
        // Update document store
        server.update_document(uri.clone(), content.to_string());

        let params = RenameParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 1, character: 8 }, // Position of "old_name"
            },
            new_name: "new_name".to_string(),
            work_done_progress_params: Default::default(),
        };

        let workspace_edit = server.compute_rename(&params).expect("Rename should succeed");

        assert!(workspace_edit.changes.is_some(), "Should have changes");
        let changes = workspace_edit.changes.as_ref().unwrap();
        assert!(changes.contains_key(&uri), "Should have changes for the document");
        
        let text_edits = &changes[&uri];
        assert_eq!(text_edits.len(), 2, "Should have 2 edits (declaration and usage)");
        
        // Check that both occurrences are replaced
        assert!(text_edits.iter().all(|edit| edit.new_text == "new_name"));
    }

    #[test]
    fn test_compute_rename_function() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn old_func() {\n    return 42;\n}\n\nfn main() {\n    old_func();\n}";
        
        // Update document store
        server.update_document(uri.clone(), content.to_string());

        let params = RenameParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 0, character: 3 }, // Position of "old_func"
            },
            new_name: "new_func".to_string(),
            work_done_progress_params: Default::default(),
        };

        let workspace_edit = server.compute_rename(&params).expect("Rename should succeed");

        assert!(workspace_edit.changes.is_some(), "Should have changes");
        let changes = workspace_edit.changes.as_ref().unwrap();
        assert!(changes.contains_key(&uri), "Should have changes for the document");
        
        let text_edits = &changes[&uri];
        assert_eq!(text_edits.len(), 2, "Should have 2 edits (definition and call)");
        
        // Check that both occurrences are replaced
        assert!(text_edits.iter().all(|edit| edit.new_text == "new_func"));
    }

    #[test]
    fn test_compute_rename_no_symbol() {
        let server = init().expect("Failed to initialize language server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    println(\"hello\");\n}";
        
        // Update document store
        server.update_document(uri.clone(), content.to_string());

        let params = RenameParams {
            text_document_position: lsp_types::TextDocumentPositionParams {
                text_document: lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 0, character: 0 }, // Position with no symbol
            },
            new_name: "new_name".to_string(),
            work_done_progress_params: Default::default(),
        };

        let workspace_edit = server.compute_rename(&params).expect("Rename should succeed");

        // Should return empty workspace edit
        assert!(workspace_edit.changes.as_ref().unwrap().is_empty(), "Should have no changes");
    }
}