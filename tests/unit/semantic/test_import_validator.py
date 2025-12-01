"""
Tests unitarios para Import Validator

Jira: TASK-021B
Historia: VELA-572
"""

import pytest
from src.semantic.import_validator import (
    ImportValidator,
    VelaKeyword,
    ImportPrefix,
    ImportRule,
    ImportViolation
)


class TestImportValidator:
    """Suite de tests para ImportValidator."""
    
    def setup_method(self):
        """Configurar cada test."""
        self.validator = ImportValidator()
    
    # ===== TESTS DE WIDGETS =====
    
    def test_widget_can_import_system_ui(self):
        """Widget PUEDE importar system:ui."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM,
            line=1,
            column=0
        )
        assert violation is None
    
    def test_widget_can_import_module(self):
        """Widget PUEDE importar module:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'module:auth'",
            prefix=ImportPrefix.MODULE,
            line=1,
            column=0
        )
        assert violation is None
    
    def test_widget_can_import_library(self):
        """Widget PUEDE importar library:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'library:utils'",
            prefix=ImportPrefix.LIBRARY,
            line=1,
            column=0
        )
        assert violation is None
    
    def test_widget_can_import_assets(self):
        """Widget PUEDE importar assets:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'assets:images'",
            prefix=ImportPrefix.ASSETS,
            line=1,
            column=0
        )
        assert violation is None
    
    def test_widget_cannot_import_package(self):
        """Widget NO PUEDE importar package: directamente."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'package:http'",
            prefix=ImportPrefix.PACKAGE,
            line=5,
            column=0
        )
        assert violation is not None
        assert violation.keyword == VelaKeyword.WIDGET
        assert violation.prefix_used == ImportPrefix.PACKAGE
        assert "package" in violation.message.lower()
        assert violation.suggestion is not None
    
    def test_stateful_widget_follows_same_rules_as_widget(self):
        """StatefulWidget sigue las mismas reglas que widget."""
        # PUEDE system:ui
        violation = self.validator.validate_import(
            keyword=VelaKeyword.STATEFUL_WIDGET,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is None
        
        # NO PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.STATEFUL_WIDGET,
            import_statement="import 'package:axios'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
    
    def test_component_follows_same_rules_as_widget(self):
        """component sigue las mismas reglas que widget."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.COMPONENT,
            import_statement="import 'package:express'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
    
    # ===== TESTS DE SERVICE =====
    
    def test_service_can_import_package(self):
        """service PUEDE importar package:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SERVICE,
            import_statement="import 'package:http'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    def test_service_can_import_module(self):
        """service PUEDE importar module:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SERVICE,
            import_statement="import 'module:users'",
            prefix=ImportPrefix.MODULE
        )
        assert violation is None
    
    def test_service_can_import_library(self):
        """service PUEDE importar library:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SERVICE,
            import_statement="import 'library:validators'",
            prefix=ImportPrefix.LIBRARY
        )
        assert violation is None
    
    def test_service_cannot_import_system_ui(self):
        """service NO PUEDE importar system:ui (separación de concerns)."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SERVICE,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM,
            line=10,
            column=0
        )
        assert violation is not None
        assert violation.keyword == VelaKeyword.SERVICE
        assert "system" in violation.message.lower()
        assert violation.line == 10
    
    def test_repository_follows_same_rules_as_service(self):
        """repository sigue las mismas reglas que service."""
        # PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.REPOSITORY,
            import_statement="import 'package:sequelize'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
        
        # NO PUEDE system:ui
        violation = self.validator.validate_import(
            keyword=VelaKeyword.REPOSITORY,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_usecase_follows_same_rules_as_service(self):
        """usecase sigue las mismas reglas que service."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.USECASE,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    # ===== TESTS DE CONTROLLER =====
    
    def test_controller_can_import_anything(self):
        """controller PUEDE importar cualquier prefijo (puente entre capas)."""
        prefixes = [
            ImportPrefix.SYSTEM,
            ImportPrefix.PACKAGE,
            ImportPrefix.MODULE,
            ImportPrefix.LIBRARY,
            ImportPrefix.EXTENSION,
            ImportPrefix.ASSETS
        ]
        
        for prefix in prefixes:
            violation = self.validator.validate_import(
                keyword=VelaKeyword.CONTROLLER,
                import_statement=f"import '{prefix.value}:test'",
                prefix=prefix
            )
            assert violation is None, f"controller debería poder importar {prefix.value}"
    
    # ===== TESTS DE ENTITIES / DTOs =====
    
    def test_entity_can_only_import_module_and_library(self):
        """entity SOLO puede importar module: y library: (debe ser puro)."""
        # PUEDE module:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.ENTITY,
            import_statement="import 'module:types'",
            prefix=ImportPrefix.MODULE
        )
        assert violation is None
        
        # PUEDE library:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.ENTITY,
            import_statement="import 'library:domain'",
            prefix=ImportPrefix.LIBRARY
        )
        assert violation is None
        
        # NO PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.ENTITY,
            import_statement="import 'package:lodash'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
        
        # NO PUEDE system:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.ENTITY,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_dto_follows_same_rules_as_entity(self):
        """dto sigue las mismas reglas que entity."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.DTO,
            import_statement="import 'package:axios'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
    
    def test_value_object_follows_same_rules_as_entity(self):
        """valueObject sigue las mismas reglas que entity."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.VALUE_OBJECT,
            import_statement="import 'extension:custom'",
            prefix=ImportPrefix.EXTENSION
        )
        assert violation is not None
    
    # ===== TESTS DE MIDDLEWARE / GUARDS =====
    
    def test_guard_can_import_package(self):
        """guard PUEDE importar package:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.GUARD,
            import_statement="import 'package:jwt'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    def test_guard_cannot_import_system_ui(self):
        """guard NO PUEDE importar system:ui."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.GUARD,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_middleware_follows_same_rules_as_guard(self):
        """middleware sigue las mismas reglas que guard."""
        # PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.MIDDLEWARE,
            import_statement="import 'package:express'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
        
        # NO PUEDE system:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.MIDDLEWARE,
            import_statement="import 'system:reactive'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_interceptor_follows_same_rules_as_guard(self):
        """interceptor sigue las mismas reglas que guard."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.INTERCEPTOR,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    # ===== TESTS DE VALIDATOR =====
    
    def test_validator_can_only_import_pure(self):
        """validator SOLO puede importar module: y library: (validación pura)."""
        # PUEDE module:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.VALIDATOR,
            import_statement="import 'module:validation'",
            prefix=ImportPrefix.MODULE
        )
        assert violation is None
        
        # PUEDE library:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.VALIDATOR,
            import_statement="import 'library:regex'",
            prefix=ImportPrefix.LIBRARY
        )
        assert violation is None
        
        # NO PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.VALIDATOR,
            import_statement="import 'package:validator'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
    
    # ===== TESTS DE HELPERS / MAPPERS =====
    
    def test_helper_can_import_module_library_package(self):
        """helper puede importar module:, library:, package:."""
        allowed = [ImportPrefix.MODULE, ImportPrefix.LIBRARY, ImportPrefix.PACKAGE]
        for prefix in allowed:
            violation = self.validator.validate_import(
                keyword=VelaKeyword.HELPER,
                import_statement=f"import '{prefix.value}:test'",
                prefix=prefix
            )
            assert violation is None
    
    def test_helper_cannot_import_system(self):
        """helper NO puede importar system:."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.HELPER,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_mapper_follows_same_rules_as_helper(self):
        """mapper sigue las mismas reglas que helper."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.MAPPER,
            import_statement="import 'system:reactive'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation is not None
    
    def test_serializer_follows_same_rules_as_helper(self):
        """serializer sigue las mismas reglas que helper."""
        # PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SERIALIZER,
            import_statement="import 'package:json'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    # ===== TESTS DE DESIGN PATTERNS =====
    
    def test_factory_can_import_anything(self):
        """factory puede importar cualquier prefijo (patrón genérico)."""
        for prefix in ImportPrefix:
            violation = self.validator.validate_import(
                keyword=VelaKeyword.FACTORY,
                import_statement=f"import '{prefix.value}:test'",
                prefix=prefix
            )
            assert violation is None
    
    def test_singleton_can_import_anything(self):
        """singleton puede importar cualquier prefijo."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.SINGLETON,
            import_statement="import 'package:db'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    # ===== TESTS DE MODULE =====
    
    def test_module_can_import_module_and_package(self):
        """module puede importar otros modules y packages."""
        # PUEDE module:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.MODULE,
            import_statement="import 'module:users'",
            prefix=ImportPrefix.MODULE
        )
        assert violation is None
        
        # PUEDE package:
        violation = self.validator.validate_import(
            keyword=VelaKeyword.MODULE,
            import_statement="import 'package:express'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    # ===== TESTS DE STORE =====
    
    def test_store_can_import_system_module_library(self):
        """store puede importar system:reactive, module:, library:."""
        allowed = [ImportPrefix.SYSTEM, ImportPrefix.MODULE, ImportPrefix.LIBRARY]
        for prefix in allowed:
            violation = self.validator.validate_import(
                keyword=VelaKeyword.STORE,
                import_statement=f"import '{prefix.value}:test'",
                prefix=prefix
            )
            assert violation is None
    
    def test_store_cannot_import_package(self):
        """store NO puede importar package: directo."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.STORE,
            import_statement="import 'package:redux'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is not None
    
    # ===== TESTS DE GENERIC TYPES =====
    
    def test_class_can_import_anything(self):
        """class genérica puede importar cualquier prefijo."""
        for prefix in ImportPrefix:
            violation = self.validator.validate_import(
                keyword=VelaKeyword.CLASS,
                import_statement=f"import '{prefix.value}:test'",
                prefix=prefix
            )
            assert violation is None
    
    def test_interface_can_import_anything(self):
        """interface puede importar cualquier prefijo."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.INTERFACE,
            import_statement="import 'package:types'",
            prefix=ImportPrefix.PACKAGE
        )
        assert violation is None
    
    # ===== TESTS DE MÚLTIPLES IMPORTS =====
    
    def test_validate_multiple_imports(self):
        """Validar múltiples imports a la vez."""
        imports = [
            ("import 'system:ui'", ImportPrefix.SYSTEM, 1, 0),
            ("import 'module:auth'", ImportPrefix.MODULE, 2, 0),
            ("import 'package:http'", ImportPrefix.PACKAGE, 3, 0),  # PROHIBIDO para widget
        ]
        
        violations = self.validator.validate_imports(VelaKeyword.WIDGET, imports)
        
        assert len(violations) == 1  # Solo el import de package: es inválido
        assert violations[0].prefix_used == ImportPrefix.PACKAGE
        assert violations[0].line == 3
    
    def test_validate_multiple_imports_all_valid(self):
        """Validar múltiples imports válidos."""
        imports = [
            ("import 'package:http'", ImportPrefix.PACKAGE, 1, 0),
            ("import 'module:users'", ImportPrefix.MODULE, 2, 0),
            ("import 'library:utils'", ImportPrefix.LIBRARY, 3, 0),
        ]
        
        violations = self.validator.validate_imports(VelaKeyword.SERVICE, imports)
        
        assert len(violations) == 0  # Todos válidos para service
    
    # ===== TESTS DE GET RULE =====
    
    def test_get_rule_returns_correct_rule(self):
        """get_rule() retorna la regla correcta."""
        rule = self.validator.get_rule(VelaKeyword.WIDGET)
        assert rule is not None
        assert rule.keyword == VelaKeyword.WIDGET
        assert ImportPrefix.SYSTEM in rule.allowed_prefixes
        assert ImportPrefix.PACKAGE in rule.forbidden_prefixes
    
    def test_get_allowed_prefixes(self):
        """get_allowed_prefixes() retorna prefijos permitidos."""
        allowed = self.validator.get_allowed_prefixes(VelaKeyword.ENTITY)
        assert ImportPrefix.MODULE in allowed
        assert ImportPrefix.LIBRARY in allowed
        assert ImportPrefix.PACKAGE not in allowed
    
    def test_get_forbidden_prefixes(self):
        """get_forbidden_prefixes() retorna prefijos prohibidos."""
        forbidden = self.validator.get_forbidden_prefixes(VelaKeyword.SERVICE)
        assert ImportPrefix.SYSTEM in forbidden
    
    # ===== TESTS DE EDGE CASES =====
    
    def test_violation_contains_all_info(self):
        """ImportViolation contiene toda la información necesaria."""
        violation = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'package:http'",
            prefix=ImportPrefix.PACKAGE,
            line=42,
            column=10
        )
        
        assert violation is not None
        assert violation.keyword == VelaKeyword.WIDGET
        assert violation.import_statement == "import 'package:http'"
        assert violation.prefix_used == ImportPrefix.PACKAGE
        assert violation.line == 42
        assert violation.column == 10
        assert len(violation.message) > 0
        assert violation.suggestion is not None
    
    def test_different_keywords_same_prefix_different_results(self):
        """Mismo prefijo puede ser válido o no según el keyword."""
        # system:ui válido para widget
        violation_widget = self.validator.validate_import(
            keyword=VelaKeyword.WIDGET,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation_widget is None
        
        # system:ui INVÁLIDO para service
        violation_service = self.validator.validate_import(
            keyword=VelaKeyword.SERVICE,
            import_statement="import 'system:ui'",
            prefix=ImportPrefix.SYSTEM
        )
        assert violation_service is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
