"""
Implementación del sistema de Controllers para Vela

Este módulo implementa el soporte runtime para la keyword 'controller' de Vela
y el decorador @controller para configurar routing.

IMPORTANTE: En Vela, 'controller' es una PALABRA RESERVADA (keyword), NO un decorador.
El decorador @controller se usa para CONFIGURAR el path y metadata del controller.

Sintaxis en Vela:
    // 1. Controller básico (keyword)
    controller UserController {
        // métodos HTTP aquí
    }
    
    // 2. Controller con configuración (keyword + decorator)
    @controller("/users", prefix="api")
    controller UserController {
        @get("/:id")
        fn getUser(id: Number) -> Result<User> { }
    }

Implementación de: TASK-035D2
Historia: VELA-575
Fecha: 2025-12-01

Características:
- ControllerMetadata dataclass con base_path, prefix, tags, metadata HTTP
- Decorador @controller(base_path) para configurar controllers
- Auto-registro en module controllers
- Helpers para introspección y routing
"""

from dataclasses import dataclass, field
from typing import Any, Optional, Type, Dict, List


# ============================================================================
# METADATA
# ============================================================================

@dataclass
class ControllerMetadata:
    """
    Metadata de un controller Vela.
    
    Un controller es una clase que maneja peticiones HTTP y define
    endpoints REST/API.
    
    Atributos:
        base_path: Ruta base del controller (ej: '/users', '/api/products')
        prefix: Prefijo adicional (ej: 'api', 'v1')
        tags: Tags para documentación OpenAPI
        description: Descripción del controller
    """
    base_path: str = "/"
    prefix: Optional[str] = None
    tags: List[str] = field(default_factory=list)
    description: Optional[str] = None
    
    def __post_init__(self):
        """Validar metadata después de inicialización."""
        # Normalizar base_path
        if not self.base_path.startswith('/'):
            self.base_path = f"/{self.base_path}"
        
        # Eliminar trailing slash (excepto para root "/")
        if self.base_path != "/" and self.base_path.endswith('/'):
            self.base_path = self.base_path.rstrip('/')
        
        # Convertir tags a lista si no lo es
        if isinstance(self.tags, str):
            self.tags = [self.tags]
        elif self.tags is None:
            self.tags = []
        else:
            self.tags = list(self.tags)
    
    def get_full_path(self) -> str:
        """
        Obtiene la ruta completa incluyendo prefix.
        
        Returns:
            Ruta completa del controller
            
        Example:
            metadata = ControllerMetadata(base_path='/users', prefix='api')
            metadata.get_full_path()  # '/api/users'
        """
        if self.prefix:
            # Normalizar prefix
            prefix = self.prefix if self.prefix.startswith('/') else f"/{self.prefix}"
            prefix = prefix.rstrip('/')
            
            # Combinar prefix + base_path
            if self.base_path == "/":
                return prefix
            return f"{prefix}{self.base_path}"
        
        return self.base_path


# ============================================================================
# DECORADOR @controller
# ============================================================================

# Atributo privado donde se guarda la metadata
_CONTROLLER_METADATA_ATTR = "__controller_metadata__"


def controller(
    base_path: str = "/",
    prefix: Optional[str] = None,
    tags: Optional[List[str]] = None,
    description: Optional[str] = None
):
    """
    Decorador @controller para CONFIGURAR un controller Vela.
    
    IMPORTANTE: En Vela, 'controller' es una KEYWORD (como 'class', 'interface').
    Este decorador @controller se usa para CONFIGURAR el path y metadata.
    
    Sintaxis en Vela:
        // Controller sin configuración (usa defaults)
        controller UserController {
            @get("/")
            fn list() -> Result<List<User>> { }
        }
        
        // Controller CON configuración (decorator + keyword)
        @controller("/users", prefix="api", tags=["Users"])
        controller UserController {
            @get("/:id")
            fn getUser(id: Number) -> Result<User> { }
            
            @post("/")
            fn createUser(dto: CreateUserDTO) -> Result<User> { }
        }
    
    Runtime Python:
        @controller("/users", prefix="api", tags=["Users"])
        class UserController:
            @get("/:id")
            def get_user(self, id: int):
                # ...
    
    Args:
        base_path: Ruta base del controller (default: "/")
        prefix: Prefijo adicional (ej: "api", "v1")
        tags: Tags para documentación OpenAPI
        description: Descripción del controller
        
    Returns:
        Decorador que agrega ControllerMetadata a la clase
        
    Example:
        @controller("/products")
        class ProductController:
            pass
        
        # Full path: "/products"
        
        @controller("/products", prefix="/api")
        class ProductController:
            pass
        
        # Full path: "/api/products"
    """
    def decorator(cls: Type) -> Type:
        # Crear metadata
        metadata = ControllerMetadata(
            base_path=base_path,
            prefix=prefix,
            tags=tags or [],
            description=description
        )
        
        # Agregar metadata a la clase
        setattr(cls, _CONTROLLER_METADATA_ATTR, metadata)
        
        # Auto-registrar el controller en el registry global
        register_controller(cls, metadata)
        
        return cls
    
    return decorator


# ============================================================================
# HELPERS - INTROSPECCIÓN
# ============================================================================

def is_controller(cls: Type) -> bool:
    """
    Verifica si una clase es un controller (tiene decorador @controller).
    
    Args:
        cls: Clase a verificar
        
    Returns:
        True si la clase tiene @controller, False en caso contrario
        
    Example:
        @controller("/users")
        class UserController:
            pass
        
        is_controller(UserController)  # True
        is_controller(str)             # False
    """
    return hasattr(cls, _CONTROLLER_METADATA_ATTR)


def get_controller_metadata(cls: Type) -> Optional[ControllerMetadata]:
    """
    Obtiene la metadata de un controller.
    
    Args:
        cls: Clase del controller
        
    Returns:
        ControllerMetadata si la clase es un controller, None en caso contrario
        
    Example:
        @controller("/users")
        class UserController:
            pass
        
        metadata = get_controller_metadata(UserController)
        print(metadata.base_path)  # "/users"
    """
    if not is_controller(cls):
        return None
    
    return getattr(cls, _CONTROLLER_METADATA_ATTR)


def get_controller_base_path(cls: Type) -> Optional[str]:
    """
    Obtiene la ruta base de un controller.
    
    Args:
        cls: Clase del controller
        
    Returns:
        Base path o None si no es controller
    """
    metadata = get_controller_metadata(cls)
    return metadata.base_path if metadata else None


def get_controller_full_path(cls: Type) -> Optional[str]:
    """
    Obtiene la ruta completa de un controller (prefix + base_path).
    
    Args:
        cls: Clase del controller
        
    Returns:
        Full path o None si no es controller
    """
    metadata = get_controller_metadata(cls)
    return metadata.get_full_path() if metadata else None


def get_controller_tags(cls: Type) -> List[str]:
    """
    Obtiene los tags de un controller.
    
    Args:
        cls: Clase del controller
        
    Returns:
        Lista de tags o lista vacía si no es controller
    """
    metadata = get_controller_metadata(cls)
    return metadata.tags if metadata else []


# ============================================================================
# CONTROLLER REGISTRY
# ============================================================================

# Registry global: controller_class -> ControllerMetadata
_controller_registry: Dict[Type, ControllerMetadata] = {}


def register_controller(controller_cls: Type, metadata: ControllerMetadata) -> None:
    """
    Registra un controller en el registry global.
    
    Args:
        controller_cls: Clase del controller
        metadata: Metadata del controller
        
    Example:
        metadata = ControllerMetadata(base_path="/users")
        register_controller(UserController, metadata)
    """
    _controller_registry[controller_cls] = metadata


def get_controller(controller_cls: Type) -> Optional[ControllerMetadata]:
    """
    Obtiene la metadata de un controller desde el registry.
    
    Args:
        controller_cls: Clase del controller
        
    Returns:
        ControllerMetadata si el controller está registrado, None en caso contrario
        
    Example:
        metadata = get_controller(UserController)
        if metadata:
            print(metadata.base_path)
    """
    return _controller_registry.get(controller_cls)


def get_all_controllers() -> Dict[Type, ControllerMetadata]:
    """
    Obtiene todos los controllers registrados.
    
    Returns:
        Diccionario de controller_class -> ControllerMetadata
        
    Example:
        all_controllers = get_all_controllers()
        for controller_cls, metadata in all_controllers.items():
            print(f"Controller: {controller_cls.__name__} -> {metadata.base_path}")
    """
    return _controller_registry.copy()


def clear_controller_registry() -> None:
    """
    Limpia el registry de controllers (útil para tests).
    
    Example:
        clear_controller_registry()
        assert len(get_all_controllers()) == 0
    """
    _controller_registry.clear()


def find_controller_by_path(path: str) -> Optional[Type]:
    """
    Encuentra el controller que maneja una ruta específica.
    
    Prioriza paths más específicos sobre paths generales (ej: "/api/users" antes que "/").
    
    Args:
        path: Ruta a buscar (ej: "/api/users/123")
        
    Returns:
        Clase del controller que maneja la ruta, None si no se encuentra
        
    Example:
        controller = find_controller_by_path("/api/users/123")
        if controller:
            print(f"Handler: {controller.__name__}")
            
    Note:
        Esta es una búsqueda simple por prefix matching con prioridad por longitud.
        Un router completo haría path matching con parámetros.
    """
    # Normalizar path
    if not path.startswith('/'):
        path = f"/{path}"
    
    # Recolectar todos los matches con su longitud de path
    matches = []
    
    for controller_cls, metadata in _controller_registry.items():
        full_path = metadata.get_full_path()
        
        # Exact match
        if path == full_path:
            matches.append((controller_cls, len(full_path)))
        
        # Prefix match (path empieza con full_path)
        elif path.startswith(full_path + "/"):
            matches.append((controller_cls, len(full_path)))
        
        # Root controller match (solo si no hay otros matches)
        elif full_path == "/" and path != "/":
            matches.append((controller_cls, len(full_path)))
    
    # Si no hay matches, retornar None
    if not matches:
        return None
    
    # Ordenar por longitud de path (más largo = más específico) y retornar el primero
    matches.sort(key=lambda x: x[1], reverse=True)
    return matches[0][0]


def get_controllers_by_tag(tag: str) -> List[Type]:
    """
    Obtiene todos los controllers con un tag específico.
    
    Args:
        tag: Tag a buscar
        
    Returns:
        Lista de clases de controllers con el tag
        
    Example:
        user_controllers = get_controllers_by_tag("Users")
        for controller in user_controllers:
            print(controller.__name__)
    """
    result = []
    for controller_cls, metadata in _controller_registry.items():
        if tag in metadata.tags:
            result.append(controller_cls)
    return result


# ============================================================================
# TESTS BÁSICOS (para verificación)
# ============================================================================

if __name__ == "__main__":
    # Test 1: Crear controller básico
    @controller("/users")
    class UserController:
        pass
    
    assert is_controller(UserController), "UserController debe ser controller"
    metadata = get_controller_metadata(UserController)
    assert metadata is not None, "Metadata no debe ser None"
    assert metadata.base_path == "/users", "base_path debe ser /users"
    
    # Test 2: Controller con prefix
    @controller("/products", prefix="/api")
    class ProductController:
        pass
    
    metadata = get_controller_metadata(ProductController)
    assert metadata.get_full_path() == "/api/products", "Full path debe ser /api/products"
    
    # Test 3: Controller con tags
    @controller("/orders", tags=["Orders", "Payments"])
    class OrderController:
        pass
    
    tags = get_controller_tags(OrderController)
    assert "Orders" in tags, "Debe tener tag Orders"
    assert "Payments" in tags, "Debe tener tag Payments"
    
    # Test 4: Registry
    assert UserController in get_all_controllers(), "UserController debe estar en registry"
    assert get_controller(UserController) == get_controller_metadata(UserController)
    
    # Test 5: find_controller_by_path
    found = find_controller_by_path("/api/products/123")
    assert found == ProductController, "Debe encontrar ProductController"
    
    # Test 6: get_controllers_by_tag
    order_controllers = get_controllers_by_tag("Orders")
    assert OrderController in order_controllers, "Debe encontrar OrderController"
    
    print("✅ Todos los tests básicos pasaron")
