"""
Tests de Symbol Table

Tests unitarios para src/semantic/symbol_table.py

Jira: TASK-016K
Historia: VELA-572
"""

import pytest
from src.semantic.symbol_table import (
    Symbol, SymbolTable, SymbolKind, Scope, ScopeType
)


class TestSymbol:
    """Tests básicos de la clase Symbol."""
    
    def test_symbol_creation(self):
        """Test de creación de símbolo."""
        sym = Symbol(
            name="myVar",
            kind=SymbolKind.VARIABLE,
            scope_level=0,
            type_annotation="Number"
        )
        assert sym.name == "myVar"
        assert sym.kind == SymbolKind.VARIABLE
        assert sym.scope_level == 0
        assert sym.type_annotation == "Number"
        assert not sym.is_public
        assert not sym.is_mutable
    
    def test_symbol_with_flags(self):
        """Test de símbolo con flags public y mutable."""
        sym = Symbol(
            name="count",
            kind=SymbolKind.STATE,
            scope_level=0,
            type_annotation="Number",
            is_public=True,
            is_mutable=True
        )
        assert sym.is_public
        assert sym.is_mutable
    
    def test_symbol_with_metadata(self):
        """Test de símbolo con metadata."""
        sym = Symbol(
            name="myFunc",
            kind=SymbolKind.FUNCTION,
            scope_level=0,
            metadata={"returns": "Number", "params": ["x", "y"]}
        )
        assert sym.metadata["returns"] == "Number"
        assert sym.metadata["params"] == ["x", "y"]


class TestScope:
    """Tests de la clase Scope."""
    
    def test_scope_creation(self):
        """Test de creación de scope."""
        scope = Scope(scope_type=ScopeType.GLOBAL, level=0)
        assert scope.scope_type == ScopeType.GLOBAL
        assert scope.level == 0
        assert len(scope.symbols) == 0
        assert scope.parent is None
    
    def test_scope_define_symbol(self):
        """Test de definir símbolo en scope."""
        scope = Scope(scope_type=ScopeType.GLOBAL, level=0)
        sym = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        
        result = scope.define(sym)
        assert result is True  # Definición exitosa
        assert "x" in scope.symbols
    
    def test_scope_define_duplicate(self):
        """Test de definir símbolo duplicado (debe fallar)."""
        scope = Scope(scope_type=ScopeType.GLOBAL, level=0)
        sym1 = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        sym2 = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        
        result1 = scope.define(sym1)
        result2 = scope.define(sym2)
        
        assert result1 is True
        assert result2 is False  # Duplicado
    
    def test_scope_lookup_local(self):
        """Test de lookup local (solo en este scope)."""
        scope = Scope(scope_type=ScopeType.GLOBAL, level=0)
        sym = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        scope.define(sym)
        
        found = scope.lookup_local("x")
        not_found = scope.lookup_local("y")
        
        assert found is not None
        assert found.name == "x"
        assert not_found is None
    
    def test_scope_lookup_with_parent(self):
        """Test de lookup con scope padre."""
        parent = Scope(scope_type=ScopeType.GLOBAL, level=0)
        child = Scope(scope_type=ScopeType.FUNCTION, level=1, parent=parent)
        
        # Definir en padre
        sym_parent = Symbol(name="global", kind=SymbolKind.VARIABLE, scope_level=0)
        parent.define(sym_parent)
        
        # Definir en hijo
        sym_child = Symbol(name="local", kind=SymbolKind.VARIABLE, scope_level=1)
        child.define(sym_child)
        
        # Lookup desde hijo
        found_local = child.lookup("local")
        found_global = child.lookup("global")
        not_found = child.lookup("nonexistent")
        
        assert found_local is not None
        assert found_local.name == "local"
        assert found_global is not None
        assert found_global.name == "global"
        assert not_found is None


class TestSymbolTable:
    """Tests de la Symbol Table completa."""
    
    def test_symbol_table_initialization(self):
        """Test de inicialización de symbol table."""
        table = SymbolTable()
        assert table.get_scope_level() == 0
        assert table.get_scope_type() == ScopeType.GLOBAL
        assert len(table.scope_stack) == 1
    
    def test_define_in_global_scope(self):
        """Test de definir símbolo en scope global."""
        table = SymbolTable()
        sym = Symbol(name="PI", kind=SymbolKind.VARIABLE, scope_level=0, type_annotation="Float")
        
        result = table.define(sym)
        assert result is True
        assert sym.scope_level == 0
    
    def test_define_duplicate_in_same_scope(self):
        """Test de definir símbolo duplicado en mismo scope (debe fallar)."""
        table = SymbolTable()
        sym1 = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        sym2 = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        
        result1 = table.define(sym1)
        result2 = table.define(sym2)
        
        assert result1 is True
        assert result2 is False
    
    def test_lookup_in_global_scope(self):
        """Test de lookup en scope global."""
        table = SymbolTable()
        sym = Symbol(name="PI", kind=SymbolKind.VARIABLE, scope_level=0)
        table.define(sym)
        
        found = table.lookup("PI")
        not_found = table.lookup("DOES_NOT_EXIST")
        
        assert found is not None
        assert found.name == "PI"
        assert not_found is None
    
    def test_push_pop_scope(self):
        """Test de push y pop de scopes."""
        table = SymbolTable()
        assert table.get_scope_level() == 0
        
        # Push function scope
        table.push_scope(ScopeType.FUNCTION)
        assert table.get_scope_level() == 1
        assert table.get_scope_type() == ScopeType.FUNCTION
        
        # Push block scope
        table.push_scope(ScopeType.BLOCK)
        assert table.get_scope_level() == 2
        assert table.get_scope_type() == ScopeType.BLOCK
        
        # Pop block scope
        popped = table.pop_scope()
        assert popped is not None
        assert table.get_scope_level() == 1
        
        # Pop function scope
        table.pop_scope()
        assert table.get_scope_level() == 0
    
    def test_cannot_pop_global_scope(self):
        """Test que no se puede eliminar el scope global."""
        table = SymbolTable()
        result = table.pop_scope()
        assert result is None
        assert table.get_scope_level() == 0
    
    def test_lookup_in_nested_scopes(self):
        """Test de lookup en scopes anidados."""
        table = SymbolTable()
        
        # Definir en global
        global_sym = Symbol(name="global", kind=SymbolKind.VARIABLE, scope_level=0)
        table.define(global_sym)
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        local_sym = Symbol(name="local", kind=SymbolKind.VARIABLE, scope_level=1)
        table.define(local_sym)
        
        # Lookup desde función
        found_global = table.lookup("global")
        found_local = table.lookup("local")
        
        assert found_global is not None
        assert found_global.name == "global"
        assert found_local is not None
        assert found_local.name == "local"
        
        # Salir de función
        table.pop_scope()
        
        # Lookup desde global
        found_global_again = table.lookup("global")
        not_found_local = table.lookup("local")
        
        assert found_global_again is not None
        assert not_found_local is None
    
    def test_shadowing_detection(self):
        """Test de detección de shadowing."""
        table = SymbolTable()
        
        # Definir en global
        global_x = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0)
        table.define(global_x)
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        
        # x NO existe en scope actual, pero SÍ en padre → shadowing
        assert table.is_shadowing("x") is True
        
        # y NO existe en ningún lado → no shadowing
        assert table.is_shadowing("y") is False
        
        # Definir x en función (shadowing)
        local_x = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=1)
        table.define(local_x)
        
        # Ahora x existe en scope actual → no es shadowing (ya fue definido)
        assert table.is_shadowing("x") is False
    
    def test_lookup_local_vs_lookup(self):
        """Test diferencia entre lookup_local y lookup."""
        table = SymbolTable()
        
        # Definir en global
        global_sym = Symbol(name="global", kind=SymbolKind.VARIABLE, scope_level=0)
        table.define(global_sym)
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        local_sym = Symbol(name="local", kind=SymbolKind.VARIABLE, scope_level=1)
        table.define(local_sym)
        
        # lookup_local: solo busca en scope actual
        found_local = table.lookup_local("local")
        not_found_global = table.lookup_local("global")
        
        assert found_local is not None
        assert not_found_global is None
        
        # lookup: busca en scope actual y padres
        found_local_full = table.lookup("local")
        found_global_full = table.lookup("global")
        
        assert found_local_full is not None
        assert found_global_full is not None
    
    def test_shadowing_with_lookup(self):
        """Test de shadowing: lookup debe retornar el símbolo más cercano."""
        table = SymbolTable()
        
        # Definir x en global
        global_x = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0, type_annotation="String")
        table.define(global_x)
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        
        # Definir x en función (shadowing)
        local_x = Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=1, type_annotation="Number")
        table.define(local_x)
        
        # Lookup debe retornar el x de la función (más cercano)
        found = table.lookup("x")
        assert found is not None
        assert found.type_annotation == "Number"
        assert found.scope_level == 1
    
    def test_multiple_nested_scopes(self):
        """Test con múltiples niveles de scopes anidados."""
        table = SymbolTable()
        
        # Global
        table.define(Symbol(name="a", kind=SymbolKind.VARIABLE, scope_level=0))
        
        # Función nivel 1
        table.push_scope(ScopeType.FUNCTION)
        table.define(Symbol(name="b", kind=SymbolKind.VARIABLE, scope_level=1))
        
        # Bloque nivel 2
        table.push_scope(ScopeType.BLOCK)
        table.define(Symbol(name="c", kind=SymbolKind.VARIABLE, scope_level=2))
        
        # Bloque nivel 3
        table.push_scope(ScopeType.BLOCK)
        table.define(Symbol(name="d", kind=SymbolKind.VARIABLE, scope_level=3))
        
        # Lookup desde nivel 3 (debe encontrar todos)
        assert table.lookup("a") is not None
        assert table.lookup("b") is not None
        assert table.lookup("c") is not None
        assert table.lookup("d") is not None
        
        # Pop a nivel 2
        table.pop_scope()
        assert table.lookup("d") is None  # d ya no es visible
        assert table.lookup("c") is not None
        
        # Pop a nivel 1
        table.pop_scope()
        assert table.lookup("c") is None
        assert table.lookup("b") is not None
        
        # Pop a nivel 0
        table.pop_scope()
        assert table.lookup("b") is None
        assert table.lookup("a") is not None
    
    def test_get_all_symbols(self):
        """Test de obtener todos los símbolos visibles."""
        table = SymbolTable()
        
        # Definir en global
        table.define(Symbol(name="a", kind=SymbolKind.VARIABLE, scope_level=0))
        table.define(Symbol(name="b", kind=SymbolKind.VARIABLE, scope_level=0))
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        table.define(Symbol(name="c", kind=SymbolKind.VARIABLE, scope_level=1))
        
        # Obtener todos los símbolos visibles
        all_symbols = table.get_all_symbols()
        
        assert len(all_symbols) == 3
        assert "a" in all_symbols
        assert "b" in all_symbols
        assert "c" in all_symbols
    
    def test_get_all_symbols_with_shadowing(self):
        """Test de get_all_symbols con shadowing."""
        table = SymbolTable()
        
        # Definir x en global
        table.define(Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0, type_annotation="String"))
        
        # Entrar a función
        table.push_scope(ScopeType.FUNCTION)
        
        # Definir x en función (shadowing)
        table.define(Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=1, type_annotation="Number"))
        
        # get_all_symbols debe retornar el x de la función (más reciente)
        all_symbols = table.get_all_symbols()
        assert len(all_symbols) == 1
        assert "x" in all_symbols
        assert all_symbols["x"].type_annotation == "Number"
    
    def test_different_symbol_kinds(self):
        """Test de diferentes tipos de símbolos."""
        table = SymbolTable()
        
        # Variable
        table.define(Symbol(name="x", kind=SymbolKind.VARIABLE, scope_level=0))
        
        # State (mutable)
        table.define(Symbol(name="count", kind=SymbolKind.STATE, scope_level=0, is_mutable=True))
        
        # Función
        table.define(Symbol(name="add", kind=SymbolKind.FUNCTION, scope_level=0))
        
        # Clase
        table.define(Symbol(name="User", kind=SymbolKind.CLASS, scope_level=0))
        
        # Module
        table.define(Symbol(name="AuthModule", kind=SymbolKind.MODULE, scope_level=0))
        
        # Lookup
        assert table.lookup("x").kind == SymbolKind.VARIABLE
        assert table.lookup("count").kind == SymbolKind.STATE
        assert table.lookup("add").kind == SymbolKind.FUNCTION
        assert table.lookup("User").kind == SymbolKind.CLASS
        assert table.lookup("AuthModule").kind == SymbolKind.MODULE


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
