"""
Reducer types for State Management System

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02

Descripción:
Este módulo define los tipos para Reducers en el sistema de State Management.
Los Reducers son funciones puras: (State, Action) → State
"""

from typing import Callable, TypeVar, Any, Dict
from abc import ABC, abstractmethod
from .action import Action


S = TypeVar('S')  # State type
A = TypeVar('A', bound=Action)  # Action type


# Type alias para una función reducer
Reducer = Callable[[S, A], S]


class ReducerProtocol(ABC):
    """
    Protocolo abstracto para reducers.
    
    Los reducers DEBEN ser funciones puras:
    - No side effects
    - Mismos inputs → mismo output
    - No mutar el estado original
    
    Ejemplo:
        def counter_reducer(state: int, action: Action) -> int:
            if action.get_type() == "INCREMENT":
                return state + 1
            elif action.get_type() == "DECREMENT":
                return state - 1
            return state
    """
    
    @abstractmethod
    def reduce(self, state: S, action: A) -> S:
        """
        Procesa una acción y retorna el nuevo estado.
        
        Args:
            state: Estado actual
            action: Acción a procesar
            
        Returns:
            Nuevo estado (NUNCA mutar el estado original)
        """
        pass


class CombinedReducer(ReducerProtocol):
    """
    Combina múltiples reducers en uno solo.
    
    Cada reducer maneja una parte del estado global.
    Similar a combineReducers() de Redux.
    
    Ejemplo:
        todos_reducer = lambda state, action: state
        counter_reducer = lambda state, action: state
        
        combined = CombinedReducer({
            "todos": todos_reducer,
            "counter": counter_reducer
        })
        
        # State: { todos: [...], counter: 0 }
    """
    
    def __init__(self, reducers: Dict[str, Reducer]):
        """
        Inicializa el combined reducer.
        
        Args:
            reducers: Dict de {key: reducer_function}
                     Cada reducer maneja state[key]
        """
        self.reducers = reducers
    
    def reduce(self, state: Dict[str, Any], action: Action) -> Dict[str, Any]:
        """
        Aplica cada reducer a su parte del estado.
        
        Args:
            state: Estado completo (dict)
            action: Acción a procesar
            
        Returns:
            Nuevo estado con todas las partes actualizadas
        """
        new_state = {}
        state_changed = False
        
        for key, reducer in self.reducers.items():
            prev_state = state.get(key)
            next_state = reducer(prev_state, action)
            
            new_state[key] = next_state
            
            # Detectar cambios (shallow comparison)
            if next_state is not prev_state:
                state_changed = True
        
        # Si nada cambió, retornar el mismo objeto (optimización)
        return new_state if state_changed else state


class ReducerBuilder:
    """
    Builder para construir reducers de forma declarativa.
    
    Ejemplo:
        reducer = (
            ReducerBuilder(initial_state=0)
            .case("INCREMENT", lambda state, action: state + 1)
            .case("DECREMENT", lambda state, action: state - 1)
            .case("SET", lambda state, action: action.payload)
            .build()
        )
    """
    
    def __init__(self, initial_state: Any = None):
        """
        Inicializa el builder.
        
        Args:
            initial_state: Estado inicial opcional
        """
        self.initial_state = initial_state
        self.handlers: Dict[str, Reducer] = {}
        self.default_handler: Reducer | None = None
    
    def case(self, action_type: str, handler: Reducer):
        """
        Registra un handler para un tipo de acción.
        
        Args:
            action_type: Tipo de acción a manejar
            handler: Función (state, action) -> new_state
            
        Returns:
            self (para encadenamiento)
        """
        self.handlers[action_type] = handler
        return self
    
    def default(self, handler: Reducer):
        """
        Registra un handler por defecto para acciones no manejadas.
        
        Args:
            handler: Función (state, action) -> new_state
            
        Returns:
            self (para encadenamiento)
        """
        self.default_handler = handler
        return self
    
    def build(self) -> Reducer:
        """
        Construye el reducer final.
        
        Returns:
            Función reducer
        """
        handlers = self.handlers
        default_handler = self.default_handler
        
        def reducer(state: Any, action: Action) -> Any:
            action_type = action.get_type()
            
            # Buscar handler específico
            if action_type in handlers:
                return handlers[action_type](state, action)
            
            # Usar handler por defecto
            if default_handler:
                return default_handler(state, action)
            
            # Sin cambios
            return state
        
        return reducer


def create_reducer(handlers: Dict[str, Reducer], default: Reducer | None = None) -> Reducer:
    """
    Función helper para crear reducers con un dict de handlers.
    
    Args:
        handlers: Dict de {action_type: handler_function}
        default: Handler por defecto opcional
        
    Returns:
        Función reducer
        
    Ejemplo:
        reducer = create_reducer({
            "INCREMENT": lambda state, action: state + 1,
            "DECREMENT": lambda state, action: state - 1,
        })
    """
    def reducer(state: Any, action: Action) -> Any:
        action_type = action.get_type()
        
        if action_type in handlers:
            return handlers[action_type](state, action)
        
        if default:
            return default(state, action)
        
        return state
    
    return reducer


def identity_reducer(state: Any, action: Action) -> Any:
    """
    Reducer que no hace nada (retorna el mismo estado).
    
    Útil como placeholder o para testing.
    """
    return state


if __name__ == "__main__":
    from .action import SimpleAction, PayloadAction
    
    # Ejemplo 1: Reducer simple
    def counter_reducer(state: int, action: Action) -> int:
        if action.get_type() == "INCREMENT":
            return state + 1
        elif action.get_type() == "DECREMENT":
            return state - 1
        elif action.get_type() == "SET":
            return action.payload if hasattr(action, 'payload') else state
        return state
    
    # Test
    state = 0
    state = counter_reducer(state, SimpleAction("INCREMENT"))
    print(f"After INCREMENT: {state}")  # 1
    
    state = counter_reducer(state, SimpleAction("INCREMENT"))
    print(f"After INCREMENT: {state}")  # 2
    
    state = counter_reducer(state, SimpleAction("DECREMENT"))
    print(f"After DECREMENT: {state}")  # 1
    
    state = counter_reducer(state, PayloadAction("SET", payload=10))
    print(f"After SET(10): {state}")  # 10
    
    # Ejemplo 2: ReducerBuilder
    print("\n--- ReducerBuilder ---")
    
    builder_reducer = (
        ReducerBuilder(initial_state=0)
        .case("INCREMENT", lambda state, action: state + 1)
        .case("DECREMENT", lambda state, action: state - 1)
        .case("MULTIPLY", lambda state, action: state * action.payload)
        .build()
    )
    
    state = 5
    state = builder_reducer(state, SimpleAction("INCREMENT"))
    print(f"After INCREMENT: {state}")  # 6
    
    state = builder_reducer(state, PayloadAction("MULTIPLY", payload=3))
    print(f"After MULTIPLY(3): {state}")  # 18
    
    # Ejemplo 3: CombinedReducer
    print("\n--- CombinedReducer ---")
    
    def todos_reducer(state: list, action: Action) -> list:
        if state is None:
            state = []
        
        if action.get_type() == "ADD_TODO":
            return [*state, action.payload]
        elif action.get_type() == "CLEAR_TODOS":
            return []
        return state
    
    combined = CombinedReducer({
        "counter": counter_reducer,
        "todos": todos_reducer
    })
    
    app_state = {"counter": 0, "todos": []}
    
    app_state = combined.reduce(app_state, SimpleAction("INCREMENT"))
    print(f"After INCREMENT: {app_state}")
    
    app_state = combined.reduce(app_state, PayloadAction("ADD_TODO", payload="Learn Vela"))
    print(f"After ADD_TODO: {app_state}")
    
    app_state = combined.reduce(app_state, PayloadAction("ADD_TODO", payload="Build app"))
    print(f"After ADD_TODO: {app_state}")
