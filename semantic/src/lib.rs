/*!
# Vela Semantic Analyzer

Semantic analysis for the Vela programming language. This crate provides:

- **Symbol Tables**: Efficient symbol management with O(1) lookups
- **Scope Management**: Lexical scoping with shadowing support
- **Name Resolution**: Multi-pass name resolution with ancestor search
- **Error Reporting**: Comprehensive semantic error detection

## Features

- ✅ Thread-safe symbol tables with atomic IDs
- ✅ Lexical scoping with parent/child relationships
- ✅ Symbol resolution across nested scopes
- ✅ Semantic error collection with span tracking
- ✅ Support for closures and variable capture
- ✅ Module system integration ready

## Example Usage

```rust
use vela_semantic::{SemanticAnalyzer, SymbolKind, Span, ScopeKind};

fn main() {
    let mut analyzer = SemanticAnalyzer::new();
    
    // Define a variable in global scope
    let symbol_id = analyzer.define_symbol(
        "x".to_string(),
        SymbolKind::Variable { type_hint: Some("Number".to_string()) },
        Span::new(0, 1),
    ).unwrap();
    
    // Enter a function scope
    analyzer.enter_scope(ScopeKind::Function);
    
    // Lookup the variable (finds it in parent scope)
    match analyzer.lookup_symbol("x", Span::new(10, 11)) {
        Ok(id) => println!("Found symbol: {:?}", id),
        Err(error) => eprintln!("Error: {}", error),
    }
    
    // Exit the scope
    analyzer.exit_scope();
    
    // Finalize and get results
    let result = analyzer.finalize();
    println!("Symbols: {}", result.symbol_table.len());
    println!("Errors: {}", result.errors.len());
}
```

## Architecture

The semantic analyzer is built on three core components:

### 1. Symbol Table

Manages all symbols (variables, functions, classes) with:
- HashMap-based O(1) lookups
- Unique SymbolId per symbol
- Bidirectional name ↔ symbol mapping

### 2. Scope Manager

Manages lexical scopes with:
- Tree structure (parent/child links)
- Support for shadowing
- Multiple scope kinds (Global, Function, Block, Class, etc.)

### 3. Semantic Analyzer

Orchestrates analysis with:
- Two-pass symbol resolution
- Error collection (non-fatal)
- Closure capture detection
- Module import resolution

## Performance

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Symbol definition | O(1) | HashMap insert |
| Symbol lookup | O(d) | d = scope depth (typically < 10) |
| Scope creation | O(1) | Atomic counter |
| Full analysis | O(n) | n = AST nodes |

## Error Handling

Errors are collected during analysis and don't stop the process:

```rust
let mut analyzer = SemanticAnalyzer::new();

// This will add an error but continue
analyzer.define_symbol("x".to_string(), SymbolKind::Variable { type_hint: None }, Span::new(0, 1)).unwrap();
analyzer.define_symbol("x".to_string(), SymbolKind::Variable { type_hint: None }, Span::new(10, 11)).unwrap_err();

// Check errors
assert!(analyzer.has_errors());
assert_eq!(analyzer.errors().len(), 1);
```
*/

pub mod analyzer;
pub mod error;
pub mod scope;
pub mod symbol;

pub use analyzer::{AnalysisResult, SemanticAnalyzer};
pub use error::SemanticError;
pub use scope::{Scope, ScopeId, ScopeKind, ScopeManager};
pub use symbol::{Span, Symbol, SymbolId, SymbolKind, SymbolTable};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_analysis_workflow() {
        let mut analyzer = SemanticAnalyzer::new();

        // Define a function
        let func_id = analyzer
            .define_symbol(
                "greet".to_string(),
                SymbolKind::Function {
                    params: vec!["name".to_string()],
                    return_type: Some("String".to_string()),
                },
                Span::new(0, 5),
            )
            .unwrap();

        // Enter function scope
        analyzer.enter_function(func_id);

        // Define parameter
        analyzer
            .define_symbol(
                "name".to_string(),
                SymbolKind::Parameter,
                Span::new(6, 10),
            )
            .unwrap();

        // Define local variable
        analyzer
            .define_symbol(
                "message".to_string(),
                SymbolKind::Variable {
                    type_hint: Some("String".to_string()),
                },
                Span::new(20, 27),
            )
            .unwrap();

        // Lookup parameter (should work)
        let param_lookup = analyzer.lookup_symbol("name", Span::new(30, 34));
        assert!(param_lookup.is_ok());

        // Exit function
        analyzer.exit_function();

        // Lookup parameter outside function (should fail and add error)
        let outside_lookup = analyzer.lookup_symbol("name", Span::new(50, 54));
        if let Err(error) = outside_lookup {
            analyzer.add_error(error);
        }

        // Finalize
        let result = analyzer.finalize();
        assert_eq!(result.symbol_table.len(), 3); // function + parameter + local
        assert_eq!(result.errors.len(), 1); // outside lookup error
    }

    #[test]
    fn test_nested_scopes_with_shadowing() {
        let mut analyzer = SemanticAnalyzer::new();

        // Global x
        let global_x = analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        // Enter function
        analyzer.enter_scope(ScopeKind::Function);

        // Shadow x in function
        let func_x = analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(10, 11),
            )
            .unwrap();

        // Lookup should find function's x
        let lookup = analyzer.lookup_symbol("x", Span::new(15, 16)).unwrap();
        assert_eq!(lookup, func_x);
        assert_ne!(lookup, global_x);

        // Exit function
        analyzer.exit_scope();

        // Lookup should find global x
        let lookup = analyzer.lookup_symbol("x", Span::new(25, 26)).unwrap();
        assert_eq!(lookup, global_x);

        let result = analyzer.finalize();
        assert!(result.errors.is_empty()); // No errors
    }
}
