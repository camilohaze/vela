use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::symbol::{Symbol, SymbolId, SymbolKind, Span};
use crate::scope::ScopeId;

/// Symbol table for managing symbols
pub struct SymbolTable {
    /// Map from SymbolId to Symbol
    symbols: HashMap<SymbolId, Symbol>,
    /// Map from (ScopeId, name) to SymbolId for lookups
    names: HashMap<(ScopeId, String), SymbolId>,
    /// Counter for generating unique SymbolIds
    next_id: AtomicUsize,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            names: HashMap::new(),
            next_id: AtomicUsize::new(0),
        }
    }

    /// Define a new symbol in the table
    pub fn define(
        &mut self,
        name: String,
        kind: SymbolKind,
        span: Span,
        scope_id: ScopeId,
    ) -> SymbolId {
        let id = SymbolId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let symbol = Symbol::new(id, name.clone(), kind, span, scope_id);
        
        self.symbols.insert(id, symbol);
        self.names.insert((scope_id, name), id);
        
        id
    }

    /// Lookup a symbol by name in a specific scope
    pub fn lookup_in_scope(&self, name: &str, scope_id: ScopeId) -> Option<&Symbol> {
        let symbol_id = self.names.get(&(scope_id, name.to_string()))?;
        self.symbols.get(symbol_id)
    }

    /// Get a symbol by its ID
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    /// Get a mutable reference to a symbol
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(&id)
    }

    /// Get the total number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Get all symbols in a specific scope
    pub fn symbols_in_scope(&self, scope_id: ScopeId) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.scope_id == scope_id)
            .collect()
    }

    /// Check if a name exists in a scope
    pub fn contains(&self, name: &str, scope_id: ScopeId) -> bool {
        self.names.contains_key(&(scope_id, name.to_string()))
    }

    /// Mark a symbol as captured by a closure
    pub fn mark_captured(&mut self, id: SymbolId) {
        if let Some(symbol) = self.symbols.get_mut(&id) {
            symbol.is_captured = true;
        }
    }

    /// Mark a symbol as mutable
    pub fn mark_mutable(&mut self, id: SymbolId) {
        if let Some(symbol) = self.symbols.get_mut(&id) {
            symbol.is_mutable = true;
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_new() {
        let table = SymbolTable::new();
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_define_symbol() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        let id = table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
        
        let symbol = table.get(id).unwrap();
        assert_eq!(symbol.name, "x");
        assert!(symbol.is_variable());
    }

    #[test]
    fn test_lookup_in_scope() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        let symbol = table.lookup_in_scope("x", scope);
        assert!(symbol.is_some());
        assert_eq!(symbol.unwrap().name, "x");
        
        let not_found = table.lookup_in_scope("y", scope);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_multiple_scopes() {
        let mut table = SymbolTable::new();
        let scope1 = ScopeId(0);
        let scope2 = ScopeId(1);
        
        table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope1,
        );
        
        table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(10, 11),
            scope2,
        );
        
        assert_eq!(table.len(), 2);
        
        let sym1 = table.lookup_in_scope("x", scope1).unwrap();
        let sym2 = table.lookup_in_scope("x", scope2).unwrap();
        
        assert_eq!(sym1.span.start, 0);
        assert_eq!(sym2.span.start, 10);
    }

    #[test]
    fn test_symbols_in_scope() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        table.define(
            "y".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(2, 3),
            scope,
        );
        
        let symbols = table.symbols_in_scope(scope);
        assert_eq!(symbols.len(), 2);
    }

    #[test]
    fn test_contains() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        assert!(table.contains("x", scope));
        assert!(!table.contains("y", scope));
    }

    #[test]
    fn test_mark_captured() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        let id = table.define(
            "x".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        assert!(!table.get(id).unwrap().is_captured);
        
        table.mark_captured(id);
        
        assert!(table.get(id).unwrap().is_captured);
    }

    #[test]
    fn test_mark_mutable() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        let id = table.define(
            "count".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 5),
            scope,
        );
        
        assert!(!table.get(id).unwrap().is_mutable);
        
        table.mark_mutable(id);
        
        assert!(table.get(id).unwrap().is_mutable);
    }

    #[test]
    fn test_unique_symbol_ids() {
        let mut table = SymbolTable::new();
        let scope = ScopeId(0);
        
        let id1 = table.define(
            "a".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(0, 1),
            scope,
        );
        
        let id2 = table.define(
            "b".to_string(),
            SymbolKind::Variable { type_hint: None },
            Span::new(2, 3),
            scope,
        );
        
        assert_ne!(id1, id2);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
    }
}
