"""
Decorator @connect para State Management

Jira: VELA-577
Historia: Sprint 15 - State Management
Subtask: TASK-035V
Fecha: 2025-12-02

Implementación del decorador @connect que conecta widgets al Store.

Funcionalidad:
- Auto-subscribe al store cuando el widget se monta
- Re-render cuando el selector cambia
- Auto-unsubscribe cuando el widget se destruye
- Inyección automática de state props
"""

from typing import TypeVar, Generic, Callable, Any, Optional, Dict
from dataclasses import dataclass, field
import sys

# Import Store
try:
    from .store import Store
except ImportError:
    sys.path.append('..')
    from src.reactive.store import Store


# ===================================================================
# TYPE DEFINITIONS
# ===================================================================

S = TypeVar('S')  # State type
P = TypeVar('P')  # Props type
W = TypeVar('W')  # Widget type


@dataclass
class ConnectOptions(Generic[S, P]):
    """
    Opciones de configuración para @connect decorator.
    
    Atributos:
        store: Store al que conectar
        selector: Función para extraer props del state (state -> props)
        equals_fn: Función custom de comparación (prev, next) -> bool
        dispatch_prop: Nombre del prop donde inyectar dispatch function
    """
    store: Store[S]
    selector: Callable[[S], P]
    equals_fn: Optional[Callable[[P, P], bool]] = None
    dispatch_prop: str = "dispatch"


# ===================================================================
# CONNECT DECORATOR
# ===================================================================

def connect(options: ConnectOptions[S, P]):
    """
    Decorator @connect para conectar widgets al Store.
    
    El decorador:
    1. Auto-subscribe al store cuando el widget se monta (mount lifecycle)
    2. Ejecuta el selector para extraer props del state
    3. Inyecta los props en el widget
    4. Re-renderiza el widget cuando los props cambian (shallow equality)
    5. Auto-unsubscribe cuando el widget se destruye (destroy lifecycle)
    
    Uso:
    ```vela
    @connect(store: AppStore, selector: (state) => ({ count: state.count }))
    widget Counter {
      # Props inyectadas automáticamente
      count: Number
      
      fn build() -> Widget {
        return Column([
          Text("Count: ${this.count}"),
          Button("+", onPressed: () => this.dispatch(INCREMENT))
        ])
      }
    }
    ```
    
    Args:
        options: Configuración del decorator
    
    Returns:
        Decorator function que wraps el widget class
    """
    def decorator(widget_class: type[W]) -> type[W]:
        """
        Decorator interno que wraps la clase del widget.
        
        Args:
            widget_class: Clase del widget a decorar
        
        Returns:
            Clase del widget con conexión al store
        """
        
        # Guardar métodos originales del lifecycle
        original_mount = getattr(widget_class, 'mount', None)
        original_update = getattr(widget_class, 'update', None)
        original_destroy = getattr(widget_class, 'destroy', None)
        
        # Función de unsubscribe (retornada por store.subscribe)
        unsubscribe_fn: Optional[Callable[[], None]] = None
        
        # Props previas (para comparación)
        prev_props: Optional[P] = None
        
        def enhanced_mount(self):
            """
            Override del mount lifecycle hook.
            
            Suscribe el widget al store y configura auto-update.
            """
            nonlocal unsubscribe_fn, prev_props
            
            # Obtener state actual
            current_state = options.store.get_state()
            
            # Ejecutar selector para obtener props iniciales
            props = options.selector(current_state)
            prev_props = props
            
            # Inyectar props en el widget
            for key, value in props.items() if hasattr(props, 'items') else []:
                setattr(self, key, value)
            
            # Inyectar dispatch function
            setattr(self, options.dispatch_prop, options.store.dispatch)
            
            # Suscribirse al store
            def on_state_change(new_state: S):
                """Callback cuando el state cambia."""
                nonlocal prev_props
                
                # Ejecutar selector
                new_props = options.selector(new_state)
                
                # Comparar props (shallow equality por defecto)
                equals_fn = options.equals_fn or shallow_equal
                if not equals_fn(prev_props, new_props):
                    # Props cambiaron → actualizar widget
                    prev_props = new_props
                    
                    # Inyectar nuevos props
                    for key, value in new_props.items() if hasattr(new_props, 'items') else []:
                        setattr(self, key, value)
                    
                    # Trigger re-render (llamar a update si existe)
                    if hasattr(self, 'update'):
                        self.update()
            
            unsubscribe_fn = options.store.subscribe(on_state_change)
            
            # Llamar mount original si existe
            if original_mount:
                original_mount(self)
        
        def enhanced_update(self):
            """
            Override del update lifecycle hook.
            
            Se ejecuta después de que los props cambian.
            """
            # Llamar update original si existe
            if original_update:
                original_update(self)
        
        def enhanced_destroy(self):
            """
            Override del destroy lifecycle hook.
            
            Desuscribe el widget del store.
            """
            nonlocal unsubscribe_fn
            
            # Unsubscribe del store
            if unsubscribe_fn:
                unsubscribe_fn()
                unsubscribe_fn = None
            
            # Llamar destroy original si existe
            if original_destroy:
                original_destroy(self)
        
        # Reemplazar lifecycle hooks
        widget_class.mount = enhanced_mount
        widget_class.update = enhanced_update
        widget_class.destroy = enhanced_destroy
        
        # Marcar clase como conectada (metadata)
        widget_class.__connected__ = True
        widget_class.__connect_options__ = options
        
        return widget_class
    
    return decorator


# ===================================================================
# HELPER FUNCTIONS
# ===================================================================

def shallow_equal(prev: Any, next: Any) -> bool:
    """
    Comparación shallow (superficial) de dos valores.
    
    Para dicts: compara keys y valores directamente (no profundo)
    Para otros: usa ==
    
    Args:
        prev: Valor previo
        next: Valor siguiente
    
    Returns:
        True si son iguales (shallow), False si no
    """
    # Mismo objeto → iguales
    if prev is next:
        return True
    
    # Tipos diferentes → diferentes
    if type(prev) != type(next):
        return False
    
    # Dict: comparar keys y valores
    if isinstance(prev, dict) and isinstance(next, dict):
        if set(prev.keys()) != set(next.keys()):
            return False
        
        for key in prev.keys():
            if prev[key] != next[key]:
                return False
        
        return True
    
    # List/Tuple: comparar elementos
    if isinstance(prev, (list, tuple)) and isinstance(next, (list, tuple)):
        if len(prev) != len(next):
            return False
        
        for i in range(len(prev)):
            if prev[i] != next[i]:
                return False
        
        return True
    
    # Otros: comparación directa
    return prev == next


def map_state_to_props(selector: Callable[[S], P]):
    """
    Helper para crear selectores de forma más legible.
    
    Uso:
    ```python
    @connect(store=app_store, selector=map_state_to_props(
        lambda state: {'count': state.count, 'user': state.user}
    ))
    ```
    
    Args:
        selector: Función selector
    
    Returns:
        El mismo selector (identity function, solo para legibilidad)
    """
    return selector


def create_selector(*input_selectors, result_func):
    """
    Crear un selector memoizado (reselect-style).
    
    El selector memoizado solo recomputa cuando alguno de los
    input_selectors cambia.
    
    Uso:
    ```python
    completed_todos_selector = create_selector(
        lambda state: state.todos,
        lambda todos: [t for t in todos if t.completed]
    )
    
    @connect(store=app_store, selector=completed_todos_selector)
    widget CompletedTodos { ... }
    ```
    
    Args:
        input_selectors: Selectores de entrada
        result_func: Función que combina los resultados
    
    Returns:
        Selector memoizado
    """
    # Cache de resultados
    prev_inputs = []
    prev_result = None
    
    def memoized_selector(state: S) -> P:
        nonlocal prev_inputs, prev_result
        
        # Ejecutar input selectors
        current_inputs = [sel(state) for sel in input_selectors]
        
        # Comparar inputs (shallow)
        if shallow_equal(prev_inputs, current_inputs):
            # Inputs iguales → retornar resultado cacheado
            return prev_result
        
        # Inputs cambiaron → recomputar
        prev_inputs = current_inputs
        prev_result = result_func(*current_inputs)
        
        return prev_result
    
    return memoized_selector


# ===================================================================
# CONNECT HELPERS (API SIMPLIFICADA)
# ===================================================================

def connect_to_store(store: Store[S], selector: Callable[[S], P]):
    """
    API simplificada para @connect.
    
    Uso:
    ```python
    @connect_to_store(app_store, lambda state: {'count': state.count})
    class Counter(Widget):
        ...
    ```
    
    Args:
        store: Store al que conectar
        selector: Función selector
    
    Returns:
        Decorator @connect configurado
    """
    return connect(ConnectOptions(
        store=store,
        selector=selector
    ))


def connect_with_dispatch(store: Store[S], selector: Callable[[S], P], dispatch_prop: str = "dispatch"):
    """
    Conectar al store e inyectar dispatch function.
    
    Uso:
    ```python
    @connect_with_dispatch(app_store, lambda state: {'count': state.count})
    class Counter(Widget):
        def increment(self):
            self.dispatch(INCREMENT)
    ```
    
    Args:
        store: Store al que conectar
        selector: Función selector
        dispatch_prop: Nombre del prop dispatch
    
    Returns:
        Decorator @connect configurado
    """
    return connect(ConnectOptions(
        store=store,
        selector=selector,
        dispatch_prop=dispatch_prop
    ))


# ===================================================================
# EXPORT
# ===================================================================

__all__ = [
    'connect',
    'ConnectOptions',
    'shallow_equal',
    'map_state_to_props',
    'create_selector',
    'connect_to_store',
    'connect_with_dispatch'
]


if __name__ == "__main__":
    print("@connect decorator loaded")
    print("Use: @connect(store=my_store, selector=lambda s: s.prop)")
