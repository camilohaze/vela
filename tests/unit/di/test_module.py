"""
Tests unitarios para @module decorator

Jira: TASK-035D
Historia: VELA-575
"""

import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '../../..')))

import pytest
from src.runtime.di.module import (
    module,
    ModuleMetadata,
    is_module,
    get_module_metadata,
    get_module_declarations,
    get_module_controllers,
    get_module_providers,
    get_module_imports,
    get_module_exports,
    register_module,
    get_module,
    get_all_modules,
    clear_module_registry,
    find_module_by_provider,
    find_module_by_controller,
)


# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture(autouse=True)
def clear_registry():
    """Limpiar registry antes y después de cada test."""
    clear_module_registry()
    yield
    clear_module_registry()


# Mock classes para tests
class MockService:
    """Mock service para tests."""
    pass


class MockController:
    """Mock controller para tests."""
    pass


class MockWidget:
    """Mock widget para tests."""
    pass


class MockRepository:
    """Mock repository para tests."""
    pass


# ============================================================================
# TEST: ModuleMetadata
# ============================================================================

class TestModuleMetadata:
    """Tests para ModuleMetadata dataclass."""
    
    def test_create_empty_metadata(self):
        """Test crear metadata vacía."""
        metadata = ModuleMetadata()
        
        assert metadata.declarations == []
        assert metadata.controllers == []
        assert metadata.providers == []
        assert metadata.imports == []
        assert metadata.exports == []
    
    def test_create_metadata_with_data(self):
        """Test crear metadata con datos."""
        metadata = ModuleMetadata(
            declarations=[MockWidget],
            controllers=[MockController],
            providers=[MockService],
            imports=[],
            exports=[MockService]
        )
        
        assert MockWidget in metadata.declarations
        assert MockController in metadata.controllers
        assert MockService in metadata.providers
        assert MockService in metadata.exports
    
    def test_validation_exports_subset_of_declarations_and_providers(self):
        """Test validación: exports ⊆ (declarations ∪ providers)."""
        # OK: export está en providers
        metadata = ModuleMetadata(
            providers=[MockService],
            exports=[MockService]
        )
        assert MockService in metadata.exports
        
        # OK: export está en declarations
        metadata = ModuleMetadata(
            declarations=[MockWidget],
            exports=[MockWidget]
        )
        assert MockWidget in metadata.exports
    
    def test_validation_invalid_exports(self):
        """Test validación falla cuando exports no está en declarations ni providers."""
        with pytest.raises(ValueError) as exc_info:
            ModuleMetadata(
                declarations=[MockWidget],
                exports=[MockService]  # MockService NO está en declarations
            )
        
        assert "Invalid exports" in str(exc_info.value)
        assert "MockService" in str(exc_info.value)
    
    def test_get_all_providers(self):
        """Test obtener todos los providers (declarations + providers)."""
        metadata = ModuleMetadata(
            declarations=[MockWidget],
            providers=[MockService],
        )
        
        all_providers = metadata.get_all_providers()
        
        assert MockWidget in all_providers
        assert MockService in all_providers
        assert len(all_providers) == 2
    
    def test_get_exported_providers(self):
        """Test obtener solo providers exportados."""
        metadata = ModuleMetadata(
            declarations=[MockWidget],
            providers=[MockService, MockRepository],
            exports=[MockService]  # Solo MockService exportado
        )
        
        exported = metadata.get_exported_providers()
        
        assert MockService in exported
        assert MockRepository not in exported
        assert len(exported) == 1
    
    def test_has_controller(self):
        """Test verificar si tiene controller."""
        metadata = ModuleMetadata(
            controllers=[MockController]
        )
        
        assert metadata.has_controller(MockController) == True
        assert metadata.has_controller(MockService) == False
    
    def test_has_provider(self):
        """Test verificar si tiene provider."""
        metadata = ModuleMetadata(
            declarations=[MockWidget],
            providers=[MockService]
        )
        
        assert metadata.has_provider(MockWidget) == True
        assert metadata.has_provider(MockService) == True
        assert metadata.has_provider(MockController) == False


# ============================================================================
# TEST: @module decorator
# ============================================================================

class TestModuleDecorator:
    """Tests para decorador @module."""
    
    def test_basic_module(self):
        """Test crear módulo básico."""
        @module(
            declarations=[MockWidget],
            providers=[MockService]
        )
        class TestModule:
            pass
        
        assert is_module(TestModule) == True
        metadata = get_module_metadata(TestModule)
        assert metadata is not None
        assert MockWidget in metadata.declarations
        assert MockService in metadata.providers
    
    def test_module_with_controllers(self):
        """Test módulo con controllers."""
        @module(
            controllers=[MockController],
            providers=[MockService]
        )
        class BackendModule:
            pass
        
        metadata = get_module_metadata(BackendModule)
        assert MockController in metadata.controllers
        assert MockService in metadata.providers
    
    def test_module_with_exports(self):
        """Test módulo con exports."""
        @module(
            providers=[MockService, MockRepository],
            exports=[MockService]  # Solo exportar MockService
        )
        class ServiceModule:
            pass
        
        metadata = get_module_metadata(ServiceModule)
        assert MockService in metadata.exports
        assert MockRepository not in metadata.exports
    
    def test_module_with_imports(self):
        """Test módulo con imports."""
        @module(providers=[MockService])
        class CommonModule:
            pass
        
        @module(
            imports=[CommonModule],
            providers=[MockRepository]
        )
        class AppModule:
            pass
        
        metadata = get_module_metadata(AppModule)
        assert CommonModule in metadata.imports
    
    def test_module_validation_fails(self):
        """Test decorador falla con exports inválidos."""
        with pytest.raises(ValueError) as exc_info:
            @module(
                declarations=[MockWidget],
                exports=[MockService]  # MockService NO está en declarations
            )
            class InvalidModule:
                pass
        
        assert "Invalid exports" in str(exc_info.value)
    
    def test_module_auto_registers(self):
        """Test módulo se auto-registra en registry."""
        @module(providers=[MockService])
        class AutoRegisterModule:
            pass
        
        # Verificar que está en registry
        registered = get_module(AutoRegisterModule)
        assert registered is not None
        assert MockService in registered.providers


# ============================================================================
# TEST: Helper Functions
# ============================================================================

class TestModuleHelpers:
    """Tests para helper functions."""
    
    def test_is_module_true(self):
        """Test is_module retorna True para módulos."""
        @module(providers=[MockService])
        class TestModule:
            pass
        
        assert is_module(TestModule) == True
    
    def test_is_module_false(self):
        """Test is_module retorna False para no-módulos."""
        class NotAModule:
            pass
        
        assert is_module(NotAModule) == False
    
    def test_get_module_metadata_returns_metadata(self):
        """Test get_module_metadata retorna metadata."""
        @module(providers=[MockService])
        class TestModule:
            pass
        
        metadata = get_module_metadata(TestModule)
        assert metadata is not None
        assert isinstance(metadata, ModuleMetadata)
        assert MockService in metadata.providers
    
    def test_get_module_metadata_returns_none(self):
        """Test get_module_metadata retorna None para no-módulos."""
        class NotAModule:
            pass
        
        assert get_module_metadata(NotAModule) is None
    
    def test_get_module_declarations(self):
        """Test get_module_declarations."""
        @module(declarations=[MockWidget, MockService])
        class TestModule:
            pass
        
        declarations = get_module_declarations(TestModule)
        assert MockWidget in declarations
        assert MockService in declarations
        assert len(declarations) == 2
    
    def test_get_module_controllers(self):
        """Test get_module_controllers."""
        @module(controllers=[MockController])
        class TestModule:
            pass
        
        controllers = get_module_controllers(TestModule)
        assert MockController in controllers
    
    def test_get_module_providers(self):
        """Test get_module_providers."""
        @module(providers=[MockService, MockRepository])
        class TestModule:
            pass
        
        providers = get_module_providers(TestModule)
        assert MockService in providers
        assert MockRepository in providers
    
    def test_get_module_imports(self):
        """Test get_module_imports."""
        @module(providers=[MockService])
        class CommonModule:
            pass
        
        @module(imports=[CommonModule])
        class TestModule:
            pass
        
        imports = get_module_imports(TestModule)
        assert CommonModule in imports
    
    def test_get_module_exports(self):
        """Test get_module_exports."""
        @module(
            providers=[MockService, MockRepository],
            exports=[MockService]
        )
        class TestModule:
            pass
        
        exports = get_module_exports(TestModule)
        assert MockService in exports
        assert MockRepository not in exports


# ============================================================================
# TEST: Module Registry
# ============================================================================

class TestModuleRegistry:
    """Tests para module registry."""
    
    def test_register_module(self):
        """Test registrar módulo manualmente."""
        class TestModule:
            pass
        
        metadata = ModuleMetadata(providers=[MockService])
        register_module(TestModule, metadata)
        
        registered = get_module(TestModule)
        assert registered is not None
        assert registered == metadata
    
    def test_get_module_returns_metadata(self):
        """Test get_module retorna metadata."""
        @module(providers=[MockService])
        class TestModule:
            pass
        
        registered = get_module(TestModule)
        assert registered is not None
        assert MockService in registered.providers
    
    def test_get_module_returns_none_for_unregistered(self):
        """Test get_module retorna None para módulos no registrados."""
        class NotRegistered:
            pass
        
        assert get_module(NotRegistered) is None
    
    def test_get_all_modules(self):
        """Test get_all_modules retorna todos los módulos."""
        @module(providers=[MockService])
        class Module1:
            pass
        
        @module(providers=[MockRepository])
        class Module2:
            pass
        
        all_modules = get_all_modules()
        
        assert Module1 in all_modules
        assert Module2 in all_modules
        assert len(all_modules) == 2
    
    def test_clear_module_registry(self):
        """Test clear_module_registry limpia registry."""
        @module(providers=[MockService])
        class TestModule:
            pass
        
        assert len(get_all_modules()) == 1
        
        clear_module_registry()
        
        assert len(get_all_modules()) == 0
    
    def test_find_module_by_provider(self):
        """Test find_module_by_provider encuentra módulo."""
        @module(providers=[MockService])
        class TestModule:
            pass
        
        found = find_module_by_provider(MockService)
        
        assert found == TestModule
    
    def test_find_module_by_provider_returns_none(self):
        """Test find_module_by_provider retorna None si no encuentra."""
        found = find_module_by_provider(MockService)
        
        assert found is None
    
    def test_find_module_by_controller(self):
        """Test find_module_by_controller encuentra módulo."""
        @module(controllers=[MockController])
        class TestModule:
            pass
        
        found = find_module_by_controller(MockController)
        
        assert found == TestModule
    
    def test_find_module_by_controller_returns_none(self):
        """Test find_module_by_controller retorna None si no encuentra."""
        found = find_module_by_controller(MockController)
        
        assert found is None


# ============================================================================
# TEST: Edge Cases
# ============================================================================

class TestModuleEdgeCases:
    """Tests para casos edge."""
    
    def test_empty_module(self):
        """Test módulo completamente vacío."""
        @module()
        class EmptyModule:
            pass
        
        metadata = get_module_metadata(EmptyModule)
        assert metadata.declarations == []
        assert metadata.controllers == []
        assert metadata.providers == []
        assert metadata.imports == []
        assert metadata.exports == []
    
    def test_module_with_duplicates_in_declarations_and_providers(self):
        """Test módulo con duplicados en declarations y providers."""
        @module(
            declarations=[MockService],
            providers=[MockService],
            exports=[MockService]
        )
        class DuplicateModule:
            pass
        
        metadata = get_module_metadata(DuplicateModule)
        all_providers = metadata.get_all_providers()
        
        # get_all_providers debe eliminar duplicados (usa set)
        assert all_providers.count(MockService) == 1
    
    def test_find_module_by_provider_in_declarations(self):
        """Test find_module_by_provider encuentra en declarations."""
        @module(declarations=[MockWidget])
        class TestModule:
            pass
        
        found = find_module_by_provider(MockWidget)
        
        assert found == TestModule
    
    def test_multiple_modules_with_same_provider(self):
        """Test múltiples módulos con mismo provider."""
        @module(providers=[MockService])
        class Module1:
            pass
        
        @module(providers=[MockService])
        class Module2:
            pass
        
        # find_module_by_provider retorna el primero que encuentra
        found = find_module_by_provider(MockService)
        
        assert found in [Module1, Module2]


# ============================================================================
# TEST: Integration
# ============================================================================

class TestModuleIntegration:
    """Tests de integración de @module."""
    
    def test_complete_module_scenario(self):
        """Test escenario completo con módulo multiplataforma."""
        # Módulo con declarations (frontend) y controllers (backend)
        @module(
            declarations=[MockWidget, MockService],
            controllers=[MockController],
            providers=[MockService, MockRepository],
            exports=[MockService]
        )
        class AppModule:
            pass
        
        # Verificar metadata
        metadata = get_module_metadata(AppModule)
        assert MockWidget in metadata.declarations
        assert MockController in metadata.controllers
        assert MockService in metadata.providers
        assert MockService in metadata.exports
        
        # Verificar registry
        assert get_module(AppModule) == metadata
        
        # Verificar helpers
        assert is_module(AppModule) == True
        assert len(get_module_declarations(AppModule)) == 2
        assert len(get_module_controllers(AppModule)) == 1
        assert len(get_module_providers(AppModule)) == 2
        assert len(get_module_exports(AppModule)) == 1
        
        # Verificar búsqueda
        assert find_module_by_provider(MockService) == AppModule
        assert find_module_by_controller(MockController) == AppModule
    
    def test_module_import_chain(self):
        """Test cadena de imports entre módulos."""
        @module(providers=[MockRepository])
        class DataModule:
            pass
        
        @module(
            imports=[DataModule],
            providers=[MockService]
        )
        class ServiceModule:
            pass
        
        @module(
            imports=[ServiceModule],
            controllers=[MockController]
        )
        class AppModule:
            pass
        
        # Verificar cadena de imports
        app_metadata = get_module_metadata(AppModule)
        assert ServiceModule in app_metadata.imports
        
        service_metadata = get_module_metadata(ServiceModule)
        assert DataModule in service_metadata.imports


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
