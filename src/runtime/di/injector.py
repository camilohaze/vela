"""
Injector Core - Sistema de Dependency Injection

Implementación de: TASK-035F
Historia: VELA-575
Fecha: 2025-12-01

Descripción:
Contenedor DI core con resolución automática de dependencias.
Soporta Singleton, Transient, Scoped scopes, detección de ciclos,
y resolución recursiva de dependencias.

Referencias:
- Spring Framework: IoC Container
- Angular: Injector API
- NestJS: DI Container
- InversifyJS: Container API
"""

from __future__ import annotations
from dataclasses import dataclass, field
from typing import Type, TypeVar, Optional, Any, Dict, List, Callable, Set
from enum import Enum
import inspect
import asyncio

# Import de módulos DI existentes
from .scopes import Scope
from .injectable import is_injectable, get_injectable_metadata
from .inject import inject as inject_decorator, get_inject_metadata
from .module import is_module, get_module_metadata
from .providers import is_provider, get_provider_metadata
from .lifecycle import (
    OnDisposable,
    AsyncOnDisposable,
    ScopeContext,
    LifecycleHooks
)


T = TypeVar('T')


# ========================================
# Exceptions
# ========================================

class InjectionError(Exception):
    """Error base para errores de inyección de dependencias."""
    pass


class CircularDependencyError(InjectionError):
    """Error cuando se detecta una dependencia circular."""
    
    def __init__(self, dependency_chain: List[Type]):
        self.dependency_chain = dependency_chain
        # Manejar tanto clases como strings (forward references)
        chain_str = " -> ".join(
            cls.__name__ if isinstance(cls, type) else str(cls)
            for cls in dependency_chain
        )
        super().__init__(f"Circular dependency detected: {chain_str}")


class ProviderNotFoundError(InjectionError):
    """Error cuando no se encuentra un provider para un token."""
    
    def __init__(self, token: Type):
        self.token = token
        # Manejar tanto clases como strings (forward references)
        token_name = token.__name__ if isinstance(token, type) else str(token)
        super().__init__(f"No provider found for: {token_name}")


class InvalidScopeError(InjectionError):
    """Error cuando se usa un scope inválido."""
    
    def __init__(self, scope: str):
        self.scope = scope
        super().__init__(f"Invalid scope: {scope}")


# ========================================
# Resolution Context
# ========================================

@dataclass
class ResolutionContext:
    """
    Contexto de resolución para tracking de dependencias.
    
    Mantiene el estado durante la resolución recursiva:
    - Stack de dependencias (para detectar ciclos)
    - Scope actual (Singleton, Transient, Scoped)
    - Cache de instancias por scope
    """
    
    # Stack de tokens siendo resueltos (para detectar ciclos)
    resolution_stack: List[Type] = field(default_factory=list)
    
    # Scope actual
    current_scope: Scope = Scope.SINGLETON
    
    # Cache de instancias: {scope -> {token -> instance}}
    cache: Dict[Scope, Dict[Type, Any]] = field(default_factory=dict)
    
    # Metadata del scope actual (ej: request ID para Scoped)
    scope_metadata: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Inicializar caches para cada scope."""
        if not self.cache:
            self.cache = {
                Scope.SINGLETON: {},
                Scope.TRANSIENT: {},  # Nunca cachea
                Scope.SCOPED: {}
            }
    
    def push_resolution(self, token: Type) -> None:
        """
        Agregar token al stack de resolución.
        
        Raises:
            CircularDependencyError: Si el token ya está en el stack.
        """
        if token in self.resolution_stack:
            # Ciclo detectado
            cycle = self.resolution_stack + [token]
            raise CircularDependencyError(cycle)
        
        self.resolution_stack.append(token)
    
    def pop_resolution(self) -> Type:
        """Remover último token del stack de resolución."""
        if not self.resolution_stack:
            raise InjectionError("Resolution stack is empty")
        return self.resolution_stack.pop()
    
    def get_cached(self, token: Type, scope: Scope) -> Optional[Any]:
        """Obtener instancia cacheada si existe."""
        if scope == Scope.TRANSIENT:
            return None  # Transient nunca cachea
        
        return self.cache.get(scope, {}).get(token)
    
    def set_cached(self, token: Type, scope: Scope, instance: Any) -> None:
        """Cachear instancia según scope."""
        if scope == Scope.TRANSIENT:
            return  # Transient nunca cachea
        
        if scope not in self.cache:
            self.cache[scope] = {}
        
        self.cache[scope][token] = instance
    
    def clear_scope(self, scope: Scope) -> None:
        """Limpiar cache de un scope específico."""
        if scope in self.cache:
            self.cache[scope].clear()
    
    def create_child_scope(self, scope: Scope, metadata: Optional[Dict[str, Any]] = None) -> ResolutionContext:
        """
        Crear un contexto hijo con nuevo scope.
        
        Usado para Scoped (ej: por request HTTP).
        """
        return ResolutionContext(
            resolution_stack=[],
            current_scope=scope,
            cache={
                Scope.SINGLETON: self.cache[Scope.SINGLETON],  # Compartir singletons
                Scope.TRANSIENT: {},
                Scope.SCOPED: {}  # Nuevo cache para scope
            },
            scope_metadata=metadata or {}
        )


# ========================================
# Provider Registry
# ========================================

@dataclass
class ProviderEntry:
    """
    Entrada en el registry de providers.
    
    Mantiene información sobre cómo crear instancias de un token.
    """
    
    # Token del provider (usualmente la clase)
    token: Type
    
    # Scope del provider
    scope: Scope
    
    # Tipo de provider: "class", "factory", "value", "async_factory"
    provider_type: str
    
    # Factory function (si es factory provider)
    factory: Optional[Callable] = None
    
    # Valor directo (si es value provider)
    value: Optional[Any] = None
    
    # Dependencias del constructor
    dependencies: List[Type] = field(default_factory=list)
    
    # Multi provider (múltiples valores para un token)
    multi: bool = False
    
    # Provider async
    is_async: bool = False


class ProviderRegistry:
    """
    Registry global de providers.
    
    Mantiene todos los providers registrados en el sistema DI.
    """
    
    def __init__(self):
        self._providers: Dict[Type, ProviderEntry] = {}
        self._multi_providers: Dict[Type, List[ProviderEntry]] = {}
    
    def register(self, entry: ProviderEntry) -> None:
        """Registrar un provider."""
        if entry.multi:
            # Multi provider: agregar a lista
            if entry.token not in self._multi_providers:
                self._multi_providers[entry.token] = []
            self._multi_providers[entry.token].append(entry)
        else:
            # Provider normal: sobrescribir existente
            self._providers[entry.token] = entry
    
    def get(self, token: Type) -> Optional[ProviderEntry]:
        """Obtener provider por token."""
        return self._providers.get(token)
    
    def get_multi(self, token: Type) -> List[ProviderEntry]:
        """Obtener todos los multi providers para un token."""
        return self._multi_providers.get(token, [])
    
    def has(self, token: Type) -> bool:
        """Verificar si existe provider para token."""
        return token in self._providers or token in self._multi_providers
    
    def clear(self) -> None:
        """Limpiar registry."""
        self._providers.clear()
        self._multi_providers.clear()


# ========================================
# Injector Core
# ========================================

class Injector:
    """
    Contenedor DI core.
    
    Responsabilidades:
    - Registrar providers (clases @injectable, @provides factories)
    - Resolver dependencias recursivamente
    - Gestionar scopes (Singleton, Transient, Scoped)
    - Detectar dependencias circulares
    - Cachear instancias según scope
    
    Ejemplo:
        injector = Injector()
        injector.register(UserService)
        injector.register(UserRepository)
        
        # Resolver con dependencias
        service = injector.get(UserService)
    """
    
    def __init__(self):
        self._registry = ProviderRegistry()
        self._context = ResolutionContext()
        
        # Lifecycle Management (TASK-035G)
        # Disposal tracking por scope
        self._disposable_instances: Dict[Scope, List[OnDisposable]] = {
            Scope.SINGLETON: [],
            Scope.SCOPED: [],  # Por contexto en ResolutionContext
            Scope.TRANSIENT: []  # NO trackear (responsabilidad del usuario)
        }
        
        # Lifecycle hooks globales
        self._lifecycle_hooks: LifecycleHooks = LifecycleHooks()
    
    # ========================================
    # Registration Methods
    # ========================================
    
    def register(
        self,
        token: Type[T],
        provider: Optional[Any] = None,
        scope: Optional[Scope] = None,
        multi: bool = False
    ) -> None:
        """
        Registrar un provider.
        
        Args:
            token: Token para identificar el provider (clase).
            provider: Provider (clase, factory, valor). Si None, usa token como clase.
            scope: Scope del provider. Si None, usa scope de metadata o Singleton.
            multi: Si True, permite múltiples providers para el mismo token.
        
        Ejemplos:
            # Clase @injectable
            injector.register(UserService)
            
            # Factory function
            injector.register(Database, factory=create_database, scope=Scope.SINGLETON)
            
            # Valor directo
            injector.register(Config, value={"host": "localhost"})
            
            # Multi provider
            injector.register(HttpMiddleware, provider=LoggingMiddleware, multi=True)
            injector.register(HttpMiddleware, provider=AuthMiddleware, multi=True)
        """
        # Si no se proporciona provider, usa token como clase
        if provider is None:
            provider = token
        
        # Determinar tipo de provider
        if callable(provider) and not isinstance(provider, type):
            # Factory function
            provider_type = "async_factory" if asyncio.iscoroutinefunction(provider) else "factory"
            factory = provider
            value_obj = None
            dependencies = self._extract_factory_dependencies(provider)
        elif isinstance(provider, type):
            # Clase @injectable
            provider_type = "class"
            factory = None
            value_obj = None
            dependencies = self._extract_constructor_dependencies(provider)
        else:
            # Valor directo
            provider_type = "value"
            factory = None
            value_obj = provider
            dependencies = []
        
        # Determinar scope
        if scope is None:
            if is_injectable(provider if isinstance(provider, type) else token):
                metadata = get_injectable_metadata(provider if isinstance(provider, type) else token)
                scope = metadata.scope if metadata else Scope.SINGLETON
            elif is_provider(provider if callable(provider) else token):
                metadata = get_provider_metadata(provider if callable(provider) else token)
                scope = metadata.scope if metadata else Scope.SINGLETON
            else:
                scope = Scope.SINGLETON
        
        # Crear entrada en registry
        entry = ProviderEntry(
            token=token,
            scope=scope,
            provider_type=provider_type,
            factory=factory,
            value=value_obj,
            dependencies=dependencies,
            multi=multi,
            is_async=(provider_type == "async_factory")
        )
        
        self._registry.register(entry)
    
    def register_module(self, module_cls: Type) -> None:
        """
        Registrar todos los providers de un módulo.
        
        Args:
            module_cls: Clase decorada con @module.
        
        Ejemplo:
            @module({
                "providers": [UserService, UserRepository],
                "exports": [UserService]
            })
            class UserModule:
                pass
            
            injector.register_module(UserModule)
        """
        if not is_module(module_cls):
            raise InjectionError(f"{module_cls.__name__} is not a @module")
        
        metadata = get_module_metadata(module_cls)
        if not metadata:
            return
        
        # Registrar todos los providers del módulo
        for provider in metadata.providers:
            self.register(provider)
        
        # Registrar módulos importados
        for imported_module in metadata.imports:
            if is_module(imported_module):
                self.register_module(imported_module)
    
    # ========================================
    # Resolution Methods
    # ========================================
    
    def get(self, token: Type[T], context: Optional[ResolutionContext] = None) -> T:
        """
        Resolver y obtener instancia de un token.
        
        Args:
            token: Token a resolver (clase).
            context: Contexto de resolución. Si None, usa contexto global.
        
        Returns:
            Instancia del token con dependencias resueltas.
        
        Raises:
            ProviderNotFoundError: Si no hay provider para el token.
            CircularDependencyError: Si hay dependencia circular.
        
        Ejemplo:
            service = injector.get(UserService)
        """
        ctx = context or self._context
        
        # Verificar si hay provider
        if not self._registry.has(token):
            raise ProviderNotFoundError(token)
        
        # Resolver
        return self._resolve(token, ctx)
    
    async def get_async(self, token: Type[T], context: Optional[ResolutionContext] = None) -> T:
        """
        Resolver y obtener instancia de un token (async).
        
        Similar a get() pero soporta providers async.
        """
        ctx = context or self._context
        
        if not self._registry.has(token):
            raise ProviderNotFoundError(token)
        
        return await self._resolve_async(token, ctx)
    
    def get_all(self, token: Type[T], context: Optional[ResolutionContext] = None) -> List[T]:
        """
        Obtener todas las instancias de un multi provider.
        
        Ejemplo:
            middlewares = injector.get_all(HttpMiddleware)
        """
        ctx = context or self._context
        
        multi_providers = self._registry.get_multi(token)
        if not multi_providers:
            return []
        
        instances = []
        for entry in multi_providers:
            instance = self._resolve_entry(entry, ctx)
            instances.append(instance)
        
        return instances
    
    def _resolve(self, token: Type[T], context: ResolutionContext) -> T:
        """Resolver token (implementación interna)."""
        entry = self._registry.get(token)
        if not entry:
            raise ProviderNotFoundError(token)
        
        return self._resolve_entry(entry, context)
    
    async def _resolve_async(self, token: Type[T], context: ResolutionContext) -> T:
        """Resolver token async (implementación interna)."""
        entry = self._registry.get(token)
        if not entry:
            raise ProviderNotFoundError(token)
        
        return await self._resolve_entry_async(entry, context)
    
    def _resolve_entry(self, entry: ProviderEntry, context: ResolutionContext) -> Any:
        """
        Resolver un provider entry.
        
        Algoritmo:
        1. Verificar cache (Singleton/Scoped)
        2. Detectar ciclos (push al stack)
        3. Resolver dependencias recursivamente
        4. Crear instancia
        5. Cachear según scope
        6. Pop del stack
        """
        # 1. Verificar cache
        cached = context.get_cached(entry.token, entry.scope)
        if cached is not None:
            return cached
        
        # 2. Detectar ciclos
        context.push_resolution(entry.token)
        
        try:
            # 3. Crear instancia según tipo de provider
            if entry.provider_type == "value":
                instance = entry.value
            
            elif entry.provider_type == "factory":
                # Resolver dependencias de factory
                deps = [self._resolve(dep, context) for dep in entry.dependencies]
                instance = entry.factory(*deps)
            
            elif entry.provider_type == "class":
                # Resolver dependencias del constructor
                deps = [self._resolve(dep, context) for dep in entry.dependencies]
                instance = entry.token(*deps)
            
            else:
                raise InjectionError(f"Unknown provider type: {entry.provider_type}")
            
            # 4. Track OnDisposable (TASK-035G)
            if isinstance(instance, OnDisposable):
                self._track_disposable(instance, entry.scope, context)
            
            # 5. Notify lifecycle hooks (TASK-035G)
            self._lifecycle_hooks.notify_created(instance)
            
            # 6. Cachear según scope
            context.set_cached(entry.token, entry.scope, instance)
            
            return instance
        
        finally:
            # 5. Pop del stack
            context.pop_resolution()
    
    async def _resolve_entry_async(self, entry: ProviderEntry, context: ResolutionContext) -> Any:
        """Resolver un provider entry (async)."""
        # Verificar cache
        cached = context.get_cached(entry.token, entry.scope)
        if cached is not None:
            return cached
        
        # Detectar ciclos
        context.push_resolution(entry.token)
        
        try:
            if entry.provider_type == "value":
                instance = entry.value
            
            elif entry.provider_type == "async_factory":
                # Resolver dependencias
                deps = []
                for dep in entry.dependencies:
                    dep_entry = self._registry.get(dep)
                    if dep_entry and dep_entry.is_async:
                        deps.append(await self._resolve_async(dep, context))
                    else:
                        deps.append(self._resolve(dep, context))
                
                # Ejecutar factory async
                instance = await entry.factory(*deps)
            
            elif entry.provider_type == "factory":
                deps = [self._resolve(dep, context) for dep in entry.dependencies]
                instance = entry.factory(*deps)
            
            elif entry.provider_type == "class":
                deps = [self._resolve(dep, context) for dep in entry.dependencies]
                instance = entry.token(*deps)
            
            else:
                raise InjectionError(f"Unknown provider type: {entry.provider_type}")
            
            # Track OnDisposable (TASK-035G)
            if isinstance(instance, OnDisposable):
                self._track_disposable(instance, entry.scope, context)
            
            # Notify lifecycle hooks (TASK-035G)
            self._lifecycle_hooks.notify_created(instance)
            
            # Cachear
            context.set_cached(entry.token, entry.scope, instance)
            
            return instance
        
        finally:
            context.pop_resolution()
    
    # ========================================
    # Scope Management
    # ========================================
    
    def create_scope(self, scope: Scope = Scope.SCOPED, metadata: Optional[Dict[str, Any]] = None) -> ResolutionContext:
        """
        Crear un nuevo contexto con scope específico.
        
        Usado para Scoped (ej: por request HTTP).
        
        Ejemplo:
            # En un request HTTP
            request_context = injector.create_scope(Scope.SCOPED, {"request_id": "123"})
            session = injector.get(UserSession, request_context)
        """
        return self._context.create_child_scope(scope, metadata)
    
    def clear_scope(self, scope: Scope) -> None:
        """Limpiar cache de un scope."""
        self._context.clear_scope(scope)
    
    def dispose_scope(self, scope: Scope, context: Optional[ResolutionContext] = None) -> None:
        """
        Disponer todas las instancias de un scope.
        
        Disposal order: LIFO (Last In, First Out).
        Instancias se disponen en orden inverso a su creación.
        
        Args:
            scope: Scope a disponer (SINGLETON, SCOPED, TRANSIENT).
            context: Contexto de resolución (requerido para SCOPED).
        
        Raises:
            ValueError: Si scope es SCOPED y no se proporciona context.
        
        Ejemplo:
            # Disponer todos los singletons
            injector.dispose_scope(Scope.SINGLETON)
            
            # Disponer scope de request
            request_context = injector.create_scope(Scope.SCOPED)
            # ... usar servicios ...
            injector.dispose_scope(Scope.SCOPED, request_context)
        """
        import logging
        logger = logging.getLogger(__name__)
        
        if scope == Scope.SINGLETON:
            # Disponer singletons en orden LIFO
            for instance in reversed(self._disposable_instances[Scope.SINGLETON]):
                try:
                    self._lifecycle_hooks.notify_disposed(instance)
                    instance.dispose()
                except Exception as e:
                    logger.error(f"Error disposing {instance}: {e}")
            
            self._disposable_instances[Scope.SINGLETON].clear()
        
        elif scope == Scope.SCOPED:
            if context is None:
                raise ValueError("Context required for scoped disposal")
            
            # Disponer scope context (recursivo si tiene children)
            if hasattr(context, 'scope_context'):
                context.scope_context.dispose_all()
        
        elif scope == Scope.TRANSIENT:
            # Transient no se trackea (responsabilidad del usuario)
            pass
        
        # Limpiar cache del scope
        self.clear_scope(scope)
    
    async def dispose_scope_async(
        self,
        scope: Scope,
        context: Optional[ResolutionContext] = None
    ) -> None:
        """
        Disponer todas las instancias de un scope (async).
        
        Similar a dispose_scope() pero soporta AsyncOnDisposable.
        
        Ejemplo:
            # Disponer singletons async (ej: RedisClient)
            await injector.dispose_scope_async(Scope.SINGLETON)
        """
        import logging
        logger = logging.getLogger(__name__)
        
        if scope == Scope.SINGLETON:
            for instance in reversed(self._disposable_instances[Scope.SINGLETON]):
                try:
                    self._lifecycle_hooks.notify_disposed(instance)
                    
                    # Intentar dispose async primero
                    if isinstance(instance, AsyncOnDisposable):
                        await instance.dispose_async()
                    elif isinstance(instance, OnDisposable):
                        instance.dispose()
                except Exception as e:
                    logger.error(f"Error disposing {instance}: {e}")
            
            self._disposable_instances[Scope.SINGLETON].clear()
        
        elif scope == Scope.SCOPED:
            if context is None:
                raise ValueError("Context required for scoped disposal")
            
            if hasattr(context, 'scope_context'):
                await context.scope_context.dispose_all_async()
        
        self.clear_scope(scope)
    
    def dispose(self) -> None:
        """
        Limpiar todo el injector.
        
        Dispone todos los scopes y limpia el registry.
        """
        # Disponer todos los singletons
        self.dispose_scope(Scope.SINGLETON)
        
        # Limpiar registry y cache
        self._registry.clear()
        self._context.cache.clear()
    
    async def dispose_async(self) -> None:
        """
        Limpiar todo el injector (async).
        
        Similar a dispose() pero soporta AsyncOnDisposable.
        """
        # Disponer todos los singletons async
        await self.dispose_scope_async(Scope.SINGLETON)
        
        # Limpiar registry y cache
        self._registry.clear()
        self._context.cache.clear()
    
    def create_child_scope(
        self,
        parent: Optional[ScopeContext] = None
    ) -> ScopeContext:
        """
        Crear scope hijo para hierarchy.
        
        Child scopes heredan lifecycle hooks del padre.
        
        Args:
            parent: Scope padre. Si None, crea scope raíz.
        
        Returns:
            Nuevo ScopeContext hijo.
        
        Ejemplo:
            # Main request scope
            main_scope = injector.create_child_scope()
            
            # Sub-request (transaction)
            transaction_scope = injector.create_child_scope(main_scope)
            
            # ... usar servicios ...
            
            # Disponer transaction primero
            transaction_scope.dispose_all()
            
            # Luego main scope
            main_scope.dispose_all()
        """
        if parent is None:
            # Crear scope raíz
            return ScopeContext()
        else:
            # Crear child scope
            return parent.create_child()
    
    def _track_disposable(
        self,
        instance: OnDisposable,
        scope: Scope,
        context: ResolutionContext
    ) -> None:
        """
        Trackear instancia OnDisposable para disposal automático.
        
        Args:
            instance: Instancia que implementa OnDisposable.
            scope: Scope de la instancia.
            context: Contexto de resolución.
        """
        if scope == Scope.SINGLETON:
            # Trackear en lista global de singletons
            self._disposable_instances[Scope.SINGLETON].append(instance)
        
        elif scope == Scope.SCOPED:
            # Trackear en scope context
            if not hasattr(context, 'scope_context'):
                context.scope_context = ScopeContext()
            
            context.scope_context.track_disposable(instance)
        
        # Transient: NO trackear (responsabilidad del usuario)
    
    # ========================================
    # Helper Methods
    # ========================================
    
    def _extract_constructor_dependencies(self, cls: Type) -> List[Type]:
        """
        Extraer dependencias del constructor de una clase.
        
        Lee metadata de @injectable para obtener dependencias.
        """
        from .injectable import get_injectable_metadata
        
        # Obtener metadata de @injectable
        metadata = get_injectable_metadata(cls)
        if metadata and metadata.dependencies:
            return metadata.dependencies
        
        return []
    
    def _extract_factory_dependencies(self, factory: Callable) -> List[Type]:
        """
        Extraer dependencias de una factory function.
        
        Lee type hints de los parámetros.
        """
        dependencies = []
        
        sig = inspect.signature(factory)
        
        for param in sig.parameters.values():
            if param.annotation != inspect.Parameter.empty:
                dependencies.append(param.annotation)
        
        return dependencies
    
    def has_provider(self, token: Type) -> bool:
        """Verificar si existe provider para un token."""
        return self._registry.has(token)
    
    def get_provider_scope(self, token: Type) -> Optional[Scope]:
        """Obtener scope de un provider."""
        entry = self._registry.get(token)
        return entry.scope if entry else None


# ========================================
# Container (Facade)
# ========================================

class Container:
    """
    Facade del Injector con API simplificada.
    
    Wrapper sobre Injector para API más limpia.
    """
    
    def __init__(self, injector: Optional[Injector] = None):
        self._injector = injector or Injector()
    
    def provide(self, token: Type[T], provider: Optional[Any] = None, scope: Optional[Scope] = None) -> None:
        """Registrar provider."""
        self._injector.register(token, provider, scope)
    
    def provide_module(self, module_cls: Type) -> None:
        """Registrar módulo."""
        self._injector.register_module(module_cls)
    
    def resolve(self, token: Type[T]) -> T:
        """Resolver token."""
        return self._injector.get(token)
    
    async def resolve_async(self, token: Type[T]) -> T:
        """Resolver token (async)."""
        return await self._injector.get_async(token)
    
    def resolve_all(self, token: Type[T]) -> List[T]:
        """Resolver todos los multi providers."""
        return self._injector.get_all(token)
    
    def create_scope(self, metadata: Optional[Dict[str, Any]] = None) -> ResolutionContext:
        """Crear scope."""
        return self._injector.create_scope(Scope.SCOPED, metadata)
    
    def clear(self) -> None:
        """Limpiar container."""
        self._injector.dispose()


# ========================================
# Global Injector Instance
# ========================================

_global_injector: Optional[Injector] = None


def get_global_injector() -> Injector:
    """Obtener injector global singleton."""
    global _global_injector
    if _global_injector is None:
        _global_injector = Injector()
    return _global_injector


def create_injector() -> Injector:
    """Crear un nuevo injector."""
    return Injector()


def create_container() -> Container:
    """Crear un nuevo container."""
    return Container()


if __name__ == "__main__":
    # Ejemplo de uso
    from .injectable import injectable
    from .inject import inject
    
    @injectable(scope=Scope.SINGLETON)
    class Database:
        def __init__(self):
            print("Database created")
    
    @injectable
    class UserRepository:
        def __init__(self, db: Database = inject(Database)):
            self.db = db
            print("UserRepository created with db:", db)
    
    @injectable
    class UserService:
        def __init__(self, repo: UserRepository = inject(UserRepository)):
            self.repo = repo
            print("UserService created with repo:", repo)
    
    # Crear injector
    injector = create_injector()
    
    # Registrar providers
    injector.register(Database)
    injector.register(UserRepository)
    injector.register(UserService)
    
    # Resolver
    service = injector.get(UserService)
    print(f"Service resolved: {service}")
    
    # Verificar singleton: segunda resolución debe devolver misma instancia
    service2 = injector.get(UserService)
    print(f"Same instance: {service is service2}")  # True
