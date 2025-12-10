/*!
# LSP Integration Tests

Integration tests for the Vela Language Server Protocol implementation.
These tests verify the full LSP protocol flow and end-to-end functionality.
*/

use vela_lsp::init;
use lsp_types::*;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test full LSP initialization sequence
    #[test]
    fn test_lsp_initialization_sequence() {
        // This would require starting the LSP server as a separate process
        // For now, we'll test the initialization logic directly
        let server = crate::init().expect("Failed to initialize server");

        // Test that server capabilities are properly set
        let capabilities = server.server_capabilities();
        assert!(capabilities.text_document_sync.is_some());
        assert!(capabilities.completion_provider.is_some());
        assert!(capabilities.hover_provider.is_some());
        assert!(capabilities.definition_provider.is_some());
        assert!(capabilities.rename_provider.is_some());
        assert!(capabilities.diagnostic_provider.is_some());
    }

    /// Test textDocument/didOpen notification handling
    #[test]
    fn test_document_open_notification() {
        let mut server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn hello() {\n    println(\"Hello, Vela!\");\n}";

        // Simulate didOpen notification
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: "vela".to_string(),
                version: 1,
                text: content.to_string(),
            },
        };

        // This would normally be handled by the message loop
        // For testing, we directly call the document update method
        server.update_document(uri.clone(), content.to_string());

        // Verify document was stored
        assert!(server.has_document(&uri));
    }

    /// Test textDocument/didChange notification handling
    #[test]
    fn test_document_change_notification() {
        let mut server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();

        // Initial content
        let initial_content = "fn hello() {\n    println(\"Hello\");\n}";
        server.update_document(uri.clone(), initial_content.to_string());

        // Simulate didChange notification
        let new_content = "fn hello() {\n    println(\"Hello, World!\");\n}";
        server.update_document(uri.clone(), new_content.to_string());

        // Verify document was updated
        let stored_content = server.get_document_content(&uri).unwrap();
        assert_eq!(stored_content, new_content);
    }

    /// Test complete LSP request-response cycle for completion
    #[test]
    fn test_completion_request_response_cycle() {
        let server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    pri\n}";

        // Update document
        server.update_document(uri.clone(), content.to_string());

        // Create completion request
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 1, character: 4 }, // Position of "pri"
            },
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        // Execute completion
        let result = server.get_completions(&params);

        // Verify response
        assert!(!result.items.is_empty(), "Should return completion items");
        assert!(result.items.iter().any(|item| item.label.contains("println")));
    }

    /// Test complete LSP request-response cycle for hover
    #[test]
    fn test_hover_request_response_cycle() {
        let server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    println(\"hello\");\n}";

        // Update document
        server.update_document(uri.clone(), content.to_string());

        // Create hover request
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 0, character: 0 }, // Position of "f" in "fn"
            },
            work_done_progress_params: Default::default(),
        };

        // Execute hover
        let result = server.get_hover(&params);

        // Verify response
        assert!(result.is_some(), "Should return hover information");
        let hover = result.unwrap();
        match hover.contents {
            HoverContents::Markup(content) => {
                assert!(!content.value.is_empty(), "Hover content should not be empty");
                // Could check for markdown formatting if needed
            }
            _ => panic!("Expected Markup content for hover"),
        }
    }

    /// Test complete LSP request-response cycle for definition
    #[test]
    fn test_definition_request_response_cycle() {
        let server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test_function() {\n    test_function();\n}";

        // Update document
        server.update_document(uri.clone(), content.to_string());

        // Create definition request
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 1, character: 4 }, // Position of "test_function" call
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        // Execute goto definition
        let result = server.get_definition(&params);

        // Verify response
        assert!(result.is_some(), "Should return definition location");
        let definition = result.unwrap();
        match definition {
            GotoDefinitionResponse::Scalar(location) => {
                assert_eq!(location.uri, uri);
                assert_eq!(location.range.start.line, 0); // Should point to function definition
            }
            _ => panic!("Expected scalar location"),
        }
    }

    /// Test complete LSP request-response cycle for rename
    #[test]
    fn test_rename_request_response_cycle() {
        let server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn old_name() {\n    old_name();\n}";

        // Update document
        server.update_document(uri.clone(), content.to_string());

        // Create rename request
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position { line: 0, character: 3 }, // Position of "old_name" in definition
            },
            new_name: "new_name".to_string(),
            work_done_progress_params: Default::default(),
        };

        // Execute rename
        let result = server.compute_rename(&params).expect("Rename should succeed");

        // Verify response
        assert!(result.changes.is_some(), "Should have changes");
        let changes = result.changes.as_ref().unwrap();
        assert!(changes.contains_key(&uri), "Should have changes for the document");

        let text_edits = &changes[&uri];
        assert_eq!(text_edits.len(), 2, "Should have 2 edits (definition and call)");
        assert!(text_edits.iter().all(|edit| edit.new_text == "new_name"));
    }

    /// Test diagnostics are published correctly
    #[test]
    fn test_diagnostics_publishing() {
        let server = crate::init().expect("Failed to initialize server");
        let uri = Url::parse("file:///test.vela").unwrap();
        let content = "fn test() {\n    println(\"hello\")\n"; // Missing closing brace

        // Update document
        server.update_document(uri.clone(), content.to_string());

        // Get diagnostics
        let diagnostics = server.analyze_diagnostics(content, &uri);

        // Verify diagnostics
        assert!(!diagnostics.is_empty(), "Should detect syntax errors");
        assert!(diagnostics.iter().any(|d| d.severity == Some(DiagnosticSeverity::ERROR)));
    }

    /// Test error handling for invalid requests
    #[test]
    fn test_error_handling_invalid_requests() {
        let server = crate::init().expect("Failed to initialize server");

        // Test rename with non-existent document
        let uri = Url::parse("file:///nonexistent.vela").unwrap();
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line: 0, character: 0 },
            },
            new_name: "new_name".to_string(),
            work_done_progress_params: Default::default(),
        };

        let result = server.compute_rename(&params);
        assert!(result.is_ok(), "Should handle gracefully");

        let workspace_edit = result.unwrap();
        assert!(workspace_edit.changes.as_ref().unwrap().is_empty(), "Should return empty changes");
    }

    /// Test concurrent document updates
    #[test]
    fn test_concurrent_document_operations() {
        let server = crate::init().expect("Failed to initialize server");
        let uri1 = Url::parse("file:///test1.vela").unwrap();
        let uri2 = Url::parse("file:///test2.vela").unwrap();

        // Update multiple documents
        server.update_document(uri1.clone(), "fn test1() {}".to_string());
        server.update_document(uri2.clone(), "fn test2() {}".to_string());

        // Verify both documents are stored
        assert!(server.has_document(&uri1));
        assert!(server.has_document(&uri2));

        // Test operations on both documents
        let completion_params1 = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri1.clone() },
                position: Position { line: 0, character: 0 },
            },
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let completion_params2 = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri2.clone() },
                position: Position { line: 0, character: 0 },
            },
            context: Some(CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            }),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let result1 = server.get_completions(&completion_params1);
        let result2 = server.get_completions(&completion_params2);

        assert!(!result1.items.is_empty());
        assert!(!result2.items.is_empty());
    }
}