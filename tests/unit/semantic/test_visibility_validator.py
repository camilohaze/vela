"""
Tests unitarios para Visibility Validator

Jira: VELA-572
Sprint: 10
TASK-023: Validar visibilidad (public/private)
"""

import pytest
import sys
from pathlib import Path

# Agregar src al path para imports
src_path = Path(__file__).parent.parent.parent / "src"
sys.path.insert(0, str(src_path))

from semantic.visibility_validator import (
    VisibilityValidator,
    AccessLevel,
    ModuleType,
    ModuleContext,
    AccessViolation,
    VisibilityError
)
from semantic.symbol_table import Symbol, SymbolKind


class TestModuleContext:
    """Tests para ModuleContext."""
    
    def test_module_context_creation(self):
        """Test de creación de ModuleContext."""
        module = ModuleContext("my_module", ModuleType.USER_MODULE, {"func1", "func2"})
        assert module.name == "my_module"
        assert module.type == ModuleType.USER_MODULE
        assert "func1" in module.exports
        assert "func2" in module.exports
    
    def test_is_stdlib(self):
        """Test de verificación de módulo stdlib."""
        stdlib = ModuleContext("system:core", ModuleType.SYSTEM, set())
        user = ModuleContext("my_module", ModuleType.USER_MODULE, set())
        
        assert stdlib.is_stdlib() == True
        assert user.is_stdlib() == False
    
    def test_is_external(self):
        """Test de verificación de módulo externo."""
        package = ModuleContext("lodash", ModuleType.PACKAGE, set())
        extension = ModuleContext("charts", ModuleType.EXTENSION, set())
        user = ModuleContext("my_module", ModuleType.USER_MODULE, set())
        
        assert package.is_external() == True
        assert extension.is_external() == True
        assert user.is_external() == False


class TestAccessLevel:
    """Tests para AccessLevel."""
    
    def test_access_level_values(self):
        """Test de valores de AccessLevel."""
        assert AccessLevel.PUBLIC.value == "public"
        assert AccessLevel.PRIVATE.value == "private"
        assert AccessLevel.PROTECTED.value == "protected"


class TestVisibilityValidator:
    """Tests para VisibilityValidator."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        self.validator = VisibilityValidator()
    
    # ========================================
    # Tests de Registro de Módulos
    # ========================================
    
    def test_register_module(self):
        """Test de registro de módulo."""
        module = self.validator.register_module("my_module", ModuleType.USER_MODULE)
        
        assert module.name == "my_module"
        assert module.type == ModuleType.USER_MODULE
        assert "my_module" in self.validator.modules
    
    def test_register_module_with_exports(self):
        """Test de registro con exports explícitos."""
        exports = {"func1", "func2"}
        module = self.validator.register_module(
            "my_module",
            ModuleType.USER_MODULE,
            exports
        )
        
        assert module.exports == exports
    
    def test_set_current_module(self):
        """Test de establecer módulo actual."""
        self.validator.register_module("my_module")
        self.validator.set_current_module("my_module")
        
        assert self.validator.current_module.name == "my_module"
    
    def test_set_current_module_not_registered(self):
        """Test de error al establecer módulo no registrado."""
        with pytest.raises(ValueError, match="not registered"):
            self.validator.set_current_module("unknown_module")
    
    # ========================================
    # Tests de Nivel de Acceso
    # ========================================
    
    def test_get_access_level_public(self):
        """Test de obtener nivel de acceso público."""
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=True)
        level = self.validator.get_access_level(symbol)
        
        assert level == AccessLevel.PUBLIC
    
    def test_get_access_level_private(self):
        """Test de obtener nivel de acceso privado."""
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=False)
        level = self.validator.get_access_level(symbol)
        
        assert level == AccessLevel.PRIVATE
    
    def test_get_access_level_protected(self):
        """Test de obtener nivel de acceso protected."""
        symbol = Symbol(
            "func",
            SymbolKind.FUNCTION,
            0,
            is_public=False,
            metadata={"access_level": "protected"}
        )
        level = self.validator.get_access_level(symbol)
        
        assert level == AccessLevel.PROTECTED
    
    # ========================================
    # Tests de Validación de Acceso
    # ========================================
    
    def test_validate_access_public_symbol_same_module(self):
        """Test de acceso a símbolo público desde mismo módulo."""
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=True)
        
        # No debe lanzar excepción
        result = self.validator.validate_access(symbol, "module_a", 10, 5)
        assert result == True
    
    def test_validate_access_public_symbol_cross_module(self):
        """Test de acceso a símbolo público desde otro módulo."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=True)
        
        # No debe lanzar excepción
        result = self.validator.validate_access(symbol, "module_a", 10, 5)
        assert result == True
    
    def test_validate_access_private_symbol_same_module(self):
        """Test de acceso a símbolo privado desde mismo módulo."""
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        
        # No debe lanzar excepción
        result = self.validator.validate_access(symbol, "module_a", 10, 5)
        assert result == True
    
    def test_validate_access_private_symbol_cross_module_fails(self):
        """Test de error al acceder a símbolo privado desde otro módulo."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        
        # Debe lanzar VisibilityError
        with pytest.raises(VisibilityError) as exc_info:
            self.validator.validate_access(symbol, "module_a", 10, 5)
        
        violation = exc_info.value.violation
        assert violation.symbol.name == "helper"
        assert violation.symbol_module == "module_a"
        assert violation.access_module == "module_b"
        assert "Cannot access private symbol" in violation.message
    
    def test_validate_access_stdlib_always_public(self):
        """Test de acceso a stdlib (siempre público)."""
        self.validator.register_module("system:core", ModuleType.SYSTEM)
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        # Símbolo privado en stdlib
        symbol = Symbol("print", SymbolKind.FUNCTION, 0, is_public=False)
        
        # No debe lanzar excepción (stdlib es siempre público)
        result = self.validator.validate_access(symbol, "system:core", 10, 5)
        assert result == True
    
    def test_validate_access_with_exports(self):
        """Test de acceso a símbolo en exports explícitos."""
        self.validator.register_module(
            "module_a",
            ModuleType.USER_MODULE,
            exports={"helper"}
        )
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        # Símbolo privado pero en exports
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        
        # No debe lanzar excepción (está en exports)
        result = self.validator.validate_access(symbol, "module_a", 10, 5)
        assert result == True
    
    def test_validate_access_no_current_module(self):
        """Test de error si no hay módulo actual."""
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=True)
        
        with pytest.raises(ValueError, match="No current module"):
            self.validator.validate_access(symbol, "module_a", 10, 5)
    
    # ========================================
    # Tests de Validación de Miembros de Clase
    # ========================================
    
    def test_validate_member_access_public_member(self):
        """Test de acceso a miembro público de clase."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        class_symbol = Symbol(
            "User",
            SymbolKind.CLASS,
            0,
            is_public=True,
            metadata={"module": "module_a"}
        )
        
        member = Symbol("name", SymbolKind.VARIABLE, 1, is_public=True)
        
        # No debe lanzar excepción
        result = self.validator.validate_member_access(class_symbol, member, 10, 5)
        assert result == True
    
    def test_validate_member_access_private_member_same_class(self):
        """Test de acceso a miembro privado desde misma clase."""
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        class_symbol = Symbol(
            "User",
            SymbolKind.CLASS,
            0,
            is_public=True,
            metadata={"module": "module_a"}
        )
        
        member = Symbol("password", SymbolKind.VARIABLE, 1, is_public=False)
        
        # No debe lanzar excepción (mismo módulo)
        result = self.validator.validate_member_access(class_symbol, member, 10, 5)
        assert result == True
    
    def test_validate_member_access_private_member_cross_class_fails(self):
        """Test de error al acceder a miembro privado desde otra clase."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        class_symbol = Symbol(
            "User",
            SymbolKind.CLASS,
            0,
            is_public=True,
            metadata={"module": "module_a"}
        )
        
        member = Symbol("password", SymbolKind.VARIABLE, 1, is_public=False)
        
        # Debe lanzar VisibilityError
        with pytest.raises(VisibilityError) as exc_info:
            self.validator.validate_member_access(class_symbol, member, 10, 5)
        
        violation = exc_info.value.violation
        assert violation.symbol.name == "password"
        assert "Cannot access private member" in violation.message
    
    # ========================================
    # Tests de Violaciones
    # ========================================
    
    def test_violations_tracking(self):
        """Test de tracking de violaciones."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        symbol1 = Symbol("helper1", SymbolKind.FUNCTION, 0, is_public=False)
        symbol2 = Symbol("helper2", SymbolKind.FUNCTION, 0, is_public=False)
        
        # Primera violación
        try:
            self.validator.validate_access(symbol1, "module_a", 10, 5)
        except VisibilityError:
            pass
        
        # Segunda violación
        try:
            self.validator.validate_access(symbol2, "module_a", 20, 10)
        except VisibilityError:
            pass
        
        violations = self.validator.get_violations()
        assert len(violations) == 2
        assert violations[0].symbol.name == "helper1"
        assert violations[1].symbol.name == "helper2"
    
    def test_clear_violations(self):
        """Test de limpiar violaciones."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.set_current_module("module_b")
        
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        
        try:
            self.validator.validate_access(symbol, "module_a", 10, 5)
        except VisibilityError:
            pass
        
        assert len(self.validator.get_violations()) == 1
        
        self.validator.clear_violations()
        assert len(self.validator.get_violations()) == 0
    
    # ========================================
    # Tests de Utilidades
    # ========================================
    
    def test_reset(self):
        """Test de resetear validador."""
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        try:
            self.validator.validate_access(symbol, "module_b", 10, 5)
        except:
            pass
        
        self.validator.reset()
        
        assert len(self.validator.modules) == 0
        assert self.validator.current_module is None
        assert len(self.validator.violations) == 0
    
    def test_get_module_info(self):
        """Test de obtener información de módulo."""
        self.validator.register_module("my_module", ModuleType.USER_MODULE)
        
        info = self.validator.get_module_info("my_module")
        assert info is not None
        assert info.name == "my_module"
        
        info = self.validator.get_module_info("unknown")
        assert info is None
    
    def test_list_public_symbols_with_exports(self):
        """Test de listar símbolos públicos con exports."""
        exports = {"func1", "func2"}
        self.validator.register_module(
            "my_module",
            ModuleType.USER_MODULE,
            exports
        )
        
        public_symbols = self.validator.list_public_symbols("my_module")
        assert set(public_symbols) == exports
    
    def test_list_public_symbols_stdlib(self):
        """Test de listar símbolos públicos de stdlib."""
        self.validator.register_module("system:core", ModuleType.SYSTEM)
        
        # Stdlib retorna lista vacía (todo es público)
        public_symbols = self.validator.list_public_symbols("system:core")
        assert public_symbols == []
    
    def test_list_public_symbols_unknown_module(self):
        """Test de listar símbolos de módulo desconocido."""
        public_symbols = self.validator.list_public_symbols("unknown")
        assert public_symbols == []
    
    # ========================================
    # Tests de AccessViolation
    # ========================================
    
    def test_access_violation_str(self):
        """Test de representación string de AccessViolation."""
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        violation = AccessViolation(
            symbol=symbol,
            symbol_module="module_a",
            access_module="module_b",
            line=10,
            column=5,
            message="Cannot access private symbol"
        )
        
        str_repr = str(violation)
        assert "line 10" in str_repr
        assert "column 5" in str_repr
        assert "Cannot access private symbol" in str_repr
    
    # ========================================
    # Tests de Edge Cases
    # ========================================
    
    def test_multiple_modules_registration(self):
        """Test de registro de múltiples módulos."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        self.validator.register_module("module_c")
        
        assert len(self.validator.modules) == 3
        assert "module_a" in self.validator.modules
        assert "module_b" in self.validator.modules
        assert "module_c" in self.validator.modules
    
    def test_switch_current_module(self):
        """Test de cambiar módulo actual múltiples veces."""
        self.validator.register_module("module_a")
        self.validator.register_module("module_b")
        
        self.validator.set_current_module("module_a")
        assert self.validator.current_module.name == "module_a"
        
        self.validator.set_current_module("module_b")
        assert self.validator.current_module.name == "module_b"
    
    def test_symbol_without_metadata(self):
        """Test de validación con símbolo sin metadata."""
        self.validator.register_module("module_a")
        self.validator.set_current_module("module_a")
        
        symbol = Symbol("func", SymbolKind.FUNCTION, 0, is_public=True)
        
        # No debe fallar con símbolo sin metadata
        result = self.validator.validate_access(symbol, "module_a", 10, 5)
        assert result == True


class TestVisibilityError:
    """Tests para VisibilityError."""
    
    def test_visibility_error_creation(self):
        """Test de creación de VisibilityError."""
        symbol = Symbol("helper", SymbolKind.FUNCTION, 0, is_public=False)
        violation = AccessViolation(
            symbol=symbol,
            symbol_module="module_a",
            access_module="module_b",
            line=10,
            column=5,
            message="Cannot access private symbol"
        )
        
        error = VisibilityError(violation)
        assert error.violation == violation
        assert "Cannot access private symbol" in str(error)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
