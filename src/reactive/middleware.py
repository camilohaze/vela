"""
Middleware System - Interceptores de Acciones

Implementación de: VELA-577 - TASK-035Y
Historia: State Management
Fecha: 2025-12-02

Descripción:
Sistema de middleware para interceptar y modificar acciones antes de que
lleguen al reducer. Permite side effects, logging, async operations, etc.

Inspirado en:
- Redux Middleware (redux-thunk, redux-saga)
- Express.js Middleware
- Koa Middleware
"""

from typing import Any, Callable, Dict, List, Optional, TypeVar, TYPE_CHECKING
from dataclasses import dataclass
from datetime import datetime
import json

if TYPE_CHECKING:
    from .store import Store
    from .action import Action

# Type aliases
S = TypeVar('S')  # State type
MiddlewareAPI = Any  # Store interface para middleware
NextFunction = Callable[[Any], None]
MiddlewareFunction = Callable[[MiddlewareAPI, NextFunction, Any], None]


@dataclass
class MiddlewareContext:
    """
    Contexto disponible para middlewares.
    
    Attributes:
        store: Referencia al Store
        get_state: Función para obtener estado actual
        dispatch: Función para dispatch de acciones
    """
    store: Any
    get_state: Callable[[], Any]
    dispatch: Callable[[Any], None]


class Middleware:
    """
    Base class para middlewares.
    
    Un middleware es una función que intercepta acciones antes
    de que lleguen al reducer.
    
    Signature:
        fn middleware(store: Store, next: Dispatch, action: Action) -> void
    
    Example:
        class LoggerMiddleware(Middleware):
            def handle(self, context, next, action):
                print(f"Dispatching: {action}")
                next(action)
    """
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """
        Maneja la acción.
        
        Args:
            context: Contexto del middleware (store, getState, dispatch)
            next: Función para pasar al siguiente middleware
            action: Acción a procesar
        """
        # Por defecto, pasar al siguiente middleware
        next(action)
    
    def __call__(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Permite usar el middleware como callable."""
        return self.handle(context, next, action)


# Middleware Prebuilts

class LoggerMiddleware(Middleware):
    """
    Middleware de logging.
    
    Registra todas las acciones dispatched y cambios de estado.
    Útil para debugging.
    
    Example:
        store = Store(
            initial_state,
            reducer,
            middlewares=[LoggerMiddleware()]
        )
    """
    
    def __init__(
        self,
        log_actions: bool = True,
        log_state: bool = True,
        collapsed: bool = False
    ):
        """
        Inicializa el logger middleware.
        
        Args:
            log_actions: Si registrar acciones
            log_state: Si registrar cambios de estado
            collapsed: Si logs deben estar colapsados
        """
        self.log_actions = log_actions
        self.log_state = log_state
        self.collapsed = collapsed
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Registra acción y cambio de estado."""
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        
        if self.log_actions:
            action_type = getattr(action, 'type', action.__class__.__name__)
            print(f"[{timestamp}] action {action_type}")
            
            # Log payload si existe
            if hasattr(action, 'payload'):
                print(f"  payload: {action.payload}")
        
        if self.log_state:
            prev_state = context.get_state()
            print(f"  prev state: {self._serialize_state(prev_state)}")
        
        # Pasar al siguiente middleware
        next(action)
        
        if self.log_state:
            next_state = context.get_state()
            print(f"  next state: {self._serialize_state(next_state)}")
    
    @staticmethod
    def _serialize_state(state: Any) -> str:
        """Serializa estado para logging."""
        try:
            if hasattr(state, '__dict__'):
                return json.dumps(state.__dict__, default=str, indent=2)
            return str(state)
        except:
            return repr(state)


class AsyncMiddleware(Middleware):
    """
    Middleware para operaciones asíncronas.
    
    Permite dispatch de funciones async que reciben dispatch y getState.
    Similar a redux-thunk.
    
    Example:
        async def fetchUsers():
            async def thunk(dispatch, getState):
                dispatch(SetLoadingAction(True))
                
                users = await api.fetch_users()
                
                dispatch(SetUsersAction(users))
                dispatch(SetLoadingAction(False))
            
            return thunk
        
        # Dispatch async
        store.dispatch(fetchUsers())
    """
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Maneja acciones async."""
        # Si es una función, ejecutarla con dispatch y getState
        if callable(action) and not hasattr(action, 'type'):
            # Es una función thunk
            try:
                result = action(context.dispatch, context.get_state)
                
                # Si es async, esperar resultado
                import asyncio
                if asyncio.iscoroutine(result):
                    # NOTE: En producción, esto se manejaría con event loop
                    # Por ahora, solo pasamos
                    pass
            except Exception as e:
                print(f"Error in async action: {e}")
            
            return  # No pasar al siguiente middleware
        
        # No es async, pasar al siguiente
        next(action)


class ThrottleMiddleware(Middleware):
    """
    Middleware de throttling.
    
    Limita la frecuencia de dispatch de acciones del mismo tipo.
    Útil para rate limiting.
    
    Example:
        store = Store(
            initial_state,
            reducer,
            middlewares=[ThrottleMiddleware(delay=1000)]  # Max 1 acción por segundo
        )
    """
    
    def __init__(self, delay: int = 1000):
        """
        Inicializa throttle middleware.
        
        Args:
            delay: Tiempo mínimo entre dispatches en ms
        """
        self.delay = delay  # ms
        self._last_dispatch: Dict[str, float] = {}
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Throttle actions por tipo."""
        import time
        
        action_type = getattr(action, 'type', action.__class__.__name__)
        now = time.time() * 1000  # ms
        
        last = self._last_dispatch.get(action_type, 0)
        elapsed = now - last
        
        if elapsed >= self.delay:
            self._last_dispatch[action_type] = now
            next(action)
        else:
            # Throttled: no pasar al siguiente middleware
            print(f"[Throttled] {action_type} (wait {self.delay - elapsed:.0f}ms)")


class DebounceMiddleware(Middleware):
    """
    Middleware de debouncing.
    
    Retrasa el dispatch de acciones hasta que deje de recibir
    acciones del mismo tipo por un tiempo determinado.
    
    Example:
        store = Store(
            initial_state,
            reducer,
            middlewares=[DebounceMiddleware(delay=300)]
        )
    """
    
    def __init__(self, delay: int = 300):
        """
        Inicializa debounce middleware.
        
        Args:
            delay: Tiempo de espera en ms
        """
        self.delay = delay  # ms
        self._timers: Dict[str, Any] = {}
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Debounce actions por tipo."""
        import threading
        
        action_type = getattr(action, 'type', action.__class__.__name__)
        
        # Cancelar timer anterior si existe
        if action_type in self._timers:
            self._timers[action_type].cancel()
        
        # Crear nuevo timer
        timer = threading.Timer(
            self.delay / 1000,  # Convertir a segundos
            lambda: next(action)
        )
        timer.start()
        self._timers[action_type] = timer


class ErrorHandlerMiddleware(Middleware):
    """
    Middleware de manejo de errores.
    
    Captura excepciones en reducers y middlewares subsecuentes.
    Dispatch una acción de error en lugar de crashear.
    
    Example:
        store = Store(
            initial_state,
            reducer,
            middlewares=[ErrorHandlerMiddleware()]
        )
    """
    
    def __init__(
        self,
        on_error: Optional[Callable[[Exception, Any], None]] = None
    ):
        """
        Inicializa error handler.
        
        Args:
            on_error: Callback opcional para manejar errores
        """
        self.on_error = on_error
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Captura errores en el pipeline."""
        try:
            next(action)
        except Exception as e:
            print(f"[Error] Action failed: {action}")
            print(f"  Error: {e}")
            
            # Callback custom si existe
            if self.on_error:
                self.on_error(e, action)
            
            # Dispatch error action
            try:
                from .action import SimpleAction
                error_action = SimpleAction(
                    "ERROR",
                    payload={
                        "error": str(e),
                        "action": action
                    }
                )
                context.dispatch(error_action)
            except:
                pass  # Evitar loop infinito


class CacheMiddleware(Middleware):
    """
    Middleware de caching.
    
    Cachea resultados de acciones basándose en su payload.
    Si la misma acción se dispatch con el mismo payload,
    retorna el resultado cacheado sin ejecutar el reducer.
    
    Example:
        store = Store(
            initial_state,
            reducer,
            middlewares=[CacheMiddleware(max_size=100)]
        )
    """
    
    def __init__(self, max_size: int = 100):
        """
        Inicializa cache middleware.
        
        Args:
            max_size: Tamaño máximo del cache
        """
        self.max_size = max_size
        self._cache: Dict[str, Any] = {}
        self._cache_keys: List[str] = []
    
    def _get_cache_key(self, action: Any) -> str:
        """Genera clave de cache para acción."""
        action_type = getattr(action, 'type', action.__class__.__name__)
        payload = getattr(action, 'payload', None)
        
        try:
            payload_str = json.dumps(payload, default=str, sort_keys=True)
        except:
            payload_str = str(payload)
        
        return f"{action_type}:{payload_str}"
    
    def handle(
        self,
        context: MiddlewareContext,
        next: NextFunction,
        action: Any
    ) -> None:
        """Cachea resultados de acciones."""
        cache_key = self._get_cache_key(action)
        
        # Si está en cache, retornar cached
        if cache_key in self._cache:
            print(f"[Cache HIT] {cache_key}")
            return  # No ejecutar reducer
        
        # Guardar estado previo
        prev_state = context.get_state()
        
        # Ejecutar acción
        next(action)
        
        # Guardar en cache
        next_state = context.get_state()
        
        # Solo cachear si cambió el estado
        if prev_state != next_state:
            self._cache[cache_key] = next_state
            self._cache_keys.append(cache_key)
            
            # Evict oldest si superó max_size
            if len(self._cache_keys) > self.max_size:
                oldest_key = self._cache_keys.pop(0)
                del self._cache[oldest_key]


# Helper Functions

def compose_middleware(*middlewares: Middleware) -> MiddlewareFunction:
    """
    Compone múltiples middlewares en una cadena.
    
    Args:
        *middlewares: Middlewares a componer
    
    Returns:
        Función middleware compuesta
    
    Example:
        composed = compose_middleware(
            LoggerMiddleware(),
            AsyncMiddleware(),
            ErrorHandlerMiddleware()
        )
    """
    def composed(context: MiddlewareContext, final_next: NextFunction, action: Any) -> None:
        """Middleware compuesto."""
        # Construir cadena de next functions
        index = 0
        
        def dispatch(current_action: Any) -> None:
            """Dispatch interno que pasa al siguiente middleware."""
            nonlocal index
            
            if index >= len(middlewares):
                # Llegamos al final, ejecutar reducer
                final_next(current_action)
                return
            
            # Obtener middleware actual
            middleware = middlewares[index]
            index += 1
            
            # Ejecutar middleware
            middleware.handle(context, dispatch, current_action)
        
        # Iniciar cadena
        dispatch(action)
    
    return composed


def apply_middleware(store: 'Store', *middlewares: Middleware) -> None:
    """
    Aplica middlewares a un Store existente.
    
    Args:
        store: Store al que aplicar middlewares
        *middlewares: Middlewares a aplicar
    
    Example:
        store = Store(initial_state, reducer)
        apply_middleware(
            store,
            LoggerMiddleware(),
            AsyncMiddleware()
        )
    """
    if not middlewares:
        return
    
    # Guardar dispatch original
    original_dispatch = store.dispatch
    
    # Crear contexto
    context = MiddlewareContext(
        store=store,
        get_state=store.get_state,
        dispatch=lambda action: original_dispatch(action)
    )
    
    # Componer middlewares
    composed = compose_middleware(*middlewares)
    
    # Reemplazar dispatch con versión con middlewares
    def dispatch_with_middleware(action: Any) -> None:
        """Dispatch que pasa por middlewares."""
        composed(context, original_dispatch, action)
    
    store.dispatch = dispatch_with_middleware


def create_middleware(handler: MiddlewareFunction) -> Middleware:
    """
    Crea un middleware desde una función.
    
    Args:
        handler: Función middleware
    
    Returns:
        Instancia de Middleware
    
    Example:
        def my_middleware(context, next, action):
            print(f"Before: {action}")
            next(action)
            print("After")
        
        middleware = create_middleware(my_middleware)
    """
    class CustomMiddleware(Middleware):
        def handle(self, context, next, action):
            return handler(context, next, action)
    
    return CustomMiddleware()
