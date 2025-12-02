"""
HTTP Route Class

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Clase Route para representar una ruta HTTP con:
- Path pattern con parámetros (:id, :name)
- HTTP method
- Handler function
- Middleware chain
- Path parameter extraction
"""

from dataclasses import dataclass, field
from typing import Callable, Optional, Dict, Any, List, Tuple
import re
from .request import Request, HttpMethod
from .response import Response
from .middleware import Middleware


@dataclass
class RouteMatch:
    """
    Resultado del matching de una ruta.
    
    Contiene:
    - matched: True si la ruta coincide
    - params: Path parameters extraídos
    - route: Route que coincidió
    """
    matched: bool
    params: Dict[str, str] = field(default_factory=dict)
    route: Optional['Route'] = None


class Route:
    """
    Route con path pattern, HTTP method y handler.
    
    Inspirado en:
    - Express.js: app.get('/users/:id', handler)
    - NestJS: @Get(':id')
    - FastAPI: @app.get('/users/{id}')
    - Spring Boot: @GetMapping("/users/{id}")
    
    Soporta:
    - Static segments: /users/list
    - Parameter segments: /users/:id
    - Wildcard: /files/*path
    - Multiple params: /posts/:postId/comments/:commentId
    """
    
    def __init__(
        self,
        method: HttpMethod,
        path: str,
        handler: Callable[[Request], Response],
        middleware: List[Middleware] = None
    ):
        """
        Initialize route.
        
        Args:
            method: HTTP method
            path: Path pattern (e.g., "/users/:id")
            handler: Handler function
            middleware: Middleware list (optional)
        """
        self.method = method
        self.path = path
        self.handler = handler
        self.middleware = middleware or []
        
        # Parse path pattern
        self.segments, self.param_names = self._parse_pattern(path)
        
        # Compile regex for matching
        self.regex = self._compile_regex()
    
    def _parse_pattern(self, path: str) -> Tuple[List[str], List[str]]:
        """
        Parse path pattern into segments y extract param names.
        
        Args:
            path: Path pattern (e.g., "/users/:id/posts/:postId")
            
        Returns:
            Tuple of (segments, param_names)
            
        Example:
            _parse_pattern("/users/:id/posts/:postId")
            # → (["users", ":id", "posts", ":postId"], ["id", "postId"])
        """
        # Remove leading/trailing slashes
        path = path.strip('/')
        
        if not path:
            return [], []
        
        segments = path.split('/')
        param_names = []
        
        for segment in segments:
            if segment.startswith(':'):
                # Parameter segment: :id → id
                param_names.append(segment[1:])
            elif segment.startswith('*'):
                # Wildcard segment: *path → path
                param_names.append(segment[1:])
        
        return segments, param_names
    
    def _compile_regex(self) -> re.Pattern:
        """
        Compile path pattern to regex for matching.
        
        Rules:
        - Static segment "users" → "users"
        - Param segment ":id" → "(?P<id>[^/]+)"
        - Wildcard "*path" → "(?P<path>.*)"
        
        Returns:
            Compiled regex pattern
            
        Example:
            Pattern: "/users/:id/posts/:postId"
            Regex: "^users/(?P<id>[^/]+)/posts/(?P<postId>[^/]+)$"
        """
        pattern_parts = []
        
        for segment in self.segments:
            if segment.startswith(':'):
                # Parameter segment: :id → capture group
                param_name = segment[1:]
                pattern_parts.append(f"(?P<{param_name}>[^/]+)")
            elif segment.startswith('*'):
                # Wildcard: *path → capture everything
                param_name = segment[1:]
                pattern_parts.append(f"(?P<{param_name}>.*)")
            else:
                # Static segment: users → literal match
                pattern_parts.append(re.escape(segment))
        
        # Join with / and add anchors
        pattern = "^" + "/".join(pattern_parts) + "$"
        return re.compile(pattern)
    
    def match(self, method: HttpMethod, path: str) -> RouteMatch:
        """
        Check if this route matches the given method and path.
        
        Args:
            method: HTTP method
            path: Request path
            
        Returns:
            RouteMatch with matched status and extracted params
            
        Example:
            route = Route(HttpMethod.GET, "/users/:id", handler)
            match = route.match(HttpMethod.GET, "/users/123")
            # → RouteMatch(matched=True, params={"id": "123"})
        """
        # Check method
        if self.method != method:
            return RouteMatch(matched=False)
        
        # Remove leading/trailing slashes
        path = path.strip('/')
        
        # Match regex
        m = self.regex.match(path)
        if not m:
            return RouteMatch(matched=False)
        
        # Extract params
        params = m.groupdict()
        
        return RouteMatch(matched=True, params=params, route=self)
    
    def __repr__(self) -> str:
        return f"Route({self.method.value} {self.path})"
    
    def __eq__(self, other: Any) -> bool:
        if not isinstance(other, Route):
            return False
        return self.method == other.method and self.path == other.path


# Helper functions para crear routes

def get(path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> Route:
    """
    Create GET route.
    
    Args:
        path: Path pattern
        handler: Handler function
        middleware: Middleware list (optional)
        
    Returns:
        Route instance
        
    Example:
        get("/users/:id", get_user_handler)
    """
    return Route(HttpMethod.GET, path, handler, middleware)


def post(path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> Route:
    """
    Create POST route.
    
    Args:
        path: Path pattern
        handler: Handler function
        middleware: Middleware list (optional)
        
    Returns:
        Route instance
        
    Example:
        post("/users", create_user_handler)
    """
    return Route(HttpMethod.POST, path, handler, middleware)


def put(path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> Route:
    """
    Create PUT route.
    
    Args:
        path: Path pattern
        handler: Handler function
        middleware: Middleware list (optional)
        
    Returns:
        Route instance
        
    Example:
        put("/users/:id", update_user_handler)
    """
    return Route(HttpMethod.PUT, path, handler, middleware)


def delete(path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> Route:
    """
    Create DELETE route.
    
    Args:
        path: Path pattern
        handler: Handler function
        middleware: Middleware list (optional)
        
    Returns:
        Route instance
        
    Example:
        delete("/users/:id", delete_user_handler)
    """
    return Route(HttpMethod.DELETE, path, handler, middleware)


def patch(path: str, handler: Callable[[Request], Response], middleware: List[Middleware] = None) -> Route:
    """
    Create PATCH route.
    
    Args:
        path: Path pattern
        handler: Handler function
        middleware: Middleware list (optional)
        
    Returns:
        Route instance
        
    Example:
        patch("/users/:id", partial_update_user_handler)
    """
    return Route(HttpMethod.PATCH, path, handler, middleware)


if __name__ == "__main__":
    from .response import ok
    
    # Test route matching
    def handler(req: Request) -> Response:
        return ok({"id": req.get_param('id')})
    
    route = Route(HttpMethod.GET, "/users/:id", handler)
    
    # Test matching
    match1 = route.match(HttpMethod.GET, "/users/123")
    print(f"Match 1: {match1.matched}, params: {match1.params}")  # → True, {id: "123"}
    
    match2 = route.match(HttpMethod.GET, "/users/abc")
    print(f"Match 2: {match2.matched}, params: {match2.params}")  # → True, {id: "abc"}
    
    match3 = route.match(HttpMethod.POST, "/users/123")
    print(f"Match 3: {match3.matched}")  # → False (wrong method)
    
    match4 = route.match(HttpMethod.GET, "/posts/123")
    print(f"Match 4: {match4.matched}")  # → False (wrong path)
    
    # Test multiple params
    route2 = Route(HttpMethod.GET, "/posts/:postId/comments/:commentId", handler)
    match5 = route2.match(HttpMethod.GET, "/posts/1/comments/5")
    print(f"Match 5: {match5.matched}, params: {match5.params}")  # → True, {postId: "1", commentId: "5"}
    
    # Test wildcard
    route3 = Route(HttpMethod.GET, "/files/*path", handler)
    match6 = route3.match(HttpMethod.GET, "/files/images/photo.jpg")
    print(f"Match 6: {match6.matched}, params: {match6.params}")  # → True, {path: "images/photo.jpg"}
