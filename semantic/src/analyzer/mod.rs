use crate::error::SemanticError;
use crate::scope::{ScopeId, ScopeKind, ScopeManager};
use crate::symbol::{Span, SymbolId, SymbolKind, SymbolTable};

/// Result of semantic analysis
pub struct AnalysisResult {
    pub symbol_table: SymbolTable,
    pub scope_manager: ScopeManager,
    pub errors: Vec<SemanticError>,
}

/// Semantic analyzer for Vela programs
pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
    scope_manager: ScopeManager,
    errors: Vec<SemanticError>,
    current_function: Option<SymbolId>,
}

impl SemanticAnalyzer {
    /// Create a new semantic analyzer
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            scope_manager: ScopeManager::new(),
            errors: Vec::new(),
            current_function: None,
        }
    }

    /// Define a new symbol in the current scope
    pub fn define_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
    ) -> Result<SymbolId, SemanticError> {
        let current_scope = self.scope_manager.current_scope();

        // Check if already defined in current scope
        if let Some(existing) = self.symbol_table.lookup_in_scope(&name, current_scope) {
            let error = match &kind {
                SymbolKind::Function { .. } => SemanticError::FunctionAlreadyDefined {
                    name,
                    original: existing.span,
                    duplicate: span,
                },
                SymbolKind::Class { .. } => SemanticError::ClassAlreadyDefined {
                    name,
                    original: existing.span,
                    duplicate: span,
                },
                _ => SemanticError::AlreadyDefined {
                    name,
                    original: existing.span,
                    duplicate: span,
                },
            };
            self.errors.push(error.clone());
            return Err(error);
        }

        // Define the symbol
        let symbol_id = self.symbol_table.define(name, kind, span, current_scope);
        self.scope_manager
            .add_symbol_to_scope(current_scope, symbol_id);

        Ok(symbol_id)
    }

    /// Lookup a symbol by name (searches current scope and ancestors)
    pub fn lookup_symbol(&self, name: &str, span: Span) -> Result<SymbolId, SemanticError> {
        let current_scope = self.scope_manager.current_scope();

        // Search current scope
        if let Some(symbol) = self.symbol_table.lookup_in_scope(name, current_scope) {
            return Ok(symbol.id);
        }

        // Search ancestor scopes
        for ancestor in self.scope_manager.ancestors(current_scope) {
            if let Some(symbol) = self.symbol_table.lookup_in_scope(name, ancestor) {
                return Ok(symbol.id);
            }
        }

        // Not found
        let error = SemanticError::UndefinedVariable {
            name: name.to_string(),
            span,
        };
        Err(error)
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let scope_id = self.scope_manager.create_scope(kind, None);
        self.scope_manager.enter_scope(scope_id);
        scope_id
    }

    /// Exit the current scope
    pub fn exit_scope(&mut self) {
        self.scope_manager.exit_scope();
    }

    /// Get the current scope
    pub fn current_scope(&self) -> ScopeId {
        self.scope_manager.current_scope()
    }

    /// Mark a variable as mutable (state keyword)
    pub fn mark_mutable(&mut self, symbol_id: SymbolId) {
        self.symbol_table.mark_mutable(symbol_id);
    }

    /// Mark a variable as captured by closure
    pub fn mark_captured(&mut self, symbol_id: SymbolId) {
        self.symbol_table.mark_captured(symbol_id);
    }

    /// Enter a function scope
    pub fn enter_function(&mut self, function_id: SymbolId) {
        self.current_function = Some(function_id);
        self.enter_scope(ScopeKind::Function);
    }

    /// Exit a function scope
    pub fn exit_function(&mut self) {
        self.exit_scope();
        self.current_function = None;
    }

    /// Get the current function being analyzed
    pub fn current_function(&self) -> Option<SymbolId> {
        self.current_function
    }

    /// Add an error
    pub fn add_error(&mut self, error: SemanticError) {
        self.errors.push(error);
    }

    /// Get all errors
    pub fn errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Check if analysis has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Finalize analysis and return result
    pub fn finalize(self) -> AnalysisResult {
        AnalysisResult {
            symbol_table: self.symbol_table,
            scope_manager: self.scope_manager,
            errors: self.errors,
        }
    }

    /// Get a reference to the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Get a reference to the scope manager
    pub fn scope_manager(&self) -> &ScopeManager {
        &self.scope_manager
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new();
        assert!(!analyzer.has_errors());
        assert_eq!(analyzer.errors().len(), 0);
        assert_eq!(analyzer.current_scope(), ScopeId(0));
    }

    #[test]
    fn test_define_variable() {
        let mut analyzer = SemanticAnalyzer::new();

        let result = analyzer.define_symbol(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
        );

        assert!(result.is_ok());
        assert!(!analyzer.has_errors());
    }

    #[test]
    fn test_already_defined_error() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        let result = analyzer.define_symbol(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(10, 11),
        );

        assert!(result.is_err());
        assert!(analyzer.has_errors());
        assert_eq!(analyzer.errors().len(), 1);
    }

    #[test]
    fn test_lookup_symbol() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        let result = analyzer.lookup_symbol("x", Span::new(5, 6));
        assert!(result.is_ok());

        let not_found = analyzer.lookup_symbol("y", Span::new(10, 11));
        assert!(not_found.is_err());
    }

    #[test]
    fn test_scope_management() {
        let mut analyzer = SemanticAnalyzer::new();
        let global = analyzer.current_scope();

        let func_scope = analyzer.enter_scope(ScopeKind::Function);
        assert_ne!(func_scope, global);
        assert_eq!(analyzer.current_scope(), func_scope);

        analyzer.exit_scope();
        assert_eq!(analyzer.current_scope(), global);
    }

    #[test]
    fn test_nested_scope_lookup() {
        let mut analyzer = SemanticAnalyzer::new();

        // Define in global
        analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        // Enter function scope
        analyzer.enter_scope(ScopeKind::Function);

        // Should find x from parent scope
        let result = analyzer.lookup_symbol("x", Span::new(10, 11));
        assert!(result.is_ok());
    }

    #[test]
    fn test_shadowing() {
        let mut analyzer = SemanticAnalyzer::new();

        // Define x in global
        analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        // Enter function scope
        analyzer.enter_scope(ScopeKind::Function);

        // Define x in function (shadowing)
        let result = analyzer.define_symbol(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(10, 11),
        );

        // Should succeed (shadowing is allowed in different scopes)
        assert!(result.is_ok());
        assert!(!analyzer.has_errors());
    }

    #[test]
    fn test_mark_mutable() {
        let mut analyzer = SemanticAnalyzer::new();

        let symbol_id = analyzer
            .define_symbol(
                "count".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 5),
            )
            .unwrap();

        analyzer.mark_mutable(symbol_id);

        let symbol = analyzer.symbol_table().get(symbol_id).unwrap();
        assert!(symbol.is_mutable);
    }

    #[test]
    fn test_mark_captured() {
        let mut analyzer = SemanticAnalyzer::new();

        let symbol_id = analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        analyzer.mark_captured(symbol_id);

        let symbol = analyzer.symbol_table().get(symbol_id).unwrap();
        assert!(symbol.is_captured);
    }

    #[test]
    fn test_function_scope() {
        let mut analyzer = SemanticAnalyzer::new();

        let func_id = analyzer
            .define_symbol(
                "greet".to_string(),
                SymbolKind::Function {
                    params: vec![],
                    return_type: None,
                },
                Span::new(0, 5),
            )
            .unwrap();

        analyzer.enter_function(func_id);
        assert_eq!(analyzer.current_function(), Some(func_id));

        analyzer.exit_function();
        assert_eq!(analyzer.current_function(), None);
    }

    #[test]
    fn test_finalize() {
        let mut analyzer = SemanticAnalyzer::new();

        analyzer
            .define_symbol(
                "x".to_string(),
                SymbolKind::Variable { type_hint: None },
                Span::new(0, 1),
            )
            .unwrap();

        let result = analyzer.finalize();
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.symbol_table.len(), 1);
    }
}
