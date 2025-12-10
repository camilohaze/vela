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

    #[test]
    fn test_find_symbol_definition_function() {
        let server = create_test_server();

        // Test finding function definitions
        let code = r#"fn add(a: Number, b: Number) -> Number {
  return a + b
}

fn main() -> void {
  let result = add(1, 2)
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Test finding 'add' function definition
        let location = server.find_symbol_definition(code, "add", &uri);
        assert!(location.is_some(), "Should find definition for 'add'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 6); // "fn add" length

        // Test finding 'main' function definition
        let location = server.find_symbol_definition(code, "main", &uri);
        assert!(location.is_some(), "Should find definition for 'main'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 4);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 7); // "fn main" length
    }

    #[test]
    fn test_find_symbol_definition_variable() {
        let server = create_test_server();

        // Test finding variable definitions
        let code = r#"state count: Number = 0
let name: String = "Vela"

fn increment() -> void {
  count = count + 1
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Test finding 'count' state variable definition
        let location = server.find_symbol_definition(code, "count", &uri);
        assert!(location.is_some(), "Should find definition for 'count'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 13); // "state count:" length

        // Test finding 'name' let variable definition
        let location = server.find_symbol_definition(code, "name", &uri);
        assert!(location.is_some(), "Should find definition for 'name'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 1);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 10); // "let name:" length
    }

    #[test]
    fn test_find_symbol_definition_class() {
        let server = create_test_server();

        // Test finding class definitions
        let code = r#"class Person {
  constructor(name: String) {
    this.name = name
  }
}

class Animal {
  fn speak() -> String {
    return "Hello"
  }
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Test finding 'Person' class definition
        let location = server.find_symbol_definition(code, "Person", &uri);
        assert!(location.is_some(), "Should find definition for 'Person'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 12); // "class Person" length

        // Test finding 'Animal' class definition
        let location = server.find_symbol_definition(code, "Animal", &uri);
        assert!(location.is_some(), "Should find definition for 'Animal'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 6);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 12); // "class Animal" length
    }

    #[test]
    fn test_find_symbol_definition_interface() {
        let server = create_test_server();

        // Test finding interface definitions
        let code = r#"interface Drawable {
  fn draw() -> void
}

interface Printable {
  fn print() -> String
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Test finding 'Drawable' interface definition
        let location = server.find_symbol_definition(code, "Drawable", &uri);
        assert!(location.is_some(), "Should find definition for 'Drawable'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 0);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 17); // "interface Drawable" length

        // Test finding 'Printable' interface definition
        let location = server.find_symbol_definition(code, "Printable", &uri);
        assert!(location.is_some(), "Should find definition for 'Printable'");
        let location = location.unwrap();
        assert_eq!(location.range.start.line, 4);
        assert_eq!(location.range.start.character, 0);
        assert_eq!(location.range.end.character, 18); // "interface Printable" length
    }

    #[test]
    fn test_find_symbol_definition_not_found() {
        let server = create_test_server();

        // Test symbols that don't exist
        let code = r#"fn add(a: Number, b: Number) -> Number {
  return a + b
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Test finding non-existent symbol
        let location = server.find_symbol_definition(code, "multiply", &uri);
        assert!(location.is_none(), "Should not find definition for 'multiply'");

        let location = server.find_symbol_definition(code, "unknown", &uri);
        assert!(location.is_none(), "Should not find definition for 'unknown'");
    }

    #[test]
    fn test_analyze_definition_symbol() {
        let server = create_test_server();

        // Test definition analysis on sample Vela code
        let code = r#"fn add(a: Number, b: Number) -> Number {
  return a + b
}

state count: Number = 0

name: String = "Vela"

class Person {
  constructor(name: String) {
    this.name = name
  }
}

interface Printable {
  fn print() -> void
}

enum Color {
  Red,
  Green,
  Blue
}

type UserId = Number
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        let test_cases = vec![
            // Function definition
            (Position { line: 0, character: 3 }, Some(("add", 0, 3))), // "fn" keyword -> find "add"
            (Position { line: 0, character: 6 }, Some(("add", 0, 3))), // function name "add"

            // State variable
            (Position { line: 4, character: 6 }, Some(("count", 4, 6))), // state variable "count"

            // Immutable variable
            (Position { line: 6, character: 0 }, Some(("name", 6, 0))), // variable "name"

            // Class definition
            (Position { line: 8, character: 6 }, Some(("Person", 8, 6))), // class name "Person"

            // Interface definition
            (Position { line: 13, character: 10 }, Some(("Printable", 13, 10))), // interface name "Printable"

            // Enum definition
            (Position { line: 17, character: 5 }, Some(("Color", 17, 5))), // enum name "Color"

            // Type alias
            (Position { line: 22, character: 5 }, Some(("UserId", 22, 5))), // type alias "UserId"

            // Out of bounds
            (Position { line: 25, character: 0 }, None),
        ];

        for (position, expected) in test_cases {
            let location = server.analyze_definition_symbol(code, position, &uri);
            match expected {
                Some((symbol, expected_line, expected_char)) => {
                    assert!(location.is_some(), "Expected definition location for '{}' at position {:?}", symbol, position);
                    let location = location.unwrap();
                    // Verify the location points to the correct definition
                    assert_eq!(location.range.start.line, expected_line as u32, "Wrong line for symbol '{}'", symbol);
                    assert_eq!(location.range.start.character, expected_char as u32, "Wrong character for symbol '{}'", symbol);
                    assert_eq!(location.range.end.character, (expected_char + symbol.len()) as u32, "Wrong end character for symbol '{}'", symbol);
                }
                None => {
                    assert!(location.is_none(), "Expected no definition at position {:?}", position);
                }
            }
        }
    }

    #[test]
    fn test_extract_function_call_context() {
        let server = create_test_server();

        // Test cases for function call context extraction
        let test_cases = vec![
            ("print(", 6, Some(("print".to_string(), 0))),
            ("add(a, ", 7, Some(("add".to_string(), 1))),
            ("len(collection", 4, Some(("len".to_string(), 0))),
            ("fn test(", 7, None), // Not a function call
            ("let x = func(", 12, Some(("func".to_string(), 0))),
            ("call(param1, param2, ", 21, Some(("call".to_string(), 2))),
        ];

        for (line, pos, expected) in test_cases {
            let result = server.extract_function_call_context(line, pos);
            match expected {
                Some((expected_name, expected_param)) => {
                    assert!(result.is_some(), "Expected function call context for line: '{}', pos: {}", line, pos);
                    let context = result.unwrap();
                    assert_eq!(context.function_name, expected_name, "Function name mismatch for line: '{}'", line);
                    assert_eq!(context.active_parameter, expected_param, "Active parameter mismatch for line: '{}'", line);
                }
                None => {
                    assert!(result.is_none(), "Expected no function call context for line: '{}', pos: {}", line, pos);
                }
            }
        }
    }

    #[test]
    fn test_get_function_signatures() {
        let server = create_test_server();

        // Test getting signatures for known functions
        let test_cases = vec![
            ("print", Some("print(value: any) -> void")),
            ("len", Some("len(collection) -> Number")),
            ("add", Some("add(a: Number, b: Number) -> Number")),
            ("unknown", None),
        ];

        for (function_name, expected_label) in test_cases {
            let signatures = server.get_function_signatures(function_name);
            match expected_label {
                Some(expected) => {
                    assert!(signatures.is_some(), "Expected signatures for function: {}", function_name);
                    let sigs = signatures.unwrap();
                    assert_eq!(sigs.len(), 1, "Expected exactly one signature for: {}", function_name);
                    assert_eq!(sigs[0].label, expected, "Signature label mismatch for: {}", function_name);
                }
                None => {
                    assert!(signatures.is_none(), "Expected no signatures for unknown function: {}", function_name);
                }
            }
        }
    }

    #[test]
    fn test_calculate_active_parameter() {
        let server = create_test_server();

        let context = vela_lsp::server::FunctionCallContext {
            function_name: "test".to_string(),
            active_parameter: 2,
        };

        let active_param = server.calculate_active_parameter(&context, 10);
        assert_eq!(active_param, 2, "Active parameter should match context");
    }

    #[test]
    fn test_analyze_signature_help() {
        let server = create_test_server();

        // Test signature help analysis on sample code
        let code = r#"fn add(a: Number, b: Number) -> Number {
  return a + b
}

fn main() -> void {
  let result = add(1, 2)
  print("Result: ")
  print(result)
}
"#;

        let test_cases = vec![
            (Position { line: 5, character: 18 }, Some(("add", 0))), // add(1, 2) - first param
            (Position { line: 5, character: 20 }, Some(("add", 1))), // add(1, 2) - second param
            (Position { line: 6, character: 8 }, Some(("print", 0))), // print("Result: ") - first param
            (Position { line: 7, character: 8 }, Some(("print", 0))), // print(result) - first param
            (Position { line: 0, character: 10 }, None), // Not in function call
        ];

        for (position, expected) in test_cases {
            let signature_help = server.analyze_signature_help(code, position);
            match expected {
                Some((expected_func, expected_param)) => {
                    assert!(signature_help.is_some(), "Expected signature help at position {:?}", position);
                    let help = signature_help.unwrap();
                    assert_eq!(help.signatures.len(), 1, "Expected one signature");
                    assert_eq!(help.active_signature, Some(0), "Expected active signature 0");
                    assert_eq!(help.active_parameter, Some(expected_param as u32), "Active parameter mismatch");
                    assert!(help.signatures[0].label.contains(expected_func), "Signature should contain function name");
                }
                None => {
                    assert!(signature_help.is_none(), "Expected no signature help at position {:?}", position);
                }
            }
        }
    }

    #[test]
    fn test_signature_help_with_multiple_parameters() {
        let server = create_test_server();

        let code = "let result = add(a, b, c)";

        // Test different positions in the function call
        let test_cases = vec![
            (Position { line: 0, character: 15 }, 0), // add(a, b, c) - first param 'a'
            (Position { line: 0, character: 18 }, 1), // add(a, b, c) - second param 'b'
            (Position { line: 0, character: 21 }, 2), // add(a, b, c) - third param 'c'
        ];

        for (position, expected_active_param) in test_cases {
            let signature_help = server.analyze_signature_help(code, position);
            assert!(signature_help.is_some(), "Expected signature help at position {:?}", position);
            let help = signature_help.unwrap();
            assert_eq!(help.active_parameter, Some(expected_active_param), "Active parameter mismatch at {:?}", position);
        }
    }

    #[test]
    fn test_analyze_references_symbol() {
        let server = create_test_server();

        // Test analyzing symbol at position
        let test_cases = vec![
            ("fn add(a: Number) {}", Position::new(0, 3), "add"),
            ("let x: String = \"\"", Position::new(0, 5), "x"),
            ("state count = 0", Position::new(0, 8), "count"),
            ("class Person {}", Position::new(0, 7), "Person"),
            ("interface Drawable {}", Position::new(0, 11), "Drawable"),
        ];

        for (code, position, expected_symbol) in test_cases {
            let symbol = server.analyze_references_symbol(code, position);
            assert!(symbol.is_ok(), "Expected symbol at position {:?}", position);
            assert_eq!(symbol.unwrap(), expected_symbol, "Symbol mismatch at {:?}", position);
        }
    }

    #[test]
    fn test_find_symbol_references() {
        let server = create_test_server();

        // Test finding references in document
        let code = r#"
fn add(a: Number, b: Number) -> Number {
    return a + b
}

fn main() {
    let result = add(1, 2)
    let sum = add(3, 4)
    print(add(5, 6))
}
"#;

        let uri = Url::parse("file:///test.vela").unwrap();

        // Find references to "add"
        let references = server.find_symbol_references(code, "add", uri.clone());
        assert_eq!(references.len(), 4, "Expected 4 references to 'add'");

        // Check that all references are to the correct symbol
        for reference in &references {
            let line = code.lines().nth(reference.range.start.line as usize).unwrap();
            let symbol_in_line = &line[reference.range.start.character as usize..reference.range.end.character as usize];
            assert_eq!(symbol_in_line, "add", "Reference should be to 'add'");
        }
    }

    #[test]
    fn test_is_word_boundary() {
        let server = create_test_server();

        // Test word boundary detection
        let test_cases = vec![
            ("fn add(a: Number)", 3, 6, true),   // "add" is a whole word
            ("let x: String", 4, 5, true),      // "x" is a whole word
            ("state count =", 6, 11, true),     // "count" is a whole word
            ("class Person", 6, 12, true),      // "Person" is a whole word
            ("fn add(a: Number)", 0, 2, true),  // "fn" is a whole word
            ("let x: String", 0, 3, true),      // "let" is a whole word
            ("hello world", 0, 5, true),        // "hello" is a whole word
            ("hello world", 6, 11, true),       // "world" is a whole word
            ("hello_world", 0, 11, true),       // "hello_world" is a whole word
            ("hello world", 5, 6, false),       // space is not a word
            ("hello,world", 5, 6, false),       // comma is not part of word
            ("hello(world)", 5, 6, false),      // parenthesis is not part of word
        ];

        for (line, start, end, expected) in test_cases {
            let result = server.is_word_boundary(line, start, end);
            assert_eq!(result, expected, "Word boundary check failed for '{}'[{}..{}]", line, start, end);
        }
    }
}