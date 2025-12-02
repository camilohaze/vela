"""
Tests unitarios para Lifecycle Management

Implementación de: TASK-035G
Historia: VELA-575
Fecha: 2025-12-02

Suite de tests para lifecycle management del sistema DI:
- OnDisposable protocol
- Disposal tracking con orden LIFO
- Scope hierarchy (parent/child)
- Lifecycle hooks
- Async disposal
- Scope isolation para testing
"""

import pytest
import asyncio
from typing import List
from src.runtime.di.lifecycle import (
    OnDisposable,
    AsyncOnDisposable,
    LifecycleHooks,
    ScopeContext,
    IsolatedScope,
    isolated_scope
)
from src.runtime.di.injector import Injector
from src.runtime.di.scopes import Scope
from src.runtime.di.injectable import injectable


# ========================================
# Test Fixtures y Helpers
# ========================================

class DisposableService(OnDisposable):
    """Servicio de prueba con OnDisposable."""
    
    def __init__(self):
        self.disposed = False
        self.disposal_order = []
    
    def dispose(self) -> None:
        self.disposed = True


class AsyncDisposableService(AsyncOnDisposable):
    """Servicio de prueba con AsyncOnDisposable."""
    
    def __init__(self):
        self.disposed = False
    
    async def dispose_async(self) -> None:
        await asyncio.sleep(0.01)  # Simular I/O
        self.disposed = True


class DatabaseConnection(OnDisposable):
    """Mock de conexión DB."""
    
    disposal_log: List[str] = []
    
    def __init__(self, name: str):
        self.name = name
        self.connected = True
    
    def dispose(self) -> None:
        self.connected = False
        DatabaseConnection.disposal_log.append(self.name)


class UserService(OnDisposable):
    """Mock de servicio de usuario."""
    
    disposal_log: List[str] = []
    
    def __init__(self, db: DatabaseConnection):
        self.db = db
    
    def dispose(self) -> None:
        UserService.disposal_log.append("UserService")


# ========================================
# Tests - OnDisposable Protocol
# ========================================

class TestOnDisposableProtocol:
    """Tests de OnDisposable protocol."""
    
    def test_disposable_interface(self):
        """OnDisposable protocol debe ser implementable."""
        service = DisposableService()
        assert isinstance(service, OnDisposable)
        assert hasattr(service, 'dispose')
        assert callable(service.dispose)
    
    def test_dispose_called(self):
        """dispose() debe ser llamado correctamente."""
        service = DisposableService()
        assert not service.disposed
        
        service.dispose()
        
        assert service.disposed
    
    def test_multiple_disposables(self):
        """Múltiples instancias OnDisposable deben ser independientes."""
        service1 = DisposableService()
        service2 = DisposableService()
        
        service1.dispose()
        
        assert service1.disposed
        assert not service2.disposed


# ========================================
# Tests - AsyncOnDisposable Protocol
# ========================================

class TestAsyncOnDisposableProtocol:
    """Tests de AsyncOnDisposable protocol."""
    
    def test_async_disposable_interface(self):
        """AsyncOnDisposable protocol debe ser implementable."""
        service = AsyncDisposableService()
        assert isinstance(service, AsyncOnDisposable)
        assert hasattr(service, 'dispose_async')
        assert callable(service.dispose_async)
    
    @pytest.mark.asyncio
    async def test_async_dispose_called(self):
        """dispose_async() debe ser awaitable."""
        service = AsyncDisposableService()
        assert not service.disposed
        
        await service.dispose_async()
        
        assert service.disposed


# ========================================
# Tests - LifecycleHooks
# ========================================

class TestLifecycleHooks:
    """Tests de lifecycle hooks."""
    
    def test_hooks_initialization(self):
        """LifecycleHooks debe inicializarse correctamente."""
        hooks = LifecycleHooks()
        assert hooks.on_create_callbacks == []
        assert hooks.on_dispose_callbacks == []
    
    def test_on_create_callback(self):
        """on_create debe registrar y ejecutar callbacks."""
        hooks = LifecycleHooks()
        created_instances = []
        
        def on_create(instance):
            created_instances.append(instance)
        
        hooks.on_create(on_create)
        
        service = DisposableService()
        hooks.notify_created(service)
        
        assert service in created_instances
    
    def test_on_dispose_callback(self):
        """on_dispose debe registrar y ejecutar callbacks."""
        hooks = LifecycleHooks()
        disposed_instances = []
        
        def on_dispose(instance):
            disposed_instances.append(instance)
        
        hooks.on_dispose(on_dispose)
        
        service = DisposableService()
        hooks.notify_disposed(service)
        
        assert service in disposed_instances
    
    def test_multiple_create_callbacks(self):
        """Múltiples callbacks on_create deben ejecutarse."""
        hooks = LifecycleHooks()
        call_count = [0]
        
        def callback1(instance):
            call_count[0] += 1
        
        def callback2(instance):
            call_count[0] += 10
        
        hooks.on_create(callback1)
        hooks.on_create(callback2)
        
        service = DisposableService()
        hooks.notify_created(service)
        
        assert call_count[0] == 11  # 1 + 10
    
    def test_hook_error_handling(self):
        """Errores en callbacks no deben interrumpir ejecución."""
        hooks = LifecycleHooks()
        executed = []
        
        def failing_callback(instance):
            raise ValueError("Test error")
        
        def success_callback(instance):
            executed.append(instance)
        
        hooks.on_create(failing_callback)
        hooks.on_create(success_callback)
        
        service = DisposableService()
        hooks.notify_created(service)
        
        # El segundo callback debe ejecutarse a pesar del error
        assert service in executed


# ========================================
# Tests - ScopeContext (Hierarchy)
# ========================================

class TestScopeContextHierarchy:
    """Tests de scope hierarchy."""
    
    def test_scope_context_initialization(self):
        """ScopeContext debe inicializarse correctamente."""
        scope = ScopeContext()
        assert scope.parent is None
        assert scope.children == []
        assert scope.disposables == []
        assert not scope.is_disposed()
    
    def test_create_child_scope(self):
        """create_child() debe crear scope hijo."""
        parent = ScopeContext()
        child = parent.create_child()
        
        assert child.parent == parent
        assert child in parent.children
    
    def test_child_inherits_hooks(self):
        """Child scope debe heredar hooks del padre."""
        parent = ScopeContext()
        created_instances = []
        
        def on_create(instance):
            created_instances.append(instance)
        
        parent.hooks.on_create(on_create)
        
        # Crear child scope
        child = parent.create_child()
        
        # Trackear instancia en child
        service = DisposableService()
        child.track_disposable(service)
        
        # Hook del padre debe ejecutarse
        assert service in created_instances
    
    def test_track_disposable(self):
        """track_disposable() debe agregar instancia a lista."""
        scope = ScopeContext()
        service = DisposableService()
        
        scope.track_disposable(service)
        
        assert service in scope.disposables
    
    def test_get_depth(self):
        """get_depth() debe retornar profundidad en hierarchy."""
        root = ScopeContext()
        child1 = root.create_child()
        child2 = child1.create_child()
        
        assert root.get_depth() == 0
        assert child1.get_depth() == 1
        assert child2.get_depth() == 2


# ========================================
# Tests - Disposal Order (LIFO)
# ========================================

class TestDisposalOrderLIFO:
    """Tests de disposal order (LIFO)."""
    
    def test_lifo_disposal_order(self):
        """Instancias deben disponerse en orden LIFO."""
        DatabaseConnection.disposal_log.clear()
        
        scope = ScopeContext()
        
        # Crear instancias en orden: db1, db2, db3
        db1 = DatabaseConnection("db1")
        db2 = DatabaseConnection("db2")
        db3 = DatabaseConnection("db3")
        
        scope.track_disposable(db1)
        scope.track_disposable(db2)
        scope.track_disposable(db3)
        
        # Disponer
        scope.dispose_all()
        
        # Orden de disposal debe ser: db3, db2, db1 (LIFO)
        assert DatabaseConnection.disposal_log == ["db3", "db2", "db1"]
    
    def test_recursive_disposal_children_first(self):
        """Children scopes deben disponerse antes que parent."""
        DatabaseConnection.disposal_log.clear()
        
        parent = ScopeContext()
        child = parent.create_child()
        
        # Parent tiene db1
        db1 = DatabaseConnection("parent_db")
        parent.track_disposable(db1)
        
        # Child tiene db2
        db2 = DatabaseConnection("child_db")
        child.track_disposable(db2)
        
        # Disponer parent (debe disponer child primero)
        parent.dispose_all()
        
        # Orden: child_db primero, luego parent_db
        assert DatabaseConnection.disposal_log == ["child_db", "parent_db"]
    
    def test_disposal_marks_disposed(self):
        """dispose_all() debe marcar scope como disposed."""
        scope = ScopeContext()
        service = DisposableService()
        scope.track_disposable(service)
        
        assert not scope.is_disposed()
        
        scope.dispose_all()
        
        assert scope.is_disposed()


# ========================================
# Tests - Async Disposal
# ========================================

class TestAsyncDisposal:
    """Tests de async disposal."""
    
    @pytest.mark.asyncio
    async def test_async_dispose_all(self):
        """dispose_all_async() debe awaitar AsyncOnDisposable."""
        scope = ScopeContext()
        service = AsyncDisposableService()
        
        scope.track_disposable(service)
        
        await scope.dispose_all_async()
        
        assert service.disposed
    
    @pytest.mark.asyncio
    async def test_mixed_sync_async_disposal(self):
        """dispose_all_async() debe manejar mix de sync y async."""
        scope = ScopeContext()
        
        sync_service = DisposableService()
        async_service = AsyncDisposableService()
        
        scope.track_disposable(sync_service)
        scope.track_disposable(async_service)
        
        await scope.dispose_all_async()
        
        assert sync_service.disposed
        assert async_service.disposed


# ========================================
# Tests - Injector Integration
# ========================================

class TestInjectorIntegration:
    """Tests de integración con Injector."""
    
    def test_injector_tracks_singleton_disposables(self):
        """Injector debe trackear singletons OnDisposable."""
        injector = Injector()
        
        @injectable(scope=Scope.SINGLETON)
        class MyService(OnDisposable):
            def __init__(self):
                self.disposed = False
            
            def dispose(self):
                self.disposed = True
        
        injector.register(MyService)
        service = injector.get(MyService)
        
        assert not service.disposed
        
        # Disponer singletons
        injector.dispose_scope(Scope.SINGLETON)
        
        assert service.disposed
    
    def test_dispose_all_calls_dispose_scope(self):
        """dispose() debe disponer todos los scopes."""
        injector = Injector()
        
        @injectable(scope=Scope.SINGLETON)
        class MyService(OnDisposable):
            def __init__(self):
                self.disposed = False
            
            def dispose(self):
                self.disposed = True
        
        injector.register(MyService)
        service = injector.get(MyService)
        
        injector.dispose()
        
        assert service.disposed
    
    @pytest.mark.asyncio
    async def test_async_dispose_integration(self):
        """dispose_async() debe disponer instancias async."""
        injector = Injector()
        
        @injectable(scope=Scope.SINGLETON)
        class MyAsyncService(AsyncOnDisposable):
            def __init__(self):
                self.disposed = False
            
            async def dispose_async(self):
                await asyncio.sleep(0.01)
                self.disposed = True
        
        injector.register(MyAsyncService)
        service = injector.get(MyAsyncService)
        
        await injector.dispose_async()
        
        assert service.disposed


# ========================================
# Tests - Scope Isolation (Testing)
# ========================================

# TODO: Implementar snapshot/restore en ProviderRegistry para habilitar estos tests
"""
class TestScopeIsolation:
    Tests de scope isolation para testing.
    
    def test_isolated_scope_context_manager(self):
        IsolatedScope debe funcionar como context manager.
        injector = Injector()
        
        with isolated_scope(injector) as scope:
            assert isinstance(scope, ScopeContext)
    
    def test_isolated_scope_auto_cleanup(self):
        IsolatedScope debe limpiar automáticamente al salir.
        injector = Injector()
        service = None
        
        @injectable(scope=Scope.SINGLETON)
        class TestService(OnDisposable):
            def __init__(self):
                self.disposed = False
            
            def dispose(self):
                self.disposed = True
        
        with isolated_scope(injector) as scope:
            injector.register(TestService)
            service = injector.get(TestService)
            assert not service.disposed
        
        # Después de salir del context, dispose debe ser llamado
        assert service.disposed
    
    def test_isolated_scope_restores_registry(self):
        IsolatedScope debe restaurar registry original.
        injector = Injector()
        
        @injectable
        class OriginalService:
            pass
        
        injector.register(OriginalService)
        assert injector.has_provider(OriginalService)
        
        with isolated_scope(injector) as scope:
            # Clear registry dentro del scope
            @injectable
            class TestService:
                pass
            
            injector.register(TestService)
            assert injector.has_provider(TestService)
        
        # Después de salir, registry debe restaurarse
        # (TestService no debe estar, OriginalService sí)
        # Nota: Esta funcionalidad depende de snapshot/restore en ProviderRegistry
"""


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
