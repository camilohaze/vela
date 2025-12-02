"""
@select Decorator - Selectors Memoizados

Implementación de: VELA-577 - TASK-035W
Historia: State Management
Fecha: 2025-12-02

Descripción:
Decorator @select para crear selectors memoizados que extraen datos
derivados del Store. Integra con Computed para reactividad automática.

Inspirado en:
- reselect (Redux)
- @ngrx/store selectors
- Vuex getters
"""

from typing import Any, Callable, Dict, Generic, List, Optional, Type, TypeVar
from dataclasses import dataclass
from functools import wraps

from .computed import Computed


S = TypeVar('S')  # State type
T = TypeVar('T')  # Selector return type


@dataclass
class SelectOptions:
    """
    Configuración del decorator @select.
    
    Attributes:
        memoize: Habilitar memoization (default: True)
        max_size: Tamaño máximo del cache (default: 1000)
        ttl: Time to live del cache en segundos (None = sin expiración)
        equals: Función de comparación personalizada
        name: Nombre del selector (para debugging)
    """
    memoize: bool = True
    max_size: int = 1000
    ttl: Optional[float] = None
    equals: Optional[Callable[[Any, Any], bool]] = None
    name: Optional[str] = None


def select(
    options: Optional[SelectOptions] = None
) -> Callable[[Callable[..., T]], property]:
    """
    Decorator para crear selectors memoizados.
    
    El decorator @select convierte un método en un selector memoizado
    que se recalcula solo cuando el state cambia. Usa Computed<T>
    internamente para tracking de dependencias.
    
    Args:
        options: Configuración del selector
    
    Returns:
        Property decorator que retorna Computed<T>
    
    Example:
        store AppStore {
            state todos: List<Todo> = []
            
            @select
            computed completed_todos: List<Todo> {
                return self.todos.filter(t => t.completed)
            }
            
            @select(SelectOptions(max_size=100))
            computed completed_count: Number {
                return self.completed_todos.length
            }
        }
    """
    opts = options or SelectOptions()
    
    def decorator(fn: Callable[..., T]) -> property:
        """Decorator interno."""
        
        # Nombre del selector (para debugging)
        selector_name = opts.name or fn.__name__
        
        # Cache de Computed por instancia (weak references)
        # Formato: {id(instance): Computed<T>}
        computed_cache: Dict[int, Computed[T]] = {}
        
        def getter(self) -> T:
            """
            Getter que retorna el valor del selector.
            
            Crea un Computed<T> la primera vez que se accede,
            y lo reutiliza en accesos posteriores.
            """
            instance_id = id(self)
            
            # Verificar si ya existe Computed para esta instancia
            if instance_id not in computed_cache:
                # Crear Computed nuevo
                compute_fn = lambda: fn(self)
                
                computed = Computed(
                    compute_fn,
                    computed_id=f"{type(self).__name__}.{selector_name}",
                    memoize=opts.memoize,
                    memo_max_size=opts.max_size,
                    memo_ttl=opts.ttl,
                )
                
                computed_cache[instance_id] = computed
            
            # Retornar valor del Computed
            return computed_cache[instance_id].get()
        
        # Marcar como selector (metadata)
        getter.__selector__ = True
        getter.__selector_name__ = selector_name
        getter.__selector_options__ = opts
        
        return property(getter)
    
    return decorator


def create_selector(
    *input_selectors: Callable[[S], Any],
    combiner: Callable[..., T]
) -> Callable[[S], T]:
    """
    Crea un selector compuesto a partir de múltiples input selectors.
    
    Similar a createSelector de reselect. Los input selectors se ejecutan
    primero, y sus resultados se pasan al combiner.
    
    Args:
        *input_selectors: Lista de funciones (state) => value
        combiner: Función (...values) => result
    
    Returns:
        Selector memoizado (state) => T
    
    Example:
        # Input selectors
        todos_selector = lambda state: state.todos
        filter_selector = lambda state: state.filter
        
        # Selector compuesto
        filtered_todos = create_selector(
            todos_selector,
            filter_selector,
            combiner=lambda todos, filter: [
                t for t in todos if filter == "all" or 
                (filter == "active" and not t.completed) or
                (filter == "completed" and t.completed)
            ]
        )
        
        # Uso
        result = filtered_todos(app_state)
    """
    # Cache de resultados
    # Formato: {state_id: (inputs_ids, result)}
    cache: Dict[int, tuple] = {}
    
    def memoized_selector(state: S) -> T:
        """Selector memoizado."""
        state_id = id(state)
        
        # Ejecutar input selectors
        inputs = tuple(selector(state) for selector in input_selectors)
        inputs_ids = tuple(id(inp) for inp in inputs)
        
        # Verificar cache
        if state_id in cache:
            cached_inputs_ids, cached_result = cache[state_id]
            if inputs_ids == cached_inputs_ids:
                return cached_result
        
        # Calcular resultado
        result = combiner(*inputs)
        
        # Guardar en cache
        cache[state_id] = (inputs_ids, result)
        
        return result
    
    return memoized_selector


def create_structured_selector(
    selectors_dict: Dict[str, Callable[[S], Any]]
) -> Callable[[S], Dict[str, Any]]:
    """
    Crea un selector que retorna un dict con múltiples valores.
    
    Útil para crear props objects en @connect.
    
    Args:
        selectors_dict: Dict de {key: selector_fn}
    
    Returns:
        Selector que retorna dict con todos los valores
    
    Example:
        todos_props = create_structured_selector({
            "todos": lambda state: state.todos,
            "filter": lambda state: state.filter,
            "completed_count": lambda state: len([
                t for t in state.todos if t.completed
            ])
        })
        
        # Uso
        props = todos_props(app_state)
        # props = {"todos": [...], "filter": "all", "completed_count": 5}
    """
    # Cache de resultados
    cache: Dict[int, Dict[str, Any]] = {}
    
    def structured_selector(state: S) -> Dict[str, Any]:
        """Selector estructurado."""
        state_id = id(state)
        
        # Verificar cache
        if state_id in cache:
            return cache[state_id]
        
        # Ejecutar todos los selectors
        result = {
            key: selector(state)
            for key, selector in selectors_dict.items()
        }
        
        # Guardar en cache
        cache[state_id] = result
        
        return result
    
    return structured_selector


def create_parametric_selector(
    selector_fn: Callable[[S, Any], T]
) -> Callable[[Any], Callable[[S], T]]:
    """
    Crea un selector que acepta parámetros.
    
    Útil para selectors que filtran por ID u otro criterio.
    
    Args:
        selector_fn: Función (state, params) => value
    
    Returns:
        Función que crea selectors parametrizados
    
    Example:
        # Selector parametrizado
        todo_by_id = create_parametric_selector(
            lambda state, todo_id: next(
                (t for t in state.todos if t.id == todo_id),
                None
            )
        )
        
        # Uso
        selector = todo_by_id(42)  # Selector para todo con id=42
        todo = selector(app_state)
    """
    # Cache de selectors por parámetro
    # Formato: {params: cached_selector}
    selector_cache: Dict[Any, Callable[[S], T]] = {}
    
    def create_with_params(params: Any) -> Callable[[S], T]:
        """Crea selector con parámetros específicos."""
        # Verificar cache
        if params in selector_cache:
            return selector_cache[params]
        
        # Crear selector memoizado
        cache: Dict[int, T] = {}
        
        def memoized_selector(state: S) -> T:
            """Selector memoizado."""
            state_id = id(state)
            
            if state_id in cache:
                return cache[state_id]
            
            result = selector_fn(state, params)
            cache[state_id] = result
            
            return result
        
        # Guardar en cache
        selector_cache[params] = memoized_selector
        
        return memoized_selector
    
    return create_with_params


class SelectorComposer:
    """
    Utility class para componer selectors con fluent API.
    
    Permite encadenar transformaciones de selectors.
    
    Example:
        composer = SelectorComposer(lambda state: state.todos)
        
        completed = (composer
            .map(lambda todos: [t for t in todos if t.completed])
            .map(lambda todos: len(todos))
            .build()
        )
        
        count = completed(app_state)
    """
    
    def __init__(self, selector: Callable[[S], Any]):
        """
        Inicializa composer con selector base.
        
        Args:
            selector: Selector inicial
        """
        self._selector = selector
    
    def map(self, transform: Callable[[Any], T]) -> 'SelectorComposer':
        """
        Aplica transformación al resultado del selector.
        
        Args:
            transform: Función de transformación
        
        Returns:
            Nuevo composer con transformación aplicada
        """
        def composed_selector(state: S) -> T:
            intermediate = self._selector(state)
            return transform(intermediate)
        
        return SelectorComposer(composed_selector)
    
    def filter(self, predicate: Callable[[Any], bool]) -> 'SelectorComposer':
        """
        Filtra el resultado (asume lista).
        
        Args:
            predicate: Función de filtro
        
        Returns:
            Nuevo composer con filtro aplicado
        """
        def filtered_selector(state: S) -> List[Any]:
            items = self._selector(state)
            return [item for item in items if predicate(item)]
        
        return SelectorComposer(filtered_selector)
    
    def reduce(
        self,
        reducer: Callable[[Any, Any], Any],
        initial: Any
    ) -> 'SelectorComposer':
        """
        Reduce el resultado (asume lista).
        
        Args:
            reducer: Función de reducción
            initial: Valor inicial
        
        Returns:
            Nuevo composer con reducción aplicada
        """
        def reduced_selector(state: S) -> Any:
            items = self._selector(state)
            result = initial
            for item in items:
                result = reducer(result, item)
            return result
        
        return SelectorComposer(reduced_selector)
    
    def build(self) -> Callable[[S], Any]:
        """
        Construye el selector final.
        
        Returns:
            Selector compuesto
        """
        return self._selector


# Metadata helpers

def is_selector(obj: Any) -> bool:
    """
    Verifica si un objeto es un selector.
    
    Args:
        obj: Objeto a verificar
    
    Returns:
        True si es selector
    """
    return hasattr(obj, '__selector__') and obj.__selector__


def get_selector_name(selector: Any) -> Optional[str]:
    """
    Obtiene el nombre de un selector.
    
    Args:
        selector: Selector
    
    Returns:
        Nombre del selector o None
    """
    return getattr(selector, '__selector_name__', None)


def get_selector_options(selector: Any) -> Optional[SelectOptions]:
    """
    Obtiene las opciones de un selector.
    
    Args:
        selector: Selector
    
    Returns:
        SelectOptions o None
    """
    return getattr(selector, '__selector_options__', None)
