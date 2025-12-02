"""
Store<T> - State Management System

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02

Descripción:
Store<T> es el contenedor principal del estado global de la aplicación.
Implementa el patrón Redux/NgRx con integración profunda al Signal System.

Características:
- Estado reactivo (integrado con Signals)
- Flujo unidireccional: Action → Reducer → State
- Middleware system
- DevTools integration
- Type-safe
"""

from typing import Any, Callable, Dict, List, TypeVar, Generic, Optional
from copy import deepcopy
import json

from .signal import Signal
from .computed import Computed
from .action import Action, InitAction, ResetAction
from .reducer import Reducer


S = TypeVar('S')  # State type
A = TypeVar('A', bound=Action)  # Action type


# Type aliases
Middleware = Callable[['Store', Callable, Action], None]
Selector = Callable[[S], Any]
Listener = Callable[[S], None]


class Store(Generic[S]):
    """
    Store<T> - Contenedor de estado global con reactividad.
    
    El Store es el corazón del sistema de State Management.
    Mantiene el estado de la aplicación y coordina updates a través
    de un flujo unidireccional predecible.
    
    Inspirado en:
    - Redux: Store con dispatch, getState, subscribe
    - NgRx: Type-safe store con selectors
    - Zustand: API minimalista
    
    Ejemplo:
        # 1. Definir estado inicial
        initial_state = {"count": 0, "todos": []}
        
        # 2. Definir reducer
        def app_reducer(state, action):
            if action.get_type() == "INCREMENT":
                return {...state, "count": state["count"] + 1}
            return state
        
        # 3. Crear store
        store = Store(initial_state, app_reducer)
        
        # 4. Subscribe a cambios
        store.subscribe(lambda state: print(f"State: {state}"))
        
        # 5. Dispatch actions
        store.dispatch(SimpleAction("INCREMENT"))
    """
    
    def __init__(
        self,
        initial_state: S,
        reducer: Reducer[S, A],
        middlewares: Optional[List[Middleware]] = None,
        enable_devtools: bool = False,
        name: str = "Store"
    ):
        """
        Inicializa el Store.
        
        Args:
            initial_state: Estado inicial de la aplicación
            reducer: Función reducer (state, action) -> new_state
            middlewares: Lista opcional de middlewares
            enable_devtools: Habilitar DevTools integration
            name: Nombre del store (para debugging)
        """
        self.name = name
        self.reducer = reducer
        self.middlewares = middlewares or []
        self.enable_devtools = enable_devtools
        
        # Estado reactivo usando Signal System
        self._state = Signal(initial_state)
        
        # Estado inicial (para reset)
        self._initial_state = deepcopy(initial_state)
        
        # Listeners
        self._listeners: List[Listener] = []
        
        # History para DevTools
        self._history: List[Dict[str, Any]] = []
        self._current_history_index = -1
        
        # Stats
        self._action_count = 0
        
        # Dispatch acción inicial
        self.dispatch(InitAction())
    
    def get_state(self) -> S:
        """
        Retorna el estado actual del store.
        
        Returns:
            Estado actual (inmutable, no modificar)
        """
        return self._state.get()
    
    def dispatch(self, action: A) -> None:
        """
        Despacha una acción al store.
        
        El flujo es:
        1. Pasar por middleware chain
        2. Aplicar reducer
        3. Actualizar estado
        4. Notificar listeners
        5. Guardar en history (si DevTools habilitado)
        
        Args:
            action: Acción a procesar
        """
        # Incrementar contador
        self._action_count += 1
        
        # Obtener estado actual
        current_state = self.get_state()
        
        # Aplicar middleware chain
        final_dispatch = self._apply_middlewares(action)
        
        # Si middlewares cancelaron la acción, salir
        if final_dispatch is None:
            return
        
        # Aplicar reducer
        new_state = self.reducer(current_state, action)
        
        # Actualizar estado
        self._state.set(new_state)
        
        # Notificar listeners
        self._notify_listeners(new_state)
        
        # Guardar en history
        if self.enable_devtools:
            self._save_to_history(action, current_state, new_state)
    
    def subscribe(self, listener: Listener) -> Callable[[], None]:
        """
        Suscribe un listener a cambios de estado.
        
        El listener se llama cada vez que el estado cambia.
        
        Args:
            listener: Función (state) -> void
            
        Returns:
            Función de unsubscribe
            
        Ejemplo:
            unsubscribe = store.subscribe(lambda state: print(state))
            # ... más tarde
            unsubscribe()
        """
        self._listeners.append(listener)
        
        # Retornar función de unsubscribe
        def unsubscribe():
            if listener in self._listeners:
                self._listeners.remove(listener)
        
        return unsubscribe
    
    def select(self, selector: Selector) -> Computed:
        """
        Crea un selector memoizado para una parte del estado.
        
        Los selectors son Computed values que se recalculan solo
        cuando la parte del estado que seleccionan cambia.
        
        Args:
            selector: Función (state) -> value
            
        Returns:
            Computed value
            
        Ejemplo:
            # Selector para contar todos
            todo_count = store.select(lambda state: len(state["todos"]))
            
            # Usar
            print(f"Todos: {todo_count.get()}")
        """
        return Computed(lambda: selector(self.get_state()))
    
    def reset(self) -> None:
        """
        Resetea el store al estado inicial.
        
        Útil para testing o logout.
        """
        self.dispatch(ResetAction())
        self._state.set(deepcopy(self._initial_state))
    
    def replace_reducer(self, new_reducer: Reducer[S, A]) -> None:
        """
        Reemplaza el reducer actual.
        
        Útil para hot-reloading o code splitting.
        
        Args:
            new_reducer: Nuevo reducer
        """
        self.reducer = new_reducer
    
    def add_middleware(self, middleware: Middleware) -> None:
        """
        Agrega un middleware al store.
        
        Args:
            middleware: Middleware function
        """
        self.middlewares.append(middleware)
    
    def get_action_count(self) -> int:
        """
        Retorna el número total de acciones despachadas.
        
        Útil para metrics y debugging.
        """
        return self._action_count
    
    def get_history(self) -> List[Dict[str, Any]]:
        """
        Retorna el historial de acciones (si DevTools habilitado).
        
        Returns:
            Lista de entries: {action, prevState, newState, timestamp}
        """
        return self._history
    
    def time_travel(self, index: int) -> None:
        """
        Time-travel: viajar a un punto en el historial.
        
        Args:
            index: Índice en el historial
        """
        if not self.enable_devtools:
            raise ValueError("DevTools no habilitado")
        
        if index < 0 or index >= len(self._history):
            raise ValueError(f"Índice inválido: {index}")
        
        entry = self._history[index]
        self._state.set(entry["newState"])
        self._current_history_index = index
    
    def undo(self) -> bool:
        """
        Deshace la última acción.
        
        Returns:
            True si se pudo deshacer, False si no hay más historial
        """
        if self._current_history_index > 0:
            self.time_travel(self._current_history_index - 1)
            return True
        return False
    
    def redo(self) -> bool:
        """
        Rehace una acción deshecha.
        
        Returns:
            True si se pudo rehacer, False si no hay más historial
        """
        if self._current_history_index < len(self._history) - 1:
            self.time_travel(self._current_history_index + 1)
            return True
        return False
    
    def clear_history(self) -> None:
        """Limpia el historial de acciones."""
        self._history.clear()
        self._current_history_index = -1
    
    def to_json(self) -> str:
        """
        Serializa el estado actual a JSON.
        
        Útil para persistencia.
        
        Returns:
            Estado como JSON string
        """
        return json.dumps(self.get_state())
    
    @staticmethod
    def from_json(
        json_str: str,
        reducer: Reducer,
        **kwargs
    ) -> 'Store':
        """
        Crea un Store desde un JSON string.
        
        Args:
            json_str: Estado serializado
            reducer: Reducer function
            **kwargs: Otros argumentos para Store()
            
        Returns:
            Store con estado restaurado
        """
        state = json.loads(json_str)
        return Store(state, reducer, **kwargs)
    
    # Métodos privados
    
    def _apply_middlewares(self, action: A) -> Optional[A]:
        """
        Aplica la cadena de middlewares a la acción.
        
        Los middlewares pueden:
        - Modificar la acción
        - Cancelar la acción (return None)
        - Despachar acciones adicionales
        - Realizar side effects
        
        Args:
            action: Acción original
            
        Returns:
            Acción final o None si fue cancelada
        """
        if not self.middlewares:
            return action
        
        # Crear chain
        index = 0
        
        def next_middleware(current_action: A):
            nonlocal index
            
            if index >= len(self.middlewares):
                return current_action
            
            middleware = self.middlewares[index]
            index += 1
            
            # Ejecutar middleware
            middleware(self, next_middleware, current_action)
        
        next_middleware(action)
        return action
    
    def _notify_listeners(self, state: S) -> None:
        """
        Notifica a todos los listeners del nuevo estado.
        
        Args:
            state: Nuevo estado
        """
        for listener in self._listeners:
            try:
                listener(state)
            except Exception as e:
                print(f"Error in listener: {e}")
    
    def _save_to_history(self, action: A, prev_state: S, new_state: S) -> None:
        """
        Guarda la acción en el historial.
        
        Args:
            action: Acción despachada
            prev_state: Estado anterior
            new_state: Estado nuevo
        """
        import time
        
        entry = {
            "action": action.to_dict(),
            "prevState": prev_state,
            "newState": new_state,
            "timestamp": time.time()
        }
        
        # Si estamos en medio del historial, truncar
        if self._current_history_index < len(self._history) - 1:
            self._history = self._history[:self._current_history_index + 1]
        
        self._history.append(entry)
        self._current_history_index += 1
    
    def __repr__(self) -> str:
        return f"Store(name={self.name}, actions={self._action_count})"


if __name__ == "__main__":
    from .action import SimpleAction, PayloadAction
    
    # Ejemplo: Counter Store
    print("=== Counter Store ===")
    
    initial_state = {"count": 0}
    
    def counter_reducer(state: dict, action: Action) -> dict:
        if action.get_type() == "INCREMENT":
            return {**state, "count": state["count"] + 1}
        elif action.get_type() == "DECREMENT":
            return {**state, "count": state["count"] - 1}
        elif action.get_type() == "SET":
            return {**state, "count": action.payload}
        return state
    
    store = Store(initial_state, counter_reducer, enable_devtools=True, name="CounterStore")
    
    # Subscribe
    unsubscribe = store.subscribe(lambda state: print(f"  State changed: {state}"))
    
    # Dispatch
    print("\nDispatching INCREMENT...")
    store.dispatch(SimpleAction("INCREMENT"))
    
    print("\nDispatching INCREMENT...")
    store.dispatch(SimpleAction("INCREMENT"))
    
    print("\nDispatching SET(10)...")
    store.dispatch(PayloadAction("SET", payload=10))
    
    print("\nDispatching DECREMENT...")
    store.dispatch(SimpleAction("DECREMENT"))
    
    # Selector
    print("\n--- Selectors ---")
    doubled = store.select(lambda state: state["count"] * 2)
    print(f"Count: {store.get_state()['count']}")
    print(f"Doubled (selector): {doubled.get()}")
    
    # Time-travel
    print("\n--- Time Travel ---")
    print(f"Current state: {store.get_state()}")
    print(f"Action count: {store.get_action_count()}")
    
    print("\nUndo...")
    store.undo()
    print(f"After undo: {store.get_state()}")
    
    print("\nUndo...")
    store.undo()
    print(f"After undo: {store.get_state()}")
    
    print("\nRedo...")
    store.redo()
    print(f"After redo: {store.get_state()}")
    
    # Unsubscribe
    unsubscribe()
    
    # Stats
    print(f"\n--- Stats ---")
    print(f"Store: {store}")
    print(f"Total actions: {store.get_action_count()}")
    print(f"History entries: {len(store.get_history())}")
