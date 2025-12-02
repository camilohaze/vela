"""
HTTP Middleware System

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Sistema de middleware para HTTP con:
- Middleware protocol (Protocol typing)
- Chain of Responsibility pattern
- next() function para control de flujo
- Error handling integrado
"""

from typing import Protocol, Callable, List, Any
from .request import Request
from .response import Response


class Middleware(Protocol):
    """
    Middleware protocol para HTTP request/response pipeline.
    
    Inspirado en:
    - Express.js: middleware(req, res, next)
    - NestJS: NestMiddleware interface
    - ASP.NET Core: IMiddleware
    
    El middleware puede:
    1. Modificar request antes del handler
    2. Llamar next() para continuar la cadena
    3. Short-circuit (no llamar next()) para terminar
    4. Capturar errores y manejarlos
    5. Modificar response después del handler
    """
    
    def handle(self, request: Request, next: Callable[[Request], Response]) -> Response:
        """
        Handle HTTP request y llamar next middleware/handler.
        
        Args:
            request: HTTP request
            next: Siguiente middleware o handler
            
        Returns:
            HTTP response
            
        Example:
            def handle(self, request, next):
                # Pre-handler logic
                print(f"Before: {request.path}")
                
                # Call next middleware/handler
                response = next(request)
                
                # Post-handler logic
                print(f"After: {response.status}")
                return response
        """
        ...


class MiddlewareChain:
    """
    Cadena de middleware que ejecuta middleware en orden.
    
    Pattern: Chain of Responsibility
    - Cada middleware decide si llamar next()
    - Ejecuta middleware en orden: [M1, M2, Handler, M2, M1]
    - Maneja errores en cualquier punto de la cadena
    """
    
    def __init__(self, middleware_list: List[Middleware], handler: Callable[[Request], Response]):
        """
        Initialize middleware chain.
        
        Args:
            middleware_list: Lista de middleware en orden
            handler: Handler final (endpoint)
        """
        self.middleware_list = middleware_list
        self.handler = handler
    
    def execute(self, request: Request) -> Response:
        """
        Execute middleware chain.
        
        Args:
            request: HTTP request
            
        Returns:
            HTTP response
            
        Raises:
            Exception: Si algún middleware o handler lanza error
        """
        # Build chain recursively
        def build_chain(index: int) -> Callable[[Request], Response]:
            """Build next() function recursively."""
            if index >= len(self.middleware_list):
                # End of chain → call handler
                return self.handler
            
            # Wrap next middleware
            current_middleware = self.middleware_list[index]
            next_fn = build_chain(index + 1)
            
            def wrapped(req: Request) -> Response:
                return current_middleware.handle(req, next_fn)
            
            return wrapped
        
        # Start chain execution
        chain = build_chain(0)
        return chain(request)


# Built-in middleware examples

class LoggingMiddleware:
    """
    Middleware para logging de requests/responses.
    
    Example:
        @controller("/users")
        @middleware(LoggingMiddleware)
        class UserController:
            ...
    """
    
    def handle(self, request: Request, next: Callable[[Request], Response]) -> Response:
        print(f"→ {request.method.value} {request.path}")
        
        try:
            response = next(request)
            print(f"← {response.status}")
            return response
        except Exception as e:
            print(f"✗ Error: {e}")
            raise


class AuthMiddleware:
    """
    Middleware para autenticación.
    
    Verifica token en Authorization header.
    Si no hay token → 401 Unauthorized (short-circuit).
    
    Example:
        @controller("/admin")
        @middleware(AuthMiddleware)
        class AdminController:
            ...
    """
    
    def __init__(self, required: bool = True):
        """
        Initialize auth middleware.
        
        Args:
            required: Si True, requiere auth token (default: True)
        """
        self.required = required
    
    def handle(self, request: Request, next: Callable[[Request], Response]) -> Response:
        token = request.authorization
        
        if self.required and not token:
            # Short-circuit: no llamar next()
            from .response import unauthorized
            return unauthorized("Missing authorization token")
        
        # Token válido → continuar
        # (En producción, aquí verificarías el token)
        return next(request)


class CorsMiddleware:
    """
    Middleware para CORS headers.
    
    Agrega headers CORS a response.
    
    Example:
        @controller("/api")
        @middleware(CorsMiddleware(origins=["http://localhost:3000"]))
        class ApiController:
            ...
    """
    
    def __init__(self, origins: List[str] = None, methods: List[str] = None):
        """
        Initialize CORS middleware.
        
        Args:
            origins: Allowed origins (default: ["*"])
            methods: Allowed methods (default: ["GET", "POST", "PUT", "DELETE"])
        """
        self.origins = origins or ["*"]
        self.methods = methods or ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]
    
    def handle(self, request: Request, next: Callable[[Request], Response]) -> Response:
        response = next(request)
        
        # Add CORS headers
        response.set_header("Access-Control-Allow-Origin", ", ".join(self.origins))
        response.set_header("Access-Control-Allow-Methods", ", ".join(self.methods))
        response.set_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        
        return response


class ErrorHandlerMiddleware:
    """
    Middleware para manejo de errores.
    
    Captura excepciones y retorna responses HTTP apropiados.
    
    Example:
        # Este middleware normalmente se agrega al final (outermost)
        @controller("/users")
        @middleware(ErrorHandlerMiddleware)
        class UserController:
            ...
    """
    
    def handle(self, request: Request, next: Callable[[Request], Response]) -> Response:
        try:
            return next(request)
        except ValueError as e:
            from .response import bad_request
            return bad_request(str(e))
        except KeyError as e:
            from .response import not_found
            return not_found(f"Resource not found: {e}")
        except PermissionError as e:
            from .response import forbidden
            return forbidden(str(e))
        except Exception as e:
            from .response import internal_server_error
            print(f"Unhandled error: {e}")
            return internal_server_error("An unexpected error occurred")


if __name__ == "__main__":
    from .request import Request, HttpMethod
    from .response import ok
    
    # Test middleware chain
    def handler(req: Request) -> Response:
        return ok({"message": f"Hello from {req.path}"})
    
    logging = LoggingMiddleware()
    auth = AuthMiddleware(required=False)
    
    chain = MiddlewareChain([logging, auth], handler)
    
    request = Request(
        method=HttpMethod.GET,
        path="/users/123",
        params={"id": "123"}
    )
    
    response = chain.execute(request)
    print(f"\nFinal response: {response}")
    print(f"Body: {response.body}")
