"""
Tests unitarios para @injectable decorator

Implementación de: TASK-035B
Historia: VELA-575
Sprint: 13
"""

import pytest
import sys
from pathlib import Path
from typing import Optional

# Agregar src/ al path para importar módulos
src_path = Path(__file__).resolve().parent.parent.parent.parent / "src"
sys.path.insert(0, str(src_path))

from runtime.di.scopes import Scope, DEFAULT_SCOPE
from runtime.di.injectable import (
    injectable,
    InjectableMetadata,
    is_injectable,
    get_injectable_metadata,
    get_scope,
    get_token,
    register_provider,
    get_provider,
    clear_registry
)


class TestInjectableDecorator:
    """Suite de tests para decorador @injectable."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        clear_registry()
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        clear_registry()
    
    def test_injectable_basic(self):
        """Test decorador básico sin argumentos."""
        @injectable()
        class MyService:
            pass
        
        assert is_injectable(MyService) == True
        metadata = get_injectable_metadata(MyService)
        assert metadata is not None
        assert metadata.scope == DEFAULT_SCOPE
    
    def test_injectable_with_singleton_scope(self):
        """Test decorador con scope singleton."""
        @injectable(scope=Scope.SINGLETON)
        class SingletonService:
            pass
        
        assert get_scope(SingletonService) == Scope.SINGLETON
    
    def test_injectable_with_transient_scope(self):
        """Test decorador con scope transient."""
        @injectable(scope=Scope.TRANSIENT)
        class TransientService:
            pass
        
        assert get_scope(TransientService) == Scope.TRANSIENT
    
    def test_injectable_with_scoped_scope(self):
        """Test decorador con scope scoped."""
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            pass
        
        assert get_scope(ScopedService) == Scope.SCOPED
    
    def test_injectable_with_custom_token(self):
        """Test decorador con token custom."""
        @injectable(token="my-custom-service")
        class MyService:
            pass
        
        assert get_token(MyService) == "my-custom-service"
    
    def test_injectable_with_token_auto_registration(self):
        """Test que token custom registra provider automáticamente."""
        @injectable(token="my-service")
        class MyService:
            pass
        
        # Verificar que se registró automáticamente
        assert get_provider("my-service") == MyService
    
    def test_injectable_with_factory(self):
        """Test decorador con factory function."""
        def my_factory():
            return "instance from factory"
        
        @injectable(factory=my_factory)
        class MyService:
            pass
        
        metadata = get_injectable_metadata(MyService)
        assert metadata.factory is not None
        assert metadata.factory() == "instance from factory"
    
    def test_injectable_default_token_is_class_name(self):
        """Test que token por defecto es el nombre de la clase."""
        @injectable()
        class UserService:
            pass
        
        token = get_token(UserService)
        assert token == "UserService"
    
    def test_injectable_metadata_attributes(self):
        """Test que metadata contiene todos los atributos."""
        @injectable(scope=Scope.TRANSIENT, token="test-service")
        class TestService:
            pass
        
        metadata = get_injectable_metadata(TestService)
        assert hasattr(metadata, 'scope')
        assert hasattr(metadata, 'token')
        assert hasattr(metadata, 'factory')
        assert hasattr(metadata, 'dependencies')
        assert isinstance(metadata.dependencies, list)


class TestInjectableHelpers:
    """Tests para funciones helper de injectable."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        clear_registry()
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        clear_registry()
    
    def test_is_injectable_true_for_decorated_class(self):
        """Test is_injectable retorna True para clase decorada."""
        @injectable()
        class MyService:
            pass
        
        assert is_injectable(MyService) == True
    
    def test_is_injectable_false_for_non_decorated_class(self):
        """Test is_injectable retorna False para clase no decorada."""
        class PlainClass:
            pass
        
        assert is_injectable(PlainClass) == False
    
    def test_get_injectable_metadata_returns_none_for_non_decorated(self):
        """Test get_injectable_metadata retorna None para no decorada."""
        class PlainClass:
            pass
        
        assert get_injectable_metadata(PlainClass) is None
    
    def test_get_scope_returns_none_for_non_decorated(self):
        """Test get_scope retorna None para no decorada."""
        class PlainClass:
            pass
        
        assert get_scope(PlainClass) is None
    
    def test_get_token_returns_none_for_non_decorated(self):
        """Test get_token retorna None para no decorada."""
        class PlainClass:
            pass
        
        assert get_token(PlainClass) is None


class TestInjectableRegistry:
    """Tests para el registry de providers."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        clear_registry()
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        clear_registry()
    
    def test_register_provider_manual(self):
        """Test registro manual de provider."""
        class MyService:
            pass
        
        register_provider(MyService, token="my-service")
        
        assert get_provider("my-service") == MyService
    
    def test_register_provider_automatic_with_token(self):
        """Test registro automático cuando se usa token."""
        @injectable(token="auto-service")
        class AutoService:
            pass
        
        # Debe estar registrado automáticamente
        assert get_provider("auto-service") == AutoService
    
    def test_get_provider_returns_none_for_unknown_token(self):
        """Test get_provider retorna None para token desconocido."""
        assert get_provider("unknown-token") is None
    
    def test_clear_registry_removes_all_providers(self):
        """Test clear_registry limpia todos los providers."""
        @injectable(token="service1")
        class Service1:
            pass
        
        @injectable(token="service2")
        class Service2:
            pass
        
        # Verificar que están registrados
        assert get_provider("service1") is not None
        assert get_provider("service2") is not None
        
        # Limpiar registry
        clear_registry()
        
        # Verificar que se limpiaron
        assert get_provider("service1") is None
        assert get_provider("service2") is None
    
    def test_register_provider_duplicate_token_overwrites(self):
        """Test que registrar mismo token sobrescribe el anterior."""
        class Service1:
            pass
        
        class Service2:
            pass
        
        register_provider(Service1, token="my-service")
        assert get_provider("my-service") == Service1
        
        register_provider(Service2, token="my-service")
        assert get_provider("my-service") == Service2


class TestInjectableEdgeCases:
    """Tests de edge cases para @injectable."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        clear_registry()
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        clear_registry()
    
    def test_injectable_with_all_parameters(self):
        """Test decorador con todos los parámetros."""
        def custom_factory():
            return "custom instance"
        
        @injectable(
            scope=Scope.SCOPED,
            token="full-service",
            factory=custom_factory
        )
        class FullService:
            pass
        
        assert is_injectable(FullService)
        assert get_scope(FullService) == Scope.SCOPED
        assert get_token(FullService) == "full-service"
        
        metadata = get_injectable_metadata(FullService)
        assert metadata.factory is not None
        assert metadata.factory() == "custom instance"
    
    def test_injectable_preserves_class_name(self):
        """Test que decorador preserva __name__ de la clase."""
        @injectable()
        class MyService:
            pass
        
        assert MyService.__name__ == "MyService"
    
    def test_injectable_preserves_class_docstring(self):
        """Test que decorador preserva __doc__ de la clase."""
        @injectable()
        class MyService:
            """This is my service."""
            pass
        
        assert MyService.__doc__ == "This is my service."
    
    def test_injectable_can_instantiate_class(self):
        """Test que clase decorada se puede instanciar normalmente."""
        @injectable()
        class MyService:
            def __init__(self, value):
                self.value = value
        
        instance = MyService(42)
        assert instance.value == 42
    
    def test_injectable_with_methods(self):
        """Test que métodos de clase funcionan normalmente."""
        @injectable()
        class Calculator:
            def add(self, a, b):
                return a + b
        
        calc = Calculator()
        assert calc.add(2, 3) == 5
    
    def test_injectable_with_class_attributes(self):
        """Test que atributos de clase se preservan."""
        @injectable()
        class MyService:
            CLASS_CONSTANT = "constant_value"
        
        assert MyService.CLASS_CONSTANT == "constant_value"
    
    def test_injectable_multiple_classes_same_module(self):
        """Test múltiples clases decoradas en mismo módulo."""
        @injectable(token="service-a")
        class ServiceA:
            pass
        
        @injectable(token="service-b")
        class ServiceB:
            pass
        
        @injectable(token="service-c")
        class ServiceC:
            pass
        
        assert is_injectable(ServiceA)
        assert is_injectable(ServiceB)
        assert is_injectable(ServiceC)
        
        assert get_provider("service-a") == ServiceA
        assert get_provider("service-b") == ServiceB
        assert get_provider("service-c") == ServiceC


class TestInjectableIntegration:
    """Tests de integración para @injectable."""
    
    def setup_method(self):
        """Setup antes de cada test."""
        clear_registry()
    
    def teardown_method(self):
        """Cleanup después de cada test."""
        clear_registry()
    
    def test_injectable_dependency_chain_metadata(self):
        """Test cadena de dependencias con metadata."""
        @injectable(token="repository")
        class UserRepository:
            pass
        
        @injectable(token="service")
        class UserService:
            def __init__(self, repository: UserRepository):
                self.repository = repository
        
        @injectable(token="controller")
        class UserController:
            def __init__(self, service: UserService):
                self.service = service
        
        # Verificar que todas están registradas
        assert get_provider("repository") == UserRepository
        assert get_provider("service") == UserService
        assert get_provider("controller") == UserController
    
    def test_injectable_scope_variations(self):
        """Test variaciones de scope en misma app."""
        @injectable(scope=Scope.SINGLETON, token="singleton-srv")
        class SingletonService:
            pass
        
        @injectable(scope=Scope.TRANSIENT, token="transient-srv")
        class TransientService:
            pass
        
        @injectable(scope=Scope.SCOPED, token="scoped-srv")
        class ScopedService:
            pass
        
        assert get_scope(SingletonService) == Scope.SINGLETON
        assert get_scope(TransientService) == Scope.TRANSIENT
        assert get_scope(ScopedService) == Scope.SCOPED


# Ejecutar tests si se corre directamente
if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
