"""
Decorador @injectable para Dependency Injection

Implementación de: TASK-035B
Historia: VELA-575 - Sistema de Dependency Injection
Sprint: 13
Fecha: 2025-12-01

Descripción:
Implementa el decorador @injectable que marca clases/keywords arquitectónicos
como inyectables en el sistema DI con soporte para scopes.
"""

from typing import Type, TypeVar, Optional, Any, Dict, Callable
from dataclasses import dataclass, field
from .scopes import Scope, DEFAULT_SCOPE


T = TypeVar('T')


@dataclass
class InjectableMetadata:
    """
    Metadata almacenada en clases decoradas con @injectable.
    
    Attributes:
        scope: Scope de lifecycle (Singleton, Transient, Scoped)
        token: Token opcional para registro en DI container
        factory: Factory function opcional para crear instancias custom
        dependencies: Lista de dependencias del constructor (inferida)
    """
    scope: Scope = DEFAULT_SCOPE
    token: Optional[str] = None
    factory: Optional[Callable[..., Any]] = None
    dependencies: list[Type] = field(default_factory=list)
    
    def __repr__(self) -> str:
        return f"InjectableMetadata(scope={self.scope}, token={self.token})"


def injectable(
    scope: Scope = DEFAULT_SCOPE,
    token: Optional[str] = None,
    factory: Optional[Callable[..., Any]] = None
) -> Callable[[Type[T]], Type[T]]:
    """
    Decorador que marca una clase/keyword como inyectable en el sistema DI.
    
    Args:
        scope: Scope de lifecycle (Singleton, Transient, Scoped).
               Default: Scope.SINGLETON
        token: Token opcional para registro. Si no se provee, usa el nombre de clase.
        factory: Factory function opcional para crear instancias custom.
    
    Returns:
        Decorator function que agrega metadata a la clase
    
    Raises:
        TypeError: Si se intenta decorar algo que no es una clase
    
    Examples:
        >>> @injectable(scope=Scope.SINGLETON)
        >>> class UserService:
        >>>     def __init__(self, repository: UserRepository):
        >>>         self.repository = repository
        
        >>> @injectable  # Default: Singleton
        >>> class Logger:
        >>>     pass
        
        >>> @injectable(scope=Scope.TRANSIENT)
        >>> class EmailMessage:
        >>>     pass
        
        >>> @injectable(scope=Scope.SCOPED, token="custom_token")
        >>> class UserSession:
        >>>     pass
    
    Usage con keywords arquitectónicos (RECOMENDADO):
        >>> @injectable(scope=Scope.SINGLETON)
        >>> service UserService:
        >>>     repository: UserRepository
        >>>     
        >>>     constructor(@inject repository: UserRepository) {
        >>>         this.repository = repository
        >>>     }
        
        >>> @injectable
        >>> repository UserRepository:
        >>>     db: DatabaseConnection
        >>>     
        >>>     constructor(@inject db: DatabaseConnection) {
        >>>         this.db = db
        >>>     }
        
        >>> @injectable
        >>> guard AuthGuard:
        >>>     fn canActivate(req: Request) -> Bool {
        >>>         # Implementación
        >>>     }
        
        >>> @injectable
        >>> middleware LoggerMiddleware:
        >>>     async fn apply(req, res, next) -> Promise<void> {
        >>>         # Implementación
        >>>     }
    
    ⚠️ IMPORTANTE:
        - Controllers NO usan @injectable (se registran en controllers: [])
        - Services, repositories, guards, middleware, pipes SÍ usan @injectable
        - Keywords arquitectónicos son el uso RECOMENDADO (no class genérica)
    """
    
    def decorator(cls: Type[T]) -> Type[T]:
        # Validar que sea una clase
        if not isinstance(cls, type):
            raise TypeError(
                f"@injectable solo puede decorar clases. "
                f"Recibido: {type(cls).__name__}"
            )
        
        # Crear metadata
        metadata = InjectableMetadata(
            scope=scope,
            token=token or cls.__name__,
            factory=factory,
            dependencies=[]  # Se inferirá después desde __init__
        )
        
        # Almacenar metadata en la clase
        setattr(cls, '__injectable_metadata__', metadata)
        
        # Marcar clase como injectable
        setattr(cls, '__injectable__', True)
        
        # Auto-registrar si tiene token
        if token:
            _provider_registry[token] = cls
        
        return cls
    
    return decorator


def is_injectable(cls: Type) -> bool:
    """
    Verifica si una clase está decorada con @injectable.
    
    Args:
        cls: Clase a verificar
    
    Returns:
        True si tiene decorador @injectable, False caso contrario
    
    Examples:
        >>> @injectable
        >>> class MyService:
        >>>     pass
        >>> 
        >>> is_injectable(MyService)  # True
        >>> is_injectable(str)        # False
    """
    return hasattr(cls, '__injectable__') and getattr(cls, '__injectable__') is True


def get_injectable_metadata(cls: Type) -> Optional[InjectableMetadata]:
    """
    Obtiene la metadata de @injectable de una clase.
    
    Args:
        cls: Clase decorada con @injectable
    
    Returns:
        InjectableMetadata si la clase es injectable, None caso contrario
    
    Examples:
        >>> @injectable(scope=Scope.SINGLETON)
        >>> class MyService:
        >>>     pass
        >>> 
        >>> metadata = get_injectable_metadata(MyService)
        >>> print(metadata.scope)  # Scope.SINGLETON
    """
    if not is_injectable(cls):
        return None
    
    return getattr(cls, '__injectable_metadata__', None)


def get_scope(cls: Type) -> Optional[Scope]:
    """
    Obtiene el scope de una clase injectable.
    
    Args:
        cls: Clase decorada con @injectable
    
    Returns:
        Scope de la clase o None si no es injectable
    
    Examples:
        >>> @injectable(scope=Scope.TRANSIENT)
        >>> class MyService:
        >>>     pass
        >>> 
        >>> get_scope(MyService)  # Scope.TRANSIENT
    """
    metadata = get_injectable_metadata(cls)
    if metadata:
        return metadata.scope
    return None


def get_token(cls: Type) -> Optional[str]:
    """
    Obtiene el token de registro de una clase injectable.
    
    Args:
        cls: Clase decorada con @injectable
    
    Returns:
        Token string o None si no es injectable
    
    Examples:
        >>> @injectable(token="my_custom_token")
        >>> class MyService:
        >>>     pass
        >>> 
        >>> get_token(MyService)  # "my_custom_token"
    """
    metadata = get_injectable_metadata(cls)
    if metadata:
        return metadata.token
    return None


# Registro global de providers (se usará en el Injector)
_provider_registry: Dict[str, Type] = {}


def register_provider(cls: Type, token: Optional[str] = None) -> None:
    """
    Registra un provider en el registro global.
    
    Args:
        cls: Clase a registrar (puede o no tener @injectable)
        token: Token custom (opcional, default: token de metadata o nombre de clase)
    
    Notes:
        - Permite registrar clases sin @injectable (para casos especiales)
        - Si la clase tiene @injectable, usa su token de metadata
        - Permite sobrescribir registros existentes
    
    Examples:
        >>> @injectable
        >>> class MyService:
        >>>     pass
        >>> 
        >>> register_provider(MyService)
        >>> register_provider(MyService, token="custom-token")
        >>> 
        >>> # También funciona con clases no decoradas
        >>> class PlainClass:
        >>>     pass
        >>> register_provider(PlainClass, token="plain-token")
    """
    # Usar token proporcionado, o token de metadata (si existe), o nombre de clase
    if token is None:
        if is_injectable(cls):
            metadata_token = get_token(cls)
            token = metadata_token if metadata_token else cls.__name__
        else:
            token = cls.__name__
    
    # Permitir sobrescribir (para testing o redefiniciones)
    _provider_registry[token] = cls


def get_provider(token: str) -> Optional[Type]:
    """
    Obtiene un provider del registro por token.
    
    Args:
        token: Token del provider
    
    Returns:
        Clase provider o None si no existe
    
    Examples:
        >>> @injectable
        >>> class MyService:
        >>>     pass
        >>> 
        >>> register_provider(MyService)
        >>> provider = get_provider("MyService")
        >>> print(provider.__name__)  # "MyService"
    """
    return _provider_registry.get(token)


def clear_registry() -> None:
    """
    Limpia el registro global de providers.
    
    Útil para testing.
    
    Examples:
        >>> clear_registry()
        >>> len(_provider_registry)  # 0
    """
    _provider_registry.clear()


if __name__ == "__main__":
    # Tests básicos
    print("=== Tests de @injectable ===")
    
    # Test 1: Decorador básico
    @injectable
    class TestService:
        pass
    
    print(f"TestService is injectable: {is_injectable(TestService)}")
    metadata = get_injectable_metadata(TestService)
    print(f"Metadata: {metadata}")
    print(f"Scope: {get_scope(TestService)}")
    print(f"Token: {get_token(TestService)}")
    
    # Test 2: Decorador con scope
    @injectable(scope=Scope.TRANSIENT)
    class TransientService:
        pass
    
    print(f"\nTransientService scope: {get_scope(TransientService)}")
    
    # Test 3: Decorador con token custom
    @injectable(token="custom_token")
    class CustomService:
        pass
    
    print(f"\nCustomService token: {get_token(CustomService)}")
    
    # Test 4: Registro de providers
    register_provider(TestService)
    print(f"\nProvider registered: {get_provider('TestService').__name__}")
    
    # Test 5: Clear registry
    clear_registry()
    print(f"Registry cleared: {len(_provider_registry)} providers")
    
    print("\n✅ Todos los tests de @injectable pasaron")
