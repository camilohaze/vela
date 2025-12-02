"""
Lifecycle Management - Sistema de gestión de ciclo de vida

Implementación de: TASK-035G
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Sistema de lifecycle management con OnDisposable protocol,
disposal tracking, scope hierarchy, y async disposal support.

Referencias:
- Spring Framework: DisposableBean
- Angular: OnDestroy
- NestJS: OnModuleDestroy
"""

from typing import Protocol, runtime_checkable, Any, Callable, List, Optional
from dataclasses import dataclass, field
import asyncio
import logging

logger = logging.getLogger(__name__)


# ========================================
# Protocols
# ========================================

@runtime_checkable
class OnDisposable(Protocol):
    """
    Protocol para objetos que requieren cleanup.
    
    Clases que implementen este protocol recibirán llamada a
    dispose() cuando su scope sea destruido.
    
    Example:
        @injectable(scope=Scope.SCOPED)
        class DatabaseConnection(OnDisposable):
            def __init__(self):
                self.conn = connect()
            
            def dispose(self) -> None:
                self.conn.close()
    """
    
    def dispose(self) -> None:
        """
        Limpia recursos del objeto.
        
        Llamado automáticamente cuando el scope es destruido.
        Debe ser idempotente (se puede llamar múltiples veces).
        
        Raises:
            Exception: Cualquier error durante cleanup (será logged)
        """
        ...


@runtime_checkable
class AsyncOnDisposable(Protocol):
    """
    Protocol para cleanup asíncrono de recursos.
    
    Útil para conexiones I/O-bound que requieren await.
    
    Example:
        @injectable(scope=Scope.SINGLETON)
        class RedisClient(AsyncOnDisposable):
            async def connect(self):
                self.conn = await aioredis.connect()
            
            async def dispose_async(self) -> None:
                await self.conn.close()
    """
    
    async def dispose_async(self) -> None:
        """
        Limpieza asíncrona de recursos.
        
        Llamado automáticamente cuando el scope es destruido
        si se usa dispose_scope_async().
        
        Raises:
            Exception: Cualquier error durante cleanup (será logged)
        """
        ...


# ========================================
# Lifecycle Hooks
# ========================================

@dataclass
class LifecycleHooks:
    """
    Hooks de lifecycle para observar creación y disposal.
    
    Útil para logging, monitoring, y debugging.
    
    Example:
        hooks = LifecycleHooks()
        hooks.on_create(lambda inst: print(f"Created: {type(inst).__name__}"))
        hooks.on_dispose(lambda inst: print(f"Disposed: {type(inst).__name__}"))
    """
    
    on_create_callbacks: List[Callable[[Any], None]] = field(default_factory=list)
    on_dispose_callbacks: List[Callable[[Any], None]] = field(default_factory=list)
    
    def on_create(self, callback: Callable[[Any], None]) -> None:
        """Registrar hook de creación."""
        self.on_create_callbacks.append(callback)
    
    def on_dispose(self, callback: Callable[[Any], None]) -> None:
        """Registrar hook de disposal."""
        self.on_dispose_callbacks.append(callback)
    
    def notify_created(self, instance: Any) -> None:
        """Notificar que instancia fue creada."""
        for callback in self.on_create_callbacks:
            try:
                callback(instance)
            except Exception as e:
                logger.error(f"Error in create hook: {e}")
    
    def notify_disposed(self, instance: Any) -> None:
        """Notificar que instancia fue disposed."""
        for callback in self.on_dispose_callbacks:
            try:
                callback(instance)
            except Exception as e:
                logger.error(f"Error in dispose hook: {e}")


# ========================================
# Scope Context con Hierarchy
# ========================================

@dataclass
class ScopeContext:
    """
    Contexto de scope con soporte para hierarchy.
    
    Permite crear scopes anidados (parent/child) con disposal
    recursivo automático.
    
    Example:
        # Request principal
        main_context = ScopeContext()
        
        # Sub-request (transaction)
        transaction_context = main_context.create_child()
        
        # Disposal recursivo
        transaction_context.dispose_all()  # Solo transaction
        main_context.dispose_all()  # Main + remaining children
    """
    
    parent: Optional['ScopeContext'] = None
    children: List['ScopeContext'] = field(default_factory=list)
    disposables: List[OnDisposable] = field(default_factory=list)
    hooks: LifecycleHooks = field(default_factory=LifecycleHooks)
    _disposed: bool = field(default=False, init=False)
    
    def create_child(self) -> 'ScopeContext':
        """
        Crear scope hijo.
        
        El hijo hereda hooks del padre y se agregará automáticamente
        a la lista de children.
        
        Returns:
            ScopeContext hijo
        """
        child = ScopeContext(parent=self)
        # Heredar hooks del padre
        child.hooks = LifecycleHooks(
            on_create_callbacks=self.hooks.on_create_callbacks.copy(),
            on_dispose_callbacks=self.hooks.on_dispose_callbacks.copy()
        )
        self.children.append(child)
        return child
    
    def track_disposable(self, instance: OnDisposable) -> None:
        """
        Trackear instancia con OnDisposable para disposal automático.
        
        Args:
            instance: Instancia que implementa OnDisposable
        """
        if not isinstance(instance, OnDisposable):
            logger.warning(
                f"Instance {instance} does not implement OnDisposable, "
                f"will not be tracked for disposal"
            )
            return
        
        self.disposables.append(instance)
        self.hooks.notify_created(instance)
    
    def dispose_all(self) -> None:
        """
        Dispose recursivo de scope y sus children.
        
        Orden de disposal:
        1. Children (recursivamente, en orden inverso)
        2. Disposables propios (LIFO - last created, first disposed)
        
        Es idempotente (se puede llamar múltiples veces).
        """
        if self._disposed:
            return
        
        # 1. Dispose children primero (LIFO)
        for child in reversed(self.children):
            child.dispose_all()
        
        # 2. Dispose propios disposables (LIFO)
        for disposable in reversed(self.disposables):
            try:
                self.hooks.notify_disposed(disposable)
                disposable.dispose()
            except Exception as e:
                logger.error(
                    f"Error disposing {type(disposable).__name__}: {e}",
                    exc_info=True
                )
        
        # Cleanup
        self.disposables.clear()
        self.children.clear()
        self._disposed = True
    
    async def dispose_all_async(self) -> None:
        """
        Dispose asíncrono recursivo.
        
        Similar a dispose_all() pero soporta AsyncOnDisposable.
        """
        if self._disposed:
            return
        
        # 1. Dispose children primero (async)
        for child in reversed(self.children):
            await child.dispose_all_async()
        
        # 2. Dispose propios disposables (soporta async)
        for disposable in reversed(self.disposables):
            try:
                self.hooks.notify_disposed(disposable)
                
                if isinstance(disposable, AsyncOnDisposable):
                    await disposable.dispose_async()
                elif isinstance(disposable, OnDisposable):
                    disposable.dispose()
            except Exception as e:
                logger.error(
                    f"Error disposing {type(disposable).__name__}: {e}",
                    exc_info=True
                )
        
        # Cleanup
        self.disposables.clear()
        self.children.clear()
        self._disposed = True
    
    def is_disposed(self) -> bool:
        """Verifica si el scope ya fue disposed."""
        return self._disposed
    
    def get_depth(self) -> int:
        """Obtiene la profundidad del scope en la jerarquía."""
        depth = 0
        current = self.parent
        while current is not None:
            depth += 1
            current = current.parent
        return depth


# ========================================
# Scope Isolation para Testing
# ========================================

class IsolatedScope:
    """
    Context manager para scopes aislados en tests.
    
    Crea un scope temporal que se limpia automáticamente,
    sin afectar el registry global.
    
    Example:
        def test_user_service():
            with isolated_scope(injector) as context:
                # Registrar mock
                injector.register(UserRepository, use_value=MockRepo())
                
                # Test
                service = injector.get(UserService, context)
                assert service is not None
            
            # ← Cleanup automático: mock removido, scope disposed
    """
    
    def __init__(self, injector: 'Injector'):
        """
        Inicializar scope aislado.
        
        Args:
            injector: Injector a aislar
        """
        from .injector import Injector
        
        if not isinstance(injector, Injector):
            raise TypeError(f"Expected Injector, got {type(injector)}")
        
        self.injector = injector
        self.test_context: Optional[ScopeContext] = None
        self.original_registry_snapshot: Optional[dict] = None
    
    def __enter__(self) -> ScopeContext:
        """
        Crear scope aislado.
        
        Returns:
            ScopeContext para el test
        """
        # Snapshot del registry actual
        self.original_registry_snapshot = self.injector._registry.snapshot()
        
        # Crear contexto de test
        self.test_context = ScopeContext()
        
        return self.test_context
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Cleanup automático al salir del scope."""
        # Dispose del scope de test
        if self.test_context:
            self.test_context.dispose_all()
        
        # Restaurar registry original
        if self.original_registry_snapshot:
            self.injector._registry.restore(self.original_registry_snapshot)


def isolated_scope(injector: 'Injector') -> IsolatedScope:
    """
    Helper function para crear scope aislado.
    
    Args:
        injector: Injector a aislar
    
    Returns:
        IsolatedScope context manager
    
    Example:
        with isolated_scope(injector) as context:
            # Test code
            pass
    """
    return IsolatedScope(injector)


# ========================================
# Exports
# ========================================

__all__ = [
    # Protocols
    'OnDisposable',
    'AsyncOnDisposable',
    
    # Lifecycle
    'LifecycleHooks',
    'ScopeContext',
    
    # Testing
    'IsolatedScope',
    'isolated_scope',
]
