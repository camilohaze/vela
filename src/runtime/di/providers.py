"""
@provides Decorator - Factory Providers para Dependency Injection

Implementación de: TASK-035E
Historia: VELA-575
Fecha: 2025-12-01

Descripción:
Sistema de factory providers para DI con soporte para:
- Custom factories con lógica personalizada
- Async providers (await en factories)
- Scoped providers (SINGLETON, TRANSIENT, SCOPED)
- Auto-detección de dependencias
- Multi providers (arrays de valores)

Version: 0.1.0
"""

import asyncio
import inspect
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Type, Union, get_type_hints


# ===================================
# Enums
# ===================================

class ProviderScope(str, Enum):
    """
    Scopes para lifecycle management de providers.
    
    - SINGLETON: Una única instancia compartida en toda la aplicación
    - TRANSIENT: Nueva instancia cada vez que se inyecta
    - SCOPED: Una instancia por scope (ej: por HTTP request)
    """
    
    SINGLETON = "singleton"
    TRANSIENT = "transient"
    SCOPED = "scoped"
    
    def __str__(self) -> str:
        return self.value
    
    def __repr__(self) -> str:
        return f"ProviderScope.{self.name}"


# ===================================
# Dataclasses
# ===================================

@dataclass
class ProviderMetadata:
    """
    Metadata para factory providers en sistema DI.
    
    Un provider define CÓMO crear instancias de una dependencia.
    
    Attributes:
        token: Token de DI (Type o string) que identifica la dependencia
        factory: Función factory que crea la instancia
        scope: Lifecycle del provider (SINGLETON, TRANSIENT, SCOPED)
        is_async: Si factory es async (usa await)
        deps: Lista de dependencias que el factory necesita
        multi: Si provider puede retornar múltiples valores (array)
        description: Descripción del provider
    
    Example:
        metadata = ProviderMetadata(
            token=Database,
            factory=lambda config: Database(config.url),
            scope=ProviderScope.SINGLETON,
            deps=[Config]
        )
    """
    
    token: Union[Type, str]
    factory: Callable
    scope: ProviderScope = ProviderScope.SINGLETON
    is_async: bool = False
    deps: List[Union[Type, str]] = field(default_factory=list)
    multi: bool = False
    description: Optional[str] = None
    
    def __post_init__(self):
        """Validaciones post-init."""
        if not callable(self.factory):
            raise TypeError(
                f"Provider factory must be callable, got {type(self.factory).__name__}"
            )
        
        if not isinstance(self.scope, ProviderScope):
            raise TypeError(
                f"Provider scope must be ProviderScope, got {type(self.scope).__name__}"
            )
        
        # Normalizar token si es string
        if isinstance(self.token, str):
            self.token = self.token.strip()
            if not self.token:
                raise ValueError("Provider token cannot be empty string")
    
    def get_token_name(self) -> str:
        """Obtiene nombre del token (para display)."""
        if isinstance(self.token, str):
            return self.token
        elif isinstance(self.token, type):
            return self.token.__name__
        else:
            return str(self.token)
    
    def is_singleton(self) -> bool:
        """Verifica si provider es singleton."""
        return self.scope == ProviderScope.SINGLETON
    
    def is_transient(self) -> bool:
        """Verifica si provider es transient."""
        return self.scope == ProviderScope.TRANSIENT
    
    def is_scoped(self) -> bool:
        """Verifica si provider es scoped."""
        return self.scope == ProviderScope.SCOPED


# ===================================
# Decorator
# ===================================

def provides(
    token: Union[Type, str],
    *,
    scope: ProviderScope = ProviderScope.SINGLETON,
    deps: Optional[List[Union[Type, str]]] = None,
    multi: bool = False,
    description: Optional[str] = None
):
    """
    Marca método como factory provider para DI.
    
    Factory provider define CÓMO crear instancias de una dependencia con lógica
    personalizada. El injector llamará al factory cuando necesite inyectar la dependencia.
    
    Args:
        token: Token de DI (Type o string) que identifica la dependencia
        scope: Lifecycle del provider (default: SINGLETON)
            - SINGLETON: Una instancia compartida
            - TRANSIENT: Nueva instancia cada vez
            - SCOPED: Una instancia por scope (ej: HTTP request)
        deps: Lista de dependencias del factory (auto-detecta de type hints si no se provee)
        multi: Si provider puede tener múltiples valores (array)
        description: Descripción del provider
    
    Returns:
        Decorator function que agrega __provider_metadata__ al método
    
    Examples:
        # Singleton database connection
        @provides(Database, scope=ProviderScope.SINGLETON)
        def provide_database(config: Config) -> Database:
            return Database(config.db_url)
        
        # Async provider con deps explícitas
        @provides("API_KEY", scope=ProviderScope.SINGLETON, deps=[Config])
        async def provide_api_key(config: Config) -> str:
            return await fetch_api_key(config.api_endpoint)
        
        # Transient provider (nueva instancia cada vez)
        @provides(Logger, scope=ProviderScope.TRANSIENT)
        def provide_logger() -> Logger:
            return Logger()
        
        # Multi provider (retorna array)
        @provides(Plugin, multi=True)
        def provide_plugins() -> List[Plugin]:
            return [AuthPlugin(), LoggingPlugin()]
    
    Notes:
        - Factory puede ser sync o async (auto-detecta con inspect.iscoroutinefunction)
        - Dependencias se auto-detectan de type hints si no se proveen
        - Factory debe retornar instancia del tipo del token
        - Multi providers retornan List[T] en lugar de T
    """
    def decorator(func: Callable) -> Callable:
        # 1. Detectar si factory es async
        is_async = asyncio.iscoroutinefunction(func)
        
        # 2. Auto-detectar dependencias de type hints si no se proveen
        if deps is None:
            auto_deps = _extract_dependencies_from_signature(func)
        else:
            auto_deps = deps
        
        # 3. Crear metadata
        metadata = ProviderMetadata(
            token=token,
            factory=func,
            scope=scope,
            is_async=is_async,
            deps=auto_deps,
            multi=multi,
            description=description
        )
        
        # 4. Agregar metadata al método
        func.__provider_metadata__ = metadata
        
        return func
    
    return decorator


# ===================================
# Helper Functions
# ===================================

def _extract_dependencies_from_signature(func: Callable) -> List[Union[Type, str]]:
    """
    Extrae dependencias automáticamente de type hints de función.
    
    Args:
        func: Función a inspeccionar
    
    Returns:
        Lista de tipos encontrados en parámetros
    
    Example:
        def my_factory(config: Config, logger: Logger) -> Database:
            ...
        
        _extract_dependencies_from_signature(my_factory)
        # Result: [Config, Logger]
    """
    try:
        sig = inspect.signature(func)
        type_hints = get_type_hints(func)
        
        deps = []
        for param_name, param in sig.parameters.items():
            # Ignorar 'self' y 'cls'
            if param_name in ('self', 'cls'):
                continue
            
            # Obtener type hint
            if param_name in type_hints:
                param_type = type_hints[param_name]
                deps.append(param_type)
        
        return deps
    
    except Exception:
        # Si falla inspection, retornar lista vacía
        return []


def is_provider(func: Callable) -> bool:
    """
    Verifica si función es provider (tiene __provider_metadata__).
    
    Args:
        func: Función a verificar
    
    Returns:
        True si función tiene metadata de provider
    
    Example:
        @provides(Database)
        def provide_db():
            return Database()
        
        is_provider(provide_db)  # True
        is_provider(lambda: None)  # False
    """
    return hasattr(func, '__provider_metadata__')


def get_provider_metadata(func: Callable) -> Optional[ProviderMetadata]:
    """
    Obtiene metadata de provider si existe.
    
    Args:
        func: Función provider
    
    Returns:
        ProviderMetadata si existe, None si no
    
    Example:
        @provides(Database, scope=ProviderScope.SINGLETON)
        def provide_db():
            return Database()
        
        metadata = get_provider_metadata(provide_db)
        print(metadata.scope)  # ProviderScope.SINGLETON
    """
    return getattr(func, '__provider_metadata__', None)


def get_all_providers(cls: Type) -> Dict[str, ProviderMetadata]:
    """
    Obtiene todos los providers de una clase.
    
    Útil para extraer providers de un módulo o clase contenedora.
    
    Args:
        cls: Clase a inspeccionar
    
    Returns:
        Dict con nombre_método -> ProviderMetadata
    
    Example:
        class AppModule:
            @provides(Database)
            def provide_db(self):
                return Database()
            
            @provides(Logger)
            def provide_logger(self):
                return Logger()
        
        providers = get_all_providers(AppModule)
        # Result: {
        #   'provide_db': ProviderMetadata(...),
        #   'provide_logger': ProviderMetadata(...)
        # }
    """
    providers = {}
    
    for name, method in inspect.getmembers(cls, predicate=inspect.isfunction):
        metadata = get_provider_metadata(method)
        if metadata:
            providers[name] = metadata
    
    return providers


def get_providers_by_scope(cls: Type, scope: ProviderScope) -> Dict[str, ProviderMetadata]:
    """
    Filtra providers de una clase por scope.
    
    Args:
        cls: Clase a inspeccionar
        scope: Scope a filtrar (SINGLETON, TRANSIENT, SCOPED)
    
    Returns:
        Dict con providers que tienen el scope especificado
    
    Example:
        providers = get_providers_by_scope(AppModule, ProviderScope.SINGLETON)
        # Solo providers con scope SINGLETON
    """
    all_providers = get_all_providers(cls)
    return {
        name: metadata
        for name, metadata in all_providers.items()
        if metadata.scope == scope
    }


def get_provider_by_token(cls: Type, token: Union[Type, str]) -> Optional[tuple[str, ProviderMetadata]]:
    """
    Busca provider por token en una clase.
    
    Args:
        cls: Clase a inspeccionar
        token: Token de DI a buscar
    
    Returns:
        Tuple (nombre_método, metadata) si se encuentra, None si no
    
    Example:
        provider = get_provider_by_token(AppModule, Database)
        if provider:
            name, metadata = provider
            print(f"Found provider: {name}")
    """
    all_providers = get_all_providers(cls)
    
    for name, metadata in all_providers.items():
        if metadata.token == token:
            return (name, metadata)
    
    return None


# ===================================
# Inline Tests (if __name__ == "__main__")
# ===================================

if __name__ == "__main__":
    # Test ProviderScope
    assert str(ProviderScope.SINGLETON) == "singleton"
    assert ProviderScope.TRANSIENT.value == "transient"
    print("✓ ProviderScope tests passed")
    
    # Test ProviderMetadata
    def simple_factory():
        return "value"
    
    metadata = ProviderMetadata(
        token=str,
        factory=simple_factory,
        scope=ProviderScope.SINGLETON
    )
    assert metadata.token == str
    assert metadata.is_singleton()
    assert not metadata.is_transient()
    print("✓ ProviderMetadata tests passed")
    
    # Test @provides decorator
    @provides(Database, scope=ProviderScope.SINGLETON)
    class Database:
        pass
    
    @provides("CONFIG", description="App config")
    def provide_config():
        return {"key": "value"}
    
    assert is_provider(provide_config)
    config_meta = get_provider_metadata(provide_config)
    assert config_meta.token == "CONFIG"
    assert config_meta.scope == ProviderScope.SINGLETON
    print("✓ @provides decorator tests passed")
    
    # Test auto-detection de deps
    @provides(str)
    def provide_with_deps(dep1: int, dep2: float) -> str:
        return "result"
    
    deps_meta = get_provider_metadata(provide_with_deps)
    assert int in deps_meta.deps
    assert float in deps_meta.deps
    print("✓ Auto-detection de deps tests passed")
    
    print("\n✅ All inline tests passed!")
