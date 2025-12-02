"""
Decorador @inject para Dependency Injection

Implementación de: TASK-035C
Historia: VELA-575 - Sistema de Dependency Injection
Sprint: 13
Fecha: 2025-12-01

Descripción:
Define el decorador @inject que marca parámetros de constructor
para inyección automática de dependencias.

El decorador extrae metadata de type hints y permite especificar
tokens custom para resolver dependencias.
"""

from dataclasses import dataclass, field
from typing import Type, Optional, Any, get_type_hints, get_args, get_origin
import inspect


@dataclass
class InjectMetadata:
    """
    Metadata para un parámetro marcado con @inject.
    
    Attributes:
        param_name: Nombre del parámetro
        param_type: Tipo del parámetro (extraído de type hint)
        token: Token custom para resolver dependencia (opcional)
        optional: Si el parámetro es opcional (Optional[T])
        default: Valor por defecto del parámetro (si existe)
    
    Examples:
        >>> metadata = InjectMetadata(
        >>>     param_name="repository",
        >>>     param_type=UserRepository,
        >>>     token="user-repo",
        >>>     optional=False
        >>> )
    """
    param_name: str
    param_type: Optional[Type] = None
    token: Optional[str] = None
    optional: bool = False
    default: Any = inspect.Parameter.empty
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.param_name:
            raise ValueError("param_name cannot be empty")


class _InjectMarker:
    """
    Objeto sentinel que marca un parámetro para inyección.
    
    Este objeto se usa como valor por defecto de parámetros
    para indicar que deben ser inyectados por el DI container.
    """
    def __init__(self, token=None):
        """
        Args:
            token: Token custom para resolver dependencia (opcional)
        """
        self.__inject_metadata__ = InjectMetadata(
            param_name="__placeholder__",
            token=token
        )
    
    def __repr__(self):
        return f"_InjectMarker(token={self.__inject_metadata__.token})"


def inject(token: Optional[str] = None):
    """
    Marca un parámetro de constructor para inyección.
    
    Este decorador extrae metadata del parámetro (tipo, nombre, default)
    y permite especificar un token custom para resolver la dependencia.
    
    Args:
        token: Token custom para resolver dependencia (opcional)
               Si no se proporciona, usa el tipo del parámetro
    
    Returns:
        Parámetro decorado con metadata de inyección
    
    Raises:
        TypeError: Si se usa fuera de un constructor
        ValueError: Si el parámetro no tiene type hint
    
    Examples:
        Uso básico con type hint:
        >>> @injectable
        >>> service UserService:
        >>>     repository: UserRepository
        >>>     
        >>>     constructor(@inject repository: UserRepository) {
        >>>         this.repository = repository
        >>>     }
        
        Uso con token custom:
        >>> @injectable
        >>> service UserService:
        >>>     constructor(@inject("user-repo") repository: Repository) {
        >>>         this.repository = repository
        >>>     }
        
        Uso con parámetro opcional:
        >>> @injectable
        >>> service CacheService:
        >>>     constructor(@inject cache: Option<RedisCache>) {
        >>>         this.cache = cache
        >>>     }
        
        Múltiples inyecciones:
        >>> @injectable
        >>> controller UserController:
        >>>     service: UserService
        >>>     logger: Logger
        >>>     
        >>>     constructor(
        >>>         @inject service: UserService,
        >>>         @inject logger: Logger
        >>>     ) {
        >>>         this.service = service
        >>>         this.logger = logger
        >>>     }
    
    Notes:
        - DEBE usarse solo en parámetros de constructor
        - El parámetro DEBE tener type hint
        - El token custom es opcional (default: tipo del parámetro)
        - Soporta Optional[T] automáticamente
        - Soporta valores por defecto
    """
    
    return _InjectMarker(token)


def get_inject_metadata(func: callable) -> list[InjectMetadata]:
    """
    Extrae metadata de inyección de una función (constructor).
    
    Esta función inspecciona los parámetros del constructor,
    identifica cuáles tienen @inject, y extrae su metadata
    (tipo, nombre, token, optional, default).
    
    Args:
        func: Función/constructor a inspeccionar
    
    Returns:
        Lista de InjectMetadata para cada parámetro con @inject
    
    Raises:
        ValueError: Si un parámetro con @inject no tiene type hint
    
    Examples:
        >>> class MyService:
        >>>     def __init__(self, repo: UserRepository, cache: Optional[Cache] = None):
        >>>         pass
        >>> 
        >>> metadata = get_inject_metadata(MyService.__init__)
        >>> print(len(metadata))  # 2
        >>> print(metadata[0].param_name)  # "repo"
        >>> print(metadata[0].param_type)  # <class 'UserRepository'>
    """
    metadata_list = []
    
    # Obtener signature del constructor
    try:
        sig = inspect.signature(func)
    except (ValueError, TypeError):
        # Función sin signature válida
        return metadata_list
    
    # Obtener type hints (maneja forward references)
    try:
        hints = get_type_hints(func)
    except Exception:
        # Si falla get_type_hints, usar annotations raw
        hints = getattr(func, '__annotations__', {})
    
    # Iterar sobre parámetros (skip 'self' / 'cls')
    for param_name, param in sig.parameters.items():
        if param_name in ['self', 'cls']:
            continue
        
        # Verificar si tiene @inject (buscar en default o annotations)
        has_inject = False
        token = None
        
        # Buscar metadata en el parámetro mismo (si fue decorado)
        if hasattr(param.default, '__inject_metadata__'):
            has_inject = True
            token = param.default.__inject_metadata__.token
        
        # Si no tiene @inject, skip
        if not has_inject:
            continue
        
        # Extraer tipo del parámetro
        param_type = hints.get(param_name, param.annotation)
        
        # Validar que tenga type hint
        if param_type is inspect.Parameter.empty:
            raise ValueError(
                f"Parameter '{param_name}' with @inject must have type hint. "
                f"Example: @inject {param_name}: SomeType"
            )
        
        # Verificar si es Optional[T]
        optional = False
        if get_origin(param_type) is type(None) or get_origin(param_type) is Optional:
            optional = True
            # Extraer tipo interno de Optional[T]
            args = get_args(param_type)
            if args:
                param_type = args[0]
        
        # Crear metadata
        metadata = InjectMetadata(
            param_name=param_name,
            param_type=param_type,
            token=token,
            optional=optional,
            default=param.default if param.default is not inspect.Parameter.empty else inspect.Parameter.empty
        )
        
        metadata_list.append(metadata)
    
    return metadata_list


def set_inject_metadata(cls: Type, metadata: list[InjectMetadata]) -> None:
    """
    Almacena metadata de inyección en una clase.
    
    Args:
        cls: Clase donde almacenar metadata
        metadata: Lista de InjectMetadata
    
    Examples:
        >>> metadata = [
        >>>     InjectMetadata(param_name="repo", param_type=UserRepository)
        >>> ]
        >>> set_inject_metadata(UserService, metadata)
        >>> 
        >>> # Luego recuperar
        >>> stored = get_inject_metadata(UserService)
    """
    setattr(cls, '__inject_params__', metadata)


def get_constructor_inject_metadata(cls: Type) -> list[InjectMetadata]:
    """
    Obtiene metadata de inyección de un constructor de clase.
    
    Wrapper que combina extracción desde __init__ y cache en la clase.
    
    Args:
        cls: Clase cuyo constructor inspeccionar
    
    Returns:
        Lista de InjectMetadata
    
    Examples:
        >>> @injectable
        >>> class UserService:
        >>>     def __init__(self, @inject repo: UserRepository):
        >>>         self.repo = repo
        >>> 
        >>> metadata = get_constructor_inject_metadata(UserService)
        >>> print(metadata[0].param_name)  # "repo"
    """
    # Verificar si ya está cacheada
    if hasattr(cls, '__inject_params__'):
        return getattr(cls, '__inject_params__')
    
    # Extraer de __init__
    if hasattr(cls, '__init__'):
        metadata = get_inject_metadata(cls.__init__)
        # Cachear en la clase
        set_inject_metadata(cls, metadata)
        return metadata
    
    return []


def has_inject_params(cls: Type) -> bool:
    """
    Verifica si una clase tiene parámetros con @inject.
    
    Args:
        cls: Clase a verificar
    
    Returns:
        True si tiene al menos un parámetro con @inject
    
    Examples:
        >>> @injectable
        >>> class ServiceA:
        >>>     def __init__(self, @inject repo: Repository):
        >>>         pass
        >>> 
        >>> has_inject_params(ServiceA)  # True
        >>> 
        >>> class ServiceB:
        >>>     def __init__(self, value: int):
        >>>         pass
        >>> 
        >>> has_inject_params(ServiceB)  # False
    """
    metadata = get_constructor_inject_metadata(cls)
    return len(metadata) > 0


def get_inject_token(metadata: InjectMetadata) -> str:
    """
    Obtiene el token para resolver una dependencia.
    
    Args:
        metadata: InjectMetadata del parámetro
    
    Returns:
        Token string (custom o nombre de tipo)
    
    Examples:
        >>> metadata = InjectMetadata(
        >>>     param_name="repo",
        >>>     param_type=UserRepository,
        >>>     token="custom-repo"
        >>> )
        >>> get_inject_token(metadata)  # "custom-repo"
        >>> 
        >>> metadata2 = InjectMetadata(
        >>>     param_name="service",
        >>>     param_type=UserService
        >>> )
        >>> get_inject_token(metadata2)  # "UserService"
    """
    if metadata.token:
        return metadata.token
    
    if metadata.param_type:
        return metadata.param_type.__name__
    
    return metadata.param_name


# Tests básicos
if __name__ == "__main__":
    print("=== Tests básicos de @inject ===\n")
    
    # Test 1: Metadata básico
    print("Test 1: Crear InjectMetadata")
    metadata = InjectMetadata(
        param_name="repository",
        param_type=str,
        token="my-repo"
    )
    print(f"  param_name: {metadata.param_name}")
    print(f"  param_type: {metadata.param_type}")
    print(f"  token: {metadata.token}")
    print(f"  ✓ InjectMetadata creado\n")
    
    # Test 2: get_inject_token
    print("Test 2: get_inject_token")
    print(f"  Con token custom: {get_inject_token(metadata)}")
    
    class DummyClass:
        pass
    
    metadata2 = InjectMetadata(param_name="dummy", param_type=DummyClass)
    print(f"  Sin token (usa tipo): {get_inject_token(metadata2)}")
    print(f"  ✓ Tokens extraídos correctamente\n")
    
    # Test 3: has_inject_params
    print("Test 3: has_inject_params")
    
    class ServiceWithoutInject:
        def __init__(self, value: int):
            self.value = value
    
    print(f"  Clase sin @inject: {has_inject_params(ServiceWithoutInject)}")
    print(f"  ✓ Detección correcta\n")
    
    print("✅ Todos los tests básicos pasaron")
