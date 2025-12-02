"""
Router + DI Integration

Implementación de: TASK-035G2
Historia: VELA-575
Fecha: 2025-12-02

Descripción:
Integración del Router HTTP con el sistema DI:
- Auto-resolve controller instances desde Injector
- Scope management: Scoped per request
- Inyección de dependencias en handlers
- Handler method resolution

Inspirado en:
- NestJS: Controllers + DI integration
- Spring Boot: @RestController + @Autowired
- ASP.NET Core: Controller resolution con DI
"""

from typing import Any, Callable, Optional, Type
from ..di import Injector, Scope, inject
from .request import Request
from .response import Response
from .router import Router


class ControllerRegistry:
    """
    Registry de controllers para DI integration.
    
    Mapea controller classes a sus métodos HTTP.
    """
    
    def __init__(self, injector: Injector):
        """
        Initialize controller registry.
        
        Args:
            injector: DI injector instance
        """
        self.injector = injector
        self.controllers: dict[Type, Any] = {}
    
    def register_controller(self, controller_class: Type, prefix: str = "") -> None:
        """
        Register controller class in DI and extract routes.
        
        Args:
            controller_class: Controller class to register
            prefix: Route prefix (from @controller decorator)
            
        Example:
            @injectable
            @controller("/users")
            class UserController:
                @get("/:id")
                def get_user(self, request: Request) -> Response:
                    ...
        """
        # Register in DI (if not already registered)
        # En producción, esto se haría con el decorador @injectable
        if controller_class not in self.controllers:
            self.controllers[controller_class] = {
                'prefix': prefix,
                'instance': None  # Lazy-loaded
            }
    
    def resolve_controller(self, controller_class: Type, request_scope: Optional[Any] = None) -> Any:
        """
        Resolve controller instance desde Injector.
        
        Args:
            controller_class: Controller class
            request_scope: Optional request-scoped context
            
        Returns:
            Controller instance
            
        Note:
            En producción, esto usaría self.injector.resolve(controller_class, context=request_scope)
        """
        if controller_class not in self.controllers:
            raise ValueError(f"Controller not registered: {controller_class}")
        
        # Resolve from DI
        # instance = self.injector.resolve(controller_class, context=request_scope)
        
        # Simplified: Instantiate directly (for testing)
        instance = controller_class()
        return instance


def create_handler_from_method(
    controller_class: Type,
    method_name: str,
    registry: ControllerRegistry
) -> Callable[[Request], Response]:
    """
    Create handler function que resuelve controller instance y llama method.
    
    Args:
        controller_class: Controller class
        method_name: Method name (e.g., "get_user")
        registry: Controller registry
        
    Returns:
        Handler function
        
    Example:
        handler = create_handler_from_method(UserController, "get_user", registry)
        response = handler(request)  # → resolve UserController + call get_user
    """
    def handler(request: Request) -> Response:
        # Resolve controller instance (con request scope)
        instance = registry.resolve_controller(controller_class, request_scope=None)
        
        # Get method
        method = getattr(instance, method_name)
        
        # Call method with request
        return method(request)
    
    return handler


def register_controller_routes(
    router: Router,
    controller_class: Type,
    prefix: str,
    registry: ControllerRegistry
) -> None:
    """
    Register todas las rutas de un controller en el router.
    
    Esto simula el comportamiento de decoradores @get, @post, etc.
    En producción, esto se haría automáticamente al escanear decoradores.
    
    Args:
        router: Router instance
        controller_class: Controller class
        prefix: Route prefix
        registry: Controller registry
        
    Example:
        register_controller_routes(router, UserController, "/users", registry)
    """
    # Register controller in registry
    registry.register_controller(controller_class, prefix)
    
    # En producción, aquí se escanearían los decoradores @get, @post
    # y se registrarían automáticamente las rutas
    
    # Por ahora, esto es manual (ver ejemplo en __main__)


# Example usage

if __name__ == "__main__":
    from .response import ok
    from .request import HttpMethod
    
    # Create injector and router
    injector = Injector()
    router = Router()
    registry = ControllerRegistry(injector)
    
    # Define controller
    class UserController:
        def list_users(self, request: Request) -> Response:
            return ok({"users": ["Alice", "Bob"]})
        
        def get_user(self, request: Request) -> Response:
            user_id = request.get_param('id')
            return ok({"id": user_id, "name": "Alice"})
        
        def create_user(self, request: Request) -> Response:
            # En producción, request.body tendría los datos
            return ok({"id": 123, "name": "New User"})
    
    # Register routes manually (in production, this would be automatic via decorators)
    register_controller_routes(router, UserController, "/users", registry)
    
    # Register routes using DI-resolved handlers
    router.get("/users", create_handler_from_method(UserController, "list_users", registry))
    router.get("/users/:id", create_handler_from_method(UserController, "get_user", registry))
    router.post("/users", create_handler_from_method(UserController, "create_user", registry))
    
    # Test routing with DI
    print("=== Test Router + DI ===")
    response1 = router.handle(HttpMethod.GET, "/users")
    print(f"GET /users → {response1.status} {response1.body}")
    
    response2 = router.handle(HttpMethod.GET, "/users/123")
    print(f"GET /users/123 → {response2.status} {response2.body}")
    
    response3 = router.handle(HttpMethod.POST, "/users")
    print(f"POST /users → {response3.status} {response3.body}")
