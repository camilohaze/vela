"""
System Tests - DI Edge Cases

Tests de casos extremos y manejo de errores:
- Missing providers
- Circular dependencies
- Invalid scopes
- Async provider errors
- Disposal errors
- Concurrent access
- Memory leaks
- Provider registration después de resolve

Jira: VELA-575, TASK-035J
"""

import pytest
import asyncio
import threading
import gc
import weakref
from src.runtime.di import (
    Injector, 
    injectable, 
    Scope, 
    DependencyNotFoundError,
    CircularDependencyError
)
from tests.system.fixtures.services import (
    DatabaseConnection,
    UserRepository,
    UserService
)


# ============================================================================
# TESTS DE MISSING PROVIDERS
# ============================================================================

class TestMissingProviders:
    """Tests de dependencias no registradas."""
    
    def test_missing_provider_raises_error(self, injector):
        """Test: Dependencia no registrada lanza DependencyNotFoundError."""
        
        @injectable
        class MissingService:
            pass
        
        # NO registrar MissingService
        
        # Act & Assert
        with pytest.raises(DependencyNotFoundError) as exc_info:
            injector.get(MissingService)
        
        assert "MissingService" in str(exc_info.value)
        assert "not found" in str(exc_info.value).lower()
    
    def test_missing_nested_dependency_raises_error(self, injector):
        """Test: Dependencia anidada no registrada lanza error."""
        
        @injectable
        class NestedService:
            pass
        
        @injectable
        class ParentService:
            def __init__(self, nested: NestedService):
                self.nested = nested
        
        # Registrar solo ParentService (NO NestedService)
        injector.register(ParentService)
        
        # Act & Assert
        with pytest.raises(DependencyNotFoundError) as exc_info:
            injector.get(ParentService)
        
        assert "NestedService" in str(exc_info.value)
    
    def test_missing_string_token_raises_error(self, injector):
        """Test: String token no registrado lanza error."""
        
        # Act & Assert
        with pytest.raises(DependencyNotFoundError) as exc_info:
            injector.get("NON_EXISTENT_TOKEN")
        
        assert "NON_EXISTENT_TOKEN" in str(exc_info.value)


# ============================================================================
# TESTS DE CIRCULAR DEPENDENCIES
# ============================================================================

class TestCircularDependencies:
    """Tests de dependencias circulares."""
    
    def test_circular_dependency_2_classes(self, injector):
        """Test: Circular dependency A → B → A."""
        
        @injectable
        class ServiceA:
            def __init__(self, b: 'ServiceB'):
                self.b = b
        
        @injectable
        class ServiceB:
            def __init__(self, a: ServiceA):
                self.a = a
        
        injector.register(ServiceA)
        injector.register(ServiceB)
        
        # Act & Assert
        with pytest.raises(CircularDependencyError) as exc_info:
            injector.get(ServiceA)
        
        error_msg = str(exc_info.value)
        assert "ServiceA" in error_msg
        assert "ServiceB" in error_msg
        assert "circular" in error_msg.lower()
    
    def test_circular_dependency_3_classes(self, injector):
        """Test: Circular dependency A → B → C → A."""
        
        @injectable
        class ServiceA:
            def __init__(self, b: 'ServiceB'):
                self.b = b
        
        @injectable
        class ServiceB:
            def __init__(self, c: 'ServiceC'):
                self.c = c
        
        @injectable
        class ServiceC:
            def __init__(self, a: ServiceA):
                self.a = a
        
        injector.register(ServiceA)
        injector.register(ServiceB)
        injector.register(ServiceC)
        
        # Act & Assert
        with pytest.raises(CircularDependencyError) as exc_info:
            injector.get(ServiceA)
        
        error_msg = str(exc_info.value)
        assert "ServiceA" in error_msg
        assert "ServiceB" in error_msg
        assert "ServiceC" in error_msg
    
    def test_self_circular_dependency(self, injector):
        """Test: Self circular dependency A → A."""
        
        @injectable
        class SelfService:
            def __init__(self, self_ref: 'SelfService'):
                self.self_ref = self_ref
        
        injector.register(SelfService)
        
        # Act & Assert
        with pytest.raises(CircularDependencyError):
            injector.get(SelfService)


# ============================================================================
# TESTS DE ASYNC PROVIDERS
# ============================================================================

class TestAsyncProviderErrors:
    """Tests de errores en async providers."""
    
    @pytest.mark.asyncio
    async def test_async_provider_error_propagates(self, injector):
        """Test: Error en async provider se propaga."""
        
        @injectable
        class AsyncService:
            pass
        
        async def failing_factory() -> AsyncService:
            await asyncio.sleep(0.01)
            raise ValueError("Async factory failed")
        
        injector.register_async_factory(AsyncService, failing_factory)
        
        # Act & Assert
        with pytest.raises(ValueError) as exc_info:
            await injector.get_async(AsyncService)
        
        assert "Async factory failed" in str(exc_info.value)
    
    @pytest.mark.asyncio
    async def test_sync_get_on_async_provider_raises_error(self, injector):
        """Test: Llamar get() en async provider lanza error."""
        
        @injectable
        class AsyncService:
            pass
        
        async def async_factory() -> AsyncService:
            await asyncio.sleep(0.01)
            return AsyncService()
        
        injector.register_async_factory(AsyncService, async_factory)
        
        # Act & Assert
        with pytest.raises(RuntimeError) as exc_info:
            injector.get(AsyncService)  # Sync get on async provider
        
        assert "async" in str(exc_info.value).lower()


# ============================================================================
# TESTS DE DISPOSAL ERRORS
# ============================================================================

class TestDisposalErrors:
    """Tests de errores en disposal/cleanup."""
    
    def test_dispose_error_does_not_block_others(self, injector):
        """Test: Error en dispose() no bloquea otros dispose()."""
        
        disposal_order = []
        
        @injectable
        class Service1:
            def on_destroy(self):
                disposal_order.append("service1")
                raise ValueError("Service1 dispose failed")
        
        @injectable
        class Service2:
            def on_destroy(self):
                disposal_order.append("service2")
        
        @injectable
        class Service3:
            def on_destroy(self):
                disposal_order.append("service3")
        
        injector.register(Service1)
        injector.register(Service2)
        injector.register(Service3)
        
        # Crear instancias
        injector.get(Service1)
        injector.get(Service2)
        injector.get(Service3)
        
        # Act: Dispose (con error en Service1)
        try:
            injector.dispose()
        except ValueError:
            pass  # Ignorar error de Service1
        
        # Assert: Todos se dispusieron (error no bloqueó)
        assert "service1" in disposal_order
        assert "service2" in disposal_order
        assert "service3" in disposal_order
    
    def test_dispose_called_in_reverse_order(self, injector):
        """Test: Dispose se llama en orden inverso de creación."""
        
        disposal_order = []
        
        @injectable
        class Service1:
            def on_destroy(self):
                disposal_order.append("service1")
        
        @injectable
        class Service2:
            def on_destroy(self):
                disposal_order.append("service2")
        
        @injectable
        class Service3:
            def on_destroy(self):
                disposal_order.append("service3")
        
        injector.register(Service1)
        injector.register(Service2)
        injector.register(Service3)
        
        # Crear en orden: Service1, Service2, Service3
        injector.get(Service1)
        injector.get(Service2)
        injector.get(Service3)
        
        # Act
        injector.dispose()
        
        # Assert: Dispose en orden inverso
        assert disposal_order == ["service3", "service2", "service1"]


# ============================================================================
# TESTS DE CONCURRENT ACCESS
# ============================================================================

class TestConcurrentAccess:
    """Tests de acceso concurrente (thread-safety)."""
    
    def test_singleton_concurrent_access_thread_safe(self, injector):
        """Test: SINGLETON es thread-safe (misma instancia en threads)."""
        
        @injectable(scope=Scope.SINGLETON)
        class SingletonService:
            def __init__(self):
                self.id = id(self)
        
        injector.register(SingletonService)
        
        instances = []
        
        def worker():
            instance = injector.get(SingletonService)
            instances.append(instance)
        
        # Act: 10 threads concurrentes
        threads = [threading.Thread(target=worker) for _ in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        
        # Assert: Todas las instancias son la misma
        assert len(instances) == 10
        first_instance = instances[0]
        for instance in instances:
            assert instance is first_instance
    
    def test_concurrent_scope_creation(self, injector):
        """Test: Creación concurrente de scopes."""
        
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            def __init__(self):
                self.id = id(self)
        
        injector.register(ScopedService)
        
        scope_ids = []
        
        def worker():
            with injector.create_scope() as scope:
                service = scope.get(ScopedService)
                scope_ids.append(service.id)
        
        # Act: 10 threads, cada uno crea su scope
        threads = [threading.Thread(target=worker) for _ in range(10)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()
        
        # Assert: Todos los scopes tienen instancias diferentes
        assert len(scope_ids) == 10
        assert len(set(scope_ids)) == 10  # 10 IDs únicos


# ============================================================================
# TESTS DE MEMORY LEAKS
# ============================================================================

class TestMemoryLeaks:
    """Tests de memory leaks prevention."""
    
    def test_transient_instances_garbage_collected(self, injector):
        """Test: Instancias TRANSIENT son garbage collected."""
        
        @injectable(scope=Scope.TRANSIENT)
        class TransientService:
            pass
        
        injector.register(TransientService)
        
        # Act: Crear instancia y referencia débil
        instance = injector.get(TransientService)
        weak_ref = weakref.ref(instance)
        
        # Assert: Referencia débil válida
        assert weak_ref() is not None
        
        # Eliminar referencia fuerte
        del instance
        
        # Forzar garbage collection
        gc.collect()
        
        # Assert: Instancia fue garbage collected
        assert weak_ref() is None
    
    def test_scoped_instances_garbage_collected_after_scope_exit(self, injector):
        """Test: Instancias SCOPED son garbage collected después del scope."""
        
        @injectable(scope=Scope.SCOPED)
        class ScopedService:
            pass
        
        injector.register(ScopedService)
        
        weak_ref = None
        
        # Act: Crear scope y instancia
        with injector.create_scope() as scope:
            instance = scope.get(ScopedService)
            weak_ref = weakref.ref(instance)
            
            # Assert: Dentro del scope, instancia viva
            assert weak_ref() is not None
        
        # Forzar garbage collection
        gc.collect()
        
        # Assert: Fuera del scope, instancia garbage collected
        assert weak_ref() is None


# ============================================================================
# TESTS DE REGISTRATION DESPUÉS DE RESOLVE
# ============================================================================

class TestLateRegistration:
    """Tests de registro de providers después de resolve."""
    
    def test_register_after_resolve_works(self, injector):
        """Test: Registrar después de resolve() funciona."""
        
        @injectable
        class Service1:
            pass
        
        @injectable
        class Service2:
            pass
        
        injector.register(Service1)
        
        # Resolver Service1
        service1 = injector.get(Service1)
        assert service1 is not None
        
        # Act: Registrar Service2 DESPUÉS de resolver Service1
        injector.register(Service2)
        
        # Assert: Service2 se puede resolver
        service2 = injector.get(Service2)
        assert service2 is not None


# ============================================================================
# TESTS DE OVERRIDE() CON TIPO INCOMPATIBLE
# ============================================================================

class TestIncompatibleOverride:
    """Tests de override con tipo incompatible."""
    
    def test_override_with_incompatible_type_works(self, injector):
        """Test: Override con tipo incompatible (duck typing)."""
        
        @injectable
        class OriginalService:
            def method(self):
                return "original"
        
        class MockService:
            def method(self):
                return "mock"
        
        injector.register(OriginalService)
        
        # Act: Override con tipo diferente (pero compatible)
        injector.override(OriginalService, MockService())
        
        # Assert: Retorna el mock
        service = injector.get(OriginalService)
        assert service.method() == "mock"


# ============================================================================
# TESTS DE INJECT() CON DEFAULT VALUE
# ============================================================================

class TestInjectDefaultValue:
    """Tests de inject() con default value."""
    
    def test_inject_with_default_value(self, injector):
        """Test: inject() con default value si provider no existe."""
        
        from src.runtime.di import inject
        
        @injectable
        class Service:
            def __init__(self):
                # inject() con default
                self.config = inject("Config", default={"default": True})
        
        injector.register(Service)
        
        # NO registrar "Config"
        
        # Act
        service = injector.get(Service)
        
        # Assert: Usa default value
        assert service.config == {"default": True}


# ============================================================================
# TESTS DE @injectable SIN REGISTER()
# ============================================================================

class TestInjectableWithoutRegister:
    """Tests de @injectable sin register()."""
    
    def test_injectable_without_register_fails(self, injector):
        """Test: @injectable sin register() falla."""
        
        @injectable
        class UnregisteredService:
            pass
        
        # NO registrar UnregisteredService
        
        # Act & Assert
        with pytest.raises(DependencyNotFoundError):
            injector.get(UnregisteredService)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
