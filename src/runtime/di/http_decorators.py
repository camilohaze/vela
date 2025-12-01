"""
HTTP Decorators para Vela DI System

Este módulo implementa decoradores HTTP (@get, @post, @put, @patch, @delete)
para configurar rutas y métodos HTTP en controllers.

En Vela, estos decoradores configuran MÉTODOS de controllers con metadata HTTP:
- Método HTTP (GET, POST, PUT, etc.)
- Path del endpoint
- Parámetros de ruta, query, body, headers
- Middleware específico del endpoint

Example en Vela:
    @controller("/users", prefix="api")
    controller UserController {
        service: UserService = inject(UserService)
        
        @get("/:id")
        fn getUser(id: Number) -> Result<User> {
            return this.service.findById(id)
        }
        
        @post("/")
        @validate
        fn createUser(@body dto: CreateUserDTO) -> Result<User> {
            return this.service.create(dto)
        }
        
        @delete("/:id")
        fn deleteUser(id: Number) -> Result<void> {
            return this.service.delete(id)
        }
    }

Author: Vela Team
Version: 0.1.0
Created: 2025-12-01
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Type, get_type_hints
from functools import wraps
import inspect


# ================================
# HTTPMethod Enum
# ================================

class HTTPMethod(str, Enum):
    """
    Métodos HTTP soportados en Vela.
    
    Los valores string permiten usar directamente en requests HTTP.
    """
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    PATCH = "PATCH"
    DELETE = "DELETE"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"
    
    def __str__(self) -> str:
        return self.value


# ================================
# Parameter Types
# ================================

class ParameterType(str, Enum):
    """
    Tipos de parámetros que se pueden decorar en métodos HTTP.
    
    - PATH: Parámetro de ruta (/:id)
    - QUERY: Parámetro de query string (?page=1)
    - BODY: Cuerpo de la request (JSON)
    - HEADER: Header HTTP
    - COOKIE: Cookie
    """
    PATH = "path"
    QUERY = "query"
    BODY = "body"
    HEADER = "header"
    COOKIE = "cookie"


# ================================
# RouteMetadata Dataclass
# ================================

@dataclass
class RouteMetadata:
    """
    Metadata de una ruta HTTP en un controller.
    
    Almacena información sobre el endpoint:
    - Método HTTP (GET, POST, etc.)
    - Path del endpoint relativo al controller
    - Parámetros esperados (path, query, body, headers)
    - Middleware específico
    - Validadores
    
    Example:
        metadata = RouteMetadata(
            method=HTTPMethod.GET,
            path="/:id",
            path_params={"id": int},
            description="Get user by ID"
        )
    """
    method: HTTPMethod
    path: str = "/"
    path_params: Dict[str, Type] = field(default_factory=dict)
    query_params: Dict[str, Type] = field(default_factory=dict)
    body_type: Optional[Type] = None
    headers: Dict[str, str] = field(default_factory=dict)
    middleware: List[Callable] = field(default_factory=list)
    description: Optional[str] = None
    tags: List[str] = field(default_factory=list)
    
    def __post_init__(self):
        """Normaliza path."""
        if not self.path.startswith('/'):
            self.path = f"/{self.path}"
        
        # Convertir tags a lista
        if isinstance(self.tags, str):
            self.tags = [self.tags]
        elif self.tags is None:
            self.tags = []
    
    def get_full_route(self, controller_path: str) -> str:
        """
        Combina el path del controller con el path del endpoint.
        
        Args:
            controller_path: Path del controller (ej: "/api/users")
            
        Returns:
            Path completo (ej: "/api/users/:id")
            
        Example:
            controller_path="/api/users", path="/:id" → "/api/users/:id"
            controller_path="/api/users", path="/" → "/api/users"
        """
        if self.path == '/':
            return controller_path
        
        if controller_path.endswith('/'):
            controller_path = controller_path[:-1]
        
        return f"{controller_path}{self.path}"


# ================================
# Parameter Metadata
# ================================

@dataclass
class ParameterMetadata:
    """
    Metadata de un parámetro en un método HTTP.
    
    Describe cómo extraer y validar un parámetro de la request.
    """
    name: str
    param_type: ParameterType
    expected_type: Type
    required: bool = True
    default: Any = None
    description: Optional[str] = None
    
    def __post_init__(self):
        if self.default is not None:
            self.required = False


# ================================
# HTTP Method Decorators
# ================================

def _http_method_decorator(method: HTTPMethod):
    """
    Factory para crear decoradores HTTP (@get, @post, etc.).
    
    Args:
        method: Método HTTP (GET, POST, etc.)
        
    Returns:
        Función decoradora que acepta path y opciones
    """
    def decorator(
        path: str = "/",
        middleware: Optional[List[Callable]] = None,
        tags: Optional[List[str]] = None,
        description: Optional[str] = None
    ):
        """
        Decorador para configurar método HTTP.
        
        Args:
            path: Path relativo al controller (ej: "/:id")
            middleware: Lista de middleware específico del endpoint
            tags: Tags para categorización
            description: Descripción del endpoint
        """
        def method_decorator(func: Callable) -> Callable:
            # Extraer parámetros del método
            sig = inspect.signature(func)
            type_hints = get_type_hints(func) if hasattr(func, '__annotations__') else {}
            
            path_params = {}
            query_params = {}
            body_type = None
            
            # Detectar parámetros de la función
            for param_name, param in sig.parameters.items():
                if param_name == 'self':
                    continue
                
                # Obtener metadata de parámetro si existe
                param_metadata = getattr(param.default, '__parameter_metadata__', None)
                
                if param_metadata:
                    if param_metadata.param_type == ParameterType.BODY:
                        body_type = param_metadata.expected_type
                    elif param_metadata.param_type == ParameterType.QUERY:
                        query_params[param_name] = param_metadata.expected_type
                    elif param_metadata.param_type == ParameterType.PATH:
                        path_params[param_name] = param_metadata.expected_type
                else:
                    # Inferir de type hints si no hay decorator
                    if param_name in type_hints:
                        # Por defecto, parámetros simples son path params
                        path_params[param_name] = type_hints[param_name]
            
            # Crear metadata de ruta
            metadata = RouteMetadata(
                method=method,
                path=path,
                path_params=path_params,
                query_params=query_params,
                body_type=body_type,
                middleware=middleware or [],
                tags=tags or [],
                description=description
            )
            
            # Agregar metadata al método
            setattr(func, '__route_metadata__', metadata)
            
            @wraps(func)
            def wrapper(*args, **kwargs):
                # Aquí el runtime ejecutará middleware, validación, etc.
                return func(*args, **kwargs)
            
            return wrapper
        
        return method_decorator
    
    return decorator


# Crear decoradores HTTP específicos
get = _http_method_decorator(HTTPMethod.GET)
post = _http_method_decorator(HTTPMethod.POST)
put = _http_method_decorator(HTTPMethod.PUT)
patch = _http_method_decorator(HTTPMethod.PATCH)
delete = _http_method_decorator(HTTPMethod.DELETE)
head = _http_method_decorator(HTTPMethod.HEAD)
options = _http_method_decorator(HTTPMethod.OPTIONS)


# ================================
# Parameter Decorators
# ================================

class ParameterMarker:
    """
    Marker class para decoradores de parámetros.
    
    Se usa como default value en parámetros decorados para
    transportar metadata del parámetro.
    """
    def __init__(self, metadata: ParameterMetadata):
        self.__parameter_metadata__ = metadata


def param(
    name: Optional[str] = None,
    param_type: Type = str,
    required: bool = True,
    default: Any = None,
    description: Optional[str] = None
):
    """
    Decorador para parámetros de PATH.
    
    Args:
        name: Nombre del parámetro (si difiere del nombre del argumento)
        param_type: Tipo esperado del parámetro
        required: Si el parámetro es requerido
        default: Valor por defecto
        description: Descripción del parámetro
        
    Returns:
        ParameterMarker con metadata
        
    Example en Vela:
        @get("/:id")
        fn getUser(@param id: Number) -> Result<User> {
            return this.service.findById(id)
        }
    """
    metadata = ParameterMetadata(
        name=name or "",
        param_type=ParameterType.PATH,
        expected_type=param_type,
        required=required,
        default=default,
        description=description
    )
    return ParameterMarker(metadata)


def query(
    name: Optional[str] = None,
    param_type: Type = str,
    required: bool = False,
    default: Any = None,
    description: Optional[str] = None
):
    """
    Decorador para parámetros de QUERY string.
    
    Example en Vela:
        @get("/")
        fn listUsers(@query page: Number = 1, @query limit: Number = 10) -> Result<List<User>> {
            return this.service.list(page, limit)
        }
    """
    metadata = ParameterMetadata(
        name=name or "",
        param_type=ParameterType.QUERY,
        expected_type=param_type,
        required=required,
        default=default,
        description=description
    )
    return ParameterMarker(metadata)


def body(
    body_type: Type,
    required: bool = True,
    description: Optional[str] = None
):
    """
    Decorador para BODY de la request.
    
    Example en Vela:
        @post("/")
        fn createUser(@body dto: CreateUserDTO) -> Result<User> {
            return this.service.create(dto)
        }
    """
    metadata = ParameterMetadata(
        name="body",
        param_type=ParameterType.BODY,
        expected_type=body_type,
        required=required,
        description=description
    )
    return ParameterMarker(metadata)


def header(
    name: str,
    param_type: Type = str,
    required: bool = False,
    default: Any = None,
    description: Optional[str] = None
):
    """
    Decorador para HEADERS HTTP.
    
    Example en Vela:
        @get("/protected")
        fn getProtected(@header("Authorization") token: String) -> Result<Data> {
            return this.service.getProtected(token)
        }
    """
    metadata = ParameterMetadata(
        name=name,
        param_type=ParameterType.HEADER,
        expected_type=param_type,
        required=required,
        default=default,
        description=description
    )
    return ParameterMarker(metadata)


def cookie(
    name: str,
    param_type: Type = str,
    required: bool = False,
    default: Any = None,
    description: Optional[str] = None
):
    """
    Decorador para COOKIES HTTP.
    
    Example en Vela:
        @get("/profile")
        fn getProfile(@cookie("session_id") sessionId: String) -> Result<User> {
            return this.service.getUserFromSession(sessionId)
        }
    """
    metadata = ParameterMetadata(
        name=name,
        param_type=ParameterType.COOKIE,
        expected_type=param_type,
        required=required,
        default=default,
        description=description
    )
    return ParameterMarker(metadata)


def request(
    description: Optional[str] = None
):
    """
    Decorador para inyectar objeto REQUEST completo.
    
    Permite acceso al objeto Request completo con headers, body, query params, etc.
    
    Example en Vela:
        @get("/:id")
        fn getUser(@request req: Request, id: Number) -> Result<User> {
            userAgent = req.headers.get("User-Agent")
            ip = req.ip
            return this.service.findById(id)
        }
    """
    # Request es un tipo especial que se inyecta en runtime
    # No tiene "name" porque se pasa como objeto completo
    metadata = ParameterMetadata(
        name="__request__",  # Nombre interno
        param_type=ParameterType.BODY,  # Usar BODY como categoria (se sobreescribe en runtime)
        expected_type=object,  # Request type (se determina en runtime)
        required=True,
        description=description
    )
    return ParameterMarker(metadata)


def response(
    description: Optional[str] = None
):
    """
    Decorador para inyectar objeto RESPONSE completo.
    
    Permite acceso al objeto Response para manipulación directa
    (headers, status code, streaming, etc).
    
    Example en Vela:
        @get("/download/:id")
        fn download(@response res: Response, id: Number) -> void {
            file = this.service.getFile(id)
            res.setHeader("Content-Type", file.mimeType)
            res.sendFile(file.path)
        }
    """
    # Response es un tipo especial que se inyecta en runtime
    metadata = ParameterMetadata(
        name="__response__",  # Nombre interno
        param_type=ParameterType.BODY,  # Usar BODY como categoria (se sobreescribe en runtime)
        expected_type=object,  # Response type (se determina en runtime)
        required=True,
        description=description
    )
    return ParameterMarker(metadata)


# ================================
# Helper Functions
# ================================

def is_route_handler(func: Callable) -> bool:
    """
    Verifica si una función es un route handler (tiene @get, @post, etc.).
    
    Args:
        func: Función a verificar
        
    Returns:
        True si tiene metadata de ruta
    """
    return hasattr(func, '__route_metadata__')


def get_route_metadata(func: Callable) -> Optional[RouteMetadata]:
    """
    Obtiene metadata de ruta de una función.
    
    Args:
        func: Función decorada
        
    Returns:
        RouteMetadata o None si no está decorada
    """
    return getattr(func, '__route_metadata__', None)


def get_all_routes(controller_cls: Type) -> Dict[str, RouteMetadata]:
    """
    Obtiene todas las rutas definidas en un controller.
    
    Args:
        controller_cls: Clase del controller
        
    Returns:
        Dict con nombre del método → RouteMetadata
        
    Example:
        routes = get_all_routes(UserController)
        # {'getUser': RouteMetadata(...), 'createUser': RouteMetadata(...)}
    """
    routes = {}
    
    for name, method in inspect.getmembers(controller_cls, predicate=inspect.isfunction):
        metadata = get_route_metadata(method)
        if metadata:
            routes[name] = metadata
    
    return routes


def get_routes_by_method(controller_cls: Type, http_method: HTTPMethod) -> Dict[str, RouteMetadata]:
    """
    Obtiene rutas filtradas por método HTTP.
    
    Args:
        controller_cls: Clase del controller
        http_method: Método HTTP a filtrar (GET, POST, etc.)
        
    Returns:
        Dict con rutas que usan ese método
        
    Example:
        get_routes = get_routes_by_method(UserController, HTTPMethod.GET)
    """
    all_routes = get_all_routes(controller_cls)
    return {
        name: metadata
        for name, metadata in all_routes.items()
        if metadata.method == http_method
    }


def get_route_by_path(controller_cls: Type, path: str) -> Optional[tuple[str, RouteMetadata]]:
    """
    Busca una ruta por su path.
    
    Args:
        controller_cls: Clase del controller
        path: Path a buscar
        
    Returns:
        Tupla (nombre_metodo, RouteMetadata) o None
    """
    all_routes = get_all_routes(controller_cls)
    
    for name, metadata in all_routes.items():
        if metadata.path == path:
            return (name, metadata)
    
    return None


# ================================
# Tests Inline (para desarrollo rápido)
# ================================

if __name__ == "__main__":
    print("=" * 60)
    print("HTTP Decorators - Tests Inline")
    print("=" * 60)
    
    # Test 1: HTTPMethod enum
    print("\n[Test 1] HTTPMethod enum")
    assert HTTPMethod.GET.value == "GET"
    assert str(HTTPMethod.POST) == "POST"
    print("✅ HTTPMethod enum funciona correctamente")
    
    # Test 2: RouteMetadata
    print("\n[Test 2] RouteMetadata normalization")
    metadata = RouteMetadata(method=HTTPMethod.GET, path="users")
    assert metadata.path == "/users"
    print("✅ Path normalizado correctamente")
    
    # Test 3: get_full_route
    print("\n[Test 3] get_full_route")
    metadata = RouteMetadata(method=HTTPMethod.GET, path="/:id")
    full_route = metadata.get_full_route("/api/users")
    assert full_route == "/api/users/:id"
    print(f"✅ Full route: {full_route}")
    
    # Test 4: @get decorator
    print("\n[Test 4] @get decorator")
    
    @get("/:id", description="Get user by ID")
    def getUser(id: int):
        return {"id": id}
    
    assert is_route_handler(getUser)
    route_meta = get_route_metadata(getUser)
    assert route_meta.method == HTTPMethod.GET
    assert route_meta.path == "/:id"
    print(f"✅ @get decorator funciona: {route_meta}")
    
    # Test 5: @post con body
    print("\n[Test 5] @post con @body")
    
    @post("/", description="Create user")
    def createUser(dto=body(dict)):
        return {"created": True}
    
    route_meta = get_route_metadata(createUser)
    assert route_meta.method == HTTPMethod.POST
    assert route_meta.body_type == dict
    print(f"✅ @post con body: {route_meta}")
    
    # Test 6: Controller completo
    print("\n[Test 6] Controller completo")
    
    class UserController:
        @get("/")
        def list(self):
            return []
        
        @get("/:id")
        def get(self, id: int):
            return {"id": id}
        
        @post("/")
        def create(self, dto=body(dict)):
            return {"created": True}
        
        @delete("/:id")
        def delete(self, id: int):
            return {"deleted": True}
    
    routes = get_all_routes(UserController)
    assert len(routes) == 4
    print(f"✅ Controller con {len(routes)} rutas")
    
    get_routes = get_routes_by_method(UserController, HTTPMethod.GET)
    assert len(get_routes) == 2
    print(f"✅ {len(get_routes)} rutas GET")
    
    # Test 7: Parameter decorators
    print("\n[Test 7] Parameter decorators")
    
    @get("/search")
    def search(q=query("query", str, required=True), page=query("page", int, default=1)):
        return {"q": q, "page": page}
    
    route_meta = get_route_metadata(search)
    assert "q" in route_meta.query_params or "page" in route_meta.query_params
    print(f"✅ Parameter decorators funcionan")
    
    print("\n" + "=" * 60)
    print("✅ TODOS LOS TESTS PASARON")
    print("=" * 60)
