"""
HTTP Router with Radix Tree

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Router HTTP con:
- Radix tree (prefix tree) para O(log n) route matching
- Route registration por HTTP method
- Path parameter extraction automática
- Query parameter parsing
- Middleware pipeline integration
- Route grouping y prefixes

Inspirado en:
- Gin (Go): Radix tree router
- Echo (Go): High-performance router
- Express.js: Route registration API
- NestJS: Decorator-based routing
"""

from typing import Dict, List, Optional, Callable
from dataclasses import dataclass, field
from .request import Request, HttpMethod, parse_query_string
from .response import Response, not_found
from .route import Route, RouteMatch
from .middleware import Middleware, MiddlewareChain


@dataclass
class RouteNode:
    """
    Nodo del radix tree para routing.
    
    Cada nodo representa un segmento del path:
    - Static segment: "users"
    - Parameter segment: ":id"
    - Wildcard segment: "*path"
    """
    segment: str = ""
    is_param: bool = False
    is_wildcard: bool = False
    param_name: Optional[str] = None
    routes: Dict[HttpMethod, Route] = field(default_factory=dict)  # Routes por method
    children: Dict[str, 'RouteNode'] = field(default_factory=dict)
    param_child: Optional['RouteNode'] = None  # Child para :param
    wildcard_child: Optional['RouteNode'] = None  # Child para *wildcard
    
    def is_terminal(self) -> bool:
        """Check if this node has routes (is endpoint)."""
        return len(self.routes) > 0


class Router:
    """
    HTTP Router con radix tree para route matching eficiente.
    
    Performance:
    - Route registration: O(n) donde n = longitud del path
    - Route matching: O(log n) donde n = número de rutas
    - Memory: O(m) donde m = número total de segmentos únicos
    
    Features:
    - Path parameters: /users/:id
    - Query parameters: /users?page=1&limit=10
    - Wildcard: /files/*path
    - Middleware: global y por ruta
    - Route groups: prefijos compartidos
    """
    
    def __init__(self):
        """Initialize router with empty radix tree."""
        self.root = RouteNode()
        self.global_middleware: List[Middleware] = []
        self.route_count = 0
    
    def register(self, route: Route) -> None:
        """
        Register route in the radix tree.
        
        Args:
            route: Route to register
            
        Raises:
            ValueError: If route already exists
            
        Example:
            router.register(Route(HttpMethod.GET, "/users/:id", handler))
        """
        # Parse path into segments
        path = route.path.strip('/')
        segments = path.split('/') if path else []
        
        # Navigate/create tree nodes
        current = self.root
        
        for segment in segments:
            if segment.startswith(':'):
                # Parameter segment
                param_name = segment[1:]
                if current.param_child is None:
                    current.param_child = RouteNode(
                        segment=segment,
                        is_param=True,
                        param_name=param_name
                    )
                current = current.param_child
                
            elif segment.startswith('*'):
                # Wildcard segment (must be last)
                param_name = segment[1:]
                if current.wildcard_child is None:
                    current.wildcard_child = RouteNode(
                        segment=segment,
                        is_wildcard=True,
                        param_name=param_name
                    )
                current = current.wildcard_child
                
            else:
                # Static segment
                if segment not in current.children:
                    current.children[segment] = RouteNode(segment=segment)
                current = current.children[segment]
        
        # Register route at terminal node
        if route.method in current.routes:
            raise ValueError(
                f"Route already exists: {route.method.value} {route.path}"
            )
        
        current.routes[route.method] = route
        self.route_count += 1
    
    def match(self, method: HttpMethod, path: str) -> Optional[RouteMatch]:
        """
        Match route in radix tree.
        
        Priority:
        1. Static segments (exact match)
        2. Parameter segments (:id)
        3. Wildcard segments (*path)
        
        Args:
            method: HTTP method
            path: Request path
            
        Returns:
            RouteMatch if found, None otherwise
            
        Example:
            match = router.match(HttpMethod.GET, "/users/123")
            # → RouteMatch(matched=True, params={"id": "123"})
        """
        path = path.strip('/')
        segments = path.split('/') if path else []
        
        params: Dict[str, str] = {}
        current = self.root
        
        for i, segment in enumerate(segments):
            matched = False
            
            # Priority 1: Static segment (exact match)
            if segment in current.children:
                current = current.children[segment]
                matched = True
            
            # Priority 2: Parameter segment
            elif current.param_child is not None:
                params[current.param_child.param_name] = segment
                current = current.param_child
                matched = True
            
            # Priority 3: Wildcard segment (rest of path)
            elif current.wildcard_child is not None:
                # Wildcard captures rest of path
                remaining = '/'.join(segments[i:])
                params[current.wildcard_child.param_name] = remaining
                current = current.wildcard_child
                matched = True
                break  # Wildcard consumes rest
            
            if not matched:
                return None  # No match found
        
        # Check if current node has route for method
        if method not in current.routes:
            return None
        
        route = current.routes[method]
        return RouteMatch(matched=True, params=params, route=route)
    
    def handle(self, method: HttpMethod, path: str, query_string: str = "") -> Response:
        """
        Handle HTTP request.
        
        Steps:
        1. Parse query parameters
        2. Match route
        3. Extract path parameters
        4. Build Request object
        5. Execute middleware chain + handler
        6. Return Response
        
        Args:
            method: HTTP method
            path: Request path
            query_string: Query string (without ?)
            
        Returns:
            HTTP response
            
        Example:
            response = router.handle(HttpMethod.GET, "/users/123", "page=1&limit=10")
        """
        # Parse query parameters
        query = parse_query_string(query_string)
        
        # Match route
        match = self.match(method, path)
        
        if match is None:
            return not_found(f"Route not found: {method.value} {path}")
        
        # Build request
        request = Request(
            method=method,
            path=path,
            params=match.params,
            query=query
        )
        
        # Build middleware chain
        all_middleware = self.global_middleware + match.route.middleware
        chain = MiddlewareChain(all_middleware, match.route.handler)
        
        # Execute chain
        try:
            return chain.execute(request)
        except Exception as e:
            # Unhandled error
            from .response import internal_server_error
            return internal_server_error(f"Internal error: {e}")
    
    def use(self, middleware: Middleware) -> None:
        """
        Add global middleware.
        
        Global middleware ejecuta antes de todas las rutas.
        
        Args:
            middleware: Middleware to add
            
        Example:
            router.use(LoggingMiddleware())
            router.use(AuthMiddleware())
        """
        self.global_middleware.append(middleware)
    
    def get(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """
        Register GET route.
        
        Args:
            path: Path pattern
            handler: Handler function
            middleware: Route-specific middleware
            
        Example:
            router.get("/users/:id", get_user_handler)
        """
        route = Route(HttpMethod.GET, path, handler, middleware)
        self.register(route)
    
    def post(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """
        Register POST route.
        
        Args:
            path: Path pattern
            handler: Handler function
            middleware: Route-specific middleware
            
        Example:
            router.post("/users", create_user_handler)
        """
        route = Route(HttpMethod.POST, path, handler, middleware)
        self.register(route)
    
    def put(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """
        Register PUT route.
        
        Args:
            path: Path pattern
            handler: Handler function
            middleware: Route-specific middleware
            
        Example:
            router.put("/users/:id", update_user_handler)
        """
        route = Route(HttpMethod.PUT, path, handler, middleware)
        self.register(route)
    
    def delete(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """
        Register DELETE route.
        
        Args:
            path: Path pattern
            handler: Handler function
            middleware: Route-specific middleware
            
        Example:
            router.delete("/users/:id", delete_user_handler)
        """
        route = Route(HttpMethod.DELETE, path, handler, middleware)
        self.register(route)
    
    def patch(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """
        Register PATCH route.
        
        Args:
            path: Path pattern
            handler: Handler function
            middleware: Route-specific middleware
            
        Example:
            router.patch("/users/:id", partial_update_user_handler)
        """
        route = Route(HttpMethod.PATCH, path, handler, middleware)
        self.register(route)
    
    def group(self, prefix: str, middleware: List[Middleware] = None) -> 'RouteGroup':
        """
        Create route group with shared prefix and middleware.
        
        Args:
            prefix: Shared prefix for all routes
            middleware: Shared middleware
            
        Returns:
            RouteGroup instance
            
        Example:
            api_v1 = router.group("/api/v1", [AuthMiddleware()])
            api_v1.get("/users", list_users_handler)
            # → /api/v1/users with AuthMiddleware
        """
        return RouteGroup(self, prefix, middleware or [])
    
    def print_tree(self, node: Optional[RouteNode] = None, indent: int = 0) -> None:
        """
        Print radix tree structure (for debugging).
        
        Args:
            node: Node to print (default: root)
            indent: Indentation level
        """
        if node is None:
            node = self.root
            print("Router Tree:")
        
        prefix = "  " * indent
        
        if node.is_param:
            print(f"{prefix}:param ({node.param_name})")
        elif node.is_wildcard:
            print(f"{prefix}*wildcard ({node.param_name})")
        elif node.segment:
            print(f"{prefix}{node.segment}")
        
        if node.routes:
            for method, route in node.routes.items():
                print(f"{prefix}  → {method.value} handler")
        
        for child in node.children.values():
            self.print_tree(child, indent + 1)
        
        if node.param_child:
            self.print_tree(node.param_child, indent + 1)
        
        if node.wildcard_child:
            self.print_tree(node.wildcard_child, indent + 1)


class RouteGroup:
    """
    Route group con shared prefix y middleware.
    
    Permite organizar rutas con prefijos comunes:
    - /api/v1/users
    - /api/v1/posts
    - /api/v1/comments
    
    Example:
        api_v1 = router.group("/api/v1", [AuthMiddleware()])
        api_v1.get("/users", list_users)
        api_v1.post("/users", create_user)
    """
    
    def __init__(self, router: Router, prefix: str, middleware: List[Middleware]):
        """
        Initialize route group.
        
        Args:
            router: Parent router
            prefix: Shared prefix
            middleware: Shared middleware
        """
        self.router = router
        self.prefix = prefix.rstrip('/')
        self.middleware = middleware
    
    def _full_path(self, path: str) -> str:
        """Build full path with prefix."""
        return f"{self.prefix}{path}"
    
    def get(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """Register GET route in group."""
        all_middleware = self.middleware + (middleware or [])
        self.router.get(self._full_path(path), handler, all_middleware)
    
    def post(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """Register POST route in group."""
        all_middleware = self.middleware + (middleware or [])
        self.router.post(self._full_path(path), handler, all_middleware)
    
    def put(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """Register PUT route in group."""
        all_middleware = self.middleware + (middleware or [])
        self.router.put(self._full_path(path), handler, all_middleware)
    
    def delete(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """Register DELETE route in group."""
        all_middleware = self.middleware + (middleware or [])
        self.router.delete(self._full_path(path), handler, all_middleware)
    
    def patch(self, path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> None:
        """Register PATCH route in group."""
        all_middleware = self.middleware + (middleware or [])
        self.router.patch(self._full_path(path), handler, all_middleware)


if __name__ == "__main__":
    from .response import ok
    from .middleware import LoggingMiddleware
    
    # Create router
    router = Router()
    
    # Add global middleware
    router.use(LoggingMiddleware())
    
    # Register routes
    def list_users(req: Request) -> Response:
        return ok({"users": ["Alice", "Bob"]})
    
    def get_user(req: Request) -> Response:
        return ok({"id": req.get_param('id'), "name": "Alice"})
    
    def create_user(req: Request) -> Response:
        return ok({"id": 123, "name": "New User"})
    
    router.get("/users", list_users)
    router.get("/users/:id", get_user)
    router.post("/users", create_user)
    
    # Test routing
    print("\n=== Test Routing ===")
    response1 = router.handle(HttpMethod.GET, "/users")
    print(f"GET /users → {response1.status} {response1.body}")
    
    response2 = router.handle(HttpMethod.GET, "/users/123")
    print(f"GET /users/123 → {response2.status} {response2.body}")
    
    response3 = router.handle(HttpMethod.POST, "/users")
    print(f"POST /users → {response3.status} {response3.body}")
    
    response4 = router.handle(HttpMethod.GET, "/users/123", "page=1&limit=10")
    print(f"GET /users/123?page=1 → {response4.status}")
    print(f"Query: {response4.body}")
    
    # Print tree
    print("\n=== Router Tree ===")
    router.print_tree()
