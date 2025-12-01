"""
Decorador @pipe context-aware para Vela DI System

Este módulo implementa el decorador @pipe que AUTO-DETECTA el contexto de uso:

1. FRONTEND (UI Pipes - Angular-style):
   @pipe(name="currency", pure=True)
   pipe CurrencyPipe implements PipeTransform {
     fn transform(value: Number) -> String { ... }
   }

2. BACKEND (HTTP Pipes - NestJS-style):
   @pipe(ValidationPipe, TransformPipe)
   @controller("/users")
   class UserController { ... }

Auto-detección por sintaxis:
- Si kwargs tiene 'name' → Frontend UI Pipe
- Si args tiene Type classes → Backend HTTP Pipe

Implementación de: TASK-035E2 (Fase 1: Backend HTTP Pipes)
Historia: VELA-575
Fecha: 2025-12-01
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Type, Union
from functools import wraps


# ============================================================================
# ENUMS
# ============================================================================


class PipeContext(str, Enum):
    """
    Contexto de ejecución del pipe.
    
    - UI: Transformación en templates (frontend)
    - HTTP: Validación/transformación en HTTP handlers (backend)
    - PARAMETER: Transformación de parámetros individuales (backend)
    """
    UI = "ui"
    HTTP = "http"
    PARAMETER = "param"


# ============================================================================
# METADATA CLASSES
# ============================================================================


@dataclass
class HTTPPipeMetadata:
    """
    Metadata para HTTP pipes (backend - NestJS-style).
    
    Ejemplo:
        @pipe(ValidationPipe, TransformPipe)
        @controller("/users")
        class UserController:
            pass
    
    Attributes:
        pipe_classes: Lista de clases de pipes a aplicar
        target: Objetivo del pipe ("input" o "output")
        context: Contexto de ejecución (PipeContext.HTTP)
        options: Opciones adicionales para configurar pipes
    """
    pipe_classes: List[Type]
    target: str = "input"
    context: PipeContext = PipeContext.HTTP
    options: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.pipe_classes:
            raise ValueError("HTTP pipe must have at least one pipe class")
        
        if not all(isinstance(cls, type) for cls in self.pipe_classes):
            raise TypeError("All pipe_classes must be Type classes")
        
        if self.target not in ("input", "output"):
            raise ValueError(f"Invalid target: {self.target}. Must be 'input' or 'output'")


@dataclass
class UIPipeMetadata:
    """
    Metadata para UI pipes (frontend - Angular-style).
    
    Ejemplo:
        @pipe(name="currency", pure=True)
        pipe CurrencyPipe implements PipeTransform {
            fn transform(value: Number) -> String { ... }
        }
    
    Attributes:
        name: Nombre del pipe para usar en templates (ej: 'currency')
        pure: Si es pipe puro (cacheable, sin side effects)
        standalone: Si es componente standalone (no requiere módulo)
        context: Contexto de ejecución (PipeContext.UI)
    """
    name: str
    pure: bool = True
    standalone: bool = False
    context: PipeContext = PipeContext.UI
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.name or not self.name.strip():
            raise ValueError("UI pipe name cannot be empty")
        
        self.name = self.name.strip()
        
        if not self.name.isidentifier():
            raise ValueError(
                f"UI pipe name '{self.name}' is not a valid identifier. "
                "Use only letters, numbers, and underscores."
            )


@dataclass
class ParameterPipeMetadata:
    """
    Metadata para parameter pipes (backend - NestJS-style).
    
    Ejemplo:
        @get("/:id")
        fn getUser(@param @pipe(ParseIntPipe) id: Number) -> User:
            pass
    
    Attributes:
        pipe_classes: Lista de clases de pipes a aplicar al parámetro
        context: Contexto de ejecución (PipeContext.PARAMETER)
        options: Opciones adicionales para configurar pipes
    """
    pipe_classes: List[Type]
    context: PipeContext = PipeContext.PARAMETER
    options: Dict[str, Any] = field(default_factory=dict)
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        if not self.pipe_classes:
            raise ValueError("Parameter pipe must have at least one pipe class")
        
        if not all(isinstance(cls, type) for cls in self.pipe_classes):
            raise TypeError("All pipe_classes must be Type classes")


# ============================================================================
# DECORADORES PRINCIPALES
# ============================================================================


def pipe(*args, **kwargs):
    """
    Decorador @pipe context-aware.
    
    AUTO-DETECTA el contexto de uso por sintaxis:
    
    1. FRONTEND (UI Pipes):
       @pipe(name="currency", pure=True)
       pipe CurrencyPipe implements PipeTransform { ... }
    
    2. BACKEND HTTP (Class-level):
       @pipe(ValidationPipe, TransformPipe)
       @controller("/users")
       class UserController { ... }
    
    3. BACKEND PARAMETER (Parameter-level):
       fn getUser(@param @pipe(ParseIntPipe) id: Number) -> User { ... }
    
    Args:
        *args: Clases de pipes (backend)
        **kwargs: Configuración de pipe (frontend)
    
    Returns:
        Decorador apropiado según contexto
    
    Raises:
        ValueError: Si la sintaxis no coincide con ningún contexto
    
    Examples:
        # Frontend UI Pipe
        @pipe(name="currency", pure=True)
        class CurrencyPipe:
            def transform(self, value: float) -> str:
                return f"${value:.2f}"
        
        # Backend HTTP Pipe
        @pipe(ValidationPipe, TransformPipe)
        @controller("/users")
        class UserController:
            pass
        
        # Backend Parameter Pipe
        @get("/:id")
        def get_user(@param @pipe(ParseIntPipe) id: int) -> dict:
            return {"id": id}
    """
    # CASO 1: Frontend UI Pipe (kwargs con 'name')
    if 'name' in kwargs:
        return _create_ui_pipe_decorator(**kwargs)
    
    # CASO 2: Backend HTTP/Parameter Pipe (args con Type classes)
    elif args and all(isinstance(arg, type) for arg in args):
        return _create_http_pipe_decorator(*args, **kwargs)
    
    # CASO 3: Error - sintaxis inválida
    else:
        raise ValueError(
            "Invalid @pipe usage. Use one of:\n"
            "  - Frontend: @pipe(name='pipeName', pure=True)\n"
            "  - Backend: @pipe(PipeClass1, PipeClass2, ...)"
        )


def _create_ui_pipe_decorator(
    name: str,
    pure: bool = True,
    standalone: bool = False,
    **extra_kwargs
) -> Callable:
    """
    Crea decorador para UI pipes (frontend).
    
    Validaciones:
    - Solo aplicable a clases que terminan en 'Pipe'
    - No compatible con @controller
    - No compatible con @injectable (backend)
    
    Args:
        name: Nombre del pipe para templates
        pure: Si es pipe puro (cacheable)
        standalone: Si es componente standalone
        **extra_kwargs: Kwargs adicionales (ignorados, pero permitidos para flexibilidad)
    
    Returns:
        Decorador que aplica UIPipeMetadata
    
    Raises:
        ValueError: Si se usa en contexto inválido
    """
    def decorator(cls: Type) -> Type:
        # ✅ Validación 1: Solo en pipe classes (sufijo 'Pipe')
        if not cls.__name__.endswith('Pipe'):
            raise ValueError(
                f"@pipe(name='...') can only be used on pipe classes. "
                f"Class name must end with 'Pipe'. Got: {cls.__name__}"
            )
        
        # ✅ Validación 2: No compatible con @controller
        if hasattr(cls, '__controller_metadata__'):
            raise ValueError(
                f"@pipe(name='{name}') cannot be used with @controller. "
                "UI pipes are for frontend templates only."
            )
        
        # ✅ Validación 3: No compatible con HTTP pipe metadata
        if hasattr(cls, '__http_pipe_metadata__'):
            raise ValueError(
                f"@pipe(name='{name}') cannot be used with backend HTTP pipes. "
                "Choose either frontend (name='...') or backend (PipeClass, ...)."
            )
        
        # ✅ Validación 4: No compatible con @injectable backend
        if hasattr(cls, '__injectable_metadata__'):
            # Permitir @injectable solo si es para UI (Angular-style)
            injectable_meta = cls.__injectable_metadata__
            # Si es dict con 'context' == 'backend', o cualquier indicador de backend
            if isinstance(injectable_meta, dict):
                if injectable_meta.get('context') == 'backend':
                    raise ValueError(
                        f"@pipe(name='{name}') cannot be used with @injectable(backend). "
                        "UI pipes are for frontend only."
                    )
            # Si tiene atributo context == 'backend'
            elif hasattr(injectable_meta, 'context') and injectable_meta.context == 'backend':
                raise ValueError(
                    f"@pipe(name='{name}') cannot be used with @injectable(backend). "
                    "UI pipes are for frontend only."
                )
        
        # Crear y adjuntar metadata
        metadata = UIPipeMetadata(
            name=name,
            pure=pure,
            standalone=standalone
        )
        cls.__ui_pipe_metadata__ = metadata
        
        return cls
    
    return decorator


def _create_http_pipe_decorator(*pipe_classes: Type, **options) -> Callable:
    """
    Crea decorador para HTTP pipes (backend).
    
    Validaciones:
    - Solo aplicable a controllers, route handlers, services, o parámetros
    - No compatible con UI pipe metadata
    - No compatible con clases que terminan en 'Pipe' (UI pipes)
    
    Args:
        *pipe_classes: Clases de pipes a aplicar
        **options: Opciones adicionales (target, etc.)
    
    Returns:
        Decorador que aplica HTTPPipeMetadata o ParameterPipeMetadata
    
    Raises:
        ValueError: Si se usa en contexto inválido
    """
    target = options.pop('target', 'input')
    
    def decorator(obj: Any) -> Any:
        # ✅ Validación 1: No compatible con UI pipes
        if hasattr(obj, '__ui_pipe_metadata__'):
            raise ValueError(
                "@pipe(PipeClass, ...) cannot be used on UI pipe classes. "
                "UI pipes use @pipe(name='...') syntax."
            )
        
        # ✅ Validación 2: No en clases que terminan en 'Pipe' (son UI pipes)
        if isinstance(obj, type) and obj.__name__.endswith('Pipe'):
            raise ValueError(
                f"@pipe(PipeClass, ...) cannot be used on class '{obj.__name__}'. "
                "Classes ending in 'Pipe' should use @pipe(name='...') syntax."
            )
        
        # CASO A: Aplicado a clase (controller o service)
        if isinstance(obj, type):
            # Validar que sea controller o service
            is_controller = hasattr(obj, '__controller_metadata__')
            is_service = hasattr(obj, '__injectable_metadata__')
            
            if not (is_controller or is_service):
                raise ValueError(
                    "@pipe(PipeClass, ...) on classes can only be used with "
                    "@controller or @injectable services"
                )
            
            metadata = HTTPPipeMetadata(
                pipe_classes=list(pipe_classes),
                target=target,
                options=options
            )
            obj.__http_pipe_metadata__ = metadata
            return obj
        
        # CASO B: Aplicado a función (route handler)
        elif callable(obj):
            # Route handler o método
            metadata = HTTPPipeMetadata(
                pipe_classes=list(pipe_classes),
                target=target,
                options=options
            )
            obj.__http_pipe_metadata__ = metadata
            return obj
        
        # CASO C: Aplicado a parámetro (parameter decorator)
        # Esto se maneja diferente porque los decoradores de parámetros
        # no reciben el callable directamente
        else:
            metadata = ParameterPipeMetadata(
                pipe_classes=list(pipe_classes),
                options=options
            )
            # Para parámetros, retornamos la metadata directamente
            # El sistema de DI la capturará
            return metadata
    
    return decorator


# ============================================================================
# HELPER FUNCTIONS
# ============================================================================


def is_ui_pipe(cls: Type) -> bool:
    """
    Verifica si una clase es un UI pipe (frontend).
    
    Args:
        cls: Clase a verificar
    
    Returns:
        True si tiene __ui_pipe_metadata__
    """
    return hasattr(cls, '__ui_pipe_metadata__')


def is_http_pipe(obj: Any) -> bool:
    """
    Verifica si un objeto tiene HTTP pipe metadata (backend).
    
    Args:
        obj: Objeto a verificar (clase o función)
    
    Returns:
        True si tiene __http_pipe_metadata__
    """
    return hasattr(obj, '__http_pipe_metadata__')


def is_parameter_pipe(obj: Any) -> bool:
    """
    Verifica si un objeto es metadata de parameter pipe.
    
    Args:
        obj: Objeto a verificar
    
    Returns:
        True si es instancia de ParameterPipeMetadata
    """
    return isinstance(obj, ParameterPipeMetadata)


def get_ui_pipe_metadata(cls: Type) -> Optional[UIPipeMetadata]:
    """
    Obtiene metadata de UI pipe de una clase.
    
    Args:
        cls: Clase a inspeccionar
    
    Returns:
        UIPipeMetadata si existe, None en caso contrario
    """
    return getattr(cls, '__ui_pipe_metadata__', None)


def get_http_pipe_metadata(obj: Any) -> Optional[HTTPPipeMetadata]:
    """
    Obtiene metadata de HTTP pipe de un objeto.
    
    Args:
        obj: Objeto a inspeccionar (clase o función)
    
    Returns:
        HTTPPipeMetadata si existe, None en caso contrario
    """
    return getattr(obj, '__http_pipe_metadata__', None)


def get_pipe_name(cls: Type) -> Optional[str]:
    """
    Obtiene el nombre del UI pipe.
    
    Args:
        cls: Clase de pipe
    
    Returns:
        Nombre del pipe si es UI pipe, None en caso contrario
    """
    metadata = get_ui_pipe_metadata(cls)
    return metadata.name if metadata else None


def get_pipe_classes(obj: Any) -> List[Type]:
    """
    Obtiene las clases de pipes aplicadas a un objeto.
    
    Args:
        obj: Objeto a inspeccionar
    
    Returns:
        Lista de clases de pipes (vacía si no tiene pipes)
    """
    # HTTP pipes
    http_meta = get_http_pipe_metadata(obj)
    if http_meta:
        return http_meta.pipe_classes
    
    # Parameter pipes
    if isinstance(obj, ParameterPipeMetadata):
        return obj.pipe_classes
    
    return []


def validate_pipe_class(cls: Type) -> None:
    """
    Valida que una clase de pipe tenga el método transform().
    
    Para UI pipes (frontend), deben implementar PipeTransform interface:
    
        interface PipeTransform {
            fn transform(value: Any, ...args) -> Any
        }
    
    Args:
        cls: Clase a validar
    
    Raises:
        TypeError: Si la clase no tiene método transform()
    """
    if not hasattr(cls, 'transform'):
        raise TypeError(
            f"Pipe class {cls.__name__} must implement transform() method. "
            "UI pipes must implement PipeTransform interface."
        )
    
    if not callable(getattr(cls, 'transform')):
        raise TypeError(
            f"{cls.__name__}.transform must be a callable method"
        )


# ============================================================================
# EXPORTS
# ============================================================================

__all__ = [
    # Enums
    'PipeContext',
    
    # Metadata
    'HTTPPipeMetadata',
    'UIPipeMetadata',
    'ParameterPipeMetadata',
    
    # Decoradores
    'pipe',
    
    # Helpers
    'is_ui_pipe',
    'is_http_pipe',
    'is_parameter_pipe',
    'get_ui_pipe_metadata',
    'get_http_pipe_metadata',
    'get_pipe_name',
    'get_pipe_classes',
    'validate_pipe_class',
]
