//! Unit tests for LSP server functionality

use std::collections::HashMap;
use lsp_types::{Position, Url, HoverParams, TextDocumentPositionParams};
use vela_lsp::server::{LanguageServer, DocumentStore};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_server() -> LanguageServer {
        // For testing, we'll create a minimal server instance
        // In a real test, you'd mock the connection
        // For now, we'll test the analysis functions directly
        unimplemented!("Full server testing requires mocking Connection")
    }

    #[test]
    fn test_extract_word_at_position() {
        let server = create_test_server();

        // Test cases for word extraction
        let test_cases = vec![
            ("fn add(a: Number", 3, Some("add".to_string())),
            ("let x: String", 5, Some("x".to_string())),
            ("state count =", 8, Some("count".to_string())),
            ("if condition", 4, Some("condition".to_string())),
            ("class Person", 7, Some("Person".to_string())),
            ("", 0, None),
            ("fn", 2, Some("fn".to_string())),
        ];

        for (line, pos, expected) in test_cases {
            let result = server.extract_word_at_position(line, pos);
            assert_eq!(result, expected, "Failed for line: '{}', pos: {}", line, pos);
        }
    }

    #[test]
    fn test_generate_hover_for_keywords() {
        let server = create_test_server();

        // Test hover for keywords
        let keywords = vec![
            ("fn", true),
            ("let", true),
            ("state", true),
            ("if", true),
            ("match", true),
            ("class", true),
            ("interface", true),
            ("public", true),
            ("return", true),
        ];

        for (word, should_have_hover) in keywords {
            let hover = server.generate_hover_for_word(word);
            if should_have_hover {
                assert!(hover.is_some(), "Expected hover for keyword: {}", word);
                let hover = hover.unwrap();
                match hover.contents {
                    lsp_types::HoverContents::Markup(content) => {
                        assert!(content.value.contains(word), "Hover should contain the word: {}", word);
                        assert!(content.value.contains("**"), "Hover should be formatted as markdown");
                    }
                    _ => panic!("Expected Markup content"),
                }
            } else {
                assert!(hover.is_none(), "Expected no hover for: {}", word);
            }
        }
    }

    #[test]
    fn test_generate_hover_for_types() {
        let server = create_test_server();

        // Test hover for types
        let types = vec![
            ("String", "Text string type"),
            ("Number", "Integer type"),
            ("Float", "Floating point type"),
            ("Bool", "Boolean type"),
            ("void", "No return type"),
        ];

        for (type_name, description) in types {
            let hover = server.generate_hover_for_word(type_name);
            assert!(hover.is_some(), "Expected hover for type: {}", type_name);
            let hover = hover.unwrap();
            match hover.contents {
                lsp_types::HoverContents::Markup(content) => {
                    assert!(content.value.contains(type_name), "Hover should contain the type: {}", type_name);
                    assert!(content.value.contains(description), "Hover should contain description: {}", description);
                }
                _ => panic!("Expected Markup content"),
            }
        }
    }

    #[test]
    fn test_generate_hover_for_functions() {
        let server = create_test_server();

        // Test hover for built-in functions
        let functions = vec![
            ("print", "Print to console"),
            ("len", "Get collection length"),
        ];

        for (func_name, description) in functions {
            let hover = server.generate_hover_for_word(func_name);
            assert!(hover.is_some(), "Expected hover for function: {}", func_name);
            let hover = hover.unwrap();
            match hover.contents {
                lsp_types::HoverContents::Markup(content) => {
                    assert!(content.value.contains(func_name), "Hover should contain the function: {}", func_name);
                    assert!(content.value.contains(description), "Hover should contain description: {}", description);
                }
                _ => panic!("Expected Markup content"),
            }
        }
    }

    #[test]
    fn test_generate_hover_for_unknown_words() {
        let server = create_test_server();

        // Test hover for unknown words
        let unknown_words = vec![
            "unknown_function",
            "custom_variable",
            "nonexistent_type",
        ];

        for word in unknown_words {
            let hover = server.generate_hover_for_word(word);
            assert!(hover.is_none(), "Expected no hover for unknown word: {}", word);
        }
    }

    #[test]
    fn test_analyze_hover_symbol() {
        let server = create_test_server();

        // Test hover analysis on sample code
        let code = r#"fn add(a: Number, b: Number) -> Number {
  return a + b
}

state count: Number = 0
let name: String = "Vela"

if count > 0 {
  print(name)
}
"#;

        let test_cases = vec![
            (Position { line: 0, character: 3 }, Some("fn")), // "fn" keyword
            (Position { line: 0, character: 6 }, Some("add")), // function name
            (Position { line: 0, character: 11 }, Some("Number")), // type
            (Position { line: 2, character: 6 }, Some("state")), // state keyword
            (Position { line: 2, character: 11 }, Some("count")), // variable name (no hover)
            (Position { line: 2, character: 19 }, Some("Number")), // type
            (Position { line: 3, character: 4 }, Some("let")), // let keyword
            (Position { line: 3, character: 9 }, Some("name")), // variable name (no hover)
            (Position { line: 3, character: 16 }, Some("String")), // type
            (Position { line: 5, character: 2 }, Some("if")), // if keyword
            (Position { line: 6, character: 2 }, Some("print")), // function call
            (Position { line: 6, character: 9 }, Some("name")), // variable reference (no hover)
        ];

        for (position, expected_word) in test_cases {
            let hover = server.analyze_hover_symbol(code, position);
            match expected_word {
                Some(word) => {
                    assert!(hover.is_some(), "Expected hover at position {:?}", position);
                    let hover = hover.unwrap();
                    match hover.contents {
                        lsp_types::HoverContents::Markup(content) => {
                            assert!(content.value.contains(word), "Hover at {:?} should contain '{}'", position, word);
                        }
                        _ => panic!("Expected Markup content"),
                    }
                }
                None => {
                    assert!(hover.is_none(), "Expected no hover at position {:?}", position);
                }
            }
        }
    }
}