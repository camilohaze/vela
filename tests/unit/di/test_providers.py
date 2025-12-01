"""
Tests para @provides Decorator y Providers System

Implementación de: TASK-035E
Historia: VELA-575
Fecha: 2025-12-01

Version: 0.1.0
"""

import asyncio
import pytest
from typing import List
from src.runtime.di.providers import (
    ProviderScope,
    ProviderMetadata,
    provides,
    is_provider,
    get_provider_metadata,
    get_all_providers,
    get_providers_by_scope,
    get_provider_by_token,
)


# ===================================
# Test ProviderScope
# ===================================

class TestProviderScope:
    """Tests para ProviderScope enum."""
    
    def test_scope_values(self):
        """Test valores de ProviderScope."""
        assert ProviderScope.SINGLETON.value == "singleton"
        assert ProviderScope.TRANSIENT.value == "transient"
        assert ProviderScope.SCOPED.value == "scoped"
    
    def test_scope_string_conversion(self):
        """Test conversión a string."""
        assert str(ProviderScope.SINGLETON) == "singleton"
        assert str(ProviderScope.TRANSIENT) == "transient"
        assert str(ProviderScope.SCOPED) == "scoped"
    
    def test_scope_comparison(self):
        """Test comparación de scopes."""
        assert ProviderScope.SINGLETON == ProviderScope.SINGLETON
        assert ProviderScope.SINGLETON != ProviderScope.TRANSIENT
    
    def test_scope_repr(self):
        """Test representación de scopes."""
        assert repr(ProviderScope.SINGLETON) == "ProviderScope.SINGLETON"


# ===================================
# Test ProviderMetadata
# ===================================

class TestProviderMetadata:
    """Tests para ProviderMetadata dataclass."""
    
    def test_metadata_creation(self):
        """Test creación básica de metadata."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(
            token=str,
            factory=factory,
            scope=ProviderScope.SINGLETON
        )
        
        assert metadata.token == str
        assert metadata.factory == factory
        assert metadata.scope == ProviderScope.SINGLETON
        assert metadata.is_async is False
        assert metadata.deps == []
        assert metadata.multi is False
        assert metadata.description is None
    
    def test_metadata_with_all_fields(self):
        """Test metadata con todos los campos."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(
            token="MY_TOKEN",
            factory=factory,
            scope=ProviderScope.TRANSIENT,
            is_async=True,
            deps=[int, str],
            multi=True,
            description="Test provider"
        )
        
        assert metadata.token == "MY_TOKEN"
        assert metadata.is_async is True
        assert len(metadata.deps) == 2
        assert metadata.multi is True
        assert metadata.description == "Test provider"
    
    def test_metadata_validates_factory_callable(self):
        """Test que factory debe ser callable."""
        with pytest.raises(TypeError, match="must be callable"):
            ProviderMetadata(
                token=str,
                factory="not_callable",  # ERROR
                scope=ProviderScope.SINGLETON
            )
    
    def test_metadata_validates_scope_type(self):
        """Test que scope debe ser ProviderScope."""
        def factory():
            return "value"
        
        with pytest.raises(TypeError, match="must be ProviderScope"):
            ProviderMetadata(
                token=str,
                factory=factory,
                scope="singleton"  # ERROR: debe ser ProviderScope.SINGLETON
            )
    
    def test_metadata_validates_string_token(self):
        """Test que token string no puede estar vacío."""
        def factory():
            return "value"
        
        with pytest.raises(ValueError, match="cannot be empty"):
            ProviderMetadata(
                token="   ",  # ERROR: empty after strip
                factory=factory
            )
    
    def test_get_token_name_with_type(self):
        """Test get_token_name con Type."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(token=str, factory=factory)
        assert metadata.get_token_name() == "str"
    
    def test_get_token_name_with_string(self):
        """Test get_token_name con string."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(token="MY_TOKEN", factory=factory)
        assert metadata.get_token_name() == "MY_TOKEN"
    
    def test_is_singleton(self):
        """Test is_singleton method."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(token=str, factory=factory, scope=ProviderScope.SINGLETON)
        assert metadata.is_singleton() is True
        assert metadata.is_transient() is False
        assert metadata.is_scoped() is False
    
    def test_is_transient(self):
        """Test is_transient method."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(token=str, factory=factory, scope=ProviderScope.TRANSIENT)
        assert metadata.is_singleton() is False
        assert metadata.is_transient() is True
        assert metadata.is_scoped() is False
    
    def test_is_scoped(self):
        """Test is_scoped method."""
        def factory():
            return "value"
        
        metadata = ProviderMetadata(token=str, factory=factory, scope=ProviderScope.SCOPED)
        assert metadata.is_singleton() is False
        assert metadata.is_transient() is False
        assert metadata.is_scoped() is True


# ===================================
# Test @provides Decorator
# ===================================

class TestProvidesDecorator:
    """Tests para @provides decorator."""
    
    def test_provides_basic(self):
        """Test @provides básico."""
        @provides(str)
        def provide_string():
            return "hello"
        
        assert is_provider(provide_string)
        metadata = get_provider_metadata(provide_string)
        assert metadata is not None
        assert metadata.token == str
        assert metadata.scope == ProviderScope.SINGLETON  # default
    
    def test_provides_with_scope(self):
        """Test @provides con scope personalizado."""
        @provides(int, scope=ProviderScope.TRANSIENT)
        def provide_int():
            return 42
        
        metadata = get_provider_metadata(provide_int)
        assert metadata.scope == ProviderScope.TRANSIENT
    
    def test_provides_with_string_token(self):
        """Test @provides con token string."""
        @provides("API_KEY", description="API key from env")
        def provide_api_key():
            return "secret123"
        
        metadata = get_provider_metadata(provide_api_key)
        assert metadata.token == "API_KEY"
        assert metadata.description == "API key from env"
    
    def test_provides_auto_detects_sync(self):
        """Test que @provides detecta factory sync."""
        @provides(str)
        def provide_sync():
            return "sync"
        
        metadata = get_provider_metadata(provide_sync)
        assert metadata.is_async is False
    
    def test_provides_auto_detects_async(self):
        """Test que @provides detecta factory async."""
        @provides(str)
        async def provide_async():
            return "async"
        
        metadata = get_provider_metadata(provide_async)
        assert metadata.is_async is True
    
    def test_provides_auto_detects_dependencies(self):
        """Test auto-detección de dependencias."""
        @provides(str)
        def provide_with_deps(dep1: int, dep2: float) -> str:
            return "result"
        
        metadata = get_provider_metadata(provide_with_deps)
        assert int in metadata.deps
        assert float in metadata.deps
        assert len(metadata.deps) == 2
    
    def test_provides_explicit_dependencies(self):
        """Test dependencias explícitas."""
        @provides(str, deps=[int, "CONFIG"])
        def provide_with_explicit_deps():
            return "result"
        
        metadata = get_provider_metadata(provide_with_explicit_deps)
        assert metadata.deps == [int, "CONFIG"]
    
    def test_provides_multi_provider(self):
        """Test multi provider."""
        @provides(str, multi=True)
        def provide_multiple():
            return ["a", "b", "c"]
        
        metadata = get_provider_metadata(provide_multiple)
        assert metadata.multi is True
    
    def test_provides_ignores_self_parameter(self):
        """Test que @provides ignora 'self' en auto-detection."""
        class TestClass:
            @provides(str)
            def provide_method(self, dep: int) -> str:
                return "result"
        
        metadata = get_provider_metadata(TestClass.provide_method)
        assert int in metadata.deps
        assert len(metadata.deps) == 1  # Solo 'dep', no 'self'
    
    def test_provides_preserves_function_execution(self):
        """Test que función decorada sigue ejecutable."""
        @provides(int)
        def provide_number():
            return 42
        
        # Función sigue ejecutable
        assert provide_number() == 42


# ===================================
# Test Helper Functions
# ===================================

class TestHelperFunctions:
    """Tests para helper functions."""
    
    def test_is_provider_true(self):
        """Test is_provider con provider."""
        @provides(str)
        def my_provider():
            return "value"
        
        assert is_provider(my_provider) is True
    
    def test_is_provider_false(self):
        """Test is_provider con función normal."""
        def normal_function():
            return "value"
        
        assert is_provider(normal_function) is False
    
    def test_get_provider_metadata_exists(self):
        """Test get_provider_metadata cuando existe."""
        @provides(str, scope=ProviderScope.TRANSIENT)
        def my_provider():
            return "value"
        
        metadata = get_provider_metadata(my_provider)
        assert metadata is not None
        assert metadata.token == str
    
    def test_get_provider_metadata_not_exists(self):
        """Test get_provider_metadata cuando no existe."""
        def normal_function():
            return "value"
        
        metadata = get_provider_metadata(normal_function)
        assert metadata is None
    
    def test_get_all_providers(self):
        """Test get_all_providers extrae todos los providers."""
        class TestModule:
            @provides(str)
            def provide_string(self):
                return "hello"
            
            @provides(int)
            def provide_int(self):
                return 42
            
            def normal_method(self):
                return "not a provider"
        
        providers = get_all_providers(TestModule)
        assert len(providers) == 2
        assert "provide_string" in providers
        assert "provide_int" in providers
        assert "normal_method" not in providers
    
    def test_get_providers_by_scope_singleton(self):
        """Test get_providers_by_scope filtra SINGLETON."""
        class TestModule:
            @provides(str, scope=ProviderScope.SINGLETON)
            def provide_singleton(self):
                return "singleton"
            
            @provides(int, scope=ProviderScope.TRANSIENT)
            def provide_transient(self):
                return 42
        
        singletons = get_providers_by_scope(TestModule, ProviderScope.SINGLETON)
        assert len(singletons) == 1
        assert "provide_singleton" in singletons
    
    def test_get_providers_by_scope_transient(self):
        """Test get_providers_by_scope filtra TRANSIENT."""
        class TestModule:
            @provides(str, scope=ProviderScope.SINGLETON)
            def provide_singleton(self):
                return "singleton"
            
            @provides(int, scope=ProviderScope.TRANSIENT)
            def provide_transient(self):
                return 42
        
        transients = get_providers_by_scope(TestModule, ProviderScope.TRANSIENT)
        assert len(transients) == 1
        assert "provide_transient" in transients
    
    def test_get_provider_by_token_found(self):
        """Test get_provider_by_token encuentra provider."""
        class TestModule:
            @provides(str)
            def provide_string(self):
                return "hello"
            
            @provides(int)
            def provide_int(self):
                return 42
        
        result = get_provider_by_token(TestModule, str)
        assert result is not None
        name, metadata = result
        assert name == "provide_string"
        assert metadata.token == str
    
    def test_get_provider_by_token_not_found(self):
        """Test get_provider_by_token no encuentra provider."""
        class TestModule:
            @provides(str)
            def provide_string(self):
                return "hello"
        
        result = get_provider_by_token(TestModule, int)
        assert result is None
    
    def test_get_provider_by_token_with_string_token(self):
        """Test get_provider_by_token con token string."""
        class TestModule:
            @provides("CONFIG")
            def provide_config(self):
                return {"key": "value"}
        
        result = get_provider_by_token(TestModule, "CONFIG")
        assert result is not None
        name, metadata = result
        assert name == "provide_config"
        assert metadata.token == "CONFIG"


# ===================================
# Test Integration
# ===================================

class TestIntegration:
    """Tests de integración de providers."""
    
    def test_complete_module_with_providers(self):
        """Test módulo completo con múltiples providers."""
        class AppModule:
            @provides("DATABASE_URL", scope=ProviderScope.SINGLETON)
            def provide_database_url(self) -> str:
                return "postgresql://localhost/mydb"
            
            @provides(str, scope=ProviderScope.SINGLETON, deps=["DATABASE_URL"])
            def provide_database(self, url: str):
                return f"Database({url})"
            
            @provides(int, scope=ProviderScope.TRANSIENT)
            def provide_counter(self) -> int:
                return 0
            
            def normal_method(self):
                return "not a provider"
        
        # Verificar todos los providers
        providers = get_all_providers(AppModule)
        assert len(providers) == 3
        
        # Verificar singletons
        singletons = get_providers_by_scope(AppModule, ProviderScope.SINGLETON)
        assert len(singletons) == 2
        
        # Verificar transients
        transients = get_providers_by_scope(AppModule, ProviderScope.TRANSIENT)
        assert len(transients) == 1
        
        # Buscar por token
        db_provider = get_provider_by_token(AppModule, str)
        assert db_provider is not None
        name, metadata = db_provider
        assert "DATABASE_URL" in metadata.deps
    
    def test_async_provider_integration(self):
        """Test integración de async provider."""
        class AsyncModule:
            @provides("API_DATA", scope=ProviderScope.SINGLETON)
            async def provide_api_data(self) -> dict:
                # Simular llamada async
                await asyncio.sleep(0.01)
                return {"data": "value"}
        
        metadata = get_provider_metadata(AsyncModule.provide_api_data)
        assert metadata.is_async is True
        assert metadata.token == "API_DATA"
    
    def test_multi_provider_integration(self):
        """Test integración de multi provider."""
        class PluginModule:
            @provides("PLUGINS", multi=True, scope=ProviderScope.SINGLETON)
            def provide_plugins(self) -> List[str]:
                return ["AuthPlugin", "LoggingPlugin", "CachePlugin"]
        
        metadata = get_provider_metadata(PluginModule.provide_plugins)
        assert metadata.multi is True
        assert metadata.token == "PLUGINS"


# ===================================
# Test Edge Cases
# ===================================

class TestEdgeCases:
    """Tests de edge cases."""
    
    def test_provider_with_no_parameters(self):
        """Test provider sin parámetros."""
        @provides(str)
        def provide_constant():
            return "CONSTANT"
        
        metadata = get_provider_metadata(provide_constant)
        assert metadata.deps == []
    
    def test_provider_with_complex_type_hints(self):
        """Test provider con type hints complejos."""
        @provides(str)
        def provide_complex(data: List[int], config: dict) -> str:
            return "result"
        
        metadata = get_provider_metadata(provide_complex)
        assert List[int] in metadata.deps
        assert dict in metadata.deps
    
    def test_empty_module_no_providers(self):
        """Test módulo sin providers."""
        class EmptyModule:
            def method1(self):
                pass
            
            def method2(self):
                pass
        
        providers = get_all_providers(EmptyModule)
        assert len(providers) == 0
    
    def test_provider_scope_defaults_to_singleton(self):
        """Test que scope por defecto es SINGLETON."""
        @provides(str)
        def provide_default():
            return "value"
        
        metadata = get_provider_metadata(provide_default)
        assert metadata.scope == ProviderScope.SINGLETON


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
