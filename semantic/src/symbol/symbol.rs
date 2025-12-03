/// Symbol identifier - unique ID for each symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

/// Span represents a location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Kind of symbol
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// Variable: x: Number = 10
    Variable {
        type_hint: Option<String>,
    },
    /// Function: fn add(a, b) -> Number
    Function {
        params: Vec<String>,
        return_type: Option<String>,
    },
    /// Class: class Person { }
    Class {
        base_class: Option<String>,
    },
    /// Module: module auth
    Module,
    /// Import: import 'package:http'
    Import {
        source: String,
    },
    /// Parameter: fn greet(name: String)
    Parameter,
    /// Method: method inside class
    Method {
        params: Vec<String>,
        return_type: Option<String>,
    },
}

/// Symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub span: Span,
    pub scope_id: crate::scope::ScopeId,
    pub is_mutable: bool,
    pub is_captured: bool,
}

impl Symbol {
    pub fn new(
        id: SymbolId,
        name: String,
        kind: SymbolKind,
        span: Span,
        scope_id: crate::scope::ScopeId,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            span,
            scope_id,
            is_mutable: false,
            is_captured: false,
        }
    }

    /// Check if symbol is a variable
    pub fn is_variable(&self) -> bool {
        matches!(self.kind, SymbolKind::Variable { .. })
    }

    /// Check if symbol is a function
    pub fn is_function(&self) -> bool {
        matches!(self.kind, SymbolKind::Function { .. })
    }

    /// Check if symbol is a class
    pub fn is_class(&self) -> bool {
        matches!(self.kind, SymbolKind::Class { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_id_equality() {
        let id1 = SymbolId(1);
        let id2 = SymbolId(1);
        let id3 = SymbolId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 10);
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new(
            SymbolId(1),
            "x".to_string(),
            SymbolKind::Variable { type_hint: Some("Number".to_string()) },
            Span::new(0, 1),
            crate::scope::ScopeId(0),
        );

        assert_eq!(symbol.id, SymbolId(1));
        assert_eq!(symbol.name, "x");
        assert!(symbol.is_variable());
        assert!(!symbol.is_function());
        assert!(!symbol.is_class());
    }

    #[test]
    fn test_symbol_kind_variable() {
        let kind = SymbolKind::Variable {
            type_hint: Some("String".to_string()),
        };
        
        match kind {
            SymbolKind::Variable { type_hint } => {
                assert_eq!(type_hint, Some("String".to_string()));
            }
            _ => panic!("Expected Variable kind"),
        }
    }

    #[test]
    fn test_symbol_kind_function() {
        let kind = SymbolKind::Function {
            params: vec!["a".to_string(), "b".to_string()],
            return_type: Some("Number".to_string()),
        };
        
        match kind {
            SymbolKind::Function { params, return_type } => {
                assert_eq!(params.len(), 2);
                assert_eq!(return_type, Some("Number".to_string()));
            }
            _ => panic!("Expected Function kind"),
        }
    }

    #[test]
    fn test_symbol_mutability() {
        let mut symbol = Symbol::new(
            SymbolId(1),
            "count".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 5),
            crate::scope::ScopeId(0),
        );

        assert!(!symbol.is_mutable);
        symbol.is_mutable = true;
        assert!(symbol.is_mutable);
    }

    #[test]
    fn test_symbol_captured() {
        let mut symbol = Symbol::new(
            SymbolId(1),
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            crate::scope::ScopeId(0),
        );

        assert!(!symbol.is_captured);
        symbol.is_captured = true;
        assert!(symbol.is_captured);
    }
}
