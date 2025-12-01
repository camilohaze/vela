"""
Decorador @guard para Vela DI System

Este módulo implementa el decorador @guard para guards de autorización HTTP (backend-only).

Inspirado en NestJS guards system:
- @guard(AuthGuard, RolesGuard)
- Aplicable a controllers y route handlers
- ExecutionContext para acceso a request/response

Implementación de: TASK-035E2 (Fase 4: Guards)
Historia: VELA-575
Fecha: 2025-12-01
"""

from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional, Type


# ============================================================================
# INTERFACES
# ============================================================================


class ExecutionContext:
    """
    Contexto de ejecución para guards.
    
    Proporciona acceso a request, response, handler, y metadata
    durante la ejecución de un guard.
    
    Attributes:
        request: HTTP request object
        response: HTTP response object
        handler: Route handler function
        metadata: Metadata adicional del route/controller
    """
    
    def __init__(
        self,
        request: Any = None,
        response: Any = None,
        handler: Optional[Callable] = None,
        metadata: Optional[Dict[str, Any]] = None
    ):
        """
        Inicializa el contexto de ejecución.
        
        Args:
            request: HTTP request object
            response: HTTP response object
            handler: Route handler function
            metadata: Metadata adicional
        """
        self.request = request
        self.response = response
        self.handler = handler
        self.metadata = metadata or {}
    
    def get_class(self) -> Optional[Type]:
        """Obtiene la clase del controller."""
        if self.handler and hasattr(self.handler, '__self__'):
            return type(self.handler.__self__)
        return None
    
    def get_handler(self) -> Optional[Callable]:
        """Obtiene el route handler."""
        return self.handler
    
    def switch_to_http(self) -> 'ExecutionContext':
        """Cambia a contexto HTTP."""
        return self


# ============================================================================
# METADATA CLASSES
# ============================================================================


@dataclass
class GuardMetadata:
    """
    Metadata para guards HTTP (backend - NestJS-style).
    
    Ejemplo:
        @guard(AuthGuard, RolesGuard)
        @controller("/users")
        class UserController:
            pass
    
    Attributes:
        guard_classes: Lista de clases de guards a aplicar
        options: Opciones adicionales para configurar guards
    """
    guard_classes: List[Type]
    options: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.guard_classes:
            raise ValueError("Guard must have at least one guard class")
        
        if not all(isinstance(cls, type) for cls in self.guard_classes):
            raise TypeError("All guard_classes must be Type classes")


# ============================================================================
# DECORADOR PRINCIPAL
# ============================================================================


def guard(*guard_classes: Type, **options) -> Callable:
    """
    Decorador @guard para guards de autorización HTTP (backend-only).
    
    Aplica guards a controllers o route handlers para validar
    autorización antes de ejecutar el handler.
    
    Inspirado en:
    - NestJS: @UseGuards()
    - Spring Boot: @PreAuthorize
    - FastAPI: Depends() con auth
    
    Args:
        *guard_classes: Clases de guards a aplicar
        **options: Opciones adicionales
    
    Returns:
        Decorador que aplica GuardMetadata
    
    Raises:
        ValueError: Si se usa en contexto inválido (frontend, UI pipes, etc.)
    
    Examples:
        # Class-level guard (aplica a todas las rutas del controller)
        @guard(AuthGuard, RolesGuard)
        @controller("/users")
        class UserController:
            pass
        
        # Method-level guard (aplica solo a una ruta)
        @guard(AdminGuard)
        @delete("/users/:id")
        def delete_user(id: int) -> dict:
            return {"deleted": id}
        
        # Guard con opciones
        @guard(RolesGuard, roles=["admin", "moderator"])
        @controller("/admin")
        class AdminController:
            pass
    """
    def decorator(target: Any) -> Any:
        # ✅ Validación 1: Solo en classes o functions (no en pipes, services sin @controller)
        if not (isinstance(target, type) or callable(target)):
            raise ValueError(
                "@guard can only be used on classes or functions"
            )
        
        # ✅ Validación 2: No en UI pipes
        if hasattr(target, '__ui_pipe_metadata__'):
            raise ValueError(
                "@guard cannot be used on UI pipes. "
                "Guards are for backend HTTP only."
            )
        
        # ✅ Validación 3: Si es clase, debe ser @controller
        if isinstance(target, type):
            if not hasattr(target, '__controller_metadata__'):
                raise ValueError(
                    "@guard on classes can only be used with @controller. "
                    "Controllers handle HTTP requests."
                )
        
        # ✅ Validación 4: Si es función, debe tener route metadata
        if callable(target) and not isinstance(target, type):
            if not hasattr(target, '__route_metadata__'):
                # Permitir en funciones sin route metadata (será validado después)
                pass
        
        # Crear y adjuntar metadata
        metadata = GuardMetadata(
            guard_classes=list(guard_classes),
            options=options
        )
        target.__guard_metadata__ = metadata
        
        return target
    
    return decorator


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================


def is_guard(obj: Any) -> bool:
    """
    Verifica si un objeto tiene guard metadata.
    
    Args:
        obj: Objeto a verificar (clase o función)
    
    Returns:
        True si tiene __guard_metadata__
    """
    return hasattr(obj, '__guard_metadata__')


def get_guard_metadata(obj: Any) -> Optional[GuardMetadata]:
    """
    Obtiene metadata de guard de un objeto.
    
    Args:
        obj: Objeto a inspeccionar (clase o función)
    
    Returns:
        GuardMetadata si existe, None en caso contrario
    """
    return getattr(obj, '__guard_metadata__', None)


def get_guard_classes(obj: Any) -> List[Type]:
    """
    Obtiene las clases de guards aplicadas a un objeto.
    
    Args:
        obj: Objeto a inspeccionar
    
    Returns:
        Lista de clases de guards (vacía si no tiene guards)
    """
    metadata = get_guard_metadata(obj)
    return metadata.guard_classes if metadata else []


def combine_guards(*objects: Any) -> List[Type]:
    """
    Combina guards de múltiples objetos.
    
    Útil para combinar guards de controller y route handler.
    Los guards del controller se ejecutan primero, luego los del handler.
    
    Args:
        *objects: Objetos con guard metadata
    
    Returns:
        Lista de clases de guards combinadas
    
    Example:
        @guard(AuthGuard)
        @controller("/users")
        class UserController:
            @guard(RolesGuard)
            @delete("/users/:id")
            def delete(self, id):
                pass
        
        # Combinar guards
        combined = combine_guards(UserController, UserController.delete)
        # Resultado: [AuthGuard, RolesGuard]
    """
    all_guards = []
    
    for obj in objects:
        metadata = get_guard_metadata(obj)
        if metadata:
            all_guards.extend(metadata.guard_classes)
    
    return all_guards


def validate_guard_class(cls: Type) -> None:
    """
    Valida que una clase de guard tenga el método canActivate().
    
    Guard debe implementar:
    
        interface Guard {
            fn canActivate(context: ExecutionContext) -> Bool
        }
    
    Args:
        cls: Clase a validar
    
    Raises:
        TypeError: Si la clase no tiene método canActivate()
    """
    if not hasattr(cls, 'canActivate'):
        raise TypeError(
            f"Guard class {cls.__name__} must implement canActivate() method. "
            "Guards must implement: fn canActivate(context: ExecutionContext) -> Bool"
        )
    
    if not callable(getattr(cls, 'canActivate')):
        raise TypeError(
            f"{cls.__name__}.canActivate must be a callable method"
        )


# ============================================================================
# EXPORTS
# ============================================================================

__all__ = [
    # Interfaces
    'ExecutionContext',
    
    # Metadata
    'GuardMetadata',
    
    # Decoradores
    'guard',
    
    # Helpers
    'is_guard',
    'get_guard_metadata',
    'get_guard_classes',
    'combine_guards',
    'validate_guard_class',
]
