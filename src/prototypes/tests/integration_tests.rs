/// Integration tests for prototypes
///
/// JIRA: VELA-565 (Sprint 4)
/// Tests completos end-to-end del lexer + parser

use vela_prototypes::parse_source;

#[test]
fn test_hello_world() {
    let source = r#"
        fn main() {
            return "Hello, World!";
        }
    "#;

    let program = parse_source(source);
    assert!(program.is_ok());

    let program = program.unwrap();
    assert_eq!(program.stmts.len(), 1);
}

#[test]
fn test_fibonacci() {
    let source = r#"
        fn fibonacci(n) {
            if n < 2 {
                return n;
            } else {
                return fibonacci(n - 1) + fibonacci(n - 2);
            }
        }

        let result = fibonacci(10);
    "#;

    let program = parse_source(source);
    assert!(program.is_ok());

    let program = program.unwrap();
    assert_eq!(program.stmts.len(), 2); // fn + let
}

#[test]
fn test_complex_arithmetic() {
    let source = r#"
        let x = 2 + 3 * 4 - 5 / 2;
        let y = 10 == 5 + 5;
        let z = 20 < 30;
    "#;

    let program = parse_source(source);
    assert!(program.is_ok());

    let program = program.unwrap();
    assert_eq!(program.stmts.len(), 3);
}

#[test]
fn test_nested_if() {
    let source = r#"
        let x = if true {
            let inner = if false {
                let deep = 1;
                deep;
            } else {
                let deep = 2;
                deep;
            };
            inner;
        } else {
            let outer = 3;
            outer;
        };
    "#;

    let program = parse_source(source);
    assert!(program.is_ok());
}

#[test]
fn test_function_with_multiple_params() {
    let source = r#"
        fn add(a, b) {
            return a + b;
        }

        fn multiply(x, y) {
            return x * y;
        }

        let result = add(multiply(2, 3), multiply(4, 5));
    "#;

    let program = parse_source(source);
    assert!(program.is_ok());

    let program = program.unwrap();
    assert_eq!(program.stmts.len(), 3); // 2 fn + 1 let
}

#[test]
fn test_error_invalid_syntax() {
    let source = "let x = ;"; // Missing value

    let program = parse_source(source);
    assert!(program.is_err());
}

#[test]
fn test_error_missing_semicolon() {
    let source = "let x = 42"; // Missing semicolon

    let program = parse_source(source);
    assert!(program.is_err());
}
