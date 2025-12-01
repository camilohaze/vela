"""
Symbol Table con Scopes Anidados

Implementación de: TASK-016K
Historia: VELA-572
Fecha: 2025-01-22

Descripción:
Estructura completa de symbol table con scopes anidados, manejo de shadowing,
visibility por scopes. Soporta múltiples tipos de scopes: global, module, 
function, block, class.
"""

from enum import Enum, auto
from typing import Optional, Dict, List, Any
from dataclasses import dataclass, field


class SymbolKind(Enum):
    """Tipo de símbolo en la tabla."""
    VARIABLE = auto()        # Variables (inmutables por defecto)
    STATE = auto()           # Variables mutables reactivas (state)
    FUNCTION = auto()        # Funciones (fn)
    CLASS = auto()           # Clases
    MODULE = auto()          # Módulos (@module)
    SERVICE = auto()         # Services (@service)
    REPOSITORY = auto()      # Repositories (@repository)
    CONTROLLER = auto()      # Controllers (@controller)
    WIDGET = auto()          # Widgets (widget/component)
    PARAMETER = auto()       # Parámetros de función
    TYPE_ALIAS = auto()      # Type aliases (type)
    ENUM = auto()            # Enumeraciones
    STRUCT = auto()          # Structs
    INTERFACE = auto()       # Interfaces


class ScopeType(Enum):
    """Tipo de scope en la symbol table."""
    GLOBAL = auto()          # Scope global del programa
    MODULE = auto()          # Scope de un módulo
    FUNCTION = auto()        # Scope de una función
    BLOCK = auto()           # Scope de un bloque (if, match, etc.)
    CLASS = auto()           # Scope de una clase
    LOOP = auto()            # Scope de un loop (métodos funcionales)


@dataclass
class Symbol:
    """
    Representa un símbolo en la tabla.
    
    Un símbolo contiene toda la información sobre una declaración:
    nombre, tipo, scope donde fue definido, nodo AST de declaración, etc.
    """
    name: str                           # Nombre del símbolo
    kind: SymbolKind                    # Tipo de símbolo
    scope_level: int                    # Nivel de scope (0 = global)
    declaration_node: Optional[Any] = None  # Nodo AST de declaración
    type_annotation: Optional[str] = None   # Tipo anotado (si existe)
    is_public: bool = False             # Modificador public
    is_mutable: bool = False            # Es mutable (state)
    metadata: Dict[str, Any] = field(default_factory=dict)  # Metadata adicional


@dataclass
class Scope:
    """
    Representa un scope (ámbito) en la symbol table.
    
    Un scope contiene símbolos definidos en ese ámbito y puede tener
    un scope padre (parent) para scopes anidados.
    """
    scope_type: ScopeType               # Tipo de scope
    level: int                          # Nivel de anidamiento (0 = global)
    symbols: Dict[str, Symbol] = field(default_factory=dict)  # Símbolos en este scope
    parent: Optional['Scope'] = None    # Scope padre (si existe)
    
    def lookup_local(self, name: str) -> Optional[Symbol]:
        """
        Busca un símbolo SOLO en este scope (no en padres).
        
        Args:
            name: Nombre del símbolo
            
        Returns:
            Symbol si existe, None si no
        """
        return self.symbols.get(name)
    
    def lookup(self, name: str) -> Optional[Symbol]:
        """
        Busca un símbolo en este scope y en padres (lookup completo).
        
        Args:
            name: Nombre del símbolo
            
        Returns:
            Symbol si existe en este scope o padres, None si no
        """
        # Buscar primero en scope actual
        symbol = self.symbols.get(name)
        if symbol:
            return symbol
        
        # Si no se encuentra, buscar en scope padre
        if self.parent:
            return self.parent.lookup(name)
        
        return None
    
    def define(self, symbol: Symbol) -> bool:
        """
        Define un símbolo en este scope.
        
        Args:
            symbol: Símbolo a definir
            
        Returns:
            True si se definió correctamente, False si ya existía
        """
        if symbol.name in self.symbols:
            return False  # Ya existe un símbolo con ese nombre
        
        self.symbols[symbol.name] = symbol
        return True


class SymbolTable:
    """
    Tabla de símbolos con soporte para scopes anidados.
    
    La symbol table mantiene un stack de scopes para manejar
    scopes anidados correctamente (funciones dentro de funciones,
    bloques dentro de bloques, etc.).
    
    Funcionalidades:
    - Define símbolos en el scope actual
    - Lookup de símbolos con búsqueda en scopes padres
    - Push/pop de scopes para entrar/salir de ámbitos
    - Manejo de shadowing (variables con mismo nombre en scopes diferentes)
    - Detección de duplicados en el mismo scope
    """
    
    def __init__(self):
        """Inicializa la symbol table con scope global."""
        # Scope global (nivel 0)
        self.global_scope = Scope(scope_type=ScopeType.GLOBAL, level=0)
        
        # Stack de scopes (empieza con global)
        self.scope_stack: List[Scope] = [self.global_scope]
        
        # Scope actual (top del stack)
        self.current_scope = self.global_scope
        
        # Contador de niveles de scope
        self.scope_level = 0
    
    def push_scope(self, scope_type: ScopeType) -> None:
        """
        Crea un nuevo scope y lo agrega al stack.
        
        Args:
            scope_type: Tipo de scope a crear
        """
        self.scope_level += 1
        new_scope = Scope(
            scope_type=scope_type,
            level=self.scope_level,
            parent=self.current_scope
        )
        self.scope_stack.append(new_scope)
        self.current_scope = new_scope
    
    def pop_scope(self) -> Optional[Scope]:
        """
        Elimina el scope actual del stack y retorna al padre.
        
        Returns:
            Scope eliminado, o None si ya estamos en global
        """
        if len(self.scope_stack) <= 1:
            # No podemos eliminar el scope global
            return None
        
        popped = self.scope_stack.pop()
        self.current_scope = self.scope_stack[-1]
        self.scope_level -= 1
        return popped
    
    def define(self, symbol: Symbol) -> bool:
        """
        Define un símbolo en el scope actual.
        
        Args:
            symbol: Símbolo a definir
            
        Returns:
            True si se definió correctamente, False si ya existe en este scope
        """
        # Actualizar scope_level del símbolo
        symbol.scope_level = self.scope_level
        
        # Definir en el scope actual
        return self.current_scope.define(symbol)
    
    def lookup(self, name: str) -> Optional[Symbol]:
        """
        Busca un símbolo por nombre en el scope actual y padres.
        
        Args:
            name: Nombre del símbolo
            
        Returns:
            Symbol si existe, None si no se encuentra
        """
        return self.current_scope.lookup(name)
    
    def lookup_local(self, name: str) -> Optional[Symbol]:
        """
        Busca un símbolo SOLO en el scope actual (no en padres).
        
        Args:
            name: Nombre del símbolo
            
        Returns:
            Symbol si existe en scope actual, None si no
        """
        return self.current_scope.lookup_local(name)
    
    def is_shadowing(self, name: str) -> bool:
        """
        Verifica si un símbolo está haciendo shadowing de otro.
        
        Shadowing ocurre cuando se define un símbolo con el mismo
        nombre en un scope anidado, ocultando el símbolo del scope padre.
        
        Args:
            name: Nombre del símbolo
            
        Returns:
            True si existe un símbolo con ese nombre en un scope padre
        """
        # Buscar solo en el scope actual
        if self.current_scope.lookup_local(name):
            return False  # Ya existe en este scope
        
        # Buscar en el scope padre (si existe)
        if self.current_scope.parent:
            return self.current_scope.parent.lookup(name) is not None
        
        return False
    
    def get_scope_type(self) -> ScopeType:
        """
        Obtiene el tipo del scope actual.
        
        Returns:
            Tipo del scope actual
        """
        return self.current_scope.scope_type
    
    def get_scope_level(self) -> int:
        """
        Obtiene el nivel del scope actual.
        
        Returns:
            Nivel de anidamiento (0 = global)
        """
        return self.scope_level
    
    def get_all_symbols(self) -> Dict[str, Symbol]:
        """
        Obtiene todos los símbolos visibles desde el scope actual.
        
        Incluye símbolos del scope actual y todos los padres,
        respetando shadowing (símbolos más cercanos ocultan a los lejanos).
        
        Returns:
            Diccionario con todos los símbolos visibles
        """
        symbols: Dict[str, Symbol] = {}
        
        # Recorrer desde el scope más externo al más interno
        # (para que los símbolos internos sobrescriban los externos)
        for scope in self.scope_stack:
            symbols.update(scope.symbols)
        
        return symbols
    
    def __repr__(self) -> str:
        """Representación string de la symbol table."""
        return f"SymbolTable(level={self.scope_level}, scope_type={self.current_scope.scope_type})"


if __name__ == "__main__":
    # Ejemplo de uso
    table = SymbolTable()
    
    # Definir símbolo global
    global_var = Symbol(
        name="PI",
        kind=SymbolKind.VARIABLE,
        scope_level=0,
        type_annotation="Float"
    )
    table.define(global_var)
    
    # Entrar a scope de función
    table.push_scope(ScopeType.FUNCTION)
    
    # Definir parámetro
    param = Symbol(
        name="x",
        kind=SymbolKind.PARAMETER,
        scope_level=1,
        type_annotation="Number"
    )
    table.define(param)
    
    # Lookup de PI (debe encontrarse en scope padre)
    found = table.lookup("PI")
    print(f"Found PI: {found}")  # Symbol(name='PI', ...)
    
    # Lookup de x (existe en scope actual)
    found_x = table.lookup("x")
    print(f"Found x: {found_x}")  # Symbol(name='x', ...)
    
    # Salir del scope de función
    table.pop_scope()
    
    # Lookup de x fuera de función (no debe encontrarse)
    not_found = table.lookup("x")
    print(f"x outside function: {not_found}")  # None
    
    print(f"All symbols: {table.get_all_symbols()}")
