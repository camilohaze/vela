"""
HTTP Module for Vela Runtime

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Sistema de routing HTTP con:
- Request/Response types
- Router con radix tree (O(log n) matching)
- Middleware pipeline
- Route grouping
- Path/query parameter extraction
"""

# Core types
from .request import (
    Request,
    HttpMethod,
    parse_query_string
)

from .response import (
    Response,
    ok,
    created,
    no_content,
    bad_request,
    unauthorized,
    forbidden,
    not_found,
    internal_server_error
)

# Routing
from .route import (
    Route,
    RouteMatch,
    get,
    post,
    put,
    delete,
    patch
)

from .router import (
    Router,
    RouteNode,
    RouteGroup
)

# Middleware
from .middleware import (
    Middleware,
    MiddlewareChain,
    LoggingMiddleware,
    AuthMiddleware,
    CorsMiddleware,
    ErrorHandlerMiddleware
)

# DI Integration
from .controller import (
    ControllerRegistry,
    create_handler_from_method,
    register_controller_routes
)


__all__ = [
    # Request/Response
    "Request",
    "HttpMethod",
    "parse_query_string",
    "Response",
    "ok",
    "created",
    "no_content",
    "bad_request",
    "unauthorized",
    "forbidden",
    "not_found",
    "internal_server_error",
    
    # Routing
    "Route",
    "RouteMatch",
    "get",
    "post",
    "put",
    "delete",
    "patch",
    "Router",
    "RouteNode",
    "RouteGroup",
    
    # Middleware
    "Middleware",
    "MiddlewareChain",
    "LoggingMiddleware",
    "AuthMiddleware",
    "CorsMiddleware",
    "ErrorHandlerMiddleware",
    
    # DI Integration
    "ControllerRegistry",
    "create_handler_from_method",
    "register_controller_routes",
]

__version__ = "0.11.0"  # Router HTTP implementation
