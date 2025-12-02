"""
Tests unitarios para Scope enum

Implementación de: TASK-035B
Historia: VELA-575
Sprint: 13
"""

import pytest
import sys
from pathlib import Path

# Agregar src/ al path para importar módulos
src_path = Path(__file__).resolve().parent.parent.parent.parent / "src"
sys.path.insert(0, str(src_path))

from runtime.di.scopes import Scope, DEFAULT_SCOPE


class TestScope:
    """Suite de tests para Scope enum."""
    
    def test_scope_values_exist(self):
        """Test que verifica que existen todos los scopes."""
        assert Scope.SINGLETON is not None
        assert Scope.TRANSIENT is not None
        assert Scope.SCOPED is not None
    
    def test_from_string_singleton(self):
        """Test from_string con 'singleton'."""
        scope = Scope.from_string("singleton")
        assert scope == Scope.SINGLETON
    
    def test_from_string_transient(self):
        """Test from_string con 'transient'."""
        scope = Scope.from_string("transient")
        assert scope == Scope.TRANSIENT
    
    def test_from_string_scoped(self):
        """Test from_string con 'scoped'."""
        scope = Scope.from_string("scoped")
        assert scope == Scope.SCOPED
    
    def test_from_string_case_insensitive(self):
        """Test from_string es case-insensitive."""
        assert Scope.from_string("SINGLETON") == Scope.SINGLETON
        assert Scope.from_string("Transient") == Scope.TRANSIENT
        assert Scope.from_string("ScOpEd") == Scope.SCOPED
    
    def test_from_string_invalid(self):
        """Test from_string con valor inválido lanza ValueError."""
        with pytest.raises(ValueError, match="Invalid scope"):
            Scope.from_string("invalid")
    
    def test_from_string_empty(self):
        """Test from_string con string vacío lanza ValueError."""
        with pytest.raises(ValueError, match="Invalid scope"):
            Scope.from_string("")
    
    def test_is_cacheable_singleton(self):
        """Test is_cacheable para SINGLETON."""
        assert Scope.SINGLETON.is_cacheable() == True
    
    def test_is_cacheable_transient(self):
        """Test is_cacheable para TRANSIENT."""
        assert Scope.TRANSIENT.is_cacheable() == False
    
    def test_is_cacheable_scoped(self):
        """Test is_cacheable para SCOPED."""
        assert Scope.SCOPED.is_cacheable() == True
    
    def test_cache_key_prefix_singleton(self):
        """Test cache_key_prefix para SINGLETON."""
        prefix = Scope.SINGLETON.cache_key_prefix()
        assert prefix == "global"
    
    def test_cache_key_prefix_transient(self):
        """Test cache_key_prefix para TRANSIENT."""
        prefix = Scope.TRANSIENT.cache_key_prefix()
        assert prefix == "transient"
    
    def test_cache_key_prefix_scoped(self):
        """Test cache_key_prefix para SCOPED."""
        prefix = Scope.SCOPED.cache_key_prefix()
        assert prefix == "scoped"
    
    def test_default_scope_is_singleton(self):
        """Test que DEFAULT_SCOPE es SINGLETON."""
        assert DEFAULT_SCOPE == Scope.SINGLETON
    
    def test_scope_string_representation(self):
        """Test representación string de cada scope."""
        assert "SINGLETON" in str(Scope.SINGLETON)
        assert "TRANSIENT" in str(Scope.TRANSIENT)
        assert "SCOPED" in str(Scope.SCOPED)
    
    def test_scope_equality(self):
        """Test comparación de scopes."""
        scope1 = Scope.from_string("singleton")
        scope2 = Scope.SINGLETON
        assert scope1 == scope2
        
        scope3 = Scope.TRANSIENT
        assert scope1 != scope3
    
    def test_scope_in_collection(self):
        """Test que scopes se pueden usar en colecciones."""
        scopes_set = {Scope.SINGLETON, Scope.TRANSIENT, Scope.SCOPED}
        assert len(scopes_set) == 3
        assert Scope.SINGLETON in scopes_set
    
    def test_scope_as_dict_key(self):
        """Test que scopes se pueden usar como keys en dict."""
        scope_map = {
            Scope.SINGLETON: "singleton_value",
            Scope.TRANSIENT: "transient_value",
            Scope.SCOPED: "scoped_value"
        }
        assert scope_map[Scope.SINGLETON] == "singleton_value"
        assert scope_map[Scope.TRANSIENT] == "transient_value"
        assert scope_map[Scope.SCOPED] == "scoped_value"


class TestScopeEdgeCases:
    """Tests de edge cases para Scope."""
    
    def test_from_string_with_whitespace(self):
        """Test from_string con whitespace."""
        assert Scope.from_string("  singleton  ") == Scope.SINGLETON
        assert Scope.from_string("\tscoped\n") == Scope.SCOPED
    
    def test_from_string_numeric_string(self):
        """Test from_string con string numérico lanza ValueError."""
        with pytest.raises(ValueError):
            Scope.from_string("123")
    
    def test_cacheable_consistency(self):
        """Test que cacheability es consistente con prefix."""
        for scope in [Scope.SINGLETON, Scope.TRANSIENT, Scope.SCOPED]:
            is_cacheable = scope.is_cacheable()
            prefix = scope.cache_key_prefix()
            
            if is_cacheable:
                # Scopes cacheables tienen prefix no-transient
                assert prefix in ["global", "scoped"]
            else:
                # Scope no-cacheable tiene prefix transient
                assert prefix == "transient"


# Ejecutar tests si se corre directamente
if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
