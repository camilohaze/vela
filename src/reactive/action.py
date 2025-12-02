"""
Action types for State Management System

Implementación de: VELA-577 (TASK-035S)
Sprint: Sprint 15
Fecha: 2025-12-02

Descripción:
Este módulo define los tipos base para Actions en el sistema de State Management.
Las Actions representan intenciones de cambiar el estado de la aplicación.
"""

from typing import Any, Dict, TypeVar, Generic
from dataclasses import dataclass
from abc import ABC, abstractmethod


T = TypeVar('T')


class Action(ABC):
    """
    Clase base abstracta para todas las acciones del sistema.
    
    Las acciones son objetos inmutables que representan una intención
    de cambiar el estado de la aplicación.
    
    Inspirado en:
    - Redux: Actions con type y payload
    - NgRx: Type-safe actions con discriminated unions
    
    Ejemplo:
        class IncrementAction(Action):
            type = "INCREMENT"
            
        class AddTodoAction(Action):
            type = "ADD_TODO"
            
            def __init__(self, text: str):
                self.text = text
    """
    
    type: str  # Tipo de acción (obligatorio)
    
    @abstractmethod
    def get_type(self) -> str:
        """Retorna el tipo de la acción."""
        pass
    
    def to_dict(self) -> Dict[str, Any]:
        """
        Convierte la acción a un diccionario (para logging/DevTools).
        
        Returns:
            Dict con type y todos los atributos de la acción
        """
        result = {"type": self.get_type()}
        for key, value in self.__dict__.items():
            if key != "type":
                result[key] = value
        return result
    
    def __repr__(self) -> str:
        attrs = ", ".join(f"{k}={v!r}" for k, v in self.__dict__.items() if k != "type")
        if attrs:
            return f"{self.__class__.__name__}({attrs})"
        return f"{self.__class__.__name__}()"


@dataclass(frozen=True)
class SimpleAction(Action):
    """
    Action simple sin payload.
    
    Ejemplo:
        increment = SimpleAction("INCREMENT")
        clear = SimpleAction("CLEAR")
    """
    
    type: str
    
    def get_type(self) -> str:
        return self.type


@dataclass(frozen=True)
class PayloadAction(Action, Generic[T]):
    """
    Action con payload tipado.
    
    Ejemplo:
        add_todo = PayloadAction("ADD_TODO", payload="Learn Vela")
        set_count = PayloadAction("SET_COUNT", payload=42)
    """
    
    type: str
    payload: T
    
    def get_type(self) -> str:
        return self.type


class ActionCreator:
    """
    Factory para crear acciones de forma type-safe.
    
    Ejemplo:
        # Definir action creators
        increment = ActionCreator.simple("INCREMENT")
        add_todo = ActionCreator.payload("ADD_TODO")
        
        # Usar
        action1 = increment()
        action2 = add_todo("Learn Vela")
    """
    
    @staticmethod
    def simple(action_type: str):
        """
        Crea un action creator para acciones sin payload.
        
        Args:
            action_type: Tipo de la acción
            
        Returns:
            Función que crea la acción
        """
        def creator() -> SimpleAction:
            return SimpleAction(action_type)
        return creator
    
    @staticmethod
    def payload(action_type: str):
        """
        Crea un action creator para acciones con payload.
        
        Args:
            action_type: Tipo de la acción
            
        Returns:
            Función que crea la acción con el payload dado
        """
        def creator(payload: Any) -> PayloadAction:
            return PayloadAction(action_type, payload)
        return creator


# Action types predefinidos (opcionales, para conveniencia)
class InitAction(SimpleAction):
    """Acción especial disparada al inicializar el store."""
    def __init__(self):
        super().__init__("@@INIT")


class ResetAction(SimpleAction):
    """Acción para resetear el store al estado inicial."""
    def __init__(self):
        super().__init__("@@RESET")


if __name__ == "__main__":
    # Ejemplos de uso
    
    # 1. Action simple
    increment = SimpleAction("INCREMENT")
    print(f"Action simple: {increment}")
    print(f"Dict: {increment.to_dict()}")
    
    # 2. Action con payload
    add_todo = PayloadAction("ADD_TODO", payload="Learn Vela")
    print(f"\nAction con payload: {add_todo}")
    print(f"Dict: {add_todo.to_dict()}")
    
    # 3. Action creators
    increment_creator = ActionCreator.simple("INCREMENT")
    add_todo_creator = ActionCreator.payload("ADD_TODO")
    
    action1 = increment_creator()
    action2 = add_todo_creator("Build awesome apps")
    
    print(f"\nUsando creators:")
    print(f"Action 1: {action1}")
    print(f"Action 2: {action2}")
    
    # 4. Actions especiales
    init = InitAction()
    reset = ResetAction()
    
    print(f"\nActions especiales:")
    print(f"Init: {init}")
    print(f"Reset: {reset}")
