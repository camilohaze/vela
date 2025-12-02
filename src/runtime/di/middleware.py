"""
Decorador @middleware para Vela DI System

Este módulo implementa el decorador @middleware para interceptores HTTP (backend-only).

Inspirado en NestJS middleware system:
- @middleware(LoggingMiddleware, RateLimitMiddleware)
- Aplicable a controllers y route handlers
- Orden de ejecución configurable

Implementación de: TASK-035E2 (Fase 3: Middleware)
Historia: VELA-575
Fecha: 2025-12-01
"""

from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, Type


# ============================================================================
# METADATA CLASSES
# ============================================================================


@dataclass
class MiddlewareMetadata:
    """
    Metadata para middleware HTTP (backend - NestJS-style).
    
    Ejemplo:
        @middleware(LoggingMiddleware, RateLimitMiddleware)
        @controller("/users")
        class UserController:
            pass
    
    Attributes:
        middleware_classes: Lista de clases de middleware a aplicar
        order: Orden de ejecución (menor = primero)
        options: Opciones adicionales para configurar middleware
    """
    middleware_classes: List[Type]
    order: int = 0
    options: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.middleware_classes:
            raise ValueError("Middleware must have at least one middleware class")
        
        if not all(isinstance(cls, type) for cls in self.middleware_classes):
            raise TypeError("All middleware_classes must be Type classes")


# ============================================================================
# DECORADOR PRINCIPAL
# ============================================================================


def middleware(*middleware_classes: Type, **options) -> Callable:
    """
    Decorador @middleware para interceptores HTTP (backend-only).
    
    Aplica middleware a controllers o route handlers para interceptar
    requests/responses antes de llegar al handler.
    
    Inspirado en:
    - NestJS: @UseMiddleware()
    - Express.js: app.use(middleware)
    - Spring Boot: HandlerInterceptor
    
    Args:
        *middleware_classes: Clases de middleware a aplicar
        **options: Opciones adicionales (order, etc.)
    
    Returns:
        Decorador que aplica MiddlewareMetadata
    
    Raises:
        ValueError: Si se usa en contexto inválido (frontend, UI pipes, etc.)
    
    Examples:
        # Class-level middleware (aplica a todas las rutas del controller)
        @middleware(LoggingMiddleware, RateLimitMiddleware)
        @controller("/users")
        class UserController:
            pass
        
        # Method-level middleware (aplica solo a una ruta)
        @middleware(AuthMiddleware)
        @post("/create")
        def create_user(data: dict) -> dict:
            return {"user": data}
        
        # Middleware con opciones
        @middleware(RateLimitMiddleware, order=1, max_requests=100)
        @controller("/api")
        class ApiController:
            pass
    """
    order = options.pop('order', 0)
    
    def decorator(target: Any) -> Any:
        # ✅ Validación 1: Solo en classes o functions (no en pipes, services sin @controller)
        if not (isinstance(target, type) or callable(target)):
            raise ValueError(
                "@middleware can only be used on classes or functions"
            )
        
        # ✅ Validación 2: No en UI pipes
        if hasattr(target, '__ui_pipe_metadata__'):
            raise ValueError(
                "@middleware cannot be used on UI pipes. "
                "Middleware is for backend HTTP only."
            )
        
        # ✅ Validación 3: Si es clase, debe ser @controller
        if isinstance(target, type):
            if not hasattr(target, '__controller_metadata__'):
                raise ValueError(
                    "@middleware on classes can only be used with @controller. "
                    "Controllers handle HTTP requests."
                )
        
        # ✅ Validación 4: Si es función, debe tener route metadata
        if callable(target) and not isinstance(target, type):
            if not hasattr(target, '__route_metadata__'):
                # Permitir en funciones sin route metadata (será validado después)
                pass
        
        # Crear y adjuntar metadata
        metadata = MiddlewareMetadata(
            middleware_classes=list(middleware_classes),
            order=order,
            options=options
        )
        target.__middleware_metadata__ = metadata
        
        return target
    
    return decorator


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================


def is_middleware(obj: Any) -> bool:
    """
    Verifica si un objeto tiene middleware metadata.
    
    Args:
        obj: Objeto a verificar (clase o función)
    
    Returns:
        True si tiene __middleware_metadata__
    """
    return hasattr(obj, '__middleware_metadata__')


def get_middleware_metadata(obj: Any) -> Optional[MiddlewareMetadata]:
    """
    Obtiene metadata de middleware de un objeto.
    
    Args:
        obj: Objeto a inspeccionar (clase o función)
    
    Returns:
        MiddlewareMetadata si existe, None en caso contrario
    """
    return getattr(obj, '__middleware_metadata__', None)


def get_middleware_classes(obj: Any) -> List[Type]:
    """
    Obtiene las clases de middleware aplicadas a un objeto.
    
    Args:
        obj: Objeto a inspeccionar
    
    Returns:
        Lista de clases de middleware (vacía si no tiene middleware)
    """
    metadata = get_middleware_metadata(obj)
    return metadata.middleware_classes if metadata else []


def get_middleware_order(obj: Any) -> int:
    """
    Obtiene el orden de ejecución del middleware.
    
    Args:
        obj: Objeto a inspeccionar
    
    Returns:
        Orden de ejecución (0 por defecto)
    """
    metadata = get_middleware_metadata(obj)
    return metadata.order if metadata else 0


def combine_middleware(*objects: Any) -> List[Type]:
    """
    Combina middleware de múltiples objetos en orden de ejecución.
    
    Útil para combinar middleware de controller y route handler.
    
    Args:
        *objects: Objetos con middleware metadata
    
    Returns:
        Lista de clases de middleware ordenadas por 'order'
    
    Example:
        @middleware(LoggingMiddleware, order=1)
        @controller("/users")
        class UserController:
            @middleware(AuthMiddleware, order=0)
            @post("/create")
            def create(self):
                pass
        
        # Combinar middleware
        combined = combine_middleware(UserController, UserController.create)
        # Resultado: [AuthMiddleware, LoggingMiddleware] (ordenado por 'order')
    """
    all_middleware = []
    
    for obj in objects:
        metadata = get_middleware_metadata(obj)
        if metadata:
            for cls in metadata.middleware_classes:
                all_middleware.append((metadata.order, cls))
    
    # Ordenar por 'order' (menor primero)
    all_middleware.sort(key=lambda x: x[0])
    
    # Retornar solo las clases (sin el order)
    return [cls for order, cls in all_middleware]


def validate_middleware_class(cls: Type) -> None:
    """
    Valida que una clase de middleware tenga el método use().
    
    Middleware debe implementar:
    
        interface Middleware {
            fn use(request: Request, response: Response, next: () -> void) -> void
        }
    
    Args:
        cls: Clase a validar
    
    Raises:
        TypeError: Si la clase no tiene método use()
    """
    if not hasattr(cls, 'use'):
        raise TypeError(
            f"Middleware class {cls.__name__} must implement use() method. "
            "Middleware must implement: fn use(request, response, next) -> void"
        )
    
    if not callable(getattr(cls, 'use')):
        raise TypeError(
            f"{cls.__name__}.use must be a callable method"
        )


# ============================================================================
# EXPORTS
# ============================================================================

__all__ = [
    # Metadata
    'MiddlewareMetadata',
    
    # Decoradores
    'middleware',
    
    # Helpers
    'is_middleware',
    'get_middleware_metadata',
    'get_middleware_classes',
    'get_middleware_order',
    'combine_middleware',
    'validate_middleware_class',
]
