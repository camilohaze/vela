use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::symbol::SymbolId;

/// Scope identifier - unique ID for each scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

/// Kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global scope (top-level)
    Global,
    /// Module scope
    Module,
    /// Function scope
    Function,
    /// Block scope (if, while, etc.)
    Block,
    /// Class scope
    Class,
    /// Loop scope (for, while)
    Loop,
}

/// A lexical scope
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub symbols: HashSet<SymbolId>,
}

impl Scope {
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Self {
            id,
            kind,
            parent,
            children: Vec::new(),
            symbols: HashSet::new(),
        }
    }

    /// Add a child scope
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }

    /// Add a symbol to this scope
    pub fn add_symbol(&mut self, symbol_id: SymbolId) {
        self.symbols.insert(symbol_id);
    }

    /// Check if this scope contains a symbol
    pub fn contains_symbol(&self, symbol_id: SymbolId) -> bool {
        self.symbols.contains(&symbol_id)
    }

    /// Get the number of symbols in this scope
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

/// Manages all scopes in the program
pub struct ScopeManager {
    scopes: HashMap<ScopeId, Scope>,
    current: ScopeId,
    next_id: AtomicUsize,
}

impl ScopeManager {
    /// Create a new scope manager with a global scope
    pub fn new() -> Self {
        let global_id = ScopeId(0);
        let mut scopes = HashMap::new();
        scopes.insert(global_id, Scope::new(global_id, ScopeKind::Global, None));

        Self {
            scopes,
            current: global_id,
            next_id: AtomicUsize::new(1), // 0 is used for global
        }
    }

    /// Get the global scope ID
    pub fn global_scope(&self) -> ScopeId {
        ScopeId(0)
    }

    /// Get the current scope ID
    pub fn current_scope(&self) -> ScopeId {
        self.current
    }

    /// Create a new scope
    pub fn create_scope(&mut self, kind: ScopeKind, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let parent_id = parent.unwrap_or(self.current);
        
        let scope = Scope::new(id, kind, Some(parent_id));
        self.scopes.insert(id, scope);
        
        // Add as child to parent
        if let Some(parent_scope) = self.scopes.get_mut(&parent_id) {
            parent_scope.add_child(id);
        }
        
        id
    }

    /// Enter a scope (make it current)
    pub fn enter_scope(&mut self, scope_id: ScopeId) {
        self.current = scope_id;
    }

    /// Exit current scope (go to parent)
    pub fn exit_scope(&mut self) {
        if let Some(scope) = self.scopes.get(&self.current) {
            if let Some(parent) = scope.parent {
                self.current = parent;
            }
        }
    }

    /// Get a scope by ID
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    /// Get a mutable reference to a scope
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&id)
    }

    /// Add a symbol to a scope
    pub fn add_symbol_to_scope(&mut self, scope_id: ScopeId, symbol_id: SymbolId) {
        if let Some(scope) = self.scopes.get_mut(&scope_id) {
            scope.add_symbol(symbol_id);
        }
    }

    /// Get the parent of a scope
    pub fn parent(&self, scope_id: ScopeId) -> Option<ScopeId> {
        self.scopes.get(&scope_id).and_then(|s| s.parent)
    }

    /// Get all ancestors of a scope (bottom-up)
    pub fn ancestors(&self, scope_id: ScopeId) -> Vec<ScopeId> {
        let mut ancestors = Vec::new();
        let mut current = scope_id;
        
        while let Some(parent) = self.parent(current) {
            ancestors.push(parent);
            current = parent;
        }
        
        ancestors
    }

    /// Get the number of scopes
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    /// Check if the manager is empty (should never happen - always has global)
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_creation() {
        let scope = Scope::new(ScopeId(0), ScopeKind::Global, None);
        assert_eq!(scope.id, ScopeId(0));
        assert_eq!(scope.kind, ScopeKind::Global);
        assert!(scope.parent.is_none());
        assert_eq!(scope.children.len(), 0);
        assert_eq!(scope.symbol_count(), 0);
    }

    #[test]
    fn test_scope_add_child() {
        let mut scope = Scope::new(ScopeId(0), ScopeKind::Global, None);
        scope.add_child(ScopeId(1));
        
        assert_eq!(scope.children.len(), 1);
        assert_eq!(scope.children[0], ScopeId(1));
    }

    #[test]
    fn test_scope_add_symbol() {
        let mut scope = Scope::new(ScopeId(0), ScopeKind::Global, None);
        scope.add_symbol(SymbolId(1));
        scope.add_symbol(SymbolId(2));
        
        assert_eq!(scope.symbol_count(), 2);
        assert!(scope.contains_symbol(SymbolId(1)));
        assert!(scope.contains_symbol(SymbolId(2)));
        assert!(!scope.contains_symbol(SymbolId(3)));
    }

    #[test]
    fn test_scope_manager_new() {
        let manager = ScopeManager::new();
        assert_eq!(manager.len(), 1); // Global scope
        assert_eq!(manager.current_scope(), ScopeId(0));
        assert_eq!(manager.global_scope(), ScopeId(0));
    }

    #[test]
    fn test_create_scope() {
        let mut manager = ScopeManager::new();
        let func_scope = manager.create_scope(ScopeKind::Function, None);
        
        assert_eq!(manager.len(), 2);
        assert_eq!(func_scope, ScopeId(1));
        
        let scope = manager.get(func_scope).unwrap();
        assert_eq!(scope.kind, ScopeKind::Function);
        assert_eq!(scope.parent, Some(ScopeId(0)));
    }

    #[test]
    fn test_enter_exit_scope() {
        let mut manager = ScopeManager::new();
        let func_scope = manager.create_scope(ScopeKind::Function, None);
        
        assert_eq!(manager.current_scope(), ScopeId(0));
        
        manager.enter_scope(func_scope);
        assert_eq!(manager.current_scope(), func_scope);
        
        manager.exit_scope();
        assert_eq!(manager.current_scope(), ScopeId(0));
    }

    #[test]
    fn test_nested_scopes() {
        let mut manager = ScopeManager::new();
        
        let func_scope = manager.create_scope(ScopeKind::Function, None);
        manager.enter_scope(func_scope);
        
        let block_scope = manager.create_scope(ScopeKind::Block, None);
        
        assert_eq!(manager.len(), 3);
        
        let block = manager.get(block_scope).unwrap();
        assert_eq!(block.parent, Some(func_scope));
        
        let func = manager.get(func_scope).unwrap();
        assert_eq!(func.children.len(), 1);
        assert_eq!(func.children[0], block_scope);
    }

    #[test]
    fn test_ancestors() {
        let mut manager = ScopeManager::new();
        
        let func_scope = manager.create_scope(ScopeKind::Function, None);
        manager.enter_scope(func_scope);
        
        let block_scope = manager.create_scope(ScopeKind::Block, None);
        
        let ancestors = manager.ancestors(block_scope);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0], func_scope);
        assert_eq!(ancestors[1], ScopeId(0)); // Global
    }

    #[test]
    fn test_add_symbol_to_scope() {
        let mut manager = ScopeManager::new();
        let global = manager.global_scope();
        
        manager.add_symbol_to_scope(global, SymbolId(1));
        manager.add_symbol_to_scope(global, SymbolId(2));
        
        let scope = manager.get(global).unwrap();
        assert_eq!(scope.symbol_count(), 2);
        assert!(scope.contains_symbol(SymbolId(1)));
    }

    #[test]
    fn test_scope_kinds() {
        let mut manager = ScopeManager::new();
        
        let module = manager.create_scope(ScopeKind::Module, None);
        let class = manager.create_scope(ScopeKind::Class, None);
        let loop_scope = manager.create_scope(ScopeKind::Loop, None);
        
        assert_eq!(manager.get(module).unwrap().kind, ScopeKind::Module);
        assert_eq!(manager.get(class).unwrap().kind, ScopeKind::Class);
        assert_eq!(manager.get(loop_scope).unwrap().kind, ScopeKind::Loop);
    }
}
