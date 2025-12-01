"""
Tests de integración de Semantic Analysis

Jira: VELA-572
Sprint: 10
TASK-024: Tests de semantic analysis

Tests end-to-end que integran todos los componentes:
- Symbol Table (TASK-021)
- Import Resolver (TASK-021A)
- Import Validator (TASK-021B)
- Name Resolver (TASK-022)
- Visibility Validator (TASK-023)
"""

import pytest
import sys
from pathlib import Path

# Agregar src al path
src_path = Path(__file__).parent.parent.parent / "src"
sys.path.insert(0, str(src_path))

from semantic.symbol_table import Symbol, SymbolTable, SymbolKind, ScopeType
from semantic.import_resolver import ImportResolver, ImportPrefix
from semantic.import_validator import ImportValidator, VelaKeyword
from semantic.name_resolver import NameResolver, ReferenceKind
from semantic.visibility_validator import VisibilityValidator, ModuleType, VisibilityError


class TestSemanticIntegration:
    """
    Tests de integración completa de semantic analysis.
    
    Estos tests validan que todos los componentes trabajan
    juntos correctamente para analizar código Vela.
    """
    
    def setup_method(self):
        """Setup antes de cada test."""
        self.symbol_table = SymbolTable()
        self.import_resolver = ImportResolver()
        self.import_validator = ImportValidator()
        self.name_resolver = NameResolver(self.symbol_table)
        self.visibility_validator = VisibilityValidator()
        
        # Registrar módulos base
        self.visibility_validator.register_module("main", ModuleType.USER_MODULE)
        self.visibility_validator.register_module("system:core", ModuleType.SYSTEM)
    
    # =================================================================
    # Tests de Integración: Symbol Table + Name Resolver
    # =================================================================
    
    def test_define_and_resolve_in_global_scope(self):
        """Test de definir y resolver en scope global."""
        # Definir variable
        symbol = self.name_resolver.define(
            "count",
            SymbolKind.STATE,
            is_mutable=True,
            is_public=False
        )
        
        assert symbol.name == "count"
        assert symbol.kind == SymbolKind.STATE
        assert symbol.is_mutable == True
        
        # Resolver variable
        resolved = self.name_resolver.resolve("count", ReferenceKind.READ, 10, 5)
        
        assert resolved is not None
        assert resolved.name == "count"
        assert resolved.kind == SymbolKind.STATE
    
    def test_scoped_resolution_with_shadowing(self):
        """Test de resolución con shadowing en scopes anidados."""
        # Definir en global scope
        global_x = self.name_resolver.define("x", SymbolKind.VARIABLE)
        
        # Entrar a función
        self.name_resolver.enter_scope(ScopeType.FUNCTION)
        
        # Definir variable local con mismo nombre
        local_x = self.name_resolver.define("x", SymbolKind.VARIABLE)
        
        # Resolver debe retornar local x (shadowing)
        resolved = self.name_resolver.resolve("x", ReferenceKind.READ, 20, 5)
        assert resolved == local_x
        assert resolved.scope_level == 1
        
        # Salir de scope
        self.name_resolver.exit_scope()
        
        # Ahora resolver debe retornar global x
        resolved = self.name_resolver.resolve("x", ReferenceKind.READ, 30, 5)
        assert resolved == global_x
        assert resolved.scope_level == 0
    
    def test_mutability_validation_with_name_resolver(self):
        """Test de validación de mutabilidad durante resolución."""
        # Definir variable inmutable
        self.name_resolver.define("PI", SymbolKind.VARIABLE, is_mutable=False)
        
        # Intentar escribir debe fallar
        with pytest.raises(Exception, match="Cannot assign to immutable"):
            self.name_resolver.resolve("PI", ReferenceKind.WRITE, 10, 5)
        
        # Definir variable mutable
        self.name_resolver.define("counter", SymbolKind.STATE, is_mutable=True)
        
        # Escribir debe funcionar
        resolved = self.name_resolver.resolve("counter", ReferenceKind.WRITE, 15, 5)
        assert resolved is not None
    
    def test_dead_code_detection(self):
        """Test de detección de código muerto."""
        # Definir varias variables
        self.name_resolver.define("used_var", SymbolKind.VARIABLE)
        self.name_resolver.define("unused_var", SymbolKind.VARIABLE)
        
        # Usar solo una
        self.name_resolver.resolve("used_var", ReferenceKind.READ, 10, 5)
        
        # Obtener símbolos no usados
        unused = self.name_resolver.get_unused_symbols()
        
        assert len(unused) == 1
        assert unused[0].name == "unused_var"
    
    # =================================================================
    # Tests de Integración: Import Resolver + Import Validator
    # =================================================================
    
    def test_resolve_and_validate_system_import(self):
        """Test de resolver y validar import de system."""
        # Resolver import de system
        import_path = "system:ui"
        resolved = self.import_resolver.resolve(import_path)
        
        # Validar que widget puede importar system
        violations = self.import_validator.validate_import(
            VelaKeyword.WIDGET,
            ImportPrefix.SYSTEM,
            import_path
        )
        
        assert len(violations) == 0  # Sin violaciones
    
    def test_validate_forbidden_import(self):
        """Test de validación de import prohibido."""
        # Service NO puede importar system (UI)
        violations = self.import_validator.validate_import(
            VelaKeyword.SERVICE,
            ImportPrefix.SYSTEM,
            "system:ui"
        )
        
        assert len(violations) == 1
        assert "service" in violations[0].message.lower()
        assert "system:" in violations[0].message
    
    def test_entity_can_only_import_pure_modules(self):
        """Test de que entity solo puede importar módulos puros."""
        # Entity PUEDE importar module
        violations = self.import_validator.validate_import(
            VelaKeyword.ENTITY,
            ImportPrefix.MODULE,
            "module:user"
        )
        assert len(violations) == 0
        
        # Entity NO PUEDE importar package (dependencia externa)
        violations = self.import_validator.validate_import(
            VelaKeyword.ENTITY,
            ImportPrefix.PACKAGE,
            "package:axios"
        )
        assert len(violations) == 1
    
    # =================================================================
    # Tests de Integración: Name Resolver + Visibility Validator
    # =================================================================
    
    def test_resolve_with_visibility_check_public_symbol(self):
        """Test de resolución con validación de visibilidad (público)."""
        # Establecer módulo actual
        self.visibility_validator.set_current_module("main")
        
        # Definir símbolo público
        symbol = Symbol(
            "publicFunc",
            SymbolKind.FUNCTION,
            0,
            is_public=True
        )
        self.symbol_table.define(symbol)
        
        # Resolver símbolo
        resolved = self.name_resolver.resolve("publicFunc", ReferenceKind.CALL, 10, 5)
        assert resolved is not None
        
        # Validar visibilidad (debe pasar porque es público)
        is_valid = self.visibility_validator.validate_access(
            resolved,
            "main",
            10,
            5
        )
        assert is_valid == True
    
    def test_resolve_with_visibility_check_private_cross_module(self):
        """Test de error al acceder símbolo privado cross-module."""
        # Registrar módulo externo
        self.visibility_validator.register_module("external", ModuleType.USER_MODULE)
        
        # Definir símbolo privado en módulo externo
        symbol = Symbol(
            "privateFunc",
            SymbolKind.FUNCTION,
            0,
            is_public=False
        )
        
        # Intentar acceder desde módulo main
        self.visibility_validator.set_current_module("main")
        
        with pytest.raises(VisibilityError):
            self.visibility_validator.validate_access(
                symbol,
                "external",  # definido en external
                10,
                5
            )
    
    # =================================================================
    # Tests de Integración: Workflow Completo
    # =================================================================
    
    def test_complete_semantic_analysis_workflow(self):
        """
        Test de workflow completo de análisis semántico.
        
        Simula el análisis completo de un módulo Vela:
        1. Resolver imports
        2. Validar imports por keyword
        3. Definir símbolos en symbol table
        4. Resolver nombres
        5. Validar visibilidad
        6. Detectar código muerto
        """
        # PASO 1: Resolver imports
        system_import = self.import_resolver.resolve("system:core")
        assert system_import.exists == True
        
        # PASO 2: Validar imports (widget puede importar system)
        violations = self.import_validator.validate_import(
            VelaKeyword.WIDGET,
            ImportPrefix.SYSTEM,
            "system:core"
        )
        assert len(violations) == 0
        
        # PASO 3: Definir símbolos
        # Definir función pública
        public_func = self.name_resolver.define(
            "initialize",
            SymbolKind.FUNCTION,
            is_mutable=False,
            is_public=True
        )
        
        # Definir variable privada
        private_var = self.name_resolver.define(
            "_internalState",
            SymbolKind.VARIABLE,
            is_mutable=False,
            is_public=False
        )
        
        # Definir variable mutable
        counter = self.name_resolver.define(
            "counter",
            SymbolKind.STATE,
            is_mutable=True,
            is_public=False
        )
        
        # PASO 4: Resolver nombres
        # Resolver función (debe funcionar)
        resolved_func = self.name_resolver.resolve(
            "initialize",
            ReferenceKind.CALL,
            10,
            5
        )
        assert resolved_func is not None
        
        # Resolver variable privada (lectura)
        resolved_var = self.name_resolver.resolve(
            "_internalState",
            ReferenceKind.READ,
            15,
            10
        )
        assert resolved_var is not None
        
        # Resolver variable mutable (escritura)
        resolved_counter = self.name_resolver.resolve(
            "counter",
            ReferenceKind.WRITE,
            20,
            5
        )
        assert resolved_counter is not None
        
        # PASO 5: Validar visibilidad
        self.visibility_validator.set_current_module("main")
        
        # Función pública debe ser accesible
        is_valid = self.visibility_validator.validate_access(
            public_func,
            "main",
            10,
            5
        )
        assert is_valid == True
        
        # PASO 6: Detectar código muerto
        # Definir variable no usada
        self.name_resolver.define("unusedVar", SymbolKind.VARIABLE)
        
        unused = self.name_resolver.get_unused_symbols()
        assert len(unused) == 1
        assert unused[0].name == "unusedVar"
    
    def test_class_with_members_complete_analysis(self):
        """Test de análisis completo de clase con miembros."""
        # Definir clase
        class_symbol = self.name_resolver.define(
            "User",
            SymbolKind.CLASS,
            is_public=True
        )
        
        # Entrar a scope de clase
        self.name_resolver.enter_scope(ScopeType.CLASS)
        
        # Definir miembro público
        public_field = self.name_resolver.define(
            "name",
            SymbolKind.VARIABLE,
            is_public=True
        )
        
        # Definir miembro privado
        private_field = self.name_resolver.define(
            "password",
            SymbolKind.VARIABLE,
            is_public=False
        )
        
        # Definir método público
        public_method = self.name_resolver.define(
            "getName",
            SymbolKind.FUNCTION,
            is_public=True
        )
        
        # Resolver miembros dentro de la clase
        resolved_name = self.name_resolver.resolve("name", ReferenceKind.READ, 30, 10)
        resolved_password = self.name_resolver.resolve("password", ReferenceKind.READ, 35, 10)
        
        assert resolved_name is not None
        assert resolved_password is not None
        
        # Salir de scope de clase
        self.name_resolver.exit_scope()
        
        # Validar visibilidad de miembros
        self.visibility_validator.set_current_module("main")
        
        # Miembro público accesible
        is_valid = self.visibility_validator.validate_member_access(
            class_symbol,
            public_field,
            40,
            10
        )
        assert is_valid == True
        
        # Miembro privado NO accesible desde otro módulo
        self.visibility_validator.register_module("other", ModuleType.USER_MODULE)
        self.visibility_validator.set_current_module("other")
        
        # Agregar metadata de módulo a class_symbol
        class_symbol.metadata = {"module": "main"}
        
        with pytest.raises(VisibilityError):
            self.visibility_validator.validate_member_access(
                class_symbol,
                private_field,
                45,
                10
            )
    
    def test_service_layer_imports_validation(self):
        """Test de validación de imports de capa de servicio."""
        # Service puede importar package (dependencias externas)
        violations = self.import_validator.validate_import(
            VelaKeyword.SERVICE,
            ImportPrefix.PACKAGE,
            "package:axios"
        )
        assert len(violations) == 0
        
        # Service NO puede importar system (UI)
        violations = self.import_validator.validate_import(
            VelaKeyword.SERVICE,
            ImportPrefix.SYSTEM,
            "system:ui"
        )
        assert len(violations) == 1
        
        # Repository puede importar package
        violations = self.import_validator.validate_import(
            VelaKeyword.REPOSITORY,
            ImportPrefix.PACKAGE,
            "package:typeorm"
        )
        assert len(violations) == 0
    
    def test_widget_layer_imports_validation(self):
        """Test de validación de imports de capa de UI."""
        # Widget puede importar system (UI)
        violations = self.import_validator.validate_import(
            VelaKeyword.WIDGET,
            ImportPrefix.SYSTEM,
            "system:ui"
        )
        assert len(violations) == 0
        
        # Widget NO puede importar package (HTTP directo)
        violations = self.import_validator.validate_import(
            VelaKeyword.WIDGET,
            ImportPrefix.PACKAGE,
            "package:axios"
        )
        assert len(violations) == 1
        
        # Widget puede importar module
        violations = self.import_validator.validate_import(
            VelaKeyword.WIDGET,
            ImportPrefix.MODULE,
            "module:components"
        )
        assert len(violations) == 0
    
    def test_multiple_scopes_with_resolution(self):
        """Test de múltiples scopes anidados con resolución."""
        # Global scope
        global_var = self.name_resolver.define("globalVar", SymbolKind.VARIABLE)
        
        # Function scope 1
        self.name_resolver.enter_scope(ScopeType.FUNCTION)
        func_var = self.name_resolver.define("funcVar", SymbolKind.VARIABLE)
        
        # Block scope
        self.name_resolver.enter_scope(ScopeType.BLOCK)
        block_var = self.name_resolver.define("blockVar", SymbolKind.VARIABLE)
        
        # Resolver desde block scope
        assert self.name_resolver.resolve("blockVar", ReferenceKind.READ, 50, 5) is not None
        assert self.name_resolver.resolve("funcVar", ReferenceKind.READ, 51, 5) is not None
        assert self.name_resolver.resolve("globalVar", ReferenceKind.READ, 52, 5) is not None
        
        # Salir de block scope
        self.name_resolver.exit_scope()
        
        # blockVar ya no accesible
        with pytest.raises(Exception):
            self.name_resolver.resolve("blockVar", ReferenceKind.READ, 55, 5)
        
        # funcVar y globalVar sí accesibles
        assert self.name_resolver.resolve("funcVar", ReferenceKind.READ, 56, 5) is not None
        assert self.name_resolver.resolve("globalVar", ReferenceKind.READ, 57, 5) is not None
        
        # Salir de function scope
        self.name_resolver.exit_scope()
        
        # Solo globalVar accesible
        assert self.name_resolver.resolve("globalVar", ReferenceKind.READ, 60, 5) is not None
        
        with pytest.raises(Exception):
            self.name_resolver.resolve("funcVar", ReferenceKind.READ, 61, 5)


class TestSemanticAnalysisMetrics:
    """Tests de métricas y estadísticas de semantic analysis."""
    
    def test_symbol_table_statistics(self):
        """Test de estadísticas de symbol table."""
        table = SymbolTable()
        
        # Definir símbolos
        table.define(Symbol("var1", SymbolKind.VARIABLE, 0))
        table.define(Symbol("func1", SymbolKind.FUNCTION, 0))
        table.define(Symbol("class1", SymbolKind.CLASS, 0))
        
        # Entrar a scope y definir más
        table.push_scope(ScopeType.FUNCTION)
        table.define(Symbol("param1", SymbolKind.VARIABLE, 1))
        table.define(Symbol("localVar", SymbolKind.VARIABLE, 1))
        
        # Verificar conteo
        assert len(table.current_scope.symbols) == 2  # param1, localVar
        
        table.pop_scope()
        assert len(table.current_scope.symbols) == 3  # var1, func1, class1
    
    def test_reference_tracking(self):
        """Test de tracking de referencias."""
        table = SymbolTable()
        resolver = NameResolver(table)
        
        # Definir símbolo
        resolver.define("myVar", SymbolKind.VARIABLE)
        
        # Resolver múltiples veces
        resolver.resolve("myVar", ReferenceKind.READ, 10, 5)
        resolver.resolve("myVar", ReferenceKind.READ, 15, 10)
        resolver.resolve("myVar", ReferenceKind.READ, 20, 5)
        
        # Obtener referencias
        refs = resolver.get_references("myVar")
        
        assert len(refs) == 3
        assert all(ref.name == "myVar" for ref in refs)
        assert all(ref.kind == ReferenceKind.READ for ref in refs)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
