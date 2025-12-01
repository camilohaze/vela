"""
Type Environment - Entorno de Tipos

Implementación de: VELA-570 (TASK-021)
Sprint: Sprint 8
Fecha: 2025-12-01

Descripción:
Maneja el entorno de tipos (symbol table) con scopes anidados.
"""

from typing import Dict, Optional, List
from dataclasses import dataclass, field
from .types import Type, TypeVariable


@dataclass
class Symbol:
    """
    Símbolo en el entorno de tipos.
    
    Representa una variable, función, clase, etc. con su tipo.
    """
    name: str
    type: Type
    mutable: bool = False  # True si es `state`, False por defecto
    is_function: bool = False
    is_class: bool = False
    is_type: bool = False  # True para type aliases
    
    def __str__(self) -> str:
        mut_str = "mut " if self.mutable else ""
        return f"{mut_str}{self.name}: {self.type}"


@dataclass
class TypeEnvironment:
    """
    Entorno de tipos con scopes anidados.
    
    Mantiene un stack de scopes para name resolution.
    """
    scopes: List[Dict[str, Symbol]] = field(default_factory=lambda: [{}])
    
    def enter_scope(self):
        """Entra a un nuevo scope anidado"""
        self.scopes.append({})
    
    def exit_scope(self):
        """Sale del scope actual"""
        if len(self.scopes) > 1:
            self.scopes.pop()
        else:
            raise RuntimeError("Cannot exit global scope")
    
    def define(self, name: str, symbol: Symbol):
        """
        Define un símbolo en el scope actual.
        
        Raises:
            NameError: Si el símbolo ya existe en el scope actual
        """
        current_scope = self.scopes[-1]
        if name in current_scope:
            raise NameError(f"Symbol '{name}' already defined in current scope")
        current_scope[name] = symbol
    
    def lookup(self, name: str) -> Optional[Symbol]:
        """
        Busca un símbolo en los scopes (de más interno a más externo).
        
        Returns:
            Symbol si se encuentra, None si no existe
        """
        # Buscar desde el scope más interno hacia afuera
        for scope in reversed(self.scopes):
            if name in scope:
                return scope[name]
        return None
    
    def update(self, name: str, new_type: Type):
        """
        Actualiza el tipo de un símbolo existente.
        
        Útil durante type narrowing.
        """
        for scope in reversed(self.scopes):
            if name in scope:
                scope[name].type = new_type
                return
        raise NameError(f"Symbol '{name}' not found")
    
    def is_defined(self, name: str) -> bool:
        """Verifica si un símbolo está definido"""
        return self.lookup(name) is not None
    
    def is_mutable(self, name: str) -> bool:
        """Verifica si una variable es mutable (state)"""
        symbol = self.lookup(name)
        return symbol is not None and symbol.mutable
    
    def get_all_symbols(self) -> List[Symbol]:
        """Retorna todos los símbolos en todos los scopes"""
        all_symbols = []
        for scope in self.scopes:
            all_symbols.extend(scope.values())
        return all_symbols
    
    def __str__(self) -> str:
        result = "TypeEnvironment:\n"
        for i, scope in enumerate(self.scopes):
            result += f"  Scope {i}: {{\n"
            for name, symbol in scope.items():
                result += f"    {symbol}\n"
            result += "  }\n"
        return result


def test_environment():
    """Tests del entorno de tipos"""
    print("=== Tests de Type Environment ===\n")
    
    from .types import NUMBER_TYPE, STRING_TYPE
    
    env = TypeEnvironment()
    
    # Test 1: Definir en scope global
    env.define("x", Symbol("x", NUMBER_TYPE))
    print(f"Test 1: Definir x: Number")
    assert env.is_defined("x")
    assert env.lookup("x").type == NUMBER_TYPE
    print("  ✅ Definición correcta")
    
    # Test 2: Enter scope y definir variable local
    env.enter_scope()
    env.define("y", Symbol("y", STRING_TYPE))
    print(f"\nTest 2: Nuevo scope con y: String")
    assert env.is_defined("y")
    assert env.is_defined("x")  # x aún visible desde scope padre
    print("  ✅ Variable local visible")
    print("  ✅ Variable padre visible")
    
    # Test 3: Shadowing
    env.define("x", Symbol("x", STRING_TYPE))  # Shadow x
    print(f"\nTest 3: Shadow x con String")
    assert env.lookup("x").type == STRING_TYPE  # Nueva x
    print("  ✅ Shadowing correcto")
    
    # Test 4: Exit scope
    env.exit_scope()
    print(f"\nTest 4: Exit scope")
    assert env.lookup("x").type == NUMBER_TYPE  # x original
    assert not env.is_defined("y")  # y ya no existe
    print("  ✅ Scope restaurado")
    
    # Test 5: Mutabilidad
    env.define("count", Symbol("count", NUMBER_TYPE, mutable=True))
    print(f"\nTest 5: Variable mutable (state count)")
    assert env.is_mutable("count")
    assert not env.is_mutable("x")
    print("  ✅ Mutabilidad correcta")
    
    print(f"\n{env}")
    print("✅ Todos los tests de environment pasaron")


if __name__ == "__main__":
    test_environment()
